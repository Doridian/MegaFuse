//! The ending's text (the staff, D_016A8C on): `func_016880` reads it
//! (`func_00BE40`) from where its task's animation long points.  An
//! entry: a byte 0x80 and a count of frames to wait; a byte above 0x80,
//! the end (read again each frame); else flags (`FLAGS`), the frames to
//! wait, the frames the text stays (to a task of `func_016A16`), then
//! the text, written at once (`func_01691E`) and once more as spaces
//! when it goes.  In the text: 1, a column and a row (`func_00BE5A`);
//! `#`, a hex digit for the colour bits (`func_00BE74`); a space; a
//! letter: two cells, one above the other (`func_0169A4`).

use crate::{Game, Regs};

/// The frames `func_016880` still waits (each eighth frame one less).
const WAIT: u32 = 0xFFAD7B;
/// The entry's flags: bit 0 the letters' first cells (else +0x80),
/// bit 1 `ARG_80C4`'s high byte 0x78, bit 2 no `func_017828`.
const FLAGS: u32 = 0xFFAD7C;
/// Not 0: `func_01691E` writes spaces in place of the letters.
const BLANK: u32 = 0xFFAD7D;
/// Flags of the entries gone (`func_016A68` sets theirs).
const GONE: u32 = 0xFFAD79;
/// The frames the text stays, as read.
const STAYS: u32 = 0xFF9ED3;
/// The two cells `func_0169A4` writes (the upper's, the lower's).
const UPPER: u32 = 0xFF9ED5;
const LOWER: u32 = 0xFF9ED4;
/// The cell for a space (and for the letters when `BLANK` is set).
const SPACE: u8 = 0x34;

/// Where `func_00BE40` reads (a long), the bits 5-6 of the cells'
/// colour (`func_00BE74`), the VRAM address of the next cell (a word,
/// `func_00BE5A`; two cells on, 0x40 a row).
const READ_AT: u32 = 0xFFAC82;
const DIGIT_BITS: u32 = 0xFFAC7E;
const CELL: u32 = 0xFFAC86;
/// The longs `func_0168BA` keeps the text's place in (the first: also
/// a word `func_017828` takes, which `func_016A16` sets).
const ARG_80C4: u32 = 0xFF80C4;
const KEPT_80D0: u32 = 0xFF80D0;
/// A count of frames: the waits go down when its low 3 bits are 0.
const BYTE_80D8: u32 = 0xFF80D8;
/// The task running (its index).
const CURRENT_TASK: u32 = 0xFF80EE;

/// The VDP's control port and data port.
const CONTROL: u32 = 0xC00004;
const DATA: u32 = 0xC00000;
/// A cell's high byte: priority, the colour bits, tile 0x600 on.
const CELL_HIGH: u8 = 0x86;

/// A task's count, flags (the entry's) and animation long (here the
/// place in the text).
const TASK_COUNT: u32 = 0x16;
const TASK_FLAGS: u32 = 0x17;
const TASK_ANIM: u32 = 0x20;
/// The routines of the text's task.
const ROUTINE_16A16: u32 = 0x016A16;
const ROUTINE_16A68: u32 = 0x016A68;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_016880`: the text's next entry, unless it waits (`WAIT`
    /// one less each eighth frame).  Gives back d0, d1, d2, d6, a6, X
    /// and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_016880(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d5: u32, d6: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, bool, bool) {
        let wait = self.read8(WAIT);
        let d0 = low_byte(d0, wait);
        if wait != 0 {
            let (d0, eighth) = self.r_0169fe(d0);
            if eighth {
                self.write8(WAIT, wait - 1);
                return (d0, d1, d2, d6, a6, false, false);
            }
            return (d0, d1, d2, d6, a6, x, false);
        }
        let at = self.read32(a6 + TASK_ANIM);
        self.write32(READ_AT, at);
        let (d0, negative, _) = self.r_00be40(d0);
        if negative {
            let byte = d0 as u8 & 0x7F;
            let d0 = low_byte(d0, byte);
            if byte != 0 {
                return (d0, d1, d2, d6, a6, x, false);
            }
            let (d0, _, _) = self.r_00be40(d0);
            self.write8(WAIT, d0 as u8);
            let (d0, c) = self.r_0168ba(a6);
            return (d0, d1, d2, d6, a6, x, c);
        }
        self.write8(FLAGS, d0 as u8);
        let (d0, _, _) = self.r_00be40(d0);
        self.write8(WAIT, d0 as u8);
        let (d0, _, _) = self.r_00be40(d0);
        self.write8(STAYS, d0 as u8);
        let (d0, _) = self.r_0168ba(a6);
        let (d6, a6, x, _) = self.r_0003e8(d4, d5, d6, 0x40, ROUTINE_16A16);
        let flags = self.read8(FLAGS);
        self.write8(a6 + TASK_FLAGS, flags);
        let stays = self.read8(STAYS);
        self.write8(a6 + TASK_COUNT, stays);
        let text = self.read32(ARG_80C4);
        self.write32(a6 + TASK_ANIM, text);
        self.write8(BLANK, 0);
        let (_, d1, d2, _) = self.r_01691e(d0, d1, d2, a6, x);
        let current = self.read8(CURRENT_TASK);
        let (d6, a6, x) = self.r_0005f8(low_byte(d6, current));
        let (d0, c) = self.r_0168ba(a6);
        (d0, d1, d2, d6, a6, x, c)
    }

    /// `func_0168BA`: the place read to (`READ_AT`) kept in the task's
    /// animation long, `ARG_80C4` and `KEPT_80D0`.  Gives back d0 (it)
    /// and C (clear).
    pub fn r_0168ba(&mut self, a6: u32) -> (u32, bool) {
        let at = self.read32(READ_AT);
        self.write32(a6 + TASK_ANIM, at);
        self.write32(ARG_80C4, at);
        self.write32(KEPT_80D0, at);
        (at, false)
    }

    /// `.L0168CA`: returns.
    pub fn r_0168ca(&mut self) {}

    /// `func_01691E`: the text from the task's animation long written
    /// to the end (a 0 byte; `READ_AT` after it).  Gives back d0, d1,
    /// d2 and X.
    pub fn r_01691e(&mut self, d0: u32, d1: u32, d2: u32, a6: u32, x: bool) -> (u32, u32, u32, bool) {
        let (mut d0, mut d1, mut d2, mut x) = (d0, d1, d2, x);
        let at = self.read32(a6 + TASK_ANIM);
        self.write32(READ_AT, at);
        loop {
            let zero;
            (d0, _, zero) = self.r_00be40(d0);
            if zero {
                return (d0, d1, d2, x);
            }
            match d0 as u8 {
                1 => {
                    (d0, _, _) = self.r_00be40(d0);
                    d2 = low_byte(d2, d0 as u8);
                    (d0, _, _) = self.r_00be40(d0);
                    d1 = low_byte(d1, d0 as u8);
                    (d1, d2, x) = self.r_00be5a(d1, d2);
                }
                b'#' => {
                    x = self.r_00be74(d0);
                }
                letter => {
                    if letter == b' ' || self.read8(BLANK) != 0 {
                        d0 = u32::from(SPACE);
                        self.write8(UPPER, SPACE);
                        self.write8(LOWER, SPACE);
                    } else {
                        let mut upper = letter.wrapping_sub(b'A').wrapping_mul(2);
                        if self.read8(FLAGS) & 1 == 0 {
                            upper |= 0x80;
                        }
                        self.write8(UPPER, upper);
                        d0 = low_byte(d0, upper.wrapping_add(1));
                        self.write8(LOWER, upper.wrapping_add(1));
                    }
                    (d0, x) = self.r_0169a4(d0);
                }
            }
        }
    }

    /// `func_0169A4`: the cells `UPPER` at `CELL` and `LOWER` a row
    /// under it (interrupts masked), `CELL` 2 on.  Gives back d0 and X.
    pub fn r_0169a4(&mut self, d0: u32) -> (u32, bool) {
        let ipl = self.ipl();
        self.set_ipl(7);
        let cell = self.read16(CELL);
        let (command, _) = self.r_000282((d0 & 0xFFFF_0000) | u32::from(cell));
        self.write32(CONTROL, command);
        let high = self.read8(DIGIT_BITS) | CELL_HIGH;
        let upper = self.read8(UPPER);
        self.write16(DATA, u16::from(high) << 8 | u16::from(upper));
        let (command, _) = self.r_000282((d0 & 0xFFFF_0000) | u32::from(cell.wrapping_add(0x40)));
        self.write32(CONTROL, command);
        let high = self.read8(DIGIT_BITS) | CELL_HIGH;
        let lower = self.read8(LOWER);
        let d0 = (command & 0xFFFF_0000) | u32::from(high) << 8 | u32::from(lower);
        self.write16(DATA, d0 as u16);
        self.set_ipl(ipl);
        let (next, x) = self.read16(CELL).overflowing_add(2);
        self.write16(CELL, next);
        (d0, x)
    }

    /// `func_0169FE`: C set on each eighth frame (`BYTE_80D8`'s low 3
    /// bits 0).  Gives back d0 (those bits) and C.
    pub fn r_0169fe(&mut self, d0: u32) -> (u32, bool) {
        let bits = self.read8(BYTE_80D8) & 7;
        (low_byte(d0, bits), bits == 0)
    }

    /// `func_016A16`, the text's task's routine: each eighth frame its
    /// count goes down; at 0 it is 0x40 again and the routine
    /// `func_016A68` (from the next frame), `ARG_80C4`'s word 0x7000
    /// (0x7800 with flag bit 1) and, without flag bit 2,
    /// `func_017828` with 0x0E.
    pub fn r_016a16(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let eighth;
        (r.d0, eighth) = self.r_0169fe(r.d0);
        r.c = false;
        if !eighth {
            return;
        }
        let count = self.read8(r.a6 + TASK_COUNT);
        r.d0 = low_byte(r.d0, count);
        if count == 0 {
            return;
        }
        self.write8(r.a6 + TASK_COUNT, count - 1);
        r.x = false;
        if count != 1 {
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x40);
        (r.x, _) = self.r_00053e(r.d6, ROUTINE_16A68);
        self.write16(ARG_80C4, 0x7000);
        let flags = self.read8(r.a6 + TASK_FLAGS);
        if flags & 2 != 0 {
            self.write8(ARG_80C4, 0x78);
        }
        r.c = false;
        if flags & 4 == 0 {
            r.d0 = 0x0E;
            (r.x, r.c) = self.r_017828(r.d0, r.d4, r.d5, r.d6);
        }
    }

    /// `.L016A66`: returns.
    pub fn r_016a66(&mut self) {}

    /// `func_016A68`, the text's task's routine: after its count's
    /// frames the text written again as spaces (`BLANK`), its flags put
    /// in `GONE`, and the task ends.  Gives back C (set when the count
    /// was 0).
    pub fn r_016a68(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(r.a6 + TASK_COUNT);
        let left = count.wrapping_sub(1);
        self.write8(r.a6 + TASK_COUNT, left);
        r.x = count == 0;
        r.c = r.x;
        if left != 0 {
            return;
        }
        let flags = self.read8(r.a6 + TASK_FLAGS);
        self.write8(FLAGS, flags);
        self.write8(BLANK, 1);
        (r.d0, r.d1, r.d2, r.x) = self.r_01691e(r.d0, r.d1, r.d2, r.a6, r.x);
        let flags = self.read8(FLAGS);
        r.d0 = low_byte(r.d0, flags);
        let gone = self.read8(GONE);
        self.write8(GONE, gone | flags);
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }
}
