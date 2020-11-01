#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Cell error: {0}")]
    Cell(String),

    #[error("Application '{0}' error: {1}")]
    Application(String, String),

    #[error("Key error: {0}")]
    Key(#[from] crate::sec::keys::Error),

    #[error("Node error: {0}")]
    Node(String),
}
