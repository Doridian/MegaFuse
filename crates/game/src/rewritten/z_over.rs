//! The screen `func_003242` goes to once its count `0xFF9543` runs
//! out (0x1B816-0x1BA3C): set up by `func_01B816`; a task
//! (`func_01B89A`) makes its two sprite-tasks and shows the code
//! (`func_01BA3C`, `z_code.rs`): a mark moved between two rows by the
//! pad (`func_01B964`) until a button sets `PICKED`, then the one
//! that waits for it (`func_01B90E`) leaves for the title
//! (`func_01A598`) or another screen (`loc_01797A`) by the row.

use crate::{Game, Regs};

/// A task's fields (`task.rs`, `anim.rs`): its animation's counters
/// and its animation (a long).
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's y's low byte (`sprite.rs`).
const SPRITE_Y_LOW: u32 = 0x0B;

/// What `func_01B684` places a task with (`z_title.rs`).
const PLACE_X: u32 = 0xFFFF_80C4;
const PLACE_Y: u32 = 0xFFFF_80C8;
const PLACE_FLAGS: u32 = 0xFFFF_80CC;
const PLACE_ANIM: u32 = 0xFFFF_80D0;

/// Bytes cleared by `func_01B816`.
const CLEARED_A945: u32 = 0xFFFF_A945;
const CLEARED_80DF: u32 = 0xFFFF_80DF;
/// A long `func_01B816` and the ways out set to 0xABA4.
const LONG_A40A: u32 = 0xFFFF_A40A;
const LONG_A40A_VALUE: u32 = 0x00ABA4;
/// The list `func_0094D8` loads graphics by (a long), and this
/// screen's (ROM).
const LOAD_LIST: u32 = 0xFFFF_92FA;
const LIST_1B87E: u32 = 0x01B87E;
/// The sprites' set (`func_000776`: set 0, frames' table in the ROM).
const FRAMES: u32 = 0x0DA7CC;
/// While this isn't 0 (a fade runs, `j_colours.rs`) the mark doesn't
/// move, and the way out waits for bit 0 to clear.
const STOPPED: u32 = 0xFFFF_A3FC;
/// Bit 0 set once a button is pressed (`func_01B964`); the byte is the
/// map's first too (`map.rs`).
const PICKED: u32 = 0xFFFF_955C;
/// The pad's new presses (bits 0-1 move the mark, 5-7 pick).
const PRESSED: u32 = 0xFFFF_AD02;
const PRESS_MOVE: u8 = 0x03;
const PRESS_PICK: u8 = 0xE0;
/// The mark's row (0 or 1).
const ROW: u32 = 0xFFFF_AE89;

/// Animations (ROM).
const ANIM_DC32A: u32 = 0x0DC32A;
const ANIM_DC330: u32 = 0x0DC330;
const ANIM_DC344: u32 = 0x0DC344;

/// The routines.
const R_01B89A: u32 = 0x01B89A;
const R_01B90E: u32 = 0x01B90E;
const R_01B964: u32 = 0x01B964;
const R_01B9C4: u32 = 0x01B9C4;
const R_01B9E2: u32 = 0x01B9E2;
const R_01BA36: u32 = 0x01BA36;
const TITLE: u32 = 0x01A598;

/// The Z80's commands: this screen's music, the mark moved, a pick,
/// and a stop (with d1 0x40, `func_00349E`).
const SOUND_SCREEN: u8 = 0x0D;
const SOUND_MOVE: u8 = 0x20;
const SOUND_PICK: u8 = 0x21;
const SOUND_STOP: u8 = 0xFE;
const STOP_ARG: u32 = 0x40;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_0013D6`: the task's animation a frame on.
    fn z_over_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `func_001222` then `func_01B684`: a sprite-task running
    /// `routine` at `x`, `y`, flags `flags`, animation `anim`.
    fn z_over_sprite_task(&mut self, r: &mut Regs, depth: u8, priority: u8, routine: u32, at: (u16, u16, u16), anim: u32) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, priority, routine);
        self.write16(PLACE_X, at.0);
        self.write16(PLACE_Y, at.1);
        self.write16(PLACE_FLAGS, at.2);
        self.write32(PLACE_ANIM, anim);
        (r.d0, r.x, r.c) = self.r_01b684(r.a5, r.a6);
    }

    /// `func_00A51A`: a fade out at speed 4.
    fn z_over_fade_out(&mut self, r: &mut Regs) {
        r.d0 = 4;
        (r.d0, r.a1, r.x, r.c) = self.r_00a51a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00B4C6` (no window), its flags unused.
    fn z_over_no_window(&mut self, r: &Regs) {
        let mut copy = *r;
        self.r_00b4c6(&mut copy);
    }

    /// `func_01B816`: the screen set up: `CLEARED_A945` and
    /// `CLEARED_80DF` 0, no window, the scroll's offsets 0
    /// (`func_008624`), objects, tasks and sprites cleared
    /// (`func_001C56`, `func_0003DA`, `func_000704`), `LONG_A40A`, the
    /// sprites' set, `LIST_1B87E` loaded, `func_01C082`, a task
    /// `func_01B89A` and its music.  Gives back the registers, X and C
    /// (clear).
    pub fn r_01b816(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        self.write8(CLEARED_A945, 0);
        self.write8(CLEARED_80DF, 0);
        self.z_over_no_window(&r);
        self.r_008624();
        (r.d6, r.a6, r.x) = self.r_001c56(r.d4, r.d5, r.d6);
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d0, r.d1, r.a0, r.x) = self.r_000704(r.d0);
        self.z_over_no_window(&r);
        self.write32(LONG_A40A, LONG_A40A_VALUE);
        r.a0 = FRAMES;
        r.d0 &= 0xFFFF_0000;
        r.d0 = self.r_000776(r.d0, r.a0, r.a1);
        self.write32(LOAD_LIST, LIST_1B87E);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_none() {
            (r.d0, r.d6, r.a0, r.a1, r.a6, r.x) =
                self.r_01c082(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
            (r.d6, r.a6, r.x, _) = self.r_0003e8(r.d4, r.d5, r.d6, 0x32, R_01B89A);
            r.d0 = low_byte(r.d0, SOUND_SCREEN);
            r.c = false;
            self.r_0034b6(r.d0);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_01B89A`, a task's routine: the task ends, two sprite-tasks
    /// made (`func_01B90E` and the mark, `func_01B964`), and the code
    /// shown (`func_01BA3C`).
    pub fn r_01b89a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
        self.z_over_sprite_task(r, 0x14, 0x32, R_01B90E, (0x58, 0xB8, 0x80), ANIM_DC32A);
        self.z_over_sprite_task(r, 0x0A, 0x28, R_01B964, (0x48, 0x48, 0x80), ANIM_DC344);
        (r.d0, r.d3, r.d5, r.a0, r.a5, r.x, r.c) =
            self.r_01ba3c(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_01B90E`, a task's routine: its animation; once `PICKED`,
    /// on row 0 its routine `func_01B9C4` with the animation
    /// `ANIM_DC330`; on row 1 its routine `func_01BA36`, a fade out and
    /// the sound stopped.
    pub fn r_01b90e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_over_animate(r);
        if self.read8(PICKED) & 1 == 0 {
            return;
        }
        if self.read8(ROW) != 0 {
            self.z_over_next(r, R_01BA36);
            self.z_over_fade_out(r);
            r.d1 = STOP_ARG;
            r.d0 = low_byte(r.d0, SOUND_STOP);
            r.c = false;
            self.r_00349e(r.d0);
            return;
        }
        self.z_over_next(r, R_01B9C4);
        self.write32(r.a6 + TASK_ANIM, ANIM_DC330);
        self.write8(r.a6 + TASK_TIMER, 0);
        self.write8(r.a6 + TASK_STEP, 0);
        r.c = false;
    }

    /// `func_00053E`: `routine` the running task's routine.
    fn z_over_next(&mut self, r: &mut Regs, routine: u32) {
        (r.x, r.c) = self.r_00053e(r.d6, routine);
    }

    /// `func_01B962`: an `rts` (the end of `func_01B90E`).
    pub fn r_01b962(&mut self) {}

    /// `func_01B964`, the mark's task: its animation; unless `STOPPED`
    /// or `PICKED`, a press of bit 0 or 1 moves it to the other row
    /// (y 0x58 + 0x20 x row) with its sound; else one of bits 5-7 sets
    /// `PICKED` with the pick's sound.
    pub fn r_01b964(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_over_animate(r);
        r.c = false;
        if self.read8(STOPPED) != 0 || self.read8(PICKED) & 1 != 0 {
            return;
        }
        let moved = self.read8(PRESSED) & PRESS_MOVE;
        r.d0 = low_byte(r.d0, moved);
        if moved != 0 {
            r.d0 = low_byte(r.d0, SOUND_MOVE);
            self.r_0034b6(r.d0);
            let row = (self.read8(ROW) & 1) ^ 1;
            self.write8(ROW, row);
            let (y, carry) = (self.read8(ROW) << 5).overflowing_add(0x58);
            r.d0 = low_byte(r.d0, y);
            r.x = carry;
            self.write8(r.a5 + SPRITE_Y_LOW, y);
            r.c = false;
            return;
        }
        let picked = self.read8(PRESSED) & PRESS_PICK;
        r.d0 = low_byte(r.d0, picked);
        if picked == 0 {
            return;
        }
        self.write8(PICKED, 1);
        r.d0 = low_byte(r.d0, SOUND_PICK);
        self.r_0034b6(r.d0);
    }

    /// `func_01B9C4`, a task's routine: its animation; when it ends
    /// (C), its routine `loc_01B9E2` and a fade out.
    pub fn r_01b9c4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_over_animate(r);
        if !r.c {
            return;
        }
        self.z_over_next(r, R_01B9E2);
        self.z_over_fade_out(r);
    }

    /// `func_01B9E0`: an `rts` (the end of `func_01B9C4`).
    pub fn r_01b9e0(&mut self) {}

    /// `loc_01B9E2`, a task's routine: once the fade is over (`STOPPED`
    /// bit 0 clear), on row 1 `loc_01BA12`; on row 0 no window,
    /// `LONG_A40A`, the tasks cleared (`func_0003DA`), `func_01C258`
    /// and on to `loc_01797A`.
    pub fn r_01b9e2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(STOPPED) & 1 != 0 {
            return;
        }
        r.c = false;
        if self.read8(ROW) != 0 {
            (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) =
                self.r_01ba12(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
            return;
        }
        self.z_over_no_window(r);
        self.write32(LONG_A40A, LONG_A40A_VALUE);
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.r_01c258();
        (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) = self.r_01797a(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_01BA10`: an `rts` (`loc_01B9E2` jumps past it).
    /// No script calls it: reviewed by reading.
    pub fn r_01ba10(&mut self) {}

    /// `loc_01BA12` (row 1's way out, also `func_01C016`'s): no window,
    /// `LONG_A40A`, the tasks cleared (`func_0003DA`), and a task
    /// `func_01A598` (the title).  Gives back d0, d1, d6, a0, a6, X and
    /// C.
    pub fn r_01ba12(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, bool, bool) {
        let r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        self.z_over_no_window(&r);
        self.write32(LONG_A40A, LONG_A40A_VALUE);
        let (d0, d1, d6, a0, _) = self.r_0003da(d2, d3, d4, d5, d7, a1, a2, a3, a4, a5);
        let (d6, a6, x, c) = self.r_0003e8(d4, d5, d6, 0x1E, TITLE);
        (d0, d1, d6, a0, a6, x, c)
    }

    /// `func_01BA36`, a task's routine: its animation, then
    /// `loc_01B9E2`.
    pub fn r_01ba36(&mut self, regs: &mut Regs) {
        self.z_over_animate(regs);
        self.r_01b9e2(regs);
    }
}
