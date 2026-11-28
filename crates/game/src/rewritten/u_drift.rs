//! The ending's 16 drifting sprites: records made at random places
//! (`func_016726`), moved each frame against `func_0161C0`'s velocity,
//! each at its own rate (`func_0167A2`, `func_0167C6`); four of them
//! made into tasks that fly off (`func_016DBA`, `func_016E3E`); and
//! tasks made at a place the script gives (`func_016E7E`,
//! `func_016F74`) with their routines.

use crate::{Game, Regs};

/// The 16 drifting sprites, 4 bytes each: the sprite record's index (a
/// negative one is passed over), the rate (0xFF, 0xEF, 0xDF, ...: the
/// velocity is scaled by it / 0x100) and the fractions of the x and y.
const RECORDS: u32 = 0xFFFF_AD83;
const RATES: u32 = 0xFFFF_AD84;
const X_FRACTIONS: u32 = 0xFFFF_AD85;
const Y_FRACTIONS: u32 = 0xFFFF_AD86;
/// The end of the entries (16 x 4).
const ENTRIES_END: u8 = 0x40;
/// The next rate `func_016726` gives.
const NEXT_RATE: u32 = 0xFFAC8B;
/// `func_0161C0`'s velocity (words, x then y); the sprites move by it
/// (its size, the sign its high byte has).
const VELOCITY_X_KEPT: u32 = 0xFFAD62;
const VELOCITY_Y_KEPT: u32 = 0xFFAD64;
/// The entry `func_016DBA` is at (0, 4, 8, 12).
const ENTRY: u32 = 0xFFAD82;
/// Not 0: `func_016E3E`'s tasks end once their x goes negative
/// (`u_makers.rs`).
const STOP: u32 = 0xFFAD80;

/// The place a task is made at (words, x then y; `func_00152A`, its
/// bytes), the speed (`func_016EC4` keeps it in the task), the byte
/// the speed is made from (`func_016DBA`: the rate) and a byte
/// `func_016E7E` picks the animation by (`func_016F08`, 0-3).
const POSITION_X: u32 = 0xFF9F04;
const POSITION_X_LOW: u32 = 0xFF9F05;
const POSITION_Y: u32 = 0xFF9F06;
const POSITION_Y_LOW: u32 = 0xFF9F07;
const SPEED: u32 = 0xFF9F00;
const RATE: u32 = 0xFF9F08;
const CHOICE: u32 = 0xFF9EF6;
/// The velocity (words, x then y: their bytes).
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_X_LOW: u32 = 0xFF9EC5;
const VELOCITY_Y: u32 = 0xFF9EC6;
const VELOCITY_Y_LOW: u32 = 0xFF9EC7;
/// The index of the task made last (`func_0003E8`).
const NEW_TASK: u32 = 0xFF80EF;

/// A task's routine (a long), its x (a word; the low byte), its y (the
/// high and low bytes), a count (its depth: `func_016F66`), its speed
/// (a word: the x velocity; `func_016FC2`'s the x and y velocities'
/// high bytes) and animation (a long).
const TASK_ROUTINE: u32 = 0x04;
const TASK_X: u32 = 0x0E;
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y: u32 = 0x10;
const TASK_Y_LOW: u32 = 0x11;
const TASK_COUNT: u32 = 0x16;
const TASK_SPEED: u32 = 0x1A;
const TASK_SPEED_LOW: u32 = 0x1B;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's frame, flags (0x80 set below y 0xA0), x and y
/// (words; the low bytes: the position on the screen).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;
const LOW_SPRITE: u8 = 0xA0;
const HIDDEN: u8 = 0x80;

/// The routines and animations of the tasks made here.
const ROUTINE_167A2: u32 = 0x0167A2;
const ROUTINE_16E3E: u32 = 0x016E3E;
const ROUTINE_16F2E: u32 = 0x016F2E;
const ROUTINE_16FC2: u32 = 0x016FC2;
const ANIM_D8F82: u32 = 0x0D8F82;
const ANIM_D8F7A: u32 = 0x0D8F7A;
const ANIM_D8F8A: u32 = 0x0D8F8A;
const ANIM_D8F92: u32 = 0x0D8F92;
/// `func_016E7E`'s second task's animations by `CHOICE` (D_016EEA,
/// longs).
const ANIMS: u32 = 0x016EEA;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// The sprite record at a5 hidden (flags 0x80) at y 0xA0 or more,
    /// else shown (0).
    fn u_hide_low(&mut self, a5: u32) {
        self.write8(a5 + SPRITE_FLAGS, 0);
        if self.read8(a5 + SPRITE_Y_LOW) >= LOW_SPRITE {
            self.write8(a5 + SPRITE_FLAGS, HIDDEN);
        }
    }

    /// `func_016726`: a task of `func_0167A2` (`func_0003E8`, priority
    /// 0x41) and the 16 sprites: each a record at depth 0xFE
    /// (`func_000796`; not checked for being made) at a random x and y
    /// (0-0xFF), frame 0x18 for the first eight, 0x19 for the rest, its
    /// rate 0xFF less 0x10 for each before (the low bit set).  Gives
    /// back d0, d1, d5, d6, a0 and a5.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_016726(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32) {
        let (d6, _, _, _) = self.r_0003e8(d4, d5, d6, 0x41, ROUTINE_167A2);
        self.write8(NEXT_RATE, 0xFF);
        let (mut d0, mut d1, mut d5, mut a5) = (d0 & 0xFFFF_0000, d1, d5, a5);
        loop {
            d1 = low_word(d1, d0 as u16);
            let at = d1 as u16 as i16 as u32;
            let rate = self.read8(NEXT_RATE);
            self.write8(RATES.wrapping_add(at), rate);
            let next = rate.wrapping_sub(0x10) | 1;
            let borrow = rate < 0x10;
            self.write8(NEXT_RATE, next);
            d0 = low_byte(d0, 0xFE);
            (d5, a5, _, _) = self.r_000796(d0, d1, d2, d3, d4, d5, d6, d7, RATES, a1, a2, a3, a4, a5, borrow);
            self.write8(RECORDS.wrapping_add(at), d5 as u8);
            (d0, d1, _) = self.r_001be2(d1);
            self.write8(a5 + SPRITE_Y_LOW, d0 as u8);
            (d0, d1, _) = self.r_001be2(d1);
            self.write8(a5 + SPRITE_X_LOW, d0 as u8);
            self.write8(a5 + SPRITE_X, 0);
            self.write8(a5 + SPRITE_Y, 0);
            self.write8(a5 + SPRITE_FLAGS, 0);
            let frame = ((d1 as u8 >> 4) & 1) + 0x18;
            self.write8(a5 + SPRITE_FRAME, frame);
            let next = (d1 as u8).wrapping_add(4);
            d0 = low_byte(low_word(d0, d1 as u16), next);
            if next == ENTRIES_END {
                break;
            }
        }
        (d0, d1, d5, d6, RECORDS, a5)
    }

    /// `func_0167A2`, a task's routine: each of the 16 sprites with a
    /// record moved (`func_0167C6`).  Gives back X and C (clear).
    pub fn r_0167a2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 &= 0xFFFF_0000;
        loop {
            r.d1 = low_word(r.d1, r.d0 as u16);
            r.a0 = RECORDS;
            let record = self.read8(RECORDS.wrapping_add(r.d1 as u16 as i16 as u32));
            r.d5 = low_byte(r.d5, record);
            if (record as i8) >= 0 {
                (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
                (r.d1, r.d2, r.d3, r.d5, r.a0, r.a5) = self.r_0167c6(r.d1, r.d2, r.d5);
            }
            let next = (r.d1 as u8).wrapping_add(4);
            r.d0 = low_byte(low_word(r.d0, r.d1 as u16), next);
            r.x = false;
            if next == ENTRIES_END {
                break;
            }
        }
        r.c = false;
    }

    /// One axis of `func_0167C6`: the sprite's word at `position` (its
    /// high byte cleared after) plus the entry's rate times the size of
    /// the velocity at `velocity` / 0x100, signed as the velocity is,
    /// with the fraction at `fractions`.  d1 (the entry; its high word
    /// becomes the product's) and d3 (the product) go through.
    #[allow(clippy::too_many_arguments)]
    fn u_drift_axis(&mut self, d1: &mut u32, d2: &mut u32, d3: &mut u32, d5: &mut u32, a5: &mut u32, velocity: u32, fractions: u32, position: u32) {
        let speed = self.read16(velocity);
        let size = if (speed as i16) < 0 { speed.wrapping_neg() } else { speed };
        *d2 = low_word(*d2, size);
        let at = *d1 as u16 as i16 as u32;
        let rate = self.read8(RATES.wrapping_add(at));
        *d3 = low_word(*d3, *d1 as u16);
        let (product, _) = self.r_0011ca(u32::from(rate), *d2);
        *d1 = *d3;
        *d3 = product;
        let record = self.read8(RECORDS.wrapping_add(at));
        (*d5, *a5, _) = self.r_000b5a(low_byte(*d5, record));
        if (self.read8(velocity) as i8) < 0 {
            *d3 = low_word(*d3, (*d3 as u16).wrapping_neg());
        }
        let whole = (*d3 as u16) >> 8;
        *d2 = low_word(*d2, whole);
        let (fraction, carry) = self.read8(fractions.wrapping_add(at)).overflowing_add(*d3 as u8);
        self.write8(fractions.wrapping_add(at), fraction);
        let moved = self.read16(*a5 + position).wrapping_add(whole).wrapping_add(u16::from(carry));
        self.write16(*a5 + position, moved);
        self.write8(*a5 + position, 0);
    }

    /// `func_0167C6`: sprite `d1` (the entry's offset) moved by
    /// `func_0161C0`'s velocity at its rate: x then y (each within
    /// 0-0xFF), hidden below y 0xA0.  Gives back d1, d2, d3, d5, a0
    /// (`Y_FRACTIONS`) and a5 (the record).
    pub fn r_0167c6(&mut self, d1: u32, d2: u32, d5: u32) -> (u32, u32, u32, u32, u32, u32) {
        let (mut d1, mut d2, mut d3, mut d5, mut a5) = (d1, d2, 0, d5, 0);
        self.u_drift_axis(&mut d1, &mut d2, &mut d3, &mut d5, &mut a5, VELOCITY_X_KEPT, X_FRACTIONS, SPRITE_X);
        self.u_drift_axis(&mut d1, &mut d2, &mut d3, &mut d5, &mut a5, VELOCITY_Y_KEPT, Y_FRACTIONS, SPRITE_Y);
        self.u_hide_low(a5);
        (d1, d2, d3, d5, Y_FRACTIONS, a5)
    }

    /// `func_016DBA`: `STOP` cleared; the first four sprites made into
    /// tasks of `func_016E3E` (`func_016EC4`) where they are (x + 0x100),
    /// their speed `func_0161C0`'s x velocity, negated, times the rate /
    /// 0x100, then negated, with animation D_0D8F82 and the count (the
    /// depth) the rate negated, even, the low bit set.
    pub fn r_016dba(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(STOP, 0);
        self.write8(ENTRY, 0);
        loop {
            let entry = self.read8(ENTRY);
            r.d3 = low_word(r.d3, u16::from(entry));
            let at = u32::from(entry);
            let record = self.read8(RECORDS + at);
            (r.d5, r.a5, r.x) = self.r_000b5a(low_byte(r.d5, record));
            let x = self.read8(r.a5 + SPRITE_X_LOW);
            self.write8(POSITION_X_LOW, x);
            self.write8(POSITION_X, 1);
            let y = self.read8(r.a5 + SPRITE_Y_LOW);
            self.write8(POSITION_Y_LOW, y);
            self.write8(POSITION_Y, 0);
            r.d2 = low_word(r.d2, self.read16(VELOCITY_X_KEPT).wrapping_neg());
            r.a0 = RATES;
            let rate = self.read8(RATES + at);
            r.d0 = low_byte(low_word(r.d0, 0), rate);
            self.write8(RATE, rate);
            (r.d1, r.x) = self.r_0011ca(r.d0, r.d2);
            r.d1 = low_word(r.d1, (r.d1 as u16).wrapping_neg());
            self.write16(SPEED, r.d1 as u16);
            (r.d5, r.d6, r.a5, r.a6) = self.r_016ec4(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            self.write32(r.a6 + TASK_ROUTINE, ROUTINE_16E3E);
            self.write32(r.a6 + TASK_ANIM, ANIM_D8F82);
            let count = (self.read8(RATE).wrapping_neg() & 0xFE) | 1;
            r.d0 = low_byte(r.d0, count);
            self.write8(r.a6 + TASK_COUNT, count);
            let entry = self.read8(ENTRY).wrapping_add(4);
            self.write8(ENTRY, entry);
            if entry == 0x10 {
                break;
            }
        }
        r.c = false;
    }

    /// `func_016E3E`, a task's routine: its animation a frame on
    /// (`func_0013D6`), moved by its speed as the x velocity
    /// (`func_001A88`); once its x is negative its x is 0x110, and when
    /// `STOP` is set it ends (`func_00121A`); else its sprite is hidden
    /// below y 0xA0 and placed (`func_015F46`).
    pub fn r_016e3e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        let speed = self.read16(r.a6 + TASK_SPEED);
        self.write16(VELOCITY_X, speed);
        let negative;
        (r.d0, r.d1, r.x, negative, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        if negative {
            self.write16(r.a6 + TASK_X, 0x110);
            if self.read8(STOP) != 0 {
                (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
                return;
            }
        }
        self.u_hide_low(r.a5);
        r.c = self.r_015f46(r.a5, r.a6);
    }

    /// `func_016E7E`: at x 0x120 and the place and speed the script
    /// gives (`func_016F08`), two tasks of `func_016F2E`
    /// (`func_016EC4`): the first with animation D_0D8F7A and count
    /// 0x71, the second (the task made last) with the animation
    /// `CHOICE` picks (`ANIMS`) and count 0x70.
    pub fn r_016e7e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write16(POSITION_X, 0x120);
        r.d0 = self.r_016f08(r.d0, r.d1);
        (r.d5, r.d6, r.a5, r.a6) = self.r_016ec4(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.write32(r.a6 + TASK_ANIM, ANIM_D8F7A);
        self.write8(r.a6 + TASK_COUNT, 0x71);
        (r.d5, r.d6, r.a5, r.a6) = self.r_016ec4(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        let at = u16::from(self.read8(CHOICE)).wrapping_mul(4);
        r.d0 = low_word(r.d0, at);
        let made = self.read8(NEW_TASK);
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, made));
        let anim = self.read32(ANIMS.wrapping_add(at as i16 as u32));
        self.write32(r.a6 + TASK_ANIM, anim);
        self.write8(r.a6 + TASK_COUNT, 0x70);
        r.c = false;
    }

    /// `func_016EC4`: a task of `func_016F2E` (depth 0xFB, priority
    /// 0x42; not checked for being made) at the place, its speed
    /// `SPEED`, its sprite's flags 0.  Gives back d5, d6, a5 and a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_016ec4(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32) {
        let (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0xFB, 0x42, ROUTINE_16F2E);
        self.r_00152a(a6);
        let speed = self.read16(SPEED);
        self.write16(a6 + TASK_SPEED, speed);
        self.write8(a5 + SPRITE_FLAGS, 0);
        (d5, d6, a5, a6)
    }

    /// `func_016F2E`, a task's routine: its animation a frame on
    /// (`func_0013D6`), moved by its speed as the x velocity
    /// (`func_001A88`); once its x is negative and its low byte below
    /// 0xF0 it ends (`func_00121A`); else its sprite is hidden below y
    /// 0xA0 and placed at its depth (`func_016F66`).
    pub fn r_016f2e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        let speed = self.read16(r.a6 + TASK_SPEED);
        self.write16(VELOCITY_X, speed);
        let negative;
        (r.d0, r.d1, r.x, negative, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        if negative {
            let low = self.read8(r.a6 + TASK_X_LOW);
            r.d0 = low_byte(r.d0, low);
            if low < 0xF0 {
                (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
                return;
            }
        }
        self.u_hide_low(r.a5);
        (r.d0, r.c) = self.r_016f66(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6, r.x);
    }

    /// `.L016F66`: the sprite placed (`func_015F46`) and the task's
    /// record at the depth its count gives (`func_0007D2`).  Gives back
    /// d0 and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_016f66(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32, x: bool) -> (u32, bool) {
        self.r_015f46(a5, a6);
        let d0 = low_byte(d0, self.read8(a6 + TASK_COUNT));
        let c = self.r_0007d2(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, x);
        (d0, c)
    }

    /// `func_016F74`: at the place and speed the script gives
    /// (`func_016EFE`), a task of `func_016FC2` (depth 0xFD, priority
    /// 0x42; not checked for being made), its speed `SPEED`, its
    /// sprite's flags 0, with animation D_0D8F8A and count 0xFD, or
    /// D_0D8F92 and 0xFC when `CHOICE` isn't 0.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments)]
    pub fn r_016f74(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) {
        let d0 = self.r_016efe(d0, d1);
        let (_, _, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0xFD, 0x42, ROUTINE_16FC2);
        self.r_00152a(a6);
        let speed = self.read16(SPEED);
        self.write16(a6 + TASK_SPEED, speed);
        self.write8(a5 + SPRITE_FLAGS, 0);
        self.write32(a6 + TASK_ANIM, ANIM_D8F8A);
        self.write8(a6 + TASK_COUNT, 0xFD);
        if self.read8(CHOICE) != 0 {
            self.write32(a6 + TASK_ANIM, ANIM_D8F92);
            self.write8(a6 + TASK_COUNT, 0xFC);
        }
    }

    /// `func_016FC2`, a task's routine: its animation a frame on
    /// (`func_0013D6`), moved by its speed's bytes as the x and y
    /// velocities' high bytes (`func_001A88`, `func_001AA2`); once its
    /// y is not negative, 0x100 or more and its low byte 0xF0 or more
    /// it ends (`func_00121A`); else it is placed at its depth
    /// (`func_016F66`).
    /// No script calls it: reviewed by reading.
    pub fn r_016fc2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        let vx = self.read8(r.a6 + TASK_SPEED);
        self.write8(VELOCITY_X, vx);
        let vy = self.read8(r.a6 + TASK_SPEED_LOW);
        self.write8(VELOCITY_Y, vy);
        self.write8(VELOCITY_X_LOW, 0);
        self.write8(VELOCITY_Y_LOW, 0);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        let negative;
        (r.d0, r.d1, r.x, negative, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        if !negative && self.read8(r.a6 + TASK_Y) != 0 && self.read8(r.a6 + TASK_Y_LOW) >= 0xF0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        (r.d0, r.c) = self.r_016f66(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6, r.x);
    }
}
