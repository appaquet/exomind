#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(#[source] anyhow::Error),

    #[error("Cell error: {0}")]
    Cell(#[source] anyhow::Error),

    #[error("Application '{0}' error: {1}")]
    Application(String, #[source] anyhow::Error),

    #[error("Key error: {0}")]
    Key(#[from] crate::sec::keys::Error),

    #[error("Node error: {0}")]
    Node(#[source] anyhow::Error),

    #[error("No directory configured in node or cell")]
    NoDirectory,

    #[error("Directory error: {0}")]
    Directory(#[from] crate::dir::Error),
}
