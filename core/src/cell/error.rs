#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Cell error: {0}")]
    Cell(String),

    #[error("Application '{}' error: {0}")]
    Application(String, String),

    #[error("Key error: {0}")]
    Key(#[from] crate::sec::keys::Error),

    #[error("Node error: {0}")]
    Node(String),
}
