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
