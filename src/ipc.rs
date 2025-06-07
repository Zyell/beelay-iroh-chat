use serde::Serialize;
use tauri_sys::core;
use chrono::prelude::*;

pub mod api {
    use serde::Deserialize;
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub(crate) struct Message {
        timestamp: DateTime<Utc>,
        text: String,
    }

    #[cfg(mobile)]
    pub(crate) mod barcode_scanner {
        use serde::{Deserialize, Serialize};
        use std::fmt::{Display, Formatter};

        #[derive(Debug, Serialize, Deserialize)]
        pub(crate) enum Format {
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

        impl Display for Format {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let rep = match self {
                    Format::QRCode => "QR_CODE",
                    Format::UPC_A => "UPC_A",
                    Format::UPC_E => "UPC_E",
                    Format::EAN8 => "EAN_8",
                    Format::EAN13 => "EAN_13",
                    Format::Code39 => "CODE_39",
                    Format::Code93 => "CODE_93",
                    Format::Code128 => "CODE_128",
                    Format::Codabar => "CODABAR",
                    Format::ITF => "ITF",
                    Format::Aztec => "AZTEC",
                    Format::DataMatrix => "DATA_MATRIX",
                    Format::PDF417 => "PDF_417",
                };
                f.write_str(rep)
            }
        }

        #[derive(Debug, Serialize)]
        pub(crate) enum CameraDirection {
            Back,
            Front,
        }

        impl Display for CameraDirection {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let rep = match self {
                    CameraDirection::Back => "back",
                    CameraDirection::Front => "front",
                };
                f.write_str(rep)
            }
        }

        #[derive(Debug, Serialize)]
        pub(crate) struct ScanOptions {
            #[serde(rename = "cameraDirection")]
            camera_direction: CameraDirection,
            formats: Vec<Format>,
            windowed: bool,
        }

        #[derive(Debug, Deserialize)]
        pub(crate) struct Scanned {
            content: String,
            format: Format,
            bounds: String, //this is unknown type in typescript so yeah...  I guess we make it a string and hope?
        }

        pub(crate) async fn scan_barcode(
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
}
