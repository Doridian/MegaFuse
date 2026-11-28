//! The tasks the boss of `t_boss.rs` makes, 0x151B2-0x15334 and
//! 0x1551A-0x15766: two that fall out of it, bounce up and then go
//! their way (`func_0151B2` on), two thrown up that come down
//! (`func_0152AE` on), and one shot up that comes down on a cell of
//! the map, breaking into four that run along the map's cells
//! (`func_01551A` on).

use super::t_common::*;
use crate::{Game, Regs};

/// The tasks' routines (code addresses).
const DROPPED: u32 = 0x0151D2;
const BOUNCED: u32 = 0x015216;
const WAITED: u32 = 0x01523E;
const GOING: u32 = 0x01525A;
const DONE: u32 = 0x015298;
const THROWN: u32 = 0x0152FA;
const SHOT: u32 = 0x015572;
const SHOT_LANDED: u32 = 0x01560E;
const FLASH: u32 = 0x0155F6;
const RUNNER: u32 = 0x01567E;
const RUNNER_PIECE: u32 = 0x0156FA;

/// Animations in the ROM.
const ANIM_THROWN_1: u32 = 0x0EACC0;
const ANIM_THROWN_2: u32 = 0x0EAC32;
const ANIM_SHOT: u32 = 0x0EABD6;
const ANIM_SHOT_LANDED: u32 = 0x0EABD0;
const ANIM_PIECE: u32 = 0x0EABDC;

/// The cell's byte a `RUNNER_PIECE` stops a put task on.
const CELL_PUT: u8 = 1;

impl Game {
    /// `func_0151B2`: a task running `DROPPED` (priority 0x42, sprite
    /// depth 4) at `POSITION_X`, `POSITION_Y`.  Gives back d5, d6, a5,
    /// a6 and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0151b2(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, ..Regs::default() };
        self.t_make_task(&mut r, 4, 0x42, DROPPED);
        self.r_00152a(r.a6);
        self.write8(r.a5 + SPRITE_FLAGS, 0x81);
        (r.d5, r.d6, r.a5, r.a6, r.x)
    }

    /// `func_0151D2`, `DROPPED`: the task's animation; when it ends,
    /// frame 0x4C (`TASK_17` 3) or 0x50, offset 0xEB going up (rate
    /// -0x100), `TASK_18` 0xFF, `func_004FF4`, and `BOUNCED` (at once).
    pub fn r_0151d2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_task_animate(r);
        if !r.c {
            self.t_task_place(r);
            return;
        }
        let frame = if self.read8(r.a6 + TASK_17) == 3 { 0x4C } else { 0x50 };
        self.write8(r.a5 + SPRITE_FRAME, frame);
        self.write8(r.a6 + TASK_OFFSET, 0xEB);
        self.write16(r.a6 + TASK_RATE, 0xFF00);
        self.write8(r.a6 + TASK_18, 0xFF);
        (r.d0, r.x, r.c) = self.r_004ff4(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.t_next_task(r, BOUNCED);
        self.r_015216(r);
    }

    /// `func_015216`, `BOUNCED`: the rate 0x10 less (pulled down), the
    /// offset by it (`func_001ACA`); once the offset is below 0xD3 (on
    /// the way down, unsigned), 0x20 frames of `WAITED` (at once).
    pub fn r_015216(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let rate = self.read16(r.a6 + TASK_RATE).wrapping_sub(0x10);
        self.write16(r.a6 + TASK_RATE, rate);
        (r.d0, r.x, _) = self.r_001aca(r.d0, r.a6);
        if r.d0 as u8 >= 0xD3 {
            self.t_task_place_offset(r);
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x20);
        self.t_next_task(r, WAITED);
        self.r_01523e(r);
    }

    /// `func_01523E`, `WAITED`: when the count is up, 0x14 frames of
    /// `GOING` (at once).
    pub fn r_01523e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.t_count_down(r, r.a6 + TASK_COUNT) {
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x14);
        self.t_next_task(r, GOING);
        self.r_01525a(r);
    }

    /// `func_01525A`, `GOING`: the task moves its way (`TASK_17`) at
    /// speed 0x80, along x only (`func_001A88`); when the count is up,
    /// `HELPERS` 0, sound 0x66, `TASK_18` 0 and `DONE` from the next
    /// frame.
    pub fn r_01525a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write16(SPEED, 0x80);
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_17));
        self.r_001620(r.d0);
        self.t_move_x(r);
        self.t_task_place_offset(r);
        if !self.t_count_down(r, r.a6 + TASK_COUNT) {
            return;
        }
        self.write8(HELPERS, 0);
        self.t_next_task(r, DONE);
        self.t_sound(r, 0x66);
        self.write8(r.a6 + TASK_18, 0);
        r.c = false;
    }

    /// `func_015298`, `DONE`: the task ends (`func_00121A`).
    pub fn r_015298(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.t_task_end(regs);
    }

    /// `func_01529C`: `POSITION_X`, `POSITION_Y` from the object at a4,
    /// 0x1C to the left and 0xC0 down.
    pub fn r_01529c(&mut self, a4: u32) {
        self.r_001542(a4);
        let y = self.read16(POSITION_Y).wrapping_add(0xC0);
        self.write16(POSITION_Y, y);
        let x = self.read16(POSITION_X).wrapping_sub(0x1C);
        self.write16(POSITION_X, x);
    }

    /// `func_0152AE`: two `func_0152D4` tasks below the object at a4
    /// (`func_01529C`), 0x38 apart, with `ANIM_THROWN_1` and
    /// `ANIM_THROWN_2`.  Gives back d5, d6, a5, a6, X and C (clear).
    #[allow(clippy::too_many_arguments)]
    pub fn r_0152ae(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        self.r_01529c(a4);
        let (d5, d6, a5, a6, _) = self.r_0152d4(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        self.write32(a6 + TASK_ANIM, ANIM_THROWN_1);
        let x = self.read16(POSITION_X).wrapping_add(0x38);
        self.write16(POSITION_X, x);
        let (d5, d6, a5, a6, x) = self.r_0152d4(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        self.write32(a6 + TASK_ANIM, ANIM_THROWN_2);
        (d5, d6, a5, a6, x, false)
    }

    /// `func_0152D4`: a task running `THROWN` (priority 0x42, sprite
    /// depth 5) at `POSITION_X`, `POSITION_Y`, going up (rate -0x600).
    /// Gives back d5, d6, a5, a6 and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0152d4(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, ..Regs::default() };
        self.t_make_task(&mut r, 5, 0x42, THROWN);
        self.r_00152a(r.a6);
        self.write8(r.a5 + SPRITE_FLAGS, 0x81);
        self.write16(r.a6 + TASK_RATE, 0xFA00);
        (r.d5, r.d6, r.a5, r.a6, r.x)
    }

    /// `func_0152FA`, `THROWN`: the task's animation, its rate up by
    /// 0x16 (`func_00C742`); once the rate's high byte is 0, `HELPERS`
    /// 0, sound 0x66 and `DONE` from the next frame.  The task moves
    /// down by the rate.
    pub fn r_0152fa(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_task_animate(r);
        r.d0 = 0x16;
        self.r_00c742(r.d0, r.a6);
        if self.read8(r.a6 + TASK_RATE) == 0 {
            self.write8(HELPERS, 0);
            self.t_sound(r, 0x66);
            self.t_next_task(r, DONE);
        }
        let rate = self.read16(r.a6 + TASK_RATE);
        self.write16(VELOCITY_Y, rate);
        self.t_move_y(r);
        self.t_task_place(r);
    }

    /// `func_01551A`: sound 0x4B and, at the object at a4, a task
    /// running `SHOT` (priority 0x42, sprite depth 2), moved 0x30 up
    /// (velocity -0x3000), `ANIM_SHOT`, 0x80 frames.  d4, a4, d5 and
    /// a5 back to the running object.  Gives back d4, d5, d6, a4, a5
    /// and a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_01551a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, ..Regs::default() };
        self.t_sound(&mut r, 0x4B);
        self.r_001542(r.a4);
        self.t_make_task(&mut r, 2, 0x42, SHOT);
        if !r.c {
            self.r_00152a(r.a6);
            self.write8(r.a5 + SPRITE_FLAGS, 0x81);
            self.write16(VELOCITY_Y, 0xD000);
            self.t_move_y(&mut r);
            self.write32(r.a6 + TASK_ANIM, ANIM_SHOT);
            self.write8(r.a6 + TASK_COUNT, 0x80);
        }
        self.t_reload_object(&mut r);
        self.t_reload_sprite(&mut r);
        (r.d4, r.d5, r.d6, r.a4, r.a5, r.a6)
    }

    /// `func_015572`, `SHOT`: the task's animation, up 4 a frame; when
    /// its count is up, it comes down on a cell picked at random
    /// (`func_00CED8`), 0x100 pixels above it at first (its y's high
    /// byte one less), `ANIM_SHOT_LANDED`, 0x40 frames
    /// of `SHOT_LANDED` from the next frame; sound 0x4C and a task
    /// running `FLASH` (priority 0x42, sprite depth 0xFE) there for
    /// 0x40 frames.
    pub fn r_015572(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_task_animate(r);
        self.write16(VELOCITY_Y, 0xFC00);
        self.t_move_y(r);
        if !self.t_count_down(r, r.a6 + TASK_COUNT) {
            self.t_task_place(r);
            return;
        }
        (r.d0, r.d1, r.a0) = self.r_00ced8(r.d1);
        (r.d0, r.x) = self.r_001550(r.d0, r.a6);
        self.r_00153a(r.a6);
        self.t_task_place(r);
        self.t_depth(r);
        let high = self.read8(r.a6 + TASK_Y).wrapping_sub(1);
        self.write8(r.a6 + TASK_Y, high);
        self.t_task_place(r);
        self.write8(r.a6 + TASK_COUNT, 0x40);
        self.write32(r.a6 + TASK_ANIM, ANIM_SHOT_LANDED);
        self.t_next_task(r, SHOT_LANDED);
        self.t_sound(r, 0x4C);
        self.t_make_task(r, 0xFE, 0x42, FLASH);
        if !r.c {
            self.r_00152a(r.a6);
            self.write8(r.a6 + TASK_COUNT, 0x40);
            self.write8(r.a5 + SPRITE_FLAGS, 0x84);
            r.c = false;
        }
    }

    /// `func_0155F6`, `FLASH`: frame 0xE0 placed for the task; it ends
    /// when its count is up.
    pub fn r_0155f6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_task_place(r);
        self.write8(r.a5 + SPRITE_FRAME, 0xE0);
        if self.t_count_down(r, r.a6 + TASK_COUNT) {
            self.t_task_end(r);
        }
    }

    /// `func_01560E`, `SHOT_LANDED`: the task's animation, down 4 a
    /// frame; when its count is up, it stays (`ANIM_PIECE`, its
    /// `TASK_12`, `TASK_13` 0) as a `RUNNER_PIECE` from the next frame,
    /// and four tasks running `RUNNER` (priority 0x42) start at it, one
    /// each way (`TASK_15` 0-3), each for 3 cells (`TASK_17`), a cell
    /// every 8 frames; fewer if no task can be had.  Gives back the
    /// last task made in d6, a6.
    pub fn r_01560e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_task_animate(r);
        self.write16(VELOCITY_Y, 0x400);
        self.t_move_y(r);
        self.t_task_place(r);
        if !self.t_count_down(r, r.a6 + TASK_COUNT) {
            return;
        }
        self.r_00153a(r.a6);
        self.write8(r.a6 + TASK_12, 0);
        self.write8(r.a6 + TASK_13, 0);
        self.write32(r.a6 + TASK_ANIM, ANIM_PIECE);
        self.t_next_task(r, RUNNER_PIECE);
        let mut way: u8 = 0;
        r.d1 = low_byte(r.d1, way);
        loop {
            self.t_make_bare_task(r, 0x42, RUNNER);
            if r.c {
                return;
            }
            self.r_00152a(r.a6);
            self.write8(r.a6 + TASK_17, 3);
            self.write8(r.a6 + TASK_COUNT, 8);
            self.write8(r.a6 + TASK_15, way);
            way += 1;
            r.d1 = low_byte(r.d1, way);
            r.x = false;
            r.c = false;
            if way == 4 {
                return;
            }
        }
    }

    /// `func_01567E`, `RUNNER`: every 8 frames the next cell its way
    /// (`TASK_15`, `func_0016F6`) from its cell; if there is one, the
    /// task moves to it and leaves a `func_0156B6` task there, until
    /// its cells (`TASK_17`) are done; then, or at none, it ends.
    pub fn r_01567e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.t_count_down(r, r.a6 + TASK_COUNT) {
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 8);
        r.d0 = self.r_0015f8(r.a6);
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_15));
        (r.d0, r.d1, r.a0, r.x) = self.r_0016f6(r.d0, r.d1);
        if r.d0 as u8 != 0 {
            (r.d0, r.x) = self.r_001550(r.d0, r.a6);
            (r.d0, r.d5, r.d6, r.a5, r.a6) = self.r_0156b6(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            if !self.t_count_down(r, r.a6 + TASK_17) {
                return;
            }
        }
        self.t_task_end_bare(r);
    }

    /// `func_0156B6`: a task running `RUNNER_PIECE` (priority 0x42,
    /// sprite depth 0x80) at the cell `CELL_X`, `CELL_Y`
    /// (`func_001550`), `ANIM_PIECE`, and sound 0x5D; then d6, a6, d5
    /// and a5 back to the running task.  Gives back d0, d5, d6, a5 and
    /// a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0156b6(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, ..Regs::default() };
        self.t_make_task(&mut r, 0x80, 0x42, RUNNER_PIECE);
        if !r.c {
            (r.d0, r.x) = self.r_001550(r.d0, r.a6);
            self.write8(r.a5 + SPRITE_FLAGS, 0x81);
            self.write32(r.a6 + TASK_ANIM, ANIM_PIECE);
            self.t_sound(&mut r, 0x5D);
        }
        self.t_reload_task(&mut r);
        (r.d0, r.d5, r.d6, r.a5, r.a6)
    }

    /// `func_0156FA`, `RUNNER_PIECE`: `func_003554` with 8 at the task;
    /// a put task on its cell (byte `CELL_PUT`) is stopped
    /// (`func_005E0E`); the task's animation, and when it ends the task
    /// ends.
    pub fn r_0156fa(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = 8;
        (r.d1, r.d2, r.x, r.c) = self.r_003554(r.d0, r.d1, r.d2, r.a6);
        r.d0 = self.r_0015f8(r.a6);
        (r.a0, r.x) = self.r_00167a();
        let cell = self.read8(r.a0);
        r.d0 = low_byte(r.d0, cell);
        if cell == CELL_PUT {
            (r.x, _, r.c) = self.r_005e0e(r.d2, r.d6, r.a6);
        }
        self.t_task_animate(r);
        if r.c {
            self.t_task_end(r);
            return;
        }
        self.t_task_place(r);
        self.t_depth(r);
    }
}

