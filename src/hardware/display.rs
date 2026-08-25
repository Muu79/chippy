use crate::cpu::Target;
use std::iter::repeat_n;

impl Target {
    pub const fn screen_width(&self) -> usize {
        match self {
            Target::Chip8 | Target::SChip8Modern | Target::SChip8Classic => 64,
        }
    }
    pub const fn screen_height(&self) -> usize {
        match self {
            Target::Chip8 | Target::SChip8Modern | Target::SChip8Classic => 32,
        }
    }

    pub const fn get_dimensions(&self) -> (usize, usize) {
        (self.screen_width(), self.screen_height())
    }
}
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
    pub(crate) height: usize,
    pub(crate) width: usize,
    pub(crate) capacity: usize,
}

impl Display {
    /// Create a new [`Display`] based on the supplied [`Target`]
    pub fn new(target: &Target) -> Display {
        Display {
            buffer: [0u128; 64],
            height: target.screen_height(),
            width: target.screen_width(),
            capacity: target.screen_width() * target.screen_height(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.buffer.fill(0);
    }
    pub fn get_screen(&self) -> &[u128] {
        &self.buffer
    }

    pub fn dimensions(&self) -> (usize, usize, usize) {
        (self.width, self.height, self.capacity)
    }
    
    pub fn get_height(&self) -> usize {
        self.height
    }
    
    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn to_string(&self) -> String {
        let mut output = String::new();
        output.push('\u{250C}');
        output.extend(repeat_n('\u{2500}', 64));
        output.push('\u{2510}');
        output.push('\n');
        for row in 0..self.height {
            output.push('\u{2502}');
            let line = self.buffer[row];
            output.extend((0..64).map(|offset| if (1 << offset) & line != 0 { '*' } else { ' ' }));
            output.push('\u{2502}');
            output.push('\n');
        }
        output.push('\u{2514}');
        output.extend(repeat_n('\u{2500}', 64));
        output.push('\u{2518}');
        output
    }
    pub fn draw_byte(&mut self, row: usize, col: usize, byte: u8) -> Result<bool, &'static str> {
        let row = row % self.height;
        let col = (col % self.width) as u64;
        let mask = (byte.reverse_bits() as u128) << col;
        let buf_line = &mut self.buffer[row];
        let vf = (*buf_line & mask) != 0;
        *buf_line ^= mask;
        Ok(vf)
    }

    pub fn draw_sprite(
        &mut self,
        row: usize,
        col: usize,
        sprite: &Sprite,
        ram: &[u8],
    ) -> Result<bool, &'static str> {
        let sprite_start = *sprite as usize;
        let mut vf = false;
        let mut line = 0;
        let (row, col) = (row % 32, col % 64);
        loop {
            let byte = ram
                .get(sprite_start + line)
                .ok_or("Invalid sprite address")?;
            vf |= self.draw_byte(row + line, col, *byte)?;
            line += 1;
            if line >= 5 || (line + row) >= self.height {
                break;
            }
        }
        Ok(vf)
    }
}

pub static CHAR_MAP: [u8; 80] = [
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
    #[default]
    Unknown = 0x1FB, // 5 bytes before ROM start, should be blank
}

impl Sprite {
    pub fn from_hex(hex: u8) -> Result<Self, &'static str> {
        match hex {
            0x0 => Ok(Self::Zero),
            0x1 => Ok(Self::One),
            0x2 => Ok(Self::Two),
            0x3 => Ok(Self::Three),
            0x4 => Ok(Self::Four),
            0x5 => Ok(Self::Five),
            0x6 => Ok(Self::Six),
            0x7 => Ok(Self::Seven),
            0x8 => Ok(Self::Eight),
            0x9 => Ok(Self::Nine),
            0xA => Ok(Self::A),
            0xB => Ok(Self::B),
            0xC => Ok(Self::C),
            0xD => Ok(Self::D),
            0xE => Ok(Self::E),
            0xF => Ok(Self::F),
            _ => Err("Invalid sprite"),
        }
    }
}

macro_rules! impl_sprite_from_int {
    ($($t:ty),* $(,)?) => {
        $(
            impl From<$t> for Sprite {
                fn from(num: $t) -> Self {
                    Self::from_hex((num & 0xF) as u8).unwrap_or(Sprite::Unknown)
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
