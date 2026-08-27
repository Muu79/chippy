use crate::Term;
use crate::frontend::Frontend;
use chippy_core::cpu::Cpu;
use ratatui::layout::Constraint::{Fill, Length, Min, Percentage, Ratio};
use ratatui::layout::Direction::Horizontal;
use ratatui::layout::{Flex, Offset};
use ratatui::prelude::Constraint::Max;
use ratatui::prelude::Direction::Vertical;
use ratatui::widgets::{BorderType, List, ListItem, Padding, Paragraph, Row, Scrollbar, Table};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};
use std::rc::Rc;

fn is_pixel_on(line: u128, row: usize, col: usize) -> bool {
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
                let width_constraints = [Max(cpu.get_display().get_width() as u16 + 4), Fill(1)];
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
                let (reg, timer, stack) = if cpu.is_extended() {
                    let right_panel =
                        Layout::vertical([Min(17), Max(4), Fill(1)]).spacing(1).split(horizontal[1]);
                    (right_panel[0], right_panel[1], right_panel[2])
                } else {
                    let bot_hor = Layout::horizontal([Fill(1), Fill(1)]).spacing(1).split(vertical[1]);
                    (horizontal[1], bot_hor[0], bot_hor[1])
                };
                [
                    screen_area,
                    reg,
                    timer,
                    stack,
                ]
                .into()
            };
            frame.render_widget(block, area);
            self.render_chip_screen(frame, cpu, areas[0]).err();
            if self.debug_mode {
                self.render_debug_window(frame, cpu, areas[1], areas[2], areas[3])
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

    fn render_debug_window(
        &self,
        frame: &mut Frame,
        cpu: &Cpu,
        reg_area: Rect,
        timer_area: Rect,
        stack_area: Rect,
    ) {
        Self::render_registers(frame, reg_area, cpu);
        Self::render_timers(frame, timer_area, cpu);
        Self::render_stack(frame, stack_area, cpu);
    }

    fn render_registers(frame: &mut Frame, area: Rect, cpu: &Cpu) {
        let v = cpu.v_regs();
        let mut buff = Text::default();
        for (i, &val) in v.iter().enumerate() {
            buff.lines
                .push(Line::from(format!("V{:01X}: {:#04X}", i, val)));
        }
        frame.render_widget(
            Paragraph::new(buff).block(Block::default().title("Vx Registers")),
            area,
        );
    }

    fn render_stack(frame: &mut Frame, area: Rect, cpu: &Cpu) {
        let mut lines = vec![
            format!("PC: {:#06X}", cpu.pc()),
            format!("I:  {:#06X}", cpu.i_reg()),
            "-- stack --".to_string(),
        ];
        for (i, &addr) in cpu.stack().iter().enumerate().rev() {
            lines.push(format!("[{i}] {:#06X}", addr));
        }

        let items: Vec<ListItem> = lines.into_iter().map(ListItem::new).collect();
        let list = List::new(items).block(
            Block::default()
                .title("PC / I / Stack"),
        );
        frame.render_widget(list, area);
    }

    fn render_timers(frame: &mut Frame, area: Rect, cpu: &Cpu) {
        let text = format!(
            "DT: {:>3}   ST: {:>3} {}",
            cpu.delay_timer(),
            cpu.sound_timer(),
            if cpu.is_making_sound() {
                "\u{1F50A}"
            } else {
                ""
            }
        );
        let p = Paragraph::new(text).block(Block::default().title("Timers"));
        frame.render_widget(p, area);
    }
}
