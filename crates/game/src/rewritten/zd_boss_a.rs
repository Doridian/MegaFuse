//! 0x1D028-0x1D7E2: the rest of a boss run as object 6 (its earlier
//! routines, `func_01CDAA`-`func_01D026`, are before this range).
//! `func_01D028` is a task that, at its count 0x17, throws three
//! objects from the boss's left, right or both sides (`SIDES`, then
//! stepped 0-2; `func_01D090`, `func_01D180`), each moving along its
//! angle (`func_01D20A`) and leaving another behind it every 4 frames
//! (`func_01D314`) until it leaves the screen.  `func_01D31C` turns
//! the boss toward object 0 (`func_00334C`) for 0xF0 frames;
//! `func_01D3BC`, `func_01D3E0` make it rise and set three objects
//! (`func_01D458`, `func_01D5E0`) going after object 0 with it
//! (`func_01D4F4`) for 0x708 frames or until it is hit, then it comes
//! back down (`func_01D580`, `func_01D59E`).  When it is hit through
//! (`func_01D632`) it shakes (`func_01D670`, `func_01D6A2`) and ends
//! (`func_01D722`, `func_01D740`), making the task `func_019262`.
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read.

use crate::{Game, Regs};

/// An object's fields: its first byte (0: free), its routine (a long),
/// a byte 6 cleared, its sprite record (address, a long, and index),
/// byte 0xF, its hits (`func_006E7C` takes one), bytes 0x13, its
/// direction and kind (the animation, `anim.rs`), its position (words:
/// x, y; their bytes), the animation's timer and step, two bytes
/// 0x26-0x27 (an angle and a count) and the offset.
const OBJ_ID: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_06: u32 = 0x06;
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_0F: u32 = 0x0F;
const OBJ_HITS: u32 = 0x11;
const OBJ_13: u32 = 0x13;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x15;
const OBJ_X_HIGH: u32 = 0x18;
const OBJ_X: u32 = 0x18;
const OBJ_X_LOW: u32 = 0x19;
const OBJ_Y_HIGH: u32 = 0x1A;
const OBJ_Y: u32 = 0x1A;
const OBJ_Y_LOW: u32 = 0x1B;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_ANGLE: u32 = 0x26;
const OBJ_COUNT: u32 = 0x26;
const OBJ_27: u32 = 0x27;
const OBJ_OFFSET: u32 = 0x2A;
/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
/// `func_01D028`'s task's count (byte 0x18).
const TASK_COUNT: u32 = 0x18;

/// A byte position: `func_01D180` makes an object there (`POSITION`'s
/// low bytes); `func_01D4F4` counts in it (a word, its bytes from
/// `COUNT_LOW` and `COUNT_HIGH`).
const AT_Y: u32 = 0xFF80C4;
const AT_X: u32 = 0xFF80C5;
/// What `func_0012A0` sets an object up with: its animations (a table
/// of longs) and its routine (longs).
const OBJ_INIT_ANIMS: u32 = 0xFF80C8;
const OBJ_INIT_ROUTINE: u32 = 0xFF80D0;
/// Cleared when the boss is hit through.
const CLEARED_80DF: u32 = 0xFF80DF;
/// A position (words: x then y; `func_001300`), and their bytes.
const POSITION_X_HIGH: u32 = 0xFF9F04;
const POSITION_X_LOW: u32 = 0xFF9F05;
const POSITION_Y_HIGH: u32 = 0xFF9F06;
const POSITION_Y_LOW: u32 = 0xFF9F07;
/// The speed (a word) `func_00410C` makes a velocity of, its bytes.
const SPEED_HIGH: u32 = 0xFF9F00;
const SPEED_LOW: u32 = 0xFF9F01;
/// Two points (words): `func_0041D0` gives the angle of A - B.
const FROM_X: u32 = 0xFF9EFA;
const FROM_Y: u32 = 0xFF9EFE;
const TO_X: u32 = 0xFF9EE6;
const TO_Y: u32 = 0xFF9EE8;
/// A word set to 0x33 before `func_00334C`.
const WORD_9EEC: u32 = 0xFF9EEC;
/// A word set to 0x66 before `func_00398C`.
const WORD_A960: u32 = 0xFFA960;
/// Object 0's position (words).
const OBJECT_0_X: u32 = 0xFFA44A;
const OBJECT_0_Y: u32 = 0xFFA44C;
/// The running object's index, and where `func_01D59E` copies it.
const RUNNING: u32 = 0xFFA42C;
const CHOSEN: u32 = 0xFFA93F;
/// Bit 6 of one, bit 0 of the other, set when the boss is hit through.
const FLAGS: u32 = 0xFF9540;
const SPARED: u32 = 0xFF9548;

/// Which sides `func_01D028` throws from: 0 the left, 1 the right, 2
/// both; stepped 0, 1, 2 after each throw.
const SIDES: u32 = 0xFFAEDE;
/// The angle of the object `func_01D180` makes (its high nibble less 6
/// is the object's direction).
const THROW_ANGLE: u32 = 0xFFAEE0;
/// `func_01D20A`'s copy of the thrower's position (bytes: x's high
/// and low, y's high and low), direction, kind, angle and index.
const KEPT_X_HIGH: u32 = 0xFFAEE1;
const KEPT_X_LOW: u32 = 0xFFAEE2;
const KEPT_Y_HIGH: u32 = 0xFFAEE3;
const KEPT_Y_LOW: u32 = 0xFFAEE4;
const KEPT_DIRECTION: u32 = 0xFFAEE6;
const KEPT_KIND: u32 = 0xFFAEE7;
const KEPT_ANGLE: u32 = 0xFFAEE8;
const KEPT_INDEX: u32 = 0xFFAEE9;
/// `func_01D31C`'s angle to object 0 and count of frames; then
/// (`func_01D4F4`) a count of frames (a word, low byte first).
const TURN_ANGLE: u32 = 0xFFAEE1;
const TURN_COUNT: u32 = 0xFFAEE2;
const COUNT_LOW: u32 = 0xFFAEE1;
const COUNT_HIGH: u32 = 0xFFAEE2;
/// `func_01D458`'s place (1-3) of the object it makes.
const PLACE: u32 = 0xFFAEE1;
/// `func_01D6A2`'s count of shakes and the offset it grows (a word, its
/// high byte the offset).
const SHAKES: u32 = 0xFFAEE3;
const SHAKE_OFFSET: u32 = 0xFFAEE4;
/// The boss's hits before `func_006E7C` (`func_01D4F4`).
const HITS_BEFORE: u32 = 0xFFAEEA;
/// Set when the boss comes back down: the objects of `func_01D5E0`
/// end.
const BACK: u32 = 0xFFAEEB;
/// 1 while the boss is up with its objects; 0xFF once hit through.
const UP: u32 = 0xFFAEEC;

/// The boss's object.
const BOSS: u32 = 6;
/// The animations (tables of longs) and routines the objects are made
/// with.
const THROWN_ANIMS: u32 = 0x01D7E2;
const ROUTINE_01D20A: u32 = 0x01D20A;
const ROUTINE_01D314: u32 = 0x01D314;
const ROUTINE_01D3E0: u32 = 0x01D3E0;
const ROUTINE_01D4F4: u32 = 0x01D4F4;
const ROUTINE_01D580: u32 = 0x01D580;
const ROUTINE_01D59E: u32 = 0x01D59E;
const ROUTINE_01D670: u32 = 0x01D670;
const ROUTINE_01D6A2: u32 = 0x01D6A2;
const ROUTINE_01D722: u32 = 0x01D722;
const ROUTINE_01D740: u32 = 0x01D740;
const ROUTINE_01CEEE: u32 = 0x01CEEE;
const ROUTINE_01CDAA: u32 = 0x01CDAA;
const FOLLOWER_ANIMS: u32 = 0x01D78A;
const ROUTINE_01D5E0: u32 = 0x01D5E0;
/// The task made when the boss ends, and its priority.
const ROUTINE_019262: u32 = 0x019262;
const END_PRIORITY: u8 = 0x50;
/// The offsets of `func_01D458`'s objects from the boss by place
/// (bytes: x, y).
const PLACE_DX: u32 = 0x01D450;
const PLACE_DY: u32 = 0x01D454;
/// `func_01D6A2`'s steps of x (bytes, one per 2 frames).
const SHAKE_STEPS: u32 = 0x01D762;

/// The Z80's commands: at a throw, every 16 frames of the turn, at the
/// boss's coming down and end.
const THROW_SOUND: u8 = 0x6C;
const TURN_SOUND: u8 = 0x6B;
const DOWN_SOUND: u8 = 0x82;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// The byte at `at` plus 1; gives back the new byte and the carry.
    fn zd_step(&mut self, at: u32) -> (u8, bool) {
        let was = self.read8(at);
        let now = was.wrapping_add(1);
        self.write8(at, now);
        (now, now < was)
    }

    /// The object's animation started again: by its direction and kind
    /// (`func_00130E`), the timer and step cleared, its first frame
    /// (`func_001336`).
    fn zd_restart(&mut self, r: &mut Regs) {
        (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.zd_animate(r);
    }

    /// `func_001336` on the family's registers.
    fn zd_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
    }

    /// `func_001B32` on the family's registers.
    fn zd_place(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
    }

    /// An object made (`func_001456`, `func_0012A0`) with the
    /// animations and routine in `OBJ_INIT_*`: frame 0, byte 0xF 0x21,
    /// sprite flags `flags`.  False (and nothing made) when none is
    /// free; else d4 and a4 are the new object's, d5 and a5 its sprite
    /// record's.
    fn zd_make(&mut self, r: &mut Regs, flags: u8) -> bool {
        let none;
        (r.d0, r.x, none) = self.r_001456(r.d0);
        r.c = none;
        if none {
            return false;
        }
        (r.d4, r.a4, r.x) = self.r_001cfa(low_byte(r.d4, r.d0 as u8));
        (r.d0, r.d5, r.a5, r.x) = self.r_0012a0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.write8(r.a4 + OBJ_0F, 0x21);
        self.write8(r.a5 + SPRITE_FLAGS, flags);
        true
    }

    /// `func_01D028`, a task's routine: at its count 0x17 it ends
    /// (`func_000560`), the Z80 gets `THROW_SOUND`, and three objects
    /// are thrown (`func_01D090`) from the left (`SIDES` 0), the right
    /// (1) or both (2); `SIDES` steps on (0 after 2).
    pub fn r_01d028(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count;
        (count, r.x) = self.zd_step(r.a6 + TASK_COUNT);
        if count != 0x17 {
            r.c = count < 0x17;
            return;
        }
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
        r.d0 = low_byte(r.d0, THROW_SOUND);
        self.r_0034b6(r.d0);
        let sides = self.read8(SIDES);
        r.d0 = low_byte(r.d0, sides);
        if sides != 1 {
            r.d0 = low_word(r.d0, 0);
            self.zd_throw(r);
        }
        if sides != 0 {
            r.d0 = 1;
            self.zd_throw(r);
        }
        let sides = self.read8(SIDES);
        let mut next = sides.wrapping_add(1);
        r.x = sides == 0xFF;
        if next == 3 {
            next = 0;
        }
        r.d0 = low_byte(r.d0, next);
        self.write8(SIDES, next);
        r.c = false;
    }

    /// `func_01D090` on the family's registers.
    fn zd_throw(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6) =
            self.r_01d090(r.d0, r.d1, r.d2, r.d3, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
    }

    /// `func_01D090`: three objects thrown (`func_01D180`) from the
    /// boss's left (d0's low byte 0: x less 0xC, 0x10, 0x14 at the
    /// angles 0x80, 0x90, 0xA0) or right (x plus the same at 0x80,
    /// 0x70, 0x60), y less 0xC, 0xE, 0x10 (bytes).  Gives back d0, d1,
    /// d2, d4, d5, d6, a0, a4, a5 and a6.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01d090(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let left = d0 as u8 == 0;
        let (d4, a4, _) = self.r_001cfa(BOSS);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        let angles: [u8; 3] = if left { [0x80, 0x90, 0xA0] } else { [0x80, 0x70, 0x60] };
        for ((dx, dy), angle) in [(0xC, 0xC), (0x10, 0xE), (0x14, 0x10)].into_iter().zip(angles) {
            let x = self.read8(r.a4 + OBJ_X_LOW);
            let x = if left { x.wrapping_sub(dx) } else { x.wrapping_add(dx) };
            self.write8(AT_X, x);
            let y = self.read8(r.a4 + OBJ_Y_LOW).wrapping_sub(dy);
            r.d0 = low_byte(r.d0, y);
            self.write8(AT_Y, y);
            self.write8(THROW_ANGLE, angle);
            (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6) =
                self.r_01d180(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        }
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6)
    }

    /// `func_01D180`: an object made (`func_001456`, `func_0012A0`) at
    /// (`AT_X`, `AT_Y`) with the routine `func_01D20A`, its angle
    /// `THROW_ANGLE`, its direction the angle's high nibble less 6
    /// (kind 4 for direction 4, else 0), byte 0x27 cleared, its
    /// animation started.  d4-d6 and a4-a6 are kept.  Gives back d0,
    /// d1, d2, d4, d5, d6, a0, a4, a5 and a6.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01d180(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let x = self.read8(AT_X);
        self.write8(POSITION_X_LOW, x);
        self.write8(POSITION_X_HIGH, 0);
        let y = self.read8(AT_Y);
        self.write8(POSITION_Y_LOW, y);
        self.write8(POSITION_Y_HIGH, 0);
        self.write32(OBJ_INIT_ANIMS, THROWN_ANIMS);
        self.write32(OBJ_INIT_ROUTINE, ROUTINE_01D20A);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        if self.zd_make(&mut r, 0x87) {
            self.write8(r.a4 + OBJ_KIND, 0);
            let angle = self.read8(THROW_ANGLE);
            let direction = (angle >> 4).wrapping_sub(6);
            r.d0 = low_byte(r.d0, direction);
            self.write8(r.a4 + OBJ_DIRECTION, direction);
            if direction == 4 {
                self.write8(r.a4 + OBJ_KIND, 4);
            }
            self.write8(r.a4 + OBJ_ANGLE, angle);
            self.write8(r.a4 + OBJ_27, 0);
            self.zd_restart(&mut r);
        }
        (r.d0, r.d1, r.d2, d4, d5, d6, r.a0, a4, a5, a6)
    }

    /// `.L01D208`: an `rts`.
    pub fn r_01d208(&mut self) {}

    /// `func_01D20A`, an object's routine: it moves along its angle at
    /// speed 0x400 (`func_00410C`, `func_0019C0`, `func_0019DA`) with
    /// its animation from the start; every 4th frame it makes an
    /// object of `func_01D314` where it was (direction and kind its
    /// own), with its angle.  Then (as the object it was, `KEPT_INDEX`)
    /// it ends (`func_01D30A`) once its x's high byte isn't 0 or its y
    /// is 0xF0 or more.
    pub fn r_01d20a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zd_animate(r);
        r.d0 = 8;
        (r.d1, r.d2, r.x) = self.r_0034de(r.d0, r.d1, r.d2, r.a4);
        for (field, kept) in [
            (OBJ_X_HIGH, KEPT_X_HIGH),
            (OBJ_X_LOW, KEPT_X_LOW),
            (OBJ_Y_HIGH, KEPT_Y_HIGH),
            (OBJ_Y_LOW, KEPT_Y_LOW),
            (OBJ_DIRECTION, KEPT_DIRECTION),
            (OBJ_KIND, KEPT_KIND),
            (OBJ_ANGLE, KEPT_ANGLE),
        ] {
            let b = self.read8(r.a4 + field);
            self.write8(kept, b);
        }
        self.write8(KEPT_INDEX, r.d4 as u8);
        self.write8(SPEED_LOW, 0);
        self.write8(SPEED_HIGH, 4);
        r.d0 = low_byte(r.d0, self.read8(r.a4 + OBJ_ANGLE));
        (r.d0, r.d1, r.d2, r.x) = self.r_00410c(r.d0, r.d1, r.d2, r.x);
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
        (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.zd_animate(r);
        self.zd_place(r);
        let count;
        (count, r.x) = self.zd_step(r.a4 + OBJ_27);
        if count != 4 {
            r.c = count < 4;
            return;
        }
        self.write8(r.a4 + OBJ_27, 0);
        for (kept, at) in [
            (KEPT_X_LOW, POSITION_X_LOW),
            (KEPT_X_HIGH, POSITION_X_HIGH),
            (KEPT_Y_LOW, POSITION_Y_LOW),
            (KEPT_Y_HIGH, POSITION_Y_HIGH),
        ] {
            let b = self.read8(kept);
            self.write8(at, b);
        }
        self.write32(OBJ_INIT_ANIMS, THROWN_ANIMS);
        self.write32(OBJ_INIT_ROUTINE, ROUTINE_01D314);
        if self.zd_make(r, 0x87) {
            let kind = self.read8(KEPT_KIND);
            self.write8(r.a4 + OBJ_KIND, kind);
            let direction = self.read8(KEPT_DIRECTION);
            self.write8(r.a4 + OBJ_DIRECTION, direction);
            self.zd_restart(r);
            let angle = self.read8(KEPT_ANGLE);
            self.write8(r.a4 + OBJ_ANGLE, angle);
        }
        self.zd_place(r);
        let index = self.read8(KEPT_INDEX);
        (r.d4, r.a4, r.x) = self.r_001cfa(low_byte(r.d4, index));
        if self.read8(r.a4 + OBJ_X_HIGH) != 0 || self.read8(r.a4 + OBJ_Y_LOW) >= 0xF0 {
            r.c = self.r_01d30a(r.a4);
            return;
        }
        r.c = true;
    }

    /// `.L01D30A`: the object at a4 ends: its sprite record freed
    /// (`func_000890`) and its first byte cleared.  Gives back C
    /// (clear).
    pub fn r_01d30a(&mut self, a4: u32) -> bool {
        self.r_000890();
        self.write8(a4 + OBJ_ID, 0);
        false
    }

    /// `func_01D314`, an object's routine: its animation; when it ends
    /// the object ends (`func_01D30A`).
    pub fn r_01d314(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zd_animate(r);
        if r.c {
            r.c = self.r_01d30a(r.a4);
        }
    }

    /// `func_01D31C`, the boss's routine: its animation two frames on;
    /// hit through (`func_006E7C`), `func_01D632`.  Else it turns
    /// (`func_00334C`, `WORD_9EEC` 0x33) to the quarter of its angle
    /// to object 0 (`func_0041D0`: under 0x21 or from 0xE1 0, under
    /// 0x61 1, 0xA1 2, else 3), the Z80 getting `TURN_SOUND` every 16
    /// frames; after 0xF0 frames it goes on as `func_01CEEE`.
    pub fn r_01d31c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zd_animate(r);
        self.zd_animate(r);
        (r.d0, r.x) = self.r_0015e6(r.a4);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e7c(r.d1, r.d2, r.a0, r.a4, r.a5);
        if r.c {
            (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_01d632(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
            return;
        }
        self.write16(WORD_9EEC, 0x33);
        let x = self.read16(r.a4 + OBJ_X);
        self.write16(FROM_X, x);
        let y = self.read16(r.a4 + OBJ_Y);
        self.write16(FROM_Y, y);
        let x = self.read16(OBJECT_0_X);
        self.write16(TO_X, x);
        let y = self.read16(OBJECT_0_Y);
        self.write16(TO_Y, y);
        r.d0 = self.r_0041d0(r.d0, r.d1);
        self.write8(TURN_ANGLE, r.d0 as u8);
        let quarter = match r.d0 as u8 {
            0x00..=0x20 | 0xE1..=0xFF => 0,
            0x21..=0x60 => 1,
            0x61..=0xA0 => 2,
            _ => 3,
        };
        r.d2 = low_byte(r.d2, quarter);
        r.d0 = low_byte(r.d0, quarter);
        (r.d1, r.a0) = self.r_00334c(r.d0, r.d1, r.d2, r.d5, r.d6, r.a0, r.a6);
        let count = self.read8(TURN_COUNT) & 0xF;
        r.d0 = low_byte(r.d0, count);
        if count == 0 {
            r.d0 = low_byte(r.d0, TURN_SOUND);
            self.r_0034b6(r.d0);
        }
        let (count, _) = self.zd_step(TURN_COUNT);
        if count == 0xF0 {
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01CEEE);
        }
        self.zd_place(r);
    }

    /// `func_01D3BC`, the boss's routine: kind 6, direction 0, its
    /// animation started; on as `func_01D3E0`.
    pub fn r_01d3bc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a4 + OBJ_KIND, 6);
        self.write8(r.a4 + OBJ_DIRECTION, 0);
        self.zd_restart(r);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01D3E0);
        r.c = false;
    }

    /// `func_01D3E0`, the boss's routine: once its animation ends it
    /// goes on as `func_01D4F4`, 8 higher, flags 0x86, angle 0xC, byte
    /// 6 0, byte 0x27 1, kind 0xC, byte 0x13 and direction 0 (its
    /// animation started), with three objects (`func_01D458`, places
    /// 1-3); `UP` 1, the count (`COUNT_LOW`, `COUNT_HIGH`) 0.
    pub fn r_01d3e0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zd_animate(r);
        if !r.c {
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01D4F4);
        let y = self.read8(r.a4 + OBJ_Y_LOW).wrapping_sub(8);
        r.d0 = low_byte(r.d0, y);
        self.write8(r.a4 + OBJ_Y_LOW, y);
        self.write8(r.a5 + SPRITE_FLAGS, 0x86);
        self.write8(r.a4 + OBJ_ANGLE, 0xC);
        self.write8(r.a4 + OBJ_06, 0);
        self.write8(r.a4 + OBJ_27, 1);
        self.write8(r.a4 + OBJ_KIND, 0xC);
        self.write8(r.a4 + OBJ_13, 0);
        self.write8(r.a4 + OBJ_DIRECTION, 0);
        self.zd_restart(r);
        for place in 1..=3 {
            r.d0 = place;
            (r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, r.x) =
                self.r_01d458(r.d0, r.d1, r.d2, r.d3, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        }
        self.write8(UP, 1);
        self.write8(COUNT_LOW, 0);
        self.write8(COUNT_HIGH, 0);
        r.c = false;
    }

    /// `.L01D44E`: an `rts`.
    pub fn r_01d44e(&mut self) {}

    /// `func_01D458`: an object made (`func_001456`, `func_0012A0`) of
    /// `func_01D5E0` at the boss's position plus the offset of `place`
    /// (d0: 1-3; `PLACE_DX`, `PLACE_DY`), flags 0x86, angle 9, byte 6
    /// 0, byte 0x27 1, kind 8, byte 0x13 and direction the place, its
    /// animation started, its hits 1.  Gives back d0, d1, d2, d4, d5,
    /// a0, a4, a5 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01d458(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        self.write8(PLACE, d0 as u8);
        let (d4, a4, x) = self.r_001cfa(BOSS);
        let place = d0 as u8 as i8 as i16 as u16;
        let d0 = low_word(d0, place);
        let at = i32::from(place as i16) as u32;
        let x_low = self.read8(a4 + OBJ_X_LOW).wrapping_add(self.read8(PLACE_DX.wrapping_add(at)));
        self.write8(POSITION_X_LOW, x_low);
        let x_high = self.read8(a4 + OBJ_X_HIGH);
        self.write8(POSITION_X_HIGH, x_high);
        let y_low = self.read8(a4 + OBJ_Y_LOW).wrapping_add(self.read8(PLACE_DY.wrapping_add(at)));
        let d1 = low_byte(d1, y_low);
        self.write8(POSITION_Y_LOW, y_low);
        let y_high = self.read8(a4 + OBJ_Y_HIGH);
        self.write8(POSITION_Y_HIGH, y_high);
        self.write32(OBJ_INIT_ANIMS, FOLLOWER_ANIMS);
        self.write32(OBJ_INIT_ROUTINE, ROUTINE_01D5E0);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x, ..Regs::default() };
        if self.zd_make(&mut r, 0x86) {
            self.write8(r.a4 + OBJ_ANGLE, 9);
            self.write8(r.a4 + OBJ_06, 0);
            self.write8(r.a4 + OBJ_27, 1);
            self.write8(r.a4 + OBJ_KIND, 8);
            let place = self.read8(PLACE);
            r.d0 = low_byte(r.d0, place);
            self.write8(r.a4 + OBJ_13, place);
            self.write8(r.a4 + OBJ_DIRECTION, place);
            self.zd_restart(&mut r);
            self.write8(r.a4 + OBJ_HITS, 1);
        }
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, r.x)
    }

    /// The boss or one of its objects goes after object 0 a frame
    /// (`func_003E44`, `func_00398C` with `WORD_A960` 0x66, then
    /// `func_01C558`) and is hit (`func_0015E6`, `func_006E7C`).
    fn zd_chase(&mut self, r: &mut Regs) {
        r.d0 = self.r_003e44();
        self.write16(WORD_A960, 0x66);
        (r.d0, r.d1, r.d4, r.a4) = self.r_00398c(r.d0, r.d1, r.d4, r.a4, r.a5);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.c) = self.r_01c558(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4);
        (r.d0, r.x) = self.r_0015e6(r.a4);
    }

    /// `func_01D4F4`, the boss's routine: it goes after object 0
    /// (`zd_chase`) until its count (`COUNT_LOW`, `COUNT_HIGH`, through
    /// `AT_Y` as a word) gets to 0x708 or it loses a hit; then it is
    /// put in its cell's middle (`func_001852`), flags 0x87, direction
    /// 0, kind 0x11 (its animation started), `BACK` 1, its count
    /// (0x26) 0, and it goes on as `func_01D580`.
    pub fn r_01d4f4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let low = self.read8(COUNT_LOW);
        self.write8(AT_X, low);
        let high = self.read8(COUNT_HIGH);
        self.write8(AT_Y, high);
        let count = self.read16(AT_Y).wrapping_add(1);
        self.write16(AT_Y, count);
        self.write8(COUNT_LOW, count as u8);
        self.write8(COUNT_HIGH, (count >> 8) as u8);
        if count != 0x708 {
            self.zd_chase(r);
            let hits = self.read8(r.a4 + OBJ_HITS);
            self.write8(HITS_BEFORE, hits);
            (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e7c(r.d1, r.d2, r.a0, r.a4, r.a5);
            let hits = self.read8(r.a4 + OBJ_HITS);
            r.d0 = low_byte(r.d0, hits);
            if hits == self.read8(HITS_BEFORE) {
                r.c = false;
                return;
            }
        }
        self.r_001852(r.a4);
        self.write8(r.a5 + SPRITE_FLAGS, 0x87);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01D580);
        self.write8(r.a4 + OBJ_DIRECTION, 0);
        self.write8(r.a4 + OBJ_KIND, 0x11);
        self.zd_restart(r);
        self.write8(BACK, 1);
        self.write8(r.a4 + OBJ_COUNT, 0);
        r.c = false;
    }

    /// `.L01D57E`: an `rts`.
    pub fn r_01d57e(&mut self) {}

    /// `func_01D580`, the boss's routine: its count (0x26) up; at 0x40
    /// the Z80 gets `DOWN_SOUND` and it goes on as `loc_01D59E`, which
    /// it runs.
    /// No script calls it: reviewed by reading.
    pub fn r_01d580(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count;
        (count, r.x) = self.zd_step(r.a4 + OBJ_COUNT);
        if count == 0x40 {
            r.d0 = low_byte(r.d0, DOWN_SOUND);
            self.r_0034b6(r.d0);
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01D59E);
            r.c = false;
        } else {
            r.c = count < 0x40;
        }
        self.r_01d59e(r);
    }

    /// `loc_01D59E`, the boss's routine: `CHOSEN` the running object;
    /// once its animation ends, offset 0xE7, direction 0, kind 2 (its
    /// animation started and placed), and it goes on as `func_01CDAA`;
    /// `BACK` and `UP` cleared.
    /// No script calls it: reviewed by reading.
    pub fn r_01d59e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let running = self.read8(RUNNING);
        self.write8(CHOSEN, running);
        self.zd_animate(r);
        if !r.c {
            return;
        }
        self.write8(r.a4 + OBJ_OFFSET, 0xE7);
        self.write8(r.a4 + OBJ_DIRECTION, 0);
        self.write8(r.a4 + OBJ_KIND, 2);
        self.zd_restart(r);
        self.zd_place(r);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01CDAA);
        self.write8(BACK, 0);
        self.write8(UP, 0);
        r.c = false;
    }

    /// `func_01D5E0`, the routine of `func_01D458`'s objects: they go
    /// after object 0 (`zd_chase`) until `BACK` is set or they are hit;
    /// then flags 0x87, kind 0x10, direction 0 (the animation started)
    /// and on as `func_01D314`.
    pub fn r_01d5e0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(BACK) == 0 {
            self.zd_chase(r);
            (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e7c(r.d1, r.d2, r.a0, r.a4, r.a5);
            if !r.c {
                return;
            }
        }
        self.write8(r.a5 + SPRITE_FLAGS, 0x87);
        self.write8(r.a4 + OBJ_KIND, 0x10);
        self.write8(r.a4 + OBJ_DIRECTION, 0);
        self.zd_restart(r);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01D314);
        r.c = false;
    }

    /// `.L01D632`: the boss hit through: its animation a frame on, then
    /// direction 0, kind 0x12 (started), on as `func_01D670`; `UP`
    /// 0xFF, bit 0 of `SPARED` and bit 6 of `FLAGS` set,
    /// `CLEARED_80DF` 0.  Gives back d0, d1, d2, d5, a0, a5, X and C
    /// (clear).
    #[allow(clippy::type_complexity)]
    pub fn r_01d632(&mut self, d0: u32, d1: u32, d2: u32, d5: u32, a4: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d5, a4, x, ..Regs::default() };
        self.zd_animate(&mut r);
        self.write8(a4 + OBJ_DIRECTION, 0);
        self.write8(a4 + OBJ_KIND, 0x12);
        self.zd_restart(&mut r);
        self.write32(a4 + OBJ_ROUTINE, ROUTINE_01D670);
        self.write8(UP, 0xFF);
        let spared = self.read8(SPARED);
        self.write8(SPARED, spared | 1);
        self.write8(CLEARED_80DF, 0);
        let flags = self.read8(FLAGS);
        self.write8(FLAGS, flags | 0x40);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, false)
    }

    /// `func_01D670`, the boss's routine: once its animation ends, the
    /// next direction (started), its count (0x26), `SHAKES` and
    /// `SHAKE_OFFSET` 0, and on as `func_01D6A2`.
    pub fn r_01d670(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zd_animate(r);
        if !r.c {
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01D6A2);
        self.zd_step(r.a4 + OBJ_DIRECTION);
        self.zd_restart(r);
        self.write8(r.a4 + OBJ_COUNT, 0);
        self.write8(SHAKES, 0);
        self.write16(SHAKE_OFFSET, 0);
        r.c = false;
    }

    /// `.L01D6A0`: an `rts`.
    pub fn r_01d6a0(&mut self) {}

    /// `func_01D6A2`, the boss's routine: its animation; x moved by
    /// `SHAKE_STEPS` (by its count / 2), the offset the high byte of
    /// `SHAKE_OFFSET` (0x4E more each frame), placed, `func_004FD0`.
    /// Its count goes to 0x50 twice (`SHAKES`); then the next
    /// direction (started), offset 0, x 4 less, placed, and on as
    /// `func_01D722`.
    pub fn r_01d6a2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zd_animate(r);
        let half = (self.read8(r.a4 + OBJ_COUNT) & 0xFE) >> 1;
        r.d0 = low_word(r.d0, u16::from(half));
        r.a0 = SHAKE_STEPS;
        let step = self.read8(SHAKE_STEPS + u32::from(half));
        r.d0 = low_byte(r.d0, step);
        let x = self.read8(r.a4 + OBJ_X_LOW).wrapping_add(step);
        self.write8(r.a4 + OBJ_X_LOW, x);
        let offset = self.read16(SHAKE_OFFSET).wrapping_add(0x4E);
        self.write16(SHAKE_OFFSET, offset);
        self.write8(r.a4 + OBJ_OFFSET, (offset >> 8) as u8);
        r.d5 = low_byte(r.d5, self.read8(r.a4 + OBJ_SPRITE));
        r.a5 = self.read32(r.a4 + OBJ_SPRITE_ADDR);
        self.zd_place(r);
        (r.d0, r.x, r.c) = self.r_004fd0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        let count;
        (count, r.x) = self.zd_step(r.a4 + OBJ_COUNT);
        if count != 0x50 {
            r.c = count < 0x50;
            return;
        }
        self.write8(r.a4 + OBJ_COUNT, 0);
        let shakes;
        (shakes, r.x) = self.zd_step(SHAKES);
        if shakes != 2 {
            r.c = shakes < 2;
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01D722);
        self.zd_step(r.a4 + OBJ_DIRECTION);
        self.zd_restart(r);
        self.write8(r.a4 + OBJ_OFFSET, 0);
        let x = self.read8(r.a4 + OBJ_X_LOW).wrapping_sub(4);
        self.write8(r.a4 + OBJ_X_LOW, x);
        self.zd_place(r);
    }

    /// `func_01D722`, the boss's routine: its count (0x26) up; at 0x3C
    /// the Z80 gets `DOWN_SOUND` and it goes on as `loc_01D740`, which
    /// it runs.
    pub fn r_01d722(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count;
        (count, r.x) = self.zd_step(r.a4 + OBJ_COUNT);
        if count == 0x3C {
            r.d0 = low_byte(r.d0, DOWN_SOUND);
            self.r_0034b6(r.d0);
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01D740);
            r.c = false;
        } else {
            r.c = count < 0x3C;
        }
        self.r_01d740(r);
    }

    /// `loc_01D740`, the boss's routine: once its animation ends the
    /// boss ends (its sprite record freed, `func_000890`, its first byte
    /// cleared) and the task `func_019262` is made (`func_0003E8`,
    /// priority 0x50).
    pub fn r_01d740(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zd_animate(r);
        if !r.c {
            return;
        }
        self.r_000890();
        self.write8(r.a4 + OBJ_ID, 0);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, END_PRIORITY, ROUTINE_019262);
    }
}
