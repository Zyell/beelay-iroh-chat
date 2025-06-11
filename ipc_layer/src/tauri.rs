use crate::{API, Message};
use beelay_protocol::{DocumentId, IrohBeelayProtocol, NodeId, NodeTicket, Router};
use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

pub struct AppData {
    router: Router,
    pub beelay_protocol: IrohBeelayProtocol,
    pub document_id: OnceCell<DocumentId>,
    pub node_ticket: OnceCell<NodeTicket>,
}

impl AppData {
    pub fn new(router: Router, beelay_protocol: IrohBeelayProtocol) -> Self {
        Self {
            router,
            beelay_protocol,
            document_id: OnceCell::new(),
            node_ticket: OnceCell::new(),
        }
    }

    pub fn get_document_id(&self) -> Result<&DocumentId, String> {
        self.document_id
            .get()
            .ok_or("Document ID not set".to_string())
    }

    pub fn set_document_id(&self, document_id: DocumentId) -> Result<(), DocumentId> {
        self.document_id.set(document_id)
    }

    pub fn get_node_tickedt(&self) -> Result<&NodeTicket, String> {
        self.node_ticket
            .get()
            .ok_or("Node Ticket not set".to_string())
    }

    pub fn set_node_ticket(&self, node_ticket: NodeTicket) -> Result<(), NodeTicket> {
        self.node_ticket.set(node_ticket)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageWithMetaData {
    pub message: Message,
    pub peer_id: NodeId,
}

impl MessageWithMetaData {
    pub fn new(message: Message, peer_id: NodeId) -> Self {
        Self { message, peer_id }
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.message.timestamp
    }
}

ipc_macros::impl_trait!(API, {
    #[tauri::command]
    async fn get_serialized_ticket(state: tauri::State<'_, AppData>) -> Result<String, String> {
        state
            .beelay_protocol
            .string_beelay_ticket()
            .await
            .map_err(|e| e.to_string())
    }

    #[tauri::command]
    async fn connect_via_serialized_ticket(
        ticket: String,
        state: tauri::State<'_, AppData>,
    ) -> Result<String, String> {
        let (doc_id, node_ticket) = state
            .beelay_protocol
            .connect_via_serialized_ticket(ticket)
            .await
            .map_err(|e| e.to_string())?;
        state
            .set_node_ticket(node_ticket)
            .expect("Should be a valid node ticket set only once");
        Ok(format!("Connected with document {}", doc_id))
    }
    #[tauri::command]
    async fn broadcast_message(
        message: Message,
        state: tauri::State<'_, AppData>,
    ) -> Result<(), String> {
        let document_id = state.get_document_id()?;
        let node_ticket = state.get_node_tickedt()?;
        let message_w_meta_data =
            MessageWithMetaData::new(message, state.beelay_protocol.node_id());
        let data = postcard::to_allocvec(&message_w_meta_data).map_err(|e| e.to_string())?;
        state
            .beelay_protocol
            .add_data_to_document(data, *document_id, node_ticket.clone())
            .await
            .map_err(|e| e.to_string())
    }
});
