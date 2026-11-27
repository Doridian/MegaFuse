//! Two of an object's steps: a frame of it (its direction, animation,
//! and what follows), and a check through `func_003E90`.

use crate::{Game, Regs};

/// An object's fields (`anim.rs`): the direction it is to take (a
/// negative byte: none), its direction (bits 0-1; bit 7 kept here).
const OBJ_NEXT_DIRECTION: u32 = 0x13;
const OBJ_DIRECTION: u32 = 0x14;
const KEPT_BITS: u8 = 0x80;

/// A byte by the low byte `func_003F1A` leaves in d0 (D_009C70), to
/// `BYTE_AC8B`.
const TABLE_9C70: u32 = 0x009C70;
const BYTE_AC8B: u32 = 0xFFAC8B;

impl Game {
    /// `func_004096`: a frame of the object at a4: `func_00398C`, its
    /// direction from `OBJ_NEXT_DIRECTION` unless that is negative, its
    /// animation by it (`func_00130E`) a frame on (`func_001336`), then
    /// `func_001B32`, `func_001B50` and `func_006E7C`.
    #[allow(clippy::too_many_arguments)]
    pub fn r_004096(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (d0, d1, d4, obj) = self.r_00398c(d0, d1, d4, a4, a5);
        let next = self.read8(obj + OBJ_NEXT_DIRECTION);
        let d0 = (d0 & 0xFFFF_FF00) | u32::from(next);
        if next & 0x80 == 0 {
            let kept = self.read8(obj + OBJ_DIRECTION) & KEPT_BITS;
            self.write8(obj + OBJ_DIRECTION, kept | next);
        }
        let (_, x, _) = self.r_00130e(d0, obj);
        let (_, d1, d2, d5, a0, a5, _, _, _) = self.r_001336(d0, d1, d2, d5, obj, x);
        let (d0, d1, _, _) = self.r_001b32(d1, obj, a5);
        self.r_001b50(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, obj, a5);
        let (d0, d1, d2, a0, x, c) = self.r_006e7c(d1, d2, a0, obj, a5);
        (d0, d1, d2, d4, d5, a0, obj, a5, x, c)
    }

    /// `func_0040C0`: `func_003E44`, `func_0015E6`, then `func_003E90`
    /// with `d0`; if that sets C, `func_003AEA`, `func_0015E6` and
    /// `func_003F1A` again, `BYTE_AC8B` from `TABLE_9C70` by the low
    /// byte it leaves in d0, and `func_003B28` (C set).  Gives back
    /// d0, d1 (as `func_003E90` or `func_003F1A` leaves it), N and C.
    pub fn r_0040c0(&mut self, d0: u32, d1: u32, a0: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        self.r_003e44();
        self.r_0015e6(a4);
        let (d1, n, c) = self.r_003e90(d0, d1);
        if !c {
            return (d0, d1, n, false);
        }
        self.r_003aea(a4);
        let (d0, x) = self.r_0015e6(a4);
        let mut regs = Regs { d0, d1, a0, a4, a5, x, n, c, ..Regs::default() };
        self.r_003f1a(&mut regs);
        let index = regs.d0 as u8 as i8 as i16 as u32;
        let b = self.read8(TABLE_9C70.wrapping_add(index));
        self.write8(BYTE_AC8B, b);
        let d1 = regs.d1;
        let (d0, n) = self.r_003b28(d1, a4, a5);
        (d0, d1, n, true)
    }
}
