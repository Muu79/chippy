//! Chippy8 emulator
pub mod cpu;
pub mod hardware;

pub fn parse_hex(hex: char) -> Result<u16, &'static str> {
    u16::from_str_radix(&hex.to_string(), 16).map_err(|_| "Invalid hex string")
}
