import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { LeaderboardState, ClientRequest } from '../types/TerryLeaderboard';
import { getLeaderboard, addRespect, setDiscoverable, sendContactRequest, removeLeaderboardEntry, acceptContactRequest, declineContactRequest } from '../api';
import KinodeClientApi from "@kinode/client-api";
import { Leaderboard } from '../leaderboard';

interface ShrineStore {
    leaderboard: LeaderboardState;
    setLeaderboard: (leaderboard) => Promise<void>;
    initializeStore: () => Promise<void>;
    updateLeaderboard: () => Promise<void>;
    addRespect: (nodeId: string) => Promise<void>;
    sendContactRequest: (nodeId: string) => Promise<void>;
    acceptContactRequest: (nodeId: string) => Promise<void>;
    removeLeaderboardEntry: (nodeId: string) => Promise<void>;
    setDiscoverable: (discoverable: boolean) => Promise<void>;
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
            }, 
            setLeaderboard: async(leaderboard) => set({ leaderboard }),
            initializeStore: async () => {
                console.log('Initializing store...');
                await Promise.all([get().updateLeaderboard(), ]);
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