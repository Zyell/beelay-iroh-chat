#[cfg(feature = "tauri")]
pub mod tauri;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Message {
    timestamp: DateTime<Utc>,
    text: String,
}

impl Message {
    pub fn new(msg: String) -> Self {
        let timestamp = Utc::now();
        Self {
            timestamp,
            text: msg,
        }
    }

    pub fn unpack_for_html_integration(self) -> (String, String) {
        (self.text, self.timestamp.to_string())
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
}

#[cfg_attr(feature = "ui", ipc_macros::invoke_bindings)]
#[allow(async_fn_in_trait)]
pub trait API {
    async fn get_serialized_ticket() -> Result<String, String>;
    async fn connect_via_serialized_ticket(ticket: String) -> Result<String, String>;
    async fn broadcast_message(message: Message) -> Result<(), String>;
}

#[cfg(feature = "ui")]
#[cfg(feature = "mobile")]
pub mod barcode_scanner {
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize)]
    pub enum Format {
        #[serde(alias = "QR_CODE")]
        QRCode,
        UPC_A,
        UPC_E,
        EAN8,
        EAN13,
        Code39,
        Code93,
        Code128,
        Codabar,
        ITF,
        Aztec,
        DataMatrix,
        PDF417,
    }

    #[derive(Debug, Serialize)]
    pub enum CameraDirection {
        #[serde(rename = "back")]
        Back,
        #[serde(rename = "front")]
        Front,
    }

    #[derive(Debug, Serialize)]
    pub struct ScanOptions {
        #[serde(rename = "cameraDirection")]
        camera_direction: CameraDirection,
        formats: Vec<Format>,
        windowed: bool,
    }

    #[derive(Debug, Deserialize)]
    pub struct Scanned {
        pub content: String,
        format: Format,
        bounds: String, //this is unknown type in typescript so yeah...  I guess we make it a string and hope?
    }

    pub async fn scan_barcode(
        format: Format,
        windowed: bool,
        camera_direction: CameraDirection,
    ) -> Scanned {
        tauri_sys::core::invoke::<Scanned>(
            "plugin:barcode-scanner|scan",
            ScanOptions {
                camera_direction,
                formats: vec![format],
                windowed,
            },
        )
        .await
    }
}

ipc_macros::derive_events! (
    ui=#[cfg(feature = "ui")],
    tauri=#[cfg(feature = "tauri")],
    {
        ("conversation", Message),
        ("connection", String),
        ("connection_type", String),
    }
);
