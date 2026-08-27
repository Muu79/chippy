use crate::frontend::Frontend;
use chippy_core::cpu::Cpu;
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Line, Stylize, Text};
use ratatui::style::Color;
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Padding, Paragraph, Shadow};

const SIDE_BORDERS: Borders = Borders::LEFT.union(Borders::RIGHT);
impl Frontend {
    pub(crate) fn render_debug_window(
        &self,
        frame: &mut Frame,
        cpu: &Cpu,
        reg_area: Rect,
        stack_area: Rect,
    ) {
        Self::render_registers(frame, reg_area, cpu);
        Self::render_special_reg_and_stack(frame, stack_area, cpu);
    }

    fn render_registers(frame: &mut Frame, area: Rect, cpu: &Cpu) {
        let v = cpu.v_regs();
        let mut buff = Text::default();
        for (i, &val) in v.iter().enumerate() {
            let (idx, val) = (format!("{:01X}", i).fg(Color::Yellow), format!("{:#04X}", val).fg(Color::Cyan).bg(Color::White));
            buff.lines
                .push(Line::from(format!("V{}: {}", idx, val)).blue());
        }
        frame.render_widget(
            Paragraph::new(buff).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Vx Registers")
                    .title_alignment(Alignment::Center)
                    .padding(Padding::uniform(1)),
            ),
            area,
        );
    }

    fn render_special_reg_and_stack(frame: &mut Frame, area: Rect, cpu: &Cpu) {
        let mut lines = vec![
            format!("PC: {:#06X}", cpu.pc()),
            format!("I:  {:#06X}", cpu.i_reg()),
            format!("DT: {:>#03X} ({}s)", cpu.delay_timer(), cpu.delay_timer() / 60),
            format!(
                "ST: {:>#03X} ({}s){}",
                cpu.sound_timer(),
                cpu.sound_timer() / 60,
                if cpu.is_making_sound() {
                    " \u{1F50A}"
                } else {
                    ""
                }
            ),
            "-- stack --".to_string(),
        ];
        for (i, &addr) in cpu.stack().iter().enumerate().rev() {
            lines.push(format!("[{i}] {:#06X}", addr));
        }

        let items: Vec<ListItem> = lines.into_iter().map(ListItem::new).collect();
        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("PC / I / DT / ST / Stack")
                .title_alignment(Alignment::Center)
                .padding(Padding::uniform(1)),
        );
        frame.render_widget(list, area);
    }
}
