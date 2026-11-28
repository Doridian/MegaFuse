//! The two objects the boss of `t_boss.rs` calls (`func_015884`,
//! 0x15884-0x159A0): each flies its angle (`OBJ_NEXT_DIRECTION`, an
//! angle for `func_00410C`), turning toward object 0 every 16 frames
//! for a while, and then is gone.

use super::t_common::*;
use crate::{Game, Regs};

/// The objects' routines (code addresses).
const FLY: u32 = 0x0158F6;
const TURN: u32 = 0x015948;
const LAST: u32 = 0x015992;

/// The objects (`func_001CFA`'s indexes) and their animations (tables
/// in the ROM, `D_015040`, `D_015020`).
const FIRST_INDEX: u8 = 7;
const SECOND_INDEX: u8 = 8;
const FIRST_ANIMS: u32 = 0x015040;
const SECOND_ANIMS: u32 = 0x015020;

/// Two points (words): `func_0041D0` gives the angle of A - B.
const A_X: u32 = 0xFFFF_9EE6;
const A_Y: u32 = 0xFFFF_9EE8;
const B_X: u32 = 0xFFFF_9EFA;
const B_Y: u32 = 0xFFFF_9EFE;
/// Object 0's position (words; `OBJ_X`, `OBJ_Y` of the object at
/// 0xFFA432).
const OBJECT_0_X: u32 = 0xFFFF_A44A;
const OBJECT_0_Y: u32 = 0xFFFF_A44C;

impl Game {
    /// `func_015884`: 0x1C to the left and to the right of the object
    /// at a4 (x - 0x1C, then + 0x38), the objects `FIRST_INDEX`
    /// (`FIRST_ANIMS`) and `SECOND_INDEX` (`SECOND_ANIMS`), each
    /// `func_0158CA`; sound 0x65.
    /// Gives back d4 and d5.
    #[allow(clippy::too_many_arguments)]
    pub fn r_015884(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32) {
        self.r_001542(a4);
        let x = self.read16(POSITION_X).wrapping_sub(0x1C);
        self.write16(POSITION_X, x);
        let (d4, a4, x) = self.r_001cfa(low_byte(d4, FIRST_INDEX));
        self.write32(OBJ_INIT_ANIMS, FIRST_ANIMS);
        let (d5, a5) = self.r_0158ca(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        let (d4, a4, _) = self.r_001cfa(low_byte(d4, SECOND_INDEX));
        let (x, carry) = self.read16(POSITION_X).overflowing_add(0x38);
        self.write16(POSITION_X, x);
        self.write32(OBJ_INIT_ANIMS, SECOND_ANIMS);
        let (d5, _) = self.r_0158ca(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, carry);
        self.r_0034b6(low_byte(d0, 0x65));
        (d4, d5)
    }

    /// `func_0158CA`: the object at a4 set up at `POSITION_X`,
    /// `POSITION_Y` (`func_0012A0`) running `FLY`: `OBJ_10` 0xFF, low
    /// byte 0x21, angle 0x80, 0x20 frames.  Gives back d5 and a5 (its
    /// sprite record).
    #[allow(clippy::too_many_arguments)]
    pub fn r_0158ca(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, x: bool) -> (u32, u32) {
        self.write32(OBJ_INIT_ROUTINE, FLY);
        let (_, d5, a5, _) = self.r_0012a0(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        self.write8(a4 + OBJ_10, 0xFF);
        self.write8(a5 + SPRITE_FLAGS, 0x81);
        self.write8(a4 + OBJ_0F, 0x21);
        self.write8(a4 + OBJ_NEXT_DIRECTION, 0x80);
        self.write8(a4 + OBJ_26, 0x20);
        (d5, a5)
    }

    /// `func_0158F6`, `FLY`: the object speeds up (its rate 0x10 more,
    /// `func_004424`) and flies (`loc_015904`) for `OBJ_26`'s frames;
    /// then `TURN` (at once).
    pub fn r_0158f6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.t_count_down(r, r.a4 + OBJ_26) {
            self.write32(r.a4 + OBJ_ROUTINE, TURN);
            r.c = false;
            self.r_015948(r);
            return;
        }
        r.d0 = 0x10;
        (r.d0, r.x, _) = self.r_004424(r.d0, r.a4);
        self.t_fly(r);
    }

    /// `loc_015904` on the registers.
    fn t_fly(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_015904(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.x);
    }

    /// `loc_015904`: the object moves at its angle (`OBJ_NEXT_DIRECTION`,
    /// `func_00410C`) at its rate (`SPEED`), `func_0034DE` with 8 at
    /// it, its kind the angle / 32 with its animation (`func_00130E`,
    /// `func_001336`), the sprite placed and its depth.  Gives back d0,
    /// d1, d2, d5, a0, a5, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_015904(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, x, ..Regs::default() };
        let rate = self.read8(a4 + OBJ_RATE_LOW);
        self.write8(SPEED_LOW, rate);
        let rate = self.read8(a4 + OBJ_RATE);
        self.write8(SPEED, rate);
        let angle = self.read8(a4 + OBJ_NEXT_DIRECTION);
        r.d0 = low_byte(r.d0, angle);
        (r.d0, r.d1, r.d2, r.x) = self.r_00410c(r.d0, r.d1, r.d2, r.x);
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, a4);
        (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, a4);
        r.d0 = 8;
        (r.d1, r.d2, r.x) = self.r_0034de(r.d0, r.d1, r.d2, a4);
        let kind = self.read8(a4 + OBJ_NEXT_DIRECTION) >> 5;
        r.d0 = low_byte(r.d0, kind);
        self.write8(a4 + OBJ_KIND, kind);
        (r.a0, r.x, r.c) = self.r_00130e(r.d0, a4);
        self.t_animate(&mut r);
        self.t_place(&mut r);
        self.t_depth(&mut r);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c)
    }

    /// `func_015948`, `TURN`: every 16 frames of `OBJ_26`'s the angle
    /// a step of 0x10 toward object 0 (`func_0041D0`, `func_0042FE`);
    /// it flies (`loc_015904`); at the end, `LAST` (at once).
    pub fn r_015948(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.t_count_down(r, r.a4 + OBJ_26) {
            self.write32(r.a4 + OBJ_ROUTINE, LAST);
            r.c = false;
            self.r_015992(r);
            return;
        }
        let phase = self.read8(r.a4 + OBJ_26) & 0x0F;
        r.d0 = low_byte(r.d0, phase);
        r.c = false;
        if phase != 0 {
            self.t_fly(r);
            return;
        }
        let x = self.read16(r.a4 + OBJ_X);
        self.write16(B_X, x);
        let y = self.read16(r.a4 + OBJ_Y);
        self.write16(B_Y, y);
        let x = self.read16(OBJECT_0_X);
        self.write16(A_X, x);
        let y = self.read16(OBJECT_0_Y);
        self.write16(A_Y, y);
        r.d0 = self.r_0041d0(r.d0, r.d1);
        r.d2 = low_byte(r.d2, r.d0 as u8);
        r.d1 = low_byte(r.d1, self.read8(r.a4 + OBJ_NEXT_DIRECTION));
        r.d0 = 0x10;
        (r.d0, r.x) = self.r_0042fe(r.d0, r.d1, r.d2);
        self.write8(r.a4 + OBJ_NEXT_DIRECTION, r.d0 as u8);
        r.c = false;
        self.t_fly(r);
    }

    /// `loc_015986`: `loc_015904`.  Gives back d0, d1, d2, d5, a0, a5,
    /// X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_015986(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        self.r_015904(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, x)
    }

    /// `func_015992`, `LAST`: it flies (`loc_015904`) for `OBJ_26`'s
    /// frames; then the object is gone (`OBJ_ID` 0, its sprite record
    /// freed, `func_000890`).
    /// No script runs it (checked by reading).
    pub fn r_015992(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.t_count_down(r, r.a4 + OBJ_26) {
            self.t_fly(r);
            return;
        }
        self.write8(r.a4 + OBJ_ID, 0);
        r.c = self.r_000890();
    }
}
