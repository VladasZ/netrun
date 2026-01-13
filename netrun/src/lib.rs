#[cfg(not_wasm)]
mod connection;
pub mod rest;
#[cfg(not_wasm)]
pub mod secret;
mod tests;

#[cfg(not_wasm)]
pub use connection::*;
pub use local_ip_address::*;
