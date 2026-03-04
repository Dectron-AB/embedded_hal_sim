use std::convert::Infallible;

use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::pixelcolor::raw::RawU4;
use embedded_graphics::prelude::{Dimensions, DrawTarget, GrayColor, OriginDimensions, Point, RawData, Size};
use embedded_hal_async::delay::DelayNs;

pub struct Frame<const N: usize> {
    pub width: u16,
    pub height: u16,
    pub buffer: [u8; N],
}

impl<const N: usize> Frame<N> {
    /// Creates a new `Frame` with the specified dimensions and buffer.
    ///
    /// # Parameters
    ///
    /// - `width`: The width of the frame buffer in pixels.
    /// - `height`: The height of the frame buffer in pixels.
    /// - `orientation`: The orientation of the display.
    /// - `buffer`: The buffer storing the pixel data.
    ///
    /// # Returns
    ///
    /// A new `Frame` instance.
    pub fn new(width: u16, height: u16, buffer: [u8; N]) -> Self {
        Self {
            width,
            height,
            buffer,
        }
    }

    /// Sets a pixel in the frame buffer to the specified color.
    ///
    /// # Parameters
    ///
    /// - `x`: The x-coordinate of the pixel.
    /// - `y`: The y-coordinate of the pixel.
    /// - `color`: The greyscale colour to set the pixel to.
    pub fn set_pixel(&mut self, x: u8, y: u8, color: Gray4) {
        let color = RawU4::from(color).into_inner();
        if x as usize >= self.width as usize || y as usize >= self.height as usize {
            return;
        }
        let idx = ((y as usize) * self.width as usize + (x as usize)) / 2;
        if idx >= self.buffer.len() {
            return;
        }

        // Extract the 4-bit color value
        let color = color & 0x0F;

        if x % 2 == 0 {
            // Set the higher 4 bits for even x coordinate
            self.buffer[idx] = (self.buffer[idx] & 0x0F) | (color << 4);
        } else {
            // Set the lower 4 bits for odd x coordinate
            self.buffer[idx] = (self.buffer[idx] & 0xF0) | color;
        }
    }
}

impl<const N: usize> DrawTarget for Frame<N> {
    type Error = ();
    type Color = Gray4;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();
        pixels
            .into_iter()
            .filter(|Pixel(pos, _color)| bb.contains(*pos))
            .for_each(|Pixel(pos, color)| self.set_pixel(pos.x as u8, pos.y as u8, color));
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let c = RawU4::from(color).into_inner();
        let packed_color = (c << 4) | c; // Pack two 4-bit color values into one byte
        for i in 0..self.buffer.len() {
            self.buffer[i] = packed_color;
        }
        Ok(())
    }
}

impl<const N: usize> OriginDimensions for Frame<N> {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

pub struct Display<const COLS: usize, const ROWS: usize> {
    pub display: super::Display<COLS, ROWS>,
}

impl<const COLS: usize, const ROWS: usize> Display<COLS, ROWS> {
    pub async fn flush_frame<const N: usize>(
        &mut self,
        frame: &Frame<N>,
    ) -> Result<(), Infallible> {
        let pixels = frame.buffer.into_iter().enumerate().map(|(i,m)| {
            let c0 = Gray4::new(m >> 4);
            let c1 = Gray4::new(m & 0x0F);
            let i0 = i * 2;
            let i1 = i * 2 + 1;
            let y0 = i0 / COLS;
            let x0 = i0 % COLS;
            let y1 = i1 / COLS;
            let x1 = i1 % COLS;
            let pos0 = Point::new(x0 as i32, y0 as i32);
            let pos1 = Point::new(x1 as i32, y1 as i32);
            [Pixel(pos0,c0),Pixel(pos1, c1)]
        }).flatten();
        self.display.draw_iter(pixels);
        Ok(())
    }

    pub async fn init_default<D>(&mut self, _delay: &mut D) -> Result<(), Infallible>
    where
        D: DelayNs,
    {
        Ok(())
    }
}
