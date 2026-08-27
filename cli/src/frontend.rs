use chippy_core::cpu::Target;
use chippy_core::hardware::Keyboard;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use std::time::Duration;

/// Represents the frontend component of the system, which handles display and keyboard state.
///
/// # Fields
///
/// * `display_buffer` (pub(crate)):
///   A fixed-size array of 64-bit integers representing the state of the display buffer.
///   Typically used to manage or render graphical output. The array has a size of 32 elements.
///
/// * `curr_keyboard` (pub(crate)):
///   The current state of the keyboard, represented as a `Keyboard` struct.
///   It manages or monitors the inputs coming from the keyboard.
///
/// # Visibility
///
/// Both fields are restricted to the crate scope using `pub(crate)`, making them
/// inaccessible outside of the crate while still being usable internally.
pub struct Frontend {
    pub(crate) keys: Keyboard,
    pub(crate) debug_mode: bool,
}

impl Frontend {
    /// Creates a new `[Frontend]` with an empty display buffer and keyboard
    pub fn new() -> Frontend {
        Frontend {
            keys: Keyboard::new(),
            debug_mode: false,
        }
    }

    /// Poll all events triggered since this function's last execution
    pub fn poll_events(&self) -> Vec<Event> {
        let mut events = vec![];
        while event::poll(Duration::ZERO).unwrap() {
            let event = event::read().unwrap();
            events.push(event);
        }
        events
    }
}
