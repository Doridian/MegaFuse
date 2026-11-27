//! The eight pieces `func_00C15A` (`l_sequence.rs`) sends off, each in
//! its direction (0-7, its count): `func_00C6AE` flies and bounces
//! three times, `func_00C72E` blinks out; and the task of
//! `func_00C766`, which runs its animation where it is made.
//! `func_00C742` is the fall the circle (`l_circle.rs`) and the
//! dropped tasks (`l_drops.rs`) use too.

use crate::{Game, Regs};

/// A task's fields: its animation's timer and step, its direction (its
/// count, then a count of frames), its bounces left, its rate (a word;
/// the high byte the pixels a frame, `a_motion.rs`) and its offset (or
/// an animation, a long).
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_BOUNCES: u32 = 0x17;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's frame and flags.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// A velocity (words, x then y; `a_motion.rs`).
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;
/// The pieces' velocities by direction (words: D_00C68E x, D_00C69E y).
const PIECE_X: u32 = 0x00C68E;
const PIECE_Y: u32 = 0x00C69E;
/// A piece's frames (0xF4-0xF7, one each 5 frames), its fall a frame,
/// its rate after a bounce.
const PIECE_FRAMES: u8 = 0xF4;
const PIECE_FALL: u32 = 0x30;
const BOUNCE_RATE: u16 = 0xFE80;
/// The routine of a piece done bouncing.
const ROUTINE_C72E: u32 = 0x00C72E;
/// The task `func_00C766` makes: its routine, depth, priority and
/// animation.
const ROUTINE_C78E: u32 = 0x00C78E;
const C766_DEPTH: u8 = 0xFD;
const C766_PRIORITY: u8 = 0x42;
const C766_ANIM: u32 = 0x0DA4C4;
/// The fastest fall (the rate's high byte).
const FASTEST: u8 = 3;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_00C6AE`, a piece's routine: its frame stepped each 5
    /// frames, it moves at its direction's velocity (`PIECE_X`,
    /// `PIECE_Y`) and falls (`func_00C742`, `func_001ACA`); when it
    /// lands it bounces (`BOUNCE_RATE`), and after its last bounce it
    /// goes on as `func_00C72E` (count 0x40, its sprite's flags 0x44).
    /// Otherwise its sprite is placed (`func_00C73A`).
    pub fn r_00c6ae(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let left = self.read8(r.a6 + TASK_TIMER).wrapping_sub(1);
        self.write8(r.a6 + TASK_TIMER, left);
        if left == 0 {
            self.write8(r.a6 + TASK_TIMER, 5);
            let step = self.read8(r.a6 + TASK_STEP).wrapping_add(1);
            self.write8(r.a6 + TASK_STEP, step);
            let frame = (step & 3).wrapping_add(PIECE_FRAMES);
            r.d0 = low_byte(r.d0, frame);
            self.write8(r.a5 + SPRITE_FRAME, frame);
        }
        let direction = self.read8(r.a6 + TASK_COUNT);
        let offset = direction.wrapping_add(direction) as i8 as i16 as u16;
        r.d0 = low_word(r.d0, offset);
        let at = offset as i16 as u32;
        let vx = self.read16(PIECE_X.wrapping_add(at));
        self.write16(VELOCITY_X, vx);
        let vy = self.read16(PIECE_Y.wrapping_add(at));
        self.write16(VELOCITY_Y, vy);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        r.d0 = PIECE_FALL;
        r.c = false;
        self.r_00c742(r.d0, r.a6);
        let up;
        (r.d0, r.x, up) = self.r_001aca(r.d0, r.a6);
        if up {
            self.l_shown(r);
            return;
        }
        self.write8(r.a6 + TASK_OFFSET, 0);
        self.write16(r.a6 + TASK_RATE, BOUNCE_RATE);
        let was = self.read8(r.a6 + TASK_BOUNCES);
        self.write8(r.a6 + TASK_BOUNCES, was.wrapping_sub(1));
        (r.x, r.c) = (was == 0, was == 0);
        if was != 1 {
            self.l_shown(r);
            return;
        }
        self.write8(r.a5 + SPRITE_FLAGS, 0x44);
        self.write8(r.a6 + TASK_COUNT, 0x40);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C72E);
    }

    /// `func_00C73A` on the family's registers.
    fn l_shown(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_00c73a(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
    }

    /// `func_00C72E`, a piece's routine: its sprite placed
    /// (`func_00C73A`) for 0x40 frames, then it ends (`func_00121A`).
    pub fn r_00c72e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let was = self.read8(r.a6 + TASK_COUNT);
        self.write8(r.a6 + TASK_COUNT, was.wrapping_sub(1));
        (r.x, r.c) = (was == 0, was == 0);
        if was != 1 {
            self.l_shown(r);
            return;
        }
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
    }

    /// `func_00C73A` (`.L00C73A`): the sprite placed up by the task's
    /// offset (`func_001B94`), its depth by its y (`func_001B5C`).
    /// Gives back d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00c73a(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, bool, bool) {
        let (d0, _, _) = self.r_001b94(a5, a6);
        self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5)
    }

    /// `func_00C742`: the task's rate up by `d0`'s low byte (a word),
    /// its high byte kept from 3 on (but negative).
    pub fn r_00c742(&mut self, d0: u32, a6: u32) {
        let rate = self.read16(a6 + TASK_RATE).wrapping_add(d0 as u16 & 0xFF);
        self.write16(a6 + TASK_RATE, rate);
        let high = (rate >> 8) as u8;
        if (high as i8) >= 0 && high >= FASTEST {
            self.write8(a6 + TASK_RATE, FASTEST);
        }
    }

    /// `func_00C766`: a task of `ROUTINE_C78E` made where the position
    /// is (`func_00CA06`), in the middle of its cell, with animation
    /// `C766_ANIM`.  Gives back d0, d5, d6, a5, a6, X and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00c766(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let (d5, d6, a5, a6, x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, C766_DEPTH, C766_PRIORITY, ROUTINE_C78E);
        let (d0, _) = self.r_00ca06(d0, a5, a6);
        self.r_00187c(a6);
        self.write32(a6 + TASK_ANIM, C766_ANIM);
        (d0, d5, d6, a5, a6, x, false)
    }

    /// `func_00C78E`, a task's routine: its sprite placed
    /// (`func_001B7E`) and its animation run (`func_0013D6`).
    pub fn r_00c78e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }
}
