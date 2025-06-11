// use chrono::prelude::*;
// use serde::Serialize;
// use tauri_sys::core;
// 
// pub mod api {
//     use super::*;
//     use serde::Deserialize;
// 
//     pub(crate) async fn get_serialized_ticket() -> Result<String, String> {
//         core::invoke_result::<String, String>("get_serialized_ticket", &()).await
//     }
// 
//     pub(crate) async fn connect_via_serialized_ticket(ticket: String) -> Result<String, String> {
//         #[derive(Debug, Serialize)]
//         struct Args {
//             ticket: String,
//         }
//         core::invoke_result::<String, String>("connect_via_serialized_ticket", Args { ticket })
//             .await
//     }
// 
//     #[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
//     pub(crate) struct Message {
//         timestamp: DateTime<Utc>,
//         text: String,
//     }
// 
//     impl Message {
//         pub(crate) fn new(msg: String) -> Self {
//             let timestamp = Utc::now();
//             Self {
//                 timestamp,
//                 text: msg,
//             }
//         }
// 
//         pub(crate) fn unpack_for_html_integration(self) -> (String, String) {
//             (self.text, self.timestamp.to_string())
//         }
// 
//         pub(crate) fn timestamp(&self) -> &DateTime<Utc> {
//             &self.timestamp
//         }
//     }
// 
//     pub(crate) async fn broadcast_message(message: Message) -> Result<(), String> {
//         #[derive(Debug, Serialize)]
//         struct Args {
//             message: Message,
//         }
//         core::invoke_result::<(), String>("broadcast_message", Args { message })
//             .await
//     }
// 
//     #[cfg(feature = "mobile")]
//     pub(crate) mod barcode_scanner {
//         use serde::{Deserialize, Serialize};
//         use std::fmt::{Display, Formatter};
// 
//         #[derive(Debug, Serialize, Deserialize)]
//         pub(crate) enum Format {
//             #[serde(alias = "QR_CODE")]
//             QRCode,
//             UPC_A,
//             UPC_E,
//             EAN8,
//             EAN13,
//             Code39,
//             Code93,
//             Code128,
//             Codabar,
//             ITF,
//             Aztec,
//             DataMatrix,
//             PDF417,
//         }
// 
//         #[derive(Debug, Serialize)]
//         pub(crate) enum CameraDirection {
//             #[serde(rename = "back")]
//             Back,
//             #[serde(rename = "front")]
//             Front,
//         }
// 
//         #[derive(Debug, Serialize)]
//         pub(crate) struct ScanOptions {
//             #[serde(rename = "cameraDirection")]
//             camera_direction: CameraDirection,
//             formats: Vec<Format>,
//             windowed: bool,
//         }
// 
//         #[derive(Debug, Deserialize)]
//         pub(crate) struct Scanned {
//             pub(crate) content: String,
//             format: Format,
//             bounds: String, //this is unknown type in typescript so yeah...  I guess we make it a string and hope?
//         }
// 
//         pub(crate) async fn scan_barcode(
//             format: Format,
//             windowed: bool,
//             camera_direction: CameraDirection,
//         ) -> Scanned {
//             tauri_sys::core::invoke::<Scanned>(
//                 "plugin:barcode-scanner|scan",
//                 ScanOptions {
//                     camera_direction,
//                     formats: vec![format],
//                     windowed,
//                 },
//             )
//             .await
//         }
//     }
// }
