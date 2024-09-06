use crate::client::Exocore;

/// Trait implemented by an application WASM module. The struct implementing
/// this trait should have the `#[exocore_app]` macro attribute.
pub trait App: Send {
    fn start(&self, client: &Exocore) -> Result<(), AppError>;
}

// Called by #[exocore_app] macro at application initialization.
pub fn __exocore_app_register(app: Box<dyn App>) {
    let exocore = Exocore::get();
    exocore.register_app(app);
}

pub(crate) fn boot_app() {
    let exocore = Exocore::get();
    exocore.with_app(|app| app.start(exocore).expect("Failed to start application"))
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
