//! The four tasks `func_019A48` sends out round object 6 (`x_object.rs`;
//! scripts stage-3-boss and exit-3-boss): each goes round its centre
//! (`func_019B4E`) with its radius growing to `OUT_END`
//! (`func_019ACE`), then with another animation to `ROUND_END`
//! (`func_019B0C`), turning by a random angle each time.  Between, on
//! a cell of the map: a cell of 1 plays an animation (`func_019C50`); a
//! cell of 2-6 near object 0 (`func_004348`) hurts the object by
//! `func_002D3E` (unless it is gone); either way, and on a cell of 7 or
//! more, the task ends with an animation (`func_019C8A`, `func_019C9E`).

use crate::{Game, Regs};

/// The index of the task running (`task.rs`).
const CURRENT_TASK: u32 = 0xFFFF_80EE;
/// The sprite record kept (index, address).
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;
/// Byte 0 of object 6 (0xFFA5B2): while it isn't 0 the tasks stay.
const OBJECT_6_STATE: u32 = 0xFFFF_A5B2;
/// Object 0's position (words, x then y) and its byte 0x32 (0xFFA464),
/// whose sign picks the object `func_002D3E` takes (0 if set, else 5).
const OBJECT_0_X: u32 = 0xFFFF_A44A;
const OBJECT_0_Y: u32 = 0xFFFF_A44C;
const OBJECT_0_32: u32 = 0xFFFF_A464;
/// An object's byte 0x10: negative, `func_002D3E` isn't called.
const OBJ_10: u32 = 0x10;
/// Point A (words, `d_angle.rs`).
const A_X: u32 = 0xFFFF_9EE6;
const A_Y: u32 = 0xFFFF_9EE8;

/// The task's fields: its first sprite record's frame (the record a5),
/// its position's low bytes (`x_object.rs`'s `TASK_X`, `TASK_Y` words),
/// the animation's counters, its angle and radius, its centre (bytes,
/// x then y) and its animation (a long, `anim.rs`).
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y_LOW: u32 = 0x11;
const TASK_ANIM_12: u32 = 0x12;
const TASK_ANIM_13: u32 = 0x13;
const TASK_ANGLE: u32 = 0x16;
const TASK_RADIUS: u32 = 0x17;
const TASK_CENTRE_X: u32 = 0x19;
const TASK_CENTRE_Y: u32 = 0x1A;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's frame (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;

/// The radius: what it grows by and where it stops, out and round; the
/// random turn (0-0x1F, less 0x10).
const OUT_STEP: u8 = 5;
const OUT_END: u8 = 0x64;
const ROUND_STEP: u8 = 2;
const ROUND_END: u8 = 0x14;
const TURN_MASK: u8 = 0x1F;
const TURN_LESS: u8 = 0x10;
const OUT_FRAME: u8 = 0x68;

/// Cells of the map (`func_00167A`): the one that plays the animation,
/// and the first of those the task ends on.
const CELL_1: u8 = 1;
const CELL_7: u8 = 7;
/// `func_004348`'s argument (how near to A).
const NEAR: u32 = 7;
/// `func_0007D2`'s argument on a cell of 1.
const ON_CELL_1: u8 = 0xF0;
/// Z80 command at the end (`func_0034B6`).
const END_SOUND: u8 = 0x69;

/// Animations (ROM): round, on a cell of 1, after it, at the end.
const ANIM_ROUND: u32 = 0x0DE2C6;
const ANIM_CELL_1: u32 = 0x0DE2CC;
const ANIM_AFTER: u32 = 0x0DE2EE;
const ANIM_END: u32 = 0x0DE2E6;

/// Where the task goes on (resume addresses), and the return addresses
/// `func_019B4E` drops.
const ROUND: u32 = 0x019B0C;
const AFTER_CELL_1: u32 = 0x019C50;
const ENDING: u32 = 0x019C9E;
const FROM_OUT: u32 = 0x019ADA;
const FROM_ROUND: u32 = 0x019B16;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// The task's radius grown by `step`; at `end` it goes back to 0 and
    /// the centre is the task's place (true), else the cell's checks
    /// follow (false).  d0's low byte the radius, X the add's carry.
    fn yc_grow(&mut self, r: &mut Regs, step: u8, end: u8) -> bool {
        let (radius, carry) = self.read8(r.a6 + TASK_RADIUS).overflowing_add(step);
        r.d0 = low_byte(r.d0, radius);
        r.x = carry;
        if radius < end {
            self.write8(r.a6 + TASK_RADIUS, radius);
            r.c = false;
            return false;
        }
        self.write8(r.a6 + TASK_RADIUS, 0);
        let x = self.read8(r.a6 + TASK_X_LOW);
        self.write8(r.a6 + TASK_CENTRE_X, x);
        let y = self.read8(r.a6 + TASK_Y_LOW);
        self.write8(r.a6 + TASK_CENTRE_Y, y);
        true
    }

    /// The cell's checks (0x19BB4) for the registers in `r`.
    fn yc_cell_checks(&mut self, r: &mut Regs) {
        let t = self.r_019bb4(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x, r.c) = t;
    }

    /// `func_019ACE` (a task's routine): its sprite's frame `OUT_FRAME`,
    /// placed round its centre (`func_019B4E`, which may end it); the
    /// radius grows by `OUT_STEP`: the cell's checks (0x19BB4); at
    /// `OUT_END`, the animation `ANIM_ROUND` and a frame on
    /// (`func_019B0C`).
    pub fn r_019ace(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, OUT_FRAME);
        let dropped;
        (r.d0, r.d1, r.d5, r.d6, r.a5, r.x, r.c, dropped) = self.r_019b4e(r.d0, r.d1, r.d3, r.d5, r.d6, r.a5, r.a6, FROM_OUT);
        if dropped {
            return;
        }
        if !self.yc_grow(r, OUT_STEP, OUT_END) {
            self.yc_cell_checks(r);
            return;
        }
        self.write32(r.a6 + TASK_ANIM, ANIM_ROUND);
        self.yc_yield(r, ROUND);
    }

    /// `func_019B0C` (a task's routine): its animation a frame on
    /// (`func_0013D6`), placed round its centre (`func_019B4E`, which may
    /// end it); the radius grows by `ROUND_STEP`: the cell's checks
    /// (0x19BB4); at `ROUND_END` the angle turns by a random
    /// (`func_001BE2`) -0x10 to 0xF.
    pub fn r_019b0c(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        let dropped;
        (r.d0, r.d1, r.d5, r.d6, r.a5, r.x, r.c, dropped) = self.r_019b4e(r.d0, r.d1, r.d3, r.d5, r.d6, r.a5, r.a6, FROM_ROUND);
        if dropped {
            return;
        }
        if !self.yc_grow(r, ROUND_STEP, ROUND_END) {
            self.yc_cell_checks(r);
            return;
        }
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        let turn = (r.d0 as u8 & TURN_MASK).wrapping_sub(TURN_LESS);
        r.d0 = low_byte(r.d0, turn);
        let (angle, carry) = self.read8(r.a6 + TASK_ANGLE).overflowing_add(turn);
        self.write8(r.a6 + TASK_ANGLE, angle);
        r.x = carry;
        r.c = carry;
    }

    /// 0x19BB4, the cell's checks (`func_0015F8`, `func_00167A`): a
    /// cell of 1: the animation `ANIM_CELL_1` (counters cleared),
    /// `func_0007D2` with 0xF0, `func_001550`, placed (`func_001B7E`),
    /// the radius 1 if `func_005A76` says so (C) and a frame on
    /// (`func_019C50`); a cell under 7: object 0's position as A and,
    /// if it is near (`func_004348`), the object `func_002D3E` takes
    /// (unless its `OBJ_10` is negative) and the end (`func_019C8A`); a
    /// cell of 7 or more: the end.  Gives back d0, d1, d2, d4, d5, d6,
    /// a0, a4, a5, a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_019bb4(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        r.d0 = self.r_0015f8(a6);
        (r.a0, r.x) = self.r_00167a();
        let cell = self.read8(r.a0);
        r.d0 = low_byte(r.d0, cell);
        if cell == CELL_1 {
            self.write8(r.a6 + TASK_ANIM_12, 0);
            self.write8(r.a6 + TASK_ANIM_13, 0);
            self.write32(r.a6 + TASK_ANIM, ANIM_CELL_1);
            r.d0 = low_byte(r.d0, ON_CELL_1);
            r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
            (r.d0, r.x) = self.r_001550(r.d0, r.a6);
            (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
            (r.d6, r.a6, r.x, _, r.c) = self.r_005a76(r.d2, r.d6, r.a6);
            if r.c {
                self.write8(r.a6 + TASK_RADIUS, 1);
            }
            self.yc_yield(&mut r, AFTER_CELL_1);
            return (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x, r.c);
        }
        if cell < CELL_7 {
            self.r_001a20(r.a6);
            self.r_001a28(r.a6);
            let x = self.read16(OBJECT_0_X);
            self.write16(A_X, x);
            let y = self.read16(OBJECT_0_Y);
            self.write16(A_Y, y);
            r.d0 = NEAR;
            (r.d0, r.x, r.c) = self.r_004348(r.d0);
            if !r.c {
                return (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x, r.c);
            }
            r.d4 = if (self.read8(OBJECT_0_32) as i8) < 0 { low_word(r.d4, 0) } else { 5 };
            (r.d4, r.a4, r.x) = self.r_001cfa(r.d4);
            if (self.read8(r.a4 + OBJ_10) as i8) < 0 {
                r.c = false;
                return (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x, r.c);
            }
            r.d0 = low_byte(r.d0, r.d4 as u8);
            (r.d0, r.a0) = self.r_002d3e(r.d0, r.d4, r.d5, r.a0);
            r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
            (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        }
        self.write32(r.a6 + TASK_ANIM, ANIM_END);
        let t = self.r_019c8a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        (r.d0, r.d1, r.d2, r.d5, r.d6, r.a0, r.a5, r.x, r.c) = t;
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_019C50` (a task's routine): still on a cell of 1 with
    /// `OBJECT_6_STATE` set: its animation a frame on (`func_0013D6`,
    /// the sprite record kept as a5); else the animation `ANIM_AFTER`
    /// and the end (`func_019C8A`).
    pub fn r_019c50(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        r.d0 = self.r_0015f8(r.a6);
        (r.a0, r.x) = self.r_00167a();
        let cell = self.read8(r.a0);
        r.d0 = low_byte(r.d0, cell);
        if cell == CELL_1 && self.read8(OBJECT_6_STATE) != 0 {
            r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
            r.a5 = self.read32(SLOT_ADDR);
            (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
            return;
        }
        self.write32(r.a6 + TASK_ANIM, ANIM_AFTER);
        let t = self.r_019c8a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        (r.d0, r.d1, r.d2, r.d5, r.d6, r.a0, r.a5, r.x, r.c) = t;
    }

    /// `func_019C8A` (0x19C8A, the task's end): the animation's counters
    /// cleared, the Z80 command `END_SOUND` (`func_0034B6`) and a frame
    /// on (`func_019C9E`).  Gives back d0, d1, d2, d5, d6, a0, a5, X and
    /// C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_019c8a(&mut self, d0: u32, d1: u32, d2: u32, _d3: u32, _d4: u32, d5: u32, d6: u32, _d7: u32, a0: u32, _a1: u32, _a2: u32, _a3: u32, _a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.write8(a6 + TASK_ANIM_12, 0);
        self.write8(a6 + TASK_ANIM_13, 0);
        let d0 = low_byte(d0, END_SOUND);
        self.r_0034b6(d0);
        let (x, c, _) = self.r_000522(d6, ENDING);
        (d0, d1, d2, d5, d6, a0, a5, x, c)
    }

    /// `func_019C9E` (a task's routine): its animation a frame on
    /// (`func_0013D6`); at its end the task ends (`func_00121A`).
    pub fn r_019c9e(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        }
    }

    /// `func_019CAA`: an `rts`.
    pub fn r_019caa(&mut self) {}
}
