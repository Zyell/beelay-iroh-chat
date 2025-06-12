//! This module contains functionality for messaging, Tauri integrations, and barcode scanning,
//! with optional features for specific platforms or use cases, such as UI, Tauri, or mobile.
//! It is designed to allow both the ui and tauri apis to coexist at the same time (all features enabled)
//! so that development is easy with rust IDE tools.  It gets messy if you try to troubleshoot in one workspace
//! separate backend and frontend implementations of the same API/types.  It is import to configure
//! the backend with the `tauri` feature and the frontend with the `ui` feature (and optionally `mobile` if you are targeting mobile)
//!
//! # Modules
//! - `tauri`: Available when the `tauri` feature is enabled, providing Tauri-specific functionality.
//! - `barcode_scanner`: Available when both the `ui` and `mobile` features are enabled, enabling barcode scanning capabilities for mobile devices.
//!
//! # Structures
//!
//! ## `Message`
//! Represents a message with a timestamp and text content.
//!
//! ### Fields
//! - `timestamp`: A `DateTime<Utc>` representing when the message was created.
//! - `text`: A `String` containing the text of the message.
//!
//! ### Methods
//! - `new(msg: String) -> Self`:
//!   Constructs a new `Message` with the current timestamp and the given text.
//! - `unpack_for_html_integration(self) -> (String, String)`:
//!   Returns a tuple containing the message text and its timestamp as strings, suitable for integration with HTML or other UIs.
//! - `timestamp(&self) -> &DateTime<Utc>`:
//!   Returns a reference to the `Message`'s timestamp.
//!
//! ## `API`
//! A trait that defines asynchronous methods for working with tickets and broadcasting messages.
//!
//! ### Methods
//! - `async fn get_serialized_ticket() -> Result<String, String>`:
//!   Retrieves a serialized ticket as a `String`. Returns an error message in case of failure.
//! - `async fn connect_via_serialized_ticket(ticket: String) -> Result<String, String>`:
//!   Connects using the provided `ticket`. Returns a success message if the connection succeeds, or an error message otherwise.
//! - `async fn broadcast_message(message: Message) -> Result<(), String>`:
//!   Broadcasts the provided `Message`. Returns `Ok(())` on success or an error message on failure.
//!
//! ## `barcode_scanner`
//! Provides functionality for scanning barcodes on mobile devices. Available when the `ui` and `mobile` features are enabled.
//!
//! ### Enums
//! - `Format`:
//!   Represents the format of a barcode. Possible variants include `QRCode`, `UPC_A`, `Code128`, `DataMatrix`, etc.
//! - `CameraDirection`:
//!   Defines the direction of the camera (`Back` or `Front`).
//!
//! ### Structures
//! - `ScanOptions`:
//!   Represents options for scanning a barcode, including the camera direction, supported formats, and whether windowing is enabled.
//! - `Scanned`:
//!   Represents the result of a barcode scan, including the scanned content, format, and bounds.
//!
//! ### Methods
//! - `async fn scan_barcode(format: Format, windowed: bool, camera_direction: CameraDirection) -> Scanned`:
//!   Initiates a barcode scan with the given `format`, whether windowing is enabled, and the given `camera_direction`.
//!   Returns a `Scanned` object containing details about the scanned barcode.
//!
//! ## Events
//! Defines IPC events with conditional compilation for the `ui` and `tauri` features.
//!
//! ### Events
//! - `"conversation"`: Associated with the `Message` type.
//! - `"connection"`: Associated with the `String` type.
//! - `"connection_type"`: Associated with the `String` type.
//!
//! # Feature Flags
//! - `tauri`: Enables the `tauri` module.
//! - `ui`: Enables user interface-related functionality, including event bindings and barcode scanning.
//! - `mobile`: Enables the `barcode_scanner` module for mobile devices.
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
