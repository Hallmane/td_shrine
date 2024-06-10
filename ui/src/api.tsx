import { LeaderboardState, Chat, ChatMessage, ClientRequest } from "./types/TerryLeaderboard";

const BASE_URL = import.meta.env.BASE_URL;

export const getLeaderboard = async (): Promise<LeaderboardState | null> => {
    try {
        const response = await fetch(`${BASE_URL}/get_leaderboard`);
        if (!response.ok) throw new Error("Failed to fetch leaderboard");
        return response.json();
    } catch (error) {
        console.error(error); 
        return null;
    }
};

export const getChat = async (): Promise<Chat | null> => {
    try {
        const response = await fetch(`${BASE_URL}/get_chat`);
        if (!response.ok) throw new Error("Failed to fetch chat");
        return response.json();
    } catch (error) {
        console.error(error); 
        return null;
    }
};

export const addRespect = async (nodeId: string): Promise<boolean> => {
    try {
        const response = await fetch(`${BASE_URL}/add_respect`, {
            method: "POST",
            headers: { "Content-Type": "application/json"},
            body: JSON.stringify( {node: nodeId} ),
        });

        return response.ok;
    } catch (error) {
        console.error(error); 
        return false;
    }
};

export const setDiscoverable = async (discoverable: boolean): Promise<boolean> => {
    try {
        const response = await fetch(`${BASE_URL}/set_discoverable`, {
            method: "POST",
            headers: {"Content-Type": "application/json"},
            body: JSON.stringify({ discoverable: discoverable }),
        });
        return response.ok;
    } catch (error) {
        console.error(error);
        return false;
    }
};

export const sendContactRequest = async (nodeId: string): Promise<boolean> => {
    try {
        const response = await fetch(`${BASE_URL}/send_contact_request`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ node: nodeId })
        });

        return response.ok;
    } catch (error) {
        console.error(error); 
        return false;
    }
};

export const acceptContactRequest = async (nodeId: string): Promise<boolean> => {
    try {
        const response = await fetch(`${BASE_URL}/accept_contact`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ node: nodeId })
        });

        return response.ok
    } catch (error) {
        console.error(error)
        return false;
    }
}

export const declineContactRequest = async (nodeId: string): Promise<boolean> => {
    try {
        const response = await fetch(`${BASE_URL}/decline_contact`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ node: nodeId })
        });

        return response.ok
    } catch (error) {
        console.error(error)
        return false;
    }
}

export const sendChatMessage = async (packet): Promise<boolean> => {
    try {
        const response = await fetch(`${BASE_URL}/send_chat_message`, {
            method: "POST",
            headers: { "Content-Type": "application/json"},
            body: JSON.stringify(packet),
        });

        return response.ok;
    } catch (error) {
        console.error(error); 
        return false;
    }
}

export const removeLeaderboardEntry = async (nodeId: string): Promise<boolean> => { 
    try {
        const response = await fetch(`${BASE_URL}/remove_leaderboard_entry`, {
            method: "POST",
            headers: { "Content-Type": "application/json"},
            body: JSON.stringify( {node: nodeId} ),
        });

        return response.ok;
    } catch (error) {
        console.error(error); 
        return false;
    }
}



