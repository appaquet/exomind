#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub const DEFAULT_DISCO_SERVER: &str = "https://disco.exocore.io";

pub mod payload;
pub use payload::{CreatePayloadRequest, CreatePayloadResponse, Payload, Pin};

#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub use server::{Server, ServerConfig};

pub mod client;
pub use client::Client;
