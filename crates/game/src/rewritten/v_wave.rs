//! A length along a direction (0x017000-0x017828): `func_01778A`
//! multiplies a length by a quarter wave (D_0177E8) mirrored and signed
//! by the direction's bits 6 and 7, a sine; `func_01776C` and
//! `func_01777C` take the cosine (the direction a quarter on), and
//! `func_017770`, `func_01776C` give it around 0x80, `func_017780`,
//! `func_01777C` as a signed word.

use crate::Game;

/// The direction kept while `func_01778A` works (a byte; bit 7 tested
/// after the look-up).
const DIRECTION: u32 = 0xFFFF_80C5;
/// The quarter wave: 64 bytes, 0x80 to 0xFF (bit 7 set, the value
/// below it, rising).
const QUARTER_WAVE: u32 = 0x0177E8;
/// A quarter of a turn.
const QUARTER: u8 = 0x40;
/// The middle `func_017770` puts the value around.
const MIDDLE: u8 = 0x80;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_01776C`: `func_017770` a quarter of a turn on (d0's low
    /// byte plus `QUARTER`).  Gives back d0 and X.
    pub fn r_01776c(&mut self, d0: u32, d1: u32, d3: u32) -> (u32, bool) {
        let d0 = low_byte(d0, (d0 as u8).wrapping_add(QUARTER));
        self.r_017770(d0, d1, d3)
    }

    /// `func_017770`: `func_01778A` plus `MIDDLE`.  Gives back d0 and X
    /// (the add's carry: the value was 0x80 or more).
    pub fn r_017770(&mut self, d0: u32, d1: u32, d3: u32) -> (u32, bool) {
        let d0 = self.r_01778a(d0, d1, d3);
        let value = d0 as u8;
        (low_byte(d0, value.wrapping_add(MIDDLE)), value >= MIDDLE)
    }

    /// `func_01777C`: `func_017780` a quarter of a turn on.  Gives back
    /// d0.
    pub fn r_01777c(&mut self, d0: u32, d1: u32, d3: u32) -> u32 {
        let d0 = low_byte(d0, (d0 as u8).wrapping_add(QUARTER));
        self.r_017780(d0, d1, d3)
    }

    /// `func_017780`: `func_01778A` as a signed word.  Gives back d0.
    pub fn r_017780(&mut self, d0: u32, d1: u32, d3: u32) -> u32 {
        let d0 = self.r_01778a(d0, d1, d3);
        low_word(d0, (d0 as u8) as i8 as i16 as u16)
    }

    /// `func_01778A`: the length `d1` (its low byte) along the direction
    /// `d0` (its low byte, kept in `DIRECTION`): the wave's byte (its
    /// index the direction's low 6 bits, turned around when bit 6 is
    /// set) less its bit 7 plus 1, times the length, over 0x100 (the
    /// shift-and-add loop of the original); negated unless the
    /// direction's bit 7 is set.  d1-d4 are kept.  Gives back d0 (the
    /// byte, the rest of it 0).
    pub fn r_01778a(&mut self, d0: u32, d1: u32, _d3: u32) -> u32 {
        let direction = d0 as u8;
        self.write8(DIRECTION, direction);
        let index = if direction & 0x40 != 0 { !direction } else { direction } & 0x3F;
        let mut wave = self.read8(QUARTER_WAVE + u32::from(index));
        if self.read8(DIRECTION) & 0x80 != 0 {
            wave &= 0x7F;
        }
        let negative = wave & 0x80 != 0;
        let factor = u16::from(wave & 0x7F) + 1;
        let value = ((u16::from(d1 as u8) * factor) >> 8) as u8;
        u32::from(if negative { value.wrapping_neg() } else { value })
    }
}
