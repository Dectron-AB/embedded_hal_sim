use std::sync::{Arc, atomic::AtomicBool};

use embedded_hal::digital::PinState;

pub struct Input {
    state: Arc<AtomicBool>,
}

pub struct InputStimulus {
    state: Arc<AtomicBool>,
}

impl Input {
    pub fn new(initial_state: PinState) -> (Self, InputStimulus) {
        let state = Arc::new(AtomicBool::new(initial_state == PinState::High));
        (
            Self {
                state: Arc::clone(&state),
            },
            InputStimulus { state },
        )
    }
}

impl embedded_hal::digital::ErrorType for Input {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::InputPin for Input {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.state.load(std::sync::atomic::Ordering::SeqCst))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

impl InputStimulus {
    pub fn set(&mut self, state: PinState) {
        self.state
            .store(state == PinState::High, std::sync::atomic::Ordering::SeqCst);
    }
}
