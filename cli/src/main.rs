mod debug_panel;
mod display;
mod frontend;
mod keyboard;

use chippy_core::cpu::{Cpu, CpuCode, Target};
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    supports_keyboard_enhancement,
};
use frontend::Frontend;
use log::warn;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::env;
use std::error::Error;
use std::io::{Stdout, stdout};
use std::thread::sleep;
use std::time::{Duration, Instant};

type Term = Terminal<CrosstermBackend<Stdout>>;
struct TerminalGuard {
    enhanced: bool,
}

impl TerminalGuard {
    fn new() -> std::io::Result<(Self, Term)> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;

        let enhanced = supports_keyboard_enhancement()?;
        if enhanced {
            execute!(
                stdout(),
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
            )?;
        }

        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok((Self { enhanced }, terminal))
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // best-effort: we're likely already unwinding/erroring, nothing sane to do
        // with a second failure here except ignore it
        if self.enhanced {
            let _ = execute!(stdout(), PopKeyboardEnhancementFlags);
        }
        let _ = execute!(stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

const CPU_HZ: u16 = 700;
const DISPLAY_HZ: u16 = 60;
const CYCLES_PER_FRAME: u16 = CPU_HZ / DISPLAY_HZ;
const FRAME_TIME: Duration = Duration::from_micros((1_000_000) / DISPLAY_HZ as u64);

fn main() -> Result<(), Box<dyn Error>> {
    let (_guard, mut terminal) = TerminalGuard::new()?;
    let rom_path = env::args().nth(1).ok_or("No ROM file specified")?;
    let target = {
        if let Some(arg) = env::args().nth(2) {
            if arg.to_lowercase() == "scmod" {
                Target::SChip8Modern
            } else if arg.to_lowercase() == "sclassic" {
                Target::SChip8Classic
            } else {
                Target::Chip8
            }
        } else {
            Target::Chip8
        }
    };
    let rom = std::fs::read(rom_path)?;
    let mut cpu = Cpu::new(target);
    let mut frontend = Frontend::new();
    cpu.load_rom(&rom)?;
    loop {
        let frame_start = Instant::now();
        run_cpu_cycles(&mut cpu, &mut frontend)?;
        frontend.update_keys(frontend.poll_events(), cpu.get_keys_mut())?;
        cpu.tick_timers();
        frontend.draw_screen(&mut terminal, &cpu)?;
        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_TIME {
            sleep(FRAME_TIME - elapsed);
        } else {
            warn!("Frame took too long: {}ms", elapsed.as_millis());
        }
    }
}

fn run_cpu_cycles(cpu: &mut Cpu, frontend: &mut Frontend) -> Result<(), &'static str> {
    for _ in 0..CYCLES_PER_FRAME {
        match cpu.tick_cpu()? {
            CpuCode::StartWaitForKey => {
                frontend.update_keys(frontend.poll_events(), cpu.get_keys_mut())?;
                frontend.keys.reset_input_key();
                break;
            }
            _ => continue,
        }
    }
    Ok(())
}
