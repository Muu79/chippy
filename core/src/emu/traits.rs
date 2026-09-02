use crate::emu::targets::{Quirk, Quirks};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

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
impl BitOr<Quirk> for Quirks {
    type Output = Self;
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
impl BitAnd for Quirks {
    type Output = Self;
    fn bitand(self, other: Self) -> Self::Output {
        Self {
            quirk_map: self.quirk_map & other.quirk_map,
        }
    }
}
impl BitAnd<Quirk> for Quirks {
    type Output = Self;
    fn bitand(self, other: Quirk) -> Self::Output {
        Self {
            quirk_map: self.quirk_map & other as u16,
        }
    }
}
impl BitAndAssign<Quirk> for Quirks {
    fn bitand_assign(&mut self, other: Quirk) {
        self.quirk_map &= other as u16
    }
}