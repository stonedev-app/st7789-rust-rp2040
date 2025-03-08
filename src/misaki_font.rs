use core::fmt::Debug;
use defmt::info;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor, *},
    primitives::Rectangle,
};

// Font width(pixel)
const WIDTH: u32 = 4;
// Font height(pixel)
const HEIGHT: u32 = 8;
// Font Scale
const SCALE: u32 = 2;

// Font Data
const FONT_DATA: &[u8] = include_bytes!("../font/misaki_4x8.raw");

/// MisakiFontText
pub struct MisakiFontText<'a> {
    text: &'a str,
    position: Point,
}

/// MisakiFontText
impl<'a> MisakiFontText<'a> {
    /// constructor
    pub fn new(text: &'a str, position: Point) -> Self {
        MisakiFontText { text, position }
    }

    /// draw
    pub fn draw<D>(&mut self, target: &mut D)
    where
        D: DrawTarget<Color = Rgb565, Error: Debug>,
    {
        info!("{}", self.text);
        info!("x:{} y:{}", self.position.x, self.position.y);

        // Process one character at a time
        for ch in self.text.chars() {
            // One character
            let byte = self.get_byte(ch);
            // Upper 4 bits
            let upper = byte >> 4;
            // Lower 4 bits
            let lower = byte & 0b00001111;

            info!("{} 0x{:02X} 0x{:01X}|0x{:01X}", ch, byte, upper, lower);

            // Length of one horizontal row (in bytes)
            // Character width(in pixels) * 16 characters / 8 bits.
            let length: u32 = WIDTH * 16 / 8;

            // Get the starting position
            let mut start_index: u32 =
                (length * HEIGHT * upper as u32) + (WIDTH * lower as u32 / 8);

            // Rendering data for one character
            let mut data: [u8; HEIGHT as usize] = [0x00; HEIGHT as usize];

            // Retrieve data one line at a time
            for item in data.iter_mut().take(HEIGHT as usize) {
                *item = FONT_DATA[start_index as usize];
                // In the case of the upper 4 bits
                if lower % 2 == 0 {
                    *item >>= 4;
                // In the case of the lower 4 bits
                } else {
                    *item &= 0b00001111;
                }
                // Move to the index of the next line
                start_index += length;
            }
            info!("{:04b}", data);

            for (i, item) in data.iter().enumerate() {
                info!("0b{:04b}", *item);
                for j in 0..WIDTH {
                    let bitmask: u8 = 1 << (WIDTH - 1 - j);
                    if bitmask & *item > 0 {
                        target
                            .fill_solid(
                                &Rectangle::new(
                                    Point::new(
                                        self.position.x + j as i32 * SCALE as i32,
                                        self.position.y + i as i32 * SCALE as i32,
                                    ),
                                    Size::new(SCALE, SCALE),
                                ),
                                Rgb565::WHITE,
                            )
                            .unwrap();
                    }
                }
            }

            // Move to the position of the next character
            self.position.x += WIDTH as i32 * SCALE as i32;
        }
    }

    /// get_byte
    fn get_byte(&self, ch: char) -> u8 {
        // Convert char to byte
        let mut buf = [0u8; 4];
        ch.encode_utf8(&mut buf);
        buf[0]
    }
}
