//! The steps the yielding tasks of 0x18000-0x19000 are made of (used
//! by `yb_ring_seq.rs`, `yb_choice.rs` and `yb_taken.rs`): the
//! running task's record looked up again (`func_0005F8`), a wait of
//! frames set in its `WAIT` and counted down by `func_01793A`
//! (`v_select.rs`), and the yield (`func_000522`, `task.rs`).

use crate::rewritten::y_screen::low_byte;
use crate::{Game, Regs};

/// The index of the task running (`task.rs`).
const CURRENT: u32 = 0xFFFF_80EE;

/// A task's frames to wait (a word), counted down by `func_01793A`.
pub(crate) const WAIT: u32 = 0x1A;

impl Game {
    /// `move.b (CURRENT).w,d6` and `func_0005F8`: a6 the running task's
    /// record (the routines here look it up again after each call that
    /// may have changed a6).
    pub(crate) fn yb_own(&mut self, r: &mut Regs) {
        r.d6 = low_byte(r.d6, self.read8(CURRENT));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
    }

    /// `func_01793A` with the registers: true while the running task's
    /// `WAIT` was not yet 0 (it went down, and the original returns
    /// for the task).
    pub(crate) fn yb_waiting(&mut self, r: &mut Regs) -> bool {
        let waiting;
        (r.d6, r.a6, r.x, r.c, waiting) = self.r_01793a(r.d6);
        waiting
    }

    /// `func_000522` with the registers: the running task goes on at
    /// `resume` the next time it runs, and the routine returns.
    pub(crate) fn yb_yield(&mut self, r: &mut Regs, resume: u32) {
        (r.x, r.c, _) = self.r_000522(r.d6, resume);
    }

    /// The running task's record looked up (`yb_own`), `frames` in its
    /// `WAIT`, and the yield to `resume` (which starts with
    /// `func_01793A`'s wait).
    pub(crate) fn yb_sleep(&mut self, r: &mut Regs, frames: u16, resume: u32) {
        self.yb_own(r);
        self.write16(r.a6 + WAIT, frames);
        self.yb_yield(r, resume);
    }

    /// `func_001BB4` with the registers: true when it stopped the task
    /// (returning past its caller).
    pub(crate) fn yb_stopped(&mut self, r: &mut Regs) -> bool {
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        stopped
    }

    /// `func_000796` with the registers: a sprite record at depth
    /// `d0` (its low byte; all of d0 is passed on).
    pub(crate) fn yb_new_sprite(&mut self, r: &mut Regs, d0: u32) {
        r.d0 = d0;
        (r.d5, r.a5, r.x, r.c) = self.r_000796(
            r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x,
        );
    }
}
