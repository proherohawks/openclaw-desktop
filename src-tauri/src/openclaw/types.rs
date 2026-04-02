use serde::{Deserialize, Serialize};

// --- Agent types (from agents.list response) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    #[serde(default)]
    pub emoji: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    pub name: String,
}

/// Deserialize model field that can be either a string ("anthropic/claude-opus-4-6")
/// or an object ({"primary": "anthropic/claude-opus-4-6"}).
fn deserialize_model_field<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde_json::Value;
    let v: Option<Value> = Option::deserialize(deserializer)?;
    match v {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Object(map)) => {
            // Extract "primary" field from model object
            Ok(map.get("primary").and_then(|v| v.as_str()).map(String::from))
        }
        Some(_) => Ok(None),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_model_field")]
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

// --- Connect result (returned to frontend) ---

#[derive(Debug, Serialize)]
pub struct ConnectResult {
    pub agents: Vec<Agent>,
    pub default_id: Option<String>,
}

// --- Agent reply (returned to frontend) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReply {
    pub agent_id: String,
    pub reply: String,
    pub timestamp: u64,
}

// --- Connection status ---

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub endpoint: Option<String>,
}

// --- JSON-RPC over WebSocket ---

#[derive(Debug, Serialize)]
pub struct RpcRequest {
    pub id: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RpcResponse {
    pub id: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
pub struct RpcError {
    pub code: Option<i32>,
    pub message: String,
}

// --- Chat types ---

#[derive(Debug, Serialize)]
pub struct ChatSendParams {
    #[serde(rename = "sessionKey")]
    pub session_key: String,
    pub message: String,
    pub deliver: bool,
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatSendAck {
    #[serde(rename = "runId")]
    pub run_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct StreamEvent {
    pub state: String,
    pub message: Option<String>,
    #[serde(rename = "sessionKey")]
    pub session_key: Option<String>,
    #[serde(rename = "runId")]
    pub run_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_serializes_correctly() {
        let agent = Agent {
            id: "agent-1".into(),
            name: Some("My Agent".into()),
            model: Some("gpt-4".into()),
            identity: Some(AgentIdentity {
                emoji: Some("\u{1F916}".into()),
                avatar: None,
                name: "Bot".into(),
            }),
        };
        let json = serde_json::to_string(&agent).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"model\""));
        assert!(json.contains("\"identity\""));
    }

    #[test]
    fn agent_deserializes_without_optional_fields() {
        let json = r#"{"id":"agent-1","name":"My Agent"}"#;
        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.id, "agent-1");
        assert!(agent.model.is_none());
        assert!(agent.identity.is_none());
    }

    #[test]
    fn agents_list_result_round_trips() {
        let json = r#"{"agents":[{"id":"a1","name":"Agent One"}],"defaultId":"a1"}"#;
        let result: AgentsListResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.agents.len(), 1);
        assert_eq!(result.default_id, Some("a1".to_string()));
    }

    #[test]
    fn agent_reply_round_trips() {
        let reply = AgentReply {
            agent_id: "a1".into(),
            reply: "hello".into(),
            timestamp: 1741234567890,
        };
        let json = serde_json::to_string(&reply).unwrap();
        let back: AgentReply = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, reply.agent_id);
        assert_eq!(back.reply, reply.reply);
        assert_eq!(back.timestamp, reply.timestamp);
    }

    #[test]
    fn rpc_request_serializes_without_params() {
        let req = RpcRequest {
            id: "1".into(),
            method: "health".into(),
            params: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("params"));
    }

    #[test]
    fn rpc_request_serializes_with_params() {
        let req = RpcRequest {
            id: "1".into(),
            method: "chat.send".into(),
            params: Some(serde_json::json!({"message": "hi"})),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("params"));
        assert!(json.contains("\"message\""));
    }

    #[test]
    fn stream_event_deserializes() {
        let json = r#"{"state":"delta","message":"Hello ","sessionKey":"agent:a1:main","runId":"run-1"}"#;
        let event: StreamEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.state, "delta");
        assert_eq!(event.message, Some("Hello ".to_string()));
        assert_eq!(event.run_id, Some("run-1".to_string()));
    }

    #[test]
    fn chat_send_ack_deserializes() {
        let json = r#"{"runId":"run-123","status":"started"}"#;
        let ack: ChatSendAck = serde_json::from_str(json).unwrap();
        assert_eq!(ack.run_id, "run-123");
        assert_eq!(ack.status, "started");
    }

    #[test]
    fn agent_model_deserializes_from_string() {
        let json = r#"{"id":"main","model":"anthropic/claude-opus-4-6"}"#;
        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.model, Some("anthropic/claude-opus-4-6".to_string()));
    }

    #[test]
    fn agent_model_deserializes_from_object() {
        let json = r#"{"id":"main","model":{"primary":"anthropic/claude-opus-4-6"}}"#;
        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.model, Some("anthropic/claude-opus-4-6".to_string()));
    }

    #[test]
    fn agent_model_deserializes_null() {
        let json = r#"{"id":"main","model":null}"#;
        let agent: Agent = serde_json::from_str(json).unwrap();
        assert!(agent.model.is_none());
    }
}
