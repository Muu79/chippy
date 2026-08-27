use crate::hardware::{CHAR_MAP, Display, Keyboard, Sprite};
use log::warn;
use rand::random;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

/// Target for CPU to emulate
#[derive(PartialEq, Copy, Clone)]
pub enum Target {
    Chip8,
    SChip8Modern,
    SChip8Classic,
}
use Target::*;

impl Target {
    pub const fn start_address(&self) -> u16 {
        match self {
            Chip8 | SChip8Classic | SChip8Modern => 0x200,
        }
    }

    pub(crate) fn default_quirks(&self) -> Quirks {
        use TargetQuirk::*;
        match self {
            SChip8Modern => {
                Quirks::default() | ShiftUsesVx | JumpUsesVx | HasScrollOps | ClScrOnResChange
            }
            SChip8Classic => {
                Quirks::default() | ShiftUsesVx | JumpUsesVx | HasScrollOps | ClScrOnResChange
            }
            Chip8 => Quirks::default() | IncrIOnLd | VfExtraReset | DispWait,
        }
    }
}

#[repr(u16)]
#[derive(Default, Clone, Copy)]
pub enum TargetQuirk {
    #[default]
    ShiftUsesVx = 1 << 0,
    IncrIOnLd = 1 << 1,
    VfExtraReset = 1 << 2,
    DispWait = 1 << 3,
    JumpUsesVx = 1 << 4,
    HasScrollOps = 1 << 5,
    ClScrOnResChange = 1 << 6,
}

#[derive(Default)]
pub(crate) struct Quirks {
    quirk_map: u16,
}
impl From<u16> for Quirks {
    fn from(quirk_map: u16) -> Self {
        Self { quirk_map }
    }
}
impl BitOr for Quirks {
    type Output = Self;
    fn bitor(self, other: Self) -> Self::Output {
        Self {
            quirk_map: self.quirk_map | other.quirk_map,
        }
    }
}
impl BitOr<TargetQuirk> for Quirks {
    type Output = Self;
    fn bitor(self, other: TargetQuirk) -> Self::Output {
        Self {
            quirk_map: self.quirk_map | other as u16,
        }
    }
}
impl BitOrAssign<TargetQuirk> for Quirks {
    fn bitor_assign(&mut self, other: TargetQuirk) {
        self.quirk_map |= other as u16;
    }
}
impl BitAnd for Quirks {
    type Output = Self;
    fn bitand(self, other: Self) -> Self::Output {
        Self {
            quirk_map: self.quirk_map & other.quirk_map,
        }
    }
}
impl BitAnd<TargetQuirk> for Quirks {
    type Output = Self;
    fn bitand(self, other: TargetQuirk) -> Self::Output {
        Self {
            quirk_map: self.quirk_map & other as u16,
        }
    }
}
impl BitAndAssign<TargetQuirk> for Quirks {
    fn bitand_assign(&mut self, other: TargetQuirk) {
        self.quirk_map &= other as u16
    }
}

const RAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const REG_COUNT: usize = 16;

pub struct Cpu {
    pub(crate) ram: [u8; RAM_SIZE],
    pub(crate) v_reg: [u8; REG_COUNT],
    pub(crate) i_reg: u16,
    pub(crate) stack: Vec<u16>,
    pub(crate) stack_ptr: u16,
    pub(crate) pc: u16,
    pub(crate) sound_timer: u8,
    pub(crate) debug_time: u8,
    pub(crate) keys: Keyboard,
    pub(crate) display: Display,
    pub(crate) waiting_for_key: Option<VRegister>,
    pub(crate) target: Target,
    pub(crate) target_quirks: Quirks,
}

pub enum CpuCode {
    Ok = 0,
    StartWaitForKey = 1,
    Skipped = 2,
}
impl Default for Cpu {
    fn default() -> Self {
        Self::new(Chip8)
    }
}
impl Cpu {
    pub fn new(target: Target) -> Self {
        let mut ram = [0; RAM_SIZE];
        ram[..CHAR_MAP.len()].copy_from_slice(CHAR_MAP.as_slice());
        Self {
            ram,
            v_reg: [0; REG_COUNT],
            i_reg: 0,
            stack: vec![0; STACK_SIZE],
            stack_ptr: 0,
            pc: target.start_address(),
            sound_timer: 0,
            debug_time: 0,
            keys: Keyboard::new(),
            display: Display::new(&target),
            waiting_for_key: None,
            target,
            target_quirks: target.default_quirks(),
        }
    }

    pub fn has_quirk(&self, quirk: TargetQuirk) -> bool {
        self.target_quirks.quirk_map & quirk as u16 != 0
    }

    pub fn get_quirk_map(&self) -> u16 {
        self.target_quirks.quirk_map
    }

    pub fn set_quirk_map(&mut self, quirk_map: u16) {
        self.target_quirks.quirk_map = quirk_map
    }

    pub fn set_quirk(&mut self, quirk: TargetQuirk) {
        self.target_quirks.quirk_map |= quirk as u16
    }

    pub fn clear_quirk(&mut self, quirk: TargetQuirk) {
        self.target_quirks.quirk_map &= !(quirk as u16)
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }
    pub fn i_reg(&self) -> u16 {
        self.i_reg
    }
    pub fn v_regs(&self) -> &[u8; REG_COUNT] {
        &self.v_reg
    }
    pub fn stack(&self) -> &[u16] {
        &self.stack[..self.stack_ptr as usize]
    }
    pub fn delay_timer(&self) -> u8 {
        self.debug_time
    }
    pub fn sound_timer(&self) -> u8 {
        self.sound_timer
    }
    pub fn target(&self) -> Target {
        self.target
    }

    pub fn get_target(&self) -> Target {
        self.target
    }

    pub fn get_display(&self) -> &Display {
        &self.display
    }

    pub fn get_mut_keys(&mut self) -> &mut Keyboard {
        &mut self.keys
    }
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), &'static str> {
        if rom.len() + 0x200 > RAM_SIZE {
            return Err("ROM too large");
        }
        self.ram[0x200..0x200 + rom.len()].copy_from_slice(rom);
        Ok(())
    }

    fn push(&mut self, val: u16) {
        if self.stack_ptr == self.stack.len() as u16 {
            warn!("Stack overflowed\nIncreasing stack size (this shouldn't happen)");
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

    pub fn reset(&mut self) {
        self.ram[self.target.start_address() as usize..].fill(0);
        self.v_reg.fill(0);
        self.i_reg = 0;
        self.stack_ptr = 0;
        self.pc = self.target.start_address();
        self.sound_timer = 0;
        self.debug_time = 0;
        self.keys.reset();
        self.display.clear();
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

    fn fetch(&mut self) -> Result<u16, &'static str> {
        let addr = self.pc;
        let opcode = (self.ram[addr as usize] as u16) << 8 | self.ram[addr as usize + 1] as u16;
        self.pc += 2;
        Ok(opcode)
    }

    pub fn tick_timers(&mut self) {
        if self.debug_time > 0 {
            self.debug_time -= 1;
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
        let operation = self.decode(opcode);
        self.execute(operation)
    }
    fn get_reg(&self, reg: VRegister) -> &u8 {
        &self.v_reg[reg.0]
    }

    fn get_reg_mut(&mut self, reg: VRegister) -> &mut u8 {
        &mut self.v_reg[reg.0]
    }

    fn display_dimensions(&self) -> (usize, usize) {
        (self.display.width, self.display.height)
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
                    self.pc += 2;
                }
            }
            SNEByte(v_x, kk) => {
                if self.get_reg(v_x) != &kk {
                    self.pc += 2;
                }
            }
            SEReg(v_x, v_y) => {
                if self.get_reg(v_x) == self.get_reg(v_y) {
                    self.pc += 2;
                }
            }
            SNEReg(v_x, v_y) => {
                if self.get_reg(v_x) != self.get_reg(v_y) {
                    self.pc += 2;
                }
            }
            LdByte(v_x, kk) => {
                *self.get_reg_mut(v_x) = kk;
            }
            AddByte(v_x, kk) => {
                *self.get_reg_mut(v_x) += kk;
            }
            LdReg(v_x, v_y) => {
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) = y;
            }
            Or(v_x, v_y) => {
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) |= y;
                if self.has_quirk(VfExtraReset) {
                    self.set_vf(false);
                }
            }
            And(v_x, v_y) => {
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) &= y;
                if self.has_quirk(VfExtraReset) {
                    self.set_vf(false);
                }
            }
            Xor(v_x, v_y) => {
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) ^= y;
                if self.has_quirk(VfExtraReset) {
                    self.set_vf(false);
                }
            }
            AddReg(v_x, v_y) => {
                let sum = *self.get_reg(v_x) as u16 + *self.get_reg(v_y) as u16;
                *self.get_reg_mut(v_x) = sum as u8;
                self.set_vf(sum > 0xff);
            }
            Sub(v_x, v_y) => {
                let x = *self.get_reg(v_x);
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) = x - y;
                self.set_vf(x >= y);
            }
            ShR(v_x, v_y) => {
                let source = if self.has_quirk(ShiftUsesVx) {
                    *self.get_reg(v_x)
                } else {
                    *self.get_reg(v_y)
                };
                *self.get_reg_mut(v_x) = source >> 1;
                self.set_vf(source & 0x1 == 0x1);
            }
            SubN(v_x, v_y) => {
                let x = *self.get_reg(v_x);
                let y = *self.get_reg(v_y);
                *self.get_reg_mut(v_x) = y - x;
                self.set_vf(y >= x);
            }
            ShL(v_x, v_y) => {
                let source = if self.has_quirk(ShiftUsesVx) {
                    *self.get_reg(v_x)
                } else {
                    *self.get_reg(v_y)
                };
                *self.get_reg_mut(v_x) = source << 1;
                self.set_vf(source & 0x80 == 0x80);
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
            Rand(v_x, kk) => {
                *self.get_reg_mut(v_x) = random::<u8>() & kk;
            }
            Drw(v_x, v_y, n) => {
                let n = if n == 0 && self.is_extended() { 16 } else { n };
                let (width, height) = self.display_dimensions();
                let start = self.i_reg as usize;
                let (col, row) = (
                    *self.get_reg(v_x) as usize % width,
                    *self.get_reg(v_y) as usize % height,
                );
                let mut vf = false;
                for line in 0..n as usize {
                    if (start + line) >= self.ram.len() || row + line >= height {
                        break;
                    }
                    let byte = self.ram[start + line];
                    if let Ok(collision) = self.draw_byte(row + line, col, byte) {
                        vf |= collision;
                    } else {
                        break;
                    }
                }
                self.set_vf(vf);
            }
            SkP(v_x) => {
                if self.keys.is_pressed(*self.get_reg(v_x))? {
                    self.pc += 2;
                }
            }
            SkNP(v_x) => {
                if !self.keys.is_pressed(*self.get_reg(v_x))? {
                    self.pc += 2;
                }
            }
            LdDTToVx(v_x) => *self.get_reg_mut(v_x) = self.debug_time,
            LDkey(v_x) => {
                self.waiting_for_key = Some(v_x);
                return Ok(CpuCode::StartWaitForKey);
            }
            LdVxToDT(v_x) => self.debug_time = *self.get_reg(v_x),
            LdVxToST(v_x) => self.sound_timer = *self.get_reg(v_x),
            AddToI(v_x) => self.i_reg += *self.get_reg(v_x) as u16,
            LdSpr(v_x) => self.i_reg = Sprite::from_hex(*self.get_reg(v_x))? as u16,
            LdDeci(v_x) => {
                let val = *self.get_reg(v_x) as u16;
                for pow in (0..3).rev() {
                    self.ram[(self.i_reg + (2 - pow)) as usize] =
                        ((val / 10u16.pow(pow as u32)) % 10) as u8;
                }
            }
            LdVxToI(v_x) => {
                for i in 0..=v_x.0 {
                    self.ram[self.i_reg as usize + i] = *self.get_reg(VRegister(i));
                }
                if self.has_quirk(IncrIOnLd) {
                    self.i_reg += v_x.0 as u16 + 1;
                }
            }
            LdIToVx(v_x) => {
                for i in 0..=v_x.0 {
                    *self.get_reg_mut(VRegister(i)) = self.ram[self.i_reg as usize + i];
                }
                if self.has_quirk(IncrIOnLd) {
                    self.i_reg += v_x.0 as u16 + 1;
                }
            }
            // SCHIP Opcodes
            ScD(n) => {
                let n = n as usize;
                let height = self.display.get_height();
                let buff = self.display.get_screen_mut();
                for curr_row in (0..height.saturating_sub(n)).rev() {
                    buff[curr_row + n] = buff[curr_row];
                }
                buff.iter_mut().take(n).for_each(|row| *row = 0);
            }
            ScR => self
                .display
                .get_screen_mut()
                .iter_mut()
                .for_each(|row| *row <<= 4),
            ScL => self
                .display
                .get_screen_mut()
                .iter_mut()
                .for_each(|row| *row >>= 4),
            LoRes => self.display.enter_lo_res(),
            HiRes => self.display.enter_hi_res(),
            _ => unimplemented!(),
        };
        Ok(CpuCode::Ok)
    }

    fn set_vf(&mut self, pred: bool) {
        self.v_reg[0xF] = if pred { 1 } else { 0 };
    }
    fn decode(&self, opcode: u16) -> Opcode {
        use Opcode::*;
        let nibbles = nibble_op_code(opcode);
        let n = nibbles.3;
        let kk = opcode as u8;
        let nnn = opcode & 0xfff;
        let (x_reg, y_reg) = (VRegister(nibbles.1 as usize), VRegister(nibbles.2 as usize));
        match nibbles {
            (0, 0, 0, 0) => NoOp,
            (0, 0, 0xC, _) => ScD(n),
            (0, 0, 0xE, 0) => ClS,
            (0, 0, 0xE, 0xE) => Ret,
            (0, 0, 0xF, 0xB) => ScR,
            (0, 0, 0xF, 0xC) => ScL,
            (0, 0, 0xF, 0xD) => Exit,
            (0, 0, 0xF, 0xE) => LoRes,
            (0, 0, 0xF, 0xF) => HiRes,
            (0x1, _, _, _) => Jp(nnn),
            (0x2, _, _, _) => Call(nnn),
            (0x3, _, _, _) => SEByte(x_reg, kk),
            (0x4, _, _, _) => SNEByte(x_reg, kk),
            (0x5, _, _, _) => SEReg(x_reg, y_reg),
            (0x6, _, _, _) => LdByte(x_reg, kk),
            (0x7, _, _, _) => AddByte(x_reg, kk),
            (0x8, _, _, 0x0) => LdReg(x_reg, y_reg),
            (0x8, _, _, 0x1) => Or(x_reg, y_reg),
            (0x8, _, _, 0x2) => And(x_reg, y_reg),
            (0x8, _, _, 0x3) => Xor(x_reg, y_reg),
            (0x8, _, _, 0x4) => AddReg(x_reg, y_reg),
            (0x8, _, _, 0x5) => Sub(x_reg, y_reg),
            (0x8, _, _, 0x6) => ShR(x_reg, y_reg),
            (0x8, _, _, 0x7) => SubN(x_reg, y_reg),
            (0x8, _, _, 0xE) => ShL(x_reg, y_reg),
            (0x9, _, _, 0) => SNEReg(x_reg, y_reg),
            (0xA, _, _, _) => LdToI(nnn),
            (0xB, _, _, _) => JpReg(nnn),
            (0xC, _, _, _) => Rand(x_reg, kk),
            (0xD, _, _, _) => Drw(x_reg, y_reg, n),
            (0xE, _, 0x9, 0xE) => SkP(x_reg),
            (0xE, _, 0xA, 0x1) => SkNP(x_reg),
            (0xF, _, 0x0, 0x7) => LdDTToVx(x_reg),
            (0xF, _, 0x0, 0xA) => LDkey(x_reg),
            (0xF, _, 0x1, 0x5) => LdVxToDT(x_reg),
            (0xF, _, 0x1, 0x8) => LdVxToST(x_reg),
            (0xF, _, 0x1, 0xE) => AddToI(x_reg),
            (0xF, _, 0x2, 0x9) => LdSpr(x_reg),
            (0xF, _, 0x3, 0x3) => LdDeci(x_reg),
            (0xF, _, 0x5, 0x5) => LdVxToI(x_reg),
            (0xF, _, 0x6, 0x5) => LdIToVx(x_reg),
            (_, _, _, _) => NoOp,
        }
    }

    pub fn write_hex(&mut self, line: usize, hex: u32) -> Result<(), &'static str> {
        for (i, sprite) in parse_hex(hex).iter().enumerate() {
            self.display
                .draw_sprite(line * 5, i * 8, sprite, &self.ram)?;
        }
        Ok(())
    }

    pub fn draw_sprite(
        &mut self,
        x: usize,
        y: usize,
        sprite: &Sprite,
    ) -> Result<bool, &'static str> {
        self.display.draw_sprite(x, y, sprite, &self.ram)
    }

    pub fn draw_byte(&mut self, x: usize, y: usize, byte: u8) -> Result<bool, &'static str> {
        self.display.draw_byte(x, y, byte)
    }

    pub fn set_keyboard(&mut self, keys: Keyboard) {
        self.keys = keys;
    }
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) struct VRegister(usize);
enum Opcode {
    NoOp,
    ScD(u8),
    ClS,
    Ret,
    ScR,
    ScL,
    Exit,
    LoRes,
    HiRes,
    Jp(u16),
    JpReg(u16),
    Call(u16),
    SEByte(VRegister, u8),
    SEReg(VRegister, VRegister),
    SNEByte(VRegister, u8),
    SNEReg(VRegister, VRegister),
    LdByte(VRegister, u8),
    LdReg(VRegister, VRegister),
    LdToI(u16),
    LDkey(VRegister),
    LdSpr(VRegister),
    LdDeci(VRegister),
    LdVxToI(VRegister),
    LdIToVx(VRegister),
    LdVxToDT(VRegister),
    LdDTToVx(VRegister),
    LdVxToST(VRegister),
    AddByte(VRegister, u8),
    AddReg(VRegister, VRegister),
    AddToI(VRegister),
    Or(VRegister, VRegister),
    And(VRegister, VRegister),
    Xor(VRegister, VRegister),
    Sub(VRegister, VRegister),
    ShR(VRegister, VRegister),
    SubN(VRegister, VRegister),
    ShL(VRegister, VRegister),
    Rand(VRegister, u8),
    Drw(VRegister, VRegister, u8),
    SkP(VRegister),
    SkNP(VRegister),
}

const fn nibble_op_code(opcode: u16) -> (u8, u8, u8, u8) {
    (
        ((opcode & 0xf000) >> 12) as u8,
        ((opcode & 0x0f00) >> 8) as u8,
        ((opcode & 0x00f0) >> 4) as u8,
        (opcode & 0x000f) as u8,
    )
}

pub fn parse_hex(hex: u32) -> [Sprite; 8] {
    let mut ans = [Sprite::default(); 8];
    for (i, sprite) in ans.iter_mut().enumerate() {
        *sprite = Sprite::from_hex(((hex >> ((7 - i) * 4)) & 0xF) as u8).unwrap_or_default();
    }
    ans
}
