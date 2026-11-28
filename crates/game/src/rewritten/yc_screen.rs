//! The task of the screen at 0x18D80-0x19262 after its choice, from
//! 0x19004 on (the routines are the places it yields to), and the
//! tasks it makes:
//!
//! - `func_019004`: the objects' stop (`func_001BB4`) and the screen
//!   by `LAST` (`func_0179B0`);
//! - `func_01906E`-`func_019140`: (from 0x1906A) the colour cycles
//!   (`func_01914A`), 0x30 frames, the tasks `func_0194AA` and
//!   `func_0193BA`, 0x30 frames, `func_002E4E`, 0x130 frames, then a
//!   frame at a time while object 0 is near (`WAIT_X`, `WAIT_Y`); once
//!   it isn't, `SIDE` set (2, 8 or 4) and on to `func_019200`;
//! - `func_0191BA` (made by `func_018860`): the screen loaded
//!   (`func_018D22`), the map's cells drawn (`func_0195FC`), the colour
//!   cycles and on to `func_019200`;
//! - `func_019200`-`func_01922E`: the task `func_01966A`, a frame at a
//!   time until `SIDE` is 0, 0x30 frames, the task ends and the routine
//!   of `SCENES` by `ROW` runs;
//! - `func_019262`-`func_019390` (a task made at the end of the scenes,
//!   e.g. by `zc_scene_a.rs`'s `func_01C982`): 0x20 frames; under stage 0xF a
//!   grid of sprite tasks `func_0193AC`, the map's cells emptied, 0x80
//!   frames, the screen again (`func_018CB0`) with `func_01966A`, and
//!   once `SIDE` is 0 the screen's routine `loc_018DDC` again;
//! - `func_01966A`, `func_0196DE`: `SIDE` by object 0's place.

use crate::{Game, Regs};

/// The index of the task running (`task.rs`).
const CURRENT_TASK: u32 = 0xFFFF_80EE;
/// A task's count of frames to wait (a word, `func_01793A`).
const TASK_WAIT: u32 = 0x1A;

/// Object 0's position's low bytes (records 0x40 apart from 0xFFA432).
const OBJECT_0_X_LOW: u32 = 0xFFFF_A44B;
const OBJECT_0_Y_LOW: u32 = 0xFFFF_A44D;
/// Object 0's byte 0x32 (0xFFA464): its sign picks `func_002E4E`'s
/// argument (0 if set, 5 if not).
const OBJECT_0_32: u32 = 0xFFFF_A464;

/// Set by `func_01966A` (and the routines here) from object 0's place:
/// 8 or 4 while its x is off `PLACE_X` (left or right of it), else 2
/// or 1 while its y is off `PLACE_Y` (above or not), 1 when it is at
/// both (cleared the frame after, when that task ends).  The tasks
/// here look at it each frame and go on once it is 0.
const SIDE: u32 = 0xFFFF_954F;
const SIDE_1: u8 = 1;
const SIDE_2: u8 = 2;
const SIDE_4: u8 = 4;
const SIDE_8: u8 = 8;
/// Where `func_01966A` measures from, and how near counts as there.
const PLACE_X: u8 = 0x88;
const PLACE_Y: u8 = 0x90;
const NEAR: u8 = 2;
/// Where the task waits for object 0 (0x190EA on): its y, its x, and the
/// x that sends it back to wait.
const WAIT_Y: u8 = 0x98;
const WAIT_X: u8 = 0x90;
const WAIT_X_BACK: u8 = 0x80;
/// Set to 1 while `func_01966A` runs, cleared at its end.
const BYTE_A93C: u32 = 0xFFFF_A93C;

/// Cleared before the scene (`func_01922E`, `func_019366`).
const BYTE_ADD7: u32 = 0xFFFF_ADD7;
/// The row (`w_choice.rs`): its scene's routine in `SCENES` (longs,
/// in the ROM; `D_01924E`).
const ROW: u32 = 0xFFFF_953A;
const SCENES: u32 = 0x01924E;

/// Flags of the game (`b_control.rs`): set to 0xC0 by `func_019262`.
const MODE: u32 = 0xFFFF_9540;
/// Bit 0 set by `func_019262`.
const SPARED: u32 = 0xFFFF_9548;
/// Cleared by `func_019262`.
const BYTE_80DF: u32 = 0xFFFF_80DF;
/// The first byte of the long `func_0193AC` waits on to be 0: set to
/// 0xFF by `func_019262`.
const LONG_ADC6: u32 = 0xFFFF_ADC6;
/// The stage (`func_019298` goes on under `STAGE_END`).
const STAGE: u32 = 0xFFFF_80F0;
const STAGE_END: u8 = 0x0F;
/// A word set to 0x2E00 before `func_017828`.
const WORD_80C4: u32 = 0xFFFF_80C4;
const WORD_80C4_VALUE: u16 = 0x2E00;

/// A map cell (column, row; `anim.rs`): the loops over the map's cells
/// go through it (columns 3-0xD, rows 2-0xA, every other), and
/// `func_019298` counts the sprites' x and y in it.
const CELL_X: u32 = 0xFFFF_9F08;
const CELL_Y: u32 = 0xFFFF_9F09;
const CELL_FIRST_X: u8 = 3;
const CELL_END_X: u8 = 0x0F;
const CELL_FIRST_Y: u8 = 2;
const CELL_END_Y: u8 = 0x0C;

/// `func_019298`'s grid of tasks with a sprite (`func_001222`): their
/// routine, priority and depth, the first x and y, the step and the
/// ends; the sprites' frame and flags.
const GRID_TASK: u32 = 0x0193AC;
const GRID_PRIORITY: u8 = 0x3C;
const GRID_DEPTH: u8 = 0xF1;
const GRID_FIRST_X: u8 = 0x30;
const GRID_FIRST_Y: u8 = 0x39;
const GRID_STEP: u8 = 0x20;
const GRID_END_X: u8 = 0xDF;
const GRID_END_Y: u8 = 0xCF;
const GRID_FRAME: u8 = 0xFF;
const GRID_FLAGS: u8 = 0x84;

/// A sprite record's fields (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;

/// The tasks made (`func_0003E8`) and their priorities.
const SOUNDS_TASK: u32 = 0x0194AA;
const SQUARES_TASK: u32 = 0x0193BA;
const TASKS_PRIORITY: u8 = 0x32;
const EXIT_TASK: u32 = 0x01966A;
const EXIT_PRIORITY: u8 = 0x28;
/// The screen's routine `func_019390` goes back to (`func_00053E`).
const SCREEN_AGAIN: u32 = 0x018DDC;

/// Z80 command `func_019262` sends (`func_00349E`, d1 0x20).
const COMMAND: u8 = 0xFE;
const COMMAND_D1: u32 = 0x20;

/// The frames waited.
const WAIT_30: u16 = 0x30;
const WAIT_20: u16 = 0x20;
const WAIT_80: u16 = 0x80;
const WAIT_130: u16 = 0x130;

/// Where the task goes on (the yields' resume addresses).
const AFTER_CYCLES: u32 = 0x01908A;
const AFTER_TASKS: u32 = 0x0190C2;
const AFTER_2E4E: u32 = 0x0190EA;
const AT_Y: u32 = 0x01910E;
const AT_X: u32 = 0x019132;
const AFTER_CELLS: u32 = 0x0191F6;
const AFTER_EXIT_TASK: u32 = 0x019214;
const BEFORE_SCENE: u32 = 0x01922E;
const AFTER_COMMAND: u32 = 0x019298;
const AFTER_GRID: u32 = 0x019312;
const AFTER_EMPTIED: u32 = 0x019366;
const AFTER_SCREEN: u32 = 0x019390;
const EXIT_AFTER: u32 = 0x0196DE;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// How far `value` is from `from` (`subi.b`, and `neg.b` if negative;
/// 0x80 stays 0x80), and the X it leaves.
fn distance(value: u8, from: u8) -> (u8, bool) {
    let d = value.wrapping_sub(from);
    if (d as i8) < 0 {
        (d.wrapping_neg(), true)
    } else {
        (d, value < from)
    }
}

/// The registers as a family member's (the flags clear).
#[allow(clippy::too_many_arguments)]
fn regs_of(d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> Regs {
    Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, ..Regs::default() }
}

/// The registers given back as a tuple: d0-a6, X, C.
#[allow(clippy::type_complexity)]
fn all_of(r: &Regs) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
    (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
}

impl Game {
    /// The yield (`func_000522`): the task goes on at `resume` next
    /// time, and the routine returns at once.  X and C from it.
    pub(crate) fn yc_yield(&mut self, r: &mut Regs, resume: u32) {
        (r.x, r.c, _) = self.r_000522(r.d6, resume);
        r.unwound = false;
    }

    /// The running task (`func_0005F8`, d6 and a6) waits `frames`
    /// (`TASK_WAIT`, counted by `func_01793A` where it goes on) and
    /// yields to `resume`.
    pub(crate) fn yc_wait(&mut self, r: &mut Regs, frames: u16, resume: u32) {
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        self.write16(r.a6 + TASK_WAIT, frames);
        self.yc_yield(r, resume);
    }

    /// `func_01793A`: true while the running task still waits (its
    /// count goes down and the routine returns).
    pub(crate) fn yc_waiting(&mut self, r: &mut Regs) -> bool {
        let waiting;
        (r.d6, r.a6, r.x, r.c, waiting) = self.r_01793a(r.d6);
        r.unwound = false;
        waiting
    }

    /// `func_001BB4`: true while the objects are stopped (the routine
    /// returns).  C from it.
    fn yc_stopped(&mut self, r: &mut Regs) -> bool {
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        r.unwound = false;
        stopped
    }

    /// `func_019004` (a task's routine, after a yield at 0x19000): once
    /// the objects run (`func_001BB4`), the screen by `LAST`
    /// (`func_0179B0`).
    pub fn r_019004(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.yc_stopped(r) {
            return;
        }
        (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) = self.r_0179b0(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_01906E` (a task's routine, after the yield at 0x1906A):
    /// once the objects run (`func_001BB4`), the colour cycles
    /// (`func_01914A`) and 0x30 frames.
    pub fn r_01906e(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.yc_stopped(r) {
            return;
        }
        (r.d0, r.a0) = self.r_01914a(r.d4, r.d5, r.d6, r.a0);
        self.yc_wait(r, WAIT_30, AFTER_CYCLES);
    }

    /// `func_01908A` (a task's routine): after the wait, the tasks
    /// `SOUNDS_TASK` and `SQUARES_TASK` (`func_0003E8`) and 0x30 frames.
    pub fn r_01908a(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.yc_waiting(r) {
            return;
        }
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, TASKS_PRIORITY, SOUNDS_TASK);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, TASKS_PRIORITY, SQUARES_TASK);
        self.yc_wait(r, WAIT_30, AFTER_TASKS);
    }

    /// `func_0190C2` (a task's routine): after the wait, `func_002E4E`
    /// with d0 0 (its low word; `OBJECT_0_32` negative) or 5, and 0x130
    /// frames.
    pub fn r_0190c2(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.yc_waiting(r) {
            return;
        }
        r.d0 = if (self.read8(OBJECT_0_32) as i8) < 0 { low_word(r.d0, 0) } else { 5 };
        (r.d0, r.a0) = self.r_002e4e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5);
        self.yc_wait(r, WAIT_130, AFTER_2E4E);
    }

    /// `func_0190EA` (a task's routine): object 0's y off `WAIT_Y` by
    /// `NEAR` or more: `SIDE` 2 and on (`func_019140`); else the wait
    /// for its x (`func_01910A`).  d0's low byte the distance.
    pub fn r_0190ea(&mut self, regs: &mut Regs) {
        let r = regs;
        let (d, _) = distance(self.read8(OBJECT_0_Y_LOW), WAIT_Y);
        r.d0 = low_byte(r.d0, d);
        if d < NEAR {
            let t = self.r_01910a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = t;
        } else {
            self.write8(SIDE, SIDE_2);
            let t = self.r_019140(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = t;
        }
        r.unwound = false;
    }

    /// `func_01910A` (0x1910A, where the task waits a frame for object
    /// 0's x): it yields to `func_01910E`.  Gives back the registers, X
    /// and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01910a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = regs_of(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6);
        self.yc_yield(&mut r, AT_Y);
        all_of(&r)
    }

    /// `func_01910E` (a task's routine): object 0's x off `WAIT_X` by
    /// `NEAR` or more: `SIDE` 8 and on (`func_019140`); else a frame
    /// more (to `func_019132`).  d0's low byte the distance.
    pub fn r_01910e(&mut self, regs: &mut Regs) {
        let r = regs;
        let (d, _) = distance(self.read8(OBJECT_0_X_LOW), WAIT_X);
        r.d0 = low_byte(r.d0, d);
        if d < NEAR {
            self.yc_yield(r, AT_X);
            return;
        }
        self.write8(SIDE, SIDE_8);
        let t = self.r_019140(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = t;
        r.unwound = false;
    }

    /// `func_019132` (a task's routine): object 0's x at `WAIT_X_BACK`:
    /// back to the wait (`func_01910A`); else `SIDE` 4 and on
    /// (`func_019140`).
    pub fn r_019132(&mut self, regs: &mut Regs) {
        let r = regs;
        let t = if self.read8(OBJECT_0_X_LOW) == WAIT_X_BACK {
            self.r_01910a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6)
        } else {
            self.write8(SIDE, SIDE_4);
            self.r_019140(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5)
        };
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = t;
        r.unwound = false;
    }

    /// `func_019140` (0x19140, where the task's wait goes on): after it
    /// (`func_01793A`), `func_019200`.  Gives back the registers, X and
    /// C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_019140(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = regs_of(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, 0);
        if self.yc_waiting(&mut r) {
            return all_of(&r);
        }
        self.r_019200(d0, d1, d2, d3, d4, d5, r.d6, d7, a0, a1, a2, a3, a4, a5)
    }

    /// `func_0191BA` (a task's routine, `func_018860`'s): the screen
    /// loaded (`func_018D22`), the map's cells drawn (`func_0195FC`,
    /// columns 3-0xD and rows 2-0xA, every other, in `CELL_X`,
    /// `CELL_Y`) and a frame on (`func_0191F6`).
    pub fn r_0191ba(&mut self, regs: &mut Regs) {
        let r = regs;
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6) =
            self.r_018d22(r.d0, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        r.unwound = false;
        if self.restarted.is_some() {
            return;
        }
        let mut row = CELL_FIRST_Y;
        loop {
            self.write8(CELL_Y, row);
            let mut column = CELL_FIRST_X;
            loop {
                self.write8(CELL_X, column);
                (r.d0, r.a0) = self.r_0195fc();
                column = self.read8(CELL_X).wrapping_add(2);
                self.write8(CELL_X, column);
                r.d0 = low_byte(r.d0, column);
                if column >= CELL_END_X {
                    break;
                }
            }
            row = self.read8(CELL_Y).wrapping_add(2);
            self.write8(CELL_Y, row);
            r.d0 = low_byte(r.d0, row);
            if row >= CELL_END_Y {
                break;
            }
        }
        self.yc_yield(r, AFTER_CELLS);
    }

    /// `func_0191F6` (a task's routine): once the objects run
    /// (`func_001BB4`), the colour cycles (`func_01914A`) and
    /// `func_019200`.
    pub fn r_0191f6(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.yc_stopped(r) {
            return;
        }
        (r.d0, r.a0) = self.r_01914a(r.d4, r.d5, r.d6, r.a0);
        let t = self.r_019200(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = t;
        r.unwound = false;
    }

    /// `func_019200` (0x19200): the task `EXIT_TASK` (`func_0003E8`) and
    /// a frame on (`func_019214`).  Gives back the registers, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_019200(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = regs_of(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, 0);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(d4, d5, d6, EXIT_PRIORITY, EXIT_TASK);
        self.yc_yield(&mut r, AFTER_EXIT_TASK);
        all_of(&r)
    }

    /// `func_019214` (a task's routine): while `SIDE` is set, nothing
    /// (C clear; it looks again next frame); then 0x30 frames and the
    /// scene (`func_01922E`).
    pub fn r_019214(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.read8(SIDE) != 0 {
            r.c = false;
            return;
        }
        self.yc_wait(r, WAIT_30, BEFORE_SCENE);
    }

    /// `func_01922E` (a task's routine): after the wait, `BYTE_ADD7`
    /// cleared, the task ends (`func_000560`) and the routine in
    /// `SCENES` by `ROW` runs (d0 the offset, a0 the routine; X and C
    /// the offset's carry).
    pub fn r_01922e(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.yc_waiting(r) {
            return;
        }
        self.write8(BYTE_ADD7, 0);
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
        let twice = self.read8(ROW).wrapping_add(self.read8(ROW));
        let (offset, carry) = twice.overflowing_add(twice);
        r.x = carry;
        r.c = carry;
        r.d0 = low_word(r.d0, offset as i8 as i16 as u16);
        r.a0 = self.read32(SCENES.wrapping_add(offset as i8 as u32));
        let mut called = Regs { n: false, z: false, v: false, unwound: false, ..*r };
        self.call_family_15(r.a0, &mut called);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7) = (called.d0, called.d1, called.d2, called.d3, called.d4, called.d5, called.d6, called.d7);
        (r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6) = (called.a0, called.a1, called.a2, called.a3, called.a4, called.a5, called.a6);
        (r.x, r.c) = (called.x, called.c);
        r.unwound = false;
    }

    /// `func_019262` (a task's routine): `SPARED`'s bit 0 set,
    /// `BYTE_80DF` cleared, `LONG_ADC6`'s first byte 0xFF, `MODE` 0xC0,
    /// `SIDE` cleared, the Z80 command `COMMAND` (`func_00349E`, d1
    /// 0x20) and 0x20 frames.
    pub fn r_019262(&mut self, regs: &mut Regs) {
        let r = regs;
        let spared = self.read8(SPARED);
        self.write8(SPARED, spared | 1);
        self.write8(BYTE_80DF, 0);
        self.write8(LONG_ADC6, 0xFF);
        self.write8(MODE, 0xC0);
        self.write8(SIDE, 0);
        r.d1 = COMMAND_D1;
        r.d0 = low_byte(r.d0, COMMAND);
        self.r_00349e(r.d0);
        self.yc_wait(r, WAIT_20, AFTER_COMMAND);
    }

    /// `func_019298` (a task's routine): after the wait, from stage
    /// `STAGE_END` on nothing more (C clear); else a grid of tasks with
    /// a sprite (`GRID_TASK`, `func_001222`) from x `GRID_FIRST_X`, y
    /// `GRID_FIRST_Y` by `GRID_STEP` (counted in `CELL_X`, `CELL_Y`),
    /// and a frame on (`func_019312`).
    pub fn r_019298(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.yc_waiting(r) {
            return;
        }
        let stage = self.read8(STAGE);
        r.d0 = low_byte(r.d0, stage);
        if stage >= STAGE_END {
            r.c = false;
            return;
        }
        self.write8(CELL_Y, GRID_FIRST_Y);
        loop {
            self.write8(CELL_X, GRID_FIRST_X);
            loop {
                (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, GRID_DEPTH, GRID_PRIORITY, GRID_TASK);
                self.write8(r.a5 + SPRITE_FRAME, GRID_FRAME);
                self.write8(r.a5 + SPRITE_FLAGS, GRID_FLAGS);
                self.write8(r.a5 + SPRITE_X, 0);
                self.write8(r.a5 + SPRITE_Y, 0);
                let y = self.read8(CELL_Y);
                self.write8(r.a5 + SPRITE_Y_LOW, y);
                let x = self.read8(CELL_X);
                self.write8(r.a5 + SPRITE_X_LOW, x);
                let x = x.wrapping_add(GRID_STEP);
                r.d0 = low_byte(r.d0, x);
                self.write8(CELL_X, x);
                if x >= GRID_END_X {
                    break;
                }
            }
            let y = self.read8(CELL_Y).wrapping_add(GRID_STEP);
            r.d0 = low_byte(r.d0, y);
            self.write8(CELL_Y, y);
            if y >= GRID_END_Y {
                break;
            }
        }
        self.yc_yield(r, AFTER_GRID);
    }

    /// `func_019312` (a task's routine): the map's cells emptied
    /// (`func_00167A`, `func_00168C`, `func_001278`, `func_0010D6`;
    /// columns 3-0xD and rows 2-0xA, every other, in `CELL_X`,
    /// `CELL_Y`), `WORD_80C4` 0x2E00 and `func_017828` with d0 1, and
    /// 0x80 frames.
    pub fn r_019312(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write8(CELL_Y, CELL_FIRST_Y);
        loop {
            self.write8(CELL_X, CELL_FIRST_X);
            loop {
                (r.a0, r.x) = self.r_00167a();
                (r.d0, r.a0) = self.r_00168c(r.d0);
                self.r_001278();
                r.x = self.r_0010d6(r.d0);
                let column = self.read8(CELL_X).wrapping_add(2);
                self.write8(CELL_X, column);
                if column >= CELL_END_X {
                    break;
                }
            }
            let row = self.read8(CELL_Y).wrapping_add(2);
            self.write8(CELL_Y, row);
            if row >= CELL_END_Y {
                break;
            }
        }
        self.write16(WORD_80C4, WORD_80C4_VALUE);
        r.d0 = 1;
        (r.x, r.c) = self.r_017828(r.d0, r.d4, r.d5, r.d6);
        self.yc_wait(r, WAIT_80, AFTER_EMPTIED);
    }

    /// `func_019366` (a task's routine): after the wait, the screen
    /// again (`func_018CB0`, `func_0094B2`), `BYTE_ADD7` cleared, the
    /// task `EXIT_TASK` and a frame on (`func_019390`).
    pub fn r_019366(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.yc_waiting(r) {
            return;
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5) =
            self.r_018cb0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return;
        }
        r.c = self.r_0094b2();
        self.write8(BYTE_ADD7, 0);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, EXIT_PRIORITY, EXIT_TASK);
        self.yc_yield(r, AFTER_SCREEN);
    }

    /// `func_019390` (a task's routine): while `SIDE` is set, nothing
    /// (C clear; it looks again next frame); then the task's routine
    /// `SCREEN_AGAIN` from now on (`func_00053E`) and `func_0099A2`.
    pub fn r_019390(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.read8(SIDE) != 0 {
            r.c = false;
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, SCREEN_AGAIN);
        (r.x, r.c) = self.r_0099a2(r.d4);
    }

    /// `func_01966A` (a task's routine): `BYTE_A93C` set, and `SIDE` by
    /// object 0's place: x `NEAR` or more off `PLACE_X`: 8 (left of
    /// it) or 4; else y off `PLACE_Y`: 2 (above) or 1; else 1 and a
    /// frame on (`func_0196DE`).  d0's low byte the last read or
    /// distance; X the distance's.
    pub fn r_01966a(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(BYTE_A93C, 1);
        let (dx, x) = distance(self.read8(OBJECT_0_X_LOW), PLACE_X);
        r.x = x;
        if dx >= NEAR {
            let at = self.read8(OBJECT_0_X_LOW);
            r.d0 = low_byte(r.d0, at);
            self.write8(SIDE, if at < PLACE_X { SIDE_8 } else { SIDE_4 });
            r.c = false;
            return;
        }
        let (dy, x) = distance(self.read8(OBJECT_0_Y_LOW), PLACE_Y);
        r.x = x;
        if dy >= NEAR {
            let at = self.read8(OBJECT_0_Y_LOW);
            r.d0 = low_byte(r.d0, at);
            self.write8(SIDE, if at < PLACE_Y { SIDE_2 } else { SIDE_1 });
            r.c = false;
            return;
        }
        r.d0 = low_byte(r.d0, dy);
        self.write8(SIDE, SIDE_1);
        self.yc_yield(r, EXIT_AFTER);
    }

    /// `func_0196DE` (a task's routine): `SIDE` and `BYTE_A93C` cleared,
    /// and the task ends (`func_000560`).
    pub fn r_0196de(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write8(SIDE, 0);
        self.write8(BYTE_A93C, 0);
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
        r.unwound = false;
    }
}
