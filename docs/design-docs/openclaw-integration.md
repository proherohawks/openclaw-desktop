# Design Doc: OpenClaw Integration

## What is OpenClaw?
OpenClaw is a locally-running open-source service that bridges messaging platforms (WhatsApp, Telegram) to LLM backends. It exposes a WebSocket Gateway for real-time communication on a configurable port (default `18789`).

This document defines how `openclaw-desktop` talks to it.

---

## Discovery

The user configures the WebSocket endpoint manually (default `ws://127.0.0.1:18789`). There is no auto-discovery in v1. The Rust client validates the endpoint by completing the WebSocket handshake and authentication flow.

**Future:** mDNS or port scanning for auto-discovery (v2).

---

## WebSocket Gateway Protocol

The OpenClaw Gateway uses a JSON-RPC-style protocol over WebSocket with three message types:

### Message Types
- **Request** (client -> gateway): `{ "type": "req", "id": "<uuid>", "method": "<method>", "params": {...} }`
- **Response** (gateway -> client): `{ "type": "res", "id": "<uuid>", "ok": true, "payload": {...} }` or `{ "type": "res", "id": "<uuid>", "ok": false, "error": { "message": "..." } }`
- **Event** (gateway -> client): `{ "type": "event", "event": "<event-name>", "payload": {...} }`

### Connection Flow
```
Client                          Gateway
  |                                |
  |--- WebSocket handshake ------->|  (Origin header must match gateway host)
  |                                |
  |<-- event: connect.challenge ---|  { nonce: "..." }
  |                                |
  |--- req: connect -------------->|  { auth: { token: "..." }, client: {...}, role: "operator" }
  |<-- res: connect (hello) -------|  { ok: true, payload: {...} }
  |                                |
  |--- req: agents.list ---------->|
  |<-- res: agents.list -----------|  { agents: [...], defaultId: "..." }
  |                                |
  |--- req: agent.identity.get --->|  (for each agent)
  |<-- res: identity --------------|  { name, emoji, avatar? }
```

### RPC Methods

#### `connect`
Authenticates the client with the gateway.
```json
// params
{
  "minProtocol": 3,
  "maxProtocol": 3,
  "client": { "id": "webchat", "version": "0.1.0", "platform": "linux", "mode": "webchat" },
  "role": "operator",
  "scopes": ["operator.admin", "operator.approvals", "operator.pairing"],
  "auth": { "token": "<auth-token>" }
}
```

#### `agents.list`
Returns available agents.
```json
// response payload
{
  "agents": [
    { "id": "agent-1", "name": "My Agent", "model": "gpt-4" }
  ],
  "defaultId": "agent-1"
}
```

#### `agent.identity.get`
Fetches display identity for an agent.
```json
// params
{ "agentId": "agent-1" }
// response payload
{ "name": "Bot", "emoji": "...", "avatar": "..." }
```

#### `chat.send`
Sends a message and initiates streaming response.
```json
// params
{
  "sessionKey": "agent:agent-1:main",
  "message": "Hello!",
  "deliver": false,
  "idempotencyKey": "<uuid>"
}
// response payload (ack)
{ "runId": "run-123", "status": "started" }
```

After the ack, the gateway streams `chat` events:

```json
// delta event
{ "type": "event", "event": "chat", "payload": { "state": "delta", "message": "Hello ", "runId": "run-123", "sessionKey": "agent:agent-1:main" } }

// final event
{ "type": "event", "event": "chat", "payload": { "state": "final", "message": { "content": [{ "type": "text", "text": "Hello world!" }] }, "runId": "run-123" } }
```

Possible `state` values: `delta`, `final`, `aborted`, `error`.

The `message` field in delta events can be a plain string. In final events, it can be a string or an object with `content` array containing `{ type: "text", text: "..." }` items.

#### `health`
Simple health check RPC (no params).

---

## Security Notes
- OpenClaw runs only on localhost — no remote network calls
- The Rust backend is the only thing that constructs WebSocket connections to OpenClaw
- The WebView frontend cannot bypass this (Tauri's CSP prevents arbitrary connections)
- Auth token is required for gateway access; persisted in frontend localStorage via Zustand persist
- Do not log message contents or auth tokens in production builds

---

## Testing the Integration

To test without a running OpenClaw, the project uses a mock WebSocket gateway in `src-tauri/src/openclaw/tests.rs`. The mock:
- Binds to a random local port
- Handles `health`, `agents.list`, and `chat.send` RPCs
- Streams delta + final events for chat messages

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```
