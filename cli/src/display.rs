use crate::frontend::Frontend;
use crate::Term;
use chippy_core::cpu::Cpu;
use ratatui::layout::Constraint::{Fill, Length, Min};
use ratatui::layout::Direction::Horizontal;
use ratatui::prelude::Constraint::Max;
use ratatui::prelude::Direction::Vertical;
use ratatui::widgets::{BorderType, Padding};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};
use std::rc::Rc;
use ratatui::layout::Flex;

#[allow(unused)]
fn is_pixel_on(line: u128, _row: usize, col: usize) -> bool {
    line & (1 << col) != 0
}
fn half_block_char(top: bool, bottom: bool) -> char {
    match (top, bottom) {
        (true, true) => '█',
        (true, false) => '▀',
        (false, true) => '▄',
        (false, false) => ' ',
    }
}
impl Frontend {
    pub fn draw_screen(&self, terminal: &mut Term, cpu: &Cpu) -> std::io::Result<()> {
        type Rects = Rc<[Rect]>;
        terminal.draw(|frame| {
            let block = Block::default().title("Chippy Emulator");
            let area = frame.area();
            let inner = block.inner(area);
            let areas: Rects = {
                let width_constraints = [Max(cpu.get_display().get_width() as u16 + 4), Min(if self.debug_mode { 12 } else { 0 })];
                let height_constraints = [
                    Max((cpu.get_display().get_height() as u16) / 2 + 4),
                    Fill(1),
                ];
                let horizontal = Layout::default()
                    .constraints(width_constraints)
                    .direction(Horizontal)
                    .spacing(1)
                    .split(inner);
                let vertical = Layout::default()
                    .constraints(height_constraints)
                    .direction(Vertical)
                    .spacing(1)
                    .split(inner);
                let mut screen_area = horizontal[0];
                screen_area.height = vertical[0].height;
                let (reg, special_reg_and_stack) = if cpu.is_extended() {
                    let right_panel = Layout::vertical([Length(18), Max(21)])
                        .spacing(1)
                        .flex(Flex::Start)
                        .split(horizontal[1]);
                    let mut reg = right_panel[0].clone();
                    reg.width = 16;
                    let mut special_reg_and_stack = right_panel[1].clone();
                    special_reg_and_stack.width = 16;
                    (reg, special_reg_and_stack)
                } else {
                    let bottom_slice = Layout::horizontal([Max(20)])
                        .spacing(1)
                        .split(vertical[1]);
                    let layout = Layout::horizontal([Max(18), Max(18)]).spacing(1).split(horizontal[1]);
                    let (mut reg_area, mut special_reg_and_stack)  = (layout[0].clone(), layout[1].clone());
                    reg_area.width = 18;
                    reg_area.height = vertical[0].height;
                    special_reg_and_stack.width = 18;
                    special_reg_and_stack.height = vertical[0].height;
                    (reg_area, special_reg_and_stack)
                };
                [screen_area, reg, special_reg_and_stack].into()
            };
            frame.render_widget(block, area);
            self.render_chip_screen(frame, cpu, areas[0]).err();
            if self.debug_mode {
                self.render_debug_window(frame, cpu, areas[1], areas[2])
            };
        })?;

        Ok(())
    }

    fn render_chip_screen(&self, frame: &mut Frame, cpu: &Cpu, area: Rect) -> std::io::Result<()> {
        let (width, height, _capacity) = cpu.get_display().dimensions();
        let screen = Block::default()
            .title("Screen")
            .border_type(BorderType::Rounded)
            .padding(Padding::uniform(1))
            .borders(Borders::ALL);
        let inner = screen
            .inner(area)
            .centered(Length(area.width), Length(area.height));
        let buf = frame.buffer_mut();
        let rows = cpu.get_display().get_screen();
        for cell_row in 0..height.div_ceil(2) {
            let top_row = rows[cell_row * 2];
            let bot_row = if cell_row * 2 + 1 < height {
                rows[cell_row * 2 + 1]
            } else {
                0
            };
            for col in 0..width {
                if inner.x + col as u16 >= area.width || inner.y + cell_row as u16 >= area.height {
                    continue;
                }
                let top_on = (top_row >> col) & 1 != 0;
                let bottom_on = (bot_row >> col) & 1 != 0;

                if let Some(cell) = buf.cell_mut((inner.x + col as u16, inner.y + cell_row as u16))
                {
                    cell.set_char(half_block_char(top_on, bottom_on));
                }
            }
        }
        frame.render_widget(screen, area);
        Ok(())
    }
}
