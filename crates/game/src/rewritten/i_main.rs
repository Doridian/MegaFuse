//! Around the main loop (`loc_009368`, left translated): what it is
//! set up with (`func_009310`), the step it runs each pass before the
//! scheduler (`func_0093BA`, through the routine at `STEP_ROUTINE`),
//! and the first task (`func_009248`) that sets the screen up and
//! waits for a button or 0xF0 frames.

use crate::{Game, Regs};

/// Cleared by `func_009310` (bytes); `FRAMES` the main loop counts its
/// passes in.
const FRAMES: u32 = 0xFFFF_8A56;
const CLEARED_D5: u32 = 0xFFFF_80D5;
const CLEARED_D6: u32 = 0xFFFF_80D6;
const CLEARED_D7: u32 = 0xFFFF_80D7;
/// Two longs set to `D_00ABA4`.
const POINTER_1: u32 = 0xFFFF_A406;
const POINTER_2: u32 = 0xFFFF_A40A;
const D_00ABA4: u32 = 0x00ABA4;
/// A routine (a long), set to the `rts` at 0x93B8.
const ROUTINE_DA: u32 = 0xFFFF_80DA;
const RTS_93B8: u32 = 0x0093B8;
/// The step's routine (a long; `func_0093F0` calls it) and the one it
/// starts as (`func_0093F6`: none), and what the step gave back (a
/// byte: not 0, the main loop doesn't run the scheduler).
const STEP_ROUTINE: u32 = 0xFFFF_80E0;
const NO_STEP: u32 = 0x0093F6;
const STEP_RESULT: u32 = 0xFFFF_80E4;
/// Whether the step runs (bytes): `STOPPED` (the objects stopped,
/// `a_objects.rs`) not set and `RUNNING` set.
const STOPPED: u32 = 0xFFFF_A3FC;
const RUNNING: u32 = 0xFFFF_80DF;
/// The task whose record and sprite the step runs with (its index).
const STEP_TASK: u32 = 0xFFFF_80E5;
/// A task's sprite record's index (a byte).
const TASK_SPRITE: u32 = 0x03;
/// Cleared by `func_009310` and `func_009248` (bytes).
const CLEARED_45: u32 = 0xFFFF_A945;
const CLEARED_44: u32 = 0xFFFF_A944;
/// The head task's routine `func_009310` sets.
const HEAD_ROUTINE: u32 = 0x00821C;

/// `func_009248`'s: a long set to 0xFE00; words cleared.
const LONG_8A5E: u32 = 0xFFFF_8A5E;
const CLEARED_D002: u32 = 0xFFFF_D002;
const CLEARED_D402: u32 = 0xFFFF_D402;
/// The view (`a_motion.rs`): a long and a word cleared.
const VIEW: u32 = 0xFFFF_80E6;
const VIEW_Y_MORE: u32 = 0xFFFF_80EA;
/// The list of things to load (`i_load.rs`) and the first task's.
const LIST: u32 = 0xFFFF_92FA;
const FIRST_LIST: u32 = 0x009300;
/// Set by `func_009248` (bytes): 0xFF, 0, 0 and `STOPPED` 0.
const SET_F9: u32 = 0xFFFF_A3F9;
const CLEARED_FA: u32 = 0xFFFF_A3FA;
const CLEARED_FB: u32 = 0xFFFF_A3FB;
/// The first task's wait: the frames waited (a word), the pad's byte
/// whose bit 7 ends it, and the most frames.
const WAITED: u32 = 0xFFFF_AED0;
const PAD: u32 = 0xFFFF_AD02;
const MOST: u16 = 0xF0;
/// `func_00A63A`'s and `func_00A51A`'s arguments.
const ON_START: u32 = 5;
const ON_END: u32 = 4;
/// Where the task goes on after each yield, and the routine it is
/// last (`func_009800`), which then keeps `COUNTED` going.
const AFTER_SET_UP: u32 = 0x0092BE;
const AFTER_WAIT: u32 = 0x0092E8;
const LAST: u32 = 0x009800;
const COUNTING: u32 = 0x00981C;
const COUNTED: u32 = 0xFFFF_A94C;

/// The VDP's control port, and two registers as the game keeps them
/// (`vdp.rs`): 12 (the mode: bits 0 and 7 the width) and 16 (the
/// planes' size).
const CONTROL: u32 = 0xC00004;
const REG12: u32 = 0xFFFF_F01C;
const REG12_LOW: u32 = 0xFFFF_F01D;
const REG16: u32 = 0xFFFF_F024;
const REG16_LOW: u32 = 0xFFFF_F025;
/// 64 by 64 cells.
const PLANES_64: u8 = 0x11;
/// Register 12's bits cleared: 32 cells wide.
const NARROW: u8 = 0x7E;

impl Game {
    /// `func_009310`: what the main loop starts with: its counts, the
    /// pointers and routines (`STEP_ROUTINE` `NO_STEP`), the tasks
    /// anew (`func_000372`) with the head's routine `HEAD_ROUTINE`, the
    /// random numbers seeded (`func_001C16`) and the sprites anew
    /// (`func_000704`).
    #[allow(clippy::too_many_arguments)]
    pub fn r_009310(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) {
        for at in [FRAMES, CLEARED_D5, CLEARED_D6, CLEARED_D7] {
            self.write8(at, 0);
        }
        self.write32(POINTER_1, D_00ABA4);
        self.write32(POINTER_2, D_00ABA4);
        self.write32(ROUTINE_DA, RTS_93B8);
        self.write32(STEP_ROUTINE, NO_STEP);
        for at in [STEP_RESULT, RUNNING, CLEARED_45, CLEARED_44] {
            self.write8(at, 0);
        }
        let (d0, ..) = self.r_000372(d2, d3, d4, d5, d7, a1, a2, a3, a4, a5);
        self.r_0004fe(HEAD_ROUTINE);
        self.r_001c16();
        self.r_000704(d0);
    }

    /// `func_0093B8`: an `rts`.
    pub fn r_0093b8(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `func_0093BA`, the main loop's step: unless `STOPPED`, and only
    /// while `RUNNING`, with `STEP_TASK`'s record (a6) and its sprite
    /// record (d5, a5), `func_0102E8` and the step's routine
    /// (`func_0093F0`), whose d0 is kept in `STEP_RESULT`.  Gives back
    /// the registers the routine leaves.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0093ba(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        if self.read8(STOPPED) != 0 || self.read8(RUNNING) == 0 {
            return (d0, d1, d2, d3, d4, d6, d7, a1, a2, a3, a4, a6);
        }
        let task = (d6 & 0xFFFF_FF00) | u32::from(self.read8(STEP_TASK));
        let (d6, a6, _) = self.r_0005f8(task);
        let sprite = (d5 & 0xFFFF_FF00) | u32::from(self.read8(a6 + TASK_SPRITE));
        let (d5, a5, _) = self.r_000b5a(sprite);
        let d0 = self.r_0102e8(d0);
        let out = self.r_0093f0(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6);
        if self.restarted.is_none() {
            self.write8(STEP_RESULT, out.0 as u8);
        }
        out
    }

    /// `func_0093F0`: the routine at `STEP_ROUTINE`.  Gives back the
    /// registers it leaves.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0093f0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let routine = self.read32(STEP_ROUTINE);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0: routine, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        self.call_family_24(routine, &mut r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a6)
    }

    /// `func_0093F6`, the step that does nothing: d0 0 (no script runs
    /// it as the step).
    pub fn r_0093f6(&mut self, regs: &mut Regs) {
        regs.d0 = 0;
        regs.unwound = false;
    }

    /// `func_0093FA`: an `rte` (no script reaches it).
    pub fn r_0093fa(&mut self) {}

    /// `func_009248`, the first task: the flags cleared, the tasks
    /// anew but this one (`func_0003DA`), the sprites (`func_000704`),
    /// the planes (`func_009822`), the screen set up (`func_0002E6`,
    /// `func_008624`, `func_008664`, `func_0002DA`), the view cleared,
    /// `FIRST_LIST` loaded (`func_0094D8`), `func_00A63A` with
    /// `ON_START`; then it yields to `func_0092BE`.
    pub fn r_009248(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.write8(CLEARED_45, 0);
        self.write8(RUNNING, 0);
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.write32(LONG_8A5E, 0xFE00);
        (r.d0, r.d1, r.a0, r.x) = self.r_000704(r.d0);
        r.c = self.r_009822();
        self.r_0002e6();
        self.r_008624();
        self.write16(CLEARED_D002, 0);
        self.write16(CLEARED_D402, 0);
        r.unwound = false;
        self.r_008664(r);
        self.r_0002da();
        self.write32(VIEW, 0);
        self.write16(VIEW_Y_MORE, 0);
        self.write32(LIST, FIRST_LIST);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        r.unwound = false;
        if self.restarted.is_some() {
            return;
        }
        self.write8(SET_F9, 0xFF);
        self.write8(CLEARED_FA, 0);
        self.write8(CLEARED_FB, 0);
        self.write8(STOPPED, 0);
        (r.d0, r.a1, r.x, r.c) = self.r_00a63a(ON_START, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        self.write16(WAITED, 0);
        self.i_yield(r, AFTER_SET_UP, Game::r_0092be);
    }

    /// The yield (`func_000522`) to `resume`; if it doesn't return
    /// from the routine (it always does), `then` runs.
    fn i_yield(&mut self, r: &mut Regs, resume: u32, then: fn(&mut Game, &mut Regs)) {
        let unwound;
        (r.x, r.c, unwound) = self.r_000522(r.d6, resume);
        if !unwound {
            then(self, r);
        }
        r.unwound = false;
    }

    /// `func_0092BE`, the first task: unless the pad's bit 7 is set or
    /// it has waited `MOST` frames (`WAITED`), it waits on; then
    /// `func_00A51A` with `ON_END` and it yields to `func_0092E8`.
    pub fn r_0092be(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        if self.read8(PAD) & 0x80 == 0 {
            let waited = self.read16(WAITED);
            let (waited, carry) = waited.overflowing_add(1);
            self.write16(WAITED, waited);
            r.x = carry;
            if waited != MOST {
                r.c = waited < MOST;
                r.unwound = false;
                return;
            }
        }
        self.write16(WAITED, 0);
        (r.d0, r.a1, r.x, r.c) = self.r_00a51a(ON_END, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        self.i_yield(r, AFTER_WAIT, Game::r_0092e8);
    }

    /// `func_0092E8`, the first task: unless the objects are stopped
    /// (`func_001BB4`), `func_008624` and `LAST` its routine.
    pub fn r_0092e8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if !stopped {
            self.r_008624();
            (r.x, r.c) = self.r_00053e(r.d6, LAST);
        }
        r.unwound = false;
    }

    /// `func_009800`, the first task's last routine: `COUNTING` its
    /// routine from the next frame, the picture off (`func_0002E0`), the
    /// planes (`func_009822`), `func_01A598`, then `COUNTED` counted
    /// (`func_00981C`).
    pub fn r_009800(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.x, r.c) = self.r_00053e(r.d6, COUNTING);
        self.r_0002e0();
        r.c = self.r_009822();
        r.unwound = false;
        self.r_01a598(r);
        if self.restarted.is_some() {
            r.unwound = false;
            return;
        }
        self.r_00981c(r);
    }

    /// `func_00981C`, a task: `COUNTED` up by 1 (X and C its carry).
    pub fn r_00981c(&mut self, regs: &mut Regs) {
        let counted = self.read8(COUNTED);
        let (counted, carry) = counted.overflowing_add(1);
        self.write8(COUNTED, counted);
        regs.x = carry;
        regs.c = carry;
        regs.unwound = false;
    }

    /// `func_009822`: the planes 64 by 64 cells (register 16) and the
    /// screen 32 cells wide (register 12), to the VDP.  Gives back C
    /// (clear).
    pub fn r_009822(&mut self) -> bool {
        self.write8(REG16_LOW, PLANES_64);
        let reg16 = self.read16(REG16);
        self.write16(CONTROL, reg16);
        let mode = self.read8(REG12_LOW);
        self.write8(REG12_LOW, mode & NARROW);
        let reg12 = self.read16(REG12);
        self.write16(CONTROL, reg12);
        false
    }
}
