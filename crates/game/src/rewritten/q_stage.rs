//! The screen `func_0127C2` sets up (0x127C2-0x12CBC) and its object 0
//! (`func_01288E`): an object with an animation (`func_0128E6`) that
//! then moves by a speed and a position kept in `SPEED_965E` and
//! `POSITION_965C` (`func_012AB4`), bobbing (`func_012A8C`), now and
//! then pausing when the position comes round to where it last paused
//! (`func_0129E4`, `func_012A46`, `func_012A62`), until `DONE_9660`
//! is 0: then a task (`func_0003E8`, `func_013750`) and its count
//! (`func_012AF8`, `func_012B48`).  `func_012B8E`, `func_012BD8` and
//! the task `func_012C7E` are a later state of the same object.

use crate::{Game, Regs};

/// An object's fields: its kind (byte 0), its routine and the one it
/// keeps (longs; the routines here also write the kept one's first
/// byte), its position's low bytes, a byte set to 1 (0x12), its next
/// direction (0x13, negative: none) and direction (0x14), its
/// animation (a long) and the animation's counters (`anim.rs`; also a
/// count here, 0x24), two bytes (0x26, 0x27) and its offset (the high
/// byte of a word, `a_motion.rs`) and a byte counted up under it.
const OBJ_KIND: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_SAVED: u32 = 0x06;
const OBJ_X_LOW: u32 = 0x0F;
const OBJ_Y_LOW: u32 = 0x11;
const OBJ_12: u32 = 0x12;
const OBJ_NEXT_DIRECTION: u32 = 0x13;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_ANIM: u32 = 0x20;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_26: u32 = 0x26;
const OBJ_27: u32 = 0x27;
const OBJ_OFFSET: u32 = 0x2A;
const OBJ_PHASE: u32 = 0x2B;

/// A task's fields (`a_motion.rs`): a count (0x16), its rate (a
/// word) and offset (byte).
const TASK_COUNT: u32 = 0x16;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;

/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// Bits the routines set and clear: `MODE` bit 6 (with `SPARED` bit 0).
const MODE: u32 = 0xFFFF_9540;
const SPARED: u32 = 0xFFFF_9548;
const MODE_BIT6: u8 = 1 << 6;
const SPARED_BIT0: u8 = 1;
/// Cleared by `func_0127C2` (`BUSY_A945` as in `q_states.rs`).
const BUSY_A945: u32 = 0xFFFF_A945;
const BYTE_A94C: u32 = 0xFFFF_A94C;
/// A byte `func_0127C2` sets to 0xC while it calls `func_001030`.
const BYTE_953A: u32 = 0xFFFF_953A;
/// Bytes `func_0127C2` sets.
const BYTE_953F: u32 = 0xFFFF_953F;
const BYTE_9EB3: u32 = 0xFFFF_9EB3;
const BYTE_9EB9: u32 = 0xFFFF_9EB9;
const BYTE_80DF: u32 = 0xFFFF_80DF;
const BYTE_AC7D: u32 = 0xFFFF_AC7D;
/// Set to 1 by `func_0127C2`, cleared by `func_0128E6`.
const BYTE_954F: u32 = 0xFFFF_954F;
/// The list `func_0094D8` reads (a long), and the ones given it (ROM).
const LIST: u32 = 0xFFFF_92FA;
const LIST_12882: u32 = 0x012882;
const LIST_12B5A: u32 = 0x012B5A;
/// `func_0127C2`'s sprite frames' table (`func_000776`, set 1; ROM).
const FRAMES_E649E: u32 = 0x0E649E;
/// `func_012B8E` waits for it to be 0.
const WAIT_9432: u32 = 0xFFFF_9432;
/// Work RAM's first bytes, 0x340 cleared by `func_012B8E`.
const RAM_START: u32 = 0xFFFF_0000;
const RAM_CLEARED: u32 = 0x340;

/// A position (words, `a_objects.rs`) and its x's low byte.
const POSITION_X: u32 = 0xFFFF_9F04;
const POSITION_X_LOW: u32 = 0xFFFF_9F05;
const POSITION_Y: u32 = 0xFFFF_9F06;
/// The velocity's x (a word, `a_motion.rs`).
const VELOCITY_X: u32 = 0xFFFF_9EC4;
/// Longs `func_01288E` sets: one to `D_010078`, one to `func_0128E6`.
const LONG_80C8: u32 = 0xFFFF_80C8;
const LONG_80D0: u32 = 0xFFFF_80D0;
const D_010078: u32 = 0x010078;
/// A word added to `POSITION_965C` each frame (`func_012AEE`), kept
/// between 0x200 and 0x600, and the word it is added to; their high
/// bytes are compared by `func_0129E4`.
const POSITION_965C: u32 = 0xFFFF_965C;
const SPEED_965E: u32 = 0xFFFF_965E;
const SPEED_965E_LOW: u32 = 0xFFFF_965F;
/// What is added to `SPEED_965E` (a byte), and that signed as a word
/// (its high byte the sign).
const ACCEL: u32 = 0xFFFF_AC8B;
const ACCEL_WORD: u32 = 0xFFFF_AC8A;
/// Set to 1 by `func_012A46`, counted round by `func_012A62`.
const COUNT_AD56: u32 = 0xFFFF_AD56;
/// Cleared by `func_012AF8`.
const COUNT_AD57: u32 = 0xFFFF_AD57;
/// Cleared by `func_01288E` (`func_01262C` waits for it).
const START_9663: u32 = 0xFFFF_9663;
/// While it is 0, `func_012AF8` ends the moving.
const DONE_9660: u32 = 0xFFFF_9660;

/// The object running (index, address) and its sprite record's
/// (`a_objects.rs`): reloaded after making tasks.
const RUNNING: u32 = 0xFFFF_A42C;
const RUNNING_ADDR: u32 = 0xFFFF_A42E;
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// Tables in the ROM: the sprite's frame by direction, the offsets by
/// phase.
const FRAMES_BY_DIRECTION: u32 = 0x0129E0;
const BOB: u32 = 0x012AAC;
/// Animations in the ROM.
const ANIM_E854C: u32 = 0x0E854C;

/// The routines.
const STARTING: u32 = 0x0128E6;
const MOVING: u32 = 0x01296C;
const CRUISING: u32 = 0x0129E4;
const PAUSED: u32 = 0x012A46;
const PAUSED_AFTER: u32 = 0x012A62;
const COUNTING: u32 = 0x012B48;
const COUNTED: u32 = 0x01338A;
const TASK_13750: u32 = 0x013750;
const LATER: u32 = 0x012BD8;
const LATER_TASK: u32 = 0x012C7E;
const LATER_NEXT: u32 = 0x012CBC;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// d4, a4, d5 and a5 back to the object running and its sprite
    /// record.
    fn q_reload(&mut self, r: &mut Regs) {
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
    }

    /// The bits of `MODE` and `SPARED` cleared.
    fn q_clear_mode(&mut self) {
        let m = self.read8(MODE);
        self.write8(MODE, m & !MODE_BIT6);
        let s = self.read8(SPARED);
        self.write8(SPARED, s & !SPARED_BIT0);
    }

    /// `func_0127C2`: the screen set up: `func_0003DA`, VRAM cleared
    /// (`func_0002E6`), the sprite records (`func_000704`), the objects
    /// (`func_001C56`), `func_00B49A`, `func_009874`, frames set 1
    /// (`FRAMES_E649E`), the picture off, `func_000E3E`, `func_001030`
    /// (with `BYTE_953A` 0xC), `func_0094D8` on `LIST_12882`,
    /// `func_000E9E`, `func_00ABE0`, `func_00ABA6`, object 0
    /// (`func_01288E`), `func_0010EC`, `func_009B0A`, `func_00AF26`, the
    /// objects run once (`func_001C96`), sound 0x10 and `func_00A506`
    /// with 4.  Gives back the registers as the original leaves them.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0127c2(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0: 0, d1: 0, d2, d3, d4, d5, d6: 0, d7, a0: 0, a1, a2, a3, a4, a5, a6: 0, x: false, n: false, z: false, v: false, c: false, unwound: false };
        self.q_set_up(&mut r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_0127C2` on the registers; it stops where the game
    /// restarts.
    fn q_set_up(&mut self, r: &mut Regs) {
        self.write8(BUSY_A945, 0);
        self.write8(BYTE_A94C, 0);
        self.write8(MODE, 0xC0);
        self.write8(SPARED, 1);
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.r_0002e6();
        (r.d0, r.d1, r.a0, r.x) = self.r_000704(r.d0);
        (r.d6, r.a6, r.x) = self.r_001c56(r.d4, r.d5, r.d6);
        self.r_00b49a(r);
        (r.d0, r.a0, r.a1, r.x) = self.r_009874(r.a1);
        r.a0 = FRAMES_E649E;
        r.d0 = 1;
        r.d0 = self.r_000776(r.d0, r.a0, r.a1);
        self.r_0002e0();
        (r.d0, r.d1, r.d6, r.a6, r.x) = self.r_000e3e(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        let kept = self.read8(BYTE_953A);
        self.write8(BYTE_953A, 0x0C);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_001030(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        if self.restarted.is_some() {
            return;
        }
        self.write8(BYTE_953A, kept);
        self.write32(LIST, LIST_12882);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return;
        }
        self.write8(BYTE_953F, 0x30);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_000e9e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return;
        }
        (r.d0, r.d1, r.x) = self.r_00abe0(r.d0, r.d1, r.x);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_00aba6(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return;
        }
        self.write8(BYTE_9EB3, 0x10);
        self.write8(BYTE_9EB9, 0x20);
        self.write8(BYTE_80DF, 1);
        (r.d0, r.d4, r.d5, r.a4, r.a5, r.x) = self.r_01288e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        self.r_0010ec();
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6) =
            self.r_009b0a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        if self.restarted.is_some() {
            return;
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x) =
            self.r_00af26(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6);
        self.write8(BYTE_AC7D, 0);
        self.write8(BYTE_954F, 1);
        r.c = false;
        self.r_001c96(r);
        if self.restarted.is_some() {
            return;
        }
        r.d0 = low_byte(r.d0, 0x10);
        self.r_0034b6(r.d0);
        r.d0 = 4;
        (r.d0, r.a1, r.x, r.c) = self.r_00a506(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_01288E`: object 6 (`func_001CFA`) at (0x88, 0x78) with a
    /// sprite record (`func_0012A0`; `LONG_80C8`, `LONG_80D0` set
    /// first), flags 0x81, frame 0x1F, its x's low byte 0x21,
    /// animation `ANIM_E854C`; `POSITION_965C`'s high byte,
    /// `SPEED_965E`, `COUNT_AD56`, `START_9663` and `DONE_9660` 0.
    /// Gives back d0, d4, d5, a4, a5 and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_01288e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool) {
        let (d4, a4, x) = self.r_001cfa(low_byte(d4, 6));
        self.write16(POSITION_X, 0x88);
        self.write16(POSITION_Y, 0x78);
        self.write32(LONG_80C8, D_010078);
        self.write32(LONG_80D0, STARTING);
        let (d0, d5, a5, x) = self.r_0012a0(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        self.write8(a5 + SPRITE_FLAGS, 0x81);
        self.write8(a4 + OBJ_X_LOW, 0x21);
        self.write8(a5 + SPRITE_FRAME, 0x1F);
        self.write32(a4 + OBJ_ANIM, ANIM_E854C);
        self.write8(POSITION_965C, 0);
        self.write16(SPEED_965E, 0);
        self.write8(COUNT_AD56, 0);
        self.write8(START_9663, 0);
        self.write8(DONE_9660, 0);
        (d0, d4, d5, a4, a5, x)
    }

    /// `func_0128E6`: `BYTE_954F` cleared, the animation a frame on;
    /// when it ends, `func_004FD0`, the sprite's frame 0x15, offset
    /// 0xF2, kind 0x12, `func_01296C` next (the kept routine's first
    /// byte 0x80, `OBJ_26` 3, `OBJ_27` 2), sound 0x11 and the bits of
    /// `MODE`, `SPARED` cleared.  At the animation's step 0x10 (its
    /// timer 0), `func_013508` with 0x3C.  Placed (`func_001B32`,
    /// `func_001B5C`).
    pub fn r_0128e6(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write8(BYTE_954F, 0);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if r.c {
            (r.d0, r.x, r.c) = self.r_004fd0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            self.write8(r.a5 + SPRITE_FRAME, 0x15);
            self.write8(r.a4 + OBJ_OFFSET, 0xF2);
            self.write8(r.a4 + OBJ_SAVED, 0x80);
            self.write8(r.a4 + OBJ_26, 3);
            self.write8(r.a4 + OBJ_27, 2);
            self.write8(r.a4 + OBJ_KIND, 0x12);
            self.write32(r.a4 + OBJ_ROUTINE, MOVING);
            r.d0 = low_byte(r.d0, 0x11);
            self.r_0034b6(r.d0);
            self.q_clear_mode();
        }
        let timer = self.read8(r.a4 + OBJ_TIMER);
        r.d0 = low_byte(r.d0, timer);
        if timer == 0 {
            let step = self.read8(r.a4 + OBJ_STEP);
            r.d0 = low_byte(r.d0, step);
            if step == 0x10 {
                r.d0 = low_byte(r.d0, 0x3C);
                (r.d0, r.d4, r.d5, r.a4, r.a5, r.x, r.c) =
                    self.r_013508(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
                self.q_reload(r);
            }
        }
        (r.d0, r.d1, r.x, r.c) = self.r_012bd0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        r.unwound = false;
    }

    /// `func_01296C`: unless the kept routine's high nibble is set (a
    /// mark), it is set (0x80), `func_0129E4` from now on with a count
    /// of 0 or 0x20 (a random bit), and run now.  Else the speed up by
    /// 8 (`func_012AB4`), bobbing (`func_012A8C`), `OBJ_12` 1,
    /// `func_007174`, the sprite's frame 0x15 (or by its next direction
    /// when it has one: `FRAMES_BY_DIRECTION` by its direction), then
    /// `func_012AF8`.
    pub fn r_01296c(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let mark = self.read8(r.a4 + OBJ_SAVED) & 0xF0;
        r.d0 = low_byte(r.d0, mark);
        if mark == 0 {
            self.write8(r.a4 + OBJ_SAVED, 0x80);
            self.write32(r.a4 + OBJ_ROUTINE, CRUISING);
            (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
            let count = (r.d0 as u8) & 0x20;
            r.d0 = low_byte(r.d0, count);
            self.write8(r.a4 + OBJ_TIMER, count);
            r.c = false;
            self.r_0129e4(r);
            return;
        }
        self.write8(ACCEL, 8);
        (r.d0, r.x) = self.r_012ab4(r.d0);
        (r.d0, r.d5, r.a5, r.x) = self.r_012a8c(r.d0, r.d5, r.a4);
        self.write8(r.a4 + OBJ_12, 1);
        r.n = false;
        r.z = false;
        r.c = false;
        self.r_007174(r);
        if self.restarted.is_some() {
            return;
        }
        self.write8(r.a5 + SPRITE_FRAME, 0x15);
        let next = self.read8(r.a4 + OBJ_NEXT_DIRECTION);
        r.d0 = low_byte(r.d0, next);
        if next & 0x80 == 0 {
            let direction = self.read8(r.a4 + OBJ_DIRECTION) & 3;
            r.d0 = low_word(r.d0, u16::from(direction));
            r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
            r.a5 = self.read32(SLOT_ADDR);
            let frame = self.read8(FRAMES_BY_DIRECTION + u32::from(direction));
            self.write8(r.a5 + SPRITE_FRAME, frame);
        }
        (r.d0, r.d4, r.d6, r.a4, r.a6, r.x, r.c) = self.r_012af8(r.d0, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x);
        r.unwound = false;
    }

    /// `func_0129E4`: the sprite's frame 0x15, the speed down by 0x20
    /// (`func_012AB4`).  When the speed is 0x200 (its high byte 2, its
    /// low 0) and the position's high byte within 1 of `OBJ_TIMER`,
    /// that becomes the position's high byte, a count of 0x1E and
    /// `func_012A46` next.  Then `func_012A34`.
    pub fn r_0129e4(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write8(r.a5 + SPRITE_FRAME, 0x15);
        self.write8(ACCEL, 0xE0);
        (r.d0, r.x) = self.r_012ab4(r.d0);
        let low = self.read8(SPEED_965E_LOW);
        r.d0 = low_byte(r.d0, low);
        r.c = false;
        if low == 0 {
            let high = self.read8(SPEED_965E);
            r.d0 = low_byte(r.d0, high);
            r.c = high < 2;
            if high == 2 {
                let mine = self.read8(r.a4 + OBJ_TIMER);
                let there = self.read8(POSITION_965C);
                let mut d = mine.wrapping_sub(there);
                r.x = mine < there;
                if d & 0x80 != 0 {
                    r.x = d != 0;
                    d = d.wrapping_neg();
                }
                r.d0 = low_byte(r.d0, d);
                r.c = d < 2;
                if d < 2 {
                    self.write8(POSITION_965C, mine);
                    self.write32(r.a4 + OBJ_ROUTINE, PAUSED);
                    self.write8(r.a4 + OBJ_TIMER, 0x1E);
                    r.c = false;
                }
            }
        }
        self.q_012a34(r);
        r.unwound = false;
    }

    /// `func_012A34` on the registers.
    fn q_012a34(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_012a34(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a6);
    }

    /// `func_012A34` (the end of the moving routines): bobbing
    /// (`func_012A8C`), placed (`func_001B32`, `func_001B5C`), then
    /// `func_012AF8`.  Gives back d0, d1, d4, d5, d6, a4, a5, a6, X and
    /// C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_012a34(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (_, d5, a5, _) = self.r_012a8c(d0, d5, a4);
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        let (d0, x, _) = self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5);
        let (d0, d4, d6, a4, a6, x, c) = self.r_012af8(d0, d4, d5, d6, a4, a5, a6, x);
        (d0, d1, d4, d5, d6, a4, a5, a6, x, c)
    }

    /// `OBJ_TIMER` counted down (X its borrow); whether it got to 0.
    fn q_count_down(&mut self, r: &mut Regs) -> bool {
        let n = self.read8(r.a4 + OBJ_TIMER);
        self.write8(r.a4 + OBJ_TIMER, n.wrapping_sub(1));
        r.x = n == 0;
        r.c = r.x;
        n == 1
    }

    /// `func_012A46` (paused): counted down; then `COUNT_AD56` 1, a
    /// count of 0x40 and `func_012A62` next.  Then `func_012A34`.
    pub fn r_012a46(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.q_count_down(r) {
            self.write8(COUNT_AD56, 1);
            self.write32(r.a4 + OBJ_ROUTINE, PAUSED_AFTER);
            self.write8(r.a4 + OBJ_TIMER, 0x40);
            r.c = false;
        }
        self.q_012a34(r);
        r.unwound = false;
    }

    /// `func_012A62`: `COUNT_AD56`, when not 0, counted up (back to 0
    /// past 0xFF: the original compares it with 5 but tests the move
    /// after, so only the wrap clears it); counted down, then
    /// `func_01296C` next.  Then `func_012A34`.
    pub fn r_012a62(&mut self, regs: &mut Regs) {
        let r = regs;
        let n = self.read8(COUNT_AD56);
        r.d0 = low_byte(r.d0, n);
        if n != 0 {
            let n = n.wrapping_add(1);
            r.d0 = low_byte(r.d0, n);
            self.write8(COUNT_AD56, n);
            if n == 0 {
                self.write8(COUNT_AD56, 0);
            }
        }
        if self.q_count_down(r) {
            self.write32(r.a4 + OBJ_ROUTINE, MOVING);
            r.c = false;
        }
        self.q_012a34(r);
        r.unwound = false;
    }

    /// `func_012A8C`: `OBJ_PHASE` counted up, the offset from `BOB` by
    /// its bits 2-4; d5 and a5 back to the sprite record (`SLOT_INDEX`,
    /// `SLOT_ADDR`).  Gives back d0 (the index, a word), d5, a5 and X
    /// (bit 1 of the phase).
    pub fn r_012a8c(&mut self, d0: u32, d5: u32, a4: u32) -> (u32, u32, u32, bool) {
        let phase = self.read8(a4 + OBJ_PHASE).wrapping_add(1);
        self.write8(a4 + OBJ_PHASE, phase);
        let index = (phase >> 2) & 7;
        let d0 = low_word(d0, u16::from(index));
        let offset = self.read8(BOB + u32::from(index));
        self.write8(a4 + OBJ_OFFSET, offset);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        (d0, d5, a5, phase & 2 != 0)
    }

    /// `func_012AB4`: `ACCEL` (signed, also kept as a word in
    /// `ACCEL_WORD`) added to `SPEED_965E`, which is kept within 0x600
    /// and, when going down (or when the sum is negative), 0x200 and
    /// up; then added to the position (`func_012AEE`).  Gives back d0
    /// and X.
    pub fn r_012ab4(&mut self, d0: u32) -> (u32, bool) {
        let accel = self.read8(ACCEL) as i8 as i16 as u16;
        self.write16(ACCEL_WORD, accel);
        let speed = accel.wrapping_add(self.read16(SPEED_965E));
        let mut d0 = low_word(d0, speed);
        if speed & 0x8000 != 0 {
            self.write16(SPEED_965E, 0x200);
        } else {
            let speed = speed.min(0x600);
            d0 = low_word(d0, speed);
            self.write16(SPEED_965E, speed);
            if self.read8(ACCEL) & 0x80 != 0 && speed < 0x200 {
                self.write16(SPEED_965E, 0x200);
            }
        }
        self.r_012aee(d0)
    }

    /// `func_012AEE`: `SPEED_965E` added to `POSITION_965C`.  Gives back
    /// d0 (its low word the speed) and X (the carry).
    pub fn r_012aee(&mut self, d0: u32) -> (u32, bool) {
        let speed = self.read16(SPEED_965E);
        let (position, carry) = self.read16(POSITION_965C).overflowing_add(speed);
        self.write16(POSITION_965C, position);
        (low_word(d0, speed), carry)
    }

    /// `func_012AF8`: while `DONE_9660` isn't 0, nothing.  Else the
    /// sprite's frame 0x15, `COUNT_AD57` 0, the task `func_013750`
    /// (`func_0003E8`, priority 0x10), d4 and a4 back to the object, the
    /// bits of `MODE`, `SPARED` set, a count of 0x20 (`func_012B48`,
    /// then `func_01338A`).  Gives back d0 (its low byte
    /// `DONE_9660`), d4, d6, a4, a6, X and C (clear).
    #[allow(clippy::too_many_arguments)]
    pub fn r_012af8(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, bool, bool) {
        let done = self.read8(DONE_9660);
        let d0 = low_byte(d0, done);
        if done != 0 {
            return (d0, d4, d6, a4, a6, x, false);
        }
        self.write8(a5 + SPRITE_FRAME, 0x15);
        self.write8(COUNT_AD57, 0);
        let (d6, a6, x, _) = self.r_0003e8(d4, d5, d6, 0x10, TASK_13750);
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        let m = self.read8(MODE);
        self.write8(MODE, m | MODE_BIT6);
        let s = self.read8(SPARED);
        self.write8(SPARED, s | SPARED_BIT0);
        self.write32(a4 + OBJ_SAVED, COUNTED);
        self.write8(a4 + OBJ_TIMER, 0x20);
        self.write32(a4 + OBJ_ROUTINE, COUNTING);
        (d0, d4, d6, a4, a6, x, false)
    }

    /// `func_012B48`: counted down, then the kept routine; then
    /// `func_0133CA`.
    pub fn r_012b48(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.q_count_down(r) {
            let kept = self.read32(r.a4 + OBJ_SAVED);
            self.write32(r.a4 + OBJ_ROUTINE, kept);
            r.c = false;
        }
        (r.d0, r.d1, r.d5, r.a5, r.x, r.c) = self.r_0133ca(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4);
        r.unwound = false;
    }

    /// `func_012B8E`: counted down; then, once `WAIT_9432` is 0,
    /// `func_0094D8` on `LIST_12B5A`, `RAM_CLEARED` bytes from
    /// `RAM_START` cleared, sound 0x1F and `func_012BD8` next.  The
    /// sprite's frame 0xC8, placed (`func_012BD0`).  `WAIT_9432` is
    /// cleared by an interrupt, so a call that waits can't run on its
    /// own (the check runs out of time there).
    pub fn r_012b8e(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.q_count_down(r) {
            while self.read8(WAIT_9432) != 0 {
                self.pass(22);
            }
            self.write32(LIST, LIST_12B5A);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
                self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
            if self.restarted.is_some() {
                return;
            }
            r.a0 = RAM_START;
            r.d0 = RAM_CLEARED;
            (r.d0, r.d1, r.a0, r.x) = self.r_0011d2(r.d0, r.d1, r.a0);
            r.d0 = low_byte(r.d0, 0x1F);
            self.r_0034b6(r.d0);
            self.write32(r.a4 + OBJ_ROUTINE, LATER);
        }
        self.write8(r.a5 + SPRITE_FRAME, 0xC8);
        r.c = false;
        (r.d0, r.d1, r.x, r.c) = self.r_012bd0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_012BD0`: placed (`func_001B32`, `func_001B5C`).  Gives back
    /// d0, d1, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_012bd0(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        let (d0, x, c) = self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5);
        (d0, d1, x, c)
    }

    /// `func_012BD8`: at the animation's step 0xA (its timer 0),
    /// `func_0137D6`, `POSITION` from the object it gives
    /// (`func_001542`), a task (`func_001222`: depth 1, priority 0x42,
    /// `func_012C7E`) at it, its x's low byte less 0x10 (`func_00152A`), flags
    /// 0x81, rate 0xFE00, offset 0xF0, a count of 0x40; the registers
    /// back to the object.  The animation a frame on; when it ends, the
    /// bits of `MODE`, `SPARED` cleared, its y's low byte 5, kind 0xA,
    /// `func_012CBC` next (the kept routine's first byte 0x80, `OBJ_26`
    /// 0xC, `OBJ_27` 2) and sound 0x13.  Else placed (`func_012BD0`).
    pub fn r_012bd8(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let timer = self.read8(r.a4 + OBJ_TIMER);
        r.d0 = low_byte(r.d0, timer);
        if timer == 0 {
            let step = self.read8(r.a4 + OBJ_STEP);
            r.d0 = low_byte(r.d0, step);
            if step == 0x0A {
                (r.d6, r.a4) = self.r_0137d6(r.d4, r.d5, r.d6);
                self.r_001542(r.a4);
                (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
                    self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 1, 0x42, LATER_TASK);
                let x_low = self.read8(POSITION_X_LOW);
                self.write8(POSITION_X_LOW, x_low.wrapping_sub(0x10));
                r.x = x_low < 0x10;
                self.r_00152a(r.a6);
                self.write8(r.a5 + SPRITE_FLAGS, 0x81);
                self.write16(r.a6 + TASK_RATE, 0xFE00);
                self.write8(r.a6 + TASK_OFFSET, 0xF0);
                self.write8(r.a6 + TASK_COUNT, 0x40);
                self.q_reload(r);
            }
        }
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if !r.c {
            (r.d0, r.d1, r.x, r.c) = self.r_012bd0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            return;
        }
        self.q_clear_mode();
        self.write8(r.a4 + OBJ_Y_LOW, 5);
        self.write8(r.a4 + OBJ_SAVED, 0x80);
        self.write8(r.a4 + OBJ_26, 0x0C);
        self.write8(r.a4 + OBJ_27, 2);
        self.write8(r.a4 + OBJ_KIND, 0x0A);
        self.write32(r.a4 + OBJ_ROUTINE, LATER_NEXT);
        r.d0 = low_byte(r.d0, 0x13);
        r.c = false;
        self.r_0034b6(r.d0);
    }

    /// `func_012C7E` (a task): the sprite's frame 0x66 (flags 0xC1 once
    /// the count is under 0x20); counted down, then it ends
    /// (`func_00121A`).  Until then moved by `VELOCITY_X` 0xFF00
    /// (`func_001A88`), `func_00C742` with 0x10, its offset by its rate
    /// (`func_001ACA`) and placed (`func_001B94`).
    pub fn r_012c7e(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write8(r.a5 + SPRITE_FRAME, 0x66);
        let count = self.read8(r.a6 + TASK_COUNT);
        r.d0 = low_byte(r.d0, count);
        if count < 0x20 {
            self.write8(r.a5 + SPRITE_FLAGS, 0xC1);
        }
        self.write8(r.a6 + TASK_COUNT, count.wrapping_sub(1));
        r.x = count == 0;
        r.c = r.x;
        if count == 1 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        } else {
            self.write16(VELOCITY_X, 0xFF00);
            (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
            r.d0 = 0x10;
            r.c = false;
            self.r_00c742(r.d0, r.a6);
            (r.d0, r.x, _) = self.r_001aca(r.d0, r.a6);
            (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
        }
        r.unwound = false;
    }
}
