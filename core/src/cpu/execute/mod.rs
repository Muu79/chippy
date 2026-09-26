use crate::cpu::{Cpu, CpuCode, VRegister};
use crate::display::font::Sprite;
use crate::display::{Direction, TargetPlane};
use crate::emu::encode_decode::Opcode;
use crate::emu::encode_decode::Opcode::{
    ClS, Exit, HiRes, LoRes, NoOp, Ret, ScL, ScR, StoreAudioBuffer,
};

pub mod alu;
pub mod audio;
pub mod control_flow;
pub mod graphics;
pub mod memory;

impl Cpu {
    pub(super) fn execute(&mut self, operation: Opcode) -> Result<CpuCode, &'static str> {
        use crate::config::quirks::Quirk::*;
        use crate::emu::encode_decode::Opcode::*;
        match operation {
            NoOp => (),
            ClS => self.clear_screen(),
            Ret => self.op_return()?,
            Jp(nnn) => self.op_jump(nnn),
            Call(nnn) => self.op_call(nnn),
            SEByte(v_x, kk) => {
                if self.get_reg(v_x) == kk {
                    self.skip_instruction()
                }
            }
            SNEByte(v_x, kk) => {
                if self.get_reg(v_x) != kk {
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
            AddVxByte(v_x, kk) => self.op_add_byte(v_x, kk),
            LdReg(v_x, v_y) => {
                let y = self.get_reg(v_y);
                *self.get_reg_mut(v_x) = y;
            }
            Or(v_x, v_y) => {
                let y = self.get_reg(v_y);
                *self.get_reg_mut(v_x) |= y;
                if self.has_quirk(VfExtraReset) {
                    self.vf_flag(false);
                }
            }
            And(v_x, v_y) => {
                let y = self.get_reg(v_y);
                *self.get_reg_mut(v_x) &= y;
                if self.has_quirk(VfExtraReset) {
                    self.vf_flag(false);
                }
            }
            Xor(v_x, v_y) => {
                let y = self.get_reg(v_y);
                *self.get_reg_mut(v_x) ^= y;
                if self.has_quirk(VfExtraReset) {
                    self.vf_flag(false);
                }
            }
            AddVxVy(v_x, v_y) => self.op_add_reg(v_x, v_y),
            Sub(v_x, v_y) => {
                let x = self.get_reg(v_x);
                let y = self.get_reg(v_y);
                *self.get_reg_mut(v_x) = x.wrapping_sub(y);
                self.vf_flag(x >= y);
            }
            ShR(v_x, v_y) => {
                let source = if self.has_quirk(ShiftUsesVx) {
                    self.get_reg(v_x)
                } else {
                    self.get_reg(v_y)
                };
                *self.get_reg_mut(v_x) = source >> 1;
                self.vf_flag(source & 0x1 == 0x1);
            }
            SubN(v_x, v_y) => {
                let x = self.get_reg(v_x);
                let y = self.get_reg(v_y);
                *self.get_reg_mut(v_x) = y.wrapping_sub(x);
                self.vf_flag(y >= x);
            }
            ShL(v_x, v_y) => {
                let source = if self.has_quirk(ShiftUsesVx) {
                    self.get_reg(v_x)
                } else {
                    self.get_reg(v_y)
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
                    nnn.wrapping_add(self.get_reg(v_x) as u16)
                } else {
                    nnn.wrapping_add(self.get_reg(VRegister(0)) as u16)
                };
                self.pc = target;
            }
            Rnd(v_x, kk) => {
                *self.get_reg_mut(v_x) = self.get_rand_byte() & kk;
            }
            Drw(v_x, v_y, n) => {
                let (width, height) = self.display_dimensions();
                let (col, row) = (
                    self.get_reg(v_x) as usize % width,
                    self.get_reg(v_y) as usize % height,
                );

                let draws_16x16 = n == 0 && self.has_quirk(DrawSpriteOnDrwXY0);
                let sprite_h: usize = if draws_16x16 { 16 } else { n as usize };
                let sprite_width: usize = if draws_16x16
                    && (self.is_extended() || self.has_quirk(LoResWideSpriteOnDrwXY0))
                {
                    2
                } else {
                    1
                };

                let mut addr = self.i_reg as usize;
                let mut vf: u8 = 0;

                for plane in self.display.get_plane_idx() {
                    let chomps: Vec<u16> = (0..sprite_h)
                        .map(|r| {
                            let base = addr + r * sprite_width;
                            if sprite_width == 2 {
                                (self.ram[base].reverse_bits() as u16)
                                    | (self.ram[base + 1].reverse_bits() as u16) << 8
                            } else {
                                self.ram[base].reverse_bits() as u16
                            }
                        })
                        .collect();

                    vf += self.display.draw_sprite(
                        row,
                        col,
                        &chomps,
                        plane,
                        self.has_quirk(WrapPixelsOnDraw),
                    );
                    addr += sprite_h * sprite_width; // only advances for planes actually consumed
                }

                self.set_vf(if self.has_quirk(DrwCountsCollisionLines) {
                    vf
                } else {
                    (vf > 0) as u8
                });

                if self.has_quirk(DispWait) && !self.is_extended() {
                    return Ok(CpuCode::DispWait);
                }
            }
            SkP(v_x) => {
                if self.keys.is_pressed(self.get_reg(v_x))? {
                    self.skip_instruction()
                }
            }
            SkNP(v_x) => {
                if !self.keys.is_pressed(self.get_reg(v_x))? {
                    self.skip_instruction()
                }
            }
            LdDTVx(v_x) => *self.get_reg_mut(v_x) = self.delay_timer,
            LdKey(v_x) => {
                self.waiting_for_key = Some(v_x);
                return Ok(CpuCode::KeyWait);
            }
            LdVxDT(v_x) => self.delay_timer = self.get_reg(v_x),
            LdVxST(v_x) => self.sound_timer = self.get_reg(v_x), // IIRC wrapping add on I is not possible
            AddIVx(v_x) => {
                self.i_reg += self.get_reg(v_x) as u16;
            }
            LdSpr(v_x) => {
                let idx = self.get_reg(v_x);
                // SChip 1.0 Quirk
                self.i_reg = if idx > 0xF && self.has_quirk(LargeSpriteOnFx29) {
                    Sprite::from_hex(idx % 0x10, true)? as u16
                } else {
                    Sprite::from_hex(idx, false)? as u16
                }
            }
            LdDeci(v_x) => {
                let val = self.get_reg(v_x) as u16;
                for pow in (0..3).rev() {
                    self.ram[(self.i_reg + (2 - pow)) as usize] =
                        ((val / 10u16.pow(pow as u32)) % 10) as u8;
                }
            }
            LdVxI(v_x) => {
                for i in 0..=v_x.0 {
                    self.ram[self.i_reg as usize + i] = self.get_reg(VRegister(i));
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
                    self.rpl_regs[x] = self.get_reg(VRegister(x));
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
                    self.ram[start + idx] = self.get_reg(VRegister(reg));
                }
            }
            LdVxToVyI(v_x, v_y) => {
                let start = self.i_reg as usize;
                for (idx, reg) in (v_x.0..=v_y.0).enumerate() {
                    *self.get_reg_mut(VRegister(reg)) = self.ram[start + idx];
                }
            }
            SelectPlane(n) => self.display.set_targeted_plane(match n & 0b11 {
                0b11 => TargetPlane::Both,
                0b01 => TargetPlane::Plane1,
                0b10 => TargetPlane::Plane2,
                _ => TargetPlane::None,
            }),
            StoreAudioBuffer => {
                let start = self.i_reg as usize;
                self.audio_pattern[..].copy_from_slice(&self.ram[start..start + 16]);
            }
            SetPitch(v_x) => {
                self.pitch = self.get_reg(v_x);
            }
            LdLargeSpr(v_x) => {
                self.i_reg = Sprite::from_hex(self.get_reg(v_x) & 0xF, true)? as u16;
            }
        };
        Ok(CpuCode::Ok)
    }
}
