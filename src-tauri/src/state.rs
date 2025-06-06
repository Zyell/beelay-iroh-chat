use beelay_protocol::{IrohBeelayProtocol, Router};


pub struct AppData {
    router: Router,
    pub(crate) beelay_protocol: IrohBeelayProtocol,
}

impl AppData {
    pub(crate) fn new(router: Router, beelay_protocol: IrohBeelayProtocol) -> Self {
        Self {
            router,
            beelay_protocol,
        }
    }
}