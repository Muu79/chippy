use crate::frontend::Frontend;
use chippy_core::hardware::Keyboard;
use chippy_core::parse_hex;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use log::{error, info};
use std::ops::Shl;

pub fn poll_keys(new_state: &Keyboard, curr_state: &mut Keyboard) {}

impl Frontend {
    pub fn update_keys(
        &mut self,
        events: Vec<Event>,
        cpu_keyboard: &mut Keyboard,
    ) -> Result<(), &'static str> {
        let key_events = events.into_iter().filter_map(|e| e.as_key_event());
        for key_event in key_events {
            use KeyEventKind::*;
            let key_char = key_event.code.as_char();
            let key_idx = if let Some(key_char) = key_char {
                parse_hex(key_char).ok()
            } else {
                None
            };
            use KeyCode::*;
            match key_event.kind {
                Press if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    info!("Control modifier pressed");
                    match key_event.code {
                        Char(ch) => match ch {
                            'q' | 'c' | 'Q' | 'C' => return Err("Quitting"),
                            _ => (),
                        },
                        _ => (),
                    }
                }
                Press => match key_event.code {
                    F(1) => self.debug_mode ^= true,
                    Char(_) => {
                        if let Some(key_idx) = key_idx {
                            self.keys.press(key_idx as u8)?;
                            continue;
                        }
                    }
                    _ => (),
                },
                Release => {
                    if let Some(key_idx) = key_idx {
                        self.keys.release(key_idx as u8)?
                    }
                }
                _ => {}
            }
        }
        *cpu_keyboard = self.keys.clone();
        Ok(())
    }
}
