use crate::Rng;
use crate::cpu::encode_decode::Opcode::LdILong;
use crate::cpu::encode_decode::{Opcode, VRegister, decode_instruction};
use crate::hardware::{CHAR_MAP, Direction, Display, Keyboard, Sprite};
use Target::*;
use TargetQuirk::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// Target for CPU to emulate
#[derive(PartialEq, Copy, Clone)]
pub enum Target {
    Chip8,
    SChip8Modern,
    SChip8Legacy,
    XOChip,
}

impl Target {
    pub const fn start_address(&self) -> u16 {
        match self {
            Chip8 | SChip8Legacy | SChip8Modern | XOChip => 0x200,
        }
    }

    pub const fn ram_size(&self) -> usize {
        match self {
            Chip8 | SChip8Legacy | SChip8Modern => 1 << 12,
            XOChip => 1 << 16,
        }
    }

    pub const fn is_extendable(&self) -> bool {
        match self {
            Chip8 => false,
            SChip8Legacy | XOChip | SChip8Modern => true,
        }
    }

    pub(super) fn default_quirks(&self) -> Quirks {
        match self {
            SChip8Modern | XOChip => {
                Quirks::default()
                    | ShiftUsesVx
                    | JumpUsesVx
                    | HasScrollOps
                    | ClScrOnResChange
                    | LoResWideSpriteOnDrwXY0
            }
            SChip8Legacy => {
                Quirks::default()
                    | ShiftUsesVx
                    | JumpUsesVx
                    | HasScrollOps
                    | ClScrOnResChange
                    | LargeSpriteOnFx29
                    | DispWait
                    | ScrHalfOnLoRes
                    | DrawSpriteOnDrwXY0
            }
            Chip8 => Quirks::default() | IncrIOnLd | VfExtraReset | DispWait,
        }
    }
}

#[repr(u16)]
#[derive(Default, Clone, Copy)]
pub enum TargetQuirk {
    // Making None default means any quirk would cause it to return false for .has_quirk()
    #[default]
    NoQuirk = 0,
    ShiftUsesVx = 1 << 0,
    IncrIOnLd = 1 << 1,
    VfExtraReset = 1 << 2,
    DispWait = 1 << 3,
    JumpUsesVx = 1 << 4,
    HasScrollOps = 1 << 5,
    ClScrOnResChange = 1 << 6,
    LargeSpriteOnFx29 = 1 << 7,
    DrwCountsCollisionLines = 1 << 8,
    ScrHalfOnLoRes = 1 << 9,
    DrawSpriteOnDrwXY0 = 1 << 10,
    LoResWideSpriteOnDrwXY0 = 1 << 11,
}

#[derive(Default)]
pub(crate) struct Quirks {
    pub(crate) quirk_map: u16,
}

pub(super) static RAM_SIZE: usize = 4096;
pub(super) static STACK_SIZE: usize = 16;
pub(super) static REG_COUNT: usize = 16;
pub(super) static RPL_REG_COUNT: usize = 16;

pub struct Cpu {
    pub(super) ram: Box<[u8]>,
    pub(super) v_reg: [u8; REG_COUNT],
    pub(super) i_reg: u16,
    pub(super) stack: Vec<u16>,
    pub(super) stack_ptr: u16,
    pub(super) pc: u16,
    pub(super) sound_timer: u8,
    pub(super) delay_timer: u8,
    pub(super) keys: Keyboard,
    pub(super) display: Display,
    pub(super) waiting_for_key: Option<VRegister>,
    pub(super) target: Target,
    pub(super) target_quirks: Quirks,
    pub(super) rng: Rng,
    // Super-Chip props
    pub(super) rpl_regs: [u8; RPL_REG_COUNT],
    // XO-Chip props
    pub(super) audio_pattern: [u8; 16],
    pub(super) pitch: u16,
}

#[repr(u8)]
pub enum CpuCode {
    Ok = 0,
    Wait = 1,
    Skipped = 2,
    Exit(&'static str) = 3,
}
impl Default for Cpu {
    fn default() -> Self {
        Self::new(Chip8)
    }
}
impl Cpu {
    pub fn new(target: Target) -> Self {
        let mut ram = vec![0; target.ram_size()].into_boxed_slice();
        ram[..CHAR_MAP.len()].copy_from_slice(CHAR_MAP.as_slice()); // We always copy the full (small and large) char sprites, may be worth changing
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
            rng: Rng::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
            audio_pattern: [0; 16],
            pitch: 4000,
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), &'static str> {
        if rom.len() + 0x200 > RAM_SIZE {
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
        self.pitch = 4000;
    }

    pub fn load_state(&mut self, mut new_state: Cpu, new_target: Target) {
        if new_target != self.target {
            self.target = new_target;
            std::mem::swap(self, &mut new_state);
        }
    }

    pub fn eject_state(&mut self) -> Cpu {
        let mut holder = Cpu::new(self.target);
        std::mem::swap(self, &mut holder);
        holder
    }

    pub fn swap_state(&mut self, other: &mut Cpu) {
        std::mem::swap(self, other);
    }

    fn push(&mut self, val: u16) {
        if self.stack_ptr == self.stack.len() as u16 {
            self.stack.push(0);
        }
        self.stack[self.stack_ptr as usize] = val;
        self.stack_ptr += 1;
    }

    fn pop(&mut self) -> Result<u16, &'static str> {
        if self.stack_ptr == 0 {
            return Err("Attempted to pop from empty stack");
        }
        self.stack_ptr -= 1;
        Ok(self.stack[self.stack_ptr as usize])
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
            if let Some(first_input) = self.keys.as_input_key()
                && !self.keys.is_pressed(first_input)?
            {
                *self.get_reg_mut(reg) = first_input;
                self.waiting_for_key = None;
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
        if next_opcode == 0xF000 && matches!(self.target, XOChip) {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }
    fn execute(&mut self, operation: Opcode) -> Result<CpuCode, &'static str> {
        use Opcode::*;
        use TargetQuirk::*;
        match operation {
            NoOp => (),
            ClS => self.display.clear(),
            Ret => {
                self.pc = self.pop()?;
            }
            Jp(nnn) => self.pc = nnn,
            Call(nnn) => {
                self.push(self.pc);
                self.pc = nnn;
            }
            SEByte(v_x, kk) => {
                if self.get_reg(v_x) == &kk {
                    self.skip_instruction()
                }
            }
            SNEByte(v_x, kk) => {
                if self.get_reg(v_x) != &kk {
                    self.skip_instruction()
                }
            }
            SEReg(v_x, v_y) => {
                if self.get_reg(v_x) == self.get_reg(v_y) {
                    self.skip_instruction()
                }
            }
            SNEReg(v_x, v_y) => {
                if self.get_reg(v_x) != self.get_reg(v_y) {
                    self.skip_instruction()
                }
            }
            LdByte(v_x, kk) => {
                *self.get_reg_mut(v_x) = kk;
            }
            AddVxByte(v_x, kk) => {
                *self.get_reg_mut(v_x) = self.get_reg(v_x).wrapping_add(kk);
            }
            LdReg(v_x, v_y) => {
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) = y;
            }
            Or(v_x, v_y) => {
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) |= y;
                if self.has_quirk(VfExtraReset) {
                    self.vf_flag(false);
                }
            }
            And(v_x, v_y) => {
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) &= y;
                if self.has_quirk(VfExtraReset) {
                    self.vf_flag(false);
                }
            }
            Xor(v_x, v_y) => {
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) ^= y;
                if self.has_quirk(VfExtraReset) {
                    self.vf_flag(false);
                }
            }
            AddVxVy(v_x, v_y) => {
                let x = *self.get_reg(v_x);
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) = x.wrapping_add(y);
                self.vf_flag((x as u16 + y as u16) > 0xff);
            }
            Sub(v_x, v_y) => {
                let x = *self.get_reg(v_x);
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) = x.wrapping_sub(y);
                self.vf_flag(x >= y);
            }
            ShR(v_x, v_y) => {
                let source = if self.has_quirk(ShiftUsesVx) {
                    *self.get_reg(v_x)
                } else {
                    *self.get_reg(v_y)
                };
                *self.get_reg_mut(v_x) = source >> 1;
                self.vf_flag(source & 0x1 == 0x1);
            }
            SubN(v_x, v_y) => {
                let x = *self.get_reg(v_x);
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) = y.wrapping_sub(x);
                self.vf_flag(y >= x);
            }
            ShL(v_x, v_y) => {
                let source = if self.has_quirk(ShiftUsesVx) {
                    *self.get_reg(v_x)
                } else {
                    *self.get_reg(v_y)
                };
                *self.get_reg_mut(v_x) = source << 1;
                self.vf_flag(source & 0x80 == 0x80);
            }
            LdToI(nnn) => {
                self.i_reg = nnn;
            }
            JpReg(nnn) => {
                let target = if self.has_quirk(JumpUsesVx) {
                    let v_x = VRegister((nnn >> 8) as usize);
                    nnn.wrapping_add(*self.get_reg(v_x) as u16)
                } else {
                    nnn.wrapping_add(*self.get_reg(VRegister(0)) as u16)
                };
                self.pc = target;
            }
            Rnd(v_x, kk) => {
                *self.get_reg_mut(v_x) = self.get_rand_byte() & kk;
            }
            Drw(v_x, v_y, n) => {
                let (width, height) = self.display_dimensions();
                let (col, row) = (
                    *self.get_reg(v_x) as usize % width,
                    *self.get_reg(v_y) as usize % height,
                );

                let draws_16x16 = n == 0 && self.has_quirk(DrawSpriteOnDrwXY0);
                let sprite_h: usize = if draws_16x16 { 16 } else { n as usize };
                let row_bytes: usize = if draws_16x16 { 2 } else { 1 };

                let mut addr = self.i_reg as usize;
                let mut vf: u8 = 0;

                for plane in self.display.get_plane_idx() {
                    let chomps: Vec<u16> = (0..sprite_h)
                        .map(|r| {
                            let base = addr + r * row_bytes;
                            if row_bytes == 2 {
                                ((self.ram[base] as u16) << 8) | self.ram[base + 1] as u16
                            } else {
                                self.ram[base] as u16
                            }
                        })
                        .collect();

                    vf += self.display.draw_sprite(row, col, &chomps, plane);
                    addr += sprite_h * row_bytes; // only advances for planes actually consumed
                }

                self.set_vf(if self.has_quirk(DrwCountsCollisionLines) {
                    vf
                } else {
                    (vf > 0) as u8
                });

                if self.has_quirk(DispWait) && !self.is_extended() {
                    return Ok(CpuCode::Wait);
                }
            }
            SkP(v_x) => {
                if self.keys.is_pressed(*self.get_reg(v_x))? {
                    self.skip_instruction()
                }
            }
            SkNP(v_x) => {
                if !self.keys.is_pressed(*self.get_reg(v_x))? {
                    self.skip_instruction()
                }
            }
            LdDTVx(v_x) => *self.get_reg_mut(v_x) = self.delay_timer,
            LdKey(v_x) => {
                self.waiting_for_key = Some(v_x);
                return Ok(CpuCode::Wait);
            }
            LdVxDT(v_x) => self.delay_timer = *self.get_reg(v_x),
            LdVxST(v_x) => self.sound_timer = *self.get_reg(v_x), // IIRC wrapping add on I is not possible
            AddIVx(v_x) => {
                self.i_reg += *self.get_reg(v_x) as u16;
            }
            LdSpr(v_x) => {
                let idx = *self.get_reg(v_x);
                // SChip 1.0 Quirk
                self.i_reg = if idx > 0xF && self.has_quirk(LargeSpriteOnFx29) {
                    Sprite::from_hex(idx % 0x10, true)? as u16
                } else {
                    Sprite::from_hex(idx, false)? as u16
                }
            }
            LdDeci(v_x) => {
                let val = *self.get_reg(v_x) as u16;
                for pow in (0..3).rev() {
                    self.ram[(self.i_reg + (2 - pow)) as usize] =
                        ((val / 10u16.pow(pow as u32)) % 10) as u8;
                }
            }
            LdVxI(v_x) => {
                for i in 0..=v_x.0 {
                    self.ram[self.i_reg as usize + i] = *self.get_reg(VRegister(i));
                }
                if self.has_quirk(IncrIOnLd) {
                    self.i_reg += v_x.0 as u16 + 1;
                }
            }
            LdIVx(v_x) => {
                for i in 0..=v_x.0 {
                    *self.get_reg_mut(VRegister(i)) = self.ram[self.i_reg as usize + i];
                }
                if self.has_quirk(IncrIOnLd) {
                    self.i_reg += v_x.0 as u16 + 1;
                }
            }
            // SCHIP Opcodes
            ScD(n) => {
                let scr_by = (if self.has_quirk(ScrHalfOnLoRes) && !self.is_extended() {
                    n / 2
                } else {
                    n
                }) as usize;
                self.display
                    .scroll_selected_planes_by(scr_by, Direction::Down)
            }
            ScR => {
                let scr_by = if self.has_quirk(ScrHalfOnLoRes) && !self.is_extended() {
                    2
                } else {
                    4
                };
                self.display
                    .scroll_selected_planes_by(scr_by, Direction::Right)
            }
            ScL => {
                let scr_by = if self.has_quirk(ScrHalfOnLoRes) && !self.is_extended() {
                    2
                } else {
                    4
                };
                self.display
                    .scroll_selected_planes_by(scr_by, Direction::Left)
            }
            Exit => {}
            LoRes => self.display.enter_lo_res(),
            HiRes => self.display.enter_hi_res(),
            SaveFlags(v_x) => {
                let top_reg = self.get_top_rpl_reg(v_x);
                for x in 0..=top_reg {
                    self.rpl_regs[x] = *self.get_reg(VRegister(x));
                }
            }
            LdFlags(v_x) => {
                let top_reg = self.get_top_rpl_reg(v_x);
                for x in 0..=top_reg {
                    *self.get_reg_mut(v_x) = self.rpl_regs[x];
                }
            }
            // Octo Opcodes
            ScU(n) => {
                let scr_by = (if self.has_quirk(ScrHalfOnLoRes) && !self.is_extended() {
                    n / 2
                } else {
                    n
                }) as usize;
                self.display
                    .scroll_selected_planes_by(scr_by, Direction::Up)
            }
            LdILong(chomp) => self.i_reg = chomp,
            LdIVxToVy(v_x, v_y) => {
                let start = self.i_reg as usize;
                for (idx, reg) in (v_x.0..=v_y.0).enumerate() {
                    self.ram[start + idx] = *self.get_reg(VRegister(reg));
                }
            }
            LdVxToVyI(v_x, v_y) => {
                let start = self.i_reg as usize;
                for (idx, reg) in (v_x.0..=v_y.0).enumerate() {
                    *self.get_reg_mut(VRegister(reg)) = self.ram[start + idx];
                }
            }
        };
        Ok(CpuCode::Ok)
    }

    fn get_top_rpl_reg(&self, v_x: VRegister) -> usize {
        if matches!(self.target, SChip8Legacy | SChip8Modern) {
            v_x.0 % 8
        } else if matches!(self.target, XOChip) {
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

pub fn parse_hex(hex: u32) -> [Sprite; 8] {
    let mut ans = [Sprite::default(); 8];
    for (i, sprite) in ans.iter_mut().enumerate() {
        *sprite = Sprite::from_hex(((hex >> ((7 - i) * 4)) & 0xF) as u8, false).unwrap_or_default();
    }
    ans
}
