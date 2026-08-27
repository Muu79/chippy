use crate::frontend::Frontend;
use chippy_core::hardware::Keyboard;
use chippy_core::parse_hex;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

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
                    if let Char(ch) = key_event.code {
                        match ch {
                            'q' | 'c' | 'Q' | 'C' => return Err("Quitting"),
                            'i' | 'I' => {},
                            _ => (),
                        }
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
        *cpu_keyboard = self.keys;
        Ok(())
    }
}
