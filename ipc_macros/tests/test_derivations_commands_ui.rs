use ipc_macros;

#[ipc_macros::invoke_bindings]
#[allow(async_fn_in_trait)]
pub trait Commands {
    async fn hello(name: String) -> Result<String, String>;
    async fn bob();
}
