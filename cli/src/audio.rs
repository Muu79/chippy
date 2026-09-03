use std::num::{NonZeroU16, NonZeroU32};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;

pub struct AudioState {
    pattern: [AtomicU64; 2],
    is_playing: AtomicBool,
    pitch: AtomicU8,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            pattern: [AtomicU64::new(0), AtomicU64::new(0)],
            is_playing: AtomicBool::new(false),
            pitch: AtomicU8::new(64),
        }
    }

    pub fn update_state(&self, is_playing: bool, pattern: &[u8], pitch: u8) {
        self.is_playing.store(is_playing, Ordering::Relaxed);
        self.pattern[0].store(u64::from_be_bytes(pattern[0..8].try_into().unwrap()), Ordering::Relaxed);
        self.pattern[1].store(u64::from_be_bytes(pattern[8..16].try_into().unwrap()), Ordering::Relaxed);
        self.pitch.store(pitch, Ordering::Relaxed);
    }
}

pub struct Chip8AudioSource {
    state: Arc<AudioState>,
    sample_rate: u32,
    phase: f64,
    tone_phase: f64, // legacy tone for pre xo-chip
    xo_chip: bool,
}

impl Chip8AudioSource {
    pub fn new(xo_chip: bool, state: Arc<AudioState>) -> Self {
        Self {
            state,
            sample_rate: 44100,
            phase: 0.0,
            tone_phase: 0.0,
            xo_chip,
        }
    }

    pub fn with_sample_rate(self, sample_rate: u32) -> Self {
        Self { sample_rate, ..self }
    }

    pub fn with_state(self, state: Arc<AudioState>) -> Self {
        Self { state, ..self }
    }

    pub fn with_xo_chip_audio(self, xo_chip_audio: bool) -> Self {
        Self { xo_chip: xo_chip_audio, ..self }
    }

}

const BEEP_HZ: f64 = 440.0;
impl Iterator for Chip8AudioSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if !self.state.is_playing.load(Ordering::Relaxed) {
            Some(0.0)
        } else if self.xo_chip {
            let pitch = self.state.pitch.load(Ordering::Relaxed);
            let freq = 4000.0 * 2.0f64.powf((pitch - 64) as f64 / 48.0);
            self.phase = (self.phase + freq / self.sample_rate as f64) % 128.0;

            let bit_idx = self.phase as usize;
            let word = self.state.pattern[bit_idx / 64].load(Ordering::Relaxed);
            let bit = (word >> (bit_idx % 64)) & 1;
            Some(if bit == 1 { 0.075 } else { -0.075 })
        } else {
            self.tone_phase = (self.tone_phase + BEEP_HZ / self.sample_rate as f64) % 1.0;
            Some(if self.tone_phase < 0.5 { 0.075 } else { -0.075 })
        }
    }
}

impl rodio::Source for Chip8AudioSource {
    fn current_span_len(&self) -> Option<usize> { None }
    fn channels(&self) -> NonZeroU16 { NonZeroU16::new(1u16).unwrap() }
    fn sample_rate(&self) -> NonZeroU32 { NonZeroU32::new(self.sample_rate).unwrap() }
    fn total_duration(&self) -> Option<std::time::Duration> { None }
}