use crate::state::AppData;
use tauri::{AppHandle, Emitter, State};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
pub async fn get_serialized_ticket(state: State<'_, AppData>) -> Result<String, String> {
    state
        .beelay_protocol
        .string_beelay_ticket()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn connect_via_serialized_ticket(
    ticket: String,
    state: State<'_, AppData>,
) -> Result<String, String> {
    let doc_id = state
        .beelay_protocol
        .connect_via_serialized_ticket(ticket)
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("Connected with document {}", doc_id))
}
