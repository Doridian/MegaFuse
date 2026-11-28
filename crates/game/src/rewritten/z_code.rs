//! The four-digit code (0x1BA3C-0x1C016): made from `CODE_STAGE` and
//! `CODE_FLAGS` and shown as four sprites (`func_01BA3C`,
//! `func_01BB2C`, `func_01BB9A`), and the screen where it is entered
//! (`func_01BD36`): the pad changes the digit under the cursor and
//! moves it, a button checks the code (`func_01C024` makes its number)
//! and either takes it (`CODE_STAGE` and `CODE_FLAGS` set, on to
//! `func_01C0D8`) or not (`func_01BFC2` waits, then the cursor again).

use crate::{Game, Regs};

/// A task's sprite record index (byte 3, `task.rs`).
const TASK_SPRITE: u32 = 0x03;
/// A sprite record's fields (`sprite.rs`): its palette (3), frame (4),
/// flags (5), x (a word at 8, 9 its low byte), y (a word at 0xA, 0xB
/// its low byte).
const SPRITE_PALETTE: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X_HIGH: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y_HIGH: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;

/// The numbers the code is made of and that a code sets: a byte (its
/// low 3 bits in the code; `CODE_STAGE_COPY` gets it too) and a byte
/// of flags (4 bits, or 3 when `CODE_STAGE` is 0; all set is told
/// apart by `ALL_SET`).
const CODE_STAGE: u32 = 0xFFFF_953B;
const CODE_STAGE_COPY: u32 = 0xFFFF_953A;
const CODE_FLAGS: u32 = 0xFFFF_953D;
/// 0x20 (made, `func_01BA3C`) or 1 (checked, `func_01BD36`) when the
/// flags were all set.
const ALL_SET: u32 = 0xFFFF_956E;
/// The four digits entered (units first) and the cursor (0-3).
const DIGITS: u32 = 0xFFFF_9564;
const CURSOR: u32 = 0xFFFF_9568;
/// The byte of the number checked.
const CHECKED: u32 = 0xFFFF_9569;
/// `func_01BFC2` waits for `RETRY` not to be 0; the result's bytes.
const RESULT_956C: u32 = 0xFFFF_956C;
const RETRY: u32 = 0xFFFF_956B;
const RESULT_956D: u32 = 0xFFFF_956D;
/// The number's digits (`func_01BB2C`): thousands, hundreds, tens,
/// units; `func_01BAF6` (no script calls it) puts a byte's hundreds,
/// tens (by word, so into `TENS`'s next byte: a bug) and units.
const THOUSANDS: u32 = 0xFFFF_956F;
const HUNDREDS: u32 = 0xFFFF_9570;
const TENS: u32 = 0xFFFF_9571;
const UNITS: u32 = 0xFFFF_9572;
/// The digits shown, units first (`func_01BA3C`, `func_01BB9A`).
const SHOWN: u32 = 0xFFFF_AED6;
/// The number made and checked (a word; `NUMBER_LOW` its low byte) and
/// a second word (`NUMBER_2`, `NUMBER_2_LOW`).
const NUMBER: u32 = 0xFFFF_80C4;
const NUMBER_HIGH: u32 = 0xFFFF_80C4;
const NUMBER_LOW: u32 = 0xFFFF_80C5;
const NUMBER_2: u32 = 0xFFFF_80C8;
const NUMBER_2_HIGH: u32 = 0xFFFF_80C8;
const NUMBER_2_LOW: u32 = 0xFFFF_80C9;
/// What the check of a number's bits is made with.
const CHECK_XOR: u8 = 0xAC;
const CHECK_MASK: u8 = 0x3E;
/// The pad's new presses: bit 0 the digit up, 1 down, 3 the cursor
/// right, 2 left, 4 out, 5-7 the check.
const PRESSED: u32 = 0xFFFF_AD02;
/// While this isn't 0 (a fade runs, `j_colours.rs`) the entry waits.
const STOPPED: u32 = 0xFFFF_A3FC;
/// A long set to 0xABA4 when a code is taken.
const LONG_A40A: u32 = 0xFFFF_A40A;
const LONG_A40A_VALUE: u32 = 0x00ABA4;

/// The digits' x (ROM, by digit), y and frame (0x74 + the digit).
const SHOWN_X: u32 = 0x01BBDC;
const SHOWN_Y: u8 = 0xA0;
const DIGIT_FRAME: u8 = 0x74;
const DIGIT_DEPTH: u8 = 0x1E;
/// The digit up, down; the cursor right, left (ROM).
const DIGIT_UP: u32 = 0x01BE00;
const DIGIT_DOWN: u32 = 0x01BE0A;
const CURSOR_RIGHT: u32 = 0x01BE14;
const CURSOR_LEFT: u32 = 0x01BE18;
/// The cursor's sprite: depth, frame, y; x 0x44 + 0x28 a digit.
const CURSOR_DEPTH: u8 = 0x0A;
const CURSOR_FRAME: u8 = 0x7E;
const CURSOR_Y: u8 = 0x5C;
const CURSOR_X: u8 = 0x44;

/// The routines.
const R_01BD36: u32 = 0x01BD36;
const R_01BFC2: u32 = 0x01BFC2;
const R_01C016: u32 = 0x01C016;
const R_01C0D8: u32 = 0x01C0D8;

/// The Z80's commands: a digit changed, the cursor moved, a code taken,
/// one refused.
const SOUND_DIGIT: u8 = 0x24;
const SOUND_CURSOR: u8 = 0x20;
const SOUND_TAKEN: u8 = 0x21;
const SOUND_REFUSED: u8 = 0x31;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_01BA3C`: the code made: a number from `CODE_STAGE` and
    /// `CODE_FLAGS` (the flags' high nibble when all are set) x 0x40
    /// plus a check of its low byte's bits (and `ALL_SET`), split into
    /// digits (`func_01BB2C`) and shown (`func_01BB9A`).  Gives back
    /// d0, d3, d5, a0, a5, X and C as `func_01BB9A` leaves them.
    pub fn r_01ba3c(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let stage = self.read8(CODE_STAGE);
        self.write8(NUMBER_LOW, (stage & 7) << 4);
        let flags = self.read8(CODE_FLAGS);
        let all = if stage != 0 { flags & 0x0F == 0x0F } else { flags & 7 == 7 };
        let (part, all_set) = if all { ((flags & 0xF0) >> 4, 0x20) } else { (flags & 0x0F, 0) };
        let low = self.read8(NUMBER_LOW).wrapping_add(part);
        self.write8(NUMBER_LOW, low);
        self.write8(ALL_SET, all_set);
        self.write8(NUMBER_HIGH, 0);
        self.write8(NUMBER_2_LOW, low);
        let value = self.read16(NUMBER) << 6;
        self.write16(NUMBER, value);
        let check = (((self.read8(NUMBER_2_LOW) ^ CHECK_XOR) & CHECK_MASK) >> 1).wrapping_add(self.read8(ALL_SET));
        self.write8(NUMBER_2_LOW, check);
        self.write8(NUMBER_2_HIGH, 0);
        let number = self.read16(NUMBER).wrapping_add(self.read16(NUMBER_2));
        self.write16(NUMBER, number);
        let x = self.r_01bb2c();
        for (i, from) in [UNITS, TENS, HUNDREDS, THOUSANDS].into_iter().enumerate() {
            let value = self.read8(from);
            self.write8(SHOWN + i as u32, value);
        }
        self.r_01bb9a(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x)
    }

    /// `func_01BAF6`: d0's low byte's hundreds to `THOUSANDS`, its tens
    /// added a word at a time to `HUNDREDS` (so into `TENS`, which the
    /// units then overwrite), the units to `TENS`.
    /// No script calls it (nothing in the ROM does): reviewed by
    /// reading.
    pub fn r_01baf6(&mut self, d0: u32) {
        let value = d0 as u8;
        self.write8(THOUSANDS, value / 100);
        self.write8(HUNDREDS, 0);
        self.write8(TENS, 0);
        for _ in 0..(value % 100) / 10 {
            let value = self.read16(HUNDREDS).wrapping_add(1);
            self.write16(HUNDREDS, value);
        }
        self.write8(TENS, value % 10);
    }

    /// `func_01BB2C`: `NUMBER` split into its digits (`THOUSANDS` ..
    /// `TENS`), the rest left in `NUMBER` and its low byte in `UNITS`.
    /// Gives back X (the last add's carry, set).
    pub fn r_01bb2c(&mut self) -> bool {
        let mut number = self.read16(NUMBER);
        let mut x = false;
        for (unit, to) in [(1000u16, THOUSANDS), (100, HUNDREDS), (10, TENS)] {
            self.write8(to, (number / unit) as u8);
            // the last subtraction borrows; adding `unit` back carries
            let left = number % unit;
            x = left.wrapping_sub(unit).overflowing_add(unit).1;
            number = left;
        }
        self.write16(NUMBER, number);
        let value = self.read8(NUMBER_LOW);
        self.write8(UNITS, value);
        x
    }

    /// `func_01BB86`: `THOUSANDS` to `NUMBER_LOW`, `HUNDREDS` to
    /// `NUMBER_HIGH`, `TENS` to `NUMBER_2_LOW`.
    /// No script calls it (nothing in the ROM does): reviewed by
    /// reading.
    pub fn r_01bb86(&mut self) {
        let value = self.read8(THOUSANDS);
        self.write8(NUMBER_LOW, value);
        let value = self.read8(HUNDREDS);
        self.write8(NUMBER_HIGH, value);
        let value = self.read8(TENS);
        self.write8(NUMBER_2_LOW, value);
    }

    /// `func_01BB9A`: the four digits at `SHOWN` as sprites (depth
    /// 0x1E, `func_000796`) at `SHOWN_X` and y 0xA0, frame 0x74 + the
    /// digit, flags 0x80.  Gives back d0 (0x1E, its low byte the last
    /// frame), d3 (its low word 4), d5 and a5 (the last record), a0
    /// (`SHOWN`), X and C (clear).
    pub fn r_01bb9a(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, x: bool) -> (u32, u32, u32, u32, u32, bool, bool) {
        let (mut d0, mut d5, mut a0, mut a5, mut x) = (0, d5, a0, a5, x);
        for i in 0..4u8 {
            d0 = u32::from(DIGIT_DEPTH);
            (d5, a5, _, _) = self.r_000796(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
            let value = self.read8(SHOWN_X + u32::from(i));
            self.write8(a5 + SPRITE_X_LOW, value);
            self.write8(a5 + SPRITE_X_HIGH, 0);
            self.write8(a5 + SPRITE_Y_LOW, SHOWN_Y);
            self.write8(a5 + SPRITE_Y_HIGH, 0);
            a0 = SHOWN;
            let frame = self.read8(SHOWN + u32::from(i)).wrapping_add(DIGIT_FRAME);
            d0 = low_byte(d0, frame);
            self.write8(a5 + SPRITE_FRAME, frame);
            self.write8(a5 + SPRITE_PALETTE, 0);
            self.write8(a5 + SPRITE_FLAGS, 0x80);
            x = false;
        }
        // the count's add (to 4) carries nothing
        (d0, low_word(d3, 4), d5, a0, a5, false, false)
    }

    /// The cursor's x: 0x44 + 0x28 x `CURSOR` (by bytes: d0 `CURSOR` <<
    /// 5, d1 `CURSOR` << 3; X the last add's carry).
    fn z_cursor_x(&mut self, r: &mut Regs) -> u8 {
        let cursor = self.read8(CURSOR);
        r.d1 = low_byte(r.d1, cursor << 3);
        let (x, carry) = (cursor << 5).wrapping_add(cursor << 3).overflowing_add(CURSOR_X);
        r.d0 = low_byte(r.d0, x);
        r.x = carry;
        x
    }

    /// `func_01BD36`, the entry's task: unless `STOPPED`, its sprite at
    /// the cursor; a press changes the digit there (up or down, by
    /// `DIGIT_UP`/`DIGIT_DOWN`) or moves the cursor, leaves (a fade
    /// out, its routine `func_01C016`), or checks the code
    /// (`z_check`).
    pub fn r_01bd36(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.c = false;
        if self.read8(STOPPED) != 0 {
            return;
        }
        let x = self.z_cursor_x(r);
        self.write8(r.a5 + SPRITE_X_LOW, x);
        let cursor = self.read8(CURSOR);
        r.d2 = low_word(low_byte(r.d2, cursor), cursor as i8 as u16);
        r.c = false;
        let at = DIGITS.wrapping_add(cursor as i8 as u32);
        let pressed = self.read8(PRESSED);
        if pressed & 0x03 != 0 {
            r.d0 = low_byte(r.d0, SOUND_DIGIT);
            self.r_0034b6(r.d0);
            r.a0 = DIGITS;
            let digit = self.read8(at);
            r.d1 = low_word(low_byte(r.d1, digit), digit as i8 as u16);
            let table = if pressed & 1 != 0 { DIGIT_UP } else { DIGIT_DOWN };
            let value = self.read8(table.wrapping_add(digit as i8 as u32));
            self.write8(at, value);
            r.c = false;
        } else if pressed & 0x0C != 0 {
            r.d0 = low_byte(r.d0, SOUND_CURSOR);
            self.r_0034b6(r.d0);
            let table = if pressed & 8 != 0 { CURSOR_RIGHT } else { CURSOR_LEFT };
            let value = self.read8(table.wrapping_add(cursor as i8 as u32));
            self.write8(CURSOR, value);
            r.c = false;
        } else if pressed & 0x10 != 0 {
            r.d0 = 4;
            (r.d0, r.a1, r.x, r.c) = self.r_00a51a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
            (r.x, r.c) = self.r_00053e(r.d6, R_01C016);
        } else if pressed & 0xE0 != 0 {
            self.z_check(r);
        }
    }

    /// The code entered checked: its number (`func_01C024`) kept in
    /// `NUMBER_2` and x 4 in `NUMBER`; its bits checked against the
    /// check in `NUMBER_2_LOW` (doubled, `ALL_SET` its bit 5), the
    /// number's low byte kept in `CHECKED`, its bits 4-6 (less than 6;
    /// 5 only with no low bits, 0 only without bit 3) to `CODE_STAGE`
    /// and its copy; with `ALL_SET` the low bits (3 or 4 by the stage)
    /// must miss exactly one, `CODE_FLAGS` then the low nibble x 0x10
    /// + 0xF; without, they must not all be set, `CODE_FLAGS` the low
    /// nibble twice.  Taken: `LONG_A40A`, `func_01C258`, its sound,
    /// `RESULT_956C` 1, the routine `func_01C0D8`; refused: its sound,
    /// the routine `func_01BFC2`, `RESULT_956D` 1.  Either way the
    /// cursor's sprite record is freed.
    fn z_check(&mut self, r: &mut Regs) {
        (r.d0, r.d3) = self.r_01c024(r.d0, r.d3);
        let value = self.read16(NUMBER);
        self.write16(NUMBER_2, value);
        let value = self.read16(NUMBER) << 2;
        self.write16(NUMBER, value);
        let all_set = self.read8(NUMBER_2_LOW) & 0x20 != 0;
        self.write8(ALL_SET, u8::from(all_set));
        let check = (self.read8(NUMBER_2_LOW).wrapping_mul(2) ^ CHECK_XOR) & CHECK_MASK;
        self.write8(NUMBER_2_LOW, check);
        let bits = self.read8(NUMBER_HIGH) & CHECK_MASK;
        r.d0 = low_byte(r.d0, bits);
        if bits == check && self.z_code_ok(r) {
            self.write32(LONG_A40A, LONG_A40A_VALUE);
            self.r_01c258();
            r.d0 = low_byte(r.d0, SOUND_TAKEN);
            self.r_0034b6(r.d0);
            self.write8(RESULT_956C, 1);
            self.write8(RETRY, 0);
            self.write8(RESULT_956D, 0);
            (r.x, r.c) = self.r_00053e(r.d6, R_01C0D8);
            r.c = self.r_000890();
            return;
        }
        r.d0 = low_byte(r.d0, SOUND_REFUSED);
        self.r_0034b6(r.d0);
        (r.x, _) = self.r_00053e(r.d6, R_01BFC2);
        self.r_000890();
        self.write8(RESULT_956C, 0);
        self.write8(RETRY, 0);
        self.write8(RESULT_956D, 1);
        r.c = false;
    }

    /// The rest of `z_check` after its first test: whether the code is
    /// taken (`CODE_STAGE`, its copy and `CODE_FLAGS` set when it is).
    /// d0's low byte as the tests leave it.
    fn z_code_ok(&mut self, r: &mut Regs) -> bool {
        let checked = self.read8(NUMBER_HIGH);
        self.write8(CHECKED, checked);
        let mut stage = (checked & 0x70) >> 4;
        r.d0 = low_byte(r.d0, stage);
        if stage >= 6 {
            return false;
        }
        if stage == 5 {
            r.d0 = low_byte(r.d0, checked & 0x0F);
            if checked & 0x0F != 0 {
                return false;
            }
            r.d0 = 5;
            stage = 5;
        } else if stage == 0 && checked & 8 != 0 {
            return false;
        }
        self.write8(CODE_STAGE, stage);
        self.write8(CODE_STAGE_COPY, stage);
        let mask = if self.read8(CODE_STAGE_COPY) != 0 { 0x0F } else { 7 };
        if self.read8(ALL_SET) != 0 {
            let missing = !self.read8(CHECKED) & mask;
            r.d0 = low_byte(r.d0, missing);
            if missing == 0 {
                return false;
            }
            // shifted right to the first bit out; nothing may be left
            let left = missing >> (missing.trailing_zeros() + 1);
            r.d0 = low_byte(r.d0, left);
            if left != 0 {
                return false;
            }
            let flags = ((self.read8(CHECKED) & 0x0F) << 4) | 0x0F;
            r.d0 = low_byte(r.d0, flags);
            self.write8(CODE_FLAGS, flags);
        } else {
            let bits = self.read8(CHECKED) & mask;
            r.d0 = low_byte(r.d0, bits);
            if bits == mask {
                return false;
            }
            let low = self.read8(CHECKED) & 0x0F;
            self.write8(CODE_FLAGS, low);
            self.write8(CODE_FLAGS, low | (low << 4));
            r.d0 = low_byte(r.d0, low << 4);
        }
        true
    }

    /// `func_01BFC2`, the refused code's task: once `RETRY` isn't 0,
    /// its routine `func_01BD36` again with a new sprite record for the
    /// cursor (depth 0xA, frame 0x7E, at the cursor, y 0x5C, flags
    /// 0x80).
    pub fn r_01bfc2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.c = false;
        if self.read8(RETRY) == 0 {
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, R_01BD36);
        r.d0 = u32::from(CURSOR_DEPTH);
        (r.d5, r.a5, r.x, r.c) = self.r_000796(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
        self.write8(r.a6 + TASK_SPRITE, r.d5 as u8);
        self.write8(r.a5 + SPRITE_FRAME, CURSOR_FRAME);
        let x = self.z_cursor_x(r);
        self.write8(r.a5 + SPRITE_X_LOW, x);
        self.write8(r.a5 + SPRITE_X_HIGH, 0);
        self.write8(r.a5 + SPRITE_Y_LOW, CURSOR_Y);
        self.write8(r.a5 + SPRITE_Y_HIGH, 0);
        self.write8(r.a5 + SPRITE_PALETTE, 0);
        self.write8(r.a5 + SPRITE_FLAGS, 0x80);
        r.c = false;
    }
}
