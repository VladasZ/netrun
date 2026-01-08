#[cfg(not_wasm)]
mod connection;
pub mod rest;

#[cfg(not_wasm)]
pub use connection::*;
