//! Objects in a ring around object 6, 0x13508-0x13750: `func_013508`
//! makes one (and, through its first routine `func_01354C`, the next
//! ones, 4 angles on each frame) at angle `d0`, numbered from
//! `PIECES`.  Each keeps on the circle of radius 0x40 about object 6
//! (`func_0135E8`) turned by the circle's angle; when `ORBIT_STATE`
//! comes to its number it bursts (`func_013638`) into three tasks of
//! `func_0136F6` that fly out until off the screen or on a cell that
//! is 1.  It is gone when hit (`func_006E28`, `func_0135D0`).
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read.

use crate::{Game, Regs};

/// An object's fields: byte 0 (in use), its routine, the routine its
/// set-up gives (`OBJ_INIT_2`), its x's high byte's flags (0xF) and
/// 0x11, its position (words), its animation (a long), its number and
/// angle.
const OBJ_USED: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_0F: u32 = 0x0F;
const OBJ_11: u32 = 0x11;
const OBJ_X: u32 = 0x18;
const OBJ_Y: u32 = 0x1A;
const OBJ_ANIM: u32 = 0x20;
const OBJ_NUMBER: u32 = 0x26;
const OBJ_ANGLE: u32 = 0x27;
/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
/// A task's fields: its x's and y's high bytes (not 0: off the
/// screen), and where it keeps its velocity: x at 0x20 (fraction at
/// 0x21), y at 0x13 (fraction at 0x12).
const TASK_X_HIGH: u32 = 0x0E;
const TASK_Y_HIGH: u32 = 0x10;
const TASK_VELOCITY_X: u32 = 0x20;
const TASK_VELOCITY_X_FRACTION: u32 = 0x21;
const TASK_VELOCITY_Y: u32 = 0x13;
const TASK_VELOCITY_Y_FRACTION: u32 = 0x12;

/// The routine `func_0012A0` gives the object it sets up.
const OBJ_INIT_2: u32 = 0xFF80D0;
/// The pieces made so far (each takes the next number).
const PIECES: u32 = 0xFF9660;
/// The angle a piece is drawn at (`func_0135E8`).
const ANGLE: u32 = 0xFFAC8A;
/// The circle's angle (its high byte) and state (`rr_orbit.rs`).
const ORBIT_ANGLE: u32 = 0xFF965C;
const ORBIT_STATE: u32 = 0xFFAD56;
/// Object 6's position (words): the middle of the circle.
const OBJECT_6_X: u32 = 0xFFA5CA;
const OBJECT_6_Y: u32 = 0xFFA5CC;
const RADIUS: u32 = 0x40;
/// The burst's angles: the next to make, the last, and the one left
/// out (the way the piece is from the middle).
const BURST_ANGLE: u32 = 0xFFAC95;
const BURST_END: u32 = 0xFFAC94;
const BURST_SKIP: u32 = 0xFFAC93;
/// The burst's frames by quarter (D_0136F2).
const BURST_FRAMES: u32 = 0x0136F2;
/// The speed `func_00410C` makes a velocity of (a word), and the
/// velocity (bytes: pixels and fraction, x then y; `a_motion.rs`).
const SPEED: u32 = 0xFF9F00;
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_X_FRACTION: u32 = 0xFF9EC5;
const VELOCITY_Y: u32 = 0xFF9EC6;
const VELOCITY_Y_FRACTION: u32 = 0xFF9EC7;
/// The running object (index, address) and the sprite record of the
/// routine being run (index, the one after, address).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_8A59: u32 = 0xFF8A59;
const SLOT_ADDR: u32 = 0xFF8A5A;

/// Routines and animations.
const ROUTINE_01354C: u32 = 0x01354C;
const ROUTINE_013584: u32 = 0x013584;
const ROUTINE_0135D0: u32 = 0x0135D0;
const ROUTINE_0136F6: u32 = 0x0136F6;
const ANIM_GONE: u32 = 0x0E84F8;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_013508`: a piece at angle `d0` (its low byte): the first
    /// free object from 6 on (`func_001456`, not checked) set up with a
    /// sprite record (`func_0012A0`, routine `func_01354C`; flags 0x81,
    /// bytes 0xF 0x21 and 0x11 3), its number the next of `PIECES`.
    /// Gives back d0 (the number), d4, d5, a4, a5 (the object and its
    /// sprite record), X and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_013508(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let angle = d0;
        let (free, _, _) = self.r_001456(d0);
        let (d4, a4, x) = self.r_001cfa(low_byte(d4, free as u8));
        self.write32(OBJ_INIT_2, ROUTINE_01354C);
        let (_, d5, a5, _) = self.r_0012a0(free, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        self.write8(a5 + SPRITE_FLAGS, 0x81);
        self.write8(a4 + OBJ_0F, 0x21);
        self.write8(a4 + OBJ_11, 3);
        self.write8(a4 + OBJ_ANGLE, angle as u8);
        let (number, carry) = self.read8(PIECES).overflowing_add(1);
        self.write8(a4 + OBJ_NUMBER, number);
        self.write8(PIECES, number);
        (low_byte(angle, number), d4, d5, a4, a5, carry, false)
    }

    /// `func_01354C`, a piece's first routine: drawn at its angle
    /// (`func_0135E8`), the angle 4 on; when that comes to a multiple
    /// of 0x40 it goes on as `func_013584`, and (unless the angle is 0)
    /// the next piece is made at it (`func_013508`).
    pub fn r_01354c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let angle = self.read8(r.a4 + OBJ_ANGLE);
        self.write8(ANGLE, angle);
        (r.d0, r.d1, r.d5, r.a5) = self.r_0135e8(r.d0, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        let (angle, carry) = self.read8(r.a4 + OBJ_ANGLE).overflowing_add(4);
        self.write8(r.a4 + OBJ_ANGLE, angle);
        r.x = carry;
        r.d0 = low_byte(r.d0, angle & 0x3F);
        r.c = false;
        if angle & 0x3F != 0 {
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_013584);
        let angle = self.read8(r.a4 + OBJ_ANGLE);
        r.d0 = low_byte(r.d0, angle);
        if angle == 0 {
            return;
        }
        (r.d0, r.d4, r.d5, r.a4, r.a5, r.x, r.c) = self.r_013508(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_013584`, a piece's routine: drawn at its angle plus the
    /// circle's (`func_0135E8`); when `ORBIT_STATE` is its number it
    /// bursts (`func_013638`, the Z80 getting 0x70).  When hit
    /// (`func_006E28`) it is one piece less and goes on as
    /// `func_0135D0` with the animation `ANIM_GONE`.
    pub fn r_013584(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let angle = self.read8(r.a4 + OBJ_ANGLE).wrapping_add(self.read8(ORBIT_ANGLE));
        r.d0 = low_byte(r.d0, angle);
        self.write8(ANGLE, angle);
        (r.d0, r.d1, r.d5, r.a5) = self.r_0135e8(r.d0, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        let state = self.read8(ORBIT_STATE);
        r.d0 = low_byte(r.d0, state);
        if state == self.read8(r.a4 + OBJ_NUMBER) {
            (r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6) = self.r_013638(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            r.d0 = low_byte(r.d0, 0x70);
            self.r_0034b6(r.d0);
        }
        (r.d0, r.x) = self.r_0015e6(r.a4);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e28(r.d0, r.d1, r.d2, r.a0, r.a4, r.a5);
        if r.c {
            let (pieces, borrow) = self.read8(PIECES).overflowing_sub(1);
            self.write8(PIECES, pieces);
            r.x = borrow;
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_0135D0);
            self.write32(r.a4 + OBJ_ANIM, ANIM_GONE);
            r.c = false;
        }
    }

    /// `func_0135D0`, a piece's routine: its animation run
    /// (`func_001336`) and its sprite placed; at the end it is gone
    /// (byte 0 cleared, its sprite record freed, `func_000890`).
    pub fn r_0135d0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if r.c {
            self.write8(r.a4 + OBJ_USED, 0);
            r.c = self.r_000890();
            return;
        }
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_0135E8`: the piece at `ANGLE` on the circle about object 6
    /// (`RADIUS`; `func_017780` sine, `func_01777C` cosine), frame 1
    /// (2 in the angle's half with bit 5 set) on the sprite at a5; then
    /// the routine's sprite record placed for it (`func_001B32`,
    /// `func_001B5C`).  Gives back d0, d1, d5 and a5 (the sprite
    /// record).
    #[allow(clippy::too_many_arguments)]
    pub fn r_0135e8(&mut self, d0: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32) {
        self.write8(a5 + SPRITE_FRAME, 1);
        if self.read8(ANGLE) & 0x20 != 0 {
            self.write8(a5 + SPRITE_FRAME, 2);
        }
        let angle = self.read8(ANGLE);
        let d0 = self.r_017780(low_byte(d0, angle), RADIUS, d3);
        let x = (d0 as u16).wrapping_add(self.read16(OBJECT_6_X));
        self.write16(a4 + OBJ_X, x);
        let d0 = self.r_01777c(low_byte(low_word(d0, x), angle), RADIUS, d3);
        let y = (d0 as u16).wrapping_add(self.read16(OBJECT_6_Y));
        self.write16(a4 + OBJ_Y, y);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        let (d0, d1, _, _) = self.r_001b32(RADIUS, a4, a5);
        let (d0, _, _) = self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5);
        (d0, d1, d5, a5)
    }

    /// `func_013638`: the burst: from the angle opposite `ANGLE`'s
    /// mirror on, a quarter turn at a time once round, a task of
    /// `func_0136F6` (depth 0x80, priority 0x42) at the object's
    /// position for each angle but the one that points back at the
    /// middle, with the frame of its quarter (`BURST_FRAMES`, on the
    /// sprite record after the routine's), flags 0x81 and the velocity
    /// at speed 0x300 in its angle (`func_00410C`) kept, moved a frame
    /// and placed.  The running object and the routine's sprite record
    /// are taken back.  Gives back d1, d2, d4, d5, d6, a0, a4, a5, a6.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_013638(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        self.r_001542(a4);
        let start = self.read8(ANGLE).wrapping_neg();
        let mut d0 = low_byte(d0, start);
        self.write8(BURST_ANGLE, start);
        self.write8(BURST_END, start);
        d0 = low_byte(d0, start ^ 0x80);
        self.write8(BURST_SKIP, start ^ 0x80);
        let (mut d1, mut d2, mut d5, mut d6, mut a0, mut a5) = (d1, d2, d5, d6, a0, a5);
        let mut a6;
        loop {
            let made;
            (d5, d6, a5, a6, _, made) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0x80, 0x42, ROUTINE_0136F6);
            if !made {
                let quarter = (self.read8(BURST_ANGLE) >> 5) & 3;
                d0 = low_word(d0, u16::from(quarter));
                a0 = BURST_FRAMES;
                let other = self.read8(SLOT_8A59);
                let x;
                (d5, a5, x) = self.r_000b5a(low_byte(d5, other));
                let frame = self.read8(BURST_FRAMES + u32::from(quarter));
                self.write8(a5 + SPRITE_FRAME, frame);
                self.r_00152a(a6);
                self.write8(a5 + SPRITE_FLAGS, 0x81);
                self.write16(SPEED, 0x300);
                let angle = self.read8(BURST_ANGLE);
                (d0, d1, d2, _) = self.r_00410c(low_byte(d0, angle), d1, d2, x);
                (d0, d1, _, _, _) = self.r_001a88(d0, d1, a6);
                (_, d1, _, _, _) = self.r_001aa2(d0, d1, a6);
                let v = self.read8(VELOCITY_X_FRACTION);
                self.write8(a6 + TASK_VELOCITY_X_FRACTION, v);
                let v = self.read8(VELOCITY_X);
                self.write8(a6 + TASK_VELOCITY_X, v);
                let v = self.read8(VELOCITY_Y_FRACTION);
                self.write8(a6 + TASK_VELOCITY_Y_FRACTION, v);
                let v = self.read8(VELOCITY_Y);
                self.write8(a6 + TASK_VELOCITY_Y, v);
                (d0, _, _) = self.r_001b7e(a5, a6);
            }
            let next = loop {
                let angle = self.read8(BURST_ANGLE).wrapping_add(0x40);
                d0 = low_byte(d0, angle);
                self.write8(BURST_ANGLE, angle);
                if angle != self.read8(BURST_SKIP) {
                    break angle;
                }
            };
            if next == self.read8(BURST_END) {
                break;
            }
        }
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        (d1, d2, d4, d5, d6, a0, a4, a5, a6)
    }

    /// `func_0136F6`, a burst task's routine: it moves by its kept
    /// velocity (`func_001A88`, `func_001AA2`) and ends
    /// (`func_00121A`) once off the screen; otherwise its sprite is
    /// placed, `func_003554` (8) runs, and on a cell that is 1 the
    /// task is stopped there (`func_005E0E`).
    pub fn r_0136f6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let v = self.read8(r.a6 + TASK_VELOCITY_X_FRACTION);
        self.write8(VELOCITY_X_FRACTION, v);
        let v = self.read8(r.a6 + TASK_VELOCITY_X);
        self.write8(VELOCITY_X, v);
        let v = self.read8(r.a6 + TASK_VELOCITY_Y_FRACTION);
        self.write8(VELOCITY_Y_FRACTION, v);
        let v = self.read8(r.a6 + TASK_VELOCITY_Y);
        self.write8(VELOCITY_Y, v);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        let mut off = self.read8(r.a6 + TASK_X_HIGH) != 0;
        if !off {
            (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
            off = self.read8(r.a6 + TASK_Y_HIGH) != 0;
        }
        if off {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        r.d0 = 8;
        (r.d1, r.d2, r.x, r.c) = self.r_003554(r.d0, r.d1, r.d2, r.a6);
        r.d0 = self.r_0015f8(r.a6);
        (r.a0, r.x) = self.r_00167a();
        let cell = self.read8(r.a0);
        r.d0 = low_byte(r.d0, cell);
        r.c = cell < 1;
        if cell == 1 {
            (r.x, _, r.c) = self.r_005e0e(r.d2, r.d6, r.a6);
        }
    }
}
