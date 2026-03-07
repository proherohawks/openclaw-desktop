# Architecture — openclaw-desktop

## System Overview

```
┌─────────────────────────────────────────────────────┐
│               Tauri Desktop App                      │
│                                                      │
│  ┌──────────────┐     IPC (invoke)  ┌─────────────┐ │
│  │ React/TS UI  │ <───────────────> │    Rust      │ │
│  │  (WebView)   │                   │   Backend    │ │
│  └──────────────┘                   └──────┬──────┘ │
│                                            │         │
└────────────────────────────────────────────┼─────────┘
                                             │ WebSocket
                                             │ JSON-RPC
                                       ┌─────▼──────┐
                                       │  OpenClaw   │
                                       │  Gateway    │
                                       │ :18789      │
                                       └─────────────┘
```

**Key rule:** The frontend NEVER makes WebSocket or HTTP calls directly. All OpenClaw communication goes through Rust Tauri commands. This is both a security boundary and a single point of control for connection management.

---

## Layer Breakdown

### Frontend (React + TypeScript)
- Thin presentation layer — renders state, dispatches actions
- Zustand store holds: `agents`, `messages`, `connectionState`, `activeAgentId`, `endpoint`, `token`, `sending`
- Store uses `persist` middleware to save `endpoint` and `token` across sessions
- All backend interaction via typed wrappers in `src/lib/api.ts`
- No business logic in components
- Default WebSocket URL defined in `src/lib/constants.ts`: `ws://127.0.0.1:18789`

### IPC Bridge (Tauri Commands)
Commands are in `src-tauri/src/commands/`. Each maps to one frontend need.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `connect` | `endpoint: String, token: String` | `Result<ConnectResult, String>` | Open WS, authenticate, fetch agents |
| `list_agents` | — | `Result<Vec<Agent>, String>` | Return cached agent list |
| `send_message` | `agent_id, message` | `Result<AgentReply, String>` | Send via `chat.send` RPC, collect streaming response |
| `get_status` | — | `Result<ConnectionStatus, String>` | Health check via WS RPC |

### OpenClaw Client (Rust)
Lives in `src-tauri/src/openclaw/`. Uses `tokio-tungstenite` for WebSocket and JSON-RPC communication. Holds the WebSocket split stream and cached agent list as managed Tauri state.

```rust
// src-tauri/src/openclaw/client.rs
pub struct OpenClawClient {
    ws_url: String,
    write: SplitSink<WsStream, WsMessage>,
    read: SplitStream<WsStream>,
    agents: Vec<Agent>,
    default_agent_id: Option<String>,
}

impl OpenClawClient {
    pub async fn connect(ws_url: &str, token: &str) -> Result<(Self, ConnectResult), AppError> { ... }
    pub async fn send_message(&mut self, agent_id: &str, msg: &str) -> Result<AgentReply, AppError> { ... }
    pub async fn health(&mut self) -> Result<bool, AppError> { ... }
    pub fn agents(&self) -> &[Agent] { ... }
    pub fn default_agent_id(&self) -> Option<&str> { ... }
    pub fn ws_url(&self) -> &str { ... }
}
```

**Connection flow:**
1. Open WebSocket to `ws_url`
2. Wait for `connect.challenge` event from gateway
3. Send `connect` RPC with auth token
4. On success (`hello`), call `agents.list` RPC
5. Fetch `agent.identity.get` for each agent (name, emoji)

**Message flow:**
1. Send `chat.send` RPC with `sessionKey`, `message`, `idempotencyKey`
2. Receive `ChatSendAck` with `runId`
3. Collect streaming `chat` events (delta/final) matching `runId`
4. Return collected text as `AgentReply` on `final` event

### Error Handling
All errors flow through `AppError` in `src-tauri/src/error.rs`:

```rust
#[derive(Debug, thiserror::Error, Serialize)]
pub enum AppError {
    #[error("OpenClaw unreachable at {0}. Is it running?")]
    ConnectionFailed(String),
    #[error("Agent '{0}' not found")]
    AgentNotFound(String),
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    #[error("Not connected. Call connect() first.")]
    NotConnected,
}
```

Tauri commands return `Result<T, String>` (String is serialized AppError message).

---

## IPC Command Pattern (Golden Pattern)

Every command follows this exact structure:

```rust
// src-tauri/src/commands/send_message.rs

use tauri::State;
use tokio::sync::Mutex;
use crate::openclaw::{client::OpenClawClient, types::AgentReply};
use crate::error::AppError;

#[tauri::command]
pub async fn send_message(
    agent_id: String,
    message: String,
    state: State<'_, Mutex<Option<OpenClawClient>>>,
) -> Result<AgentReply, String> {
    let mut guard = state.lock().await;
    let client = guard
        .as_mut()
        .ok_or_else(|| AppError::NotConnected.to_string())?;
    client
        .send_message(&agent_id, &message)
        .await
        .map_err(|e| e.to_string())
}
```

> **Why `as_mut()`?** The `OpenClawClient` holds a mutable WebSocket stream. Methods like `send_message` and `health` require `&mut self` because they read/write to the WS connection.

> **Why `tokio::sync::Mutex`?** Commands are async and hold the lock across `.await` points. `std::sync::Mutex` guards are not `Send`, causing compiler errors in async Tauri commands. Use `tokio::sync::Mutex` for all managed state that async commands access.

Frontend TypeScript wrapper:

```typescript
// src/lib/api.ts
import { invoke } from "@tauri-apps/api/core";

export async function sendMessage(agentId: string, message: string): Promise<AgentReply> {
  return invoke<AgentReply>("send_message", { agentId, message });
}
```

---

## Data Models

### Shared Types (mirrored in TS and Rust)

**Rust** (`src-tauri/src/openclaw/types.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    #[serde(default)]
    pub emoji: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub identity: Option<AgentIdentity>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentsListResult {
    pub agents: Vec<Agent>,
    #[serde(rename = "defaultId")]
    pub default_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConnectResult {
    pub agents: Vec<Agent>,
    pub default_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReply {
    pub agent_id: String,
    pub reply: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub endpoint: Option<String>,
}
```

**WebSocket/RPC types** (Rust-only, not mirrored in TS):
```rust
pub struct RpcRequest { id, method, params? }      // Outgoing RPC
pub struct RpcResponse { id?, result?, error? }     // Incoming RPC response
pub struct RpcError { code?, message }              // RPC error detail
pub struct ChatSendParams { session_key, message, deliver, idempotency_key }
pub struct ChatSendAck { run_id, status }           // Ack from chat.send
pub struct StreamEvent { state, message?, session_key?, run_id? }  // Streaming chat events
```

**TypeScript** (`src/lib/types.ts`) — must stay in sync with Rust shared types:
```typescript
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

// Frontend-only type
export interface Message {
  id: string;           // local uuid
  role: "user" | "assistant" | "error";
  text: string;
  ts: number;           // epoch ms
}
```

---

## OpenClaw WebSocket Gateway Protocol

See `docs/design-docs/openclaw-integration.md` for full details.

**TL;DR:**
- Default WebSocket URL: `ws://127.0.0.1:18789`
- Protocol: JSON-RPC over WebSocket with event streaming
- Auth: Token-based via `connect` RPC after `connect.challenge` event
- Requests: `{ type: "req", id, method, params }`
- Responses: `{ type: "res", id, ok: true, payload }` or `{ type: "res", id, ok: false, error }`
- Events: `{ type: "event", event: "chat", payload: { state, message, runId } }`
- Key RPC methods: `connect`, `agents.list`, `agent.identity.get`, `chat.send`, `health`

---

## State Management

Tauri managed state in `src-tauri/src/lib.rs`:

```rust
use tokio::sync::Mutex;

tauri::Builder::default()
    .manage(Mutex::new(Option::<OpenClawClient>::None))
    // ...
```

The `connect` command locks the mutex, creates the client (opening a WebSocket and authenticating), and sets the `Option` to `Some`. All other commands lock, check for `Some`, and return `AppError::NotConnected` if `None`.

Frontend state in `src/lib/store.ts` (Zustand with `persist` middleware):
- Persists `endpoint` and `token` to localStorage across sessions
- Transient state (agents, messages, connectionState) resets on reload
- Per-agent `sending` flag prevents duplicate in-flight requests

---

## Architectural Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03 | Tauri v2 over Electron | Smaller binary, Rust native, no Node.js runtime |
| 2026-03 | Rust handles all WS/HTTP | Security boundary; prevents localhost SSRF from WebView |
| 2026-03 | Zustand over Redux | Simpler API, sufficient for this scale |
| 2026-03 | In-memory history only (v1) | Simplicity; SQLite persistence is v2 feature |
| 2026-03 | WebSocket JSON-RPC over REST | OpenClaw Gateway uses WS protocol with streaming events for chat responses |
| 2026-03 | Token-based auth | Gateway requires auth token on connect; persisted in Zustand store |
| 2026-03 | `tokio::sync::Mutex` for managed state | Async commands hold lock across `.await`; `std::sync::Mutex` guards are not `Send` |
| 2026-03 | `&mut self` on client methods | WebSocket stream is stateful; read/write require mutable access |
| 2026-03 | Persist endpoint + token only | Store uses `partialize` to avoid persisting transient state (messages, agents) |
| 2026-03 | snake_case for IPC wire format | Rust serde defaults to snake_case; TS types match. Gateway uses camelCase for some fields (handled with `#[serde(rename)]`) |
