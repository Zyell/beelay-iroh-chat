use ipc_macros;
use tauri::Runtime;

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Bob {
    name: String,
}

ipc_macros::derive_events! (
    ui=#[cfg(not(test_ui))],
    tauri=#[cfg(not(test_tauri))],
    {
        ("test_event", Bob),
        ("TestEvent2", String),
    }
);
