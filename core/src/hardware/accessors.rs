use crate::emu::targets::{Quirk, Target};
use crate::hardware::cpu::{Cpu, VRegister};
use crate::hardware::{Display, Keyboard};

impl Cpu {
    pub fn get_quirk_map(&self) -> u16 {
        self.target_quirks.quirk_map
    }
    pub fn set_quirk_map(&mut self, quirk_map: u16) {
        self.target_quirks.quirk_map = quirk_map
    }
    pub fn has_quirk(&self, quirk: Quirk) -> bool {
        self.target_quirks.quirk_map & quirk as u16 != 0
    }
    pub fn set_quirk(&mut self, quirk: Quirk) {
        self.target_quirks.quirk_map |= quirk as u16
    }
    pub fn clear_quirk(&mut self, quirk: Quirk) {
        self.target_quirks.quirk_map &= !(quirk as u16)
    }
    pub fn get_pc(&self) -> u16 {
        self.pc
    }
    pub fn get_i_reg(&self) -> u16 {
        self.i_reg
    }
    pub fn get_v_regs(&self) -> &[u8] {
        &self.v_reg
    }
    pub fn get_stack(&self) -> &[u16] {
        &self.stack[..self.stack_ptr as usize]
    }
    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer
    }
    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer
    }
    pub fn get_target(&self) -> Target {
        self.target
    }
    pub fn get_display(&self) -> &Display {
        &self.display
    }
    pub fn get_rand_byte(&mut self) -> u8 {
        self.rng.next()
    }
    pub fn get_keys_mut(&mut self) -> &mut Keyboard {
        &mut self.keys
    }
    pub fn set_keyboard(&mut self, keys: Keyboard) {
        self.keys = keys;
    }
    pub fn get_reg(&self, reg: VRegister) -> &u8 {
        &self.v_reg[reg.0]
    }

    pub(super) fn get_reg_mut(&mut self, reg: VRegister) -> &mut u8 {
        &mut self.v_reg[reg.0]
    }

    pub fn display_dimensions(&self) -> (usize, usize) {
        (self.display.width, self.display.height)
    }
}