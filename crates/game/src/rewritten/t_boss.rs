//! The boss object's routines, 0x15334-0x15884 (the object `t_start.rs`
//! makes, a4, its sprite record a5): it lands bouncing (`func_015334`),
//! drops two tasks, throws two, then walks the map (`func_015406`):
//! by object 0's cell it shoots (`func_0154EA`), jumps
//! (`func_015766`) or makes two objects that chase object 0
//! (`func_0157DA`); once it has no hits left it is beaten
//! (`func_015432`, `t_burst.rs`).  Each routine sets the one of the
//! next frame by writing `OBJ_ROUTINE` (and most fall into it at
//! once).

use super::t_common::*;
use crate::{Game, Regs};

/// The object's routines (code addresses).
const LANDED: u32 = 0x01537A;
const DROPPING: u32 = 0x015394;
const THROWING: u32 = 0x0153B6;
const WALK: u32 = 0x015406;
const SHOOT: u32 = 0x0154EA;
const JUMP: u32 = 0x015766;
const JUMPING: u32 = 0x0157A4;
const JUMP_LANDED: u32 = 0x0157AC;
const JUMP_END: u32 = 0x0157BA;
const CALL: u32 = 0x0157DA;
const CALL_HELD: u32 = 0x01582C;
const CALL_THROWN: u32 = 0x015854;
const CALL_END: u32 = 0x015874;
/// The routine of the beaten boss (`t_burst.rs`).
const BEATEN: u32 = 0x0159A0;

/// Animations in the ROM.
const ANIM_LANDED: u32 = 0x0EACF0;
const ANIM_DROP_RIGHT: u32 = 0x0EAD0A;
const ANIM_DROP_LEFT: u32 = 0x0EAD14;
const ANIM_SHOOT: u32 = 0x0EAB64;
const ANIM_BEATEN: u32 = 0x0EAB94;

/// The boss's offset as it lands (object 6's `OBJ_OFFSET`, `t_start.rs`).
const HEIGHT: u32 = 0xFFFF_A5DC;
/// The way `func_007438` found (bits 0-1), turned at random on a jump.
const WAY: u32 = 0xFFFF_AABD;
/// The sprite frame `SHOOT` shoots at.
const SHOOT_FRAME: u8 = 5;

impl Game {
    /// `func_015334`: frame 0x28, the object falls (rate up 0x10,
    /// `func_004424`; offset by it, `func_001ABC`); when it is down
    /// (offset not negative), `HEIGHT` 0, a shake (`func_0043A8`, 0x20
    /// frames) and it bounces up (rate -0x100) while `OBJ_26` lasts;
    /// at the last, `ANIM_LANDED` and `LANDED` from the next frame.
    pub fn r_015334(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, 0x28);
        r.d0 = 0x10;
        (r.d0, r.x, _) = self.r_004424(r.d0, r.a4);
        let falling;
        (r.d0, r.x, falling) = self.r_001abc(r.d0, r.a4);
        if !falling {
            self.write8(HEIGHT, 0);
            r.d0 = 0x20;
            (r.d0, r.x) = self.r_0043a8(r.d0, r.d4, r.d5, r.d6);
            if self.t_count_down(r, r.a4 + OBJ_26) {
                self.write32(r.a4 + OBJ_ROUTINE, LANDED);
                self.write32(r.a4 + OBJ_ANIM, ANIM_LANDED);
                r.c = false;
                return;
            }
            self.write16(r.a4 + OBJ_RATE, 0xFF00);
        }
        self.t_place(r);
        self.t_depth_offset(r);
    }

    /// `func_01537A`, `LANDED`: the object's animation; when it ends,
    /// `DROPPING` from the next frame and two `func_0151B2` tasks at
    /// its position's x low bytes 0x68 (way 1, `ANIM_DROP_RIGHT`) and
    /// 0xA8 (way 3, `ANIM_DROP_LEFT`).
    pub fn r_01537a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_place(r);
        self.t_animate(r);
        if !r.c {
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, DROPPING);
        self.r_001542(r.a4);
        for (x_low, anim, way) in [(0x68, ANIM_DROP_RIGHT, 1), (0xA8, ANIM_DROP_LEFT, 3)] {
            self.write8(POSITION_X_LOW, x_low);
            (r.d5, r.d6, r.a5, r.a6, r.x) = self.r_0151b2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            self.write32(r.a6 + TASK_ANIM, anim);
            self.write8(r.a6 + TASK_17, way);
        }
        r.c = false;
    }

    /// `func_015394`, `DROPPING`: once `HELPERS` is 0, frame 0x29,
    /// `THROWING` from the next frame, two tasks thrown
    /// (`func_0152AE`) and `HELPERS` one more.
    pub fn r_015394(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.c = false;
        if self.read8(HELPERS) != 0 {
            return;
        }
        self.write8(r.a5 + SPRITE_FRAME, 0x29);
        self.write32(r.a4 + OBJ_ROUTINE, THROWING);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_0152ae(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        let (helpers, carry) = self.read8(HELPERS).overflowing_add(1);
        self.write8(HELPERS, helpers);
        r.x = carry;
        r.c = carry;
    }

    /// `loc_0153B4`: an `rts`.
    pub fn r_0153b4(&mut self) {}

    /// `func_0153B6`, `THROWING`: once `HELPERS` is 0, frame 0x2A,
    /// `MODE_BIT_6` and `SPARED_BIT` cleared, direction 2, `OBJ_10`
    /// 0x20, and it walks (`loc_0153E0`).
    pub fn r_0153b6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.c = false;
        if self.read8(HELPERS) != 0 {
            return;
        }
        self.write8(r.a5 + SPRITE_FRAME, 0x2A);
        let mode = self.read8(MODE);
        self.write8(MODE, mode & !MODE_BIT_6);
        let spared = self.read8(SPARED);
        self.write8(SPARED, spared & !SPARED_BIT);
        r.d0 = low_byte(r.d0, 2);
        self.write8(r.a4 + OBJ_DIRECTION, 2);
        self.write8(r.a4 + OBJ_NEXT_DIRECTION, 2);
        self.write8(r.a4 + OBJ_10, 0x20);
        r.c = self.r_0153e0(r.a4);
    }

    /// `loc_0153E0`: the object walks (`WALK` from the next frame; kind
    /// 0, its mark clear, `OBJ_26` 3, `OBJ_27` 1, `OBJ_ID` 2).  Gives
    /// back C (clear).
    pub fn r_0153e0(&mut self, a4: u32) -> bool {
        self.write8(a4 + OBJ_KIND, 0);
        self.write32(a4 + OBJ_ROUTINE, WALK);
        self.write8(a4 + OBJ_SAVED, 0x10);
        self.write8(a4 + OBJ_26, 3);
        self.write8(a4 + OBJ_27, 1);
        self.write8(a4 + OBJ_ID, 2);
        false
    }

    /// `func_015406`, `WALK`: the first frame (its mark clear in
    /// `OBJ_SAVED`'s high nibble; then set), object 0's cell kept
    /// (`func_003E44`) and the object's own; then, in this order:
    /// in object 0's row, at random 1 in 2, it shoots (`SHOOT`, at
    /// once); with an entry of `func_007438` in line, at random 1 in
    /// 2, it jumps (`loc_01572A`); in object 0's column with object 0
    /// not above it, at random 1 in 4, it calls (`loc_0157C8`).
    /// Else, and after the first frame, it walks (`loc_015412`).
    pub fn r_015406(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let mark = self.read8(r.a4 + OBJ_SAVED) & 0xF0;
        r.d0 = low_byte(r.d0, mark);
        r.c = false;
        if mark != 0 {
            self.t_walk(r);
            return;
        }
        r.d0 = low_byte(r.d0, mark | 0x10);
        self.write8(r.a4 + OBJ_SAVED, mark | 0x10);
        r.d0 = self.r_003e44();
        (r.d0, r.x) = self.r_0015e6(r.a4);
        let row = self.read8(CELL_Y);
        r.d0 = low_byte(r.d0, row);
        if row == self.read8(KEPT_Y) {
            let n = self.t_random(r) & 7;
            r.d0 = low_byte(r.d0, n);
            if n < 4 {
                self.write8(r.a4 + OBJ_TIMER, 0);
                self.write8(r.a4 + OBJ_STEP, 0);
                self.write32(r.a4 + OBJ_ANIM, ANIM_SHOOT);
                self.write32(r.a4 + OBJ_ROUTINE, SHOOT);
                r.c = false;
                self.r_0154ea(r);
                return;
            }
        }
        (r.d0, r.x, r.c) = self.r_007438(r.d0, r.x);
        if r.c {
            self.write8(WAY, r.d0 as u8);
            if (self.t_random(r) as i8) < 0 {
                self.t_jump(r);
                return;
            }
        }
        (r.d0, r.x) = self.r_0015e6(r.a4);
        let column = self.read8(CELL_X);
        r.d0 = low_byte(r.d0, column);
        let kept_column = self.read8(KEPT_X);
        // C as the last compare leaves it
        r.c = column < kept_column;
        if column == kept_column {
            let kept_row = self.read8(KEPT_Y);
            r.d0 = low_byte(r.d0, kept_row);
            r.c = kept_row < self.read8(CELL_Y);
            if !r.c {
                let n = self.t_random(r) & 3;
                r.d0 = low_byte(r.d0, n);
                r.c = false;
                if n == 0 {
                    self.write8(TURNERS_MADE, 0);
                    self.write32(r.a4 + OBJ_ROUTINE, CALL);
                    self.write8(r.a4 + OBJ_COUNT, 0x30);
                    self.r_0157da(r);
                    return;
                }
            }
        }
        self.t_walk(r);
    }

    /// `loc_01572A`: the jump: `WAY` turned at random (one more, or
    /// three), the object's direction and next direction, kind 8 and
    /// its animation (`func_00130E`), then `JUMP` (at once).
    fn t_jump(&mut self, r: &mut Regs) {
        let way = self.read8(WAY).wrapping_add(1);
        self.write8(WAY, way);
        if (self.t_random(r) as i8) < 0 {
            let way = self.read8(WAY).wrapping_add(2);
            self.write8(WAY, way);
        }
        let way = self.read8(WAY) & 3;
        r.d0 = low_byte(r.d0, way);
        self.write8(r.a4 + OBJ_DIRECTION, way);
        self.write8(r.a4 + OBJ_NEXT_DIRECTION, way);
        self.write8(r.a4 + OBJ_KIND, 8);
        (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.write32(r.a4 + OBJ_ROUTINE, JUMP);
        r.c = false;
        self.r_015766(r);
    }

    /// `loc_015412` on the registers.
    fn t_walk(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_015412(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
    }

    /// `loc_015412`: the object walks (`func_007174`); if that says it
    /// has no hits left (C), it is beaten (`loc_015432`).  Gives back
    /// every register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_015412(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c, ..Regs::default() };
        self.r_007174(&mut r);
        if self.restarted.is_none() && r.c {
            r.c = self.r_015432(r.a4);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `loc_01541C` on the registers.
    fn t_step(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_01541c(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `loc_01541C`: the sprite placed for the object, its depth, its
    /// cell, then `func_006E28`; if it has no hits left (C), it is
    /// beaten (`loc_015432`).  Gives back d0, d1, d2, a0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_01541c(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, ..Regs::default() };
        self.t_place(&mut r);
        self.t_depth_offset(&mut r);
        (r.d0, r.x) = self.r_0015e6(a4);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e28(r.d0, r.d1, r.d2, r.a0, a4, a5);
        if r.c {
            r.c = self.r_015432(a4);
        }
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c)
    }

    /// `loc_015432`: the boss is beaten: `SPARED_BIT` and `MODE_BIT_6`
    /// set, `ANIM_BEATEN` from its start, `BEATEN` from the next frame.
    /// Gives back C (clear).
    pub fn r_015432(&mut self, a4: u32) -> bool {
        let spared = self.read8(SPARED);
        self.write8(SPARED, spared | SPARED_BIT);
        let mode = self.read8(MODE);
        self.write8(MODE, mode | MODE_BIT_6);
        self.write32(a4 + OBJ_ROUTINE, BEATEN);
        self.write32(a4 + OBJ_ANIM, ANIM_BEATEN);
        self.write8(a4 + OBJ_TIMER, 0);
        self.write8(a4 + OBJ_STEP, 0);
        false
    }

    /// `func_0154EA`, `SHOOT`: the object's animation; at its first
    /// pair's frame `SHOOT_FRAME` a shot (`func_01551A`); when it ends,
    /// `WALK` from the next frame.  Then `loc_01541C`.
    pub fn r_0154ea(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_animate(r);
        if r.c {
            self.write32(r.a4 + OBJ_ROUTINE, WALK);
        } else if self.read8(r.a4 + OBJ_TIMER) == 0 && self.read8(r.a5 + SPRITE_FRAME) == SHOOT_FRAME {
            (r.d4, r.d5, r.d6, r.a4, r.a5, r.a6) = self.r_01551a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
        self.t_step(r);
    }

    /// `func_015766`, `JUMP`: the object's animation; at its fifth pair
    /// (`OBJ_STEP` 4, its first frame) it jumps: rate -0x200, `OBJ_26`
    /// 0x100 (a word), offset 0, `func_004FD0`, `JUMPING` from the next
    /// frame and `JUMP_LANDED` kept.  Then `loc_01541C`.
    pub fn r_015766(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_animate(r);
        if self.read8(r.a4 + OBJ_TIMER) == 0 && self.read8(r.a4 + OBJ_STEP) == 4 {
            self.write16(r.a4 + OBJ_RATE, 0xFE00);
            self.write16(r.a4 + OBJ_26, 0x100);
            self.write8(r.a4 + OBJ_OFFSET, 0);
            (r.d0, r.x, r.c) = self.r_004fd0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            self.write32(r.a4 + OBJ_ROUTINE, JUMPING);
            self.write32(r.a4 + OBJ_SAVED, JUMP_LANDED);
        }
        self.t_step(r);
    }

    /// `loc_0157A0`: `loc_01541C`.  Gives back d0, d1, d2, a0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0157a0(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        self.r_01541c(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5)
    }

    /// `func_0157A4`, `JUMPING`: the object's animation, then the jump
    /// itself (`func_004E5C`), which puts `JUMP_LANDED` back when it
    /// lands.
    pub fn r_0157a4(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.t_animate(regs);
        self.r_004e5c(regs);
    }

    /// `func_0157AC`, `JUMP_LANDED`: a shake (`func_0043A8`, 0x40
    /// frames), then `JUMP_END` (at once).
    pub fn r_0157ac(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = 0x40;
        (r.d0, r.x) = self.r_0043a8(r.d0, r.d4, r.d5, r.d6);
        self.write32(r.a4 + OBJ_ROUTINE, JUMP_END);
        r.c = false;
        self.r_0157ba(r);
    }

    /// `func_0157BA`, `JUMP_END`: the object's animation, then
    /// `loc_01541C`; when it ends, kind 0 and it walks (`loc_0153E0`).
    pub fn r_0157ba(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_animate(r);
        if !r.c {
            self.t_step(r);
            return;
        }
        self.write8(r.a4 + OBJ_KIND, 0);
        r.c = self.r_0153e0(r.a4);
    }

    /// `func_0157DA`, `CALL`: after 0x30 frames, the first time, frame
    /// 0x29, the two objects of `func_015884` (d4, a4, d5, a5 back to
    /// this one) and 0x30 frames more; the second, its mark 0xF in
    /// `OBJ_SAVED`'s high nibble, kind 4, `CALL_HELD` (at once).  Each
    /// frame of the wait `loc_01541C`.
    pub fn r_0157da(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.t_count_down(r, r.a4 + OBJ_COUNT) {
            self.t_step(r);
            return;
        }
        r.c = false;
        if self.read8(TURNERS_MADE) != 0 {
            self.write8(r.a4 + OBJ_SAVED, 0xF0);
            self.write8(r.a4 + OBJ_KIND, 4);
            self.write32(r.a4 + OBJ_ROUTINE, CALL_HELD);
            self.r_01582c(r);
            return;
        }
        let (made, carry) = self.read8(TURNERS_MADE).overflowing_add(1);
        self.write8(TURNERS_MADE, made);
        r.x = carry;
        r.c = carry;
        (r.d4, r.d5) = self.r_015884(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.t_reload_object(r);
        self.t_reload_sprite(r);
        self.write8(r.a4 + OBJ_COUNT, 0x30);
        self.write8(r.a5 + SPRITE_FRAME, 0x29);
        r.c = false;
        self.t_step(r);
    }

    /// `func_01582C`, `CALL_HELD`: while its mark is set (`func_007174`
    /// clears it), it walks (`loc_015412`); then frame 0x29,
    /// `HELPERS` 1, `CALL_THROWN` from the next frame and two tasks
    /// thrown (`func_0152AE`).
    pub fn r_01582c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let mark = self.read8(r.a4 + OBJ_SAVED) & 0xF0;
        r.d0 = low_byte(r.d0, mark);
        r.c = false;
        if mark != 0 {
            self.t_walk(r);
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, CALL_THROWN);
        self.write8(r.a5 + SPRITE_FRAME, 0x29);
        self.write8(HELPERS, 1);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_0152ae(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_015854`, `CALL_THROWN`: `loc_01541C` until `HELPERS` is 0;
    /// then frame 0x2A and 0x10 frames of `CALL_END` (at once).
    /// No script runs it (checked by reading).
    pub fn r_015854(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(HELPERS) != 0 {
            self.t_step(r);
            return;
        }
        self.write8(r.a4 + OBJ_COUNT, 0x10);
        self.write32(r.a4 + OBJ_ROUTINE, CALL_END);
        self.write8(r.a5 + SPRITE_FRAME, 0x2A);
        r.c = false;
        self.r_015874(r);
    }

    /// `func_015874`, `CALL_END`: `loc_01541C` for the count's frames,
    /// then it walks (`loc_0153E0`).
    /// No script runs it (checked by reading).
    pub fn r_015874(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.t_count_down(r, r.a4 + OBJ_COUNT) {
            r.c = self.r_0153e0(r.a4);
            return;
        }
        self.t_step(r);
    }
}
