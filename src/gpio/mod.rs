use embedded_hal::digital::PinState;
use std::sync::{Arc, atomic::AtomicBool};

pub use input::Input;
pub use output::Output;

use crate::utils;

mod input;
mod output;

pub fn new(initial_state: PinState) -> (Input, Output) {
    let state = Arc::new(AtomicBool::new(initial_state == PinState::High));
    let (tx, rx) = utils::signal(initial_state);
    (
        Input {
            state: Arc::clone(&state),
            w: rx,
        },
        Output { state, w: tx },
    )
}
