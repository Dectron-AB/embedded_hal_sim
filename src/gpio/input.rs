use embedded_hal::digital::{InputPin, PinState};
use std::sync::{Arc, atomic::AtomicBool};

use crate::utils::SignalRx;
pub struct Input {
    pub(crate) state: Arc<AtomicBool>,
    pub(crate) w: SignalRx<PinState>,
}

impl embedded_hal::digital::ErrorType for Input {
    type Error = core::convert::Infallible;
}

impl InputPin for Input {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.state.load(std::sync::atomic::Ordering::SeqCst))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

impl embedded_hal_async::digital::Wait for Input {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        if self.is_high()? {
            return Ok(());
        }
        loop {
            if self.w.wait().await == PinState::High {
                return Ok(());
            }
        }
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        if self.is_low()? {
            return Ok(());
        }
        loop {
            if self.w.wait().await == PinState::Low {
                return Ok(());
            }
        }
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await?;
        self.wait_for_high().await
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await?;
        self.wait_for_low().await
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        match self.is_high()? {
            true => self.wait_for_low().await,
            false => self.wait_for_high().await,
        }
    }
}
