use tauri::Emitter;
use ipc_macros;

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
struct Bob {
    name: String,
}

// #[ipc_macros::derive_event(ui)]
#[ipc_macros::derive_event(tauri)]
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
struct TestEvent(Bob);
