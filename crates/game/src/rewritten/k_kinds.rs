//! The routines a cell's kind runs: a cell's byte 0x200 on (in the
//! map `func_00167A` finds) has a kind 1-7 in bits 4-6, and
//! `func_005378` calls the kind's routine in `KIND_CALLS`,
//! `func_0053A6` the one in `KIND_CALLS_2`, with d0 the kind.  What
//! sets them (`func_00BB06`, `func_00BB6E`, `func_00BD54`, from the
//! ROM's tables by stage) and some of them (`func_00BB30`,
//! `func_00BB54`, `func_00BC4E`, `func_00BD6C`); and other routines of
//! the same tables (`func_00BC72`, `func_00BCC2`, `func_00BC70`).

use crate::{Game, Regs};

/// The kinds' routines (longs, kinds 1-7): `func_005378`'s and
/// `func_0053A6`'s.
const KIND_CALLS: u32 = 0xFFFF_A97A;
const KIND_CALLS_2: u32 = 0xFFFF_A996;
/// Kind 7's entries.
const KIND_7_CALL: u32 = KIND_CALLS + 4 * 6;
const KIND_7_CALL_2: u32 = KIND_CALLS_2 + 4 * 6;

/// The routines put in the tables (code addresses).
const ROUTINE_BB30: u32 = 0x00BB30;
const ROUTINE_BB54: u32 = 0x00BB54;
const ROUTINE_BC4E: u32 = 0x00BC4E;
const ROUTINE_BD6C: u32 = 0x00BD6C;
/// An `rts`.
const ROUTINE_52EA: u32 = 0x0052EA;

/// The index of the task running (`task.rs`).
const CURRENT: u32 = 0xFFFF_80EE;
/// A long by kind (`D_005F2A`, in the ROM) `func_00BB54` puts in
/// `PUSH_X` (`b_walk.rs`).
const PUSHES: u32 = 0x005F2A;
const PUSH_X: u32 = 0xFFFF_9EF2;
/// `b_walk.rs`: while this isn't 0, `func_00254C` doesn't try a slide.
const NO_SLIDE: u32 = 0xFFFF_A93E;
/// A position (bytes: the low bytes of two words) `func_00BC72` puts
/// a sprite near.
const NEAR_X_LOW: u32 = 0xFFFF_9F05;
const NEAR_Y_LOW: u32 = 0xFFFF_9F07;

/// A task's fields.
/// Bits 0-2 0: `func_00BB30` and `func_00BD6C` set them (2, 3).
const TASK_STATE: u32 = 0x18;
const STATE_BITS: u8 = 0x07;
/// The kind less 1 (`func_00BB30`, `func_00BC4E`).
const TASK_KIND: u32 = 0x15;

/// A sprite record's fields (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_Y: u32 = 0x0A;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// Kinds 1-4 run `first` and `second`, then `func_00BB94`.
    fn k_kinds(&mut self, regs: &mut Regs, first: u32, second: u32) {
        for kind in 0..4 {
            self.write32(KIND_CALLS + 4 * kind, first);
            self.write32(KIND_CALLS_2 + 4 * kind, second);
        }
        (regs.d0, regs.d1, regs.x, regs.c) = self.r_00bb94(regs.d0);
        regs.unwound = false;
    }

    /// `func_00BB06`: kinds 1-4 run `func_00BB30` and `func_00BB54`;
    /// then `func_00BB94`.
    /// No script runs it (checked by reading).
    pub fn r_00bb06(&mut self, regs: &mut Regs) {
        self.k_kinds(regs, ROUTINE_BB30, ROUTINE_BB54);
    }

    /// `func_00BB6E`: kinds 1-4 run `func_00BC4E` and `func_0052EA`
    /// (an `rts`); then `func_00BB94`.
    /// No script runs it (checked by reading).
    pub fn r_00bb6e(&mut self, regs: &mut Regs) {
        self.k_kinds(regs, ROUTINE_BC4E, ROUTINE_52EA);
    }

    /// `loc_00BB94`: d0's low byte made a cell's byte, kind (bits 0-1)
    /// plus 1 in bits 4-6 and 6 in bits 0-3; d1 1 and C set if bit 2
    /// is clear, else d1 0 and C clear.  Gives back d0, d1, X and C.
    /// No script runs it (checked by reading).
    pub fn r_00bb94(&mut self, d0: u32) -> (u32, u32, bool, bool) {
        let set = d0 & 1 << 2 != 0;
        let kind = (d0 as u8 & 3) + 1;
        // asl.b #4: X the kind's bit 4 (never set)
        let x = kind & 0x10 != 0;
        (low_byte(d0, kind << 4 | 6), u32::from(!set), x, !set)
    }

    /// `func_00BB30`, kind 1-4's routine: the running task's (a6) kind
    /// set to d0 - 1 and its state to 2, if its state (bits 0-2) is 0.
    /// Gives back d0, d1, d6, a6, X, N, Z and C.
    /// No script runs it (checked by reading).
    pub fn r_00bb30(&mut self, regs: &mut Regs) {
        let kind = (regs.d0 as u8).wrapping_sub(1);
        regs.d0 = low_byte(regs.d0, kind);
        let d6 = low_byte(regs.d6, self.read8(CURRENT));
        (regs.d6, regs.a6, regs.x) = self.r_0005f8(d6);
        let state = self.read8(regs.a6 + TASK_STATE) & STATE_BITS;
        regs.d1 = low_byte(regs.d1, state);
        regs.n = false;
        regs.z = state == 0;
        regs.c = false;
        if state == 0 {
            regs.d1 = low_byte(regs.d1, state | 2);
            self.write8(regs.a6 + TASK_STATE, state | 2);
            self.write8(regs.a6 + TASK_KIND, kind);
            regs.n = (kind as i8) < 0;
            regs.z = kind == 0;
        }
        regs.unwound = false;
    }

    /// `func_00BB54`, kind 1-4's second routine: `PUSH_X` the kind's
    /// long in `PUSHES`, and `NO_SLIDE` one less.  Gives back d0 (the
    /// kind less 1, x 4), a0, and the flags of the count.
    /// No script runs it (checked by reading).
    pub fn r_00bb54(&mut self, regs: &mut Regs) {
        let kind = (regs.d0 as u8).wrapping_sub(1);
        let offset = (kind as i8 as u16).wrapping_mul(4);
        regs.d0 = low_word(regs.d0, offset);
        regs.a0 = PUSHES;
        let push = self.read32(PUSHES.wrapping_add(offset as i16 as u32));
        self.write32(PUSH_X, push);
        let count = self.read8(NO_SLIDE);
        let left = count.wrapping_sub(1);
        self.write8(NO_SLIDE, left);
        regs.n = (left as i8) < 0;
        regs.z = left == 0;
        regs.c = count == 0;
        regs.x = regs.c;
        regs.unwound = false;
    }

    /// `func_00BC4E`, kind 1-4's routine: if `func_001896` gives Z for
    /// the running task, its kind set to d0 - 1 and sound 0x36.  Gives
    /// back d1 (the kind less 1) and d6.
    /// No script runs it (checked by reading).
    pub fn r_00bc4e(&mut self, regs: &mut Regs) {
        let kind = (regs.d0 as u8).wrapping_sub(1);
        let d0 = low_byte(regs.d0, kind);
        regs.d1 = low_byte(regs.d1, kind);
        let d6 = low_byte(regs.d6, self.read8(CURRENT));
        let (d6, task, _) = self.r_0005f8(d6);
        regs.d6 = d6;
        let (d0, z) = self.r_001896(d0, task);
        if z {
            self.write8(task + TASK_KIND, kind);
            self.r_0034b6(low_byte(d0, 0x36));
        }
        regs.unwound = false;
    }

    /// `func_00BC70`: an `rts`.
    /// No script runs it (checked by reading).
    pub fn r_00bc70(&mut self) {}

    /// `func_00BC72`: a sprite record (depth 5, `func_000796`) showing
    /// frame 0xD6 (d0's low byte 0) or 0xD7 at `NEAR_X_LOW` - 8,
    /// `NEAR_Y_LOW` + 0x10.  Gives back d0 6, d5 and a5 (the record),
    /// X (the y's carry) and C clear.
    /// No script runs it (checked by reading).
    pub fn r_00bc72(&mut self, regs: &mut Regs) {
        let frame: u8 = if regs.d0 as u8 == 0 { 0xD6 } else { 0xD7 };
        let (d5, sprite, _, _) = self.r_000796(5, regs.d1, regs.d2, regs.d3, regs.d4, regs.d5, regs.d6, regs.d7, regs.a0, regs.a1, regs.a2, regs.a3, regs.a4, regs.a5, regs.x);
        regs.d5 = d5;
        regs.a5 = sprite;
        let x = self.read8(NEAR_X_LOW).wrapping_sub(8);
        self.write8(sprite + SPRITE_X + 1, x);
        let (y, carry) = self.read8(NEAR_Y_LOW).overflowing_add(0x10);
        self.write8(sprite + SPRITE_Y + 1, y);
        self.write8(sprite + SPRITE_X, 0);
        self.write8(sprite + SPRITE_Y, 0);
        self.write8(sprite + SPRITE_FRAME, frame);
        self.write8(sprite + SPRITE_FLAGS, 4);
        regs.d0 = 6;
        regs.x = carry;
        regs.c = false;
        regs.unwound = false;
    }

    /// `func_00BCC2`: an `rts`.
    /// No script runs it (checked by reading).
    pub fn r_00bcc2(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `func_00BD54`: kind 7 runs `func_00BD6C` and `func_0052EA` (an
    /// `rts`).  Gives back d0 0x76 (kind 7, 6) and C clear.
    pub fn r_00bd54(&mut self, regs: &mut Regs) {
        self.write32(KIND_7_CALL, ROUTINE_BD6C);
        self.write32(KIND_7_CALL_2, ROUTINE_52EA);
        regs.d0 = 0x76;
        regs.c = false;
        regs.unwound = false;
    }

    /// `func_00BD6C`, kind 7's routine: the running task's state set
    /// to 3 if it is 0.  Gives back d6 (and d1 as it was).
    /// No script runs it (checked by reading).
    pub fn r_00bd6c(&mut self, regs: &mut Regs) {
        let d6 = low_byte(regs.d6, self.read8(CURRENT));
        let (d6, task, _) = self.r_0005f8(d6);
        regs.d6 = d6;
        let state = self.read8(task + TASK_STATE) & STATE_BITS;
        if state == 0 {
            self.write8(task + TASK_STATE, state | 3);
        }
        regs.unwound = false;
    }
}
