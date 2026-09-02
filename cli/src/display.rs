use crate::frontend::Frontend;
use crate::Term;
use chippy_core::hardware::cpu::Cpu;
use ratatui::layout::Constraint::{Fill, Length, Min};
use ratatui::layout::Direction::Horizontal;
use ratatui::layout::Flex;
use ratatui::prelude::Constraint::Max;
use ratatui::prelude::Direction::Vertical;
use ratatui::widgets::{BorderType, Padding};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};
use std::rc::Rc;

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
                let width_constraints = [
                    Max(cpu.get_display().width() as u16 + 4),
                    Min(if self.debug_mode { 12 } else { 0 }),
                ];
                let height_constraints =
                    [Max((cpu.get_display().height() as u16) / 2 + 4), Fill(1)];
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
                    let layout = Layout::horizontal([Max(18), Max(18)])
                        .spacing(1)
                        .split(horizontal[1]);
                    let (mut reg_area, mut special_reg_and_stack) =
                        (layout[0].clone(), layout[1].clone());
                    reg_area.width = 18;
                    reg_area.height = vertical[0].height;
                    special_reg_and_stack.width = 18;
                    special_reg_and_stack.height = vertical[0].height;
                    (reg_area, special_reg_and_stack)
                };
                [screen_area, reg, special_reg_and_stack].into()
            };
            frame.render_widget(block, area);
            self.render_chip_screen(frame, cpu, areas[0]);
            if self.debug_mode {
                self.render_debug_window(frame, cpu, areas[1], areas[2])
            };
        })?;

        Ok(())
    }

    fn render_chip_screen(&self, frame: &mut Frame, cpu: &Cpu, area: Rect) {
        let (width, height, _capacity) = cpu.get_display().dimensions();
        let planes = cpu.get_display().get_screen();

        let screen_block = Block::default()
            .title("Screen")
            .border_type(BorderType::Rounded)
            .padding(Padding::uniform(1))
            .borders(Borders::ALL);
        let inner = screen_block.inner(area);
        let buff = frame.buffer_mut();
        (0..height.div_ceil(2)).for_each(|cell_row| {
            let top_row = cell_row * 2;
            let bot_row = top_row + 1;
            let (p1_top, p2_top) = (planes.0[top_row], planes.1[top_row]);
            let (p1_bot, p2_bot) = if bot_row < height {
                (planes.0[bot_row], planes.1[bot_row])
            } else {
                (0, 0)
            };

            (0..width).for_each(|col| {
                let top = self.pixel_color((p1_top >> col) & 1 != 0, (p2_top >> col) & 1 != 0);
                let bot = self.pixel_color((p1_bot >> col) & 1 != 0, (p2_bot >> col) & 1 != 0);
                let cell =
                    buff.cell_mut(Position::new(inner.x + col as u16, inner.y + cell_row as u16));
                if let Some(cell) = cell {
                    cell.set_char('▀').set_fg(top).set_bg(bot);
                }
            });
        });
        frame.render_widget(screen_block, area);
    }
    fn pixel_color(&self, plane1: bool, plane2: bool) -> Color {
        match (plane1, plane2) {
            (true, true) => self.overlap_color,
            (true, false) => self.main_color,
            (false, true) => self.sub_color,
            _ => self.background_color,
        }
    }
}
