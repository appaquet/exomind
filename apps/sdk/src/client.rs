use std::sync::{Arc, Mutex};

use crate::{app::App, store::Store};

lazy_static! {
    static ref EXOCORE: Exocore = Exocore {
        store: Arc::new(Store::new()),
        app: Arc::new(Mutex::new(None)),
    };
}

/// Exocore client.
pub struct Exocore {
    pub store: Arc<Store>,
    app: Arc<Mutex<Option<Box<dyn App>>>>,
}

impl Exocore {
    pub(crate) fn get() -> &'static Exocore {
        &EXOCORE
    }

    pub(crate) fn register_app(&self, app: Box<dyn App>) {
        let mut app_box = self.app.lock().unwrap();
        if app_box.is_some() {
            panic!("An application is already registered")
        }
        *app_box = Some(app);
    }

    pub(crate) fn with_app<F: FnMut(&mut dyn App)>(&self, mut f: F) {
        let mut app_box = self.app.lock().unwrap();
        let app = app_box
            .as_mut()
            .expect("No application is registered (no application struct with #[exocore_app])");
        f(app.as_mut());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _test_send_sync() {
        fn send_sync<A: Send + Sync>(_a: Option<A>) {}
        let opt: Option<Exocore> = None;
        send_sync(opt);
    }
}
