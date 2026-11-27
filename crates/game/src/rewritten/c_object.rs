//! Objects 0-5 (`func_001CFA`'s first, before the stage's from object
//! 6, `c_spawn.rs`): the count at `OBJ_COUNT` an object loses
//! (`func_0034E6`), its end when it runs out (`func_003032`) and the
//! routines it runs then; objects 0 and 5 (object 0 where bit 7 of its
//! `OBJ_WHICH` is set, else object 5).

use crate::{Game, Regs};

/// An object's fields (`anim.rs`, `c_chase.rs` have more).
/// Its routine (a long; the objects' family) and one kept.
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_ROUTINE_KEPT: u32 = 0x06;
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
/// Copied to its sprite record's byte 3.
const OBJ_0F: u32 = 0x0F;
/// Bit 7: the object has ended (`func_003032`).
const OBJ_FLAGS: u32 = 0x10;
/// A count: `func_0034E6` takes one off, `func_003032` ends the object
/// once it's 0.
const OBJ_COUNT: u32 = 0x11;
/// While this isn't 0, `func_003032` leaves `OBJ_COUNT` alone; 0x40
/// once it takes one off, 0x80 from `func_0034E6`.
const OBJ_WAIT: u32 = 0x12;
const OBJ_DIRECTION: u32 = 0x13;
const OBJ_ANIM_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x15;
/// Its position (words, x then y; their low bytes alone for
/// `func_00334C`).
const OBJ_X: u32 = 0x18;
const OBJ_X_LOW: u32 = 0x19;
const OBJ_Y: u32 = 0x1A;
const OBJ_Y_LOW: u32 = 0x1B;
/// The animation running and its timer and step (`anim.rs`).
const OBJ_ANIM: u32 = 0x20;
const OBJ_ANIM_TIMER: u32 = 0x24;
const OBJ_ANIM_STEP: u32 = 0x25;
const OBJ_26: u32 = 0x26;
const OBJ_28: u32 = 0x28;
/// Set to 0xFF for objects 0 and 5 by `func_0033C0`.
const OBJ_29: u32 = 0x29;
/// Bit 5 set by `func_003032`.
const OBJ_2D: u32 = 0x2D;
/// Bit 7 (object 0's): object 0 is the one taken, else object 5.
const OBJ_WHICH: u32 = 0x32;
const OBJ_3A: u32 = 0x3A;

/// A sprite record's fields (`sprite.rs`).
const SPRITE_DEPTH: u32 = 0x02;
const SPRITE_3: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// Objects 0 and 5.
const OBJECT_0: u32 = 0xFFFF_A432;
const OBJECT_5: u32 = 0xFFFF_A572;
/// Object 0's position (words).
const OBJECT_0_X: u32 = OBJECT_0 + OBJ_X;
const OBJECT_0_Y: u32 = OBJECT_0 + OBJ_Y;

/// The object being run (`func_001CFA`'s index, its address).
const CURRENT_INDEX: u32 = 0xFFA42C;
const CURRENT: u32 = 0xFFA42E;
/// While this isn't 0, objects lose nothing (`func_003032`,
/// `func_0034E6`).
const SPARED: u32 = 0xFF9548;
/// Bit 0 picks what happens (`func_003032`, `func_003166`); bits 2-3
/// set by `func_0031D4`.
const MODE: u32 = 0xFF9540;
/// The map cell `func_00167A` last looked at (a pointer).
const CELL_POINTER: u32 = 0xFF9ED6;
/// An object's index (`func_001CFA`) whose `OBJ_WAIT` keeps another's
/// count (`func_003032`).
const OTHER_INDEX: u32 = 0xFFA93F;
/// Indexes of objects by object (from object 5; `func_003032`).
const OBJECT_BY_OBJECT: u32 = 0xFFFF_A968;
/// Set by `func_003166`.
const BYTE_AC7C: u32 = 0xFFAC7C;
const BYTE_AC7D: u32 = 0xFFAC7D;
/// Bytes set by `func_00320A`.
const BYTE_A939: u32 = 0xFFA939;
const BYTE_954E: u32 = 0xFF954E;
const BYTE_A93A: u32 = 0xFFA93A;
/// Counted by `func_0032AE`.
const COUNT_A93C: u32 = 0xFFA93C;
/// `func_00334C`'s argument, and what it sets for `func_004ED8`.
const ARG_AABD: u32 = 0xFFAABD;
const BYTE_A932: u32 = 0xFFA932;
const BYTE_9F02: u32 = 0xFF9F02;
const BYTE_9F03: u32 = 0xFF9F03;
const WORD_9EEC: u32 = 0xFF9EEC;
const BYTE_9EEB: u32 = 0xFF9EEB;
const LONG_9EEE: u32 = 0xFF9EEE;
/// Set while `func_00341E` runs for the object (`func_0033C0`), and
/// the routine it had kept (object 1's `OBJ_ROUTINE_KEPT`, by its
/// address), back once `func_002E24` says so.
const DIVERTED: u32 = 0xFFA941;
const KEPT_ROUTINE: u32 = 0xFFA478;
/// 2, or 3 where the low 4 bits of the byte at `BYTES_AD0A` + 3 aren't
/// 0 (`func_00341E`).
const BYTE_A942: u32 = 0xFFA942;
const BYTES_AD0A: u32 = 0xFFFF_AD0A;

/// Routines and data the objects get.
const ROUTINE_1F7E: u32 = 0x001F7E;
const ROUTINE_20AE: u32 = 0x0020AE;
const ROUTINE_3166: u32 = 0x003166;
const ROUTINE_31D4: u32 = 0x0031D4;
const ROUTINE_31F4: u32 = 0x0031F4;
const ROUTINE_320A: u32 = 0x00320A;
const ROUTINE_3242: u32 = 0x003242;
const ROUTINE_32E2: u32 = 0x0032E2;
const ROUTINE_334A: u32 = 0x00334A;
const ROUTINE_341E: u32 = 0x00341E;
const ROUTINE_45E2: u32 = 0x0045E2;
const ROUTINE_4E5C: u32 = 0x004E5C;
const ANIM_ECA70: u32 = 0x0ECA70;
const ANIM_ECA7A: u32 = 0x0ECA7A;
/// The priority of the tasks made here.
const PRIORITY: u8 = 0x20;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// Object 0 or 5 (bit 7 of object 0's `OBJ_WHICH`): its index and
    /// address.
    fn object_0_or_5(&mut self) -> (u32, u32) {
        if self.read8(OBJECT_0 + OBJ_WHICH) & 0x80 != 0 {
            (0, OBJECT_0)
        } else {
            (5, OBJECT_5)
        }
    }

    /// `func_003032`: whether the object at a4 ends (C).  Unless
    /// `SPARED`: with its count at 0, or on a cell of kind 0, 3 or 7
    /// and on, not ended, while the object at `OTHER_INDEX` has no
    /// `OBJ_WAIT` it loses one of its count (`OBJ_WAIT` 0x40 after).
    /// It ends once the count is out: sound 0x32, its routine
    /// `func_003166`; an object from 5 on makes the object
    /// `OBJECT_BY_OBJECT` gives for it a new one (routine
    /// `func_004E5C`, `func_004FD0`), d4 and a4 the object being run
    /// after; one before 5 goes to `func_001EA2`.  Gives back d0, d1,
    /// d4, d5, a0, a4, a5, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_003032(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, obj: u32, a5: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (mut d0, mut d1, mut a0, mut x) = (d0, d1, a0, x);
        if self.read8(SPARED) != 0 {
            return (d0, d1, d4, d5, a0, obj, a5, x, false);
        }
        if self.read8(obj + OBJ_COUNT) != 0 {
            a0 = self.read32(CELL_POINTER);
            let cell = self.read8(a0);
            d0 = low_byte(d0, cell);
            if cell != 3 && cell != 0 && cell < 7 {
                return (d0, d1, d4, d5, a0, obj, a5, x, false);
            }
            if self.read8(obj + OBJ_FLAGS) & 0x80 != 0 {
                return (d0, d1, d4, d5, a0, obj, a5, x, false);
            }
            let other_index = self.read8(OTHER_INDEX);
            let (_, other, x1) = self.r_001cfa(low_byte(d4, other_index));
            x = x1;
            let wait = self.read8(other + OBJ_WAIT);
            d0 = low_byte(d0, wait);
            if wait != 0 {
                return (d0, d1, d4, d5, a0, obj, a5, x, false);
            }
            let count = self.read8(obj + OBJ_COUNT);
            self.write8(obj + OBJ_COUNT, count.wrapping_sub(1));
            x = count == 0;
            if count != 1 {
                self.write8(obj + OBJ_WAIT, 0x40);
                return (d0, d1, d4, d5, a0, obj, a5, x, false);
            }
        }
        d0 = low_byte(d0, 0x32);
        self.r_0034b6(d0);
        self.write8(obj + OBJ_ANIM_DIRECTION, 0);
        self.write8(obj + OBJ_KIND, 0xC);
        // (its a0 and X are set again below)
        self.r_00130e(d0, obj);
        self.write8(obj + OBJ_ANIM_TIMER, 0);
        self.write8(obj + OBJ_ANIM_STEP, 0);
        let flags = self.read8(obj + OBJ_FLAGS);
        self.write8(obj + OBJ_FLAGS, flags | 0x80);
        self.write32(obj + OBJ_ROUTINE, ROUTINE_3166);
        if (d4 as u8) < 5 {
            let b = self.read8(obj + OBJ_2D);
            self.write8(obj + OBJ_2D, b | 0x20);
            (d0, d1, a0, x) = self.r_001ea2(d0, d4);
            return (d0, d1, d4, d5, a0, obj, a5, x, true);
        }
        if self.read8(MODE) & 1 == 0 {
            self.write8(OBJECT_0 + OBJ_WAIT, 0x60);
        }
        let index = (d4 as u8) as i8 as i16;
        a0 = OBJECT_BY_OBJECT;
        let new_index = self.read8(OBJECT_BY_OBJECT.wrapping_sub(5).wrapping_add(index as i32 as u32));
        let (d4, new, _) = self.r_001cfa(low_byte(low_word(d4, index as u16), new_index));
        self.write8(new + OBJ_3A, 0);
        self.write16(new + OBJ_28, 0xFD00);
        self.write8(new + OBJ_ANIM_STEP, 0);
        self.write8(new + OBJ_ANIM_TIMER, 0);
        let anim = self.read8(new + OBJ_ANIM_DIRECTION) | 0x80;
        self.write8(new + OBJ_ANIM_DIRECTION, anim);
        d0 = low_byte(d0, anim & 3);
        self.write8(new + OBJ_DIRECTION, anim & 3);
        self.write8(new + OBJ_KIND, 0);
        self.write8(new + OBJ_WHICH, 0xFF);
        self.write16(new + OBJ_26, 0);
        self.write32(new + OBJ_ROUTINE_KEPT, ROUTINE_1F7E);
        self.write32(new + OBJ_ROUTINE, ROUTINE_4E5C);
        (d0, x, _) = self.r_004fd0(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, new, a5);
        let d4 = low_byte(d4, self.read8(CURRENT_INDEX));
        let current = self.read32(CURRENT);
        (d0, d1, d4, d5, a0, current, a5, x, true)
    }

    /// `func_003166` (an object's routine, after `func_003032`): its
    /// animation (`func_001336`) to its end; then the sprite's frame 0
    /// and, by bit 0 of `MODE`, 0xF0 or 0x10 to `BYTE_AC7D` (and
    /// `BYTE_AC7C` for objects before 5): with bit 0, `func_006802` and
    /// `func_011FD4`; without, routine `func_0031D4` after 0x80 frames,
    /// sound 0xC.
    pub fn r_003166(&mut self, regs: &mut Regs) {
        let obj = regs.a4;
        let byte = self.read8(obj + OBJ_0F);
        self.write8(regs.a5 + SPRITE_3, byte);
        let (d0, d1, d2, d5, a0, a5, x, _, c) = self.r_001336(regs.d0, regs.d1, regs.d2, regs.d5, obj, regs.x);
        (regs.d0, regs.d1, regs.d2, regs.d5, regs.a0, regs.a5, regs.x, regs.c) = (d0, d1, d2, d5, a0, a5, x, c);
        if !c {
            return;
        }
        self.write8(a5 + SPRITE_FRAME, 0);
        self.write8(obj, 0);
        regs.c = false;
        if self.read8(MODE) & 1 != 0 {
            regs.d0 = low_byte(regs.d0, 0xF0);
            self.write8(BYTE_AC7D, 0xF0);
            if regs.d4 as u8 >= 5 {
                return;
            }
            self.write8(BYTE_AC7C, 0xF0);
            self.r_006802(regs.d1, regs.d2, regs.d3, regs.d4, regs.d5, regs.d6, regs.d7, regs.a1, regs.a2, regs.a3, regs.a4, regs.a5);
            let (d0, d4, d6, a4, a6, x, c) = self.r_011fd4(regs.d0, regs.d4, regs.d5, regs.d6, regs.a6);
            (regs.d0, regs.d4, regs.d6, regs.a4, regs.a6, regs.x, regs.c) = (d0, d4, d6, a4, a6, x, c);
        } else {
            regs.d0 = 0x10;
            self.write8(BYTE_AC7D, 0x10);
            if regs.d4 as u8 >= 5 {
                return;
            }
            self.write8(BYTE_AC7C, 0x10);
            self.write8(obj, 1);
            regs.x = false;
            self.write8(obj + OBJ_ANIM_TIMER, 0x80);
            self.write32(obj + OBJ_ROUTINE, ROUTINE_31D4);
            regs.d0 = low_byte(regs.d0, 0xC);
            self.r_0034b6(regs.d0);
        }
    }

    /// `func_0031D4`: once `OBJ_ANIM_TIMER` counts out, routine
    /// `func_0031F4`, bits 2-3 of `MODE` set, `func_00A51A` with 4.
    pub fn r_0031d4(&mut self, regs: &mut Regs) {
        let obj = regs.a4;
        let timer = self.read8(obj + OBJ_ANIM_TIMER);
        self.write8(obj + OBJ_ANIM_TIMER, timer.wrapping_sub(1));
        regs.x = timer == 0;
        regs.c = regs.x;
        if timer != 1 {
            return;
        }
        self.write32(obj + OBJ_ROUTINE, ROUTINE_31F4);
        let mode = self.read8(MODE);
        self.write8(MODE, mode | 0xC);
        let (d0, a1, x, c) = self.r_00a51a(4, regs.d1, regs.d2, regs.d3, regs.d4, regs.d5, regs.d6, regs.d7, regs.a2, regs.a3, regs.a4, regs.a5);
        (regs.d0, regs.a1, regs.x, regs.c) = (d0, a1, x, c);
    }

    /// `func_0031F4`: object 0's byte 0 cleared, and a task running
    /// `func_00320A`.
    pub fn r_0031f4(&mut self, regs: &mut Regs) {
        self.write8(OBJECT_0, 0);
        let (d6, a6, x, c) = self.r_0003e8(regs.d4, regs.d5, regs.d6, PRIORITY, ROUTINE_320A);
        (regs.d6, regs.a6, regs.x, regs.c) = (d6, a6, x, c);
    }

    /// `func_00320A` (a task): `func_0099A2`, bytes set, the tasks
    /// (`func_0003DA`), `func_001C56`, the sprites (`func_000704`), and
    /// a task running `func_003242`.
    pub fn r_00320a(&mut self, regs: &mut Regs) {
        let (x, c) = self.r_0099a2(regs.d4);
        (regs.x, regs.c) = (x, c);
        regs.d0 = low_byte(regs.d0, 3);
        let b = self.read8(BYTE_A939);
        self.write8(BYTE_A939, b & 3);
        self.write8(BYTE_954E, 0xFF);
        self.write8(BYTE_A93A, 1);
        let (d0, d1, d6, a0, a6) = self.r_0003da(regs.d2, regs.d3, regs.d4, regs.d5, regs.d7, regs.a1, regs.a2, regs.a3, regs.a4, regs.a5);
        (regs.d0, regs.d1, regs.d6, regs.a0, regs.a6) = (d0, d1, d6, a0, a6);
        let (d6, a6, x) = self.r_001c56(regs.d4, regs.d5, regs.d6);
        (regs.d6, regs.a6, regs.x) = (d6, a6, x);
        let (d0, d1, a0, x) = self.r_000704(regs.d0);
        (regs.d0, regs.d1, regs.a0, regs.x) = (d0, d1, a0, x);
        let (d6, a6, x, c) = self.r_0003e8(regs.d4, regs.d5, regs.d6, PRIORITY, ROUTINE_3242);
        (regs.d6, regs.a6, regs.x, regs.c) = (d6, a6, x, c);
    }

    /// `func_0032AE` (an object's routine): counted in `COUNT_A93C`.
    /// Object 5: its sprite's frame 1, bit 4 of its flags, at depth
    /// (its depth - 1, `func_0007D2`), and nothing more
    /// (`func_00334A`).  Others: the animation `ANIM_ECA70` (with bit 7
    /// of object 0's `OBJ_WHICH`) or `ANIM_ECA7A`, and `func_0032E2`.
    pub fn r_0032ae(&mut self, regs: &mut Regs) {
        let obj = regs.a4;
        let count = self.read8(COUNT_A93C);
        self.write8(COUNT_A93C, count.wrapping_add(1));
        regs.x = count == 0xFF;
        if regs.d4 as u8 == 5 {
            let sprite = regs.a5;
            self.write8(sprite + SPRITE_FRAME, 1);
            let flags = self.read8(sprite + SPRITE_FLAGS);
            self.write8(sprite + SPRITE_FLAGS, flags | 0x10);
            let byte = self.read8(obj + OBJ_0F);
            self.write8(sprite + SPRITE_3, byte);
            let depth = self.read8(sprite + SPRITE_DEPTH);
            regs.d0 = low_byte(regs.d0, depth.wrapping_sub(1));
            regs.x = depth == 0;
            let r = *regs;
            self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
            self.write32(obj + OBJ_ROUTINE, ROUTINE_334A);
            regs.c = false;
            self.r_00334a(regs);
            return;
        }
        let anim = if self.read8(OBJECT_0 + OBJ_WHICH) & 0x80 != 0 { ANIM_ECA70 } else { ANIM_ECA7A };
        self.write32(obj + OBJ_ANIM, anim);
        self.write8(OBJECT_0 + OBJ_ANIM_TIMER, 0);
        self.write8(OBJECT_0 + OBJ_ANIM_STEP, 0);
        self.write32(obj + OBJ_ROUTINE, ROUTINE_32E2);
        regs.c = false;
        self.r_0032e2(regs);
    }

    /// `func_0032E2` (an object's routine): moves (`func_001B32`) and
    /// animates (`func_001336`); at the animation's end, routine
    /// `func_001F7E`, object 0's `OBJ_ANIM_DIRECTION` 2; but where bit 7 of object 0's
    /// `OBJ_WHICH` is clear, `func_0020AE`, and object 5 gets
    /// `func_0045E2` (direction 2).
    pub fn r_0032e2(&mut self, regs: &mut Regs) {
        let obj = regs.a4;
        let (d0, d1, x, _) = self.r_001b32(regs.d1, obj, regs.a5);
        let (d0, d1, d2, d5, a0, a5, x, _, c) = self.r_001336(d0, d1, regs.d2, regs.d5, obj, x);
        (regs.d0, regs.d1, regs.d2, regs.d5, regs.a0, regs.a5, regs.x, regs.c) = (d0, d1, d2, d5, a0, a5, x, c);
        if !c {
            return;
        }
        self.write32(obj + OBJ_ROUTINE, ROUTINE_1F7E);
        self.write8(OBJECT_0 + OBJ_ANIM_DIRECTION, 2);
        regs.c = false;
        if self.read8(OBJECT_0 + OBJ_WHICH) & 0x80 != 0 {
            return;
        }
        self.write32(obj + OBJ_ROUTINE, ROUTINE_20AE);
        let (d4, object_5, x) = self.r_001cfa(5);
        (regs.d4, regs.a4, regs.x) = (d4, object_5, x);
        self.write32(object_5 + OBJ_ROUTINE, ROUTINE_45E2);
        self.write8(object_5 + OBJ_ANIM_DIRECTION, 2);
        self.write8(OBJECT_0 + OBJ_ANIM_DIRECTION, 0);
    }

    /// `func_00334A` (an object's routine): nothing.  (No script runs
    /// it.)
    pub fn r_00334a(&mut self, _regs: &mut Regs) {}

    /// `func_00334C`: `func_004ED8` with `d0` (its low byte, kept in
    /// `ARG_AABD`) for object 0 or 5 (unless it has ended), run as the
    /// object being run, with its cell (`func_0015E6`, `func_00157A`),
    /// the low bytes of its position and words set up.  Gives back d1
    /// and a0.  (No script calls it: checked by reading.)
    #[allow(clippy::too_many_arguments)]
    pub fn r_00334c(&mut self, d0: u32, d1: u32, d2: u32, d5: u32, d6: u32, a0: u32, a6: u32) -> (u32, u32) {
        self.write8(ARG_AABD, d0 as u8);
        self.write8(OTHER_INDEX, 0);
        let (index, obj) = self.object_0_or_5();
        if self.read8(obj + OBJ_FLAGS) & 0x80 != 0 {
            return (d1, a0);
        }
        let current_index = self.read8(CURRENT_INDEX);
        let current = self.read32(CURRENT);
        self.write8(CURRENT_INDEX, index as u8);
        self.write32(CURRENT, obj);
        self.write8(BYTE_A932, 0);
        let (d0, _) = self.r_0015e6(obj);
        self.r_00157a();
        let x_low = self.read8(obj + OBJ_X_LOW);
        self.write8(BYTE_9F02, x_low);
        let y_low = self.read8(obj + OBJ_Y_LOW);
        self.write8(BYTE_9F03, y_low);
        self.write32(LONG_9EEE, 0);
        let word = self.read16(WORD_9EEC).wrapping_neg();
        self.write8(BYTE_9EEB, word as u8);
        let d0 = low_byte(low_word(d0, word), self.read8(ARG_AABD));
        let (d1, _, _, _, a0, _, _, _) = self.r_004ed8(d0, d1, d2, index, d5, d6, a6);
        self.write32(CURRENT, current);
        self.write8(CURRENT_INDEX, current_index);
        (d1, a0)
    }

    /// `func_0033C0`: object 0 and 5's `OBJ_29` to 0xFF; object 0 or 5,
    /// while its count isn't out: sound 0x57, and the first time
    /// (`DIVERTED`), its routine kept in `OBJ_ROUTINE_KEPT` (that one in
    /// `KEPT_ROUTINE`) and `func_00341E` run instead.  Gives back X.
    /// (No script calls it: checked by reading.)
    pub fn r_0033c0(&mut self) -> bool {
        self.write8(OBJECT_0 + OBJ_29, 0xFF);
        self.write8(OBJECT_5 + OBJ_29, 0xFF);
        let (index, _) = self.object_0_or_5();
        let (_, obj, x) = self.r_001cfa(index);
        if self.read8(obj + OBJ_COUNT) == 0 {
            return x;
        }
        self.r_0034b6(0x57);
        if self.read8(DIVERTED) != 0 {
            return x;
        }
        self.write8(DIVERTED, 1);
        let kept = self.read32(obj + OBJ_ROUTINE_KEPT);
        self.write32(KEPT_ROUTINE, kept);
        let routine = self.read32(obj + OBJ_ROUTINE);
        self.write32(obj + OBJ_ROUTINE_KEPT, routine);
        self.write32(obj + OBJ_ROUTINE, ROUTINE_341E);
        false
    }

    /// `func_00341E` (an object's routine, from `func_0033C0`):
    /// `BYTE_A942` 2 or 3, `func_001F06`; once `func_002E24` says so,
    /// the kept routine back (`KEPT_ROUTINE`) and `func_001EA2`; then
    /// its cell and `func_003032` (which ending clears `DIVERTED`).
    /// (No script runs it: checked by reading.)
    pub fn r_00341e(&mut self, regs: &mut Regs) {
        let obj = regs.a4;
        self.write8(BYTE_A942, 2);
        regs.a0 = BYTES_AD0A;
        let low = self.read8(BYTES_AD0A + 3) & 0xF;
        regs.d0 = low_byte(regs.d0, low);
        if low != 0 {
            self.write8(BYTE_A942, 3);
        }
        regs.d1 = self.r_001f06(regs.d0, 0);
        regs.d5 = low_byte(regs.d5, self.read8(obj + OBJ_SPRITE));
        regs.a5 = self.read32(obj + OBJ_SPRITE_ADDR);
        if self.r_002e24(obj) {
            let kept = self.read32(KEPT_ROUTINE);
            self.write32(obj + OBJ_ROUTINE_KEPT, kept);
            self.write8(DIVERTED, 0);
            let (d0, d1, a0, x) = self.r_001ea2(regs.d0, 0);
            (regs.d0, regs.d1, regs.a0, regs.x) = (d0, d1, a0, x);
        }
        let (d0, _) = self.r_0015e6(obj);
        let (a0, x) = self.r_00167a();
        let r = *regs;
        let (d0, d1, d4, d5, a0, a4, a5, x, c) = self.r_003032(d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, a0, r.a1, r.a2, r.a3, obj, r.a5, x);
        (regs.d0, regs.d1, regs.d4, regs.d5, regs.a0, regs.a4, regs.a5, regs.x, regs.c) = (d0, d1, d4, d5, a0, a4, a5, x, c);
        if c {
            self.write8(DIVERTED, 0);
            regs.c = false;
        }
    }

    /// `func_0034DE`: `func_0034E6` for the object at a4's position.
    /// Gives back d1, d2 and X.
    pub fn r_0034de(&mut self, d0: u32, d1: u32, d2: u32, obj: u32) -> (u32, u32, bool) {
        let d1 = low_word(d1, self.read16(obj + OBJ_X));
        let d2 = low_word(d2, self.read16(obj + OBJ_Y));
        let (d1, d2, x, _) = self.r_0034e6(d0, d1, d2);
        (d1, d2, x)
    }

    /// `func_003554`: `func_0034E6` for the position at a6 + 0xE (a
    /// task's, `anim.rs`).  Gives back d1, d2, X and C.
    pub fn r_003554(&mut self, d0: u32, d1: u32, d2: u32, task: u32) -> (u32, u32, bool, bool) {
        let d1 = low_word(d1, self.read16(task + 0xE));
        let d2 = low_word(d2, self.read16(task + 0x10));
        self.r_0034e6(d0, d1, d2)
    }

    /// 0x34E6: whether the position (`d1`, `d2`: words) is less than
    /// `d0` (its low word) from object 0's both ways (C); then, unless
    /// `SPARED` or object 0's `OBJ_WAIT` is set, object 0 or 5 (by
    /// object 0's `OBJ_WHICH`), unless it has ended, loses one of its
    /// count and object 0 gets `OBJ_WAIT` 0x80.  Gives back d1 and d2
    /// (the distances), X and C (one lost).
    pub fn r_0034e6(&mut self, d0: u32, d1: u32, d2: u32) -> (u32, u32, bool, bool) {
        let range = d0 as u16;
        let (d1, x) = self.distance_to(d1, OBJECT_0_X);
        if d1 as u16 >= range {
            return (d1, d2, x, false);
        }
        let (d2, x) = self.distance_to(d2, OBJECT_0_Y);
        if d2 as u16 >= range {
            return (d1, d2, x, false);
        }
        if self.read8(SPARED) != 0 || self.read8(OBJECT_0 + OBJ_WAIT) != 0 {
            return (d1, d2, x, false);
        }
        let (_, obj) = self.object_0_or_5();
        if self.read8(obj + OBJ_FLAGS) & 0x80 != 0 {
            return (d1, d2, x, false);
        }
        let count = self.read8(obj + OBJ_COUNT);
        self.write8(obj + OBJ_COUNT, count.wrapping_sub(1));
        self.write8(OBJECT_0 + OBJ_WAIT, 0x80);
        (d1, d2, count == 0, true)
    }

    /// |`reg`'s low word - the word at `at`| (`sub.w`, then `neg.w`
    /// where negative), and the X it leaves.
    fn distance_to(&mut self, reg: u32, at: u32) -> (u32, bool) {
        let w = reg as u16;
        let other = self.read16(at);
        let d = w.wrapping_sub(other);
        if d & 0x8000 != 0 {
            (low_word(reg, d.wrapping_neg()), true)
        } else {
            (low_word(reg, d), w < other)
        }
    }
}
