//! The beaten boss (`t_boss.rs`'s `loc_015432`), 0x159A0-0x15C26: its
//! routines while it bursts (`func_0159A0`, `func_0159F4`,
//! `func_015A20`), the bursts at random around it every 8 frames
//! (`func_015AB8`), the pieces thrown out that bounce and fade
//! (`func_015B56`), and the task that ends the stage after it
//! (`func_015A8C`).

use super::t_common::*;
use crate::{Game, Regs};

/// The routines (code addresses).
const BURSTING: u32 = 0x0159F4;
const FALLING_APART: u32 = 0x015A20;
const AFTER: u32 = 0x015A8C;
const BURST: u32 = 0x015B42;
const PIECE: u32 = 0x015BC6;
const PIECE_FADING: u32 = 0x015C1C;
/// What `AFTER` starts when its time is up (`func_019262`).
const NEXT_STAGE: u32 = 0x019262;

/// Animations in the ROM.
const ANIM_BURSTING: u32 = 0x0EAB98;
const ANIM_FALLING_APART: u32 = 0x0EABAA;
const ANIM_BURST: u32 = 0x0EAB8A;

/// The sprite frames of the four pieces thrown as it starts to burst.
const FIRST_PIECES: [u8; 4] = [0x4A, 0x4B, 0x8E, 0x64];

/// The frame count (`sprite.rs`): a burst on frames whose low 3 bits
/// are 0.
const FRAME: u32 = 0xFFFF_80D8;

/// A piece's velocity (words, kept in the task: x, y).
const TASK_VELOCITY_X: u32 = 0x16;
const TASK_VELOCITY_Y: u32 = 0x18;

impl Game {
    /// `func_0159A0` (the beaten boss's routine): bursts
    /// (`func_015AAA`) and its animation; when that ends,
    /// `ANIM_BURSTING`, four pieces (`func_015B56` with
    /// `FIRST_PIECES`), and 0x80 frames of `BURSTING` from the next
    /// frame.
    pub fn r_0159a0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_burst_now(r);
        self.t_animate(r);
        if !r.c {
            return;
        }
        self.r_001542(r.a4);
        self.write32(r.a4 + OBJ_ANIM, ANIM_BURSTING);
        for frame in FIRST_PIECES {
            r.d3 = low_byte(r.d3, frame);
            self.t_piece(r);
        }
        self.write32(r.a4 + OBJ_ROUTINE, BURSTING);
        self.write8(r.a4 + OBJ_26, 0x80);
        r.c = false;
    }

    /// `func_0159F4`, `BURSTING`: bursts and its animation for
    /// `OBJ_26`'s frames; then `ANIM_FALLING_APART` from its start and
    /// `FALLING_APART` (at once).
    pub fn r_0159f4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_burst_now(r);
        self.t_animate(r);
        if !self.t_count_down(r, r.a4 + OBJ_26) {
            return;
        }
        self.write32(r.a4 + OBJ_ANIM, ANIM_FALLING_APART);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.write32(r.a4 + OBJ_ROUTINE, FALLING_APART);
        r.c = false;
        self.r_015a20(r);
    }

    /// `func_015A20`, `FALLING_APART`: bursts and its animation; when
    /// that ends, sound 0x5F, the task `AFTER` (priority 0x42, 0x80
    /// frames), the object gone (its sprite record freed, `OBJ_ID` 0),
    /// a last burst (`func_015AB8`) and 16 pieces (`func_015B16`) at
    /// speed 0x200.  Gives back every register as left (d0 0xFFFF).
    /// Differs in the check as its translation does: the random
    /// numbers change under an interrupt during the call.
    pub fn r_015a20(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_burst_now(r);
        self.t_animate(r);
        if !r.c {
            return;
        }
        self.t_sound(r, 0x5F);
        self.t_make_bare_task(r, 0x42, AFTER);
        self.write8(r.a6 + TASK_COUNT, 0x80);
        self.t_reload_object(r);
        r.c = self.r_000890();
        self.write8(r.a4 + OBJ_ID, 0);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x) =
            self.r_015ab8(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.t_reload_object(r);
        self.r_001542(r.a4);
        self.write16(SPEED, 0x200);
        // d0 is kept around each call (movem), the dbra's count
        for _ in 0..16 {
            (_, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x, r.c) =
                self.r_015b16(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
        r.d0 = 0xFFFF;
    }

    /// `func_015A8C`, `AFTER`: when its count is up, the task ends and
    /// a task running `NEXT_STAGE` (priority 0x32) starts.
    pub fn r_015a8c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.t_count_down(r, r.a6 + TASK_COUNT) {
            return;
        }
        self.t_task_end_bare(r);
        self.t_make_bare_task(r, 0x32, NEXT_STAGE);
    }

    /// `func_015AAA` on the registers.
    fn t_burst_now(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a4, r.a6, r.x) =
            self.r_015aaa(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
    }

    /// `func_015AAA`: on frames whose low 3 bits are 0, a burst
    /// (`func_015AB8`).  Gives back d0-d6, a4, a6 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_015aaa(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let phase = self.read8(FRAME) & 7;
        if phase != 0 {
            return (low_byte(d0, phase), d1, d2, d3, d4, d5, d6, a4, a6, x);
        }
        let (d0, d1, d2, d3, d4, d5, d6, a4, _, a6, x) = self.r_015ab8(d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        (d0, d1, d2, d3, d4, d5, d6, a4, a6, x)
    }

    /// `func_015AB8`: a task running `BURST` (priority 0xC2, sprite
    /// depth 1, `ANIM_BURST`) at the object at a4, moved at random 0x20
    /// pixels or less either way along x, 0x10 along y; then, at speed
    /// a random byte, a piece (`func_015B16`).  If no task can be had,
    /// d4, a4, d5 and a5 back to the running object.  Gives back d0-d6,
    /// a4, a5, a6 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_015ab8(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let mut r = Regs { d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, ..Regs::default() };
        self.r_001542(a4);
        let x = self.read8(POSITION_X_LOW).wrapping_sub(0x20);
        self.write8(POSITION_X_LOW, x);
        let n = self.t_random(&mut r) & 0x3F;
        r.d0 = low_byte(r.d0, n);
        let x = self.read8(POSITION_X_LOW).wrapping_add(n);
        self.write8(POSITION_X_LOW, x);
        let y = self.read8(POSITION_Y_LOW).wrapping_sub(0x20);
        self.write8(POSITION_Y_LOW, y);
        let n = self.t_random(&mut r) & 0x1F;
        r.d0 = low_byte(r.d0, n);
        let y = self.read8(POSITION_Y_LOW).wrapping_add(n);
        self.write8(POSITION_Y_LOW, y);
        self.t_make_task(&mut r, 1, 0xC2, BURST);
        if r.c {
            (r.d4, r.d5, r.a4, r.a5, r.c) = self.r_015b30(r.d4, r.d5);
            return (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x);
        }
        self.r_00152a(r.a6);
        self.write32(r.a6 + TASK_ANIM, ANIM_BURST);
        self.write8(r.a5 + SPRITE_FLAGS, 0x81);
        self.write16(SPEED, 0);
        let speed = self.t_random(&mut r);
        self.write8(SPEED_LOW, speed);
        let (d0, d1, d2, d3, d4, d5, d6, a4, a5, a6, x, _) =
            self.r_015b16(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (d0, d1, d2, d3, d4, d5, d6, a4, a5, a6, x)
    }

    /// `func_015B16`: a piece (`func_015B56`) of frame 0x1F-0x21 at
    /// random (`func_001BC0`), sound 0x5E; d4, a4, d5 and a5 back to
    /// the running object.  Gives back d0-d6, a4, a5, a6, X and C
    /// (clear).  Differs in the check as its translation does, in
    /// `func_015A20`'s calls (an interrupt during them).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_015b16(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, ..Regs::default() };
        r.d0 = self.r_001bc0(2, r.d1);
        let frame = (r.d0 as u8).wrapping_add(0x1F);
        r.d0 = low_byte(r.d0, frame);
        r.d3 = low_byte(r.d3, frame);
        self.t_piece(&mut r);
        self.t_sound(&mut r, 0x5E);
        (r.d4, r.d5, r.a4, r.a5, r.c) = self.r_015b30(r.d4, r.d5);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `loc_015B30`: d4, a4, d5 and a5 back to the running object.
    /// Gives back those and C (clear).
    pub fn r_015b30(&mut self, d4: u32, d5: u32) -> (u32, u32, u32, u32, bool) {
        let mut r = Regs { d4, d5, ..Regs::default() };
        self.t_reload_object(&mut r);
        self.t_reload_sprite(&mut r);
        (r.d4, r.d5, r.a4, r.a5, false)
    }

    /// `func_015B42`, `BURST`: the task rises a pixel a frame with its
    /// animation, and ends with it.
    pub fn r_015b42(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let y = self.read8(r.a6 + TASK_Y_LOW).wrapping_sub(1);
        self.write8(r.a6 + TASK_Y_LOW, y);
        self.t_task_animate(r);
        if r.c {
            self.t_task_end(r);
            return;
        }
        self.t_task_place(r);
    }

    /// `func_015B56` on the registers.
    fn t_piece(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x) =
            self.r_015b56(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_015B56`: a task running `PIECE` (priority 0xC2, sprite
    /// depth 1) at `POSITION_X`, `POSITION_Y`, of frame d3 (its low
    /// byte), thrown at a random angle at `SPEED` (`func_00410C`, the
    /// velocity kept), offset 0 going up (rate 0xFB and a random low
    /// byte), to bounce once or twice (`TASK_12`); then d4, a4, d5 and
    /// a5 back to the running object.  Gives back d0, d1, d2, d4, d5,
    /// d6, a4, a5, a6 and X.  Differs in the check as its translation
    /// does, in `func_015A20`'s calls (an interrupt during them).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_015b56(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, ..Regs::default() };
        self.t_make_task(&mut r, 1, 0xC2, PIECE);
        if !r.c {
            self.r_00152a(r.a6);
            self.write8(r.a5 + SPRITE_FLAGS, 0x81);
            self.write8(r.a5 + SPRITE_FRAME, d3 as u8);
            self.t_random(&mut r);
            (r.d0, r.d1, r.d2, r.x) = self.r_00410c(r.d0, r.d1, r.d2, r.x);
            let vx = self.read16(VELOCITY_X);
            self.write16(r.a6 + TASK_VELOCITY_X, vx);
            let vy = self.read16(VELOCITY_Y);
            self.write16(r.a6 + TASK_VELOCITY_Y, vy);
            self.write8(r.a6 + TASK_OFFSET, 0);
            let low = self.t_random(&mut r);
            self.write8(r.a6 + TASK_RATE_LOW, low);
            self.write8(r.a6 + TASK_RATE, 0xFB);
            let bounces = (self.t_random(&mut r) & 1) + 1;
            r.d0 = low_byte(r.d0, bounces);
            self.write8(r.a6 + TASK_12, bounces);
            self.t_task_place_offset(&mut r);
        }
        self.t_reload_object(&mut r);
        self.t_reload_sprite(&mut r);
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x)
    }

    /// `func_015BC6`, `PIECE`: the task moves by its velocity, falls
    /// (rate up 0x30, `func_00C742`; offset by it); each time it is
    /// down (offset not negative) it bounces (offset 0, rate -0x100);
    /// at the last, 0x40 frames of `PIECE_FADING` (at once), its sprite
    /// flags 0xC1.  The sprite placed and its depth.
    pub fn r_015bc6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let vx = self.read16(r.a6 + TASK_VELOCITY_X);
        self.write16(VELOCITY_X, vx);
        let vy = self.read16(r.a6 + TASK_VELOCITY_Y);
        self.write16(VELOCITY_Y, vy);
        self.t_move_x(r);
        self.t_move_y(r);
        r.d0 = 0x30;
        self.r_00c742(r.d0, r.a6);
        let up;
        (r.d0, r.x, up) = self.r_001aca(r.d0, r.a6);
        if !up {
            self.write8(r.a6 + TASK_OFFSET, 0);
            self.write16(r.a6 + TASK_RATE, 0xFF00);
            let left = self.read8(r.a6 + TASK_12).wrapping_sub(1);
            self.write8(r.a6 + TASK_12, left);
            if left == 0 {
                self.write8(r.a6 + TASK_COUNT, 0x40);
                self.write8(r.a5 + SPRITE_FLAGS, 0xC1);
                self.t_next_task(r, PIECE_FADING);
                self.r_015c1c(r);
                return;
            }
        }
        self.t_piece_place(r);
    }

    /// `loc_015BFC` on the registers.
    fn t_piece_place(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_015bfc(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
    }

    /// `loc_015BFC`: the sprite placed for the task with its offset
    /// (`func_001B94`) and its depth (`func_001B74`).  Gives back d0, X
    /// and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_015bfc(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, bool, bool) {
        let (d0, _, _) = self.r_001b94(a5, a6);
        self.r_001b74(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5, a6)
    }

    /// `func_015C1C`, `PIECE_FADING`: placed (`loc_015BFC`) for its
    /// count's frames, then the task ends.
    pub fn r_015c1c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.t_count_down(r, r.a6 + TASK_COUNT) {
            self.t_task_end(r);
            return;
        }
        self.t_piece_place(r);
    }
}
