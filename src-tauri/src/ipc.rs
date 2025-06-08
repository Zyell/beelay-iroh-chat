use beelay_protocol::{Commit, CommitHash, DocumentId, NodeId};
use crate::state::AppData;
use tauri::{State};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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
    let (doc_id, node_ticket) = state
        .beelay_protocol
        .connect_via_serialized_ticket(ticket)
        .await
        .map_err(|e| e.to_string())?;
    state.set_node_ticket(node_ticket).expect("Should be a valid node ticket set only once");
    Ok(format!("Connected with document {}", doc_id))
}


// todo: refactor out this with common types between frontend and backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    timestamp: DateTime<Utc>,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageWithMetaData {
    pub message: Message,
    pub peer_id: NodeId
}

impl MessageWithMetaData {
    pub fn new(message: Message, peer_id: NodeId) -> Self {
        Self {
            message,
            peer_id
        }
    }
    
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.message.timestamp
    }
}

#[tauri::command]
pub async fn broadcast_message(message: Message, state: State<'_, AppData>) -> Result<(), String> {
    let document_id = state.get_document_id()?;
    let node_ticket = state.get_node_tickedt()?;
    let message_w_meta_data = MessageWithMetaData::new(message, state.beelay_protocol.node_id());
    let data = postcard::to_allocvec(&message_w_meta_data).map_err(|e| e.to_string())?;
    state
        .beelay_protocol
        .add_data_to_document(data, *document_id, node_ticket.clone())
        .await
        .map_err(|e| e.to_string())
}
