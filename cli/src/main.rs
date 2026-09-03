mod debug_panel;
mod display;
mod frontend;
mod keyboard;
pub mod audio;

use chippy_core::emu::targets::Target;
use chippy_core::hardware::cpu::{Cpu, CpuCode};
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use frontend::Frontend;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::env;
use std::error::Error;
use std::io::{stdout, Stdout};
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use rodio::{MixerDeviceSink, Player};
use crate::audio::{AudioState, Chip8AudioSource};

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

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // restore terminal FIRST, so the panic message that follows
        // actually lands on the user's visible screen, not the alternate buffer
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        // no need to conditionally pop keyboard enhancement flags here with
        // perfect accuracy, best-effort is fine mid-panic
        let _ = execute!(stdout(), PopKeyboardEnhancementFlags);

        original_hook(panic_info); // now print the actual panic message/backtrace
    }));
}

const DISPLAY_HZ: u16 = 60;
const FRAME_TIME: Duration = Duration::from_micros((1_000_000) / DISPLAY_HZ as u64);

fn main() -> Result<(), Box<dyn Error>> {
    install_panic_hook();
    let (_guard, mut terminal) = TerminalGuard::new()?;
    let rom_path = env::args().nth(1).ok_or("No ROM file specified")?;
    let target = {
        if let Some(arg) = env::args().nth(2) {
            if arg.to_lowercase() == "scmod" {
                Target::SChip8Modern
            } else if arg.to_lowercase() == "scleg" {
                Target::SChip8Legacy
            } else if arg.to_lowercase() == "xoc" {
                Target::XOChip
            } else {
                Target::Chip8
            }
        } else {
            Target::Chip8
        }
    };

    let rom = std::fs::read(rom_path)?;
    let mut cpu = Cpu::new(target);
    cpu.load_rom(&rom)?;

    let mut frontend = Frontend::new();
    let (audio_state, _audio_player, _audio_handle) = init_audio(target == Target::XOChip);
    let instructions_per_frame = cpu.get_target().default_instructions_per_frame();
    let mut next_tick = Instant::now();
    loop {
        next_tick += FRAME_TIME;
        frontend.update_keys(frontend.poll_events(), cpu.get_keys_mut())?;
        run_cpu_cycles(&mut cpu, &mut frontend, instructions_per_frame)?;
        cpu.tick_timers();
        frontend.draw_screen(&mut terminal, &cpu)?;
        audio_state.update_state(cpu.is_making_sound(), cpu.get_audio_pattern(), cpu.get_pitch());
        let end_time = Instant::now();
        if next_tick > end_time {
            sleep(next_tick - end_time);
        }
    }
}

fn init_audio(is_xo_chip: bool) -> (Arc<AudioState>, Player, MixerDeviceSink) {
    let state = Arc::new(AudioState::new());
    let audio_source = Chip8AudioSource::new(is_xo_chip, state.clone());
    let audio_handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    let player = rodio::Player::connect_new(audio_handle.mixer());
    audio_handle.mixer().add(audio_source);
    (state, player, audio_handle)
}

fn run_cpu_cycles(cpu: &mut Cpu, frontend: &mut Frontend, instructions_per_frame: usize) -> Result<(), &'static str> {
    for _ in 0..instructions_per_frame {
        match cpu.tick_cpu()? {
            CpuCode::KeyWait => {
                frontend.update_keys(frontend.poll_events(), cpu.get_keys_mut())?;
                frontend.keys.reset_input_key();
                break;
            },
            CpuCode::DispWait => break,
            _ => continue,
        }
    }
    Ok(())
}
