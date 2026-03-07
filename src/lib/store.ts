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
        }),

      setActiveAgent: (id) => set({ activeAgentId: id }),

      sendMessage: async (text) => {
        const { activeAgentId, messages, sending } = get();
        if (!activeAgentId || sending[activeAgentId]) return;

        const userMsg: Message = {
          id: uuidv4(),
          role: "user",
          text,
          ts: Date.now(),
        };

        set({
          messages: {
            ...messages,
            [activeAgentId]: [...(messages[activeAgentId] ?? []), userMsg],
          },
          sending: { ...sending, [activeAgentId]: true },
        });

        try {
          const reply = await api.sendMessage(activeAgentId, text);
          const assistantMsg: Message = {
            id: uuidv4(),
            role: "assistant",
            text: reply.reply,
            ts: reply.timestamp,
          };
          set((state) => ({
            messages: {
              ...state.messages,
              [activeAgentId]: [...(state.messages[activeAgentId] ?? []), assistantMsg],
            },
            sending: { ...state.sending, [activeAgentId]: false },
          }));
        } catch (err) {
          const errorMsg: Message = {
            id: uuidv4(),
            role: "error",
            text: err instanceof Error ? err.message : String(err),
            ts: Date.now(),
          };
          set((state) => ({
            messages: {
              ...state.messages,
              [activeAgentId]: [...(state.messages[activeAgentId] ?? []), errorMsg],
            },
            sending: { ...state.sending, [activeAgentId]: false },
          }));
        }
      },

      clearMessages: (agentId) =>
        set((state) => ({
          messages: { ...state.messages, [agentId]: [] },
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
