use crate::config::quirks::{Quirk, Quirks};
use crate::cpu::{Cpu, VRegister};
use crate::hardware::Keyboard;

impl Cpu {
    /// Set (Overwrite) the current quirk profile for the [Cpu]
    pub fn set_quirks(&mut self, new_quirks: Quirks) {
        self.target_quirks = new_quirks;
    }
    /// Enable a specific [Quirk] for the [Cpu]
    pub fn set_quirk(&mut self, quirk: Quirk) {
        self.target_quirks |= quirk
    }
    /// Disable a specific [Quirk] for the [Cpu]
    pub fn clear_quirk(&mut self, quirk: Quirk) {
        self.target_quirks -= quirk
    }
    pub fn get_rand_byte(&mut self) -> u8 {
        self.rng.next_rand()
    }
    pub fn get_keys_mut(&mut self) -> &mut Keyboard {
        &mut self.keys
    }
    pub fn set_keyboard(&mut self, keys: Keyboard) {
        self.keys = keys;
    }
    pub(super) fn get_reg_mut(&mut self, reg: VRegister) -> &mut u8 {
        &mut self.v_reg[reg.0]
    }
}