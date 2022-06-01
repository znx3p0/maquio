#[cfg(not(target_arch = "wasm32"))]
mod tcp;
mod wss;

#[cfg(unix)]
mod unix;

#[cfg(not(target_arch = "wasm32"))]
pub use tcp::Tcp;
#[cfg(unix)]
pub use unix::Unix;
pub use wss::WebSocket;
