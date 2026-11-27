//! Tasks that drop things: `func_00CB4A` and `func_00CB7E` throw tasks
//! of `func_00CC06` (they fly in their direction, `func_00CC20`, and
//! fall, `loc_00CC5E`); the task `func_00CCB6` makes (`func_00CCEA`,
//! `func_00CD18`, `func_00CDB2`) sends tasks of `func_00CDDE` up, which
//! come down on a cell picked at random (`func_00CED8`) as
//! `func_00CE58` and hit it (`func_00CF42`), leaving `func_00CEAC`
//! behind.

use crate::{Game, Regs};

/// A task's fields: its x's low byte, its y's high byte, its
/// animation's timer and step, a count, a second count (also a
/// direction, 0-7), a byte 0x1A (a count; or the rate, a word) and
/// 0x1B, its offset (the high byte: the sprite drawn so many pixels
/// up; `a_motion.rs`) or animation (a long).
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y_HIGH: u32 = 0x10;
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_COUNT_2: u32 = 0x17;
const TASK_1A: u32 = 0x1A;
const TASK_1B: u32 = 0x1B;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's frame and its y's low byte.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_Y_LOW: u32 = 0x0B;

/// The running task's index.
const CURRENT_TASK: u32 = 0xFF80EE;
/// The sprite record of the routine being run (index, address), and
/// the byte after its index (`func_00CD18` takes that record).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
const SLOT_8A59: u32 = 0xFF8A59;
/// A byte whose bit 2 picks the frame of a falling task.
const BYTE_80D8: u32 = 0xFF80D8;
/// A velocity (words, x then y, each a byte of pixels and a byte of
/// fraction; `a_motion.rs`).
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_X_FRACTION: u32 = 0xFF9EC5;
const VELOCITY_Y: u32 = 0xFF9EC6;
const VELOCITY_Y_FRACTION: u32 = 0xFF9EC7;
/// The frame the made task's sprite gets (`func_00CA06`), and a count
/// handed from the making task to the made one.
const MADE_FRAME: u32 = 0xFFAC91;
const HANDED: u32 = 0xFFAC90;
/// The thrown tasks' pixels a frame by direction (D_00CCA6 x,
/// D_00CCAE y).
const THROWN_X: u32 = 0x00CCA6;
const THROWN_Y: u32 = 0x00CCAE;
/// The x's low byte of `func_00CD18`'s tasks by its count (D_00CDD6).
const SENT_X: u32 = 0x00CDD6;
/// The frames of `func_00CEAC` by its count's high nibble (D_00CED4).
const LEFT_FRAMES: u32 = 0x00CED4;

/// The cell (column, row) and the map's size; rows left to look
/// through (`func_00CED8`); the random numbers' state's high byte
/// (`a_random.rs`), stirred.
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
const WIDTH: u32 = 0xFF9EB2;
const HEIGHT: u32 = 0xFF9EB3;
const ROWS_LEFT: u32 = 0xFFAC8A;
const RANDOM_STATE: u32 = 0xFFA400;

/// The routines, depths and priorities of the tasks.
const ROUTINE_CB4A: u32 = 0x00CB4A;
const ROUTINE_CB7E: u32 = 0x00CB7E;
const ROUTINE_CC06: u32 = 0x00CC06;
const ROUTINE_CC20: u32 = 0x00CC20;
const ROUTINE_CC5E: u32 = 0x00CC5E;
const ROUTINE_CCEA: u32 = 0x00CCEA;
const ROUTINE_CD18: u32 = 0x00CD18;
const ROUTINE_CDB2: u32 = 0x00CDB2;
const ROUTINE_CDDE: u32 = 0x00CDDE;
const ROUTINE_CE58: u32 = 0x00CE58;
const ROUTINE_CE9C: u32 = 0x00CE9C;
const ROUTINE_CEAC: u32 = 0x00CEAC;
const PRIORITY: u8 = 0x42;
const THROWN_PRIORITY: u8 = 0xC2;
/// Animations.
const CCB6_ANIM: u32 = 0x0DA4F6;
const SENT_ANIM: u32 = 0x0DA4DC;
const LANDED_ANIM: u32 = 0x0DA5C6;
/// A falling task's fall a frame (`func_00C742`), and its rate when
/// thrown.
const FALL: u8 = 0x18;
const THROWN_RATE: u16 = 0xFDD0;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// A byte at `at` less 1; gives back the byte it was.
    fn l_take_one(&mut self, at: u32) -> u8 {
        let was = self.read8(at);
        self.write8(at, was.wrapping_sub(1));
        was
    }

    /// `func_00CBE8` on the family's registers.
    fn l_run_own(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.d6, r.a0, r.a5, r.a6, r.x, r.c) = self.r_00cbe8(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3);
    }

    /// `func_001222` on the family's registers: a task of `routine`.
    fn l_make_task(&mut self, r: &mut Regs, depth: u8, priority: u8, routine: u32) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, priority, routine);
    }

    /// `func_00CB4A`, a task's routine: each time its second count runs
    /// out its count goes down; when that goes below 0 the Z80 gets
    /// 0x4A (if its sprite is on the screen), and it goes on as
    /// `func_00CB7E` (second count 1, three to throw).  It runs as
    /// itself (`func_00CBE8`).
    /// No script calls it: reviewed by reading.
    pub fn r_00cb4a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.l_take_one(r.a6 + TASK_COUNT_2) != 1 {
            return self.l_run_own(r);
        }
        let count = self.l_take_one(r.a6 + TASK_COUNT).wrapping_sub(1);
        if (count as i8) >= 0 {
            return self.l_run_own(r);
        }
        r.d0 = low_byte(r.d0, 0x4A);
        r.c = self.r_0013c0(r.d0, r.a5);
        self.write8(r.a6 + TASK_COUNT_2, 1);
        self.write8(r.a6 + TASK_1A, 3);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_CB7E);
        self.r_00cb7e(r);
    }

    /// `func_00CB7E`, a task's routine: each 6 frames a task of
    /// `func_00CC06` is thrown from where it is (rising, `THROWN_RATE`;
    /// its direction `TASK_1B`), and after the third it goes back to
    /// `func_00CB4A` (count 4).  It runs as itself (`func_00CBE8`).
    /// No script calls it: reviewed by reading.
    pub fn r_00cb7e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.l_take_one(r.a6 + TASK_COUNT_2) != 1 {
            return self.l_run_own(r);
        }
        self.write8(r.a6 + TASK_COUNT_2, 6);
        if self.l_take_one(r.a6 + TASK_1A) == 1 {
            self.write16(r.a6 + TASK_COUNT, 0x0400);
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_CB4A);
        }
        let direction = self.read8(r.a6 + TASK_1B);
        self.write8(HANDED, direction);
        self.write8(MADE_FRAME, 0);
        self.r_00153a(r.a6);
        self.l_make_task(r, 0x10, THROWN_PRIORITY, ROUTINE_CC06);
        if !r.c {
            (r.d0, r.c) = self.r_00ca06(r.d0, r.a5, r.a6);
            self.write16(r.a6 + TASK_RATE, THROWN_RATE);
            self.write8(r.a6 + TASK_COUNT, 7);
            self.write8(r.a6 + TASK_OFFSET, 0);
            let direction = self.read8(HANDED);
            self.write8(r.a6 + TASK_COUNT_2, direction);
            r.c = false;
        }
        self.l_run_own(r);
    }

    /// `func_00CBE8` (`.L00CBE8`): the running task's own record and
    /// sprite again, its sprite placed (`func_001B7E`, `func_001B5C`)
    /// and its animation run (`func_0013D6`).  Gives back d0, d1, d2,
    /// d5, d6, a0, a5, a6, X and C.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00cbe8(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let current = self.read8(CURRENT_TASK);
        let (d6, a6, _) = self.r_0005f8(low_byte(d6, current));
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        let (d0, _, _) = self.r_001b7e(a5, a6);
        let (d0, _, _) = self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5);
        let (d0, d1, d2, d5, a0, a5, x, c) = self.r_0013d6(d0, d1, d2, d5, a6);
        (d0, d1, d2, d5, d6, a0, a5, a6, x, c)
    }

    /// `func_00CC06`, a thrown task's routine: it falls in place
    /// (`loc_00CC5E`) for 7 frames, then flies (`func_00CC20`) for 0x20.
    /// No script calls it: reviewed by reading.
    pub fn r_00cc06(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let was = self.l_take_one(r.a6 + TASK_COUNT);
        (r.x, r.c) = (was == 0, was == 0);
        if was != 1 {
            return self.r_00cc5e(r);
        }
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_CC20);
        self.write8(r.a6 + TASK_COUNT, 0x20);
        r.c = false;
        self.r_00cc20(r);
    }

    /// `func_00CC20`, a thrown task's routine: it moves a pixel a frame
    /// in its direction (`THROWN_X`, `THROWN_Y`) and falls
    /// (`loc_00CC5E`); after its count only falls.
    /// No script calls it: reviewed by reading.
    pub fn r_00cc20(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.l_take_one(r.a6 + TASK_COUNT) == 1 {
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_CC5E);
        }
        self.write8(VELOCITY_X_FRACTION, 0);
        self.write8(VELOCITY_Y_FRACTION, 0);
        let direction = self.read8(r.a6 + TASK_COUNT_2) as i8 as i16;
        r.d1 = low_word(r.d1, direction as u16);
        let x = self.read8(THROWN_X.wrapping_add(direction as u32));
        self.write8(VELOCITY_X, x);
        let y = self.read8(THROWN_Y.wrapping_add(direction as u32));
        self.write8(VELOCITY_Y, y);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        self.r_00cc5e(r);
    }

    /// `loc_00CC5E`, a thrown task's routine: it falls (`func_00C742`,
    /// `func_001ACA`) and ends when it is down (`func_00121A`); while up
    /// its frame is 0x3E or 0x3F (bit 2 of `BYTE_80D8`), its sprite
    /// placed up by its offset (`func_001B94`) and its depth from its
    /// y less that (`func_0007D2`).
    /// No script calls it: reviewed by reading.
    pub fn r_00cc5e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = low_byte(r.d0, FALL);
        r.c = false;
        self.r_00c742(r.d0, r.a6);
        let up;
        (r.d0, r.x, up) = self.r_001aca(r.d0, r.a6);
        if !up {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        let frame = if self.read8(BYTE_80D8) & 0x04 != 0 { 0x3F } else { 0x3E };
        self.write8(r.a5 + SPRITE_FRAME, frame);
        (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
        let y = self.read8(r.a5 + SPRITE_Y_LOW);
        let (less, borrow) = y.overflowing_sub(self.read8(r.a6 + TASK_OFFSET));
        r.x = borrow;
        r.d0 = low_byte(r.d0, (!less & 0xFE) | 2);
        r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
    }

    /// `func_00CCB6`: a task of `func_00CCEA` (depth 0xF0, frame 0) at
    /// the position with animation `CCB6_ANIM`.  Gives back d0 (0) and
    /// C (clear).
    pub fn r_00ccb6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(MADE_FRAME, 0);
        self.l_make_task(r, 0xF0, PRIORITY, ROUTINE_CCEA);
        (r.d0, r.c) = self.r_00ca06(r.d0, r.a5, r.a6);
        self.write32(r.a6 + TASK_ANIM, CCB6_ANIM);
        self.write8(r.a6 + TASK_COUNT, 0);
        r.d0 = low_byte(r.d0, 0);
        r.c = false;
    }

    /// `func_00CCEA`, a task's routine: when its count runs out (0x100
    /// frames the first time) it goes on as `func_00CD18` (count 0x20,
    /// three to send), its sprite placed and the Z80 getting 0x4B (if
    /// it is on the screen).
    pub fn r_00ccea(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let was = self.l_take_one(r.a6 + TASK_COUNT);
        (r.x, r.c) = (was == 0, was == 0);
        if was != 1 {
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x20);
        self.write8(r.a6 + TASK_COUNT_2, 3);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_CD18);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        r.d0 = low_byte(r.d0, 0x4B);
        r.c = self.r_0013c0(r.d0, r.a5);
    }

    /// `func_00CD18`, a task's routine: each time its count runs out
    /// (then 4 frames) a task of `func_00CDDE` is sent up from where it
    /// is (animation `SENT_ANIM`, its x's low byte from `SENT_X` by the
    /// count left, the sprite record after the routine's, count 0x80);
    /// after the last it goes on as `func_00CDB2`.  Its own frame is
    /// 0x4C, its sprite placed.
    pub fn r_00cd18(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.l_take_one(r.a6 + TASK_COUNT) == 1 {
            self.write8(r.a6 + TASK_COUNT, 4);
            let left = self.read8(r.a6 + TASK_COUNT_2);
            self.write8(HANDED, left);
            if left == 0 {
                (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_CDB2);
            }
            self.l_take_one(r.a6 + TASK_COUNT_2);
            self.r_00153a(r.a6);
            self.write8(MADE_FRAME, 0x40);
            self.l_make_task(r, 0x10, THROWN_PRIORITY, ROUTINE_CDDE);
            if !r.c {
                (r.d0, r.c) = self.r_00ca06(r.d0, r.a5, r.a6);
                self.write32(r.a6 + TASK_ANIM, SENT_ANIM);
                let left = self.read8(HANDED) as i8 as i16;
                r.d1 = low_word(r.d1, left as u16);
                let x = (self.read8(r.a6 + TASK_X_LOW) & 0xF0) | self.read8(SENT_X.wrapping_add(left as u32));
                r.d0 = low_byte(r.d0, x);
                self.write8(r.a6 + TASK_X_LOW, x);
                let other = self.read8(SLOT_8A59);
                (r.d5, r.a5, r.x) = self.r_000b5a(low_byte(r.d5, other));
                self.write8(r.a6 + TASK_COUNT, 0x80);
                (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
            }
        }
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        self.write8(r.a5 + SPRITE_FRAME, 0x4C);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_00CDB2`, a task's routine: its sprite placed and its
    /// animation run (`func_0013D6`); at its end its count is 0 (0x100
    /// frames), its frame 0 and it goes back to `func_00CCEA`.
    pub fn r_00cdb2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            self.write8(r.a6 + TASK_COUNT, 0);
            self.write8(r.a5 + SPRITE_FRAME, 0);
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_CCEA);
        }
    }

    /// `func_00CDDE`, a sent task's routine: it rises 8 pixels a frame
    /// for its count; then it is put on a cell picked at random
    /// (`func_00CED8`, `func_001550`) and goes on as `func_00CE58`
    /// (count 0x40), the Z80 getting 0x4C (if it is on the screen), a
    /// pixel up, frame 0; and a task of `func_00CEAC` (depth 0xFC,
    /// count 0x40) is made on the same cell.
    pub fn r_00cdde(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.l_take_one(r.a6 + TASK_COUNT) != 1 {
            self.write16(VELOCITY_Y, 0xF800);
            (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
            (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x40);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_CE58);
        (r.d0, r.d1, r.a0) = self.r_00ced8(r.d1);
        (r.d0, r.x) = self.r_001550(r.d0, r.a6);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        r.d0 = low_byte(r.d0, 0x4C);
        r.c = self.r_0013c0(r.d0, r.a5);
        self.l_take_one(r.a6 + TASK_Y_HIGH);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.write8(MADE_FRAME, 0);
        self.l_make_task(r, 0xFC, THROWN_PRIORITY, ROUTINE_CEAC);
        if r.c {
            return;
        }
        (r.d0, r.c) = self.r_00ca06(r.d0, r.a5, r.a6);
        (r.d0, r.x) = self.r_001550(r.d0, r.a6);
        self.write8(r.a6 + TASK_COUNT, 0x40);
        r.c = false;
    }

    /// `func_00CE58`, a sent task's routine: it comes down 4 pixels a
    /// frame, its animation run (`func_0013D6`), for its count; then
    /// the Z80 gets 0x4D (if it is on the screen), it takes
    /// `LANDED_ANIM` and goes on as `func_00CE9C`, and its cell is hit
    /// (`func_00CF42`).
    pub fn r_00ce58(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if self.l_take_one(r.a6 + TASK_COUNT) == 1 {
            r.d0 = low_byte(r.d0, 0x4D);
            r.c = self.r_0013c0(r.d0, r.a5);
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_CE9C);
            self.write32(r.a6 + TASK_ANIM, LANDED_ANIM);
            self.write8(r.a6 + TASK_TIMER, 0);
            self.write8(r.a6 + TASK_STEP, 0);
            (r.d0, r.d2, r.a0) = self.r_00cf42(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        }
        self.write16(VELOCITY_Y, 0x0400);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_00CE9C`, a landed task's routine: its animation run
    /// (`func_0013D6`) and its sprite placed; at the animation's end it
    /// ends (`func_00121A`).
    pub fn r_00ce9c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        } else {
            (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        }
    }

    /// `func_00CEAC`, a task's routine: its frame by its count's high
    /// nibble (`LEFT_FRAMES`) and its sprite placed, until the count
    /// runs out and it ends (`func_00121A`).
    pub fn r_00ceac(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let was = self.l_take_one(r.a6 + TASK_COUNT);
        (r.x, r.c) = (was == 0, was == 0);
        if was == 1 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        let left = was.wrapping_sub(1);
        let frame = self.read8(LEFT_FRAMES + u32::from(left >> 4));
        r.d0 = low_word(r.d0, u16::from(frame));
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        self.write8(r.a5 + SPRITE_FRAME, frame);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_00CED8`: a cell picked: a column at random (below the
    /// width; `func_001BE2`, masked by the width less 1) and a row from
    /// 2 to the height less 1 (`func_001BC0`), then on through the map,
    /// row by row from row 2 again after the last, to the first cell
    /// whose byte is not 0.  `CELL_X`, `CELL_Y` are left at it.  Gives
    /// back d0 (its byte), d1 and a0 (its address).
    pub fn r_00ced8(&mut self, d1: u32) -> (u32, u32, u32) {
        let height = self.read8(HEIGHT);
        self.write8(ROWS_LEFT, height);
        let d1 = low_byte(d1, self.read8(WIDTH).wrapping_sub(1));
        let (d0, d1, _) = self.r_001be2(d1);
        let column = d0 as u8 & d1 as u8;
        self.write8(CELL_X, column);
        let state = self.read8(RANDOM_STATE);
        self.write8(RANDOM_STATE, state.wrapping_add(1));
        let d0 = low_byte(d0, self.read8(HEIGHT).wrapping_sub(3));
        let d0 = self.r_001bc0(d0, d1);
        let mut d0 = low_byte(d0, (d0 as u8).wrapping_add(2));
        loop {
            self.write8(CELL_Y, d0 as u8);
            let (mut a0, _) = self.r_00167a();
            loop {
                let byte = self.read8(a0);
                d0 = low_byte(d0, byte);
                if byte != 0 {
                    return (d0, d1, a0);
                }
                a0 = a0.wrapping_add(1);
                let column = self.read8(CELL_X).wrapping_add(1);
                self.write8(CELL_X, column);
                d0 = low_byte(d0, column);
                if column != self.read8(WIDTH) {
                    continue;
                }
                self.write8(CELL_X, 0);
                // (the count of rows left isn't acted on: a `nop` when 0)
                self.l_take_one(ROWS_LEFT);
                let row = self.read8(CELL_Y).wrapping_add(1);
                self.write8(CELL_Y, row);
                d0 = low_byte(d0, row);
                if row == self.read8(HEIGHT) {
                    break;
                }
            }
            d0 = 2;
        }
    }

    /// `func_00CF42`: the cell of the task at a6 hit: a put task there
    /// (byte 1) is stopped (`func_005E0E`); otherwise a cell of 2 is
    /// made again from its source and cleared on the plane
    /// (`func_00168C`, `func_0052F0`), its record taken
    /// (`func_0066FE`), and an object there hit (`func_002D3E`).
    /// Gives back d0, d2 and a0.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00cf42(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32) {
        let d0 = self.r_0015f8(a6);
        let (mut a0, _) = self.r_00167a();
        let byte = self.read8(a0);
        let mut d0 = low_byte(d0, byte);
        if byte == 1 {
            self.r_005e0e(d2, d6, a6);
            return (d0, d2, a0);
        }
        if byte == 2 {
            (d0, a0) = self.r_00168c(d0);
            self.r_001278();
            self.r_0052f0(d0);
        }
        let (d2, x) = self.r_0066fe(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        let hit;
        (d0, hit) = self.r_00191c(a6, x);
        if hit {
            (d0, a0) = self.r_002d3e(d0, d4, d5, a0);
        }
        (d0, d2, a0)
    }
}
