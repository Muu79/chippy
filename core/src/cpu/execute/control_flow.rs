use crate::cpu::Cpu;

impl Cpu {
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
    pub(super) fn op_return(&mut self) -> Result<(), &'static str> {
        self.pc = self.pop()?;
        Ok(())
    }
    pub(super) fn op_jump(&mut self, nnn: u16) {
        self.pc = nnn;
    }
    
    pub(super) fn op_call(&mut self, nnn: u16) {
        self.push(self.pc);
        self.pc = nnn;
    }
    
    pub(super) fn op_skip
}