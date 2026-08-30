use crate::cpu::Target;
use std::fmt::Write;
use std::iter::{Iterator, repeat_n};

/// A structure representing a graphical display with a fixed-size buffer.
///
/// # Fields
///
/// * `buffer` - An array of 64 `u128` values used to store the pixel data of the display.
///   This serves as the internal representation of the display's graphical content.
///
/// * `height` - The height of the display, specified in pixels. This field is publicly accessible
///   within the current crate.
///
/// * `width` - The width of the display, specified in pixels. This field is also publicly accessible
///   within the current crate.
///
/// * `capacity` - The total capacity of the display's buffer, which defines the maximum number of
///   pixels that the display can accommodate. This field is publicly accessible within the current crate.
///
/// # Notes
///
/// This structure is primarily used to handle and store graphical data and dimensions of the display.
/// Access to the `height`, `width`, and `capacity` fields is restricted to the current crate, while
/// the `buffer` remains private to encapsulate the display's graphical state.
pub struct Display {
    buffer: [u128; 64],
    pub(crate) width: usize,
    pub(crate) height: usize,
    is_extended: bool,
}

impl Display {
    /// Create a new [`Display`] based on the supplied [`Target`]
    pub fn new(target: &Target) -> Display {
        Display {
            buffer: [0u128; 64],
            width: 64,
            height: 32,
            is_extended: false,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.buffer.fill(0);
    }
    pub fn get_screen(&self) -> &[u128] {
        &self.buffer
    }
    pub fn get_screen_mut(&mut self) -> &mut [u128] {
        &mut self.buffer
    }

    pub fn enter_hi_res(&mut self) {
        self.width = 128;
        self.height = 64;
        self.clear();
        self.is_extended = true;
    }

    pub fn enter_lo_res(&mut self) {
        self.width = 64;
        self.height = 32;
        self.clear();
        self.is_extended = false;
    }

    pub fn is_extended(&self) -> bool {
        self.is_extended
    }

    pub fn dimensions(&self) -> (usize, usize, usize) {
        (self.width, self.height, self.width * self.height)
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn draw_byte(&mut self, row: usize, col: usize, byte: u8) -> u8 {
        let row = row % self.height;
        let col = col % self.width;
        let mask = (byte.reverse_bits() as u128) << col;
        let buf_line = &mut self.buffer[row];
        let collisions = (*buf_line & mask).count_ones() as u8;
        *buf_line ^= mask;
        collisions
    }

    pub fn draw_chomp(&mut self, row: usize, col: usize, chomp: u16) -> u8 {
        let row = row % self.height;
        let col = col % self.width;
        let mask = (chomp.reverse_bits() as u128) << col;
        let buf_line = &mut self.buffer[row];
        let collision = if col + 16 >= self.width { 1 } else { 0 };
        *buf_line ^= mask;
        collision
    }

    pub fn draw_n_bytes(
        &mut self,
        mut row: usize,
        col: usize,
        n: usize,
        bytes: impl Iterator<Item = u8>,
    ) -> u8 {
        let mut collision_count = if self.height > row + n {
            ((row + n) - self.height) as u8
        } else {
            0
        };
        for byte in bytes.take(n) {
            if row >= self.height {
                break;
            }
            let mask = (byte.reverse_bits() as u128) << col;
            let buf_line = &mut self.buffer[row];
            collision_count += (*buf_line & mask).count_ones() as u8;
            *buf_line ^= mask;
            row += 1;
        }
        collision_count
    }
    // pub fn draw_n_chomps(
    //     &mut self,
    //     row: usize,
    //     col: usize,
    //     n: usize,
    //     chomps: impl Iterator<Item = u16>,
    // ) -> u8 {
    //     let mut collision_count = 0;
    //     for chomp in chomps.take(n) {}
    // }

    pub fn draw_sprite(
        &mut self,
        row: usize,
        col: usize,
        sprite: &Sprite,
        ram: &[u8],
    ) -> Result<bool, &'static str> {
        let sprite_start = *sprite as usize;
        let mut collision_count = 0;
        let mut line = 0;
        let (row, col) = (row % 32, col % 64);
        loop {
            let byte = ram
                .get(sprite_start + line)
                .ok_or("Invalid sprite address")?;
            collision_count += self.draw_byte(row + line, col, *byte);
            line += 1;
            if line >= 5 || (line + row) >= self.height {
                break;
            }
        }
        Ok(collision_count > 0)
    }
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('\u{250C}')?;
        f.write_str(&repeat_n('\u{2500}', 64).collect::<String>())?;
        f.write_char('\u{2510}')?;
        f.write_char('\n')?;
        for row in 0..self.height {
            f.write_char('\u{2502}')?;
            let line = self.buffer[row];
            f.write_str(&(0..self.width).fold(String::new(), |mut acc, offset| {
                if (1 << offset) & line != 0 {
                    acc.push('*')
                } else {
                    acc.push(' ')
                };
                acc
            }))?;
            f.write_char('\u{2502}')?;
            f.write_char('\n')?;
        }
        f.write_char('\u{2514}')?;
        f.write_str(&repeat_n('\u{2500}', 64).collect::<String>())?;
        f.write_char('\u{2518}')?;
        Ok(())
    }
}

pub const CHAR_MAP: [u8; 240] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    0xFF, 0xFF, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, // 0
    0x18, 0x78, 0x78, 0x18, 0x18, 0x18, 0x18, 0x18, 0xFF, 0xFF, // 1
    0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, // 2
    0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, // 3
    0xC3, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, 0x03, 0x03, 0x03, 0x03, // 4
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, // 5
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, // 6
    0xFF, 0xFF, 0x03, 0x03, 0x06, 0x0C, 0x18, 0x18, 0x18, 0x18, // 7
    0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, // 8
    0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, // 9
    0x7E, 0xFF, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, 0xC3, 0xC3, 0xC3, // A
    0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, // B
    0x3C, 0xFF, 0xC3, 0xC0, 0xC0, 0xC0, 0xC0, 0xC3, 0xFF, 0x3C, // C
    0xFC, 0xFE, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xFE, 0xFC, // D
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, // E
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xC0, 0xC0, // F
];

#[derive(Copy, Clone, Default)]
pub enum Sprite {
    Zero = 0x00,
    One = 0x05,
    Two = 0x0A,
    Three = 0x0F,
    Four = 0x14,
    Five = 0x19,
    Six = 0x1E,
    Seven = 0x23,
    Eight = 0x28,
    Nine = 0x2D,
    A = 0x32,
    B = 0x37,
    C = 0x3C,
    D = 0x41,
    E = 0x46,
    F = 0x4B,
    BigZero = 0x50,
    BigOne = 0x5A,
    BigTwo = 0x64,
    BigThree = 0x6E,
    BigFour = 0x78,
    BigFive = 0x82,
    BigSix = 0x8C,
    BigSeven = 0x96,
    BigEight = 0xA0,
    BigNine = 0xAA,
    BigA = 0xB4,
    BigB = 0xBE,
    BigC = 0xC8,
    BigD = 0xD2,
    BigE = 0xDC,
    BigF = 0xE6,
    #[default]
    Unknown = 0x1FB, // 5 bytes before ROM start, should be blank
}

impl Sprite {
    pub fn from_hex(hex: u8, is_extended: bool) -> Result<Self, &'static str> {
        match hex {
            0x0 => Ok(if is_extended {
                Self::BigZero
            } else {
                Self::Zero
            }),
            0x1 => Ok(if is_extended { Self::BigOne } else { Self::One }),
            0x2 => Ok(if is_extended { Self::BigTwo } else { Self::Two }),
            0x3 => Ok(if is_extended {
                Self::BigThree
            } else {
                Self::Three
            }),
            0x4 => Ok(if is_extended {
                Self::BigFour
            } else {
                Self::Four
            }),
            0x5 => Ok(if is_extended {
                Self::BigFive
            } else {
                Self::Five
            }),
            0x6 => Ok(if is_extended { Self::BigSix } else { Self::Six }),
            0x7 => Ok(if is_extended {
                Self::BigSeven
            } else {
                Self::Seven
            }),
            0x8 => Ok(if is_extended {
                Self::BigEight
            } else {
                Self::Eight
            }),
            0x9 => Ok(if is_extended {
                Self::BigNine
            } else {
                Self::Nine
            }),
            0xA => Ok(if is_extended { Self::BigA } else { Self::A }),
            0xB => Ok(if is_extended { Self::BigB } else { Self::B }),
            0xC => Ok(if is_extended { Self::BigC } else { Self::C }),
            0xD => Ok(if is_extended { Self::BigD } else { Self::D }),
            0xE => Ok(if is_extended { Self::BigE } else { Self::E }),
            0xF => Ok(if is_extended { Self::BigF } else { Self::F }),
            _ => Err("Invalid sprite"),
        }
    }
}

macro_rules! impl_sprite_from_int {
    ($($t:ty),* $(,)?) => {
        $(
            impl From<$t> for Sprite {
                fn from(num: $t) -> Self {
                    Self::from_hex((num & 0xF) as u8, false).unwrap_or(Sprite::Unknown) // defaults to small chars
                }
            }
        )*
    }
}

impl_sprite_from_int!(u8, u16, u32, u64, i8, i16, i32, i64);
impl From<char> for Sprite {
    fn from(char: char) -> Self {
        match char {
            '0' => Self::Zero,
            '1' => Self::One,
            '2' => Self::Two,
            '3' => Self::Three,
            '4' => Self::Four,
            '5' => Self::Five,
            '6' => Self::Six,
            '7' => Self::Seven,
            '8' => Self::Eight,
            '9' => Self::Nine,
            'a' | 'A' => Self::A,
            'b' | 'B' => Self::B,
            'c' | 'C' => Self::C,
            'd' | 'D' => Self::D,
            'e' | 'E' => Self::E,
            'f' | 'F' => Self::F,
            _ => Self::Unknown,
        }
    }
}
