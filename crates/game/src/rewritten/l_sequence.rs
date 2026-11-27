//! The tasks `func_00BF86` (a routine of D_00A0D0 and D_00A158, the
//! lists by world of D_00A04A) makes: `func_00C054` bobbing until bit 5
//! of `FLAGS` is set, then its animation (`func_00C0D6`), a count down
//! and a task of `func_00B8C0` in its place (`func_00C100`); the task it
//! makes (`func_00C15A`) waits, sends eight pieces off (`l_pieces.rs`)
//! and bobs (`func_00C1F6`) until an object of index below 5 is in the
//! cell under it; then the circle around object 0 (`l_circle.rs`), and
//! at its end the cells looked through (`func_00C4DA`), `func_00C4FC`,
//! `func_00C544` and `func_00C568`.  `func_00C028` (`func_00BF86`'s
//! other task) goes to `func_00C4FC` when an object is under it.

use crate::{Game, Regs};

/// A task's fields: its animation's timer and step, a count, a second count,
/// its rate (a word) and its offset (the high byte: the sprite drawn so
/// many pixels up; `a_motion.rs`), which is also where an animation (a
/// long) is kept.
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_COUNT_2: u32 = 0x17;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;
const TASK_ANIM: u32 = 0x20;
/// The fields `func_00C100` gives the task it makes (by their offsets):
/// a word at 0x10 and 0x12, a long at 0x0C (`B8C0_TABLE`), bytes at
/// 0x14, 0x15, 0x16, 0x17, 0x18.
const NEW_10: u32 = 0x10;
const NEW_12: u32 = 0x12;
const NEW_0C: u32 = 0x0C;
const NEW_14: u32 = 0x14;
const NEW_15: u32 = 0x15;
const NEW_16: u32 = 0x16;
const NEW_17: u32 = 0x17;
const NEW_18: u32 = 0x18;
/// A sprite record's flags.
const SPRITE_FLAGS: u32 = 0x05;

/// The running task's index.
const CURRENT_TASK: u32 = 0xFF80EE;
/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// Flags: bit 5 tested by the bobbing tasks; 0x50 set when an object
/// is under them, bit 2 (`func_00C4FC`; `func_00C4DA` goes on to it
/// while it is set), bit 3 (`func_00C544`).
const FLAGS: u32 = 0xFF9540;
const UNDER_SET: u8 = 0x50;
/// Cleared with `FLAGS`' 0x50.
const BYTE_954F: u32 = 0xFF954F;
/// Bit 0 set by `func_00C4FC`, cleared by `func_00C568`.
const SPARED: u32 = 0xFF9548;
/// Counted up by `func_00C1F6` and `func_00C4FC`.
const COUNT_A957: u32 = 0xFFA957;
/// Object 0's byte 0x11: `func_00C4FC` does nothing while it is 0.
const OBJECT_0_11: u32 = 0xFFA443;
/// Bytes `func_00C568` compares (`BYTE_953A` against `BYTE_953B`);
/// when the same, bit `BYTE_953C` of `BITS` (`BIT_TABLE`) is set, and
/// the low nibble copied to the high one.
const BYTE_953A: u32 = 0xFF953A;
const BYTE_953B: u32 = 0xFF953B;
const BYTE_953C: u32 = 0xFF953C;
const BITS: u32 = 0xFF953D;
/// Bytes 1, 2, 4, 8 (D_00C5AA).
const BIT_TABLE: u32 = 0x00C5AA;
/// The number of the piece being made (`func_00C15A`).
const PIECE_NUMBER: u32 = 0xFFAC8B;
/// The frame the made task's sprite gets (`func_00CA06`).
const MADE_FRAME: u32 = 0xFFAC91;
/// The cells `func_00C4DA` looks through: its address in the cell map
/// now and the last one (longs, set by `func_00C3E4`); and the second
/// byte it stops at.
const LOOK_AT: u32 = 0xFFAC96;
const LOOK_LAST: u32 = 0xFFAC9A;
const SECOND: u32 = 0x200;
const STOP_AT: u8 = 0x60;

/// The offsets of the bobbing (D_00C282, 16 bytes).
const BOB: u32 = 0x00C282;
/// The animation `func_00C054` sets, and the made tasks' routines,
/// depths and priorities.
const ANIM_C054: u32 = 0x0DA5D6;
const ROUTINE_C0D6: u32 = 0x00C0D6;
const ROUTINE_C100: u32 = 0x00C100;
const ROUTINE_C15A: u32 = 0x00C15A;
const ROUTINE_C1F6: u32 = 0x00C1F6;
const ROUTINE_C292: u32 = 0x00C292;
const ROUTINE_C544: u32 = 0x00C544;
const ROUTINE_C568: u32 = 0x00C568;
const ROUTINE_B8C0: u32 = 0x00B8C0;
const B8C0_TABLE: u32 = 0x00C152;
const B8C0_PRIORITY: u8 = 0x20;
const PIECE: u32 = 0x00C6AE;
const PIECES: u8 = 8;
const MADE_DEPTH: u8 = 3;
const MADE_PRIORITY: u8 = 0x42;
const PIECE_PRIORITY: u8 = 0xC2;
/// The cell under the task (direction 2), and the objects that count
/// there (indexes below this).
const DOWN: u8 = 2;
const OBJECTS_COUNTED: u8 = 6;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// A byte at `at` less 1; gives back what is left and the borrow.
    fn l_count_down(&mut self, at: u32) -> (u8, bool) {
        let was = self.read8(at);
        let left = was.wrapping_sub(1);
        self.write8(at, left);
        (left, was == 0)
    }

    /// `func_001B7E`, `func_001B5C` on the family's registers.
    fn l_place(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// The object (an index below `OBJECTS_COUNTED`) in the cell under
    /// the task (`func_0018C4`), if any.
    fn l_under(&mut self, r: &mut Regs) -> bool {
        let found;
        (r.d0, r.d1, r.a0, r.x, found) = self.r_0018c4(low_byte(r.d0, DOWN), r.d1, r.a0, r.a6);
        r.c = found;
        if !found {
            return false;
        }
        r.c = (r.d0 as u8) < OBJECTS_COUNTED;
        r.c
    }

    /// `func_00C028`, a task's routine: its sprite placed; once an
    /// object is in the cell under it, `FLAGS` gets `UNDER_SET`,
    /// `BYTE_954F` is cleared and `func_00C4FC` runs.
    pub fn r_00c028(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.l_place(r);
        if !self.l_under(r) {
            return;
        }
        let flags = self.read8(FLAGS);
        self.write8(FLAGS, flags | UNDER_SET);
        self.write8(BYTE_954F, 0);
        (r.d0, r.x, r.c) = self.r_00c4fc(r.d0, r.d6, r.a6, r.x);
    }

    /// `func_00C054`, a task's routine: it bobs (its offset from `BOB`
    /// by its count, halved) and its sprite is placed; when bit 5 of
    /// `FLAGS` is set at the bottom of a bob it takes `ANIM_C054`, its
    /// routine is `func_00C0D6` from the next frame, and a task of
    /// `func_00C15A` is made where it is (its count 0x48).
    pub fn r_00c054(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(r.a6 + TASK_COUNT).wrapping_add(1);
        self.write8(r.a6 + TASK_COUNT, count);
        let step = (count >> 3) & 0xF;
        r.a0 = BOB;
        let offset = self.read8(BOB + u32::from(step)) >> 1;
        r.d0 = low_word(r.d0, u16::from(offset));
        self.write8(r.a6 + TASK_OFFSET, offset);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b74(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        if self.read8(FLAGS) & 0x20 == 0 {
            return;
        }
        r.c = false;
        if self.read8(r.a6 + TASK_OFFSET) != 0 {
            return;
        }
        self.write32(r.a6 + TASK_ANIM, ANIM_C054);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C0D6);
        self.r_00153a(r.a6);
        self.write8(MADE_FRAME, 0xF3);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, MADE_DEPTH, MADE_PRIORITY, ROUTINE_C15A);
        (r.d0, r.c) = self.r_00ca06(r.d0, r.a5, r.a6);
        self.write8(r.a6 + TASK_COUNT, 0x48);
        r.c = false;
    }

    /// `func_00C0D6`, a task's routine: its sprite placed and its
    /// animation run (`func_0013D6`); at the animation's end its count
    /// is 0x40, its sprite's flags 0x44, and its routine `func_00C100`
    /// from the next frame.
    pub fn r_00c0d6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.l_place(r);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            self.write8(r.a6 + TASK_COUNT, 0x40);
            self.write8(r.a5 + SPRITE_FLAGS, 0x44);
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C100);
        }
    }

    /// `func_00C0FE` (`.L00C0FE`): returns.
    pub fn r_00c0fe(&mut self) {}

    /// `func_00C100`, a task's routine: its sprite placed; when its
    /// count runs out it ends (`func_00121A`) and a task of
    /// `func_00B8C0` is made (`func_0003E8`) with the fields `NEW_*`.
    pub fn r_00c100(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.l_place(r);
        let (left, borrow) = self.l_count_down(r.a6 + TASK_COUNT);
        (r.x, r.c) = (borrow, borrow);
        if left != 0 {
            return;
        }
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, B8C0_PRIORITY, ROUTINE_B8C0);
        let task = r.a6;
        self.write16(task + NEW_10, 0x2F80);
        self.write16(task + NEW_12, 0x003F);
        self.write32(task + NEW_0C, B8C0_TABLE);
        self.write8(task + NEW_15, 2);
        self.write8(task + NEW_17, 4);
        self.write8(task + NEW_16, 0);
        self.write8(task + NEW_14, 0);
        self.write8(task + NEW_18, 3);
        r.c = false;
    }

    /// `func_00C15A`, a task's routine: its sprite placed; when its
    /// count runs out it is not drawn up (offset 0), its routine is
    /// `func_00C1F6` from the next frame (timer 1, second count 0x40),
    /// the Z80 gets 0x47, and `PIECES` tasks of `PIECE` are made where
    /// it is, each with its number (`PIECE_NUMBER`) as its direction
    /// (0-7), then `func_00C766`.  It stops when a task can't be made.
    pub fn r_00c15a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.l_place(r);
        let (left, borrow) = self.l_count_down(r.a6 + TASK_COUNT);
        (r.x, r.c) = (borrow, borrow);
        if left != 0 {
            return;
        }
        self.write8(r.a6 + TASK_OFFSET, 0);
        self.write8(r.a6 + TASK_TIMER, 1);
        self.write8(r.a6 + TASK_COUNT_2, 0x40);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C1F6);
        r.d0 = low_byte(r.d0, 0x47);
        self.r_0034b6(r.d0);
        self.r_00153a(r.a6);
        let mut number = 0u8;
        loop {
            r.d0 = low_byte(r.d0, number);
            self.write8(PIECE_NUMBER, number);
            (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, MADE_DEPTH, PIECE_PRIORITY, PIECE);
            if r.c {
                return;
            }
            self.l_piece_made(r.a5, r.a6);
            let number_now = self.read8(PIECE_NUMBER);
            self.write8(r.a6 + TASK_COUNT, number_now & 7);
            number = self.read8(PIECE_NUMBER).wrapping_add(1);
            r.d0 = low_byte(r.d0, number);
            if number == PIECES {
                break;
            }
        }
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00c766(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// A piece's fields as `func_00C15A` makes it: where the position
    /// is (`func_00152A`), rising (rate 0xFC00), its sprite's flags 4,
    /// timer 1, three bounces, offset 0xE0, step 0.
    fn l_piece_made(&mut self, sprite: u32, task: u32) {
        self.r_00152a(task);
        self.write16(task + TASK_RATE, 0xFC00);
        self.write8(sprite + SPRITE_FLAGS, 4);
        self.write8(task + TASK_TIMER, 1);
        self.write8(task + TASK_COUNT_2, 3);
        self.write8(task + TASK_OFFSET, 0xE0);
        self.write8(task + TASK_STEP, 0);
    }

    /// `func_00C1F6`, a task's routine: it bobs (`BOB`, a step each 6
    /// frames); after its second count, once an object is in the cell
    /// under it, `FLAGS` is `UNDER_SET`, `BYTE_954F` cleared,
    /// `COUNT_A957` counted, its routine `func_00C292` from the next
    /// frame and the Z80 gets 0xFE; its sprite is placed.
    pub fn r_00c1f6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (left, _) = self.l_count_down(r.a6 + TASK_TIMER);
        if left == 0 {
            self.write8(r.a6 + TASK_TIMER, 6);
            let count = self.read8(r.a6 + TASK_COUNT).wrapping_add(1);
            self.write8(r.a6 + TASK_COUNT, count);
            let step = count & 0xF;
            r.d0 = low_word(r.d0, u16::from(step));
            r.a0 = BOB;
            let offset = self.read8(BOB + u32::from(step));
            self.write8(r.a6 + TASK_OFFSET, offset);
        }
        let wait = self.read8(r.a6 + TASK_COUNT_2);
        if wait != 0 {
            self.write8(r.a6 + TASK_COUNT_2, wait - 1);
        } else if self.l_under(r) {
            self.write8(FLAGS, UNDER_SET);
            self.write8(BYTE_954F, 0);
            let count = self.read8(COUNT_A957);
            self.write8(COUNT_A957, count.wrapping_add(1));
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C292);
            r.d1 = low_byte(r.d1, 0x20);
            r.d0 = low_byte(r.d0, 0xFE);
            self.r_00349e(r.d0);
            let current = self.read8(CURRENT_TASK);
            (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
        }
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_00C4DA`, a task's routine: the cells from `LOOK_AT` to
    /// `LOOK_LAST` looked through, one more each frame, for a second
    /// byte of `STOP_AT`; at the last cell, or at such a cell while bit
    /// 2 of `FLAGS` is set, `func_00C4FC`.  (It stays on a cell of
    /// `STOP_AT` while the bit is clear.)
    pub fn r_00c4da(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d1 = self.read32(LOOK_LAST);
        r.a0 = self.read32(LOOK_AT);
        loop {
            let second = self.read8(r.a0.wrapping_add(SECOND));
            r.d0 = low_byte(r.d0, second);
            if second == STOP_AT {
                r.c = false;
                if self.read8(FLAGS) & 0x04 == 0 {
                    return;
                }
                break;
            }
            if r.d1 == r.a0 {
                break;
            }
            r.a0 = r.a0.wrapping_add(1);
            self.write32(LOOK_AT, r.a0);
        }
        (r.d0, r.x, r.c) = self.r_00c4fc(r.d0, r.d6, r.a6, r.x);
    }

    /// `func_00C4FC` (`.L00C4FC`): unless `OBJECT_0_11` is 0,
    /// `COUNT_A957` counted, bit 2 of `FLAGS` and bit 0 of `SPARED`
    /// set, the task's count 0x80 and its routine `func_00C544` from
    /// the next frame; the Z80 gets 0x7F unless its byte
    /// (`func_0034CE`) is not 0.  Gives back d0, X and C (clear).
    /// No script calls it: reviewed by reading.
    pub fn r_00c4fc(&mut self, d0: u32, d6: u32, a6: u32, x: bool) -> (u32, bool, bool) {
        if self.read8(OBJECT_0_11) == 0 {
            return (d0, x, false);
        }
        let count = self.read8(COUNT_A957);
        self.write8(COUNT_A957, count.wrapping_add(1));
        let flags = self.read8(FLAGS);
        self.write8(FLAGS, flags | 0x04);
        let spared = self.read8(SPARED);
        self.write8(SPARED, spared | 0x01);
        self.write8(a6 + TASK_COUNT, 0x80);
        let (x, _) = self.r_00053e(d6, ROUTINE_C544);
        let mut d0 = self.r_0034ce(d0);
        if d0 as u8 == 0 {
            d0 = low_byte(d0, 0x7F);
            self.r_0034b6(d0);
        }
        (d0, x, false)
    }

    /// `func_00C544`, a task's routine: when its count runs out its
    /// routine is `func_00C568` from the next frame, bit 3 of `FLAGS`
    /// is set and `func_00A51A` runs with 4.
    /// No script calls it: reviewed by reading.
    pub fn r_00c544(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (left, borrow) = self.l_count_down(r.a6 + TASK_COUNT);
        (r.x, r.c) = (borrow, borrow);
        if left != 0 {
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C568);
        let flags = self.read8(FLAGS);
        self.write8(FLAGS, flags | 0x08);
        (r.d0, r.a1, r.x, r.c) = self.r_00a51a(4, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00C568`, a task's routine: unless the objects are stopped
    /// (`func_001BB4`), bit 0 of `SPARED` cleared; when `BYTE_953A` and
    /// `BYTE_953B` are the same, `BITS` gets its low nibble copied to
    /// the high one and bit `BYTE_953C` (`BIT_TABLE`) set; then
    /// `func_0099A2` and `func_01884A`.
    /// No script calls it: reviewed by reading.
    pub fn r_00c568(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (c, stop) = self.r_001bb4();
        if stop {
            r.c = c;
            return;
        }
        let spared = self.read8(SPARED);
        self.write8(SPARED, spared & 0xFE);
        let a = self.read8(BYTE_953A);
        r.d0 = low_byte(r.d0, a);
        if a == self.read8(BYTE_953B) {
            let which = self.read8(BYTE_953C);
            r.d1 = low_word(r.d1, u16::from(which));
            let bits = self.read8(BITS);
            r.a0 = BIT_TABLE;
            let bits = (bits << 4) | bits | self.read8(BIT_TABLE + u32::from(which));
            r.d0 = low_byte(r.d0, bits);
            self.write8(BITS, bits);
        }
        (r.x, r.c) = self.r_0099a2(r.d4);
        (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) = self.r_01884a(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }
}
