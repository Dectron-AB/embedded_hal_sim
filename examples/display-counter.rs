///
/// Use this to run in browser
/// trunk serve --example morse --features=egui
///
/// Use this to run
mod utils;

use core::time::Duration;
use egui::TextureHandle;
use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::{Dimensions, GrayColor};
use embedded_graphics::text::Text;
use embedded_hal::digital::OutputPin;
use embedded_hal::digital::PinState;
use embedded_hal_sim::gpio::{self, Input, Output};
use embedded_hal_sim::{graphics, sleep};
use futures::select;
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
    use embedded_graphics::Drawable;
    use embedded_graphics::draw_target::DrawTarget;
    use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
    use embedded_graphics::prelude::Point;
    use embedded_graphics::text::Alignment;
    use embedded_hal_async::digital::Wait;
    use futures::FutureExt;

    let character_style = MonoTextStyle::new(&FONT_6X10, Gray8::WHITE);
    let mut counter = 0u32;

    let mut is_started = false;

    loop {
        display.clear(Gray8::BLACK).unwrap();

        // Draw centered text.
        let text = &format!("embedded_graphics\ndisplay in egui\ncounter: {counter}");
        Text::with_alignment(
            text,
            Point::new(display.bounding_box().center().x, 7),
            character_style,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();

        select! {
            _ = start_stop.wait_for_rising_edge().fuse() => is_started = !is_started,
            _ = reset.wait_for_rising_edge().fuse() => {
               counter = 0;
               is_started = false;
            },
            _ = sleep(Duration::from_millis(1000)).fuse() => if is_started {
                counter += 1;
            }
        }
    }
}

const COLS: usize = 128;
const ROWS: usize = 32;

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
                let size = self.texture.size_vec2();
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
