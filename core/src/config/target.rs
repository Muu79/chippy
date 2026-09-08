use crate::config::quirks::Quirk::*;
use Target::*;
use crate::config::quirks::Quirks;
use crate::hardware::cpu::Cpu;
/// Target for CPU to emulate, chippy8 currently supports CHIP-8, Super Chip 1.x, and XO-Chip.
/// The target is set at [Cpu] creation time and cannot be changed due to RAM and register differences
#[derive(PartialEq, Copy, Clone)]
pub enum Target {
    Chip8,
    SChip8Modern,
    SChip8Legacy,
    XOChip,
}

impl Target {
    pub const fn start_address(&self) -> u16 {
        match self {
            Chip8 | SChip8Legacy | SChip8Modern | XOChip => 0x200,
        }
    }

    pub const fn ram_size(&self) -> usize {
        match self {
            Chip8 | SChip8Legacy | SChip8Modern => 1 << 12,
            XOChip => 1 << 16,
        }
    }

    pub const fn default_instructions_per_frame(&self) -> usize {
        match self {
            Chip8 => 13,
            SChip8Legacy => 15,
            SChip8Modern | XOChip => 30,
        }
    }

    /// Get the default quirk profile for the target. These are based off of data by [Steffen @Gulrak Schümann](https://chip8.gulrak.net/)
    pub fn default_quirks(&self) -> Quirks {
        match self {
            Chip8 => Quirks::default() | IncrIOnLd | VfExtraReset | DispWait,
            SChip8Legacy => {
                Quirks::default()
                    | ShiftUsesVx
                    | JumpUsesVx
                    | HasScrollOps
                    | ClScrOnResChange
                    | LargeSpriteOnFx29
                    | DispWait
                    | ScrHalfOnLoRes
                    | DrawSpriteOnDrwXY0
            }
            SChip8Modern => {
                Quirks::default()
                    | ShiftUsesVx
                    | JumpUsesVx
                    | HasScrollOps
                    | ClScrOnResChange
                    | LoResWideSpriteOnDrwXY0
                    | DrawSpriteOnDrwXY0
            }
            XOChip => {
                Quirks::default()
                    | IncrIOnLd
                    | HasScrollOps
                    | ClScrOnResChange
                    | DrawSpriteOnDrwXY0
                    | LoResWideSpriteOnDrwXY0
                    | WrapPixelsOnDraw
            }
        }
    }
}
