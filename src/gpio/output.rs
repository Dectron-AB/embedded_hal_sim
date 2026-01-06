use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

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
        self.state.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.state.store(true, Ordering::SeqCst);
        Ok(())
    }
}

impl OutputStimulus {
    pub fn get(&mut self) -> PinState {
        match self.state.load(Ordering::SeqCst) {
            true => PinState::High,
            false => PinState::Low,
        }
    }
}
