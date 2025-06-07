use beelay_protocol::{DocEvent, DocumentId, IrohBeelayProtocol, Router};
use tauri::async_runtime::Receiver;

use once_cell::sync::OnceCell;

pub struct DocumentState {
    pub(crate) document_id: DocumentId,
    pub(crate) rx: Receiver<(DocumentId, DocEvent)>
}

impl DocumentState {
    fn new(document_id: DocumentId, rx: Receiver<(DocumentId, DocEvent)>) -> Self {
        Self {document_id, rx}
    }
}

pub struct AppData {
    router: Router,
    pub(crate) beelay_protocol: IrohBeelayProtocol,
    pub(crate) document_state: OnceCell<DocumentState>
}

impl AppData {
    pub(crate) fn new(router: Router, beelay_protocol: IrohBeelayProtocol) -> Self {
        Self {
            router,
            beelay_protocol,
            document_state: OnceCell::new()
        }
    }

    pub(crate) fn build_document_state(&mut self, document_id: DocumentId, rx: Receiver<(DocumentId, DocEvent)>) -> Result<(), DocumentState> {
        self.document_state.set(DocumentState::new(document_id, rx))
    }
}
