use crate::config::quirks::{Quirk, Quirks};
use crate::config::target::Target;
use crate::cpu::{Cpu, VRegister};
use crate::display::Display;
use crate::hardware::Keyboard;

impl Cpu {
    /// Get the current quirk profile for the [Cpu]
    pub fn get_quirks(&self) -> Quirks {
        self.target_quirks
    }
    
    /// Check if the [Cpu] has a specific [Quirk] enabled
    pub fn has_quirk(&self, quirk: Quirk) -> bool {
        self.target_quirks.contains(quirk)
    }
    
    pub fn get_pc(&self) -> u16 {
        self.pc
    }
    pub fn get_i_reg(&self) -> u16 {
        self.i_reg
    }
    /// Get a slice over the [Cpu]'s V registers from 0 to F
    pub fn get_v_regs(&self) -> &[u8] {
        &self.v_reg
    }
    /// Get a slice over the [Cpu]'s active stack
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

    pub fn get_reg(&self, reg: VRegister) -> u8 {
        self.v_reg[reg.0]
    }

    pub fn display_dimensions(&self) -> (usize, usize) {
        (self.display.width, self.display.height)
    }

    pub fn get_pitch(&self) -> u8 {
        self.pitch
    }

    pub fn get_audio_pattern(&self) -> &[u8] {
        &self.audio_pattern[..]
    }
}
