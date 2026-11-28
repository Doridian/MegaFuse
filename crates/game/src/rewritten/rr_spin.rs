//! 0x137D6-0x139DC: the task `func_0137D6` makes (`func_01381E`,
//! copying to VRAM through `func_00B8C0` from `D_013838` until its
//! byte 0x16 is 3), and sprites put about a point, each at its angle
//! and a growing radius, from the records at `RECORDS` (`func_01393C`,
//! `func_0138F8`, `func_013988`).
//!
//! `D_013838` is 8 bytes of `func_00B8C0`'s data followed by code
//! never disassembled (it makes tasks of `func_01393C`, the only
//! caller of `func_0138F8`, `func_0138DE` and `func_013988`); no
//! script runs that code, so those four are reviewed by reading.
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read.

use crate::{Game, Regs};

/// A task's fields as `func_00B8C0` reads them (set by
/// `func_0137D6`): its data (a long at 0xC), two words at 0x10 and
/// 0x12, bytes 0x14 to 0x18 (0x16 counted up to `COPY_DONE`).
const TASK_DATA: u32 = 0x0C;
const TASK_10: u32 = 0x10;
const TASK_12: u32 = 0x12;
const TASK_14: u32 = 0x14;
const TASK_15: u32 = 0x15;
const TASK_16: u32 = 0x16;
const TASK_17: u32 = 0x17;
const TASK_18: u32 = 0x18;
const COPY_DONE: u8 = 3;
/// A task's x's and y's low bytes (the point the sprites spin about).
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y_LOW: u32 = 0x11;
/// A sprite record's frame and position (words; `sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;
/// The records (8 bytes each): a sprite record's index (negative: the
/// end), angles (by the frame's parity, it seems: only the one at 1 +
/// d1's low word is read), the growth (a word) and the radius (a
/// word, its high byte the radius).
const RECORDS: u32 = 0xFFFF_A1FA;
const RECORD_SIZE: u32 = 8;
const RECORD_SPRITE: u32 = 0x00;
const RECORD_ANGLES: u32 = 0x01;
const RECORD_GROWTH: u32 = 0x04;
const RECORD_RADIUS: u32 = 0x06;
/// The next record's number (stepped by 7 up to `RECORDS_END`).
const RECORD_NUMBER: u32 = 0xFF9ED3;
const RECORD_STEP: u8 = 7;
const RECORDS_END: u8 = 0xB0;
/// The record's angle (`func_017780`'s).
const ANGLE: u32 = 0xFF9ED5;
/// The point (low bytes of words).
const CENTER_X: u32 = 0xFF9EFB;
const CENTER_Y: u32 = 0xFF9EFF;
/// The velocity's y fraction (`a_motion.rs`).
const VELOCITY_Y_FRACTION: u32 = 0xFF9EC7;
/// The running object (`a_objects.rs`) and the sprite record of the
/// routine being run (index, address).
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;

const ROUTINE_01381E: u32 = 0x01381E;
const DATA_013838: u32 = 0x013838;
const TASK_PRIORITY: u8 = 0x20;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_0137D6`: a task of `func_01381E` (priority 0x20) with the
    /// data `D_013838`, words 0x4000 and 0x9F, bytes 0x14-0x18 0, 1, 0,
    /// 4, 1.  Gives back d6 (the task's index) and a4 (the running
    /// object).
    pub fn r_0137d6(&mut self, d4: u32, d5: u32, d6: u32) -> (u32, u32) {
        let (d6, a6, _, _) = self.r_0003e8(d4, d5, d6, TASK_PRIORITY, ROUTINE_01381E);
        self.write16(a6 + TASK_10, 0x4000);
        self.write16(a6 + TASK_12, 0x9F);
        self.write32(a6 + TASK_DATA, DATA_013838);
        self.write8(a6 + TASK_15, 1);
        self.write8(a6 + TASK_17, 4);
        self.write8(a6 + TASK_16, 0);
        self.write8(a6 + TASK_14, 0);
        self.write8(a6 + TASK_18, 1);
        (d6, self.read32(RUNNING_ADDR))
    }

    /// `func_01381E`, a task's routine: `func_00B8C0` until its byte
    /// 0x16 is `COPY_DONE`, then the task ends (`func_000560`).
    pub fn r_01381e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let step = self.read8(r.a6 + TASK_16);
        r.d0 = low_byte(r.d0, step);
        r.c = step < COPY_DONE;
        if step == COPY_DONE {
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
        } else {
            self.r_00b8c0(r);
        }
    }

    /// `func_0138DE`: a2 = record `d2` (its low byte) of `RECORDS`; d2
    /// is kept.  N, Z, X and C as the add leaves them.
    /// No script calls it: reviewed by reading.
    pub fn r_0138de(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (address, carry) = (r.d2 & 0xFF).wrapping_mul(RECORD_SIZE).overflowing_add(RECORDS);
        r.a2 = address;
        r.n = (address as i32) < 0;
        r.z = address == 0;
        (r.x, r.c) = (carry, carry);
    }

    /// `func_0138F8` as `rr_next_record` does it, but N and Z.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0138f8(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, bool) {
        let (d5, a2, a5, c, _) = self.rr_next_record(Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, ..Regs::default() });
        (d5, a2, a5, c)
    }

    /// `func_0138F8`: the next record (`RECORD_NUMBER`, 7 on): its
    /// sprite record made the routine's and `ANGLE` its byte at 1 +
    /// d1's low word (the frame's parity is worked out into d0's low
    /// word but d1 is read: a slip in the original; d0 isn't given
    /// back, its low word is overwritten by `func_013988`).  C set at `RECORDS_END` or
    /// on a record whose index is negative (then N is set, else Z).
    /// Gives back d5, a2 (the record), a5, C, and whether N ends it.
    fn rr_next_record(&mut self, regs: Regs) -> (u32, u32, u32, bool, bool) {
        let mut r = regs;
        let number = self.read8(RECORD_NUMBER);
        if number == RECORDS_END {
            return (r.d5, r.a2, r.a5, true, false);
        }
        self.write8(RECORD_NUMBER, number.wrapping_add(RECORD_STEP));
        r.d2 = low_byte(r.d2, number);
        self.r_0138de(&mut r);
        let index = self.read8(r.a2 + RECORD_SPRITE);
        let d5 = low_byte(r.d5, index);
        if (index as i8) < 0 {
            return (d5, r.a2, r.a5, true, true);
        }
        let (d5, a5, _) = self.r_000b5a(d5);
        self.write8(SLOT_INDEX, d5 as u8);
        self.write32(SLOT_ADDR, a5);
        let at = r.a2.wrapping_add(RECORD_ANGLES).wrapping_add(r.d1 as u16 as i16 as u32);
        let angle = self.read8(at);
        self.write8(ANGLE, angle);
        (d5, r.a2, a5, false, false)
    }

    /// `func_01393C`, a task's routine (unless the objects are stopped,
    /// `func_001BB4`): it rises (`func_001AA2`), the routine's sprite
    /// is put at its position's low bytes, which become the point,
    /// with frame 0x18; then each record's sprite put about the point
    /// (`func_013988`) from the first to the end.
    /// No script calls it: reviewed by reading.
    pub fn r_01393c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        self.write8(VELOCITY_Y_FRACTION, 0xFF);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        self.write8(r.a5 + SPRITE_X, 0);
        self.write8(r.a5 + SPRITE_Y, 0);
        let x = self.read8(r.a6 + TASK_X_LOW);
        r.d0 = low_byte(r.d0, x);
        self.write8(r.a5 + SPRITE_X_LOW, x);
        self.write8(CENTER_X, x);
        let y = self.read8(r.a6 + TASK_Y_LOW);
        r.d0 = low_byte(r.d0, y);
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        self.write8(CENTER_Y, y);
        self.write8(r.a5 + SPRITE_FRAME, 0x18);
        self.write8(RECORD_NUMBER, 0);
        loop {
            let negative;
            (r.d5, r.a2, r.a5, r.c, negative) = self.rr_next_record(*r);
            if r.c {
                (r.n, r.z) = (negative, !negative);
                return;
            }
            (r.d0, r.d1, r.d5, r.a5) = self.r_013988(r.d0, r.d1, r.d3, r.d5, r.a2, r.a5);
        }
    }

    /// `func_013988`: the record's radius grown, and the routine's
    /// sprite put at the point plus that radius times the sine
    /// (`func_017780`) and cosine (`func_01777C`) of `ANGLE`, frame
    /// 0xDD.  Gives back d0, d1, d5 and a5 (the routine's sprite
    /// record).
    /// No script calls it: reviewed by reading.
    pub fn r_013988(&mut self, d0: u32, d1: u32, d3: u32, d5: u32, a2: u32, a5: u32) -> (u32, u32, u32, u32) {
        self.write8(a5 + SPRITE_FRAME, 0xDD);
        let growth = self.read16(a2 + RECORD_GROWTH);
        let d0 = low_word(d0, growth);
        let radius = self.read16(a2 + RECORD_RADIUS).wrapping_add(growth);
        self.write16(a2 + RECORD_RADIUS, radius);
        let d1 = low_byte(d1, self.read8(a2 + RECORD_RADIUS));
        let angle = self.read8(ANGLE);
        let d0 = self.r_017780(low_byte(d0, angle), d1, d3);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        let center_x = self.read8(CENTER_X);
        let d1 = low_word(d1, u16::from(center_x));
        let x = (d0 as u16).wrapping_add(u16::from(center_x));
        let d0 = low_word(d0, x);
        self.write16(a5 + SPRITE_X, x);
        let d1 = low_byte(d1, self.read8(a2 + RECORD_RADIUS));
        let angle = self.read8(ANGLE);
        let d0 = self.r_01777c(low_byte(d0, angle), d1, d3);
        let center_y = self.read8(CENTER_Y);
        let d1 = low_word(d1, u16::from(center_y));
        let y = (d0 as u16).wrapping_add(u16::from(center_y));
        self.write16(a5 + SPRITE_Y, y);
        (low_word(d0, y), d1, d5, a5)
    }
}
