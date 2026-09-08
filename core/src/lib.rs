//! Chippy8 emulator
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod emu;
pub mod hardware;
pub mod isa;
pub mod config;
pub mod cpu;
pub mod display;
pub mod error;
mod rng;

