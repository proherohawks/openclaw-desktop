use tauri::State;
use tokio::sync::Mutex;
use crate::openclaw::{client::OpenClawClient, types::Agent};
use crate::error::AppError;

#[tauri::command]
pub async fn list_agents(
    state: State<'_, Mutex<Option<OpenClawClient>>>,
) -> Result<Vec<Agent>, String> {
    let guard = state.lock().await;
    let client = guard
        .as_ref()
        .ok_or_else(|| AppError::NotConnected.to_string())?;
    Ok(client.agents().to_vec())
}
