pub mod ssd1322;
use parking_lot::Mutex;
use std::sync::Arc;

use embedded_graphics::{
    Pixel, pixelcolor::Gray4, prelude::{DrawTarget, GrayColor, OriginDimensions, Size}
};

pub type ImageBuffer<const COLS: usize, const ROWS: usize> = Arc<Mutex<[[u8; COLS]; ROWS]>>;

pub struct Display<const COLS: usize, const ROWS: usize> {
    pub framebuffer: ImageBuffer<COLS, ROWS>,
}

impl<const COLS: usize, const ROWS: usize> Display<COLS, ROWS> {
    pub fn new() -> (Self, ImageBuffer<COLS, ROWS>) {
        let framebuffer = Arc::new(Mutex::new([[0; _]; _]));
        (
            Self {
                framebuffer: Arc::clone(&framebuffer),
            },
            framebuffer,
        )
    }
}

impl<const COLS: usize, const ROWS: usize> DrawTarget for Display<COLS, ROWS> {
    type Color = Gray4;
    // `ExampleDisplay` uses a framebuffer and doesn't need to communicate with the display
    // controller to draw pixel, which means that drawing operations can never fail. To reflect
    // this the type `Infallible` was chosen as the `Error` type.
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // The `DrawTarget` implementation is required to discard any out of bounds
            // pixels without returning an error or causing a panic.

            // Calculate the index in the framebuffer.
            if let Some(p) = self
                .framebuffer
                .lock()
                .get_mut(coord.y as usize)
                .and_then(|row| row.get_mut(coord.x as usize))
            {
                *p = 16 * color.luma();
            }
        }
        Ok(())
    }
}

impl<const COLS: usize, const ROWS: usize> OriginDimensions for Display<COLS, ROWS> {
    fn size(&self) -> Size {
        Size::new(COLS as u32, ROWS as u32)
    }
}