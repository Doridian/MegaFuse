//! Tasks' routines of 0xC000-0x18000 that yield or end the stage
//! (`func_00C8C8`, the end of a stage; `func_00E29E`, a blinking
//! sprite; `func_0118B4`, a sprite moved up; `func_01748E`, the
//! ending's skip), the yields between two steps of `func_011150`
//! (`func_011246`, `func_01127A`), and the tail shared by
//! `func_00CFF8` and `func_00D07C` (`loc_00D074`).

use crate::{Game, Regs};

/// Counted up when the stage ends; at 5 `func_00C8C8` goes to
/// `func_0127C2` instead.
const STAGE: u32 = 0xFFFF_953E;
const LAST_STAGE: u8 = 5;
/// Bits: bit 0 makes `func_00E29E` end its sprite; `func_00C8C8`
/// clears bits 3 and 5.
const FLAGS: u32 = 0xFFFF_9540;
/// Set to 1 by `func_00C8C8` before `func_009B0A` (`RUNNING` of
/// `i_main.rs`).
const RUNNING: u32 = 0xFFFF_80DF;
/// Cleared by `func_00C8C8`.
const BYTE_A94C: u32 = 0xFFFF_A94C;
/// The pad's byte read: bit 7 skips the ending.
const PAD: u32 = 0xFFFF_AD02;

/// A sprite record's frame and its y (a word).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_Y: u32 = 0x0A;

/// Where the tasks go on: `func_00E29E`'s blinking (`func_00E2B4`,
/// after its yield the `rts` at 0xE30A), `func_011150`'s second yields,
/// `func_0118B4`'s and `func_01748E`'s (at 0x174AE, a stop check and
/// the game started over, `func_007AAE`: a place the translation has
/// no routine for).
const BLINK: u32 = 0x00E2B4;
const AFTER_E29E: u32 = 0x00E30A;
const AFTER_011246: u32 = 0x01124A;
const AFTER_01127A: u32 = 0x01127E;
const AFTER_0118B4: u32 = 0x0118C0;
const AFTER_01748E: u32 = 0x0174AE;

impl Game {
    /// `func_00C8C8`, a task's routine, when the stage is over: unless
    /// the objects are stopped, `func_0099A2`; at stage `LAST_STAGE` the
    /// Z80's command 0xFE (d1 0x20) first and `func_0127C2` after;
    /// otherwise (`loc_009916`) the next stage: the tasks cleared
    /// (`func_0003DA`), `func_000704`, `func_001C56`, `func_0002E6`,
    /// `STAGE` counted, `func_009966`, `func_000E3E`, `func_000E9E`,
    /// `func_00644A`, `func_00ABA6`, `func_00AF26`, `RUNNING` 1,
    /// `func_009B0A`, `BYTE_A94C` cleared, bits 3 and 5 of `FLAGS`
    /// cleared and `func_009ACA`.
    pub fn r_00c8c8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.ya_stopped(r) {
            return;
        }
        if self.read8(STAGE) == LAST_STAGE {
            self.ya_command_fe(r, 0x20);
            (r.x, r.c) = self.r_0099a2(r.d4);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
                self.r_0127c2(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            return;
        }
        (r.x, r.c) = self.r_0099a2(r.d4);
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d0, r.d1, r.a0, r.x) = self.r_000704(r.d0);
        (r.d6, r.a6, r.x) = self.r_001c56(r.d4, r.d5, r.d6);
        self.r_0002e6();
        let stage = self.read8(STAGE);
        self.write8(STAGE, stage.wrapping_add(1));
        (r.d0, r.d1, r.x) = self.r_009966(r.d0, r.d1);
        (r.d0, r.d1, r.d6, r.a6, r.x) = self.r_000e3e(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_000e9e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return;
        }
        (r.d0, r.d1, r.d2, r.d3, r.a0, r.a1, r.a2, r.x) = self.r_00644a(r.d1, r.d2, r.d3);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_00aba6(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return;
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x) =
            self.r_00af26(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6);
        self.write8(RUNNING, 1);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6) =
            self.r_009b0a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        if self.restarted.is_some() {
            return;
        }
        self.write8(BYTE_A94C, 0);
        let flags = self.read8(FLAGS);
        self.write8(FLAGS, flags & 0xD7);
        (r.d6, r.a6, r.x, r.c) = self.r_009aca(r.d4, r.d5, r.d6);
    }

    /// `loc_00D074`: the sprite at a5 placed for the task at a6
    /// (`func_001B7E`) and its depth (`func_001B5C`).  Gives back d0, X
    /// and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00d074(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, bool, bool) {
        let (d0, _, _) = self.r_001b7e(a5, a6);
        self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5)
    }

    /// `func_00E29E`, a task's routine: while bit 0 of `FLAGS` is clear,
    /// its routine is `BLINK` (`func_00E2B4`, run at once); when it is
    /// set, the sprite at a5 gets frame 0, is placed (`func_00E2F2`) and
    /// the task yields (to an `rts`).
    pub fn r_00e29e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(FLAGS) & 1 == 0 {
            (r.x, r.c) = self.r_00053e(r.d6, BLINK);
            self.ya_call(r, Game::r_00e2b4);
            return;
        }
        self.write8(r.a5 + SPRITE_FRAME, 0);
        (r.d0, r.x, r.c) = self.r_00e2f2(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        self.ya_yield(r, AFTER_E29E);
    }

    /// `func_011246`, the second of two yields in a row (`func_011150`
    /// goes on at `func_01124A`).
    pub fn r_011246(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.ya_yield(r, AFTER_011246);
    }

    /// `func_01127A`, the second of two yields in a row (`func_011150`
    /// goes on at `func_01127E`).
    pub fn r_01127a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.ya_yield(r, AFTER_01127A);
    }

    /// `func_0118B4`, a task's routine: the sprite at a5 moved up 0x20
    /// (its y, d0's low word -0x20), and it yields to `func_0118C0`.
    pub fn r_0118b4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let step: u16 = 0xFFE0;
        r.d0 = (r.d0 & 0xFFFF_0000) | u32::from(step);
        let y = self.read16(r.a5 + SPRITE_Y);
        self.write16(r.a5 + SPRITE_Y, y.wrapping_add(step));
        self.ya_yield(r, AFTER_0118B4);
    }

    /// `func_01748E`, a task's routine in the ending: when the pad's
    /// bit 7 is set, the Z80's command 0xFE (d1 0x40), a fade
    /// (`func_00A51A` with 4), and it yields (`AFTER_01748E`).
    pub fn r_01748e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(PAD) & 0x80 == 0 {
            return;
        }
        self.ya_command_fe(r, 0x40);
        self.ya_fade(r, 4);
        self.ya_yield(r, AFTER_01748E);
    }
}
