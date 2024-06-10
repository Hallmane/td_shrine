// ChatStore.ts
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { Chat, ChatMessage } from '../types/TerryLeaderboard';
import { getChat, sendChatMessage } from '../api';
import KinodeClientApi from "@kinode/client-api";

interface ChatStore {
    chat: Chat;
    updateChat: () => Promise<void>;
    clearChatHistory: () => void;
    sendChatMessage: (content: string) => Promise<void>;
    receiveChatMessage: (message: ChatMessage) => void;
    api: KinodeClientApi | null;
    setApi: (api: KinodeClientApi) => void;
}

const useChatStore = create<ChatStore>()(
    persist(
        (set) => ({
            chat: {
                chat_history: [],
            },
            updateChat: async () => {
                const chatMessages = await getChat();
                console.log('Fetched chat state:', chatMessages);
                if (chatMessages) set({ chat: { chat_history: chatMessages.chat_history } });
            },
            clearChatHistory: () => {
                set({ chat: { chat_history: [] } });
            },
            sendChatMessage: async (content: string) => {
                const message: ChatMessage = {
                    sender: window.our.node, 
                    content: content,
                    timestamp: Date.now(),
                };
                const msg_pkt = {"ClientRequest": {"SendToServer": {"ChatMessage": message}}};
                await sendChatMessage(msg_pkt);
            },
            receiveChatMessage: (message: ChatMessage) => {
                set(state => ({
                    chat: {
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
            name: 'chat-store',
            storage: createJSONStorage(() => sessionStorage), // Use sessionStorage for temporary chat history
        }
    )
);

export default useChatStore;
