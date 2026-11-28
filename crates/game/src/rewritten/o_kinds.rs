//! The routines of a few of the kinds' objects (the kinds' table
//! D_003704, `c_spawn.rs`): `func_00F000` (two kinds), `func_00F1CC`,
//! `func_00F20E`, `func_00F3C6` and `loc_00F4AA` (two kinds), with the
//! moves they start and the tasks they make.  Each kind's routine
//! starts by setting itself up once (bits 4-7 of `OBJ_STARTED`), then
//! goes as `func_007174` and the like move it, and a puff task
//! (`PUFF`) is made when those say so (C; `func_00F1D0`,
//! `func_00F4AE`).
//!
//! They keep the registers the objects' routines share (`Regs`): a4
//! and d4 the object and its index, a5 and d5 a sprite record and its
//! index, a6 a task.

use crate::{Game, Regs};

/// An object's fields (`anim.rs`, `d_pair.rs`): not 0 while in use
/// (`CARRYING` when the object before it rides it: its height is set
/// from this one's sprite frame); its routine (a long); the high byte
/// of a kept routine, bits 4-7 set once the routine is set up; its
/// direction (bits 0-1); what its animation is by; a count; the
/// animation and its frames to go and step.
const OBJ_USED: u32 = 0x00;
const CARRYING: u8 = 0x24;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_STARTED: u32 = 0x06;
const STARTED: u8 = 0xF0;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x15;
const OBJ_COUNT: u32 = 0x16;
const OBJ_ANIM: u32 = 0x20;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
/// The object before's height (its byte 0x2A, `d_pair.rs`), from this
/// one (0x2A - 0x40).
const BEFORE_HEIGHT: u32 = 0xFFFF_FFEA;
/// Heights by the step of the animation (/2, D_00EFB8), and by the
/// sprite frame (0x86 on: from D_00EFB8 - 0x86 while the animation
/// ends, from D_00EFB8 - 0x85 while it goes on).
const HEIGHTS: u32 = 0x00EFB8;
const HEIGHTS_BY_FRAME: u32 = HEIGHTS - 0x86;
const HEIGHTS_BY_FRAME_ON: u32 = HEIGHTS - 0x85;

/// A sprite record's frame and flags.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// A task's fields: its animation's frames to go and step, a count
/// (the puff's) or the velocity it moves by (a long, x then y; its
/// second byte the direction it was made with), a word for `SPEED`, its
/// animation.
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_VELOCITY: u32 = 0x16;
const TASK_DIRECTION: u32 = 0x17;
const TASK_SPEED: u32 = 0x1A;
const TASK_ANIM: u32 = 0x20;

/// The object being run (index, address), the sprite record of the
/// routine being run (index, address).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// A direction the routines are handed (a byte).
const DIRECTION: u32 = 0xFFAABD;
/// The velocity `func_001A88`, `func_001AA2` move by (a long, x then
/// y) and the speed of `func_00410C`, `func_001620` (a word).
const VELOCITY: u32 = 0xFF9EC4;
const SPEED: u32 = 0xFF9F00;
/// The cell's address in the cell map (`func_00167A`).
const CELL_AT: u32 = 0xFF9ED6;

/// The puff: its routine, priority, depth (`func_00F1D0`; 2 for
/// `func_00F4AE`), animations and count.
const PUFF: u32 = 0x007A90;
const PUFF_PRIORITY: u8 = 0xC2;
const PUFF_DEPTH: u8 = 0x80;
const PUFF_DEPTH_2: u8 = 0x02;
const PUFF_ANIM: u32 = 0x0E5614;
const PUFF_ANIM_2: u32 = 0x0E642A;
const PUFF_COUNT: u8 = 0x40;

/// `func_00F000`'s routines and animations: set up (`START_ANIM`),
/// then counting down from `HOLD_COUNT` (`HOLD_ANIM`, `func_00F102`'s
/// shots every 4 frames), then `END_ANIM` to the end.
const START_ANIM: u32 = 0x0E5626;
const HOLD_ANIM: u32 = 0x0E5642;
const END_ANIM: u32 = 0x0E5634;
const HOLD_COUNT: u8 = 0x84;
const STARTING: u32 = 0x00F02A;
const HOLDING: u32 = 0x00F048;
const ENDING: u32 = 0x00F072;
const START: u32 = 0x00F000;
/// `func_00F102`'s shot: routine, priority, animation, speed, sprite
/// command; the animation's byte it sets.
const SHOT: u32 = 0x00F192;
const SHOT_PRIORITY: u8 = 0xC2;
const SHOT_DEPTH: u8 = 0x80;
const SHOT_SPEED: u16 = 0x180;
const SHOT_COMMAND: u8 = 0x5D;
const SHOT_ANIM: u32 = 0x0E564C;
const SHOT_ANIM_BYTE: u8 = 0xF3;

/// `func_00F20E`'s routine and kind, the step its thrown task is made
/// at, the task (routine, priority, animations by direction D_00F2F6,
/// speed, `TASK_SPEED`, command); its landing (routine, animation).
const THROWING: u32 = 0x00F23A;
const THROW_START: u32 = 0x00F20E;
const THROW_KIND: u8 = 5;
const THROW_STEP: u8 = 0x0E;
const THROWN: u32 = 0x00F306;
const THROWN_PRIORITY: u8 = 0x42;
const THROWN_DEPTH: u8 = 0x80;
const THROWN_ANIMS: u32 = 0x00F2F6;
const THROWN_SPEED: u16 = 0x800;
const THROWN_TASK_SPEED: u16 = 0x100;
const THROWN_COMMAND: u8 = 0x87;
const LANDED: u32 = 0x00F3AA;
const LANDED_ANIM: u32 = 0x0E6386;

/// `func_00F3C6`'s routine and kind, the step it acts at (command),
/// the cell value it acts on, its task (routine `PUFF`, depth 2,
/// animation).
const DIGGING: u32 = 0x00F3FC;
const DIG_START: u32 = 0x00F3C6;
const DIG_STEP: u8 = 0x12;
const DIG_COMMAND: u8 = 0x60;
const DIG_ON: u8 = 2;
const DIG_ANIM: u32 = 0x0E6438;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// The routine set up: whether it was already (bits 4-7 of
    /// `OBJ_STARTED`); if not they are set.  d0's low byte as the
    /// original leaves it.
    fn o_started(&mut self, r: &mut Regs) -> bool {
        let started = self.read8(r.a4 + OBJ_STARTED) & STARTED;
        r.d0 = low_byte(r.d0, started);
        r.c = false;
        if started != 0 {
            return true;
        }
        r.d0 = low_byte(r.d0, STARTED);
        self.write8(r.a4 + OBJ_STARTED, STARTED);
        false
    }

    /// `OBJ_USED` compared with `CARRYING` (d0's low byte, C).
    fn o_carrying(&mut self, r: &mut Regs) -> bool {
        let used = self.read8(r.a4 + OBJ_USED);
        r.d0 = low_byte(r.d0, used);
        r.c = used < CARRYING;
        used == CARRYING
    }

    /// The sprite record of the routine being run to d5, a5.
    fn o_slot(&mut self, r: &mut Regs) {
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
    }

    /// The object being run to d4, a4.
    fn o_running(&mut self, r: &mut Regs) {
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
    }

    /// The object before's height by the sprite frame (`table`).
    fn o_height_by_frame(&mut self, r: &mut Regs, table: u32) {
        let frame = self.read8(r.a5 + SPRITE_FRAME);
        r.d0 = low_word(r.d0, u16::from(frame));
        let height = self.read8(table + u32::from(frame));
        self.write8(r.a4.wrapping_add(BEFORE_HEIGHT), height);
    }

    /// `func_007174`, then `func_00F1D0`.
    fn o_walk(&mut self, r: &mut Regs) {
        self.r_007174(r);
        if self.restarted.is_none() {
            self.o_puff(r);
        }
        r.unwound = false;
    }

    /// `func_0071A0`, then `func_00F1D0` (`puff`) or `func_00F4AE`.
    fn o_move(&mut self, r: &mut Regs, second: bool) {
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_0071a0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        if second {
            self.o_puff_2(r);
        } else {
            self.o_puff(r);
        }
        r.unwound = false;
    }

    fn o_puff(&mut self, r: &mut Regs) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00f1d0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
    }

    fn o_puff_2(&mut self, r: &mut Regs) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00f4ae(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
    }

    /// `func_00F000`, a kind's routine: set up the first time (its
    /// animation `START_ANIM`, then `func_00F02A`); after that it walks
    /// (`func_007174`), or, carrying the object before
    /// (`CARRYING`), sets that one's height by its step (`HEIGHTS`) and
    /// goes as `func_00722E`; a puff when they say so.
    pub fn r_00f000(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        if self.o_started(r) {
            if !self.o_carrying(r) {
                self.o_walk(r);
                return;
            }
            let step = (self.read8(r.a4 + OBJ_STEP) >> 1).wrapping_sub(1);
            let step = if step & 0x80 != 0 { 0 } else { step };
            r.d0 = low_word(r.d0, u16::from(step));
            let height = self.read8(HEIGHTS + u32::from(step));
            self.write8(r.a4.wrapping_add(BEFORE_HEIGHT), height);
            self.o_slot(r);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_00722e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            if self.restarted.is_none() {
                self.o_puff(r);
            }
            r.unwound = false;
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, STARTING);
        self.write32(r.a4 + OBJ_ANIM, START_ANIM);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.r_00f02a(r);
    }

    /// `func_00F02A`, its routine while `START_ANIM` runs: at its end
    /// (`func_001336`'s C) it holds (`HOLD_ANIM`, `HOLD_COUNT` frames,
    /// `func_00F048`); until then it goes as `func_00F082`.
    pub fn r_00f02a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, ended) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if !ended {
            self.o_end_hold(r);
            return;
        }
        self.write8(r.a4 + OBJ_COUNT, HOLD_COUNT);
        self.write32(r.a4 + OBJ_ANIM, HOLD_ANIM);
        self.write32(r.a4 + OBJ_ROUTINE, HOLDING);
        r.c = false;
        self.r_00f048(r);
    }

    /// `func_00F048`, holding: the animation goes on, a shot now and
    /// then (`func_00F102`), `OBJ_COUNT` counts down; at 0 the
    /// animation `END_ANIM` from the start, `func_00F072`.  Until then,
    /// carrying the object before, that one is moved with it.
    pub fn r_00f048(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6) = self.r_00f102(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        let count = self.read8(r.a4 + OBJ_COUNT);
        self.write8(r.a4 + OBJ_COUNT, count.wrapping_sub(1));
        if count != 1 {
            r.x = count == 0;
            if self.o_carrying(r) {
                (r.d0, r.x) = self.r_0015e6(r.a4);
            }
            self.o_move(r, false);
            return;
        }
        self.write32(r.a4 + OBJ_ANIM, END_ANIM);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write32(r.a4 + OBJ_ROUTINE, ENDING);
        r.x = false;
        r.c = false;
        self.r_00f072(r);
    }

    /// `func_00F072`: `END_ANIM` runs; at its end the routine is
    /// `func_00F000` again (and it goes as `func_00F082`).  Carrying
    /// the object before, its height is set by the sprite frame
    /// (`HEIGHTS_BY_FRAME_ON`) and it moves with it.
    pub fn r_00f072(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, ended) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if ended {
            self.write32(r.a4 + OBJ_ROUTINE, START);
            self.o_end_hold(r);
            return;
        }
        if self.o_carrying(r) {
            self.o_height_by_frame(r, HEIGHTS_BY_FRAME_ON);
            r.a0 = HEIGHTS_BY_FRAME_ON;
            self.o_slot(r);
            (r.d0, r.x) = self.r_0015e6(r.a4);
        }
        self.o_move(r, false);
    }

    /// `func_00F082` on the routines' registers.
    fn o_end_hold(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x, r.c) = self.r_00f082(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        r.unwound = false;
    }

    /// `func_00F082`: as `func_0071A0` moves it (`func_00F0B4`), or,
    /// carrying the object before, that one's height by the sprite
    /// frame (`HEIGHTS_BY_FRAME`) and both moved (`func_00725A`); a
    /// puff when they say so.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00f082(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let used = self.read8(a4 + OBJ_USED);
        if used != CARRYING {
            let (d0, d1, d2, d5, d6, a0, a5, a6, x, c) = self.r_00f0b4(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6);
            return (d0, d1, d2, d4, d5, d6, a0, a4, a5, a6, x, c);
        }
        let frame = self.read8(a5 + SPRITE_FRAME);
        let height = self.read8(HEIGHTS_BY_FRAME + u32::from(frame));
        self.write8(a4.wrapping_add(BEFORE_HEIGHT), height);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        self.r_0015e6(a4);
        let (d0, d1, d2, d4, d5, a0, a4, a5, x, c) = self.r_00725a(d1, d2, d3, d4, d5, d6, d7, HEIGHTS_BY_FRAME, a1, a2, a3, a4, a5);
        let (d5, d6, a5, a6, x, c) = self.r_00f1d0(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, x, c);
        (d0, d1, d2, d4, d5, d6, a0, a4, a5, a6, x, c)
    }

    /// `func_00F0B4`: `func_0071A0`, then a puff when it says so.
    /// Only `func_00F082` calls it; checked through it.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00f0b4(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (d0, d1, d2, a0, x, c) = self.r_0071a0(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        let (d5, d6, a5, a6, x, c) = self.r_00f1d0(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, x, c);
        (d0, d1, d2, d5, d6, a0, a5, a6, x, c)
    }

    /// `func_00F102`: when bits 0-1 of `OBJ_COUNT` are both set, a shot
    /// (`SHOT`) in the direction of the count (bit 2 clear: 0x80 added),
    /// moving at `SHOT_SPEED` (`func_00410C`'s velocity, kept in
    /// `TASK_VELOCITY`) from a step on; the object's sprite record
    /// given `SHOT_COMMAND`.  d4, a4, d5, a5 come back the object being
    /// run and its sprite record.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00f102(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let count = self.read8(a4 + OBJ_COUNT);
        let mut d0 = low_byte(d0, count & 3);
        if count & 3 != 3 {
            return (d0, d1, d2, d4, d5, d6, a4, a5, a6);
        }
        let direction = if count & 4 != 0 { count } else { count.wrapping_add(0x80) };
        d0 = low_byte(d0, direction);
        self.write8(DIRECTION, direction);
        self.r_001542(a4);
        let (mut d1, mut d2) = (d1, d2);
        let (_, d6, sprite, a6, x, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, SHOT_DEPTH, SHOT_PRIORITY, SHOT);
        if !failed {
            self.r_00152a(a6);
            let direction = self.read8(DIRECTION);
            self.write8(a6 + TASK_DIRECTION, direction);
            self.write8(sprite + SPRITE_FLAGS, 1);
            self.write16(SPEED, SHOT_SPEED);
            (d0, d1, d2, _) = self.r_00410c(low_byte(d0, direction), d1, d2, x);
            let velocity = self.read32(VELOCITY);
            self.write32(a6 + TASK_VELOCITY, velocity);
            (d0, d1, _, _, _) = self.r_001a88(d0, d1, a6);
            (d0, d1, _, _, _) = self.r_001aa2(d0, d1, a6);
            let slot = self.read32(SLOT_ADDR);
            d0 = low_byte(d0, SHOT_COMMAND);
            self.r_0013c0(d0, slot);
        }
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        (d0, d1, d2, d4, d5, d6, a4, a5, a6)
    }

    /// `func_00F192`, the shot's routine: its animation `SHOT_ANIM`;
    /// it ends with the animation (`func_00121A`); until then it hits
    /// what is at its cell (`func_003554`, `func_0075D6`) and moves by
    /// `TASK_VELOCITY`, and is drawn.
    pub fn r_00f192(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write32(r.a6 + TASK_ANIM, SHOT_ANIM);
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, ended) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if ended {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        let x;
        (r.d1, r.d2, x, _) = self.r_003554(8, r.d1, r.d2, r.a6);
        (r.d0, r.d1, r.a0, _, _) = self.r_0075d6(8, r.d1, r.d2, r.d6, r.a0, r.a6, x);
        self.write8(r.a6 + TASK_ANIM, SHOT_ANIM_BYTE);
        let velocity = self.read32(r.a6 + TASK_VELOCITY);
        self.write32(VELOCITY, velocity);
        (r.d0, r.d1, _, _, _) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, _, _, _) = self.r_001aa2(r.d0, r.d1, r.a6);
        (r.d0, _, _) = self.r_001b94(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b74(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
    }

    /// `func_00F1CC`, a kind's routine: it walks (`func_007174`), a puff
    /// when that says so.
    pub fn r_00f1cc(&mut self, regs: &mut Regs) {
        self.o_walk(regs);
    }

    /// `func_00F1D0`: with C, a puff task (`PUFF`, depth `PUFF_DEPTH`)
    /// with `PUFF_ANIM` and `PUFF_COUNT`, the object's sprite record
    /// hidden first (`func_001542`).  C: none made (or none asked).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00f1d0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, bool, bool) {
        if !c {
            return (d5, d6, a5, a6, x, c);
        }
        self.r_001542(a4);
        let (d5, d6, a5, a6, x, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, PUFF_DEPTH, PUFF_PRIORITY, PUFF);
        if failed {
            return (d5, d6, a5, a6, x, true);
        }
        self.r_00152a(a6);
        self.write32(a6 + TASK_ANIM, PUFF_ANIM);
        self.write8(a6 + TASK_COUNT, PUFF_COUNT);
        self.write8(a5 + SPRITE_FLAGS, 1);
        (d5, d6, a5, a6, x, false)
    }

    /// `func_00F20E`, a kind's routine: set up the first time (kind
    /// `THROW_KIND`, its animation from the start, `func_00F23A`);
    /// after that as `loc_00F4AA`.
    pub fn r_00f20e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        if self.o_started(r) {
            self.r_00f4aa(r);
            r.unwound = false;
            return;
        }
        self.write8(r.a4 + OBJ_KIND, THROW_KIND);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write32(r.a4 + OBJ_ROUTINE, THROWING);
        r.c = false;
        self.r_00f23a(r);
        r.unwound = false;
    }

    /// `func_00F23A`: its animation runs (at its end, kind 0 and
    /// `func_00F20E` again); at step `THROW_STEP` a thrown task
    /// (`THROWN`) in its direction (`OBJ_DIRECTION`, bits 0-1), its
    /// animation by the direction (D_00F2F6), from a step on at
    /// `THROWN_SPEED`, the sprite record given `THROWN_COMMAND`.  It
    /// moves as `func_0071A0`, with a puff (`func_00F4AE`).
    pub fn r_00f23a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, ended) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        r.c = ended;
        if ended {
            self.write32(r.a4 + OBJ_ROUTINE, THROW_START);
            self.write8(r.a4 + OBJ_KIND, 0);
        } else {
            let timer = self.read8(r.a4 + OBJ_TIMER);
            r.d0 = low_byte(r.d0, timer);
            if timer == 0 && self.read8(r.a4 + OBJ_STEP) == THROW_STEP {
                self.o_throw(r);
            }
        }
        (r.d0, r.x) = self.r_0015e6(r.a4);
        self.o_move(r, true);
    }

    /// `func_00F23A`'s throw.
    fn o_throw(&mut self, r: &mut Regs) {
        self.r_001542(r.a4);
        let direction = self.read8(r.a4 + OBJ_DIRECTION);
        self.write8(DIRECTION, direction);
        let failed;
        (r.d5, r.d6, r.a5, r.a6, r.x, failed) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, THROWN_DEPTH, THROWN_PRIORITY, THROWN);
        r.c = failed;
        if !failed {
            self.r_00152a(r.a6);
            self.write8(r.a5 + SPRITE_FLAGS, 1);
            let direction = self.read8(DIRECTION) & 3;
            self.write8(r.a6 + TASK_DIRECTION, direction);
            r.d0 = low_word(r.d0, u16::from(direction) * 4);
            let anim = self.read32(THROWN_ANIMS + u32::from(direction) * 4);
            self.write32(r.a6 + TASK_ANIM, anim);
            self.write16(SPEED, THROWN_SPEED);
            r.d0 = low_byte(r.d0, direction);
            self.r_001620(r.d0);
            (r.d0, r.d1, _, _, _) = self.r_001a88(r.d0, r.d1, r.a6);
            (r.d0, r.d1, _, _, _) = self.r_001aa2(r.d0, r.d1, r.a6);
            self.write16(r.a6 + TASK_SPEED, THROWN_TASK_SPEED);
            self.o_slot(r);
            r.d0 = low_byte(r.d0, THROWN_COMMAND);
            r.c = self.r_0013c0(r.d0, r.a5);
        }
        self.o_running(r);
        self.o_slot(r);
    }

    /// `func_00F306`, the thrown task's routine: animated, moved by its
    /// velocity at its speed (`TASK_SPEED` to `SPEED`; `func_00C742`
    /// changes it), drawn; then it lands, whatever the cell it is over
    /// but 6 (on 6 only if `func_003554` gives C): a cell 2 is made 0
    /// (`func_00168C`, `func_001278`, `func_0052F0`) and 2 or 5 taken
    /// (`func_0066FE`); on 1, `func_005E0E` is asked first.  Landing:
    /// `LANDED_ANIM` from the start, routine `func_00F3AA` from now.
    pub fn r_00f306(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        r.d0 = 0x10;
        self.r_00c742(r.d0, r.a6);
        let speed = self.read16(r.a6 + TASK_SPEED);
        self.write16(SPEED, speed);
        let direction = self.read8(r.a6 + TASK_DIRECTION);
        r.d0 = low_byte(r.d0, direction);
        self.r_001620(r.d0);
        (r.d0, r.d1, _, _, _) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, _, _, _) = self.r_001aa2(r.d0, r.d1, r.a6);
        (r.d0, _, _) = self.r_001b7e(r.a5, r.a6);
        (r.d0, _, _) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        r.d0 = self.r_0015f8(r.a6);
        (r.a0, r.x) = self.r_00167a();
        let cell = self.read8(r.a0);
        r.d0 = low_byte(r.d0, cell);
        if cell == 1 {
            (r.x, _, _) = self.r_005e0e(r.d2, r.d6, r.a6);
        }
        if cell < 7 {
            match cell {
                2 => {
                    (r.d0, r.a0) = self.r_00168c(r.d0);
                    self.r_001278();
                    (r.x, _) = self.r_0052f0(r.d0);
                    (r.d2, r.x) = self.r_0066fe(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
                }
                5 => {
                    (r.d2, r.x) = self.r_0066fe(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
                }
                6 => {
                    r.d0 = 8;
                    let missed;
                    (r.d1, r.d2, r.x, missed) = self.r_003554(r.d0, r.d1, r.d2, r.a6);
                    if !missed {
                        r.c = false;
                        return;
                    }
                }
                _ => {}
            }
        }
        (r.x, _) = self.r_00053e(r.d6, LANDED);
        self.write32(r.a6 + TASK_ANIM, LANDED_ANIM);
        self.write8(r.a6 + TASK_TIMER, 0);
        self.write8(r.a6 + TASK_STEP, 0);
        r.c = false;
        self.r_00f3aa(r);
        r.unwound = false;
    }

    /// `func_00F3AA`, landed: animated and drawn until the animation
    /// ends, then the task ends (`func_00121A`).
    pub fn r_00f3aa(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, ended) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if ended {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        (r.d0, _, _) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// The cell in front of the object (its direction, `func_00173E`):
    /// d0's low byte its value.
    fn o_cell_ahead(&mut self, r: &mut Regs) -> u8 {
        (r.d0, r.x) = self.r_0015e6(r.a4);
        let direction = self.read8(r.a4 + OBJ_DIRECTION) & 3;
        r.d0 = low_byte(r.d0, direction);
        r.x = self.r_00173e(r.d0, r.a4);
        r.a0 = self.read32(CELL_AT);
        let cell = self.read8(r.a0);
        r.d0 = low_byte(r.d0, cell);
        cell
    }

    /// `func_00F3C6`, a kind's routine: while the cell in front isn't
    /// `DIG_ON` it goes as `loc_00F4AA`; when it is, kind `THROW_KIND`,
    /// its animation from the start and `func_00F3FC`.
    pub fn r_00f3c6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let cell = self.o_cell_ahead(r);
        if cell != DIG_ON {
            r.c = cell < DIG_ON;
            self.r_007174(r);
            if self.restarted.is_none() {
                self.o_puff_2(r);
            }
            r.unwound = false;
            return;
        }
        self.write8(r.a4 + OBJ_KIND, THROW_KIND);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write32(r.a4 + OBJ_ROUTINE, DIGGING);
        r.c = false;
        self.r_00f3fc(r);
        r.unwound = false;
    }

    /// `func_00F3FC`: its animation runs (at its end kind 0 and
    /// `func_00F3C6` again); at step `DIG_STEP` the sprite record is
    /// given `DIG_COMMAND` and, the cell in front still `DIG_ON`, that
    /// cell is made 0 (`func_00168C`, `func_001278`, `func_0052F0`,
    /// `func_0066FE`) with a task (`PUFF`, depth 2, `DIG_ANIM`).  It
    /// moves as `func_0071A0`, with a puff (`func_00F4AE`).
    pub fn r_00f3fc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, ended) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        r.c = ended;
        if ended {
            self.write8(r.a4 + OBJ_KIND, 0);
            self.write32(r.a4 + OBJ_ROUTINE, DIG_START);
        } else {
            let timer = self.read8(r.a4 + OBJ_TIMER);
            r.d0 = low_byte(r.d0, timer);
            if timer == 0 && self.read8(r.a4 + OBJ_STEP) == DIG_STEP {
                r.d0 = low_byte(r.d0, DIG_COMMAND);
                r.c = self.r_0013c0(r.d0, r.a5);
                if self.o_cell_ahead(r) == DIG_ON {
                    self.o_dig(r);
                }
            }
        }
        (r.d0, r.x) = self.r_0015e6(r.a4);
        self.o_move(r, true);
    }

    /// `func_00F3FC`'s cell made 0, with its task.
    fn o_dig(&mut self, r: &mut Regs) {
        (r.d0, r.a0) = self.r_00168c(r.d0);
        self.r_001278();
        (r.x, _) = self.r_0052f0(r.d0);
        (r.d2, r.x) = self.r_0066fe(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        let failed;
        (r.d5, r.d6, r.a5, r.a6, r.x, failed) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, PUFF_DEPTH_2, PUFF_PRIORITY, PUFF);
        r.c = failed;
        if !failed {
            self.write32(r.a6 + TASK_ANIM, DIG_ANIM);
            (r.d0, r.x) = self.r_001550(r.d0, r.a6);
            self.write8(r.a5 + SPRITE_FLAGS, 1);
            self.write8(r.a6 + TASK_COUNT, 1);
        }
        self.o_slot(r);
        self.o_running(r);
    }

    /// `loc_00F4AA`, a kind's routine: it walks (`func_007174`), a puff
    /// (`func_00F4AE`) when that says so.
    pub fn r_00f4aa(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.r_007174(r);
        if self.restarted.is_none() {
            self.o_puff_2(r);
        }
        r.unwound = false;
    }

    /// `func_00F4AE`: as `func_00F1D0` with depth `PUFF_DEPTH_2` and
    /// `PUFF_ANIM_2`.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00f4ae(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, bool, bool) {
        if !c {
            return (d5, d6, a5, a6, x, c);
        }
        self.r_001542(a4);
        let (d5, d6, a5, a6, x, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, PUFF_DEPTH_2, PUFF_PRIORITY, PUFF);
        if failed {
            return (d5, d6, a5, a6, x, true);
        }
        self.write32(a6 + TASK_ANIM, PUFF_ANIM_2);
        self.r_00152a(a6);
        self.write8(a5 + SPRITE_FLAGS, 1);
        self.write8(a6 + TASK_COUNT, PUFF_COUNT);
        (d5, d6, a5, a6, x, false)
    }
}
