use serde::{Serialize};
use tauri_sys::core;

pub mod api {
    use super::*;

    pub(crate) async fn get_serialized_ticket() -> Result<String, String> {
        core::invoke_result::<String, String>("get_serialized_ticket", &()).await
    }

    pub(crate) async fn connect_via_serialized_ticket(ticket: String) -> Result<String, String> {
        #[derive(Debug, Serialize)]
        struct Args {
            ticket: String,
        }
        core::invoke_result::<String, String>("connect_via_serialized_ticket", Args { ticket })
            .await
    }
}
