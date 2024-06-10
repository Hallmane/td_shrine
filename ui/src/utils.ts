import { ClientRequest, ServerRequest, ChatMessage } from './types/TerryLeaderboard'; 

export function createChatMessagePacket(message: ChatMessage): ClientRequest {
    const packet: ClientRequest = {
        type: "ClientRequest",
        request: "SendToServer",
        data: {
            type: "ServerRequest",
            request: "ChatMessage",
            data: message,
        }
    };
    return packet;
}


