use beelay_protocol::primitives::IrohEvent;
use beelay_protocol::{
    CommitOrBundle, DocEvent, DocumentId, NoticeSubscriberClosure, start_beelay_node,
};
use ipc_layer::events;
use ipc_layer::tauri::{AppData, MessageWithMetaData, command_handler};
use tauri::async_runtime::{Receiver, channel};
use tauri::{AppHandle, Manager};

async fn handle_doc_events<R: tauri::Runtime>(
    mut rx: Receiver<(DocumentId, DocEvent)>,
    handle: AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let this_node_id = handle.state::<AppData>().beelay_protocol.node_id();
    let mut recent_timestamp: i64 = 0;
    while let Some((doc_id, doc_event)) = rx.recv().await {
        match doc_event {
            DocEvent::Data { data } => {
                match data {
                    CommitOrBundle::Commit(commit) => {
                        let contents = commit.contents();
                        // ensure we don't capture empty messages, like the initial commits
                        if contents.len() > 0 {
                            let message: MessageWithMetaData =
                                postcard::from_bytes(commit.contents())?;
                            let new_timestamp = message.timestamp().timestamp();
                            // prevent replay of this node's messages and prevent already seen timestamps
                            if message.peer_id != this_node_id && new_timestamp > recent_timestamp {
                                recent_timestamp = new_timestamp;
                                events::tauri::conversation(message.message).emit(&handle)?;
                            }
                        }
                    }
                    // this is a collection of commit hashes, we don't do anything with this yet
                    CommitOrBundle::Bundle(bundle) => {}
                };
            }
            DocEvent::Discovered => {
                let state = handle.state::<AppData>();
                // todo: is this the best way to manage setting the document id? We need it for most actions, but investigate a cleaner ordering, OnceCells are awesome though and at least make managing this easy for the time being.
                let _ = state.set_document_id(doc_id);
                // todo: this is a hack, it is pretty useless sending this string, clean this up.
                events::tauri::connection("connected".into()).emit(&handle)?;
            }
            DocEvent::AccessChanged { .. } => {}
        }
    }
    Ok(())
}

async fn handle_connections<R: tauri::Runtime>(
    mut rx: Receiver<IrohEvent>,
    handle: AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // send connection type to frontend for the most recent connection dialed or received
    while let Some(iroh_event) = rx.recv().await {
        let (node_ticket, connection_type) = iroh_event.unpack();
        // convert to string to avoid import of beelay protocol types in the frontend for now
        events::tauri::connection_type(format!("{:?}", connection_type)).emit(&handle)?;
        let state = handle.state::<AppData>();
        let _ = state.set_node_ticket(node_ticket);
    }
    Ok(())
}

async fn setup<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> anyhow::Result<()> {
    let (tx, mut rx) = channel(100);
    let (tx_iroh, rx_iroh) = channel(100);

    // Note: this is a messy bit of code since types cannot implement impl traits.
    let notice_closure: NoticeSubscriberClosure =
        Box::new(move |doc_id: DocumentId, event: DocEvent| {
            let tx = tx.clone();
            Box::pin(async move {
                // todo: in the future, it may be worth processing these events to allow for document separation for multiple chats
                let send_result = tx.send((doc_id, event)).await;
                // throw out results for now...
                match send_result {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Notification closure error: {}", e);
                    }
                }
            })
        });

    let (router, beelay_protocol) = start_beelay_node(notice_closure, Some(tx_iroh)).await?;
    let app_data = AppData::new(router, beelay_protocol);
    handle.manage(app_data);

    let handle1 = handle.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = handle_doc_events(rx, handle1).await {
            eprintln!("Task error: {}", e);
        }
    });

    let handle2 = handle.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = handle_connections(rx_iroh, handle2).await {
            eprintln!("Task error: {}", e);
        }
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // todo: add more tracing integration to this app.
    // todo: add persistence of chats, will require changes to beely protocol
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
        .invoke_handler(command_handler())
        // NOTE: This shows as an error in Rustrover, but it is not an issue!
        // It just can't reconcile the build context with the ipc_macros crate in this workspace.
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
