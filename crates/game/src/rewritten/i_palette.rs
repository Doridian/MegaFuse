//! The colours: 64 words at `COLOURS` (four lines of 16), filled from
//! packed colours (`func_0093FC`) or copied in (`func_00946E`), and a
//! block of RAM copied with its nibbles' high bits set (`func_009056`).

use crate::Game;

/// The colours (64 words, in the Mega Drive's colour format).
const COLOURS: u32 = 0x00FF_5800;
const COLOUR_COUNT: u32 = 64;
/// A word: its low byte the colour `func_0093FC` starts at (masked to
/// 0x3F, the word, once it is 0x40 or more).
const FIRST: u32 = 0xFFFF_80D0;
const FIRST_LOW: u32 = 0xFFFF_80D1;
/// Where the packed colours are (a long; `i_load.rs` sets it).
const PACKED: u32 = 0xFFFF_930A;
/// At most this many colours from `func_0093FC`.
const MOST: u16 = 0x40;
/// Set while the colours are being changed (`func_0094B2`); while it
/// is, `func_0094C8` doesn't call `func_00858A`.
const CHANGING: u32 = 0xFFFF_92F8;
/// Bit 0: set by `func_0094B2`.  Cleared with the colours.
const FLAGS: u32 = 0xFFFF_805B;

/// `func_009056`'s copy: 0x34 longs from `SOURCE` to `TARGET`, each
/// or-ed with `HIGH_BITS`.
const SOURCE: u32 = 0xFFFF_989C;
const TARGET: u32 = 0xFFFF_9C9C;
const LONGS: u32 = 0x34;
const HIGH_BITS: u32 = 0xF0F0_F0F0;

impl Game {
    /// `func_009056`: `LONGS` longs from `SOURCE` to `TARGET`, each with
    /// its bytes' high nibbles set.  Gives back d0 (the last long), d2
    /// (`HIGH_BITS`) and a0 (past the source).
    pub fn r_009056(&mut self) -> (u32, u32, u32) {
        let mut last = 0;
        for i in 0..LONGS {
            last = self.read32(SOURCE + 4 * i) | HIGH_BITS;
            self.write32(TARGET + 4 * i, last);
        }
        (last, HIGH_BITS, SOURCE + 4 * LONGS)
    }

    /// `func_0093FC`: `d1` colours (its low word, at most `MOST`; its
    /// low byte 0 is 256) made from the packed ones at `PACKED`, from
    /// colour `FIRST_LOW` on.  The packed colours come in groups: a
    /// byte of eight top bits (from bit 7), then eight bytes, a colour
    /// each: with its top bit, 9 bits, blue in bits 8-6, green in 5-3,
    /// red in 2-0.  Gives back X (clear).
    pub fn r_0093fc(&mut self, d1: u32, _d2: u32) -> bool {
        let mut count = if d1 as u16 > MOST { MOST as u8 } else { d1 as u8 };
        if self.read16(FIRST) >= 0x40 {
            let first = self.read16(FIRST);
            self.write16(FIRST, first & 0x3F);
        }
        let mut at = COLOURS + 2 * u32::from(self.read8(FIRST_LOW));
        let mut from = self.read32(PACKED);
        loop {
            let mut high = self.read8(from);
            from += 1;
            for _ in 0..8 {
                let packed = self.read8(from);
                from += 1;
                let top = u16::from(high & 0x80) << 1;
                high <<= 1;
                // 9 bits: bbb ggg rrr, each to the high three of a
                // nibble
                let bits = (top | u16::from(packed)) << 2;
                let red = (packed & 7) << 1;
                let green = bits as u8 & 0xE0;
                let blue = (bits << 1) & 0xE00;
                self.write16(at, blue | u16::from(green | red));
                at += 2;
                count = count.wrapping_sub(1);
                if count == 0 {
                    return false;
                }
            }
        }
    }

    /// `func_00946E`: `d1` words (its low byte; 0 is 0x10000) from `a0`
    /// to the colours, from colour `d0` / 2 (its bits 1-6).  Gives back
    /// X: the count was 0.
    pub fn r_00946e(&mut self, d0: u32, d1: u32, a0: u32) -> bool {
        let to = COLOURS + (d0 & 0x7E);
        let count = d1 & 0xFF;
        let words = if count == 0 { 0x10000 } else { count };
        for i in 0..words {
            let w = self.read16(a0.wrapping_add(2 * i));
            self.write16(to + 2 * i, w);
        }
        count == 0
    }

    /// `func_009490`: the colours cleared, and `CHANGING` and `FLAGS`.
    pub fn r_009490(&mut self) {
        for i in 0..COLOUR_COUNT {
            self.write16(COLOURS + 2 * i, 0);
        }
        self.write8(CHANGING, 0);
        self.write8(FLAGS, 0);
    }

    /// `func_0094B0`: an `rts` (no script calls it).
    pub fn r_0094b0(&mut self) {}

    /// `func_0094B2`: `FLAGS` bit 0 set, with `CHANGING` set meanwhile.
    /// Gives back C (clear).
    pub fn r_0094b2(&mut self) -> bool {
        let changing = self.read8(CHANGING);
        self.write8(CHANGING, 1);
        let flags = self.read8(FLAGS);
        self.write8(FLAGS, flags | 1);
        self.write8(CHANGING, changing);
        false
    }

    /// `func_0094C8`: `func_00858A` unless the colours are `CHANGING`.
    pub fn r_0094c8(&mut self, d0: u32) {
        if self.read8(CHANGING) == 0 {
            self.r_00858a(d0);
        }
    }
}
