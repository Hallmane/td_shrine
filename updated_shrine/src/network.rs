use std::collections::HashMap;
use crate::structs::{ChatMessage, ChatRequest, State};

use kinode_process_lib::{
    get_blob,
    http::{
        bind_http_path, bind_ws_path, send_response, send_ws_push, serve_ui, HttpServerRequest,
        StatusCode, WsMessageType,
    },
    println, Address, LazyLoadBlob, Message, ProcessId, Request, Response,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Contact {
    pub node_id: NodeId,
    pub connection: Option<WebSocket>,
}

pub struct Network {
    pub our_node: NodeId,
    pub contacts: HashMap<NodeId, Contact>,
}

impl Network {
    pub fn new(our_node: NodeId, contacts: Vec<NodeId>) -> Self {
        let mut contact_map = HashMap::new();
        for contact in contacts {
            contact_map.insert(contact.clone(), Contact { node_id: contact, connection: None });
        }
        Network {
            our_node,
            contacts: contact_map,
        }
    }

    pub fn connect_to_contacts(&mut self) {
        for contact in self.contacts.values_mut() {
            let ws_url = format!("ws://{}:{}", contact.node_id, "websocket_port");
            let ws_connection = WebSocket::connect(&ws_url).expect("Failed to connect");
            contact.connection = Some(ws_connection);
        }
    }

    pub fn send_message(&self, message: &ChatMessage) {
        let serialized_message = serde_json::to_string(&ChatRequest::ChatMessageReceived(message.clone())).expect("Serialization failed");
        for contact in self.contacts.values() {
            if let Some(connection) = &contact.connection {
                connection.send(&serialized_message).expect("Failed to send message");
            }
        }
    }

    pub fn handle_incoming_message(&mut self, message: &str, state: &mut State) {
        if let Ok(chat_request) = serde_json::from_str::<ChatRequest>(message) {
            match chat_request {
                ChatRequest::ChatMessageReceived(chat_message) => {
                    println!("Received message: {:?}", chat_message);
                    state.add_chat_message(chat_message);
                }
            }
        }
    }
}
