use std::collections::HashMap;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use kinode_process_lib::{  
    Address, NodeId, Message, ProcessId, Request, Response,
    await_message, call_init, http, get_blob, clear_state, println, 
    http::{bind_http_path, bind_ws_path, send_response, send_ws_push, serve_ui},
    vfs::{open_file, create_drive},
    
};

mod structs;
use structs::LeaderboardEntry;
use structs::State;
use structs::ContactRequest;
use structs::ContactRequestBody;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
});


/// make this nicer
fn handle_http_request(our: &Address, state: &mut State, message: &Message) {
    if let Message::Request { ref body, .. } = message {
        if let Some(response) = process_http_request(our, body, state)  {
            send_http_response(response);
        }
    }
}

/// TODO: do I really need this strict return type?
fn process_http_request(
    our: &Address,
    body: &[u8],
    state: &mut State
) -> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
    let server_request = http::HttpServerRequest::from_bytes(body).ok()?;
    let http_request = server_request.request()?;
    //println!("--------------http_request value: {:?}", http_request);
    let bound_path = http_request.bound_path(Some(&our.process())).rsplit('/').next().unwrap_or("");

    match http_request.method().ok()? {
        http::Method::GET => match bound_path {
            "leaderboard" => {
                println!("leaderboart get request");
                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                let body = match serde_json::to_vec(state) {
                    Ok(body) => body,
                    Err(e) => {
                        println!("Error serializing state: {:?}", e);
                        return None;
                    }
                };
                Some((http::StatusCode::OK, headers, body))
            }, 
            _ => None
        },
        http::Method::POST => match bound_path {
            "add_respect" => {
                state.add_respect();
                Some((http::StatusCode::OK, HashMap::new(), Vec::new()))
            },
            "send_contact_request" => {
                let body = get_blob()?;
                let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
                //println!("body={:?}\nbody_str={:?}", body, body_str);

                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());

                match serde_json::from_str::<ContactRequestBody>(&body_str) {
                    Ok(parsed_body) => {  //getting the node value from our local http request
                        // Call the function to add to pending_contact_requests
                        let their_addy = Address {
                            node: parsed_body.node.clone(), 
                            process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok()?, //this sucks?
                        };
                        println!("their constructed address clone: {:?}", their_addy.clone());
                        Request::new()
                            .body(serde_json::to_vec(&ContactRequest::RequestContact(parsed_body.node.clone())).ok()?)
                            //.body(serde_json::to_vec(&ContactRequest::RequestContact {parsed_body: parsed_body.node.clone()})?)                                                                                                                     
                            .target(&their_addy)
                            .send().ok()?;

                        println!("sent request to {:?}", parsed_body.node);
                        state.append_outgoing_contact_request(parsed_body.node); 
                        Some((http::StatusCode::OK, headers, Vec::new()))
                    },
                    Err(e) => {
                        println!("Failed to parse the contact request body: {:?}", e);
                        Some((http::StatusCode::BAD_REQUEST, headers, Vec::new()))
                    }
                }
            },
            "set_discoverable" => { //works
                //println!("PATH: set_discoverable");
                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());

                let mut current_val = state.discoverable;
                state.set_discoverable(!current_val);

                Some((http::StatusCode::OK, headers, Vec::new()))

            },
            //TODO: why isn't the incoming_contact_requests updating correctly?
            "accept_contact" => { //this gets called locally, we need to respond to the original sender with our address
                let body = get_blob()?;
                let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
                //println!("body={:?}\nbody_str={:?}", body, body_str);

                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());

                match serde_json::from_str::<ContactRequestBody>(&body_str) {
                    Ok(parsed_body) => {
                        let their_node = parsed_body.node.clone();
                        println!("inside the OK'd parsed body");
                        state.add_contact(their_node.clone()); 
                        state.incoming_contact_requests.retain(|incoming| *incoming != their_node);
                        //xs.retain(|&x| x != some_x);
                        // Call the function to add to pending_contact_requests
                        let their_addy = Address {
                            node: their_node.clone(), 
                            process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok()?, //this sucks!
                        };
                        println!("their constructed address clone: {:?}", their_addy.clone());
                        Request::new()
                            .body(serde_json::to_vec(&ContactRequest::ContactAccepted(their_node.clone())).ok()?)
                            //.body(serde_json::to_vec(&ContactRequest::RequestContact {parsed_body: parsed_body.node.clone()})?)                                                                                                                     
                            .target(&their_addy)
                            .send().ok()?;

                        println!("sent request to {:?}", their_node);
                        Some((http::StatusCode::OK, headers, Vec::new()))
                    },
                    Err(e) => {
                        println!("failed to parse the local contact request {e:?}");
                        Some((http::StatusCode::BAD_REQUEST, headers, Vec::new()))
                    }
                }
                //println!("PATH: accept_contact");
                //state.confirm_contact(node_id);
                //let mut headers = HashMap::new();
                //headers.insert("Content-Type".to_string(), "application/json".to_string());
                //Some((http::StatusCode::OK, headers, Vec::new()))
            },
            //"remove_leaderboard_entry" => {
            //    println!("PATH: remove_leaderboard_entry");
            //    state.remove_entry(&our.node);
            //    Some((http::StatusCode::OK, HashMap::new(), Vec::new()))
            //},
            _ => None
        },
        _ => None
    }
}

fn send_http_response(response: (http::StatusCode, HashMap<String, String>, Vec<u8>)) {
    let (status, headers, body) = response;
    http::send_response(status, Some(headers), body);
    println!("Response sent: {:?}", status);
}

fn handle_alien_message(our: &Address, state: &mut State, message: &Message) {
    if let Ok(alien_request) = serde_json::from_slice::<ContactRequest>(message.body()) {
        let their_node = &message.source().node;
        //println!("message: {:?}", message);
        match alien_request {
            ContactRequest::RequestContact(node_id) => {
                // append the incoming node_id to the incoming_contact_requests
                if !state.contacts.contains(&their_node) && !state.incoming_contact_requests.contains(&their_node) && their_node != &our.node{ // temp solution for now
                    state.incoming_contact_requests.push(their_node.clone());
                    println!("contact request from {:?}", &their_node);
                }
            },
            //TODO: remove that node from pending
            ContactRequest::ContactAccepted(node_id) => { 
                // pressing accept in the UI triggers sender recieves this ACK from the originial receiver
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
            _ =>  {
                println!("contact request didn't match anything");
                return
            }
        }
    }
}

fn push_update_to_your_contacts(our: &Address, state: &State) -> Option<()> {
    let our_respects = state.stats.get(&state.node_id).unwrap_or(&LeaderboardEntry {respects: 0});
    let our_respect_update = ContactRequest::ContactUpdate(our_respects.clone());

    for contact in &state.contacts {
        let their_addy = Address {
            node: contact.clone(), 
            process: ProcessId::from_str("updated_shrine:td_shrine:sharmouta.os").ok()?,
        };
        Request::new()
            .body(serde_json::to_vec(&our_respect_update).ok()?)
            .target(&their_addy)
            .send().ok()?;
    }
    Some(())
}

call_init!(init);
fn init(our: Address) {
    println!("{our} started!");

    clear_state();
    let mut state = State::fetch(our.node().to_string()); 

    //let our = Address::from_str(&our).unwrap();

    // Bind UI files to routes; index.html is bound to "/"
    serve_ui(&our, "ui", true, true, vec!["/"]).unwrap();

    //TODO: look at what this actually does
    bind_http_path("/leaderboard", true, false).unwrap();
    bind_http_path("/add_respect", true, false).unwrap();
    bind_http_path("/set_discoverable", true, false).unwrap();
    bind_http_path("/remove_leaderboard_entry", true, false).unwrap();
    bind_http_path("/send_contact_request", true, false).unwrap();
    bind_http_path("/accept_contact", true, false).unwrap();

    // Bind WebSocket path
    bind_ws_path("/", true, false).unwrap(); //wat is

    kinode_process_lib::timer::set_timer(10_000, None); //for pushing local updates to your contacts

    while let Ok(message) = await_message() {
        println!("our state: {:?}", state);
        //println!("Received message from Node: {}, Process: {}", message.source().node, message.source().process);
        if message.source().node == our.node {
            if message.source().process == "timer:distro:sys" {
                println!("timer update!");
                //push your respect to all your contacts, i.e. map over contacts and send the singular value to each one of them. 
                push_update_to_your_contacts(&our, &state);
                kinode_process_lib::timer::set_timer(30_000, None); //for pushing local updates to your contacts
            } else if message.source().process == "http_server:distro:sys" { //local http 
                handle_http_request(&our, &mut state, &message);
            } else {
                println!("other process than the shrine");
                continue
            }
        } else { //alien
            println!("incoming alien message");
            if state.discoverable {
                println!("you are discoverable, message will be handled");
                handle_alien_message(&our, &mut state, &message); 
            }
        }
        state.save();
    }
}