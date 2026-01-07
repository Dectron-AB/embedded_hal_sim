use crate::utils::SignalTx;
use embedded_hal::digital::PinState;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub struct Output {
    pub(crate) state: Arc<AtomicBool>,
    pub(crate) w: SignalTx<PinState>,
}

impl embedded_hal::digital::ErrorType for Output {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::OutputPin for Output {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.state.store(false, Ordering::SeqCst);
        self.w.signal(PinState::Low);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.state.store(true, Ordering::SeqCst);
        self.w.signal(PinState::High);
        Ok(())
    }
}
