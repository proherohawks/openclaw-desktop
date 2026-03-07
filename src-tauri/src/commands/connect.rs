use tauri::State;
use tokio::sync::Mutex;
use crate::openclaw::{client::OpenClawClient, types::ConnectResult};

#[tauri::command]
pub async fn connect(
    endpoint: String,
    token: String,
    state: State<'_, Mutex<Option<OpenClawClient>>>,
) -> Result<ConnectResult, String> {
    let (client, result) = OpenClawClient::connect(&endpoint, &token)
        .await
        .map_err(|e| e.to_string())?;

    let mut guard = state.lock().await;
    *guard = Some(client);

    Ok(result)
}
