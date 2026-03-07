use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::Message as WsMessage, MaybeTlsStream, WebSocketStream,
};
use uuid::Uuid;

use super::types::*;
use crate::error::AppError;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct OpenClawClient {
    ws_url: String,
    write: SplitSink<WsStream, WsMessage>,
    read: SplitStream<WsStream>,
    agents: Vec<Agent>,
    default_agent_id: Option<String>,
}

impl OpenClawClient {
    /// Connect to the OpenClaw WebSocket Gateway, authenticate, and fetch agents.
    ///
    /// Protocol:
    /// 1. Open WebSocket to ws_url (no token in URL)
    /// 2. Wait for `connect.challenge` event from gateway (contains nonce)
    /// 3. Send `connect` RPC with auth.token in params
    /// 4. On success (hello), call agents.list
    pub async fn connect(ws_url: &str, token: &str) -> Result<(Self, ConnectResult), AppError> {
        // Step 1: Open WebSocket connection with Origin header matching the gateway host.
        // The gateway checks Origin for webchat/control-ui mode.
        // Origin must be just scheme://host:port (no trailing slash, no path).
        let without_scheme = ws_url
            .strip_prefix("wss://")
            .or_else(|| ws_url.strip_prefix("ws://"))
            .unwrap_or(ws_url);
        let host_port = without_scheme.split('/').next().unwrap_or("127.0.0.1:18789");
        let scheme = if ws_url.starts_with("wss://") { "https" } else { "http" };
        let origin = format!("{}://{}", scheme, host_port);

        let request = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(ws_url)
            .header("Host", host_port)
            .header("Origin", &origin)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", tokio_tungstenite::tungstenite::handshake::client::generate_key())
            .body(())
            .map_err(|e| AppError::ConnectionFailed(e.to_string()))?;

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| AppError::ConnectionFailed(e.to_string()))?;

        let (write, read) = ws_stream.split();

        let mut client = Self {
            ws_url: ws_url.to_string(),
            write,
            read,
            agents: Vec::new(),
            default_agent_id: None,
        };

        // Step 2: Wait for connect.challenge event
        let _nonce = client.wait_for_challenge().await?;

        // Step 3: Send connect RPC with auth token
        let connect_params = serde_json::json!({
            "minProtocol": 3,
            "maxProtocol": 3,
            "client": {
                "id": "webchat",
                "version": env!("CARGO_PKG_VERSION"),
                "platform": std::env::consts::OS,
                "mode": "webchat"
            },
            "role": "operator",
            "scopes": ["operator.admin", "operator.approvals", "operator.pairing"],
            "auth": {
                "token": token
            }
        });

        let _hello: serde_json::Value = client
            .rpc("connect", Some(connect_params))
            .await
            .map_err(|e| AppError::ConnectionFailed(format!("Gateway auth failed: {}", e)))?;

        // Step 4: Get agents
        let agents_result: AgentsListResult = client.rpc("agents.list", None).await?;

        // Step 5: Fetch identity (name, emoji) for each agent
        let mut agents_with_identity = agents_result.agents;
        for agent in agents_with_identity.iter_mut() {
            let params = serde_json::json!({ "agentId": agent.id });
            if let Ok(identity) = client.rpc::<AgentIdentity>("agent.identity.get", Some(params)).await {
                if agent.name.is_none() {
                    agent.name = Some(identity.name.clone());
                }
                agent.identity = Some(identity);
            }
        }

        client.agents = agents_with_identity.clone();
        client.default_agent_id = agents_result.default_id.clone();

        let connect_result = ConnectResult {
            agents: agents_with_identity,
            default_id: agents_result.default_id,
        };

        Ok((client, connect_result))
    }

    /// Wait for the gateway's `connect.challenge` event (contains a nonce).
    /// Must be called right after WebSocket open, before sending any RPCs.
    async fn wait_for_challenge(&mut self) -> Result<String, AppError> {
        // Gateway sends challenge within ~1s of connect
        for _ in 0..50 {
            let value = self.read_next_json().await?;

            // Gateway events have { type: "event", event: "...", payload: {...} }
            let event_type = value.get("type").and_then(|v| v.as_str());
            let event_name = value.get("event").and_then(|v| v.as_str());

            if event_type == Some("event") && event_name == Some("connect.challenge") {
                let nonce = value
                    .get("payload")
                    .and_then(|p| p.get("nonce"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                return Ok(nonce);
            }
        }

        Err(AppError::ConnectionFailed(
            "Timed out waiting for connect.challenge from gateway".to_string(),
        ))
    }

    /// Read the next JSON text message from the WebSocket, handling ping/pong.
    async fn read_next_json(&mut self) -> Result<serde_json::Value, AppError> {
        loop {
            let msg = self
                .read
                .next()
                .await
                .ok_or_else(|| {
                    AppError::ConnectionFailed("WebSocket connection closed".to_string())
                })?
                .map_err(|e| AppError::ConnectionFailed(e.to_string()))?;

            match msg {
                WsMessage::Text(text) => {
                    return serde_json::from_str(&text)
                        .map_err(|e| AppError::ParseError(e.to_string()));
                }
                WsMessage::Ping(data) => {
                    self.write
                        .send(WsMessage::Pong(data))
                        .await
                        .map_err(|e| AppError::ConnectionFailed(e.to_string()))?;
                }
                WsMessage::Close(_) => {
                    return Err(AppError::ConnectionFailed(
                        "WebSocket closed by server".to_string(),
                    ));
                }
                _ => {}
            }
        }
    }

    /// Send a JSON-RPC request and wait for the matching response.
    ///
    /// Gateway protocol uses: { type: "req", id, method, params }
    /// Response: { type: "res", id, ok: true, payload } or { type: "res", id, ok: false, error }
    async fn rpc<T: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<T, AppError> {
        let id = Uuid::new_v4().to_string();

        // Gateway expects { type: "req", id, method, params }
        let request = serde_json::json!({
            "type": "req",
            "id": id,
            "method": method,
            "params": params.unwrap_or(serde_json::json!({}))
        });

        let json = serde_json::to_string(&request)
            .map_err(|e| AppError::ParseError(e.to_string()))?;

        self.write
            .send(WsMessage::Text(json.into()))
            .await
            .map_err(|e| AppError::RequestFailed(e.to_string()))?;

        loop {
            let value = self.read_next_json().await?;

            let msg_type = value.get("type").and_then(|v| v.as_str());

            // Skip events while waiting for our response
            if msg_type == Some("event") {
                continue;
            }

            // Check if this is our response
            if msg_type == Some("res") {
                if value.get("id").and_then(|v| v.as_str()) == Some(&id) {
                    let ok = value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);

                    if !ok {
                        let message = value
                            .get("error")
                            .and_then(|e| e.get("message"))
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown RPC error");
                        return Err(AppError::RequestFailed(message.to_string()));
                    }

                    let payload = value.get("payload").cloned().unwrap_or(serde_json::json!({}));

                    return serde_json::from_value(payload)
                        .map_err(|e| AppError::ParseError(e.to_string()));
                }
                // Not our response id, skip
                continue;
            }

            // Unknown message type, skip
        }
    }

    /// Send a chat message and block until the full response is collected.
    pub async fn send_message(
        &mut self,
        agent_id: &str,
        message: &str,
    ) -> Result<AgentReply, AppError> {
        let session_key = format!("agent:{}:main", agent_id);
        let idempotency_key = Uuid::new_v4().to_string();

        let params = serde_json::json!({
            "sessionKey": session_key,
            "message": message,
            "deliver": false,
            "idempotencyKey": idempotency_key
        });

        // Send chat.send RPC and get the ack
        let ack: ChatSendAck = self.rpc("chat.send", Some(params)).await?;

        // Collect streaming events until "final"
        let mut collected = String::new();

        loop {
            let value = self.read_next_json().await?;

            let msg_type = value.get("type").and_then(|v| v.as_str());

            // Chat stream events come as: { type: "event", event: "chat", payload: { state, message, runId, sessionKey } }
            if msg_type != Some("event") {
                continue;
            }

            let event_name = value.get("event").and_then(|v| v.as_str());
            if event_name != Some("chat") {
                continue;
            }

            let payload = match value.get("payload") {
                Some(p) => p,
                None => continue,
            };

            let state = match payload.get("state").and_then(|s| s.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            let run_id = payload
                .get("runId")
                .and_then(|r| r.as_str())
                .unwrap_or("");
            if run_id != ack.run_id {
                continue;
            }

            match state.as_str() {
                "delta" => {
                    // payload.message contains the full accumulated text so far
                    if let Some(msg) = payload.get("message") {
                        // message can be a string or an object with content
                        if let Some(text) = msg.as_str() {
                            collected = text.to_string();
                        } else if let Some(content) = msg.get("content") {
                            // content is array of { type: "text", text: "..." }
                            if let Some(arr) = content.as_array() {
                                let mut text_parts = String::new();
                                for item in arr {
                                    if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                                        if let Some(t) = item.get("text").and_then(|t| t.as_str()) {
                                            text_parts.push_str(t);
                                        }
                                    }
                                }
                                if !text_parts.is_empty() {
                                    collected = text_parts;
                                }
                            }
                        }
                    }
                }
                "final" => {
                    // Final message may contain the complete response
                    if let Some(msg) = payload.get("message") {
                        if let Some(content) = msg.get("content") {
                            if let Some(arr) = content.as_array() {
                                let mut text_parts = String::new();
                                for item in arr {
                                    if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                                        if let Some(t) = item.get("text").and_then(|t| t.as_str()) {
                                            text_parts.push_str(t);
                                        }
                                    }
                                }
                                if !text_parts.is_empty() {
                                    collected = text_parts;
                                }
                            }
                        } else if let Some(text) = msg.as_str() {
                            if !text.is_empty() {
                                collected = text.to_string();
                            }
                        }
                    }

                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);

                    return Ok(AgentReply {
                        agent_id: agent_id.to_string(),
                        reply: collected,
                        timestamp,
                    });
                }
                "aborted" => {
                    return Err(AppError::RequestFailed(
                        "Response was aborted".to_string(),
                    ));
                }
                "error" => {
                    let err_msg = payload
                        .get("errorMessage")
                        .or_else(|| payload.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown streaming error");
                    return Err(AppError::RequestFailed(err_msg.to_string()));
                }
                _ => {}
            }
        }
    }

    /// Check if OpenClaw is still reachable via the health RPC.
    pub async fn health(&mut self) -> Result<bool, AppError> {
        match self.rpc::<serde_json::Value>("health", None).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn default_agent_id(&self) -> Option<&str> {
        self.default_agent_id.as_deref()
    }

    pub fn ws_url(&self) -> &str {
        &self.ws_url
    }
}
