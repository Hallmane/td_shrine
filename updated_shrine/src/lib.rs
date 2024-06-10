use std::collections::HashMap;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use kinode_process_lib::{
    Address, NodeId, Message, ProcessId, Request, Response, LazyLoadBlob,
    await_message, call_init, http, get_blob, clear_state, println,
    http::{WsMessageType, HttpServerRequest, send_ws_push, bind_http_path, bind_ws_path, send_response, serve_ui},
};

mod structs;
use structs::{
    TerryPacket, ServerRequest, ServerUpdate, ClientRequest, State, ClientState, ServerState,
    LeaderboardEntry, PeerRequest, ContactRequestBody, ChatMessage, ChatMessageBody, ChatRequest, WsUpdate
};

wit_bindgen::generate!({
    path: "wit",
    world: "process",
});

const SERVER_NODE : &str= "sharmouta.os";
const PROCESS : &str = "updated_shrine:td_shrine:sharmouta.os";

call_init!(init);
fn init(our: Address) {
    println!("{our} started!");

    clear_state();
    let mut state = State::fetch(our.node().to_string());

    serve_ui(&our, "ui", true, true, vec!["/"]).unwrap();

    //TODO: clean 
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

    //connect to the remote server
    let server_address = Address::from_str(&format!("{}@{}", SERVER_NODE, PROCESS)).expect("Invalid address format");

    let _ = connect_to_server(&mut state, &server_address);

    kinode_process_lib::timer::set_timer(10_000, None); //TODO: remove this kind of functionality

    while let Ok(message) = await_message() {
        //println!("our state: {:?}", state);
        match handle_message(&our, &mut state, message) {
            Ok(()) => {}
            Err(e) => {println!("message handling error: {:?}", e);}
        }
        state.save();
    }
}

fn connect_to_server(state: &mut State, server: &Address) -> anyhow::Result<()> {
    if let Some(old_server) = &mut state.client.server {
        if *old_server != *server {
            poke_server(state, ServerRequest::Unsubscribe)?;
            set_server(state, server)?;
        } 
    } else {
        set_server(state, server)?;
    }

    let send_body : Vec<u8> = serde_json::to_vec(&TerryPacket::ServerRequest(ServerRequest::Subscribe)).unwrap();
    Request::new()
        .target(server)
        .body(send_body)
        .send()
        .unwrap();

    Ok(())
}

fn set_server(state: &mut State, new_server: &Address) -> anyhow::Result<()> {
    state.client = ClientState {
        server: Some(new_server.clone()),
        chat_state: Vec::new(),
        ws_channels: state.client.ws_channels.clone(),
    };
    Ok(())
}

fn handle_message(
    our: &Address, 
    state: &mut State, 
    message: Message
) -> anyhow::Result<()> {
    if message.source().node == our.node { //local inter-processing comms
        let pid_str =  message.source().process.to_string();
        match pid_str.as_str() {  
            "timer:distro:sys" => handle_timer_events(our, state),
            "http_server:distro:sys" => handle_http_server_request(our, state, &message), // extract stuff form this handler to the match statement below
            _ => return Ok(())
        };
    } else {
        match serde_json::from_slice(&message.body())? {
            TerryPacket::ServerRequest(sreq) => { 
                if our.node != SERVER_NODE {
                    return Ok(());
                }
                handle_server_request(message.source(), state, sreq)?; 
            }
            TerryPacket::ServerUpdate(supt) => { handle_server_update(message.source().clone(), state, supt)?; }
            TerryPacket::ClientRequest(creq) => { handle_client_request(state, creq)?; }
            TerryPacket::PeerRequest(preq) => { handle_peer_message(our, &message.source().node, state, preq)?; }
            _ => return Ok(()) //??
        }
    }
    Ok(())
}

// dont touch for now
fn handle_server_request(
    source: &Address,
    state: &mut State, 
    req: ServerRequest
) -> anyhow::Result<()> {
    match req {
        ServerRequest::ChatMessage(chat_message) => {
            let chat_message = ChatMessage {
                sender: source.node.clone(),
                content: chat_message.content, //??
                timestamp: std::time::SystemTime::now(),
            };
            send_server_update(state, ServerUpdate::ChatMessage(chat_message.clone()));
        }
        ServerRequest::Subscribe => {
            state.server.subscribers.insert(source.clone());
            let send_body : Vec<u8> = serde_json::to_vec(&TerryPacket::ServerUpdate(ServerUpdate::SubscribeAck)).unwrap();
            Request::new()
                .target(source.clone()) 
                .body(send_body)
                .send()
                .unwrap();
            let send_body : Vec<u8> = serde_json::to_vec(&TerryPacket::ServerUpdate(ServerUpdate::ChatState(state.server.chat_state.clone()))).unwrap();
            Request::new()
                .target(source.clone()) 
                .body(send_body)
                .send()
                .unwrap();
        }
        ServerRequest::Unsubscribe => { 
            if let _ = state.server.subscribers.remove(&source)  {
                return Ok(())
            } else {
                return Ok(()) //?? no
            }
        } //TODO: maybe some client teardown is needed?
    } 
    Ok(())
}

fn handle_server_update (
    source: Address,
    state: &mut State, 
    supt: ServerUpdate
) -> anyhow::Result<()> {
    if let Some(server) = &mut state.client.server {
        if source != *server {
            return Ok(()) //??
        }
        match supt {
            ServerUpdate::ChatMessage(chat_message) => {ws_broadcast(state, WsUpdate::NewChatMessage(chat_message)).unwrap(); }
            ServerUpdate::ChatState(chat_history) => {ws_broadcast(state, WsUpdate::ChatHistory(chat_history)).unwrap(); }
            ServerUpdate::SubscribeAck => {return Ok(());}
        }
        Ok(())
    } else {
        return Ok(()) //??
    }
}

fn send_server_update(state: &mut State, update: ServerUpdate) -> anyhow::Result<()> {
    let update_body = TerryPacket::ServerUpdate(update);
    let send_body : Vec<u8> = serde_json::to_vec(&update_body).unwrap();
    for sub in state.server.subscribers.iter() {
        Request::new()
            .target(sub)
            .body(send_body.clone())
            .send()
            .unwrap();
    }
    Ok(())
}

// the timing needs to be more sophisicated 
fn handle_timer_events(our: &Address, state: &mut State) -> anyhow::Result<()>{
    //println!("timer update.");
    push_update_to_your_contacts(our, state);
    if !state.pending_contact_requests.is_empty() {
        resend_pending_requests(state);
    }
    kinode_process_lib::timer::set_timer(30_000, None);
    return Ok(())
}

// TODO: clean (yeh sure)
fn handle_http_server_request(
    our: &Address, 
    state: &mut State, 
    message: &Message
) -> anyhow::Result<()> {
    if let Message::Request { ref body, .. } = message {
        let Ok(server_request) = serde_json::from_slice::<HttpServerRequest>(body) else { return Ok(()); };

        match server_request {
            HttpServerRequest::WebSocketOpen { channel_id, ..} => { 
                state.client.ws_channels.insert(channel_id); 
                ws_broadcast(&state, WsUpdate::ChatHistory(state.client.chat_state.clone())).unwrap();//newly connected clients get to see the chat history
            }
            HttpServerRequest::WebSocketPush { .. } => {
                let Some(blob) = get_blob() else { return Ok(());};
                let Ok(blob_string) = String::from_utf8(blob.bytes) else { return Ok(());};
                let chat_message: ChatMessageBody = serde_json::from_str(&blob_string)?;
                let chat_message = ChatMessage {
                    sender: state.node_id.clone(),
                    content: chat_message.content.clone(),
                    timestamp: std::time::SystemTime::now(),
                };
                state.add_chat_message(chat_message.clone());
                ws_broadcast(state, WsUpdate::NewChatMessage(chat_message)).unwrap();
            },
            HttpServerRequest::WebSocketClose(channel_id) =>  { state.client.ws_channels.remove(&channel_id); },

            HttpServerRequest::Http(request) => { //move to own handler
                let bound_path = request.bound_path(Some(&our.process())).rsplit('/').next().unwrap_or("");

                match request.method()? {
                    http::Method::GET => handle_get_request(bound_path, state),
                    //http::Method::POST => handle_post_request(bound_path, state, &request),
                    http::Method::POST => { 

                        println!("1. bound_path of http POST request: {:?}", bound_path);

                        let Some(blob) = get_blob() else { return Ok(())};
                        let Ok(post_string) = String::from_utf8(blob.bytes) else {return Ok(())};

                        println!("2. Http > POST > (post_string): {:?}", &post_string);

                        match serde_json::from_str(&post_string)? {
                            ClientRequest::SendToServer(req) => {
                                println!("3. Http > POST > post_string > serde_json {:?}", req);

                                handle_client_request(state, ClientRequest::SendToServer(req))?;
                                
                                Ok(())

                                //let send_body : Vec<u8> = serde_json::to_vec(&TerryPacket::ClientRequest(req)).unwrap();
                                //Request::new()
                                //    .target(our)
                                //    .body(send_body)
                                //    .send()
                                //    .unwrap();
                                //Ok(())
                            }
                            _ => { 
                                println!("catch-rest clause");
                                Ok(()) 
                            }
                        }
                    }
                    _ => return Err(anyhow::anyhow!("blabla"))
                };
            },
            _ => {return Err(anyhow::anyhow!("not sure what this request is: {:?}", server_request));}
        }
        Ok(())
    } else {
        return Err(anyhow::anyhow!("message is not a request: {:?}", message));
    }
}

// How did this even work before?
fn handle_get_request(bound_path: &str, state: &State) -> anyhow::Result<()> {
    println!("get request handler");
    match bound_path {
        "get_leaderboard" => {
            //println!("get_leaderboard matched");
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            let body = serde_json::to_vec(state)?;
            Ok(())
        },
        _ => Ok(())
    }
}

// these are all done through the frontend ui
fn handle_post_request(bound_path: &str, state: &mut State, http_request: &http::IncomingHttpRequest) 
-> anyhow::Result<()> {
    match bound_path {
        "add_respect" => {
            state.add_respect();
            Ok(())
        },
        "send_contact_request" => handle_send_contact_request(state, http_request), 
        "set_discoverable" => {
            state.set_discoverable(!state.discoverable);
            Ok(())
        },
        "accept_contact" => handle_accept_contact(state, http_request),
        "decline_contact" => handle_decline_contact(state, http_request),
        "send_chat_message" => handle_send_chat_message(state, http_request),
        _ => return Err(anyhow::anyhow!("bound path not valid: {:?}", bound_path))
    }
}


fn send_http_response(response: (http::StatusCode, HashMap<String, String>, Vec<u8>)) {
    let (status, headers, body) = response;
    http::send_response(status, Some(headers), body);
    println!("Response sent: {:?}", status);
}

fn handle_client_request(
    state: &mut State,
    creq: ClientRequest,
) -> anyhow::Result<()> {
    match creq {
        ClientRequest::SendToServer(sreq) => {
            match sreq.clone() {
                ServerRequest::ChatMessage(chat_message) => { 
                    let wrapped_chat_message = ServerRequest::ChatMessage(chat_message.clone());
                    poke_server(state, wrapped_chat_message)?;
                }, 
                _ => { poke_server(state, sreq)? } // sub, unsub or something that wont be matched
                //ServerRequest::Subscribe => { connect_to_server(state) },
                //ServerRequest::Unsubscribe => { connect_to_server(state) },
            }
        },
        ClientRequest::ToggleServer(server) => {
            if let Some(address) = server { 
                connect_to_server(state, &address)?;
            } else { // the client wants to disconnect
                poke_server(state, ServerRequest::Unsubscribe);

                //perhaps also let other ws clients know that this client is bailing?
                
                state.client.server = None;
            }
        }, 
        ClientRequest::SendToPeer(pmes) => { // TODO:
            println!("got a peer message");
            //RequestPeer(NodeId),
            //PeerAccepted(NodeId),
            //PeerUpdate(LeaderboardEntry),
        }, 
    }
    Ok(())
}

//TODO: implement
fn poke_peer(state: &State, req: PeerRequest, peer: Address) -> anyhow::Result<()> {
    if let node = &state.contacts.contains(&peer.node){
        let request_body : Vec<u8> = serde_json::to_vec(&TerryPacket::PeerRequest(req)).unwrap();
        Request::new()
            .target(peer)
            .body(request_body)
            .send()
            .unwrap();
        Ok(()) //
    } else {
        Ok(()) //
    }
}

fn poke_server(state: &State, req: ServerRequest) -> anyhow::Result<()> {
    if let Some(server) = &state.client.server {
        let request_body : Vec<u8> = serde_json::to_vec(&TerryPacket::ServerRequest(req)).unwrap();
        Request::new()
            .target(server)
            .body(request_body)
            .send()
            .unwrap();
        Ok(()) //
    } else {
        Ok(()) //
    }
}

// strictly p2p
fn handle_peer_message(
    our: &Address, 
    source_node: &NodeId,
    state: &mut State, 
    preq: PeerRequest,
) -> anyhow::Result<()> {
    if state.discoverable || state.pending_contact_requests.contains(source_node) || state.contacts.contains(source_node) {
        match preq {
            PeerRequest::RequestPeer(_) => {
                // append the incoming node_id to the incoming_contact_requests
                if !state.contacts.contains(&source_node) && !state.incoming_contact_requests.contains(&source_node) && source_node != &our.node{ // temp solution for now
                    state.incoming_contact_requests.push(source_node.clone());
                    println!("contact request from {:?}", &source_node);
                    return Ok(())
                } else {
                    return Err(anyhow::anyhow!("bla"))
                }
            },
            PeerRequest::PeerAccepted(_) => { 
                // pressing accept in the UI triggers that the sender receives this ACK from the originial receiver
                state.contacts.push(source_node.to_string());
                println!("{} accepted your request. You are now frens <3", &source_node);
                Ok(())
            },
            PeerRequest::PeerUpdate(entry) => { 
                //if they're in our contacts, update their score
                if state.contacts.contains(&source_node) {
                    state.stats.insert(source_node.to_string(),entry);
                    println!("updated {:?}", &source_node);
                    return Ok(())
                } else  {
                    return Ok(()) //request from non-contact, ~ignore
                }
            },
            _ => Err(anyhow::anyhow!("couldn't match the peer request: {:?}", preq))
        }
      //return Err(anyhow::anyhow!("alien request was not ok: {:?}", message.body()));
    } else {
        return Ok(()) //ignore
    }
}

//TODO: These contact request can be simplified to one function using stronger typing
fn handle_send_contact_request(state: &mut State, http_request: &http::IncomingHttpRequest) -> anyhow::Result<()> {
    let body = get_blob().ok_or(anyhow::anyhow!("couln't get body from blob"))?;
    let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    match serde_json::from_str::<ContactRequestBody>(body_str) {
        Ok(parsed_body) => {
            let their_addy = Address {
                node: parsed_body.node.clone(),
                process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os")?
            };
            Request::new()
                .body(serde_json::to_vec(&PeerRequest::RequestPeer(parsed_body.node.clone()))?)
                .target(&their_addy)
                .send()?;
            state.append_outgoing_contact_request(parsed_body.node);
            Ok(())
        },
        _ => {
            return Err(anyhow::anyhow!("failed to pares the body {:?}", body_str));
        }
    }
}

fn handle_accept_contact(state: &mut State, http_request: &http::IncomingHttpRequest) -> anyhow::Result<()> {
    let Some(body) = get_blob() else {return Ok(())};
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
                process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os")?,
            };
            Request::new()
                .body(serde_json::to_vec(&PeerRequest::PeerAccepted(their_node.clone()))?)
                .target(&their_addy)
                .send()
                .unwrap();
            println!("sent contact accepted to {:?}", &their_node.to_string());
            Ok(())
        },
        _ => {
            return Err(anyhow::anyhow!("Failed to parse body: {:?}", body_str));
        }
    }
}

fn handle_decline_contact(state: &mut State, http_request: &http::IncomingHttpRequest) 
-> anyhow::Result<()> {
    let body = get_blob().ok_or(anyhow::anyhow!("couldn't get body from blob"))?;
    let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    match serde_json::from_str::<ContactRequestBody>(body_str) {
        Ok(parsed_body) => {
            let their_node = parsed_body.node.clone();
            state.decline_contact(their_node.clone());
            Ok(())
        },
        _ => {
            return Err(anyhow::anyhow!("failed to parse body {:?}", body_str));
        }
    }
}

//TODO: This should get replaced by the server message handling
fn handle_send_chat_message(state: &mut State, http_request: &http::IncomingHttpRequest)-> anyhow::Result<()> {
    let body = get_blob().ok_or(anyhow::anyhow!("couldn't get body from blob"))?;
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

            match serde_json::to_vec(&chat_message) {
                Ok(serialized_message) => {
                    for contact in &state.contacts {
                        let their_addy = Address {
                            node: contact.clone(),
                            process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").unwrap(),
                        };
                        Request::new()
                            .body(serialized_message.clone())
                            .target(&their_addy)
                            .send()
                            .unwrap();
                    }
                }
                Err(_e) => println!("Failed to serialize chat message: {:?}", chat_message)
            }
            Ok(())
        },
        _ => {
            return Err(anyhow::anyhow!("failed to parse the body {:?}", body_str));
        }
    }
}

// pushing your score to your contacts p2p
fn push_update_to_your_contacts(our: &Address, state: &State) {
    let our_respects = state.stats.get(&state.node_id).unwrap_or(&LeaderboardEntry { respects: 0 });
    let our_respect_update = PeerRequest::PeerUpdate(our_respects.clone());

    for contact in &state.contacts {
        let their_addy = Address {
            node: contact.clone(),
            process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").unwrap(),
        };
        Request::new()
            .body(serde_json::to_vec(&our_respect_update).unwrap())
            .target(&their_addy)
            .send().ok().unwrap();
    }
}

// TODO: relocate this?
fn resend_pending_requests(state: &mut State) {
     //println!("resending contact requests");

     let mut nodes_to_remove = Vec::new();

     for node in &state.pending_contact_requests {
        if !state.contacts.contains(&node) {
            let their_addy = Address {
                node: node.clone(),
                process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok().unwrap(),  
            };
            let contact_request = PeerRequest::RequestPeer(node.clone());
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

//fn handle_websocket_event(
//    our: Address, 
//    state: &mut State, 
//    message: &Message
//) -> anyhow::Result<()> {
//    let Ok(server_request) = serde_json::from_slice::<HttpServerRequest>(message.body()) else {
//        return Err(anyhow::anyhow!("couldn't get the server request: {:?}", message));
//    };
//
//    match server_request {
//        HttpServerRequest::WebSocketOpen { channel_id, ..} => { 
//            state.ws_channels.insert(channel_id); 
//            ws_broadcast(&state, WsUpdate::ChatHistory(state.chat_history.clone())).unwrap();//newly connected clients get to see the chat history
//        },
//        HttpServerRequest::WebSocketPush { .. } => {
//            let Some(blob) = get_blob() else { return Ok(());};
//            let Ok(blob_string) = String::from_utf8(blob.bytes) else { return Ok(());};
//            let chat_message: ChatMessageBody = serde_json::from_str(&blob_string)?;
//            let chat_message = ChatMessage {
//                sender: state.node_id.clone(),
//                content: chat_message.content.clone(),
//                timestamp: std::time::SystemTime::now(),
//            };
//            state.add_chat_message(chat_message.clone());
//            ws_broadcast(state, WsUpdate::NewChatMessage(chat_message)).unwrap();
//        },
//        HttpServerRequest::WebSocketClose(channel_id) =>  { state.ws_channels.remove(&channel_id); },
//        _ => {}
//    };
//
//    Ok(())
//}

fn ws_broadcast(state: &State, update: WsUpdate) -> anyhow::Result<()> {
    let blob = LazyLoadBlob {
        mime: Some("application/json".to_string()),
        bytes: serde_json::json!({
            "WsUpdate": update
        })
        .to_string()
        .as_bytes()
        .to_vec(),
    };

    for channel in &state.client.ws_channels {
        send_ws_push(*channel, WsMessageType::Text, blob.clone());
    }
    Ok(())
}