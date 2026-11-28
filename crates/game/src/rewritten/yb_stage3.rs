//! The first steps of the two tasks stage 3's object makes
//! (`y_stage3.rs`; 0x1A202-0x1A328, in exit-3-boss): each counts
//! frames in its `TASK_COUNT` and yields on to the routine that makes
//! its tasks (`func_01A228`, `func_01A328`).

use crate::rewritten::y_screen::low_byte;
use crate::{Game, Regs};

/// A task's frames to wait (`TASK_ANGLE` in `y_stage3.rs`, where the
/// tasks it makes use it as their angle).
const TASK_COUNT: u32 = 0x16;

/// Where the tasks go on.
const COUNTING_A: u32 = 0x01A20C;
const MAKING_A: u32 = 0x01A228;
const COUNTING_B: u32 = 0x01A314;
const MAKING_B: u32 = 0x01A328;

impl Game {
    /// `func_01A202`, a task: `TASK_COUNT` 0x20 and a yield to
    /// `func_01A20C`.
    pub fn r_01a202(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(r.a6 + TASK_COUNT, 0x20);
        self.yb_yield(r, COUNTING_A);
    }

    /// `func_01A20C`, each frame: `TASK_COUNT` down; at 0, 0x12 again,
    /// the Z80's command 0x5F and a yield to `func_01A228`.  Gives back
    /// X and C (the count's borrow, or the yield's).
    pub fn r_01a20c(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let (count, borrow) = self.read8(r.a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = borrow;
        r.c = borrow;
        if count != 0 {
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x12);
        r.d0 = low_byte(r.d0, 0x5F);
        self.r_0034b6(r.d0);
        self.yb_yield(r, MAKING_A);
    }

    /// `func_01A30A`, a task: `TASK_COUNT` 0x38 and a yield to
    /// `func_01A314`.
    pub fn r_01a30a(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(r.a6 + TASK_COUNT, 0x38);
        self.yb_yield(r, COUNTING_B);
    }

    /// `func_01A314`, each frame: `TASK_COUNT` down; at 0, 0x10 again
    /// and a yield to `func_01A328`.  Gives back X and C (the count's
    /// borrow, or the yield's).
    pub fn r_01a314(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let (count, borrow) = self.read8(r.a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = borrow;
        r.c = borrow;
        if count != 0 {
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x10);
        self.yb_yield(r, MAKING_B);
    }
}
