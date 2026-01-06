pub mod adc;
#[cfg(feature = "ethernet")]
pub mod ethernet;
#[cfg(feature = "flash")]
pub mod flash;
pub mod gpio;
pub mod serial;
pub mod graphics;

#[cfg(target_arch = "wasm32")]
pub use gloo_timers::future::sleep;
#[cfg(feature = "tokio")]
pub use tokio::time::sleep;
