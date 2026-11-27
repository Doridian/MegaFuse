//! Routines of the table at `D_00A2F4` that make a stage's tasks (as
//! `n_stage_a.rs`: each sets `NEW_FRAME`, makes its task with a sprite,
//! `func_001222`, and gives back the byte d0 its caller takes, C clear):
//! the task that rises and opens cells (`func_00E5A2`), one that steps
//! its frame round (`func_00E828`), one that clears the cells round it
//! (`func_00E8A0`), and one that waits, shows a frame cycle and makes a
//! task that hits what is in front of it (`func_00E958`).

use crate::{Game, Regs};

/// Settings bits: bit 5 lets `func_00E5E8`'s task rise.
const SETTINGS: u32 = 0xFF9540;
/// The frame `func_00CA06` gives the sprite of the task just made.
const NEW_FRAME: u32 = 0xFFAC91;
/// A count, one more for each task `func_00E958` makes: the task's
/// count (its frames to wait) starts at it.
const MADE: u32 = 0xFFACE0;
/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// The running task's index.
const CURRENT_TASK: u32 = 0xFF80EE;
/// A cell (column, row; one word).
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
/// The cell map's second byte, 0x200 on.
const SECOND: u32 = 0x200;
/// A velocity's y (a word) for `func_001AA2`.
const VELOCITY_Y: u32 = 0xFF9EC6;
/// Where `TASK_17` goes when `func_00E8E4` ends a task (`func_0060CC`).
const TIMER_OUT: u32 = 0xFFAABE;

/// A task's fields: a step (`func_00E860`), a kind (0-2,
/// `func_00E5A2`), a count of frames (a byte, or a word), a byte flag
/// or count, a byte given when made.
const TASK_STEP: u32 = 0x12;
const TASK_KIND: u32 = 0x15;
const TASK_COUNT: u32 = 0x16;
const TASK_17: u32 = 0x17;
const TASK_18: u32 = 0x18;
/// A sprite record's frame and flags.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// `func_00E576`'s frame (for d0 = 7).
const SEVEN_FRAME: u8 = 0xBE;
/// `func_00E5A2`'s task: routine, frames by caller, the next routines;
/// the byte d0 it gives back.
const RISER: u32 = 0x00E5E8;
const RISER_FRAME: u8 = 0xBF;
const RISER_FRAME_2: u8 = 0xC1;
const RISER_FRAME_1: u8 = 0xC3;
const RISER_STARTED: u32 = 0x00E61E;
const RISING: u32 = 0x00E63A;
const RISER_GIVES: u8 = 0x74;
/// The cells it opens: the cell's byte and second byte.
const OPEN: u8 = 4;
const OPEN_SECOND: u8 = 0x70;

/// `func_00E828`'s task and its first frame (its frames are this plus
/// its step, 0-2).
const STEPPER: u32 = 0x00E860;
const STEPPER_FRAME: u8 = 0xB9;

/// `func_00E8A0`'s task and the next routine; the cell steps by count
/// (`D_00E950` x, `D_00E954` y; signed bytes in the ROM).
const CLEARER: u32 = 0x00E8C6;
const CLEARING: u32 = 0x00E8E4;
const STEP_X: u32 = 0x00E950;
const STEP_Y: u32 = 0x00E954;
/// The cell kind (bit 7 aside) that it always ends on.
const ENDS_ON: u8 = 6;

/// `func_00E958`'s task (frame `WAITER_FRAME`) and its routines; the
/// frames shown while it counts (`D_00EA5E`, bytes in the ROM).
const WAITER: u32 = 0x00E9A2;
const WAITER_FRAME: u8 = 0xC8;
const SHOWING: u32 = 0x00E9C4;
const HOLDING: u32 = 0x00EA62;
const RETURNING: u32 = 0x00EA74;
const WAIT_FRAMES: u32 = 0x00EA5E;
/// The task it makes (`HITTER`), its routines and frames (`D_00EB20`,
/// bytes in the ROM, by its count and direction).
const HITTER: u32 = 0x00EA94;
const HITTER_CYCLING: u32 = 0x00EAE0;
const HITTER_ENDING: u32 = 0x00EB10;
const HITTER_FRAMES: u32 = 0x00EB20;
/// The sound when it hits something (`func_002E4E`).
const HIT_SOUND: u8 = 0x3A;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// A byte sign-extended to an offset.
fn signed(b: u8) -> u32 {
    b as i8 as i32 as u32
}

impl Game {
    /// `func_00E576`: for d0 = 7, `func_00C79C`'s task with frame
    /// `SEVEN_FRAME`, its sprite flags 4.  Gives back d0's low byte 0,
    /// C clear (and d5, d6, a5, a6, X from the task made).
    pub fn r_00e576(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        if r.d0 as u8 == 7 {
            self.write8(NEW_FRAME, SEVEN_FRAME);
            (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00c79c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            self.write8(r.a5 + SPRITE_FLAGS, 4);
        }
        r.d0 = low_byte(r.d0, 0);
        r.c = false;
        r.unwound = false;
    }

    /// `func_00E59A`: a task `RISER` of kind 0, frame `RISER_FRAME`
    /// (`func_00E5A2`).
    pub fn r_00e59a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.write8(NEW_FRAME, RISER_FRAME);
        r.d0 = low_byte(r.d0, 0);
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00e5a2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        r.unwound = false;
    }

    /// `func_00E5A2`: a task `RISER` whose kind is d0's low byte.  Gives
    /// back d0 (`RISER_GIVES` in its low byte), d5, d6, a5, a6, X and C
    /// (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00e5a2(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let (d5, d6, a5, a6, x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0xF0, 0x42, RISER);
        self.r_00ca06(d0, a5, a6);
        self.write8(a6 + TASK_KIND, d0 as u8);
        (low_byte(d0, RISER_GIVES), d5, d6, a5, a6, x, false)
    }

    /// `func_00E5D2`: two tasks `RISER`, kind 2 with frame
    /// `RISER_FRAME_2` and kind 1 with `RISER_FRAME_1`.
    pub fn r_00e5d2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        for (frame, kind) in [(RISER_FRAME_2, 2), (RISER_FRAME_1, 1)] {
            self.write8(NEW_FRAME, frame);
            (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00e5a2(kind, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
        r.unwound = false;
    }

    /// `func_00E5E6`: it returns.
    /// No script calls it: reviewed by reading.
    pub fn r_00e5e6(&mut self) {}

    /// `func_00E5E8`, `RISER`'s routine: once bit 5 of `SETTINGS` is
    /// set, a Z80 command (0x5A for a kind not 0, else 0x59) and
    /// `RISER_STARTED` from now on; the sprite is placed
    /// (`func_00E5F2`).
    pub fn r_00e5e8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(SETTINGS) & (1 << 5) == 0 {
            (r.d0, r.x, r.c) = self.r_00e5f2(r.a5, r.a6);
            return;
        }
        let command = if self.read8(r.a6 + TASK_KIND) != 0 { 0x5A } else { 0x59 };
        r.d0 = low_byte(r.d0, command);
        self.r_0034b6(r.d0);
        (r.x, r.c) = self.r_00053e(r.d6, RISER_STARTED);
        self.r_00e61e(r);
    }

    /// `loc_00E5F2`: the task's sprite placed (`func_001B7E`).  Gives
    /// back d0, X and C.
    pub fn r_00e5f2(&mut self, a5: u32, a6: u32) -> (u32, bool, bool) {
        self.r_001b7e(a5, a6)
    }

    /// `func_00E61E`, `RISER_STARTED`: `RISING` from the next frame,
    /// for 0x20 frames; the sprite is placed (`func_00E5F2`).
    pub fn r_00e61e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.r_00053e(r.d6, RISING);
        self.write8(r.a6 + TASK_COUNT, 0x20);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.x, r.c) = self.r_00e5f2(r.a5, r.a6);
        r.unwound = false;
    }

    /// `func_00E63A`, `RISING`: the task moves up a pixel a frame
    /// (`func_00E6B8`); at the end of its count it ends
    /// (`func_00121A`), the cells it opens set to `OPEN` first: for
    /// kind 0 or 1 the one 4 rows below the task's cell; for kind 1 also
    /// columns 7 and 9 of that row and of row 0 (row 0's second byte
    /// `OPEN_SECOND`).  Else the sprite is placed (`func_00E5F2`).
    pub fn r_00e63a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d5, r.a5) = self.r_00e6b8(r.d0, r.d1, r.d5, r.a6);
        let (count, borrow) = self.read8(r.a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = borrow;
        if count != 0 {
            (r.d0, r.x, r.c) = self.r_00e5f2(r.a5, r.a6);
            return;
        }
        if self.read8(r.a6 + TASK_KIND) != 2 {
            r.d0 = self.r_0015f8(r.a6);
            let row = self.read8(CELL_Y);
            self.write8(CELL_Y, row.wrapping_add(4));
            r.x = self.n_open(0);
            if self.read8(r.a6 + TASK_KIND) != 0 {
                self.write8(CELL_X, 7);
                self.n_open(0);
                self.write8(CELL_X, 9);
                self.n_open(0);
                self.write8(CELL_Y, 0);
                self.n_open(OPEN_SECOND);
                self.write8(CELL_X, 7);
                r.x = self.n_open(OPEN_SECOND);
            }
        }
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
    }

    /// The cell at `CELL_X`, `CELL_Y` set to `OPEN`, its second byte
    /// `second` (`func_00167A`'s X given back).
    fn n_open(&mut self, second: u8) -> bool {
        let (at, x) = self.r_00167a();
        self.write8(at, OPEN);
        self.write8(at + SECOND, second);
        x
    }

    /// `func_00E6B8`: the task moved a pixel up (`func_001AA2`), d5 and
    /// a5 its sprite record again.  Gives back d0, d1, d5, a5.
    pub fn r_00e6b8(&mut self, d0: u32, d1: u32, d5: u32, a6: u32) -> (u32, u32, u32, u32) {
        self.write16(VELOCITY_Y, 0xFF00);
        let (d0, d1, _, _, _) = self.r_001aa2(d0, d1, a6);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        (d0, d1, d5, a5)
    }

    /// `func_00E828`: a task `STEPPER` whose `TASK_18` is d0 less 8 (0:
    /// it steps up, else down).  Gives back d0 (that, its low byte 0),
    /// d5, d6, a5, a6, X, C clear.
    pub fn r_00e828(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let kept = low_byte(r.d0, (r.d0 as u8).wrapping_sub(8));
        self.write8(NEW_FRAME, STEPPER_FRAME);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(kept, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 0x80, 0x42, STEPPER);
        self.r_00ca06(kept, r.a5, r.a6);
        self.write8(r.a6 + TASK_18, kept as u8);
        r.d0 = low_byte(kept, 0);
        r.c = false;
        r.unwound = false;
    }

    /// `func_00E860`, `STEPPER`'s routine: the sprite placed
    /// (`func_001B7E`, `func_001B5C`), its step one on (or back, for
    /// `TASK_18` not 0) round 0-2, its frame `STEPPER_FRAME` plus it.
    pub fn r_00e860(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let (d0, _, _) = self.r_001b7e(r.a5, r.a6);
        let (d0, _, _) = self.r_001b5c(d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        let by: u8 = if self.read8(r.a6 + TASK_18) != 0 { 0xFF } else { 1 };
        r.d2 = low_byte(r.d2, by);
        let step = self.read8(r.a6 + TASK_STEP).wrapping_add(by);
        r.d0 = if (step as i8) < 0 {
            2
        } else if step == 3 {
            low_byte(d0, 0)
        } else {
            low_byte(d0, step)
        };
        let step = r.d0 as u8;
        self.write8(r.a6 + TASK_STEP, step);
        let (frame, carry) = step.overflowing_add(STEPPER_FRAME);
        r.d0 = low_byte(r.d0, frame);
        r.x = carry;
        self.write8(r.a5 + SPRITE_FRAME, frame);
        r.c = false;
        r.unwound = false;
    }

    /// `func_00E8A0`: the sound 0x83 (`func_000FBE`), a task `CLEARER`
    /// (`func_0003E8`) at `POSITION` (`func_00152A`).  Gives back
    /// d0 = 6, d6, a6, X, C clear.
    pub fn r_00e8a0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.d0, _, _) = self.r_000fbe(low_byte(r.d0, 0x83));
        (r.d6, r.a6, r.x, _) = self.r_0003e8(r.d4, r.d5, r.d6, 0x42, CLEARER);
        self.r_00152a(r.a6);
        r.d0 = low_byte(r.d0, 6);
        r.c = false;
        r.unwound = false;
    }

    /// `func_00E8C6`, `CLEARER`'s routine: unless the objects are
    /// stopped (`func_001BB4`), its cell cleared (both bytes) and
    /// `CLEARING` from now on.
    pub fn r_00e8c6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (c, stopped) = self.r_001bb4();
        r.c = c;
        if stopped {
            return;
        }
        r.d0 = self.r_0015f8(r.a6);
        let at;
        (at, _) = self.r_00167a();
        r.a0 = at;
        self.write8(at, 0);
        self.write8(at + SECOND, 0);
        (r.x, r.c) = self.r_00053e(r.d6, CLEARING);
        self.r_00e8e4(r);
    }

    /// `func_00E8E4`, `CLEARING`: each frame it looks at the next of the
    /// four cells round its own (`STEP_X`, `STEP_Y` by its count): a
    /// cell holding `ENDS_ON` (bit 7 aside), or 4 for a cell to the
    /// side (count odd) or 5 for one above or below, is cleared
    /// (`func_001984`, `func_001278`, `func_0010D6`), the task ended
    /// (`func_000564`) and the cell's task's end done (`func_0060CC`,
    /// `TIMER_OUT` 5).
    pub fn r_00e8e4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = self.r_0015f8(r.a6);
        let count = self.read8(r.a6 + TASK_COUNT).wrapping_add(1);
        self.write8(r.a6 + TASK_COUNT, count);
        let way = count & 3;
        r.d1 = low_word(r.d1, u16::from(way));
        let kind = (way & 1) | 4;
        r.d2 = low_byte(low_word(r.d2, u16::from(way)), kind);
        let dx = self.read8(STEP_X + u32::from(way));
        let column = self.read8(CELL_X);
        self.write8(CELL_X, column.wrapping_add(dx));
        let dy = self.read8(STEP_Y + u32::from(way));
        let row = self.read8(CELL_Y);
        self.write8(CELL_Y, row.wrapping_add(dy));
        (r.a0, r.x) = self.r_0016ac();
        let held = self.read8(r.a0);
        r.d0 = low_byte(r.d0, held);
        r.c = false;
        if held == 0 {
            return;
        }
        let held = held & 0x7F;
        r.d0 = low_byte(r.d0, held);
        if held != ENDS_ON {
            r.c = held < kind;
            if held != kind {
                return;
            }
        }
        r.d0 = self.r_0015f8(r.a6);
        self.r_00167a();
        r.a0 = self.r_001984();
        self.r_001278();
        r.x = self.r_0010d6(r.d0);
        (r.x, _) = self.r_000564(r.d5, r.d6, r.x);
        self.write8(TIMER_OUT, 5);
        (r.d0, r.d1, r.d2, r.d5, r.d6, r.a0, r.a5, r.a6, r.x, r.c) = self.r_0060cc(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, false);
    }

    /// `func_00E958`: a task `WAITER` whose `TASK_18` is (d0 less 2)
    /// times 4 (the direction of the task it makes, in bits 2-3), its
    /// count `MADE` once one more.  Gives back d0 (that, its low byte
    /// 0), d5, d6, a5, a6, X, C clear.
    pub fn r_00e958(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let kept = low_byte(r.d0, (r.d0 as u8).wrapping_sub(2).wrapping_mul(4));
        self.write8(NEW_FRAME, WAITER_FRAME);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(kept, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 0x80, 0x43, WAITER);
        self.r_00ca06(kept, r.a5, r.a6);
        self.write8(r.a6 + TASK_17, 0);
        let (made, carry) = self.read8(MADE).overflowing_add(1);
        self.write8(MADE, made);
        r.x = carry;
        self.write8(r.a6 + TASK_COUNT, made);
        self.write8(r.a6 + TASK_18, kept as u8);
        r.d0 = low_byte(kept, 0);
        r.c = false;
        r.unwound = false;
    }

    /// `func_00E9A2`, `WAITER`: its count (a word) runs down; at its end,
    /// unless `TASK_17` is set, 0x10 frames of `SHOWING`.  The sprite is
    /// drawn (`loc_00EA46`).
    pub fn r_00e9a2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read16(r.a6 + TASK_COUNT);
        self.write16(r.a6 + TASK_COUNT, count.wrapping_sub(1));
        r.x = count < 1;
        if count != 1 || self.read8(r.a6 + TASK_17) != 0 {
            (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00ea46(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3);
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x10);
        (r.x, r.c) = self.r_00053e(r.d6, SHOWING);
        self.r_00e9c4(r);
    }

    /// `func_00E9C4`, `SHOWING`: its frame by count (`loc_00EA30`); at
    /// the end, `HOLDING` for 0x20 frames, the sound 0x5B, and a task
    /// `HITTER` made at its cell (frame 0, sprite flags 0x44, in the
    /// middle of the cell, `TASK_18` its direction).  The sprite is
    /// drawn (`loc_00EA46`).
    pub fn r_00e9c4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (count, borrow) = self.read8(r.a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = borrow;
        if count != 0 {
            (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00ea30(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a6);
            return;
        }
        self.r_00053e(r.d6, HOLDING);
        self.write8(r.a6 + TASK_COUNT, 0x20);
        let direction = low_byte(r.d0, self.read8(r.a6 + TASK_18));
        self.r_0013c0(low_byte(direction, 0x5B), r.a5);
        r.d0 = self.r_0015f8(r.a6);
        self.write8(NEW_FRAME, 0);
        let failed;
        (r.d5, r.d6, r.a5, r.a6, r.x, failed) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 0x10, 0x43, HITTER);
        r.d0 = direction;
        if !failed {
            self.write8(r.a6 + TASK_18, direction as u8);
            (r.d0, _) = self.r_00ca06(r.d0, r.a5, r.a6);
            (r.d0, r.x) = self.r_001550(r.d0, r.a6);
            self.write8(r.a6 + TASK_COUNT, 0);
            self.write8(r.a5 + SPRITE_FLAGS, 0x44);
        }
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00ea46(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3);
    }

    /// `loc_00EA30`: the sprite's frame `WAIT_FRAMES` by the count over
    /// 4; drawn (`loc_00EA46`).  Gives back d0, d5, d6, a5, a6, X, C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00ea30(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a6: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let index = self.read8(a6 + TASK_COUNT) >> 2;
        let a5 = self.read32(SLOT_ADDR);
        let frame = self.read8(WAIT_FRAMES + u32::from(index));
        self.write8(a5 + SPRITE_FRAME, frame);
        self.r_00ea46(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3)
    }

    /// `loc_00EA46`: the running task (`func_0005F8`) and its sprite
    /// record again, the sprite placed (`func_001B7E`) and its depth
    /// from its y (`func_001B5C`).  Gives back d0, d5, d6, a5, a6, X, C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00ea46(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let current = self.read8(CURRENT_TASK);
        let (d6, a6, _) = self.r_0005f8(low_byte(d6, current));
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        let (d0, _, _) = self.r_001b7e(a5, a6);
        let (d0, x, c) = self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5);
        (d0, d5, d6, a5, a6, x, c)
    }

    /// `func_00EA62`, `HOLDING`: at the end of its count, `RETURNING`
    /// from now on.  The sprite is drawn (`loc_00EA46`).
    pub fn r_00ea62(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (count, borrow) = self.read8(r.a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = borrow;
        if count != 0 {
            (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00ea46(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3);
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, RETURNING);
        self.r_00ea74(r);
    }

    /// `func_00EA74`, `RETURNING`: the count up to 0x10, the frames
    /// shown back (`loc_00EA30`); then `WAITER` again from the next
    /// frame, its count 0x200 (a word).
    pub fn r_00ea74(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (count, carry) = self.read8(r.a6 + TASK_COUNT).overflowing_add(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = carry;
        if count != 0x10 {
            (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00ea30(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a6);
            return;
        }
        self.r_00053e(r.d6, WAITER);
        self.write16(r.a6 + TASK_COUNT, 0x200);
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00ea46(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3);
    }

    /// `func_00EA94`, `HITTER`'s routine: it hits what is in front
    /// (`func_00EB28`) and counts up to 0x10; then `HITTER_CYCLING`
    /// from the next frame, for 0x10 frames (`TASK_17`).  Drawn
    /// (`loc_00EABA`).
    pub fn r_00ea94(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.a0) = self.r_00eb28(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        let (count, carry) = self.read8(r.a6 + TASK_COUNT).overflowing_add(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = carry;
        if count == 0x10 {
            self.write8(r.a6 + TASK_17, 0x10);
            self.r_00053e(r.d6, HITTER_CYCLING);
        }
        (r.d0, r.d5, r.a5, r.x, r.c) = self.r_00eaba(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a6);
    }

    /// `loc_00EABA`: the sprite's frame `HITTER_FRAMES` by its count
    /// (bits 2-3) and direction (`TASK_18`), placed (`func_001B7E`)
    /// with its depth from its y (`func_001B5C`).  Gives back d0, d5,
    /// a5, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00eaba(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a6: u32) -> (u32, u32, u32, bool, bool) {
        let index = ((self.read8(a6 + TASK_COUNT) >> 2) & 3) | self.read8(a6 + TASK_18);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        let frame = self.read8(HITTER_FRAMES.wrapping_add(signed(index)));
        self.write8(a5 + SPRITE_FRAME, frame);
        let (d0, _, _) = self.r_001b7e(a5, a6);
        let (d0, x, c) = self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5);
        (d0, d5, a5, x, c)
    }

    /// `func_00EAE0`, `HITTER_CYCLING`: it hits (`func_00EB28`), its
    /// count round 8-15; after `TASK_17` frames, `HITTER_ENDING` from
    /// now on with the count 0x10.  Drawn (`loc_00EABA`).
    pub fn r_00eae0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.a0) = self.r_00eb28(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        let count = (self.read8(r.a6 + TASK_COUNT).wrapping_add(1) & 7) | 8;
        r.d0 = low_byte(r.d0, count);
        self.write8(r.a6 + TASK_COUNT, count);
        let (left, borrow) = self.read8(r.a6 + TASK_17).overflowing_sub(1);
        self.write8(r.a6 + TASK_17, left);
        r.x = borrow;
        if left != 0 {
            (r.d0, r.d5, r.a5, r.x, r.c) = self.r_00eaba(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a6);
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x10);
        (r.x, r.c) = self.r_00053e(r.d6, HITTER_ENDING);
        self.r_00eb10(r);
    }

    /// `func_00EB10`, `HITTER_ENDING`: it hits (`func_00EB28`) as its
    /// count runs down, then the task ends (`func_00121A`).  Drawn
    /// (`loc_00EABA`) till then.
    pub fn r_00eb10(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.a0) = self.r_00eb28(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        let (count, borrow) = self.read8(r.a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = borrow;
        if count != 0 {
            (r.d0, r.d5, r.a5, r.x, r.c) = self.r_00eaba(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a6);
            return;
        }
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
    }

    /// `func_00EB28`: the object in front of the task (`func_0018C4`
    /// with its direction, `TASK_18` over 4, and bit 1 from its count),
    /// if one of the first 6, is hit (`func_002E4E`) and the Z80 sent
    /// `HIT_SOUND`.  Gives back d0, d1, a0.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00eb28(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, u32) {
        let half = self.read8(a6 + TASK_COUNT).wrapping_mul(2) & 2;
        let d1 = low_byte(d1, self.read8(a6 + TASK_18) >> 2);
        let d0 = low_byte(d0 & 0xFFFF_0000, half | d1 as u8);
        let (mut d0, d1, mut a0, _, found) = self.r_0018c4(d0, d1, a0, a6);
        if found && (d0 as u8) < 6 {
            (d0, a0) = self.r_002e4e(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a5);
            d0 = low_byte(d0, HIT_SOUND);
            self.r_0034b6(d0);
        }
        (d0, d1, a0)
    }
}
