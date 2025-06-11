// use beelay_protocol::{DocumentId, IrohBeelayProtocol, NodeTicket, Router};
//
// use once_cell::sync::OnceCell;
//
// pub struct AppData {
//     router: Router,
//     pub(crate) beelay_protocol: IrohBeelayProtocol,
//     pub(crate) document_id: OnceCell<DocumentId>,
//     pub(crate) node_ticket: OnceCell<NodeTicket>
// }
//
// impl AppData {
//     pub(crate) fn new(router: Router, beelay_protocol: IrohBeelayProtocol) -> Self {
//         Self {
//             router,
//             beelay_protocol,
//             document_id: OnceCell::new(),
//             node_ticket: OnceCell::new()
//         }
//     }
//
//     pub(crate) fn get_document_id(&self) -> Result<&DocumentId, String> {
//         self.document_id.get().ok_or("Document ID not set".to_string())
//     }
//
//     pub(crate) fn set_document_id(&self, document_id: DocumentId) -> Result<(), DocumentId> {
//         self.document_id.set(document_id)
//     }
//
//     pub(crate) fn get_node_tickedt(&self) -> Result<&NodeTicket, String> {
//         self.node_ticket.get().ok_or("Node Ticket not set".to_string())
//     }
//
//     pub(crate) fn set_node_ticket(&self, node_ticket: NodeTicket) -> Result<(), NodeTicket> {
//         self.node_ticket.set(node_ticket)
//     }
// }
