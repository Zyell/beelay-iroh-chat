mod ipc;
mod state;

use beelay_protocol::{start_beelay_node, CommitOrBundle, DocEvent, DocumentId, NoticeSubscriberClosure};
use tauri::async_runtime::channel;
use tauri::{Emitter, Manager};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use ipc::Message;
use crate::state::AppData;

async fn setup<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> anyhow::Result<()> {

    let (tx, mut rx) = channel(100);

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

    let (router, beelay_protocol) = start_beelay_node(notice_closure).await?;
    let app_data = state::AppData::new(router, beelay_protocol);
    handle.manage(app_data);

    // extract out commits to the document we use for chat
    // todo: eventually we want to separate documents for chat.
    while let Some((doc_id, doc_event)) = rx.recv().await {
        println!("Got notice: {:?}", doc_event);
        match doc_event {
            DocEvent::Data { data } => {
                match data {
                    CommitOrBundle::Commit(commit) => {
                        let message: Message = postcard::from_bytes(commit.contents())?;
                        handle.emit("conversation", message)?
                    }
                    CommitOrBundle::Bundle(bundle) => {}
                };
                
            }
            DocEvent::Discovered => {
                handle.state::<AppData>().set_document_id(doc_id).expect("failed to set document id");
                handle.emit("connection", "connected")?;
            }
            DocEvent::AccessChanged { .. } => {}
        }
    }

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
