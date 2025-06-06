mod ipc;
mod state;

use beelay_protocol::start_beelay_node;
use tauri::{Manager};

async fn setup<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> anyhow::Result<()> {
    let (router, beelay_protocol) = start_beelay_node().await?;
    let app_data = state::AppData::new(router, beelay_protocol);
    handle.manage(app_data);

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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::get_serialized_ticket,
            ipc::connect_via_serialized_ticket
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
