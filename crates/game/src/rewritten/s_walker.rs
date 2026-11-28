//! An object's routines (0x1401A-0x14380): `func_01401A` and the ones it
//! goes on as (`OBJ_ROUTINE`).  It walks at `WALK_SPEED` a cell at a
//! time (`func_001620`, `func_0019C0`) in a direction drawn at random
//! every few cells, turned back at x 0x38 and 0xD8; now and then it
//! stops (`func_0140FC`, counted in `STOPPED`) and runs an animation
//! during which, at its steps 0x36-0x3D, it makes two tasks
//! (`func_014188`) that fly off from it and move along their way
//! (`func_0142B2`).  Its sprite record's byte 3 is copied from
//! `LEADER`'s each frame (`func_01479A`).

use crate::{Game, Regs};

/// The object's fields: its routine (a long), a direction (bits 0-1,
/// `func_001620`'s), a flag (0xFF once it is in the middle of a cell,
/// cleared when it isn't), its x's (a word) high and low bytes, its
/// y's high byte, the animation (a long),
/// the animation's timer and step, how many cells it walks before a
/// new direction is drawn, and bytes 0x2B (cleared by
/// `func_01401A`); its sprite record (`func_0019C0`).
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_DIRECTION: u32 = 0x13;
const OBJ_CENTRED: u32 = 0x14;
const OBJ_X_HIGH: u32 = 0x18;
const OBJ_X_LOW: u32 = 0x19;
const OBJ_Y_HIGH: u32 = 0x1A;
const OBJ_ANIM: u32 = 0x20;
const OBJ_ANIM_TIMER: u32 = 0x24;
const OBJ_ANIM_STEP: u32 = 0x25;
const OBJ_CELLS: u32 = 0x27;
const OBJ_2B: u32 = 0x2B;
/// The made tasks' fields: their position (words), their count (they
/// end at 0), their frame (with 1 added on odd frames of `FRAME`'s bit
/// 1) and their velocity (x, its fraction, y, its fraction: bytes
/// 0x18-0x1B, the order of `func_00410C`'s `VELOCITY_*`); its offset.
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_COUNT: u32 = 0x16;
const TASK_FRAME: u32 = 0x17;
const TASK_VELOCITY_X_FRACTION: u32 = 0x18;
const TASK_VELOCITY_X: u32 = 0x19;
const TASK_VELOCITY_Y_FRACTION: u32 = 0x1A;
const TASK_VELOCITY_Y: u32 = 0x1B;
const TASK_OFFSET: u32 = 0x20;
/// An object's sprite record's frame, flags, y (a word) and byte 3.
const SPRITE_BYTE_3: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_Y: u32 = 0x08;

/// The velocity (bytes: x, its fraction, y, its fraction) `func_0019C0`
/// and `func_0019DA` move by; `func_00410C` sets it.
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_X_FRACTION: u32 = 0xFF9EC5;
const VELOCITY_Y: u32 = 0xFF9EC6;
const VELOCITY_Y_FRACTION: u32 = 0xFF9EC7;
/// The speed (a word) `func_001620` makes the velocity of.
const SPEED: u32 = 0xFF9F00;
/// A position (words, x then y): `func_001542` and `func_001300`.
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
/// The frame count: its bit 1 picks the made tasks' frame.
const FRAME: u32 = 0xFF80D8;
/// The new task's index (`task.rs`).
const NEW_TASK: u32 = 0xFF80EF;
/// The object being run (its index and address) and its sprite
/// record's (index, address); `func_014188` puts them back.
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// A sprite record's index (its low byte), read by `func_014188` for
/// `func_000B5A`: the byte after `SLOT_INDEX`'s word.
const SLOT_INDEX_LOW: u32 = 0xFF8A59;
/// The sprite record whose byte 3 the object's copies (`func_01479A`).
const LEADER_ADDR: u32 = 0xFFA63C;
/// A byte: the y's high byte of the made tasks (`func_014188`).
const BYTE_A61C: u32 = 0xFFA61C;
/// How often the object stopped: `func_0140FC` adds 1 (2 half the
/// time), `func_014142` clears it when the animation ends;
/// `func_01412A` walks on when it is 0.  `func_014380` clears it too.
const STOPPED: u32 = 0xFFAD56;
/// `func_014188`'s count (2 tasks: from an even one to the next even
/// one) and the tasks' angle, their y step a word, their speed.
const MADE: u32 = 0xFFAABD;
const MADE_ANGLE: u32 = 0xFF9679;
const MADE_STEP: u32 = 0xFF967A;
/// A count of frames for `func_0142B2`'s moves (the low byte of
/// `MADE_STEP`'s word).
const MOVES: u32 = 0xFF967A;
/// Bit 6 of one, bit 0 of the other, cleared by `func_01401A`.
const BITS_9540: u32 = 0xFF9540;
const SPARED: u32 = 0xFF9548;

/// `func_01401A`'s step of the rate (-2, `func_004424`), and the
/// offset (`func_001ABC`) under which it walks.
const RATE_STEP: u8 = 0xFE;
const OFFSET_LANDED: u8 = 0xF2;
/// The walking animation, the stopped one.
const WALK_ANIM: u32 = 0x0E868A;
const STOP_ANIM: u32 = 0x0E8646;
const WALK_SPEED: u16 = 0x99;
/// The x where it turns: right at the one, left at the other.
const LEFT_TURN: u8 = 0x38;
const RIGHT_TURN: u8 = 0xD8;
/// The animation's steps at which tasks are made, the Z80's command
/// then.
const MAKE_FROM: u8 = 0x36;
const MAKE_TO: u8 = 0x3E;
const MAKE_SOUND: u8 = 0x71;
/// The made tasks: their routine, depth and priority, sprite flags,
/// speed, offset, count; their y step's count, the count at which
/// their frame goes 2 on and 2 back.
const ROUTINE_142B2: u32 = 0x0142B2;
const MADE_DEPTH: u8 = 3;
const MADE_PRIORITY: u8 = 0x42;
const MADE_FLAGS: u8 = 0xC1;
const MADE_SPEED: u16 = 0x1000;
const MADE_OFFSET: u8 = 0x70;
const MADE_COUNT: u8 = 0x28;
const MADE_MOVES: u8 = 0x0A;
const FRAME_UP_AT: u8 = 0x22;
const FRAME_DOWN_AT: u8 = 0x06;
/// The tables of `func_014188` by `MADE`: the first by the animation's
/// step / 2 (0x1B-0x1E; the bytes after `func_0142B2`'s rts, 0x1435E,
/// read from `func_0142B2` + 0x91), then the tasks' angle, y step,
/// frame, velocity x and y (D_014362 - D_01437A).
const MADE_FIRST: u32 = 0x014343;
const ANGLES: u32 = 0x014362;
const STEPS: u32 = 0x014368;
const FRAMES: u32 = 0x01436E;
const VELOCITIES_X: u32 = 0x014374;
const VELOCITIES_Y: u32 = 0x01437A;
/// Objects 7 and 1 (`func_001CFA`): the position is taken from 7 and
/// moved by 1.
const OBJECT_7: u8 = 7;
const OBJECT_1: u8 = 1;
/// What the cell under a made task is compared with (1), and what
/// `func_0034DE` is told for it.
const CELL_PUT: u8 = 1;
const CELL_HIT: u32 = 8;

/// The routines it goes on as.
const ROUTINE_14054: u32 = 0x014054;
const ROUTINE_140FC: u32 = 0x0140FC;
const ROUTINE_1412A: u32 = 0x01412A;
const ROUTINE_14142: u32 = 0x014142;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_001336` on the family's registers: the animation a frame
    /// on (C: it ended).
    fn s_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
    }

    /// `func_01479A` on the family's registers.
    fn s_copy_leader(&mut self, r: &mut Regs) {
        (r.d0, r.d5, r.a5, r.c) = self.r_01479a(r.d0, r.d5);
    }

    /// `func_01401A`, an object's routine: its animation a frame on
    /// (`func_001336`) and its offset down by 2 (`func_004424`,
    /// `func_001ABC`); while that is 0xF2 or more it goes on as
    /// `.L013F98`; then it walks (`func_014054` from now on), a cell
    /// before a new direction, bits 6 of `BITS_9540` and 0 of `SPARED`
    /// cleared.
    pub fn r_01401a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.s_animate(r);
        r.d0 = low_byte(r.d0, RATE_STEP);
        (r.d0, r.x, _) = self.r_004424(r.d0, r.a4);
        (r.d0, r.x, _) = self.r_001abc(r.d0, r.a4);
        if r.d0 as u8 >= OFFSET_LANDED {
            (r.d0, r.d1, r.x, r.c) = self.r_013f98(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            return;
        }
        self.write8(r.a4 + OBJ_2B, 0);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14054);
        self.write8(r.a4 + OBJ_CELLS, 1);
        self.write8(r.a4 + OBJ_CENTRED, 0);
        let v = self.read8(BITS_9540) & !0x40;
        self.write8(BITS_9540, v);
        let v = self.read8(SPARED) & !0x01;
        self.write8(SPARED, v);
        self.r_014054(r);
    }

    /// `func_014054`, an object's routine: it walks.  Its animation is
    /// `WALK_ANIM`, run a frame on (`func_001336`); each time it comes
    /// to the middle of a cell (`func_00182E`; `OBJ_CENTRED` keeps it
    /// from counting a cell twice) its count of cells goes down, and at
    /// 0 a random number's sign may stop it (`func_0140FC` from now
    /// on), the count becomes 3 or 7 and the direction 1 or 3 (at
    /// random).  At x 0x38 it turns right (1), at 0xD8 left (3).  Then
    /// it moves at `WALK_SPEED` (`func_001620`, `func_0019C0`),
    /// `func_012A8C`, its sprite is placed (`func_001B32`,
    /// `func_001B50`) and gets `LEADER`'s byte 3 (`func_01479A`).
    pub fn r_014054(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write32(r.a4 + OBJ_ANIM, WALK_ANIM);
        self.s_animate(r);
        let centred;
        (r.d0, centred, r.c) = self.r_00182e(r.d0, r.a4);
        if !centred {
            self.write8(r.a4 + OBJ_CENTRED, 0);
        } else {
            let was = self.read8(r.a4 + OBJ_CENTRED);
            r.d0 = low_byte(r.d0, was);
            if was == 0 {
                self.write8(r.a4 + OBJ_CENTRED, 0xFF);
                let cells = self.read8(r.a4 + OBJ_CELLS).wrapping_sub(1);
                self.write8(r.a4 + OBJ_CELLS, cells);
                if cells == 0 {
                    (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
                    if (r.d0 as u8 as i8) >= 0 {
                        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_140FC);
                    }
                    (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
                    r.d0 = low_byte(r.d0, (r.d0 as u8 & 7) | 3);
                    self.write8(r.a4 + OBJ_CELLS, r.d0 as u8);
                    (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
                    r.d0 = low_byte(r.d0, (r.d0 as u8 & 2) | 1);
                    self.write8(r.a4 + OBJ_DIRECTION, r.d0 as u8);
                }
            }
        }
        let x_low = self.read8(r.a4 + OBJ_X_LOW);
        r.d0 = low_byte(r.d0, x_low);
        if x_low == LEFT_TURN {
            self.write8(r.a4 + OBJ_DIRECTION, 1);
        }
        if x_low == RIGHT_TURN {
            self.write8(r.a4 + OBJ_DIRECTION, 3);
        }
        self.write16(SPEED, WALK_SPEED);
        r.d0 = low_byte(r.d0, self.read8(r.a4 + OBJ_DIRECTION));
        self.r_001620(r.d0);
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
        (r.d0, r.d5, r.a5, r.x) = self.r_012a8c(r.d0, r.d5, r.a4);
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        (r.d0, r.x, r.c) = self.r_001b50(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.s_copy_leader(r);
    }

    /// `func_0140FC`, an object's routine: it stops.  `STOPPED` goes up
    /// by 1; then half the time (a random number's sign) it waits for
    /// `STOPPED` to be cleared (`func_01412A`), else `STOPPED` goes up
    /// again and it runs `STOP_ANIM` (`func_014142`).
    pub fn r_0140fc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let v = self.read8(STOPPED).wrapping_add(1);
        self.write8(STOPPED, v);
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        if (r.d0 as u8 as i8) >= 0 {
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_1412A);
            r.c = false;
            self.r_01412a(r);
            return;
        }
        let was = self.read8(STOPPED);
        self.write8(STOPPED, was.wrapping_add(1));
        r.x = was == 0xFF;
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14142);
        self.write32(r.a4 + OBJ_ANIM, STOP_ANIM);
        r.c = false;
        self.r_014142(r);
    }

    /// `func_01412A`, an object's routine: it waits; when `STOPPED` is
    /// 0 it walks again (`func_014054` from the next frame).  Its
    /// sprite gets `LEADER`'s byte 3 (`func_01479A`).
    pub fn r_01412a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let stopped = self.read8(STOPPED);
        r.d0 = low_byte(r.d0, stopped);
        if stopped == 0 {
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14054);
        }
        self.s_copy_leader(r);
    }

    /// `func_014142` (`loc_014142`), an object's routine: its animation
    /// a frame on (`func_001336`); when it ends, `STOPPED` is cleared
    /// and it walks again (`func_014054`).  At the animation's steps
    /// 0x36-0x3D (its timer 0) it makes two tasks (`func_014188`) and
    /// the Z80 is told `MAKE_SOUND`.  Its sprite gets `LEADER`'s byte 3
    /// (`func_01479A`).
    pub fn r_014142(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.s_animate(r);
        if r.c {
            self.write8(STOPPED, 0);
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14054);
        }
        let timer = self.read8(r.a4 + OBJ_ANIM_TIMER);
        r.d0 = low_byte(r.d0, timer);
        if timer == 0 {
            let step = self.read8(r.a4 + OBJ_ANIM_STEP);
            r.d0 = low_byte(r.d0, step);
            if (MAKE_FROM..MAKE_TO).contains(&step) {
                (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a6, r.x) = self.r_014188(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
                r.d0 = low_byte(r.d0, MAKE_SOUND);
                self.r_0034b6(r.d0);
            }
        }
        self.s_copy_leader(r);
    }

    /// `func_014188`: two tasks of `func_0142B2` made (`func_001222`,
    /// from the count `MADE` that the first table gives by the
    /// animation's step `d0` / 2, to the next even count), each with
    /// its frame, velocity and angle from the tables by the count, at
    /// the position of object 7 moved by object 1 by the velocity
    /// (`func_0019C0`, `func_0019DA`), its y in the low byte
    /// `BYTE_A61C`, moved by the y step, its velocity at `MADE_SPEED`
    /// by the angle (`func_00410C`).  The running object and sprite
    /// record are put back.  Gives back d0, d1, d2, d4, d5, d6, a0
    /// (`FRAMES`), a4, a6 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_014188(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let (mut d0, mut d1, mut d2, mut d4, mut d5, mut d6) = (d0, d1, d2, d4, d5, d6);
        let (mut a4, mut a5) = (a4, a5);
        let mut a6;
        let mut x;
        d0 = low_byte(d0, (d0 as u8) >> 1);
        d0 = low_word(d0, d0 as u8 as i8 as u16);
        let v = self.read8(MADE_FIRST.wrapping_add(d0 as u16 as i16 as u32));
        self.write8(MADE, v);
        self.write16(SPEED, MADE_SPEED);
        loop {
            (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, MADE_DEPTH, MADE_PRIORITY, ROUTINE_142B2);
            self.write8(a5 + SPRITE_FLAGS, MADE_FLAGS);
            let made = self.read8(MADE);
            d0 = low_word(d0, made as i8 as u16);
            let i = made as i8 as u32;
            let v = self.read8(VELOCITIES_X.wrapping_add(i));
            self.write8(VELOCITY_X, v);
            let v = self.read8(VELOCITIES_Y.wrapping_add(i));
            self.write8(VELOCITY_Y, v);
            let v = self.read8(ANGLES.wrapping_add(i));
            self.write8(MADE_ANGLE, v);
            let step = self.read8(STEPS.wrapping_add(i));
            d1 = low_word(d1, step as i8 as u16);
            self.write16(MADE_STEP, d1 as u16);
            let frame = self.read8(FRAMES.wrapping_add(i));
            d1 = low_byte(d1, frame);
            d5 = low_byte(d5, self.read8(SLOT_INDEX_LOW));
            (d5, a5, _) = self.r_000b5a(d5);
            self.write8(a5 + SPRITE_FRAME, frame);
            self.write8(a6 + TASK_FRAME, frame);
            self.write8(VELOCITY_X_FRACTION, 0);
            self.write8(VELOCITY_Y_FRACTION, 0);
            d4 = low_byte(d4, OBJECT_7);
            (d4, a4, _) = self.r_001cfa(d4);
            self.r_001542(a4);
            d4 = low_byte(d4, OBJECT_1);
            (d4, a4, _) = self.r_001cfa(d4);
            self.r_001300(a4);
            (d0, d1, _) = self.r_0019c0(d0, d1, a4);
            (_, d1, _) = self.r_0019da(d0, d1, a4);
            self.r_001542(a4);
            d6 = low_byte(d6, self.read8(NEW_TASK));
            (d6, a6, _) = self.r_0005f8(d6);
            let y = (0xFF00 | u16::from(self.read8(BYTE_A61C))).wrapping_add(self.read16(POSITION_Y));
            self.write16(a6 + TASK_Y, y);
            let v = self.read16(POSITION_X);
            self.write16(a6 + TASK_X, v);
            self.write8(a6 + TASK_OFFSET, MADE_OFFSET);
            (d0, _, _) = self.r_001b94(a5, a6);
            let step = self.read16(MADE_STEP);
            d0 = low_word(d0, step);
            let sprite_y = self.read16(a5 + SPRITE_Y);
            self.write16(a5 + SPRITE_Y, sprite_y.wrapping_add(step));
            x = sprite_y.checked_add(step).is_none();
            d0 = low_byte(d0, self.read8(MADE_ANGLE));
            (d0, d1, d2, _) = self.r_00410c(d0, d1, d2, x);
            let v = self.read8(VELOCITY_X_FRACTION);
            self.write8(a6 + TASK_VELOCITY_X_FRACTION, v);
            let v = self.read8(VELOCITY_X);
            self.write8(a6 + TASK_VELOCITY_X, v);
            let v = self.read8(VELOCITY_Y_FRACTION);
            self.write8(a6 + TASK_VELOCITY_Y_FRACTION, v);
            let v = self.read8(VELOCITY_Y);
            self.write8(a6 + TASK_VELOCITY_Y, v);
            self.write8(a6 + TASK_COUNT, MADE_COUNT);
            let was = self.read8(MADE);
            self.write8(MADE, was.wrapping_add(1));
            x = was == 0xFF;
            if was.wrapping_add(1) & 1 == 0 {
                break;
            }
        }
        d4 = low_byte(d4, self.read8(RUNNING));
        a4 = self.read32(RUNNING_ADDR);
        d5 = low_byte(d5, self.read8(SLOT_INDEX));
        (d0, d1, d2, d4, d5, d6, FRAMES, a4, a6, x)
    }

    /// `func_0142B2`, the routine of `func_014188`'s tasks: at count 0
    /// the task ends (`func_00121A`); else its sprite gets its frame
    /// (one more on frames with `FRAME`'s bit 1), its count goes down
    /// (the frame 2 on at 0x22, 2 back at 6), and object 1 is moved to
    /// the task's position by the task's velocity `MADE_MOVES` times
    /// (`func_0019C0`, `func_0019DA`; fewer once its x or y is 0x100 or
    /// more), telling the Z80 of the cell it is on
    /// (`func_0034DE`) and stopping the put task there (`func_005E0E`)
    /// if the cell is `CELL_PUT`.
    pub fn r_0142b2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(r.a6 + TASK_COUNT);
        r.d0 = low_byte(r.d0, count);
        if count == 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        let mut frame = self.read8(r.a6 + TASK_FRAME);
        if self.read8(FRAME) & 0x02 != 0 {
            frame = frame.wrapping_add(1);
        }
        self.write8(r.a5 + SPRITE_FRAME, frame);
        r.d0 = low_byte(r.d0, count);
        self.write8(r.a6 + TASK_COUNT, count.wrapping_sub(1));
        if count == FRAME_UP_AT {
            let v = self.read8(r.a6 + TASK_FRAME).wrapping_add(2);
            self.write8(r.a6 + TASK_FRAME, v);
        } else if count == FRAME_DOWN_AT {
            let v = self.read8(r.a6 + TASK_FRAME).wrapping_sub(2);
            self.write8(r.a6 + TASK_FRAME, v);
        }
        self.write8(MOVES, MADE_MOVES);
        let v = self.read8(r.a6 + TASK_VELOCITY_X_FRACTION);
        self.write8(VELOCITY_X_FRACTION, v);
        let v = self.read8(r.a6 + TASK_VELOCITY_X);
        self.write8(VELOCITY_X, v);
        let v = self.read8(r.a6 + TASK_VELOCITY_Y_FRACTION);
        self.write8(VELOCITY_Y_FRACTION, v);
        let v = self.read8(r.a6 + TASK_VELOCITY_Y);
        self.write8(VELOCITY_Y, v);
        self.r_00153a(r.a6);
        r.d4 = u32::from(OBJECT_1);
        (r.d4, r.a4, r.x) = self.r_001cfa(r.d4);
        self.r_001300(r.a4);
        r.c = false;
        loop {
            (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
            if self.read8(r.a4 + OBJ_X_HIGH) != 0 {
                break;
            }
            (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
            if self.read8(r.a4 + OBJ_Y_HIGH) != 0 {
                break;
            }
            r.d0 = CELL_HIT;
            (r.d1, r.d2, r.x) = self.r_0034de(r.d0, r.d1, r.d2, r.a4);
            (r.d0, r.x) = self.r_0015e6(r.a4);
            (r.a0, r.x) = self.r_00167a();
            let cell = self.read8(r.a0);
            r.d0 = low_byte(r.d0, cell);
            if cell == CELL_PUT {
                let _ = self.r_005e0e(r.d2, r.d6, r.a6);
            }
            let moves = self.read8(MOVES).wrapping_sub(1);
            self.write8(MOVES, moves);
            r.x = false;
            if moves == 0 {
                break;
            }
        }
        r.c = false;
    }

    /// `func_01479A`: the running object's sprite record gets byte 3
    /// of `LEADER`'s.  Gives back d0 (that byte), d5 (`SLOT_INDEX`), a5
    /// (`SLOT_ADDR`) and C (clear).
    pub fn r_01479a(&mut self, d0: u32, d5: u32) -> (u32, u32, u32, bool) {
        let leader = self.read32(LEADER_ADDR);
        let byte = self.read8(leader + SPRITE_BYTE_3);
        let a5 = self.read32(SLOT_ADDR);
        self.write8(a5 + SPRITE_BYTE_3, byte);
        (low_byte(d0, byte), low_byte(d5, self.read8(SLOT_INDEX)), a5, false)
    }
}
