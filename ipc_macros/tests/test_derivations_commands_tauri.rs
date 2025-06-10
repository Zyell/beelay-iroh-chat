use ipc_macros;

#[allow(async_fn_in_trait)]
pub trait Commands {
    async fn hello(name: String) -> Result<String, String>;
    async fn bob();
}

struct AppData;

ipc_macros::impl_trait!(Commands, {
    #[tauri::command]
    async fn hello(state: tauri::State<'_, AppData>, name: String) -> Result<String, String> {
        Ok(format!("Hello {}", name))
    }
    #[tauri::command]
    async fn bob() {
        println!("Bob");
    }
});
