use std::collections::HashMap;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use kinode_process_lib::{
    Address, NodeId, Message, ProcessId, Request, Response, await_message, call_init, http, get_blob, clear_state, println,
    http::{bind_http_path, bind_ws_path, send_response, send_ws_push, serve_ui},
};

mod structs;
use structs::{LeaderboardEntry, State, ContactRequest, ContactRequestBody, ChatMessage, ChatMessageBody, ChatRequest};

wit_bindgen::generate!({
    path: "wit",
    world: "process",
});

call_init!(init);
fn init(our: Address) {
    println!("{our} started!");

    clear_state();
    let mut state = State::fetch(our.node().to_string());

    serve_ui(&our, "ui", true, true, vec!["/"]).unwrap();

    bind_http_path("/get_leaderboard", true, false).unwrap();
    bind_http_path("/add_respect", true, false).unwrap();
    bind_http_path("/set_discoverable", true, false).unwrap();
    bind_http_path("/remove_leaderboard_entry", true, false).unwrap();
    bind_http_path("/send_contact_request", true, false).unwrap();
    bind_http_path("/accept_contact", true, false).unwrap();
    bind_http_path("/decline_contact", true, false).unwrap();
    bind_http_path("/send_chat_message", true, false).unwrap();

    // Bind WebSocket path
    bind_ws_path("/", true, false).unwrap();

    kinode_process_lib::timer::set_timer(10_000, None);

    while let Ok(message) = await_message() {
        println!("our state: {:?}", state);
        handle_message(&our, &mut state, message);
        state.save();
    }
}

// handle local and alien messages
fn handle_message(our: &Address, state: &mut State, message: Message) {
    if message.source().node == our.node {
        let pid_str =  message.source().process.to_string();
        match pid_str.as_str() {  
            "timer:distro:sys" => handle_timer_events(our, state),
            "http_server:distro:sys" => handle_http_request(our, state, &message),
            _ => println!("other process than the shrine"),
        }
    } else if state.discoverable || state.pending_contact_requests.contains(&message.source().node) || state.contacts.contains(&message.source().node){ 
        println!("Incoming alien message");
        handle_alien_message(our, state, &message);
    }
}

// the timing needs to be more sophisicated 
fn handle_timer_events(our: &Address, state: &mut State) {
    //println!("timer update.");
    push_update_to_your_contacts(our, state);
    if !state.pending_contact_requests.is_empty() {
        resend_pending_requests(state);
    }
    kinode_process_lib::timer::set_timer(30_000, None);
}

// unneccessary
fn handle_http_request(our: &Address, state: &mut State, message: &Message) {
    if let Message::Request { ref body, .. } = message {
        if let Some(response) = process_http_request(our, body, state) {
            send_http_response(response);
        }
    }
}

fn process_http_request(
    our: &Address,
    body: &[u8],
    state: &mut State
) -> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
    let server_request = http::HttpServerRequest::from_bytes(body).ok()?;
    let http_request = server_request.request()?;
    let bound_path = http_request.bound_path(Some(&our.process())).rsplit('/').next().unwrap_or("");

    match http_request.method().ok()? {
        http::Method::GET => handle_get_request(bound_path, state),
        http::Method::POST => handle_post_request(bound_path, state, &http_request),
        _ => None,
    }
}

fn handle_get_request(bound_path: &str, state: &State) -> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
    match bound_path {
        "get_leaderboard" => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            let body = serde_json::to_vec(state).ok()?;
            Some((http::StatusCode::OK, headers, body))
        },
        _ => None,
    }
}

// I should get my return types in order
fn handle_post_request(bound_path: &str, state: &mut State, http_request: &http::IncomingHttpRequest) 
-> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
    match bound_path {
        "add_respect" => {
            state.add_respect();
            Some((http::StatusCode::OK, HashMap::new(), Vec::new()))
        },
        "send_contact_request" => handle_send_contact_request(state, http_request),
        "set_discoverable" => {
            state.set_discoverable(!state.discoverable);
            Some((http::StatusCode::OK, HashMap::new(), Vec::new()))
        },
        "accept_contact" => handle_accept_contact(state, http_request),
        "decline_contact" => handle_decline_contact(state, http_request),
        "send_chat_message" => handle_send_chat_message(state, http_request),
        _ => None,
    }
}

fn handle_send_contact_request(state: &mut State, http_request: &http::IncomingHttpRequest) 
-> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
    let body = get_blob()?;
    let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    match serde_json::from_str::<ContactRequestBody>(body_str) {
        Ok(parsed_body) => {
            let their_addy = Address {
                node: parsed_body.node.clone(),
                process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok()?,
            };
            Request::new()
                .body(serde_json::to_vec(&ContactRequest::RequestContact(parsed_body.node.clone())).ok()?)
                .target(&their_addy)
                .send().ok()?;
            state.append_outgoing_contact_request(parsed_body.node);
            Some((http::StatusCode::OK, headers, Vec::new()))
        },
        Err(e) => {
            println!("Failed to parse the contact request body: {:?}", e);
            Some((http::StatusCode::BAD_REQUEST, headers, Vec::new()))
        }
    }
}

fn handle_accept_contact(state: &mut State, http_request: &http::IncomingHttpRequest) 
-> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
    let body = get_blob()?;
    let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    match serde_json::from_str::<ContactRequestBody>(body_str) {
        Ok(parsed_body) => {
            let their_node = parsed_body.node.clone();
            state.add_contact(their_node.clone());
            state.incoming_contact_requests.retain(|incoming| *incoming != their_node);
            let their_addy = Address {
                node: their_node.clone(),
                process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok()?,
            };
            Request::new()
                .body(serde_json::to_vec(&ContactRequest::ContactAccepted(their_node.clone())).ok()?)
                .target(&their_addy)
                .send().ok()?;
            println!("sent contact accepted to {:?}", &their_node.to_string());
            Some((http::StatusCode::OK, headers, Vec::new()))
        },
        Err(e) => {
            println!("failed to parse the local contact request {e:?}");
            Some((http::StatusCode::BAD_REQUEST, headers, Vec::new()))
        }
    }
}

fn handle_decline_contact(state: &mut State, http_request: &http::IncomingHttpRequest) 
-> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
    let body = get_blob()?;
    let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    match serde_json::from_str::<ContactRequestBody>(body_str) {
        Ok(parsed_body) => {
            let their_node = parsed_body.node.clone();
            state.decline_contact(their_node.clone());
            Some((http::StatusCode::OK, headers, Vec::new()))
        },
        Err(e) => {
            println!("failed to parse the local contact request {e:?}");
            Some((http::StatusCode::BAD_REQUEST, headers, Vec::new()))
        }
    }
}

fn handle_send_chat_message(
    state: &mut State, 
    http_request: &http::IncomingHttpRequest
) -> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
    let body = get_blob()?;
    let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
    //println!("body_str: {:?}", body_str);
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    match serde_json::from_str::<ChatMessageBody>(body_str) {
        Ok(parsed_body) => {
            let chat_message = ChatMessage {
                sender: state.node_id.clone(),
                content: parsed_body.content.clone(),
                timestamp: std::time::SystemTime::now(), 
            };
            state.add_chat_message(chat_message.clone());

            let chat_message = ChatRequest::ChatMessageReceived(chat_message.clone());
            // us.send(message) -> their handler, which should somehow be picked up by the websocket match statement.
            match serde_json::to_vec(&chat_message) {
                Ok(serialized_message) => {
                    for contact in &state.contacts {
                        let their_addy = Address {
                            node: contact.clone(),
                            process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok().unwrap(),
                        };
                        Request::new()
                            .body(serialized_message.clone())
                            .target(&their_addy)
                            .send().ok().unwrap();
                    }
                }
                Err(_e) => println!("Failed to serialize chat message: {:?}", chat_message)
            }
            Some((http::StatusCode::OK, headers, Vec::new()))
        },
        Err(e) => {
            println!("(LOCAL) failed to parse chat message from front-end. Error: {:?}", e);
            Some((http::StatusCode::BAD_REQUEST, headers, Vec::new()))
        }
    }
}

fn send_http_response(response: (http::StatusCode, HashMap<String, String>, Vec<u8>)) {
    let (status, headers, body) = response;
    http::send_response(status, Some(headers), body);
    println!("Response sent: {:?}", status);
}

fn handle_alien_message(our: &Address, state: &mut State, message: &Message) {
    if let Ok(alien_request) = serde_json::from_slice::<ContactRequest>(message.body()) {
        println!("alien request in handling");
        let their_node = &message.source().node;
        match alien_request {
            ContactRequest::RequestContact(_) => {
                // append the incoming node_id to the incoming_contact_requests
                if !state.contacts.contains(&their_node) && !state.incoming_contact_requests.contains(&their_node) && their_node != &our.node{ // temp solution for now
                    state.incoming_contact_requests.push(their_node.clone());
                    println!("contact request from {:?}", &their_node);
                }
            },
            ContactRequest::ContactAccepted(_) => { 
                // pressing accept in the UI triggers that the sender receives this ACK from the originial receiver
                state.contacts.push(their_node.to_string());
                println!("{} accepted your request. You are now frens <3", &their_node);
            },
            ContactRequest::ContactUpdate(entry) => { 
                //if they're in our contacts, update their score
                if state.contacts.contains(&their_node) {
                    state.stats.insert(their_node.to_string(),entry);
                    println!("updated {:?}", &their_node);
                } else  {
                    println!("request from non-contact (delete this later)");
                }
            },
            _ => println!("contact request didn't match anything"),
        }
    } else if let Ok(inc_chat_message) = serde_json::from_slice::<ChatRequest>(message.body()) {
        println!("chat message request in handling");
        match inc_chat_message {
            ChatRequest::ChatMessageReceived(chat_message) => {
                println!("alien chat message = {:?}", chat_message);
                state.add_chat_message(chat_message); 
            }
            _ => println!("something else than a chat message")
        }
    }
}

// pushing your score to your contacts
fn push_update_to_your_contacts(our: &Address, state: &State) {
    let our_respects = state.stats.get(&state.node_id).unwrap_or(&LeaderboardEntry { respects: 0 });
    let our_respect_update = ContactRequest::ContactUpdate(our_respects.clone());

    for contact in &state.contacts {
        let their_addy = Address {
            node: contact.clone(),
            process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok().unwrap(),
        };
        Request::new()
            .body(serde_json::to_vec(&our_respect_update).ok().unwrap())
            .target(&their_addy)
            .send().ok().unwrap();
    }
}

// Resend pending contact requests
fn resend_pending_requests(state: &mut State) {
     //println!("resending contact requests");

     let mut nodes_to_remove = Vec::new();

     for node in &state.pending_contact_requests {
        if !state.contacts.contains(&node) {
            let their_addy = Address {
                node: node.clone(),
                process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok().unwrap(),  
            };
            let contact_request = ContactRequest::RequestContact(node.clone());
            match serde_json::to_vec(&contact_request) {
                Ok(body) => {
                    let request = Request::new()
                        .body(body)
                        .target(&their_addy);
                    
                    if let Err(e) = request.send() {
                        println!("Failed to resend contact request to {}: {:?}", node, e);
                    } else {
                        println!("Resent contact request to {}", node);
                    }
                }
                Err(e) => println!("Failed to serialize contact request for {}: {:?}", node, e),
            }
        } else {
            println!("{:?} already in your contacts, clearing it from the pending list", node);
            nodes_to_remove.push(node.clone());
        }
    }
    state.pending_contact_requests.retain(|pending| !nodes_to_remove.contains(pending));
}


//use std::collections::{HashMap, HashSet};
//use std::str::FromStr;
//use kinode_process_lib::{
//    Address, NodeId, Message, ProcessId, Request, Response, LazyLoadBlob, 
//    await_message, call_init, http, get_blob, clear_state, println,
//    http::{bind_http_path, bind_ws_path, send_response, send_ws_push, WsMessageType, serve_ui},
//};
//
//mod structs;
//use structs::{LeaderboardEntry, State, ContactRequest, ContactRequestBody, ChatMessage, ChatMessageBody, ChatRequest};
//
//wit_bindgen::generate!({
//    path: "wit",
//    world: "process",
//});
//
//call_init!(init);
//fn init(our: Address) {
//    println!("{our} started!");
//
//    clear_state();
//    let mut state = State::fetch(our.node().to_string());
//
//    //state.clients = HashSet::new(); // Not sure if I need this in the state
//
//    serve_ui(&our, "ui", true, true, vec!["/"]).unwrap();
//
//    bind_http_path("/get_leaderboard", true, false).unwrap();
//    bind_http_path("/get_chat", true, false).unwrap();
//    bind_http_path("/add_respect", true, false).unwrap();
//    bind_http_path("/set_discoverable", true, false).unwrap();
//    bind_http_path("/remove_leaderboard_entry", true, false).unwrap();
//    bind_http_path("/send_contact_request", true, false).unwrap();
//    bind_http_path("/accept_contact", true, false).unwrap();
//    bind_http_path("/decline_contact", true, false).unwrap();
//    bind_http_path("/send_chat_message", true, false).unwrap();
//
//    // Bind WebSocket path for allowing updates to be sent to the frontend
//    bind_ws_path("/", true, false).unwrap(); //wat is
//
//    kinode_process_lib::timer::set_timer(10_000, None);
//
//    while let Ok(message) = await_message() {
//        println!("our state: {:?}", state); //debug
//        handle_message(&our, &mut state, message);
//        state.save();
//    }
//}
//
//fn handle_message(our: &Address, state: &mut State, message: Message) {
//    if message.source().node == our.node {
//        let pid_str =  message.source().process.to_string();
//        match pid_str.as_str() {  
//            "timer:distro:sys" => handle_timer_events(our, state),
//            "http_server:distro:sys" => handle_http_server_request(our, state, &message), //WS chat update
//            _ => println!("other process than the shrine"),
//        }
//    } else if state.discoverable || state.pending_contact_requests.contains(&message.source().node) || state.contacts.contains(&message.source().node){ 
//        println!("Incoming alien message");
//        handle_alien_message(our, state, &message);
//    }
//}
//
//fn handle_http_server_request(our: &Address, state: &mut State, message: &Message) {
//    if let Message::Request { ref body, .. } = message {
//        let Ok(server_request) = serde_json::from_slice::<http::HttpServerRequest>(body) else {
//            return;
//        };
//
//        match server_request {
//            http::HttpServerRequest::WebSocketOpen { channel_id, .. } => {
//                *state.channel_id = channel_id
//            }
//            http::HttpServerRequest::WebSocketPush { .. } => {
//                let Some(blob) = get_blob() else {
//                    return;
//                };
//
//                handle_chat_request(
//                    our, 
//                    state, 
//                    &blob.bytes
//                );
//            }
//            http::HttpServerRequest::WebSocketClose(channel_id) => {
//                state.clients.remove(&channel_id);
//            }
//            http::HttpServerRequest::Http(ref incoming) => {
//                match process_http_request(our, incoming, state) {
//                    Ok(()) => (),
//                    Err(e) => {
//                        println!("error handling http request: {:?}", e);
//                        send_response(
//                            http::StatusCode::SERVICE_UNAVAILABLE,
//                            None,
//                            "Service Unavailable".to_string().as_bytes().to_vec(),
//                        );
//                    }
//                }
//            }
//        }
//    }
//}
//
//fn handle_chat_request(our: &Address, source: &Address, state: &mut State, body: &[u8]) -> anyhow::Result<()> {
//    let chat_request = serde_json::from_slice::<ChatRequest>(body)?;
//
//    match chat_request {
//        ChatRequest::Send {
//            ref target,
//            ref content,
//        } => {
//            let (counterparty, sender) = if target == &our.node {
//                (&source.node, source.node.clone())
//            } else {
//                (target, our.node.clone())
//            };
//
//            if target == &our.node {
//                Request::new()
//                    .target(address {
//                        node: target.clone(),
//                        process: processid::from_str("updated_shrine:td_shrine:sharmouta.os")?,
//                    })
//                    .body(body)
//                    .send_and_await_response(5)??;
//            }
//
//            let messages = match message_archive {
//                Some(messages) => messages,
//                None => {
//                    message_archive.insert(Vec::new());
//                }
//            };
//
//            let new_message = ChatMessage {
//                sender: sender.clone(),
//                content : content.clone(),
//            };
//
//            let blob = LazyLoadBlob {
//                mime: Some("application/json".to_string()),
//                bytes: serde_json::to_vec(&ChatRequest::ChatMessageReceived(chat_message.clone()))?,
//            };
//
//            for &channel_id in &state.clients {
//                send_ws_push(
//                    channel_id, 
//                    WsMessageType::Text, 
//                    blob.clone()
//                );
//            }
//        }
//    }
//    Ok(())
//}
//
//fn handle_timer_events(our: &Address, state: &mut State) {
//    push_update_to_your_contacts(state);
//    if !state.pending_contact_requests.is_empty() {
//        resend_pending_requests(state);
//    }
//    kinode_process_lib::timer::set_timer(30_000, None);
//}
//
//fn process_http_request(
//    our: &Address,
//    http_request: &http::IncomingHttpRequest,
//    state: &mut State
//) -> anyhow::Result<()> {
//    let bound_path = http_request.bound_path(Some(&our.process())).rsplit('/').next().unwrap_or("");
//
//    match http_request.method()?.as_str() {
//        "GET" => {
//            if let Some(response) = handle_get_request(bound_path, state) {
//                //send_http_response(response);
//                let (status, headers, body) = response;
//                http::send_response(status, Some(headers), body);
//            }
//        }
//        "POST" => {
//            if let Some(response) = handle_post_request(bound_path, state, http_request) {
//                //send_http_response(response);
//                let (status, headers, body) = response;
//                http::send_response(status, Some(headers), body);
//            }
//        }
//        _ => {}
//    }
//    Ok(())
//}
//
//fn handle_get_request(bound_path: &str, state: &State) -> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
//    match bound_path {
//        "get_leaderboard" => {
//            let mut headers = HashMap::new();
//            headers.insert("Content-Type".to_string(), "application/json".to_string());
//            let body = serde_json::to_vec(state).ok()?;
//            Some((http::StatusCode::OK, headers, body))
//        },
//        "get_chat" => {
//            let mut headers = HashMap::new();
//            headers.insert("Content-Type".to_string(), "application/json".to_string());
//            println!("get_chat request gets this message: {:?}", &state.chat_messages);
//            let body = serde_json::to_vec(&state.chat_messages).ok()?;
//            Some((http::StatusCode::OK, headers, body))
//        },
//        _ => None,
//    }
//}
//
//fn handle_post_request(bound_path: &str, state: &mut State, http_request: &http::IncomingHttpRequest) 
//-> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
//    match bound_path {
//        "add_respect" => { 
//            state.add_respect();
//            Some((http::StatusCode::OK, HashMap::new(), Vec::new()))
//        },
//        "send_contact_request" => handle_send_contact_request(state, http_request),
//        "set_discoverable" => {
//            state.set_discoverable(!state.discoverable);
//            Some((http::StatusCode::OK, HashMap::new(), Vec::new()))
//        },
//        "accept_contact" => handle_accept_contact(state, http_request),
//        "decline_contact" => handle_decline_contact(state, http_request),
//        "send_chat_message" => handle_send_chat_message(state, http_request),
//        _ => None,
//    }
//}
//
//fn handle_send_contact_request(state: &mut State, _http_request: &http::IncomingHttpRequest) 
//-> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
//    let body = get_blob()?;
//    let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
//    let mut headers = HashMap::new();
//    headers.insert("Content-Type".to_string(), "application/json".to_string());
//
//    match serde_json::from_str::<ContactRequestBody>(body_str) {
//        Ok(parsed_body) => {
//            let their_addy = Address {
//                node: parsed_body.node.clone(),
//                process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok()?,
//            };
//            Request::new()
//                .body(serde_json::to_vec(&ContactRequest::RequestContact(parsed_body.node.clone())).ok()?)
//                .target(&their_addy)
//                .send().ok()?;
//            state.append_outgoing_contact_request(parsed_body.node);
//            Some((http::StatusCode::OK, headers, Vec::new()))
//        },
//        Err(e) => {
//            println!("Failed to parse the contact request body: {:?}", e);
//            Some((http::StatusCode::BAD_REQUEST, headers, Vec::new()))
//        }
//    }
//}
//
//fn handle_accept_contact(state: &mut State, _http_request: &http::IncomingHttpRequest) 
//-> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
//    let body = get_blob()?;
//    let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
//    let mut headers = HashMap::new();
//    headers.insert("Content-Type".to_string(), "application/json".to_string());
//
//    match serde_json::from_str::<ContactRequestBody>(body_str) {
//        Ok(parsed_body) => {
//            let their_node = parsed_body.node.clone();
//            state.add_contact(their_node.clone());
//            state.incoming_contact_requests.retain(|incoming| *incoming != their_node);
//            let their_addy = Address {
//                node: their_node.clone(),
//                process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok()?,
//            };
//            Request::new()
//                .body(serde_json::to_vec(&ContactRequest::ContactAccepted(their_node.clone())).ok()?)
//                .target(&their_addy)
//                .send().ok()?;
//            println!("sent contact accepted to {:?}", &their_node.to_string());
//            Some((http::StatusCode::OK, headers, Vec::new()))
//        },
//        Err(e) => {
//            println!("failed to parse the local contact request {e:?}");
//            Some((http::StatusCode::BAD_REQUEST, headers, Vec::new()))
//        }
//    }
//}
//
//fn handle_decline_contact(state: &mut State, _http_request: &http::IncomingHttpRequest) 
//-> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
//    let body = get_blob()?;
//    let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
//    let mut headers = HashMap::new();
//    headers.insert("Content-Type".to_string(), "application/json".to_string());
//
//    match serde_json::from_str::<ContactRequestBody>(body_str) {
//        Ok(parsed_body) => {
//            let their_node = parsed_body.node.clone();
//            state.decline_contact(their_node.clone());
//            Some((http::StatusCode::OK, headers, Vec::new()))
//        },
//        Err(e) => {
//            println!("failed to parse the local contact request {e:?}");
//            Some((http::StatusCode::BAD_REQUEST, headers, Vec::new()))
//        }
//    }
//}
//
//fn handle_send_chat_message(state: &mut State, _http_request: &http::IncomingHttpRequest) 
//-> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
//    let body = get_blob()?;
//    let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
//    let mut headers = HashMap::new();
//    headers.insert("Content-Type".to_string(), "application/json".to_string());
//
//    match serde_json::from_str::<ChatMessageBody>(body_str) {
//        Ok(parsed_body) => {
//            let chat_message = ChatMessage {
//                sender: state.node_id.clone(),
//                content: parsed_body.content.clone(),
//                timestamp: std::time::SystemTime::now(), 
//            };
//            state.add_chat_message(chat_message.clone());
//
//            let chat_message = ChatRequest::ChatMessageReceived(chat_message.clone());
//            match serde_json::to_vec(&chat_message) {
//                Ok(serialized_message) => {
//                    for contact in &state.contacts {
//                        let their_addy = Address {
//                            node: contact.clone(),
//                            process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok().unwrap(),
//                        };
//                        Request::new()
//                            .body(serialized_message.clone())
//                            .target(&their_addy)
//                            .send().ok().unwrap();
//                    }
//                }
//                Err(_e) => println!("Failed to serialize chat message: {:?}", chat_message)
//            }
//            Some((http::StatusCode::OK, headers, Vec::new()))
//        },
//        Err(e) => {
//            println!("(LOCAL) failed to parse chat message from front-end. Error: {:?}", e);
//            Some((http::StatusCode::BAD_REQUEST, headers, Vec::new()))
//        }
//    }
//}
//
////fn send_http_response(response: (http::StatusCode, HashMap<String, String>, Vec<u8>)) {
////    let (status, headers, body) = response;
////    http::send_response(status, Some(headers), body);
////    println!("Response sent: {:?}", status);
////}
//
//fn handle_alien_message(our: &Address, state: &mut State, message: &Message) {
//    if let Ok(alien_request) = serde_json::from_slice::<ContactRequest>(message.body()) {
//        println!("alien request in handling");
//        let their_node = &message.source().node;
//        match alien_request {
//            ContactRequest::RequestContact(_) => {
//                // append the incoming node_id to the incoming_contact_requests
//                if !state.contacts.contains(&their_node) && !state.incoming_contact_requests.contains(&their_node) && their_node != &our.node{ // temp solution for now
//                    state.incoming_contact_requests.push(their_node.clone());
//                    println!("contact request from {:?}", &their_node);
//                }
//            },
//            ContactRequest::ContactAccepted(_) => { 
//                // pressing accept in the UI triggers that the sender receives this ACK from the originial receiver
//                state.contacts.push(their_node.to_string());
//                println!("{} accepted your request. You are now frens <3", &their_node);
//            },
//            ContactRequest::ContactUpdate(entry) => { 
//                //if they're in our contacts, update their score
//                if state.contacts.contains(&their_node) {
//                    state.stats.insert(their_node.to_string(),entry);
//                    println!("updated {:?}", &their_node);
//                } else  {
//                    println!("request from non-contact (delete this later)");
//                }
//            },
//            _ => println!("contact request didn't match anything"),
//        }
//    } else if let Ok(inc_chat_message) = serde_json::from_slice::<ChatRequest>(message.body()) {
//        println!("chat message request in handling");
//        match inc_chat_message {
//            ChatRequest::ChatMessageReceived(chat_message) => {
//                println!("alien chat message = {:?}", chat_message);
//                state.add_chat_message(chat_message); 
//            }
//            _ => println!("something else than a chat message")
//        }
//    }
//}
//
//// pushing your score to your contacts
//fn push_update_to_your_contacts(state: &State) {
//    let our_respects = state.stats.get(&state.node_id).unwrap_or(&LeaderboardEntry { respects: 0 });
//    let our_respect_update = ContactRequest::ContactUpdate(our_respects.clone());
//
//    for contact in &state.contacts {
//        let their_addy = Address {
//            node: contact.clone(),
//            process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok().unwrap(),
//        };
//        Request::new()
//            .body(serde_json::to_vec(&our_respect_update).ok().unwrap())
//            .target(&their_addy)
//            .send().ok().unwrap();
//    }
//}
//
//// Resend pending contact requests
//fn resend_pending_requests(state: &mut State) {
//    let mut nodes_to_remove = Vec::new();
//
//    for node in &state.pending_contact_requests {
//        if !state.contacts.contains(&node) {
//            let their_addy = Address {
//                node: node.clone(),
//                process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok().unwrap(),
//            };
//            let contact_request = ContactRequest::RequestContact(node.clone());
//            match serde_json::to_vec(&contact_request) {
//                Ok(body) => {
//                    let request = Request::new()
//                        .body(body)
//                        .target(&their_addy);
//
//                    if let Err(e) = request.send() {
//                        println!("Failed to resend contact request to {}: {:?}", node, e);
//                    } else {
//                        println!("Resent contact request to {}", node);
//                    }
//                }
//                Err(e) => println!("Failed to serialize contact request for {}: {:?}", node, e),
//            }
//        } else {
//            println!("{:?} already in your contacts, clearing it from the pending list", node);
//            nodes_to_remove.push(node.clone());
//        }
//    }
//    state.pending_contact_requests.retain(|pending| !nodes_to_remove.contains(pending));
//}
//