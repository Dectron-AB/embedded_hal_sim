use std::sync::{Arc, atomic::AtomicU16};

pub struct Adc;

impl Adc {
    pub fn read(&mut self, channel: &AdcChannel) -> u16 {
        channel.value.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// A simulated adc channel
///
/// The value read by this channel is the same
/// as was stored in the corresponding [AdcChannelStimulus]
pub struct AdcChannel {
    value: Arc<AtomicU16>,
}

pub struct AdcChannelStimulus {
    value: Arc<AtomicU16>,
}

impl AdcChannel {
    pub fn new(initial_value: u16) -> (Self, AdcChannelStimulus) {
        let value = Arc::new(AtomicU16::new(initial_value));
        (
            Self {
                value: Arc::clone(&value),
            },
            AdcChannelStimulus { value },
        )
    }
}

impl AdcChannelStimulus {
    pub fn set(&mut self, value: u16) {
        self.value.store(value, std::sync::atomic::Ordering::SeqCst);
    }
}
