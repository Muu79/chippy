mod execute;
pub mod registers;
pub mod state;
pub mod inspect;
pub mod mutate;

use crate::config::quirks::Quirks;
use crate::config::target::Target;
use crate::config::target::Target::Chip8;
use crate::display::font::{Sprite, CHAR_MAP};
use crate::display::{Direction, Display, TargetPlane};
use crate::emu::encode_decode::Opcode::*;
use crate::emu::encode_decode::{decode_instruction, Opcode};
use crate::hardware::Keyboard;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use crate::rng::Rng;

pub(super) static STACK_SIZE: usize = 16;
pub(super) static REG_COUNT: usize = 16;
pub(super) static RPL_REG_COUNT: usize = 16;

/// The CPU makes up the heart of the emulator. It is responsible for the flow of instructions,
/// as well as owning and managing the keyboard and display buffer
pub struct Cpu {
    ram: Box<[u8]>,
    v_reg: [u8; REG_COUNT],
    i_reg: u16,
    stack: Vec<u16>,
    stack_ptr: u16,
    pc: u16,
    sound_timer: u8,
    delay_timer: u8,
    keys: Keyboard,
    display: Display,
    waiting_for_key: Option<VRegister>,
    target: Target,
    target_quirks: Quirks,
    rng: Rng,
    // Super-Chip props
    rpl_regs: [u8; RPL_REG_COUNT],
    // XO-Chip props
    audio_pattern: [u8; 16],
    pitch: u8,
}
#[derive(Copy, Clone, PartialEq)]
pub(crate) struct VRegister(pub(crate) usize);
impl From<VRegister> for u8 {
    fn from(value: VRegister) -> Self {
        0xF & value.0 as u8
    }
}
impl From<VRegister> for u16 {
    fn from(value: VRegister) -> Self {
        0xF & value.0 as u16
    }
}

pub enum CpuError {
    InvalidOpcode(u8),
    InvalidRegister(u8),
    InvalidAddress(u16),
}
#[repr(u8)]
pub enum CpuCode {
    Ok = 0,
    KeyWait = 1,
    DispWait = 2,
    Skipped = 3,
    Exit(&'static str) = 4,
}
impl Default for Cpu {
    fn default() -> Self {
        Self::new(Chip8, None)
    }
}
impl Cpu {
    pub fn new(target: Target, rng_seed: Option<u64>) -> Self {
        let mut ram = vec![0; target.ram_size()].into_boxed_slice();
        ram[..CHAR_MAP.len()].copy_from_slice(&CHAR_MAP[..]); // We always copy the full (small and large) char sprites, may be worth changing
        Self {
            ram,
            v_reg: [0; REG_COUNT],
            i_reg: 0,
            stack: vec![0; STACK_SIZE],
            stack_ptr: 0,
            pc: target.start_address(),
            sound_timer: 0,
            delay_timer: 0,
            keys: Keyboard::new(),
            display: Display::new(),
            waiting_for_key: None,
            target,
            target_quirks: target.default_quirks(),
            rpl_regs: [0; RPL_REG_COUNT],
            rng: Rng::new(rng_seed.unwrap_or(123 << 5)),
            audio_pattern: [0; 16],
            pitch: 64,
        }
    }

    
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), &'static str> {
        if rom.len() + self.target.start_address() as usize > self.target.ram_size() {
            return Err("ROM too large");
        }
        self.ram[0x200..0x200 + rom.len()].copy_from_slice(rom);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.ram[self.target.start_address() as usize..].fill(0);
        self.v_reg.fill(0);
        self.i_reg = 0;
        self.stack_ptr = 0;
        self.stack.clear();
        self.pc = self.target.start_address();
        self.sound_timer = 0;
        self.delay_timer = 0;
        self.keys.reset();
        self.display.clear();
        self.audio_pattern = [0; 16];
        self.pitch = 64;
    }

    pub fn load_state(&mut self, mut new_state: Cpu, new_target: Target) {
        if new_target != self.target {
            self.target = new_target;
            core::mem::swap(self, &mut new_state);
        }
    }

    pub fn eject_state(&mut self) -> Cpu {
        let mut holder = Cpu::new(self.target, None);
        core::mem::swap(self, &mut holder);
        holder
    }

    pub fn swap_state(&mut self, other: &mut Cpu) {
        core::mem::swap(self, other);
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn is_making_sound(&self) -> bool {
        self.sound_timer > 0
    }

    pub fn is_waiting_for_key(&self) -> bool {
        self.waiting_for_key.is_some()
    }

    pub fn is_extended(&self) -> bool {
        self.display.is_extended()
    }

    pub fn tick_cpu(&mut self) -> Result<CpuCode, &'static str> {
        if let Some(reg) = self.waiting_for_key {
            if let Some(first_input) = self.keys.as_input_key() {
                if !self.keys.is_pressed(first_input)? {
                    *self.get_reg_mut(reg) = first_input;
                    self.waiting_for_key = None;
                }
            }
            return Ok(CpuCode::Skipped);
        }
        let opcode = self.fetch()?;
        let operation = if opcode == 0xF000 {
            let idx = self.pc as usize;
            let chomp = (self.ram[idx] as u16) << 8 | self.ram[idx + 1] as u16;
            self.pc += 2;
            LdILong(chomp)
        } else {
            decode_instruction(opcode)
        };
        self.execute(operation)
    }

    fn fetch(&mut self) -> Result<u16, &'static str> {
        let addr = self.pc;
        let opcode = (self.ram[addr as usize] as u16) << 8 | self.ram[addr as usize + 1] as u16;
        self.pc += 2;
        Ok(opcode)
    }

    fn skip_instruction(&mut self) {
        let next_opcode =
            (self.ram[self.pc as usize] as u16) << 8 | self.ram[self.pc as usize + 1] as u16;
        // In XO-chip we need to account for 32bit wide opcode F000 aaaa
        if next_opcode == 0xF000 && matches!(self.target, Target::XOChip) {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn get_top_rpl_reg(&self, v_x: VRegister) -> usize {
        if matches!(self.target, Target::SChip8Legacy | Target::SChip8Modern) {
            v_x.0 % 8
        } else if matches!(self.target, Target::XOChip) {
            v_x.0 % 16
        } else {
            0
        }
    }
    fn vf_flag(&mut self, pred: bool) {
        self.v_reg[0xF] = if pred { 1 } else { 0 };
    }

    fn set_vf(&mut self, num: u8) {
        self.v_reg[0xF] = num;
    }
}
