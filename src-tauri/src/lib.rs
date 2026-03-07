mod commands;
mod error;
mod openclaw;

use tokio::sync::Mutex;
use openclaw::client::OpenClawClient;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(Option::<OpenClawClient>::None))
        .invoke_handler(tauri::generate_handler![
            commands::connect::connect,
            commands::list_agents::list_agents,
            commands::send_message::send_message,
            commands::get_status::get_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
