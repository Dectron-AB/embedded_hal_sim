use std::sync::{Arc, atomic::AtomicBool};

use embedded_hal::digital::PinState;

pub struct Output {
    state: Arc<AtomicBool>,
}

pub struct OutputStimulus {
    state: Arc<AtomicBool>,
}

impl Output {
    pub fn new(initial_state: PinState) -> (Self, OutputStimulus) {
        let state = Arc::new(AtomicBool::new(initial_state == PinState::High));
        (
            Self {
                state: Arc::clone(&state),
            },
            OutputStimulus { state },
        )
    }
}

impl embedded_hal::digital::ErrorType for Output {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::OutputPin for Output {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.state.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.state.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

impl OutputStimulus {
    pub fn get(&mut self) -> PinState {
        match self.state.load(std::sync::atomic::Ordering::SeqCst) {
            true => PinState::High,
            false => PinState::Low,
        }
    }
}
