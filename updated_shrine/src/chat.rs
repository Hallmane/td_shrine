use std::collections::HashSet;
use serde_json::json;
use kinode_process_lib::{
    Address, Message, LazyLoadBlob, http::{self, send_ws_push, WsMessageType},
};
use crate::structs::{State, ChatRequest, ChatMessage};

pub fn handle_chat_request(our: &Address, state: &mut State, body: &[u8]) -> anyhow::Result<()> {
    let chat_request = serde_json::from_slice::<ChatRequest>(body)?;

    match chat_request {
        ChatRequest::ChatMessageReceived(chat_message) => {
            state.add_chat_message(chat_message.clone());

            let blob = LazyLoadBlob {
                mime: Some("application/json".to_string()),
                bytes: serde_json::to_vec(&ChatRequest::ChatMessageReceived(chat_message.clone()))?,
            };

            for &channel_id in &state.clients {
                send_ws_push(
                    our.node.clone(),
                    channel_id,
                    WsMessageType::Text,
                    blob.clone()
                )?;
            }
        }
    }
    Ok(())
}

pub fn handle_http_server_request(our: &Address, state: &mut State, message: &Message) {
    if let Message::Request { ref body, .. } = message {
        let Ok(server_request) = serde_json::from_slice::<http::HttpServerRequest>(body) else {
            return;
        };

        match server_request {
            http::HttpServerRequest::WebSocketOpen { channel_id, .. } => {
                state.clients.insert(channel_id);
            }
            http::HttpServerRequest::WebSocketPush { .. } => {
                let Some(blob) = get_blob() else {
                    return;
                };
                let _ = handle_chat_request(our, state, &blob.bytes);
            }
            http::HttpServerRequest::WebSocketClose(channel_id) => {
                state.clients.remove(&channel_id);
            }
            http::HttpServerRequest::Http(ref incoming) => {
                match process_http_request(our, incoming, state) {
                    Ok(()) => (),
                    Err(e) => {
                        println!("error handling http request: {:?}", e);
                        send_response(
                            http::StatusCode::SERVICE_UNAVAILABLE,
                            None,
                            "Service Unavailable".to_string().as_bytes().to_vec(),
                        );
                    }
                }
            }
        }
    }
}

fn process_http_request(
    our: &Address,
    http_request: &http::IncomingHttpRequest,
    state: &mut State
) -> anyhow::Result<()> {
    let bound_path = http_request.bound_path(Some(&our.process())).rsplit('/').next().unwrap_or("");

    match http_request.method()?.as_str() {
        "GET" => {
            if let Some(response) = handle_get_request(bound_path, state) {
                send_http_response(response);
            }
        }
        "POST" => {
            if let Some(response) = handle_post_request(bound_path, state, http_request) {
                send_http_response(response);
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_get_request(bound_path: &str, state: &State) -> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
    match bound_path {
        "get_chat" => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            let body = serde_json::to_vec(&state.chat_messages).ok()?;
            Some((http::StatusCode::OK, headers, body))
        },
        _ => None,
    }
}

fn handle_post_request(bound_path: &str, state: &mut State, http_request: &http::IncomingHttpRequest) 
-> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
    match bound_path {
        "send_chat_message" => handle_send_chat_message(state, http_request),
        _ => None,
    }
}

fn handle_send_chat_message(state: &mut State, _http_request: &http::IncomingHttpRequest) 
-> Option<(http::StatusCode, HashMap<String, String>, Vec<u8>)> {
    let body = get_blob()?;
    let body_str = std::str::from_utf8(&body.bytes).unwrap_or_default();
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

