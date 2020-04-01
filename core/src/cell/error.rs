#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Configuration error: {}", _0)]
    Config(String),

    #[fail(display = "Cell error: {}", _0)]
    Cell(String),

    #[fail(display = "Application '{}' error: {}", _0, _1)]
    Application(String, String),

    #[fail(display = "Key error: {}", _0)]
    Key(crate::crypto::keys::Error),

    #[fail(display = "Node error: {}", _0)]
    Node(String),
}
