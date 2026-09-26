use crate::cpu::Cpu;

impl Cpu {
    pub(super) fn clear_screen(&mut self) {
        self.display.clear()
    }
}