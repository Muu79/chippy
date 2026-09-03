use Quirk::*;
use Target::*;
/// Target for CPU to emulate
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

#[derive(Default)]
pub struct Quirks {
    pub(crate) quirk_map: u16,
}
