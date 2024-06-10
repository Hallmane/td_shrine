import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { Chat, ChatMessage, LeaderboardState, ClientRequest } from '../types/TerryLeaderboard';
import { getLeaderboard, getChat, addRespect, setDiscoverable, sendContactRequest, removeLeaderboardEntry, acceptContactRequest, declineContactRequest, sendChatMessage } from '../api';
import KinodeClientApi from "@kinode/client-api";
//import { Leaderboard } from '../leaderboard';

interface ShrineStore {
    leaderboard: LeaderboardState;
    initializeStore: () => Promise<void>;
    updateLeaderboard: () => Promise<void>;
    addRespect: (nodeId: string) => Promise<void>;
    sendContactRequest: (nodeId: string) => Promise<void>;
    acceptContactRequest: (nodeId: string) => Promise<void>;
    removeLeaderboardEntry: (nodeId: string) => Promise<void>;
    setDiscoverable: (discoverable: boolean) => Promise<void>;
    chat: Chat;
    updateChat: () => Promise<void>;
    clearChatHistory: () => Promise<void>;
    sendChatMessage: (content: string) => Promise<void>;
    receiveChatMessage: (message: ChatMessage) => void; 
    api: KinodeClientApi | null;
    setApi: (api: KinodeClientApi) => void;
}

const useShrineStore = create<ShrineStore>()(
    persist(
        (set, get) => ({
            leaderboard: {
                node_id: "",
                discoverable: false, 
                contacts: [], 
                stats: {},
                pending_contact_requests: [],
                incoming_contact_requests: [],
                chat_history: [],
            }, 
            initializeStore: async () => {
                console.log('Initializing store...');
                await Promise.all([get().updateLeaderboard(), get().updateChat()]);
                console.log('Store initialized');
            },

            updateLeaderboard: async () => {
                const state = await getLeaderboard();
                console.log('Fetched leaderboard state:', state);
                if (state) set({ leaderboard: state });
            },

            addRespect: async (nodeId: string) => {
                if (await addRespect(nodeId)) get().updateLeaderboard();
            },


            sendContactRequest: async (nodeId: string) => {
                const success = await sendContactRequest(nodeId);
                if (success) get().updateLeaderboard();
            },

            acceptContactRequest: async (nodeId: string) => {
                const success = await acceptContactRequest(nodeId);
                if (success) get().updateLeaderboard();
            },

            removeLeaderboardEntry: async (nodeId: string) => {
                if (await removeLeaderboardEntry(nodeId)) get().updateLeaderboard();
            },

            setDiscoverable: async (discoverable: boolean) => {
                if (await setDiscoverable(discoverable)) {
                    get().updateLeaderboard();
                }
            },

            chat: {
                chat_history: []
            },

            updateChat: async () => {
                const chatMessages = await getChat();
                console.log('Fetched chat state:', chatMessages);
                if (chatMessages) set({ chat: { chat_history: chatMessages.chat_history } });
            },

            clearChatHistory: async () => {
                set({ chat: { chat_history: [] } });
            },

            sendChatMessage: async (content: string) => {
                const { api } = get();
                if (!api) {
                    console.error("API client is not initialized.");
                    return;
                }
        
                const message: ChatMessage = {
                    sender: window.our.node, 
                    content: content,
                    timestamp: Date.now(),
                };
        
                //const terry_packet: ClientRequest = {
                //    type: "ClientRequest",
                //    request: "SendToServer",
                //    data: { "ChatMessage",
                //        data: message,
                //    }
                //};

                const msg_pkt = {"ClientRequest": {"SendToServer": {"ChatMessage": message}}}
                
                await sendChatMessage(msg_pkt);
            },

            receiveChatMessage: (message: ChatMessage) => {
                set((state) => ({
                    chat: {
                        ...state.chat,
                        chat_history: [...state.chat.chat_history, message].slice(-100)
                    }
                }));
            },
            api: null,
            setApi: (newApi) => {
                console.log("setting API in store", newApi);
                set({ api: newApi });
            },
        }),
        {
            name: 'shrine-store',
            storage: createJSONStorage(() => localStorage),
        }
    )
);

export default useShrineStore;