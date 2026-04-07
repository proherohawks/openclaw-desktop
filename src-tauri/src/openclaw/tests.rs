use super::client::OpenClawClient;
use super::types::*;
use crate::error::AppError;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message as WsMessage;

// --- Serde round-trip tests ---

#[test]
fn agent_serde_round_trip() {
    let agent = Agent {
        id: "a1".into(),
        name: Some("Agent One".into()),
        model: Some("gpt-4".into()),
        identity: Some(AgentIdentity {
            emoji: Some("\u{1F916}".into()),
            avatar: None,
            name: "Bot".into(),
        }),
    };
    let json = serde_json::to_string(&agent).unwrap();
    let deserialized: Agent = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, agent.id);
    assert_eq!(deserialized.name, agent.name);
}

#[test]
fn agent_reply_serde_round_trip() {
    let reply = AgentReply {
        agent_id: "a1".into(),
        reply: "Hello there!".into(),
        timestamp: 1741234567890,
    };
    let json = serde_json::to_string(&reply).unwrap();
    let deserialized: AgentReply = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.agent_id, reply.agent_id);
    assert_eq!(deserialized.reply, reply.reply);
    assert_eq!(deserialized.timestamp, reply.timestamp);
}

#[test]
fn rpc_request_omits_null_params() {
    let req = RpcRequest {
        id: "1".into(),
        method: "health".into(),
        params: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(!json.contains("params"));
}

#[test]
fn stream_event_deserializes() {
    let json =
        r#"{"state":"delta","message":"Hi","sessionKey":"agent:a1:main","runId":"run-1"}"#;
    let event: StreamEvent = serde_json::from_str(json).unwrap();
    assert_eq!(event.state, "delta");
    assert_eq!(event.message, Some("Hi".to_string()));
    assert_eq!(event.run_id, Some("run-1".to_string()));
}

// --- AppError display tests ---

#[test]
fn app_error_connection_failed_display() {
    let err = AppError::ConnectionFailed("timeout".into());
    assert_eq!(
        err.to_string(),
        "OpenClaw unreachable at timeout. Is it running?"
    );
}

#[test]
fn app_error_agent_not_found_display() {
    let err = AppError::AgentNotFound("wa-99".into());
    assert_eq!(err.to_string(), "Agent 'wa-99' not found");
}

#[test]
fn app_error_request_failed_display() {
    let err = AppError::RequestFailed("RPC error".into());
    assert_eq!(err.to_string(), "Request failed: RPC error");
}

#[test]
fn app_error_parse_error_display() {
    let err = AppError::ParseError("invalid json".into());
    assert_eq!(
        err.to_string(),
        "Failed to parse response: invalid json"
    );
}

#[test]
fn app_error_not_connected_display() {
    let err = AppError::NotConnected;
    assert_eq!(err.to_string(), "Not connected. Call connect() first.");
}

// --- Mock WebSocket Gateway ---

/// Spin up a mock WS server that handles health, agents.list, and chat.send.
async fn start_mock_gateway() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://{}", addr);

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
        let (mut write, mut read) = ws_stream.split();

        // Send connect.challenge event immediately
        let challenge = serde_json::json!({
            "type": "event",
            "event": "connect.challenge",
            "payload": { "nonce": "mock-nonce-12345" }
        });
        write
            .send(WsMessage::Text(challenge.to_string().into()))
            .await
            .unwrap();

        while let Some(Ok(msg)) = read.next().await {
            if let WsMessage::Text(text) = msg {
                let req: serde_json::Value =
                    serde_json::from_str(&text).unwrap();
                let id = req["id"].as_str().unwrap_or("").to_string();
                let method = req["method"].as_str().unwrap_or("");

                match method {
                    "connect" => {
                        let resp = serde_json::json!({
                            "type": "res",
                            "id": id,
                            "ok": true,
                            "payload": { "protocol": 3 }
                        });
                        write
                            .send(WsMessage::Text(resp.to_string().into()))
                            .await
                            .unwrap();
                    }
                    "health" => {
                        let resp = serde_json::json!({
                            "type": "res",
                            "id": id,
                            "ok": true,
                            "payload": { "status": "ok" }
                        });
                        write
                            .send(WsMessage::Text(resp.to_string().into()))
                            .await
                            .unwrap();
                    }
                    "agents.list" => {
                        let resp = serde_json::json!({
                            "type": "res",
                            "id": id,
                            "ok": true,
                            "payload": {
                                "agents": [{
                                    "id": "test-1",
                                    "name": "Test Agent",
                                    "model": "gpt-4"
                                }],
                                "defaultId": "test-1"
                            }
                        });
                        write
                            .send(WsMessage::Text(resp.to_string().into()))
                            .await
                            .unwrap();
                    }
                    "agent.identity.get" => {
                        let resp = serde_json::json!({
                            "type": "res",
                            "id": id,
                            "ok": true,
                            "payload": {
                                "name": "Test Agent",
                                "emoji": "\u{1F916}"
                            }
                        });
                        write
                            .send(WsMessage::Text(resp.to_string().into()))
                            .await
                            .unwrap();
                    }
                    "chat.send" => {
                        let run_id = "run-mock-1";
                        // Ack
                        let ack = serde_json::json!({
                            "type": "res",
                            "id": id,
                            "ok": true,
                            "payload": { "runId": run_id, "status": "started" }
                        });
                        write
                            .send(WsMessage::Text(ack.to_string().into()))
                            .await
                            .unwrap();

                        // Delta
                        let delta = serde_json::json!({
                            "type": "event",
                            "event": "chat",
                            "payload": {
                                "state": "delta",
                                "message": "Hello ",
                                "sessionKey": "agent:test-1:main",
                                "runId": run_id
                            }
                        });
                        write
                            .send(WsMessage::Text(delta.to_string().into()))
                            .await
                            .unwrap();

                        // Final
                        let final_ev = serde_json::json!({
                            "type": "event",
                            "event": "chat",
                            "payload": {
                                "state": "final",
                                "message": "Hello world!",
                                "sessionKey": "agent:test-1:main",
                                "runId": run_id
                            }
                        });
                        write
                            .send(WsMessage::Text(final_ev.to_string().into()))
                            .await
                            .unwrap();
                    }
                    _ => {
                        let resp = serde_json::json!({
                            "type": "res",
                            "id": id,
                            "ok": false,
                            "error": { "message": format!("Unknown method: {}", method) }
                        });
                        write
                            .send(WsMessage::Text(resp.to_string().into()))
                            .await
                            .unwrap();
                    }
                }
            }
        }
    });

    url
}

// --- Integration tests ---

#[tokio::test]
async fn connect_success_returns_agents() {
    let url = start_mock_gateway().await;
    let (client, result) = OpenClawClient::connect(&url, "test-token")
        .await
        .expect("connect should succeed");

    assert_eq!(result.agents.len(), 1);
    assert_eq!(result.agents[0].id, "test-1");
    assert_eq!(result.agents[0].name, Some("Test Agent".to_string()));
    assert_eq!(result.default_id, Some("test-1".to_string()));
    assert_eq!(client.agents().len(), 1);
    assert_eq!(client.ws_url(), url);
}

#[tokio::test]
async fn connect_with_bad_url_returns_connection_failed() {
    let result = OpenClawClient::connect("ws://127.0.0.1:1", "token").await;
    match result {
        Err(AppError::ConnectionFailed(_)) => {}
        Err(other) => panic!("Expected ConnectionFailed, got: {other:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[tokio::test]
async fn send_message_collects_deltas_until_final() {
    let url = start_mock_gateway().await;
    let (mut client, _) = OpenClawClient::connect(&url, "test-token")
        .await
        .expect("connect should succeed");

    let reply = client
        .send_message("test-1", "Hello!")
        .await
        .expect("send_message should succeed");

    assert_eq!(reply.agent_id, "test-1");
    assert_eq!(reply.reply, "Hello world!");
}

#[tokio::test]
async fn send_message_unescapes_newlines_for_markdown() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://{}", addr);

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
        let (mut write, mut read) = ws_stream.split();

        let challenge = serde_json::json!({
            "type": "event",
            "event": "connect.challenge",
            "payload": { "nonce": "mock-nonce-12345" }
        });
        write
            .send(WsMessage::Text(challenge.to_string().into()))
            .await
            .unwrap();

        while let Some(Ok(msg)) = read.next().await {
            if let WsMessage::Text(text) = msg {
                let req: serde_json::Value = serde_json::from_str(&text).unwrap();
                let id = req["id"].as_str().unwrap_or("").to_string();
                let method = req["method"].as_str().unwrap_or("");

                match method {
                    "connect" => {
                        let resp = serde_json::json!({
                            "type": "res",
                            "id": id,
                            "ok": true,
                            "payload": { "protocol": 3 }
                        });
                        write.send(WsMessage::Text(resp.to_string().into())).await.unwrap();
                    }
                    "agents.list" => {
                        let resp = serde_json::json!({
                            "type": "res",
                            "id": id,
                            "ok": true,
                            "payload": {
                                "agents": [{"id": "test-1", "name": "Test Agent", "model": "gpt-4"}],
                                "defaultId": "test-1"
                            }
                        });
                        write.send(WsMessage::Text(resp.to_string().into())).await.unwrap();
                    }
                    "agent.identity.get" => {
                        let resp = serde_json::json!({
                            "type": "res",
                            "id": id,
                            "ok": true,
                            "payload": { "name": "Test Agent", "emoji": "🤖" }
                        });
                        write.send(WsMessage::Text(resp.to_string().into())).await.unwrap();
                    }
                    "chat.send" => {
                        let run_id = "run-markdown-1";
                        let ack = serde_json::json!({
                            "type": "res",
                            "id": id,
                            "ok": true,
                            "payload": { "runId": run_id, "status": "started" }
                        });
                        write.send(WsMessage::Text(ack.to_string().into())).await.unwrap();

                        let final_ev = serde_json::json!({
                            "type": "event",
                            "event": "chat",
                            "payload": {
                                "state": "final",
                                "message": "## Title\\n\\n```ts\\nconst x = 1;\\n```",
                                "sessionKey": "agent:test-1:main",
                                "runId": run_id
                            }
                        });
                        write.send(WsMessage::Text(final_ev.to_string().into())).await.unwrap();
                    }
                    _ => {
                        let resp = serde_json::json!({
                            "type": "res",
                            "id": id,
                            "ok": true,
                            "payload": {}
                        });
                        write.send(WsMessage::Text(resp.to_string().into())).await.unwrap();
                    }
                }
            }
        }
    });

    let (mut client, _) = OpenClawClient::connect(&url, "test-token")
        .await
        .expect("connect should succeed");

    let reply = client
        .send_message("test-1", "show markdown")
        .await
        .expect("send_message should succeed");

    assert_eq!(reply.reply, "## Title\n\n```ts\nconst x = 1;\n```");
}

#[tokio::test]
async fn health_returns_true_when_connected() {
    let url = start_mock_gateway().await;
    let (mut client, _) = OpenClawClient::connect(&url, "test-token")
        .await
        .expect("connect should succeed");

    let healthy = client.health().await.expect("health should succeed");
    assert!(healthy);
}
