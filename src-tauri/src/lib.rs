mod ipc;
mod state;

use crate::state::AppData;
use beelay_protocol::{
    CommitOrBundle, DocEvent, DocumentId, NoticeSubscriberClosure, start_beelay_node,
};
use chrono::prelude::*;
use ipc::Message;
use serde::{Deserialize, Serialize};
use tauri::async_runtime::{Receiver, channel};
use tauri::{AppHandle, Emitter, Manager};
use crate::ipc::MessageWithMetaData;

async fn handle_doc_events<R: tauri::Runtime>(
    mut rx: Receiver<(DocumentId, DocEvent)>,
    handle1: AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let this_node_id = handle1.state::<AppData>().beelay_protocol.node_id();
    let mut recent_timestamp: i64 = 0;
    while let Some((doc_id, doc_event)) = rx.recv().await {
        println!("\n----------------------------------------------------\n");
        println!("***********Got notice: {:?}", doc_event);
        match doc_event {
            DocEvent::Data { data } => {
                match data {
                    CommitOrBundle::Commit(commit) => {
                        let contents = commit.contents();
                        // ensure we don't capture empty messages, like the initial commits
                        if contents.len() > 0 {
                            let message: MessageWithMetaData = postcard::from_bytes(commit.contents())?;
                            println!("!!!!!!!!!!!!!message: {:?}", message);
                            let new_timestamp = message.timestamp().timestamp();
                            // prevent replay of this node's messages and prevent already seen timestamps
                            if message.peer_id != this_node_id && new_timestamp > recent_timestamp {
                                println!("<<<<<<<<<<<<<<<<<<<< sending message: {:?}", message);
                                recent_timestamp = new_timestamp;
                                handle1.emit("conversation", message.message)?;
                            }
                        }
                    }
                    CommitOrBundle::Bundle(bundle) => {
                        println!("?????????? Bundle: {:?}", bundle);
                    }
                };
            }
            DocEvent::Discovered => {
                let state = handle1.state::<AppData>();
                let _ = state.set_document_id(doc_id);
                handle1.emit("connection", "connected")?;
            }
            DocEvent::AccessChanged { .. } => {}
        }
    }
    Ok(())
}

async fn setup<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> anyhow::Result<()> {
    let (tx, mut rx) = channel(100);
    let (tx_iroh, mut rx_iroh) = channel(100);

    // Note: this is a messy bit of code since types cannot implement impl traits.
    let notice_closure: NoticeSubscriberClosure =
        Box::new(move |doc_id: DocumentId, event: DocEvent| {
            let tx = tx.clone();
            Box::pin(async move {
                println!("Notice closure called: {}, {:?}", doc_id, event);
                let send_result = tx.send((doc_id, event)).await;
                // throw out results for now...
                match send_result {
                    Ok(_) => {}
                    Err(_) => {}
                }
            })
        });

    let (router, beelay_protocol) = start_beelay_node(notice_closure, Some(tx_iroh)).await?;
    let app_data = state::AppData::new(router, beelay_protocol);
    handle.manage(app_data);

    let handle1 = handle.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = handle_doc_events(rx, handle1).await {
            eprintln!("Task error: {}", e);
        }
    });

    let handle2 = handle.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(iroh_event) = rx_iroh.recv().await {
            let (node_ticket, connection_type) = iroh_event.unpack();
            println!("Connection type: {:?}", connection_type);
            let state = handle2.state::<AppData>();
            let _ = state.set_node_ticket(node_ticket);
        }
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let subscriber = tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .compact()
        // Display source code file paths
        .with_file(true)
        // Display source code line numbers
        .with_line_number(true)
        // Display the thread ID an event was recorded on
        .with_thread_ids(true)
        // Don't display the event's target (module path)
        .with_target(false)
        // Build the subscriber
        // .with_max_level(Level::TRACE)
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).unwrap();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                println!("starting backend...");
                if let Err(err) = setup(handle).await {
                    eprintln!("failed: {:?}", err);
                }
            });
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_barcode_scanner::init())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::get_serialized_ticket,
            ipc::connect_via_serialized_ticket,
            ipc::broadcast_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
