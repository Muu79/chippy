use std::fmt::Formatter;
use std::ops::{BitOrAssign, BitXorAssign};

#[derive(Clone, Copy)]
pub struct Keyboard {
    map: u16,
    pub input_key: Option<u8>,
}
const INVALID_KEY_ERROR: &str = "Invalid key, only hexadecimal keys are supported";
impl Keyboard {
    pub fn new() -> Self {
        Self {
            map: 0,
            input_key: None,
        }
    }

    pub fn reset(&mut self) {
        self.map = 0;
        self.input_key = None;
    }

    pub fn press(&mut self, key: u8) -> Result<(), &'static str> {
        if key > 15 {
            return Err(INVALID_KEY_ERROR);
        }
        if self.input_key.is_none() {
            self.input_key = Some(key);
        }
        Ok(self.map |= 1 << key)
    }

    pub fn release(&mut self, key: u8) -> Result<(), &'static str> {
        if key > 15 {
            return Err(INVALID_KEY_ERROR);
        }
        Ok(self.map &= !(1 << key))
    }

    pub fn is_pressed(&self, key: u8) -> Result<bool, &'static str> {
        if key > 15 {
            return Err(INVALID_KEY_ERROR);
        }
        Ok((self.map & (1 << key)) != 0)
    }

    pub fn as_input_key(&self) -> Option<u8> {
        self.input_key
    }

    pub fn reset_input_key(&mut self) {
        self.input_key = None;
    }
}

impl std::fmt::Display for Keyboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..16 {
            write!(
                f,
                "{:X}: {}",
                i,
                if self.is_pressed(i).unwrap_or(false) {
                    'X'
                } else {
                    ' '
                }
            )?;
        }
        Ok(())
    }
}
impl From<u16> for Keyboard {
    fn from(num: u16) -> Self {
        Self {
            map: num,
            input_key: None,
        }
    }
}

impl BitXorAssign for Keyboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.map ^= rhs.map;
    }
}

impl BitOrAssign for Keyboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.map |= rhs.map;
    }
}
