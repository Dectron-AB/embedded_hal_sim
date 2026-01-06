pub mod adc;
#[cfg(feature = "ethernet")]
pub mod ethernet;
#[cfg(feature = "flash")]
pub mod flash;
pub mod gpio;
pub mod serial;

#[cfg(not(target_arch = "wasm32"))]
pub use tokio::time::sleep;
#[cfg(target_arch = "wasm32")]
pub use gloo_timers::future::sleep;