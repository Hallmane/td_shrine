use serde::{Serialize, Deserialize};
use kinode_process_lib::{get_state, set_state, NodeId, Address};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub node_id: NodeId,
    pub discoverable: bool,
    pub contacts: Vec<NodeId>,
    pub stats: HashMap<NodeId, LeaderboardEntry>,
    pub pending_contact_requests: Vec<NodeId>,  
    pub incoming_contact_requests: Vec<NodeId>,
    // chat stuff below
    //pub server: ServerState,
    pub client: ClientState, 
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ServerState {
    pub chat_state: Vec<ChatMessage>, 
    pub subscribers: HashSet<Address>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClientState {
    //pub server: Option<Address>,
    //pub chat_state: Vec<ChatMessage>,
    pub ws_channels: HashSet<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerryPacket {
    //ServerRequest(ServerRequest), // client -> сервер
    //ServerUpdate(ServerUpdate), // server -> client/s (updated chat)
    ClientRequest(ClientRequest), // client -> server (SetServer and SendToServer)
    PeerRequest(PeerRequest), // peer -> peer
}


//#[derive(Debug, Clone, Serialize, Deserialize)]
//pub enum ServerRequest { 
//    //ChatMessage(ChatMessage),
//    Subscribe,
//    Unsubscribe,
//}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRequest {
    //SendToServer(ServerRequest),
    //ToggleServer(Option<Address>),
    SendToPeer(PeerRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerRequest {
    RequestPeer(NodeId),
    PeerAccepted(NodeId),
    PeerUpdate(LeaderboardEntry),
}

//#[derive(Debug, Clone, Serialize, Deserialize)]
//pub enum ServerUpdate {
//    ChatMessage(ChatMessage),
//    ChatState(Vec<ChatMessage>),
//    SubscribeAck,
//}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender: NodeId,
    pub content: String,
    pub timestamp: SystemTime,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub respects: u64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatRequest {
    ChatMessageReceived(ChatMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactRequestBody {
    pub node: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsUpdate {
    NewChatMessage(ChatMessage),
    ChatHistory(Vec<ChatMessage>), //??:w
    SubscribeAck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageBody {
    pub content: String,
}

impl State {
    /// upon init and the host hasn't added any respects to their shrine yet
    pub fn new(node_id: NodeId) -> Self {
        let stats = HashMap::from([(node_id.clone(), LeaderboardEntry { respects: 0 })]);
        State {
            node_id, //your node
            discoverable: true, // perhaps this should be on by default
            contacts: Vec::new(), // your contacts. Use these to ask them about updates, if they have discoverable on
            stats, // HashMap<contact.node, their entry>, or what to use for rendering the frontend
            pending_contact_requests: Vec::new(),
            incoming_contact_requests: Vec::new(),
            //server: ServerState::default(),
            client: ClientState::default(),
        }
    }

    pub fn fetch(our_node: NodeId) -> State {
        match get_state() {
            Some(state_bytes) => {
                let desbytes = bincode::deserialize(&state_bytes).expect("Correctly deserialized state");
                desbytes
            },
            None => State::new(our_node)
        }
    }

    pub fn save(&self) {
        let state_bytes = bincode::serialize(self).expect("Failed to serialize state");
        set_state(&state_bytes);
    }

    pub fn add_respect(&mut self) {
        let entry = self.stats.entry(self.node_id.clone()).or_insert_with(|| LeaderboardEntry { respects: 0});
        entry.respects += 1;
    }

    pub fn set_discoverable(&mut self, discoverable: bool) {
        self.discoverable = discoverable;
    }

    pub fn append_outgoing_contact_request(&mut self, other_node: NodeId) {
        if !self.contacts.contains(&other_node) && !self.pending_contact_requests.contains(&other_node) {
            self.pending_contact_requests.push(other_node.clone());
        }
    }

    pub fn accept_contact_request(&mut self, other_node: NodeId) {
        if self.incoming_contact_requests.contains(&other_node) {
            self.add_contact(other_node.clone());
            //self.contacts.push(other_node.clone());
            self.incoming_contact_requests.retain(|node| node != &other_node); //? this should remove the added node from the incoming_contact_requests but doesn't
        }
    }

    pub fn add_contact(&mut self, other_node: NodeId) {
        if !self.contacts.contains(&other_node) { 
            self.contacts.push(other_node);
        } else {
            println!("{:?} already in your contacts.", other_node);
        }
    }

    pub fn decline_contact(&mut self, other_node: NodeId) {
        if self.incoming_contact_requests.contains(&other_node) {
            self.incoming_contact_requests.retain(|node| node != &other_node);
        } else {
            println!("tried to decline a node that wasn't in your pending")
        }
    }

    pub fn remove_entry(&mut self, node_id: &NodeId) {
        self.stats.remove(node_id).is_some();
        //println!("Removed entry for node_id {}: {}", node_id, removed);
    }

    //pub fn add_chat_message(&mut self, chat_message: ChatMessage) {
    //    if self.server.chat_state.len() >= 50 {
    //        self.server.chat_state.remove(0);
    //        self.client.chat_state.remove(0);
    //    }
    //    self.server.chat_state.push(chat_message.clone());
    //    self.client.chat_state.push(chat_message.clone());
    //}
}

    //// crdt merge op
    //pub fn merge(&mut self, other: &State) {
    //    for (node_id, other_entry) in &other.stats {
    //        let our_entry = self.stats.entry(node_id.clone()).or_default();
    //        our_entry.respects = std::cmp::max(our_entry.respects, other_entry.respects);
    //    }
    //}

    //fn broadcast_state(state: &State) {
    //    let our_state_serialized = bincode::serialize(state).expect("Failed to serialize data before broadcast");
    //    send_to_all_nodes(our_state_serialized)
    //}

    //fn send_to_all_nodes(data: Vec<u8>) {
    //    // skeleton        
    //}

    //pub fn save(&self) {
    //    match bincode::serialize(self) {
    //        Ok(serialized_state) => {
    //            set_state(&serialized_state);
    //            println!("State successfully saved.");
    //        },
    //        Err(e) => {
    //            eprintln!("Failed to serialize state: {:?}", e);
    //        }
    //    }
    //}
