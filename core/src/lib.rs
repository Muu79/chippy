//! Chippy8 emulator
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use alloc::string::{ToString};

pub mod emu;
pub mod hardware;
pub mod isa;
pub mod config;
pub mod cpu;
pub mod display;
pub mod error;

pub fn parse_hex(hex: char) -> Result<u16, &'static str> {
    u16::from_str_radix(&hex.to_string(), 16).map_err(|_| "Invalid hex string")
}

pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next_rand(&mut self) -> u8 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 56) as u8
    }
}
