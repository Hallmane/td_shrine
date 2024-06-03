import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { Chat, ChatMessage, LeaderboardState } from '../types/TerryLeaderboard';
import { getLeaderboard, getChat, addRespect, setDiscoverable, sendContactRequest, removeLeaderboardEntry, acceptContactRequest, declineContactRequest, sendChatMessage } from '../api';

interface ShrineStore {
    leaderboard: LeaderboardState;
    chat: Chat;
    initializeStore: () => Promise<void>;
    updateLeaderboard: () => Promise<void>;
    updateChat: () => Promise<void>;
    clearChatHistory: () => Promise<void>;
    addRespect: (nodeId: string) => Promise<void>;
    acceptContactRequest: (nodeId: string) => Promise<void>;
    removeLeaderboardEntry: (nodeId: string) => Promise<void>;
    setDiscoverable: (discoverable: boolean) => Promise<void>;
    sendContactRequest: (nodeId: string) => Promise<void>;
    sendChatMessage: (content: string) => Promise<void>;
    receiveChatMessage: (message: ChatMessage) => void;
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
            chat: {
                chat_history: []
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

            updateChat: async () => {
                const chatMessages = await getChat();
                console.log('Fetched chat state:', chatMessages);
                if (chatMessages) set({ chat: { chat_history: chatMessages.chat_history } });
            },

            clearChatHistory: async () => {
                set({ chat: { chat_history: [] } });
            },

            addRespect: async (nodeId: string) => {
                if (await addRespect(nodeId)) get().updateLeaderboard();
            },

            setDiscoverable: async (discoverable: boolean) => {
                if (await setDiscoverable(discoverable)) {
                    get().updateLeaderboard();
                }
            },

            sendContactRequest: async (nodeId: string) => {
                const success = await sendContactRequest(nodeId);
                if (success) get().updateLeaderboard();
            },

            acceptContactRequest: async (nodeId: string) => {
                const success = await acceptContactRequest(nodeId);
                if (success) get().updateLeaderboard();
            },

            declineContactRequest: async (nodeId: string) => {
                const success = await declineContactRequest(nodeId);
                if (success) get().updateLeaderboard();
            },

            removeLeaderboardEntry: async (nodeId: string) => {
                if (await removeLeaderboardEntry(nodeId)) get().updateLeaderboard();
            },

            sendChatMessage: async (content: string) => {
                const success = await sendChatMessage(content);
                if (success) {
                    const newChatMessage: ChatMessage = { 
                        sender: get().leaderboard.node_id,
                        content,
                        timestamp: Date.now(),
                    };
                    set(state => ({
                        chat: {
                            chat_history: [...state.chat.chat_history, newChatMessage].slice(-100)
                        }
                    }));
                }
            },

            receiveChatMessage: (message: ChatMessage) => {
                set(state => ({
                    chat: {
                        chat_history: [...state.chat.chat_history, message].slice(-100)
                    }
                }));
            }
        }),
        {
            name: 'shrine-store',
            storage: createJSONStorage(() => localStorage),
        }
    )
);

export default useShrineStore;





//import { create } from 'zustand';
//import { persist, createJSONStorage } from 'zustand/middleware';
//import { LeaderboardState, ChatMessage } from '../types/TerryLeaderboard';
//import { getLeaderboard, addRespect, setDiscoverable, sendContactRequest, removeLeaderboardEntry, acceptContactRequest, declineContactRequest, sendChatMessage } from '../api';
//
//interface ShrineStore extends LeaderboardState {
//    initializeStore: () => Promise<void>;
//    updateLeaderboard: () => Promise<void>;
//    addRespect: (nodeId: string) => Promise<void>;
//    acceptContactRequest: (nodeId: string) => Promise<void>;
//    removeLeaderboardEntry: (nodeId: string) => Promise<void>;
//    setDiscoverable: (discoverable: boolean) => Promise<void>;
//    sendContactRequest: (nodeId: string) => Promise<void>;
//    sendChatMessage: (content: string) => Promise<void>;
//    receiveChatMessage: (message: ChatMessage) => void;
//}
//
//const useShrineStore = create<ShrineStore>()(
//    persist(
//        (set, get) => ({
//            node_id: "",
//            discoverable: false,
//            contacts: [],
//            stats: {},
//            pending_contact_requests: [],
//            incoming_contact_requests: [],
//            chat_history: [],
//
//            initializeStore: async () => {
//                console.log('Initializing store...');
//                await get().updateLeaderboard();
//                console.log('Store initialized');
//            },
//
//            updateLeaderboard: async () => {
//                const state = await getLeaderboard();
//                console.log('Fetched leaderboard state:', state);
//                if (state) set(state);
//            },
//
//            addRespect: async (nodeId: string) => {
//                if (await addRespect(nodeId)) get().updateLeaderboard();
//            },
//
//            setDiscoverable: async (discoverable: boolean) => {
//                if (await setDiscoverable(get().node_id, discoverable)) {
//                    set({ discoverable });
//                    get().updateLeaderboard();
//                }
//            },
//
//            sendContactRequest: async (nodeId: string) => {
//                const success = await sendContactRequest(nodeId);
//                if (success) get().updateLeaderboard();
//            },
//
//            acceptContactRequest: async (nodeId: string) => {
//                const success = await acceptContactRequest(nodeId);
//                if (success) get().updateLeaderboard();
//            },
//
//            declineContactRequest: async (nodeId: string) => {
//                const success = await declineContactRequest(nodeId);
//                if (success) get().updateLeaderboard();
//            },
//
//            removeLeaderboardEntry: async (nodeId: string) => {
//                if (await removeLeaderboardEntry(nodeId)) get().updateLeaderboard();
//            },
//
//            sendChatMessage: async (content: string) => {
//                const success = await sendChatMessage(content);
//                if (success) {
//                    const newChatMessage: ChatMessage = {
//                        sender: get().node_id,
//                        content,
//                        timestamp: Date.now(),
//                    };
//                    set(state => ({
//                        chat: {
//                            ...state.chat,
//                            chat_history: [...state.chat.chat_history, newChatMessage].slice(-100)
//                        }
//                    }));
//                }
//            },
//
//            receiveChatMessage: (message: ChatMessage) => {
//                set(state => ({
//                    chat: {
//                        ...state.chat,
//                        chat_history: [...state.chat.chat_history, message].slice(-100)
//                    }
//                }));
//            }
//        }),
//        {
//            name: 'shrine-store',
//            storage: createJSONStorage(() => localStorage),
//        }
//    )
//);
//
//export default useShrineStore;
//

//import { create } from 'zustand';
//import { persist, createJSONStorage } from 'zustand/middleware';
//import { LeaderboardState, ChatMessage, Chat } from '../types/TerryLeaderboard';
//import { getLeaderboard, addRespect, setDiscoverable, sendContactRequest, removeLeaderboardEntry, acceptContactRequest, declineContactRequest, sendChatMessage } from '../api';
//
//interface ShrineStore extends LeaderboardState {
//    initializeStore: () => Promise<void>;
//    updateLeaderboard: () => Promise<void>;
//    clearChatHistory: () => void;
//    addRespect: (nodeId: string) => Promise<void>;
//    acceptContactRequest: (nodeId: string) => Promise<void>;
//    removeLeaderboardEntry: (nodeId: string) => Promise<void>;
//    setDiscoverable: (discoverable: boolean) => Promise<void>;
//    sendContactRequest: (nodeId: string) => Promise<void>;
//    sendChatMessage: (content: string) => Promise<void>;
//    receiveChatMessage: (message: ChatMessage) => void;
//}
//
//const useShrineStore = create<ShrineStore>()(
//    persist(
//        (set, get) => ({
//            node_id: "",
//            discoverable: false,
//            contacts: [],
//            stats: {},
//            pending_contact_requests: [],
//            incoming_contact_requests: [],
//            chat: {
//                chat_history: []
//            },
//
//            initializeStore: async () => {
//                console.log('Initializing store...');
//                await get().updateLeaderboard();
//                console.log('Store initialized');
//            },
//
//            updateLeaderboard: async () => {
//                const state = await getLeaderboard();
//                console.log('Fetched leaderboard state:', state);
//                if (state) set(state);
//            },
//
//            clearChatHistory: () => {
//                set(state => ({
//                    chat: {
//                        ...state.chat,
//                        chat_history: []
//                    }
//                }));
//            },
//
//            addRespect: async (nodeId: string) => {
//                if (await addRespect(nodeId)) get().updateLeaderboard();
//            },
//
//            setDiscoverable: async (discoverable: boolean) => {
//                if (await setDiscoverable(discoverable)) {
//                    set({ discoverable });
//                    get().updateLeaderboard();
//                }
//            },
//
//            sendContactRequest: async (nodeId: string) => {
//                const success = await sendContactRequest(nodeId);
//                if (success) get().updateLeaderboard();
//            },
//
//            acceptContactRequest: async (nodeId: string) => {
//                const success = await acceptContactRequest(nodeId);
//                if (success) get().updateLeaderboard();
//            },
//
//            declineContactRequest: async (nodeId: string) => {
//                const success = await declineContactRequest(nodeId);
//                if (success) get().updateLeaderboard();
//            },
//
//            removeLeaderboardEntry: async (nodeId: string) => {
//                if (await removeLeaderboardEntry(nodeId)) get().updateLeaderboard();
//            },
//
//            sendChatMessage: async (content: string) => {
//                const success = await sendChatMessage(content);
//                if (success) {
//                    const newChatMessage: ChatMessage = {
//                        sender: get().node_id,
//                        content,
//                        timestamp: Date.now(),
//                    };
//                    set(state => ({
//                        chat: {
//                            ...state.chat,
//                            chat_history: [...state.chat.chat_history, newChatMessage].slice(-100)
//                        }
//                    }));
//                }
//            },
//
//            receiveChatMessage: (message: ChatMessage) => {
//                set(state => ({
//                    chat: {
//                        ...state.chat,
//                        chat_history: [...state.chat.chat_history, message].slice(-100)
//                    }
//                }));
//            }
//        }),
//        {
//            name: 'shrine-store',
//            storage: createJSONStorage(() => localStorage),
//        }
//    )
//);
//
//export default useShrineStore;


