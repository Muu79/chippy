use core::ops::{BitOr, BitOrAssign};
use std::ops::{Sub, SubAssign};

/// Quirks the emulator enables/disables to better account for implementation quirks
/// between the chip-8 reference and the physical hardware it was written for.
///
/// When making a new [Cpu], the target passed in determined the default quirk profile
///
/// Each [Cpu] has a set of [Quirks] which can be created by ORing (|) any two [Quirk] enums.
/// [Cpu] also exposes [Cpu::set_quirk], [Cpu::clear_quirk], and [Cpu::has_quirk] methods to
/// change individual quirks.
///
/// ### Example
/// ```rust
/// # use chippy8::emu::targets::*;
/// # use chippy8::hardware::cpu::Cpu;
/// // Cpu built with the CHIP-8 default quirk profile
/// let mut cpu = Cpu::new(Target::Chip8, None);
/// // CHIP-8 expects execution to wait for the next display update
/// assert!(cpu.has_quirk(Quirk::DispWait));
/// // However, we can disable that behavior by clearing it
/// cpu.clear_quirk(Quirk::DispWait);
/// assert!(!cpu.has_quirk(Quirk::DispWait));
///
/// // We can also OR together multiple quirks to create a new quirk profile
/// let profile = Quirk::IncrIOnLd | Quirk::VfExtraReset;
/// // Note that set_quirks will clear any existing quirks as well
/// cpu.set_quirks(profile);
/// assert!(
/// cpu.has_quirk(Quirk::IncrIOnLd)
/// && cpu.has_quirk(Quirk::VfExtraReset)
/// && !cpu.has_quirk(Quirk::DispWait)
/// );
///```
#[repr(u16)]
#[derive(Default, Clone, Copy)]
pub enum Quirk {
    // Making None default means any quirk would cause it to return false for .has_quirk()
    #[default]
    NoQuirk = 0,
    ShiftUsesVx = 1 << 0,
    IncrIOnLd = 1 << 1,
    VfExtraReset = 1 << 2,
    DispWait = 1 << 3,
    JumpUsesVx = 1 << 4,
    HasScrollOps = 1 << 5,
    ClScrOnResChange = 1 << 6,
    LargeSpriteOnFx29 = 1 << 7,
    DrwCountsCollisionLines = 1 << 8,
    ScrHalfOnLoRes = 1 << 9,
    DrawSpriteOnDrwXY0 = 1 << 10,
    LoResWideSpriteOnDrwXY0 = 1 << 11,
    WrapPixelsOnDraw = 1 << 12,
}

/// A collection of [Quirk]s that can be ORed together to create a [Quirks] struct.
#[derive(Default, Copy, Clone)]
pub struct Quirks {
    quirk_map: u16,
}

impl Quirks {
    pub fn contains(&self, quirk: Quirk) -> bool {
        self.quirk_map & (1 << quirk as u16) != 0
    }
}

impl From<u16> for Quirks {
    fn from(quirk_map: u16) -> Self {
        Self { quirk_map }
    }
}

// Bit OR is used to create Quirks structs as well as enabling a specific Quirk on a set of Quirks
impl BitOr for Quirks {
    type Output = Self;
    fn bitor(self, other: Self) -> Self::Output {
        Self {
            quirk_map: self.quirk_map | other.quirk_map,
        }
    }
}
impl BitOr for Quirk {
    type Output = Quirks;
    fn bitor(self, other: Self) -> Self::Output {
        Quirks::from(self as u16 | other as u16)
    }
}
impl BitOr<Quirk> for Quirks {
    type Output = Quirks;
    fn bitor(self, other: Quirk) -> Self::Output {
        Self {
            quirk_map: self.quirk_map | other as u16,
        }
    }
}
impl BitOrAssign<Quirk> for Quirks {
    fn bitor_assign(&mut self, other: Quirk) {
        self.quirk_map |= other as u16;
    }
}

// Sub is used to disable a specific Quirk on a set of Quirks
impl Sub<Quirk> for Quirks {
    type Output = Quirks;
    fn sub(self, other: Quirk) -> Self::Output {
        Quirks::from(self.quirk_map & !(other as u16))
    }
}

impl Sub for Quirks {
    type Output = Quirks;
    fn sub(self, other: Quirks) -> Self::Output {
        Quirks::from(self.quirk_map & !other.quirk_map)
    }
}

impl SubAssign<Quirk> for Quirks {
    fn sub_assign(&mut self, other: Quirk) {
        self.quirk_map &= !(other as u16);
    }
}
