use beelay_protocol::{start_beelay_node, IrohBeelayProtocol, Router};
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

struct AppData {
    router: Router,
    beelay_protocol: IrohBeelayProtocol,
}

impl AppData {
    fn new(router: Router, beelay_protocol: IrohBeelayProtocol) -> Self {
        Self {
            router,
            beelay_protocol,
        }
    }
}

async fn setup<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> anyhow::Result<()> {
    let (router, beelay_protocol) = start_beelay_node().await?;
    let app_data = AppData::new(router, beelay_protocol);
    handle.manage(app_data);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
