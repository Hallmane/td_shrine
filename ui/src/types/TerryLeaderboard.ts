export interface Address {
    node: string; 
    process: string; 
}

export interface LeaderboardEntry {
    respects: number;
}

 export interface LeaderboardState {
    node_id: string;
    discoverable: boolean;
    contacts: string[];
    stats: Record<string, LeaderboardEntry>;
    pending_contact_requests: string[];
    incoming_contact_requests: string[];
    //chat_history: ChatMessage[];
 }

export interface ChatMessage {
    sender: string;
    content: string;
    timestamp: number;
}

export interface Chat {
    chat_history: ChatMessage[]
}

export interface ServerRequest {
    type: "ServerRequest";
    request: "ChatMessage";
    data: ChatMessage;
}

export interface ClientRequest {
    type: "ClientRequest";
    request: "SendToServer";
    data: ServerRequest;
}
