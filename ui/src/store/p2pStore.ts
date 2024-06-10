// P2PStore.ts
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { getLeaderboard, addRespect, setDiscoverable, sendContactRequest, acceptContactRequest, declineContactRequest, removeLeaderboardEntry } from '../api';
import { LeaderboardState } from '../types/TerryLeaderboard';

interface P2PStore {
    leaderboard: LeaderboardState;
    initializeP2PStore: () => Promise<void>;
    updateLeaderboard: () => Promise<void>;
    addRespect: (nodeId: string) => Promise<void>;
    sendContactRequest: (nodeId: string) => Promise<void>;
    acceptContactRequest: (nodeId: string) => Promise<void>;
    removeLeaderboardEntry: (nodeId: string) => Promise<void>;
    setDiscoverable: (discoverable: boolean) => Promise<void>;
}

const useP2PStore = create<P2PStore>()(
    persist(
        (set, get) => ({
            leaderboard: {
                node_id: "",
                discoverable: false, 
                contacts: [], 
                stats: {},
                pending_contact_requests: [],
                incoming_contact_requests: [],
            },
            initializeP2PStore: async () => {
                console.log('Initializing p2p Store...');
                await get().updateLeaderboard();
                console.log('p2p Store initialized');
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
                if (await sendContactRequest(nodeId)) get().updateLeaderboard();
            },
            acceptContactRequest: async (nodeId: string) => {
                if (await acceptContactRequest(nodeId)) get().updateLeaderboard();
            },
            removeLeaderboardEntry: async (nodeId: string) => {
                if (await removeLeaderboardEntry(nodeId)) get().updateLeaderboard();
            },
            setDiscoverable: async (discoverable: boolean) => {
                if (await setDiscoverable(discoverable)) {
                    get().updateLeaderboard();
                }
            },
        }),
        {
            name: 'p2p-store',
            storage: createJSONStorage(() => localStorage),
        }
    )
);

export default useP2PStore;
