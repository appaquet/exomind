pub mod client;
pub use client::{Client, ClientConfiguration, ClientHandle};

#[cfg(feature = "local-store")]
pub mod server;
#[cfg(feature = "local-store")]
pub use server::{Server, ServerConfiguration};

#[cfg(test)]
mod tests;
