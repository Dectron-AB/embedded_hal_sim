use std::time::Duration;

use embedded_hal::digital::{OutputPin, PinState};
use embedded_hal_sim::{
    gpio::{
        self,
        output::{Output, OutputStimulus},
    },
    serial::{self, UartStimulus},
};
use tokio::time::sleep;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (mut uart, stimulus) = serial::Uart::new(Duration::from_millis(20), 10);
    let (mut led, led_stimulus) = Output::new(PinState::Low);

    tokio::spawn(stimulus_task(stimulus, led_stimulus));

    let mut buf = [0; 256];
    loop {
        let count = uart.read_until_idle(&mut buf).await;
        let data = &buf[..count];
        for byte in data {
            blink_morse(*byte as char, &mut led).await;
        }
    }
}

async fn blink_morse(character: char, led: &mut Output) {
    enum Blink {
        Short,
        Long,
    }
    use Blink::*;

    let seq = match character {
        's' => &[Short],
        'o' => &[Long],
        _ => unimplemented!(),
    };

    for blink in seq {
        led.set_high();
        match blink {
            Short => sleep(Duration::from_millis(250)).await,
            Long => sleep(Duration::from_millis(500)).await,
        }
        led.set_low();
        sleep(Duration::from_millis(250)).await;
    }
}

async fn stimulus_task(uart: UartStimulus, led_stimulus: OutputStimulus) -> ! {
    // TODO: Use egui to have a prompt for sending uart data and a colored box for the led status
    loop {
        sleep(Duration::from_millis(1000)).await;
    }
}
