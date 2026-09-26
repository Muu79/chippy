use crate::cpu::{Cpu, VRegister};

impl Cpu {
    pub(super) fn op_add_byte(&mut self, v_x: VRegister, byte: u8) {
        *self.get_reg_mut(v_x) = self.get_reg(v_x).wrapping_add(byte)
    }

    pub(super) fn op_add_reg(&mut self, v_x: VRegister, v_y: VRegister) {
        let x = self.get_reg(v_x);
        let y = self.get_reg(v_y);
        self.op_add_byte(v_x, y);
        self.set_vf(((x as u16 + y as u16) > 0xFF) as u8)
    }

    pub(super) fn op_sub(&mut self, v_x: VRegister, byte: u8) {

    }
}