use std::collections::Bound;
use crate::cpu::{Cpu, TargetQuirk};
use std::fmt::Display;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, RangeBounds};
use crate::cpu::encode_decode::VRegister;
use crate::cpu::lib::Quirks;

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_display())
    }
}

impl From<u16> for Quirks {
    fn from(quirk_map: u16) -> Self {
        Self { quirk_map }
    }
}
impl BitOr for Quirks {
    type Output = Self;
    fn bitor(self, other: Self) -> Self::Output {
        Self {
            quirk_map: self.quirk_map | other.quirk_map,
        }
    }
}
impl BitOr<TargetQuirk> for Quirks {
    type Output = Self;
    fn bitor(self, other: TargetQuirk) -> Self::Output {
        Self {
            quirk_map: self.quirk_map | other as u16,
        }
    }
}
impl BitOrAssign<TargetQuirk> for Quirks {
    fn bitor_assign(&mut self, other: TargetQuirk) {
        self.quirk_map |= other as u16;
    }
}
impl BitAnd for Quirks {
    type Output = Self;
    fn bitand(self, other: Self) -> Self::Output {
        Self {
            quirk_map: self.quirk_map & other.quirk_map,
        }
    }
}
impl BitAnd<TargetQuirk> for Quirks {
    type Output = Self;
    fn bitand(self, other: TargetQuirk) -> Self::Output {
        Self {
            quirk_map: self.quirk_map & other as u16,
        }
    }
}
impl BitAndAssign<TargetQuirk> for Quirks {
    fn bitand_assign(&mut self, other: TargetQuirk) {
        self.quirk_map &= other as u16
    }
}