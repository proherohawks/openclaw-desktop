import { create } from "zustand";
import { persist } from "zustand/middleware";
import { v4 as uuidv4 } from "uuid";
import * as api from "./api";
import type { Agent, Message } from "./types";
import { DEFAULT_WS_URL } from "./constants";

type ConnectionState = "disconnected" | "connecting" | "connected" | "error";

interface AppState {
  // Connection
  endpoint: string;
  token: string;
  connectionState: ConnectionState;
  connectionError: string | null;

  // Agents
  agents: Agent[];
  activeAgentId: string | null;

  // Messages — keyed by agent id
  messages: Record<string, Message[]>;

  // Visual notification state per agent
  unreadCounts: Record<string, number>;

  // Loading per agent
  sending: Record<string, boolean>;

  // Actions
  setEndpoint: (ep: string) => void;
  setToken: (token: string) => void;
  connect: (endpoint: string, token: string) => Promise<void>;
  disconnect: () => void;
  setActiveAgent: (id: string) => void;
  sendMessage: (text: string) => Promise<void>;
  clearMessages: (agentId: string) => void;
}

export const useStore = create<AppState>()(
  persist(
    (set, get) => ({
      endpoint: DEFAULT_WS_URL,
      token: "",
      connectionState: "disconnected",
      connectionError: null,
      agents: [],
      activeAgentId: null,
      messages: {},
      unreadCounts: {},
      sending: {},

      setEndpoint: (ep) => set({ endpoint: ep }),

      setToken: (token) => set({ token }),

      connect: async (endpoint, token) => {
        set({ connectionState: "connecting", connectionError: null });
        try {
          const result = await api.connect(endpoint, token);
          set({
            connectionState: "connected",
            agents: result.agents,
            endpoint,
            token,
            activeAgentId: result.default_id ?? result.agents[0]?.id ?? null,
            unreadCounts: {},
          });
        } catch (err) {
          set({
            connectionState: "error",
            connectionError: err instanceof Error ? err.message : String(err),
          });
        }
      },

      disconnect: () =>
        set({
          connectionState: "disconnected",
          agents: [],
          activeAgentId: null,
          connectionError: null,
          unreadCounts: {},
        }),

      setActiveAgent: (id) =>
        set((state) => ({
          activeAgentId: id,
          unreadCounts: { ...state.unreadCounts, [id]: 0 },
        })),

      sendMessage: async (text) => {
        const { activeAgentId, messages, sending } = get();
        if (!activeAgentId || sending[activeAgentId]) return;

        const targetAgentId = activeAgentId;

        const userMsg: Message = {
          id: uuidv4(),
          role: "user",
          text,
          ts: Date.now(),
        };

        set({
          messages: {
            ...messages,
            [targetAgentId]: [...(messages[targetAgentId] ?? []), userMsg],
          },
          sending: { ...sending, [targetAgentId]: true },
        });

        try {
          const reply = await api.sendMessage(targetAgentId, text);
          const assistantMsg: Message = {
            id: uuidv4(),
            role: "assistant",
            text: reply.reply,
            ts: reply.timestamp,
          };
          set((state) => {
            const isUnread = get().activeAgentId !== targetAgentId;
            return {
              messages: {
                ...state.messages,
                [targetAgentId]: [...(state.messages[targetAgentId] ?? []), assistantMsg],
              },
              unreadCounts: {
                ...state.unreadCounts,
                [targetAgentId]: isUnread ? (state.unreadCounts[targetAgentId] ?? 0) + 1 : 0,
              },
              sending: { ...state.sending, [targetAgentId]: false },
            };
          });
        } catch (err) {
          const errorMsg: Message = {
            id: uuidv4(),
            role: "error",
            text: err instanceof Error ? err.message : String(err),
            ts: Date.now(),
          };
          set((state) => {
            const isUnread = get().activeAgentId !== targetAgentId;
            return {
              messages: {
                ...state.messages,
                [targetAgentId]: [...(state.messages[targetAgentId] ?? []), errorMsg],
              },
              unreadCounts: {
                ...state.unreadCounts,
                [targetAgentId]: isUnread ? (state.unreadCounts[targetAgentId] ?? 0) + 1 : 0,
              },
              sending: { ...state.sending, [targetAgentId]: false },
            };
          });
        }
      },

      clearMessages: (agentId) =>
        set((state) => ({
          messages: { ...state.messages, [agentId]: [] },
          unreadCounts: { ...state.unreadCounts, [agentId]: 0 },
        })),
    }),
    {
      name: "openclaw-desktop",
      // Only persist connection credentials — not transient state
      partialize: (state) => ({
        endpoint: state.endpoint,
        token: state.token,
      }),
    },
  ),
);
