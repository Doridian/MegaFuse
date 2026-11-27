//! The pads as the menus read them: up to four of the pad records at
//! `PADS` (those whose first word is below 2) listed at `PAD_LIST`
//! (`func_0103D4`) and copied to `COPIES` (`func_01041A`); the bits of
//! the copies or'd together, the held directions repeated
//! (`func_010268`) into `INPUT`; and settings stepped up and down by
//! `INPUT` (`func_01031A`).

use crate::Game;

/// The pad records: `PAD_COUNT` of `PAD_SIZE` bytes, each a word (below
/// 2 for the ones listed) and bytes.
const PADS: u32 = 0xFFFF_800A;
const PAD_COUNT: u8 = 8;
const PAD_SIZE: u32 = 0x0A;
/// The listed records' addresses (four longs; `PADS` for the ones
/// left).
const PAD_LIST: u32 = 0xFFFF_AD2A;
const LISTED: u8 = 4;
/// Copies of the listed records, `COPY_SIZE` bytes each: the word and
/// the bytes 2 and 3 (`func_01041A`), and the bytes 4 and 5.  Bytes 2
/// and 4 are or'd for the held bits (`func_0102B6`), bytes 3 and 5 for
/// the pressed ones (`func_0102E8`).
const COPIES: u32 = 0xFFFF_AD0A;
const COPY_SIZE: u32 = 8;
const PAD_KIND: u32 = 0;
const PAD_HELD: u32 = 2;
const PAD_PRESSED: u32 = 3;
const PAD_HELD_2: u32 = 4;
const PAD_PRESSED_2: u32 = 5;
/// Bit 0: all four copies made (else only the first).
const SETTINGS: u32 = 0xFF9540;

/// The menus' input (a word): the held directions (`DIRECTIONS`) when
/// they change and every `REPEAT_RATE` frames after `REPEAT_DELAY`, or'd
/// with the pressed bits (`PRESSED_BITS`).  Its high byte's bits 0-3
/// step the settings of `func_01031A`.
const INPUT: u32 = 0xFFAD02;
const LAST_HELD: u32 = 0xFFAD04;
const REPEAT: u32 = 0xFFAD06;
const DIRECTIONS: u16 = 0x0F00;
const PRESSED_BITS: u16 = 0xF00F;
const REPEAT_DELAY: u8 = 0x20;
const REPEAT_RATE: u8 = 6;

/// A setting's step, its highest and lowest values (`func_0103B0`, from
/// the list `func_01031A` is given).
const SETTING_STEP: u32 = 0xFFAD07;
const SETTING_MAX: u32 = 0xFFAD08;
const SETTING_MIN: u32 = 0xFFAD09;

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_010268`: `INPUT` made: the held directions when they
    /// differ from `LAST_HELD`, and after that once `REPEAT_DELAY`
    /// frames then every `REPEAT_RATE` frames; or'd with the pressed
    /// bits.
    pub fn r_010268(&mut self, d0: u32) {
        let held = self.r_0102b6(d0) as u16 & DIRECTIONS;
        if held != self.read16(LAST_HELD) {
            self.write16(LAST_HELD, held);
            self.write16(INPUT, held);
            self.write8(REPEAT, REPEAT_DELAY);
        } else {
            let left = self.read8(REPEAT).wrapping_sub(1);
            self.write8(REPEAT, left);
            if left == 0 {
                self.write16(INPUT, held);
                self.write8(REPEAT, REPEAT_RATE);
            } else {
                self.write16(INPUT, 0);
            }
        }
        let pressed = self.r_0102e8(d0) as u16 & PRESSED_BITS;
        let input = self.read16(INPUT) | pressed;
        self.write16(INPUT, input);
    }

    /// `func_0102B6`: the bytes 2 of the four copies or'd (those whose
    /// word is below 2) in d0's high byte of the low word, the bytes 4
    /// in its low byte.
    pub fn r_0102b6(&mut self, d0: u32) -> u32 {
        self.o_pad_bits(d0, PAD_HELD, PAD_HELD_2)
    }

    /// `func_0102E8`: as `func_0102B6` with the bytes 3 and 5.
    pub fn r_0102e8(&mut self, d0: u32) -> u32 {
        self.o_pad_bits(d0, PAD_PRESSED, PAD_PRESSED_2)
    }

    fn o_pad_bits(&mut self, d0: u32, high: u32, low: u32) -> u32 {
        let (mut h, mut l) = (0u8, 0u8);
        for i in 0..4 {
            let copy = COPIES + i * COPY_SIZE;
            if self.read16(copy + PAD_KIND) < 2 {
                h |= self.read8(copy + high);
                l |= self.read8(copy + low);
            }
        }
        low_word(d0, u16::from(h) << 8 | u16::from(l))
    }

    /// `func_01031A`: two settings stepped by `INPUT` (the list at
    /// `list`, two entries, `func_0103B0`): the first up by bit 9, else
    /// down by bit 8; the second up by bit 11, else down by bit 10.
    /// C and X: one changed.
    pub fn r_01031a(&mut self, a1: u32, list: u32) -> (bool, bool) {
        let mut d1 = 0u32;
        let (mut list, mut a1) = (list, a1);
        for (up, down) in [(1, 0), (3, 2)] {
            let end;
            (list, a1, end) = self.r_0103b0(list, a1);
            if !end {
                let bits = self.read8(INPUT);
                if bits & (1 << up) != 0 {
                    d1 = self.r_01037c(d1, a1);
                } else if bits & (1 << down) != 0 {
                    d1 = self.r_010398(d1, a1);
                }
            }
        }
        let changed = d1 as u8 & 0x80 != 0;
        (changed, changed)
    }

    /// `func_01037C`: the byte at a1 a step up, past `SETTING_MAX` to
    /// `SETTING_MIN`.  d1 one less.
    pub fn r_01037c(&mut self, d1: u32, a1: u32) -> u32 {
        let value = self.read8(a1).wrapping_add(self.read8(SETTING_STEP));
        let value = if value <= self.read8(SETTING_MAX) { value } else { self.read8(SETTING_MIN) };
        self.r_010394(u32::from(value), a1);
        d1.wrapping_sub(1)
    }

    /// `func_010394`: d0's low byte to a1.
    pub fn r_010394(&mut self, d0: u32, a1: u32) {
        self.write8(a1, d0 as u8);
    }

    /// `func_010398`: the byte at a1 a step down, below `SETTING_MIN`
    /// (or below 0) to `SETTING_MAX`.  d1 one less.
    pub fn r_010398(&mut self, d1: u32, a1: u32) -> u32 {
        let (value, borrow) = self.read8(a1).overflowing_sub(self.read8(SETTING_STEP));
        let value = if !borrow && value >= self.read8(SETTING_MIN) { value } else { self.read8(SETTING_MAX) };
        self.r_010394(u32::from(value), a1);
        d1.wrapping_sub(1)
    }

    /// `func_0103B0`: a setting's entry at a0 (8 bytes): its step,
    /// highest and lowest values to `SETTING_STEP`.. and its byte's
    /// address in a1.  A step with bit 7 set ends the list (C; a0 two
    /// bytes on).
    pub fn r_0103b0(&mut self, a0: u32, a1: u32) -> (u32, u32, bool) {
        let step = self.read8(a0);
        if step & 0x80 != 0 {
            return (a0.wrapping_add(2), a1, true);
        }
        self.write8(SETTING_STEP, step);
        let max = self.read8(a0.wrapping_add(1));
        self.write8(SETTING_MAX, max);
        let min = self.read8(a0.wrapping_add(2));
        self.write8(SETTING_MIN, min);
        let target = self.read32(a0.wrapping_add(4));
        (a0.wrapping_add(8), target, false)
    }

    /// `func_0103D4`: `PAD_LIST`: the first four pad records whose word
    /// is below 2 (`PADS` for the rest).  X clear.
    pub fn r_0103d4(&mut self) -> bool {
        for i in 0..u32::from(LISTED) {
            self.write32(PAD_LIST + 4 * i, PADS);
        }
        let mut slot = PAD_LIST;
        let mut left = LISTED;
        for i in 0..u32::from(PAD_COUNT) {
            let pad = PADS + i * PAD_SIZE;
            if self.read16(pad + PAD_KIND) < 2 {
                self.write32(slot, pad);
                slot += 4;
                left -= 1;
                if left == 0 {
                    break;
                }
            }
        }
        false
    }

    /// `func_01041A`: the listed pads' words and bytes 2-3 copied to
    /// `COPIES`: four with bit 0 of `SETTINGS`, else one.  Gives back
    /// d1 (its low word 0xFFFF), a1 and a2 after the last.
    pub fn r_01041a(&mut self, d1: u32) -> (u32, u32, u32) {
        let count = if self.read8(SETTINGS) & 1 != 0 { 4 } else { 1 };
        let (mut list, mut copy) = (PAD_LIST, COPIES);
        for _ in 0..count {
            let pad = self.read32(list);
            list += 4;
            let kind = self.read16(pad + PAD_KIND);
            self.write16(copy + PAD_KIND, kind);
            let held = self.read8(pad + PAD_HELD);
            self.write8(copy + PAD_HELD, held);
            let pressed = self.read8(pad + PAD_PRESSED);
            self.write8(copy + PAD_PRESSED, pressed);
            copy += COPY_SIZE;
        }
        let d1 = low_word(d1, 0xFFFF);
        (d1, list, copy)
    }
}
