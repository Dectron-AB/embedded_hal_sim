///
/// Use this to run in browser
/// trunk serve --example morse --features=egui
///
/// Use this to run
mod utils;

use core::time::Duration;
use egui::TextureHandle;
use embassy_futures::select::select3;
use embedded_graphics::{
    pixelcolor::Gray8,
    prelude::{Dimensions, GrayColor},
    text::Text,
};
use embedded_hal::digital::{OutputPin, PinState};
use embedded_hal_sim::{
    gpio::{self, Input, Output},
    graphics, sleep,
};
use parking_lot::Mutex;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use eframe::EventLoopBuilderHook;
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;
#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use crate::utils::run_wasm;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let (reset, reset_stimulus) = gpio::new(PinState::Low);
    let (start_stop, start_stop_stimulus) = gpio::new(PinState::Low);
    let (display, frame_buffer) = graphics::Display::new();

    run_wasm(
        |cc| MyApp::new(cc, frame_buffer, reset_stimulus, start_stop_stimulus),
        || async { simulated_app(display, reset, start_stop).await },
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    use std::thread;

    let (reset, reset_stimulus) = gpio::new(PinState::Low);
    let (start_stop, start_stop_stimulus) = gpio::new(PinState::Low);
    let (display, frame_buffer) = graphics::Display::new();

    thread::spawn(|| ui(frame_buffer, reset_stimulus, start_stop_stimulus));

    simulated_app(display, reset, start_stop).await;
}

#[cfg(not(target_arch = "wasm32"))]
fn ui(frame_buffer: Arc<Mutex<[[u8; COLS]; ROWS]>>, reset: Output, start_stop: Output) {
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
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, frame_buffer, reset, start_stop)))),
    )
    .unwrap();
}

async fn simulated_app(
    mut display: graphics::Display<COLS, ROWS>,
    mut reset: Input,
    mut start_stop: Input,
) -> ! {
    use embedded_graphics::{
        Drawable,
        draw_target::DrawTarget,
        mono_font::{MonoTextStyle, ascii::FONT_6X10, iso_8859_1::FONT_10X20},
        prelude::Point,
        text::Alignment,
    };
    use embedded_hal_async::digital::Wait;

    let style_small = MonoTextStyle::new(&FONT_6X10, Gray8::WHITE);
    let style_big = MonoTextStyle::new(&FONT_10X20, Gray8::WHITE);
    let mut counter = 0.0f32;

    let mut is_started = false;

    loop {
        display.clear(Gray8::BLACK).unwrap();

        // Draw centered text.
        Text::with_alignment(
            "embedded_graphics\ndisplay in egui",
            Point::new(display.bounding_box().center().x, 10),
            style_small,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();

        Text::with_alignment(
            &heapless::format!(32; "counter: {counter:.1}").unwrap(),
            Point::new(display.bounding_box().center().x, 42),
            style_big,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();

        match select3(
            start_stop.wait_for_rising_edge(),
            reset.wait_for_rising_edge(),
            sleep(Duration::from_millis(100)),
        )
        .await
        {
            embassy_futures::select::Either3::First(_) => is_started = !is_started,
            embassy_futures::select::Either3::Second(_) => {
                counter = 0.0;
                is_started = false;
            }
            embassy_futures::select::Either3::Third(_) => {
                if is_started {
                    counter += 0.1;
                }
            }
        }
    }
}

const COLS: usize = 256;
const ROWS: usize = 64;

struct MyApp {
    frame_buffer: Arc<Mutex<[[u8; COLS]; ROWS]>>,
    reset: Output,
    start_stop: Output,
    texture: TextureHandle,
}

impl MyApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        frame_buffer: Arc<Mutex<[[u8; COLS]; ROWS]>>,
        reset: Output,
        start_stop: Output,
    ) -> Self {
        Self {
            frame_buffer,
            reset,
            start_stop,
            texture: cc.egui_ctx.load_texture(
                "noise",
                egui::ColorImage::example(),
                egui::TextureOptions::NEAREST,
            ),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(20));
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My display-counter Application");
            {
                let image = egui::ColorImage::from_gray(
                    [COLS, ROWS],
                    self.frame_buffer.lock().as_flattened(),
                );
                self.texture.set(image, egui::TextureOptions::NEAREST);
                let size = self.texture.size_vec2() * 4.0;
                let sized_texture = egui::load::SizedTexture::new(&self.texture, size);
                ui.add(egui::Image::new(sized_texture).fit_to_exact_size(size));
            }

            self.reset
                .set_state(PinState::from(
                    ui.button("Reset").is_pointer_button_down_on(),
                ))
                .unwrap();

            self.start_stop
                .set_state(PinState::from(
                    ui.button("Start/Stop").is_pointer_button_down_on(),
                ))
                .unwrap();
        });
    }
}
