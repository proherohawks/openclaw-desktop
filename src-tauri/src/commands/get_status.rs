use tauri::State;
use tokio::sync::Mutex;
use crate::openclaw::{client::OpenClawClient, types::ConnectionStatus};

#[tauri::command]
pub async fn get_status(
    state: State<'_, Mutex<Option<OpenClawClient>>>,
) -> Result<ConnectionStatus, String> {
    let mut guard = state.lock().await;
    match guard.as_mut() {
        None => Ok(ConnectionStatus {
            connected: false,
            endpoint: None,
        }),
        Some(client) => {
            let alive = client.health().await.unwrap_or(false);
            Ok(ConnectionStatus {
                connected: alive,
                endpoint: Some(client.ws_url().to_string()),
            })
        }
    }
}
