//! The first four objects put in order by bits 6-7 of their byte 0x2D
//! (`func_00B126`, `func_00B158`), for `func_00AF26`'s list.

use crate::Game;

/// The four object indices in their order (bytes).
const ORDER: u32 = 0xFFFF_9E88;
/// The bits 6-7 `func_00B158` looks for.
const SOUGHT: u32 = 0xFFFF_AC8B;
/// The object's byte whose bits 6-7 are compared.
const OBJ_GROUP: u32 = 0x2D;
const GROUP_BITS: u8 = 0xC0;
/// How many objects are ordered.
const COUNT: u8 = 4;

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_00B126`: `ORDER` made: first the objects (of the first
    /// four) with bits 6-7 0b10, then 0b11, then 0b00, until it has
    /// four.
    pub fn r_00b126(&mut self, d1: u32, d4: u32) {
        let mut d1 = low_word(d1, 0);
        for group in [0x80, 0xC0, 0x00] {
            self.write8(SOUGHT, group);
            let full;
            (d1, full) = self.r_00b158(d1, d4);
            if full {
                break;
            }
        }
    }

    /// `func_00B158`: the objects 0-3 whose bits 6-7 are `SOUGHT`'s
    /// added to `ORDER` from entry `d1` (its low word) on.  Gives back
    /// d1 (the next entry) and C, set once there are four.
    pub fn r_00b158(&mut self, d1: u32, d4: u32) -> (u32, bool) {
        let mut d1 = d1;
        let sought = self.read8(SOUGHT);
        for index in 0..COUNT {
            let (_, object, _) = self.r_001cfa((d4 & 0xFFFF_0000) | u32::from(index));
            if (self.read8(object + OBJ_GROUP) ^ sought) & GROUP_BITS == 0 {
                let entry = d1 as u16;
                self.write8(ORDER.wrapping_add(entry as i16 as u32), index);
                d1 = low_word(d1, entry.wrapping_add(1));
                if d1 as u8 == COUNT {
                    return (d1, true);
                }
            }
        }
        (d1, false)
    }
}
