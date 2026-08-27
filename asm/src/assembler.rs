use std::collections::BTreeMap;
use std::error::Error;
use std::io::{BufRead, Cursor, Read, Write};

struct Assembler<R: Read + BufRead, W: Write> {
    input: Cursor<R>,
    output: W,
    symbol_table: BTreeMap<String, u16>,
}

impl<R: Read + BufRead, W: Write> Assembler<R, W> {
    pub fn new(input: R, output: W) -> Self {
        Self {
            input: Cursor::new(input),
            output,
            symbol_table: BTreeMap::new(),
        }
    }

    pub fn read_line(&mut self) -> Result<(), Box<dyn Error>> {
        let line = self.input.get_mut().read_line(&mut String::new())?;
        Ok(())
    }

    pub fn reset_input(&mut self) {
        self.input.set_position(0);
    }
}