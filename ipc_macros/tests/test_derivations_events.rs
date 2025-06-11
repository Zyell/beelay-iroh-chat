use ipc_macros;
use tauri::Runtime;

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Bob {
    name: String,
}

ipc_macros::derive_events! (
    ui=#[cfg(not(feature = "ui"))],
    tauri=#[cfg(not(feature = "tauri"))],
    {
        ("test_event", Bob),
        ("TestEvent2", String),
    }
);
