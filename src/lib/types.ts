// Must stay in sync with src-tauri/src/openclaw/types.rs
// Any changes to Rust types require updates here too.

export interface AgentIdentity {
  emoji?: string;
  avatar?: string;
  name: string;
}

export interface Agent {
  id: string;
  name?: string;
  model?: string;
  identity?: AgentIdentity;
}

export interface ConnectResult {
  agents: Agent[];
  default_id: string | null;
}

export interface AgentReply {
  agent_id: string;
  reply: string;
  timestamp: number;
}

export interface ConnectionStatus {
  connected: boolean;
  endpoint: string | null;
}

// Frontend-only type — not in Rust
export interface Message {
  id: string;                                   // local uuid
  role: "user" | "assistant" | "error";
  text: string;
  ts: number;                                   // epoch ms
}
