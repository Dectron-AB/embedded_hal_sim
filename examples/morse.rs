///
/// Use this to run in browser
/// trunk serve --example morse --features=egui
///
/// Use this to run
mod utils;

use core::time::Duration;
use egui::{Color32, Pos2};
use embedded_hal::digital::InputPin;
use embedded_hal::digital::{OutputPin, PinState};
use embedded_hal_sim::gpio::{self, Input};
use embedded_hal_sim::sleep;
use embedded_hal_sim::{
    gpio::Output,
    serial::{self, Uart, UartStimulus},
};

#[cfg(not(target_arch = "wasm32"))]
use eframe::EventLoopBuilderHook;
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;
#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;

const TIME_UNIT: Duration = Duration::from_millis(250);

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use crate::utils::run_wasm;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let (uart, uart_stimulus) = serial::Uart::new(Duration::from_millis(20), 10);
    let (led_stimulus, led) = gpio::new(PinState::Low);

    run_wasm(
        |_| MyApp {
            uart: uart_stimulus,
            led: led_stimulus,
            message: String::new(),
        },
        || async { simulated_app(uart, led).await },
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    use std::thread;

    let (uart, uart_stimulus) = serial::Uart::new(Duration::from_millis(20), 10);
    let (led_stimulus, led) = gpio::new(PinState::Low);

    thread::spawn(|| ui(uart_stimulus, led_stimulus));

    simulated_app(uart, led).await;
}

#[cfg(not(target_arch = "wasm32"))]
fn ui(uart: UartStimulus, led: Input) {
    let event_loop_builder: Option<EventLoopBuilderHook> = Some(Box::new(|event_loop_builder| {
        event_loop_builder.with_any_thread(true);
    }));
    let options = eframe::NativeOptions {
        event_loop_builder,
        ..Default::default()
    };
    eframe::run_native(
        "My hardware simulator",
        options,
        Box::new(|_| {
            Ok(Box::new(MyApp {
                uart,
                led,
                message: String::new(),
            }))
        }),
    )
    .unwrap();
}

async fn simulated_app(mut uart: Uart, mut led: Output) -> ! {
    let mut buf = [0; 256];
    loop {
        let count = uart.read_until_idle(&mut buf).await;
        let data = &buf[..count];
        for byte in data {
            blink_morse(byte.to_ascii_uppercase() as char, &mut led).await;
        }
    }
}

// https://en.wikipedia.org/wiki/Morse_code#/media/File:International_Morse_Code.svg
async fn blink_morse(character: char, led: &mut Output) {
    enum Blink {
        Dot,
        Dash,
    }
    use Blink::*;

    let seq: &[Blink] = match character {
        'A' => &[Dot, Dash],
        'B' => &[Dash, Dot, Dot, Dot],
        'C' => &[Dash, Dot, Dash, Dot],
        'D' => &[Dash, Dot, Dot],
        'E' => &[Dot],
        'F' => &[Dot, Dot, Dash, Dot],
        'G' => &[Dash, Dash, Dot],
        'H' => &[Dot, Dot, Dot, Dot],
        'I' => &[Dot, Dot],
        'J' => &[Dot, Dash, Dash, Dash],
        'K' => &[Dash, Dot, Dash],
        'L' => &[Dot, Dash, Dot, Dot],
        'M' => &[Dash, Dash],
        'N' => &[Dash, Dot],
        'O' => &[Dash, Dash, Dash],
        'P' => &[Dot, Dash, Dash, Dot],
        'Q' => &[Dash, Dash, Dot, Dash],
        'R' => &[Dot, Dash, Dot],
        'S' => &[Dot, Dot, Dot],
        'T' => &[Dash],
        'U' => &[Dot, Dot, Dash],
        'V' => &[Dot, Dot, Dot, Dash],
        'W' => &[Dot, Dash, Dash],
        'X' => &[Dash, Dot, Dot, Dash],
        'Y' => &[Dash, Dot, Dash, Dash],
        'Z' => &[Dash, Dash, Dot, Dot],

        '1' => &[Dot, Dash, Dash, Dash, Dash],
        '2' => &[Dot, Dot, Dash, Dash, Dash],
        '3' => &[Dot, Dot, Dot, Dash, Dash],
        '4' => &[Dot, Dot, Dot, Dot, Dash],
        '5' => &[Dot, Dot, Dot, Dot, Dot],
        '6' => &[Dash, Dot, Dot, Dot, Dot],
        '7' => &[Dash, Dash, Dot, Dot, Dot],
        '8' => &[Dash, Dash, Dash, Dot, Dot],
        '9' => &[Dash, Dash, Dash, Dash, Dot],
        '0' => &[Dash, Dash, Dash, Dash, Dash],

        ' ' => {
            sleep(7 * TIME_UNIT).await;
            return;
        }
        c => unimplemented!("{c:?}"),
    };

    for blink in seq {
        led.set_high().unwrap();
        let t = match blink {
            Dot => TIME_UNIT,
            Dash => 3 * TIME_UNIT,
        };
        sleep(t).await;
        led.set_low().unwrap();
        sleep(TIME_UNIT).await;
    }
    sleep(2 * TIME_UNIT).await;
}

struct MyApp {
    message: String,
    led: Input,
    uart: UartStimulus,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(20));
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My morse Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Message: ");
                ui.text_edit_singleline(&mut self.message)
                    .labelled_by(name_label.id);
            });
            if ui.button("Send").clicked()
                && let Ok(()) = self.uart.try_write(self.message.as_bytes())
            {
                self.message.clear();
            }

            let color = match self.led.is_high().unwrap() {
                true => Color32::GREEN,
                false => Color32::WHITE,
            };
            ui.painter()
                .circle_filled(Pos2::new(50.0, 150.0), 25.0, color);
        });
    }
}
