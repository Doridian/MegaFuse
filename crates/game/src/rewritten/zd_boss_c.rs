//! 0x1DC7C-0x1E12A: object 7 of the boss of `zd_boss_b.rs`, and the
//! objects it throws.  Object 7 (`func_01DC7C`) stays 0x48 above the
//! boss; when the boss asks (`STEP`, `func_01E374`) it swings round a
//! circle (`func_01DD58`, `func_01DD9E`: its centre half-way to object
//! 0, its radius at most 0xA0; `func_01DE02`, `func_01DE14`: half a
//! turn at `STEP` - 5 a frame) and back.  Once the boss is beaten it
//! makes four tasks (`func_01DE66`, `func_01DEC4`) and ends
//! (`func_01DEFC`), setting bit 0 of `STATE`.  `func_01DE36` and
//! `func_01DE4E` are the circle's x and y (`func_017780`,
//! `func_01777C`: `ANGLE`, `RADIUS`).  Objects 8-0xF (`func_01E3A0`'s)
//! spread out round a circle (`func_01DF1E`, `func_01DFA4`,
//! `func_01DFC0`, `func_01DFDC`), then fly after object 0
//! (`func_01E032`, turning a step at a time) at speed 3 until they
//! leave (`func_01E078`, `func_01E09A`) or meet it (`func_01E0BC`,
//! `func_01E0FC`); the last one gone clears `STATE` (unless the boss is
//! beaten).
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read.

use crate::{Game, Regs};

/// An object's fields: its first byte (0: free), its routine (a long),
/// its animation (a long), its position (words: x, y), its offset; and
/// for the thrown objects an angle and radius (0x26, 0x27; the
/// angle's high byte also object 7's count), the phase (0: round the
/// circle, else flying) and a count (0x29).
const OBJ_ID: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_ANIM: u32 = 0x20;
const OBJ_X: u32 = 0x18;
const OBJ_Y: u32 = 0x1A;
const OBJ_ANGLE: u32 = 0x26;
const OBJ_COUNT: u32 = 0x26;
const OBJ_RADIUS: u32 = 0x27;
const OBJ_PHASE: u32 = 0x28;
const OBJ_29: u32 = 0x29;
const OBJ_OFFSET: u32 = 0x2A;
/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
/// A task's animation (a long).
const TASK_ANIM: u32 = 0x20;

/// A position (words: x then y; `func_001300`, `func_001542`).
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
/// The speed (a word) `func_00410C` makes the velocity of, and the
/// velocity (words: x, y).
const SPEED: u32 = 0xFF9F00;
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;
/// Two points (words): `func_0041D0` gives the angle of A - B.
const FROM_X: u32 = 0xFF9EFA;
const FROM_Y: u32 = 0xFF9EFE;
const TO_X: u32 = 0xFF9EE6;
const TO_Y: u32 = 0xFF9EE8;
/// Object 0's position (words).
const OBJECT_0_X: u32 = 0xFFA44A;
const OBJECT_0_Y: u32 = 0xFFA44C;

/// The boss's state (`zd_boss_b.rs`): negative once beaten; bit 0 set
/// when object 7 ends, bit 7 kept (the rest cleared) when its swing
/// ends.
const STATE: u32 = 0xFFAEEE;
/// The centre of the circles (words: x, y).
const CENTRE_X: u32 = 0xFFAEF2;
const CENTRE_Y: u32 = 0xFFAEF4;
/// The angle, the angle object 7's swing ends at, and the radius of
/// `func_01DE36` and `func_01DE4E`.
const ANGLE: u32 = 0xFFAF01;
const END_ANGLE: u32 = 0xFFAF02;
const RADIUS: u32 = 0xFFAF03;
/// Set while the boss is hurt (`func_01E428`): object 7's offset 0.
const HURT: u32 = 0xFFAF04;
/// The animation the boss's task shows (`func_01DB0C`): object 7
/// stays above the boss while it is set.
const SHOWN: u32 = 0xFFAF07;
/// Object 7's swing: 1 starts it, then 7 or 3 (each frame the angle
/// moves by it less 5), 0 when not swinging.
const STEP: u32 = 0xFFAF09;
/// Set when a thrown object meets object 0 (`func_01E1CA` reads it).
const MET: u32 = 0xFFAF12;
/// The thrown objects left.
const THROWN_LEFT: u32 = 0xFFAF13;
/// A word: what the last sum of `func_01DD9E`, `func_01E078` and
/// `func_01E09A` came to.
const SUM: u32 = 0xFFAF14;
/// A counter of the tasks `func_01DE66` makes (by 2).
const LEFT: u32 = 0xFFAF15;
/// Words: `func_01DD9E`'s distances along x (then the larger; its low
/// byte `DISTANCE_LOW`) and y; `func_01DE66`'s position.
const DISTANCE: u32 = 0xFFAF16;
const DISTANCE_LOW: u32 = 0xFFAF17;
const DISTANCE_Y: u32 = 0xFFAF18;
const AT_X: u32 = 0xFFAF16;
const AT_Y: u32 = 0xFFAF18;

/// Object 7: its depth (`func_0007D2`), how far above the boss, its
/// frame, the routine once the boss is beaten and the one after.
const SEVEN_DEPTH: u8 = 4;
const ABOVE: u16 = 0x48;
const SEVEN_FRAME: u8 = 0xC;
const ROUTINE_01DE66: u32 = 0x01DE66;
const ROUTINE_01DEFC: u32 = 0x01DEFC;
/// The swing's animation and step, the Z80's command, what `func_0034DE`
/// is told, the most radius.
const SWING_ANIM: u32 = 0x0EB4EC;
const SWING_STEP: u8 = 7;
const SWING_SOUND: u8 = 0x7C;
const SWING_HIT: u32 = 0xC;
const MOST_RADIUS: u8 = 0xA0;
/// The tasks of `func_01DE66`: routine, depth, priority, flags,
/// animation, the last place, and the steps of `func_01DEC4` (words, by
/// `LEFT`: x, y).
const ROUTINE_01DC60: u32 = 0x01DC60;
const TASK_DEPTH: u8 = 1;
const TASK_PRIORITY: u8 = 0x48;
const FLAGS: u8 = 0x81;
const ANIM_EB536: u32 = 0x0EB536;
const PLACES_END: u8 = 8;
const STEPS_X: u32 = 0x01DEEC;
const STEPS_Y: u32 = 0x01DEF4;
/// Object 7 ends after this many frames.
const END_FRAMES: u8 = 8;
/// The thrown objects: depth, speed, animations (flying, meeting), the
/// turn's step, how far they spread before they fly (`OBJ_29` turns
/// negative), the count they turn for (to 0x18), the margin and size
/// of the field they stay in, how near object 0 they meet it.
const THROWN_DEPTH: u8 = 3;
const THROWN_SPEED: u16 = 3;
const FLY_ANIM: u32 = 0x0EB550;
const MEET_ANIM: u32 = 0x0EB518;
const TURN_STEP: u32 = 1;
const TURNS: u8 = 0x18;
const MARGIN: u16 = 0x40;
const FIELD: u16 = 0x180;
const NEAR: u16 = 0xC;
const ROUTINE_01E0FC: u32 = 0x01E0FC;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// The points of `func_0041D0`: from `POSITION` to object 0.
    fn zd_toward_0(&mut self) {
        let x = self.read16(POSITION_X);
        self.write16(FROM_X, x);
        let y = self.read16(POSITION_Y);
        self.write16(FROM_Y, y);
        let x = self.read16(OBJECT_0_X);
        self.write16(TO_X, x);
        let y = self.read16(OBJECT_0_Y);
        self.write16(TO_Y, y);
    }

    /// A thrown object gone: its first byte cleared, its sprite record
    /// freed (`func_000890`), `THROWN_LEFT` down; the last one clears
    /// `STATE` unless the boss is beaten.  Gives back X and C.
    fn zd_thrown_gone(&mut self, a4: u32) -> (bool, bool) {
        self.write8(a4 + OBJ_ID, 0);
        self.r_000890();
        let was = self.read8(THROWN_LEFT);
        self.write8(THROWN_LEFT, was.wrapping_sub(1));
        let borrow = was == 0;
        if was != 1 {
            return (borrow, borrow);
        }
        if (self.read8(STATE) as i8) >= 0 {
            self.write8(STATE, 0);
        }
        (borrow, false)
    }

    /// `func_01DC7C`, object 7's routine (depth 4, `func_0007D2`): on as
    /// `func_01DE66` once the boss is beaten.  While `SHOWN` is set or
    /// it isn't swinging (`STEP` 0) it is 0x48 above the boss (its
    /// offset the boss's bob, `func_01DAE0`; 0 while `HURT`), frame
    /// 0xC.  `STEP` 1 starts a swing (`func_01DD58`, `func_01DE02`; the
    /// Z80 gets `SWING_SOUND`, animation `SWING_ANIM`, `STEP` 7); then
    /// each frame the angle moves (`func_01DE14`) and it is put on the
    /// circle (`func_0034DE` told 0xC, animated) until the swing ends:
    /// `STEP` 0, `STATE` all but bit 7 cleared.
    pub fn r_01dc7c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = low_byte(r.d0, SEVEN_DEPTH);
        r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
        if (self.read8(STATE) as i8) < 0 {
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01DE66);
        }
        let shown = self.read8(SHOWN);
        let step = self.read8(STEP);
        if shown == 0 {
            r.d0 = low_byte(r.d0, step);
        }
        if shown == 0 && step != 0 {
            r.d0 = low_byte(r.d0, step.wrapping_sub(1));
            if step == 1 {
                r.d0 = low_byte(r.d0, SWING_SOUND);
                self.r_0034b6(r.d0);
                r.d0 = self.r_01dd58(r.d0, r.d1, r.d3, r.a4);
                self.r_01de02();
                let centre = self.read16(CENTRE_X);
                let object_0 = self.read16(OBJECT_0_X);
                let sum = centre.wrapping_sub(object_0);
                r.d0 = low_word(r.d0, sum);
                r.x = centre < object_0;
                self.write16(SUM, sum);
                // the original branches on C after the move, which is clear
                self.write32(r.a4 + OBJ_ANIM, SWING_ANIM);
                self.write8(STEP, SWING_STEP);
                r.c = false;
                return;
            }
            let ended;
            (r.d0, r.x, ended) = self.r_01de14(r.d0);
            if ended {
                self.write8(STEP, 0);
                let state = self.read8(STATE);
                self.write8(STATE, state & 0x80);
                r.c = false;
                return;
            }
            r.d0 = self.r_01de36(r.d0, r.d3);
            r.d0 = low_word(r.d0, (r.d0 as u16).wrapping_add(self.read16(CENTRE_X)));
            self.write16(POSITION_X, r.d0 as u16);
            r.d0 = self.r_01de4e(r.d0, r.d3);
            r.d0 = low_word(r.d0, (r.d0 as u16).wrapping_add(self.read16(CENTRE_Y)));
            self.write16(POSITION_Y, r.d0 as u16);
            self.r_001300(r.a4);
            r.d0 = SWING_HIT;
            (r.d1, r.d2, r.x) = self.r_0034de(r.d0, r.d1, r.d2, r.a4);
            (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
            (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
            return;
        }
        (r.d4, r.d5, r.a4, r.a5) = self.r_01dae0(r.d4, r.d5);
        let y = self.read16(POSITION_Y);
        self.write16(POSITION_Y, y.wrapping_sub(ABOVE));
        r.x = y < ABOVE;
        self.r_001300(r.a4);
        if self.read8(HURT) != 0 {
            self.write8(r.a4 + OBJ_OFFSET, 0);
        }
        self.write8(r.a5 + SPRITE_FRAME, SEVEN_FRAME);
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
    }

    /// `func_01DD58`: a circle from the object at a4 to object 0: the
    /// radius (`func_01DD9E`), the angle the way back (`func_0041D0`
    /// negated) to `ANGLE`, the centre the object's position plus the
    /// circle's point at it.  Gives back d0.
    pub fn r_01dd58(&mut self, d0: u32, d1: u32, d3: u32, a4: u32) -> u32 {
        let x = self.read16(a4 + OBJ_X);
        self.write16(FROM_X, x);
        let y = self.read16(a4 + OBJ_Y);
        self.write16(FROM_Y, y);
        let x = self.read16(OBJECT_0_X);
        self.write16(TO_X, x);
        let y = self.read16(OBJECT_0_Y);
        self.write16(TO_Y, y);
        self.r_01dd9e();
        let d0 = self.r_0041d0(d0, d1);
        let angle = (d0 as u8).wrapping_neg();
        self.write8(ANGLE, angle);
        let d0 = self.r_01de36(low_byte(d0, angle), d3);
        let centre_x = (d0 as u16).wrapping_add(self.read16(a4 + OBJ_X));
        self.write16(CENTRE_X, centre_x);
        let d0 = self.r_01de4e(low_word(d0, centre_x), d3);
        let centre_y = (d0 as u16).wrapping_add(self.read16(a4 + OBJ_Y));
        self.write16(CENTRE_Y, centre_y);
        low_word(d0, centre_y)
    }

    /// `func_01DD9E`: `RADIUS` the larger of the distances (words)
    /// along x and y between `FROM` and `TO` (`DISTANCE`,
    /// `DISTANCE_Y`, their difference to `SUM`), when it is 1 to 0x9F;
    /// else `MOST_RADIUS`.
    pub fn r_01dd9e(&mut self) {
        let dx = self.read16(FROM_X).wrapping_sub(self.read16(TO_X));
        self.write16(DISTANCE, (dx as i16).unsigned_abs());
        let dy = self.read16(FROM_Y).wrapping_sub(self.read16(TO_Y));
        self.write16(DISTANCE_Y, (dy as i16).unsigned_abs());
        let difference = self.read16(DISTANCE).wrapping_sub(self.read16(DISTANCE_Y));
        self.write16(SUM, difference);
        if (difference as i16) < 0 {
            let dy = self.read16(DISTANCE_Y);
            self.write16(DISTANCE, dy);
        }
        let low = self.read8(DISTANCE_LOW);
        let radius = if self.read8(DISTANCE) == 0 && low != 0 && low < MOST_RADIUS { low } else { MOST_RADIUS };
        self.write8(RADIUS, radius);
    }

    /// `func_01DE02`: `ANGLE` half a turn on, and `END_ANGLE` that.
    pub fn r_01de02(&mut self) {
        let angle = self.read8(ANGLE).wrapping_add(0x80);
        self.write8(ANGLE, angle);
        self.write8(END_ANGLE, angle);
    }

    /// `func_01DE14`: `ANGLE` moved by `STEP` - 5; C set when it is
    /// `END_ANGLE`.  Gives back d0 (the angle), X (the carry) and C.
    pub fn r_01de14(&mut self, d0: u32) -> (u32, bool, bool) {
        let step = self.read8(STEP).wrapping_sub(5);
        let was = self.read8(ANGLE);
        let angle = step.wrapping_add(was);
        self.write8(ANGLE, angle);
        (low_byte(d0, angle), angle < step, angle == self.read8(END_ANGLE))
    }

    /// `func_01DE36`: the circle's x at `ANGLE`, `RADIUS`
    /// (`func_017780`).  Gives back d0.
    pub fn r_01de36(&mut self, d0: u32, d3: u32) -> u32 {
        let radius = self.read8(RADIUS);
        let angle = self.read8(ANGLE);
        self.r_017780(low_byte(d0, angle), u32::from(radius), d3)
    }

    /// `func_01DE4E`: the circle's y at `ANGLE`, `RADIUS`
    /// (`func_01777C`).  Gives back d0.
    pub fn r_01de4e(&mut self, d0: u32, d3: u32) -> u32 {
        let radius = self.read8(RADIUS);
        let angle = self.read8(ANGLE);
        self.r_01777c(low_byte(d0, angle), u32::from(radius), d3)
    }

    /// `func_01DE66`, object 7's routine once the boss is beaten: its
    /// count 0, on as `func_01DEFC`; four tasks of `func_01DC60` made
    /// at its position plus the steps of `func_01DEC4`, flags 0x81,
    /// animation 0xEB536 started and placed.
    pub fn r_01de66(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a4 + OBJ_COUNT, 0);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01DEFC);
        let x = self.read16(r.a4 + OBJ_X);
        self.write16(AT_X, x);
        let y = self.read16(r.a4 + OBJ_Y);
        self.write16(AT_Y, y);
        self.write8(LEFT, 0);
        loop {
            (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, TASK_DEPTH, TASK_PRIORITY, ROUTINE_01DC60);
            self.r_01dec4();
            self.r_00152a(r.a6);
            self.write8(r.a5 + SPRITE_FLAGS, FLAGS);
            self.write32(r.a6 + TASK_ANIM, ANIM_EB536);
            (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
            (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
            let was = self.read8(LEFT);
            let place = was.wrapping_add(2);
            self.write8(LEFT, place);
            r.x = place < was;
            if place == PLACES_END {
                r.c = false;
                return;
            }
        }
    }

    /// `func_01DEC4`: `POSITION` the position `AT_X`, `AT_Y` plus the
    /// words at `LEFT` (a byte offset) in `STEPS_X` and `STEPS_Y`.
    pub fn r_01dec4(&mut self) {
        let at = self.read8(LEFT) as i8 as i32 as u32;
        let x = self.read16(STEPS_X.wrapping_add(at)).wrapping_add(self.read16(AT_X));
        self.write16(POSITION_X, x);
        let y = self.read16(STEPS_Y.wrapping_add(at)).wrapping_add(self.read16(AT_Y));
        self.write16(POSITION_Y, y);
    }

    /// `func_01DEFC`, object 7's last routine: placed for `END_FRAMES`
    /// frames (its count), then it ends (`func_000890`, its first byte
    /// cleared) and bit 0 of `STATE` is set.
    pub fn r_01defc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let was = self.read8(r.a4 + OBJ_COUNT);
        let count = was.wrapping_add(1);
        self.write8(r.a4 + OBJ_COUNT, count);
        r.x = count < was;
        if count < END_FRAMES {
            (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
            return;
        }
        self.r_000890();
        self.write8(r.a4 + OBJ_ID, 0);
        let state = self.read8(STATE);
        self.write8(STATE, state | 1);
        r.c = false;
    }

    /// `func_01DF1E`, a thrown object's routine (depth 3,
    /// `func_0007D2`).  Phase 0: round the circle at the centre
    /// (`func_01DFA4`, `func_01DFC0`, `func_01DFDC`).  Then it turns
    /// toward object 0 (`func_01E032`) and flies at `THROWN_SPEED`
    /// (`func_00410C`); out of the field (`func_01E078`,
    /// `func_01E09A`) it is gone (`zd_thrown_gone`); else it may meet
    /// object 0 (`func_01E0BC`) and is animated.  Placed
    /// (`func_001B32`).
    pub fn r_01df1e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = low_byte(r.d0, THROWN_DEPTH);
        r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
        let phase = self.read8(r.a4 + OBJ_PHASE);
        r.d0 = low_byte(r.d0, phase);
        if phase == 0 {
            r.d0 = self.r_01dfa4(r.d0, r.d3, r.a4);
            r.d0 = self.r_01dfc0(r.d0, r.d3, r.a4);
            self.r_01dfdc(r.d0, r.d1, r.a4);
            self.r_001300(r.a4);
            (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
            return;
        }
        self.r_001542(r.a4);
        (r.d0, r.d1, r.d2, r.x) = self.r_01e032(r.d0, r.d1, r.d2, r.a4);
        self.write16(SPEED, THROWN_SPEED);
        r.d0 = low_byte(r.d0, self.read8(r.a4 + OBJ_ANGLE));
        (r.d0, r.d1, r.d2, r.x) = self.r_00410c(r.d0, r.d1, r.d2, r.x);
        let mut out;
        (r.d0, out) = self.r_01e078(r.d0);
        if !out {
            (r.d0, out) = self.r_01e09a(r.d0);
        }
        if out {
            (r.x, r.c) = self.zd_thrown_gone(r.a4);
            return;
        }
        self.r_001300(r.a4);
        r.x = self.r_01e0bc(r.a4);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
    }

    /// `func_01DFA4`: `POSITION_X` the centre's x plus the circle's x
    /// at the object's angle and radius.  Gives back d0.
    pub fn r_01dfa4(&mut self, d0: u32, d3: u32, a4: u32) -> u32 {
        let angle = self.read8(a4 + OBJ_ANGLE);
        self.write8(ANGLE, angle);
        let radius = self.read8(a4 + OBJ_RADIUS);
        self.write8(RADIUS, radius);
        let d0 = self.r_01de36(d0, d3);
        let x = (d0 as u16).wrapping_add(self.read16(CENTRE_X));
        self.write16(POSITION_X, x);
        low_word(d0, x)
    }

    /// `func_01DFC0`: `POSITION_Y` the centre's y plus the circle's y
    /// at the object's angle and radius.  Gives back d0.
    pub fn r_01dfc0(&mut self, d0: u32, d3: u32, a4: u32) -> u32 {
        let angle = self.read8(a4 + OBJ_ANGLE);
        self.write8(ANGLE, angle);
        let radius = self.read8(a4 + OBJ_RADIUS);
        self.write8(RADIUS, radius);
        let d0 = self.r_01de4e(d0, d3);
        let y = (d0 as u16).wrapping_add(self.read16(CENTRE_Y));
        self.write16(POSITION_Y, y);
        low_word(d0, y)
    }

    /// `func_01DFDC`: the angle 2 on, `OBJ_29` 1 on; once that is
    /// negative the object flies: phase 1 on, its angle -(angle +
    /// 0x40), its radius byte the angle to object 0 (`func_0041D0`),
    /// `OBJ_29` 1, animation `FLY_ANIM`, and the Z80 gets
    /// `SWING_SOUND`.
    pub fn r_01dfdc(&mut self, d0: u32, d1: u32, a4: u32) {
        let angle = self.read8(a4 + OBJ_ANGLE).wrapping_add(2);
        self.write8(a4 + OBJ_ANGLE, angle);
        let count = self.read8(a4 + OBJ_29).wrapping_add(1);
        self.write8(a4 + OBJ_29, count);
        if (count as i8) >= 0 {
            return;
        }
        let phase = self.read8(a4 + OBJ_PHASE).wrapping_add(1);
        self.write8(a4 + OBJ_PHASE, phase);
        let angle = self.read8(a4 + OBJ_ANGLE).wrapping_add(0x40).wrapping_neg();
        self.write8(a4 + OBJ_ANGLE, angle);
        self.zd_toward_0();
        let d0 = self.r_0041d0(low_byte(d0, angle), d1);
        self.write8(a4 + OBJ_RADIUS, d0 as u8);
        self.write8(a4 + OBJ_29, 1);
        self.write32(a4 + OBJ_ANIM, FLY_ANIM);
        self.r_0034b6(low_byte(d0, SWING_SOUND));
    }

    /// `func_01E032`: for its first `TURNS` frames (`OBJ_29`) the angle
    /// it aims at (byte 0x27) is the angle to object 0 (`func_0041D0`);
    /// its angle turns a step toward it (`func_0042FE`).  Gives back d0,
    /// d1, d2 and X.
    pub fn r_01e032(&mut self, d0: u32, d1: u32, d2: u32, a4: u32) -> (u32, u32, u32, bool) {
        let count = self.read8(a4 + OBJ_29);
        let mut d0 = low_byte(d0, count);
        if count < TURNS {
            let count = count.wrapping_add(1);
            d0 = low_byte(d0, count);
            self.write8(a4 + OBJ_29, count);
            self.zd_toward_0();
            d0 = self.r_0041d0(d0, d1);
            self.write8(a4 + OBJ_RADIUS, d0 as u8);
        }
        let d1 = low_byte(d1, self.read8(a4 + OBJ_ANGLE));
        let d2 = low_byte(d2, self.read8(a4 + OBJ_RADIUS));
        let (d0, x) = self.r_0042fe(TURN_STEP, d1, d2);
        self.write8(a4 + OBJ_ANGLE, d0 as u8);
        (d0, d1, d2, x)
    }

    /// One axis's move for `func_01E078`, `func_01E09A`: the position
    /// word at `at` plus the velocity's; C set when it plus `MARGIN` is
    /// `FIELD` or more (unsigned; the sum to `SUM`).  Gives back d0 and
    /// C.
    fn zd_fly(&mut self, d0: u32, at: u32, velocity: u32) -> (u32, bool) {
        let step = self.read16(velocity);
        let p = self.read16(at).wrapping_add(step);
        self.write16(at, p);
        let sum = p.wrapping_add(MARGIN);
        self.write16(SUM, sum);
        (low_word(d0, sum.wrapping_sub(FIELD)), sum >= FIELD)
    }

    /// `func_01E078`: `POSITION_X` moved by `VELOCITY_X`; C set when it
    /// is out of the field.  Gives back d0 and C.
    pub fn r_01e078(&mut self, d0: u32) -> (u32, bool) {
        self.zd_fly(d0, POSITION_X, VELOCITY_X)
    }

    /// `func_01E09A`: `POSITION_Y` moved by `VELOCITY_Y`; C set when it
    /// is out of the field.  Gives back d0 and C.
    pub fn r_01e09a(&mut self, d0: u32) -> (u32, bool) {
        self.zd_fly(d0, POSITION_Y, VELOCITY_Y)
    }

    /// `func_01E0BC`: when `POSITION` is within `NEAR` of object 0
    /// along x (words) and y (the low byte of the distance only), object
    /// 0 is hit (`func_0033C0`), the object goes on as `func_01E0FC`
    /// and `MET` is set.  Gives back X (the last subtraction's or
    /// negation's, or `func_0033C0`'s).
    pub fn r_01e0bc(&mut self, a4: u32) -> bool {
        let (dx, x) = self.zd_distance(POSITION_X, OBJECT_0_X);
        if dx >= NEAR {
            return x;
        }
        let (dy, x) = self.zd_distance(POSITION_Y, OBJECT_0_Y);
        if dy as u8 >= NEAR as u8 {
            return x;
        }
        let x = self.r_0033c0();
        self.write32(a4 + OBJ_ROUTINE, ROUTINE_01E0FC);
        self.write8(MET, 1);
        x
    }

    /// The distance (a word) from the word at `to` to the one at `at`,
    /// with the X the subtraction (or the negation) leaves.
    fn zd_distance(&mut self, at: u32, to: u32) -> (u16, bool) {
        let a = self.read16(at);
        let b = self.read16(to);
        let d = a.wrapping_sub(b);
        if (d as i16) < 0 {
            (d.wrapping_neg(), true)
        } else {
            (d, a < b)
        }
    }

    /// `func_01E0FC`, a thrown object's routine once it met object 0:
    /// animation `MEET_ANIM`; when it ends the object is gone
    /// (`zd_thrown_gone`).
    pub fn r_01e0fc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write32(r.a4 + OBJ_ANIM, MEET_ANIM);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if !r.c {
            return;
        }
        let state = self.read8(STATE);
        (r.x, r.c) = self.zd_thrown_gone(r.a4);
        if !r.c && self.read8(THROWN_LEFT) == 0 {
            r.d0 = low_byte(r.d0, state);
        }
    }
}
