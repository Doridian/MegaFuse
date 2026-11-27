//! Tasks whose sprite follows another's: `func_004FD0` makes one that
//! follows an object, `func_004FF4` one that follows a task (their index
//! in `FOLLOWED`).  Its sprite is put at the other's sprite, lower by
//! the height the other is at, its frame from that height; once the
//! other is down (no height, not rising) the task counts down and ends.
//! And `func_0054AE`, a routine of the cell kinds, that makes a task.

use crate::{Game, Regs};

/// The followed one's index (a task record's byte), and the count of
/// frames left once it is down.
const FOLLOWED: u32 = 0x13;
const COUNT: u32 = 0x12;
/// An object's: the high bytes of its rate and offset (signed; see
/// `a_motion.rs`), its sprite record's index.
const OBJ_USED: u32 = 0x00;
const OBJ_RATE: u32 = 0x28;
const OBJ_OFFSET: u32 = 0x2A;
const OBJ_SPRITE: u32 = 0x0E;
/// A task's: its flags (bit 5: it is followed), the high bytes of its
/// rate and offset, its sprite record's index.
const TASK_FLAGS: u32 = 0x18;
const TASK_FOLLOWED: u8 = 1 << 5;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;
const TASK_SPRITE: u32 = 0x03;
/// A sprite record's frame, flags and position (words, x then y).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_Y: u32 = 0x0A;

/// The frame for a height of 0x30 or more, and those by height / 16
/// below it (`D_0050EA`, 3 bytes in the ROM).
const FRAME_HIGH: u8 = 0xE2;
const FRAMES: u32 = 0x0050EA;
/// Taken from the height over a cell whose byte in the cell map is
/// below `LOW_CELL`.
const LOW_CELL: u8 = 4;
const LOW_CELL_LESS: u8 = 6;

/// The running task's index.
const CURRENT_TASK: u32 = 0xFF80EE;
/// `func_0054AE` makes nothing while this object index (a byte) is not
/// 0.
const CHOSEN: u32 = 0xFFA93F;
/// The task `func_0054AE` makes: its routine, priority and depth; a
/// long for its 0x20 (an address in the ROM), its sprite's flags; the
/// sound (`func_0034B6`) and what `func_00AC64` is given.
const CELL_TASK: u32 = 0x005506;
const CELL_TASK_PRIORITY: u8 = 0xC2;
const CELL_TASK_DEPTH: u8 = 0x10;
const TASK_20: u32 = 0x20;
const CELL_TASK_20: u32 = 0x0DA5E4;
const CELL_TASK_SPRITE_FLAGS: u8 = 0x84;
const CELL_TASK_SOUND: u8 = 0x45;
const CELL_TASK_AC64: u32 = 5;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_005038`, a task's routine: it follows object `FOLLOWED`
    /// (`func_005082`), and ends if the object is not in use.
    pub fn r_005038(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let byte = self.read8(r.a6 + FOLLOWED);
        let (d4, object, _) = self.r_001cfa(low_byte(r.d4, byte));
        r.d4 = d4;
        r.a4 = object;
        r.d1 = low_byte(r.d1, self.read8(object + OBJ_RATE));
        r.d2 = low_byte(r.d2, self.read8(object + OBJ_OFFSET));
        r.d3 = low_byte(r.d3, self.read8(object + OBJ_SPRITE));
        let (d0, x) = self.r_0015e6(object);
        r.d0 = d0;
        r.x = x;
        if self.read8(object + OBJ_USED) == 0 {
            (r.d6, r.x, r.c) = self.r_00509e(r.d5, r.d6, r.x);
        } else {
            (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) = self.r_005082(r.d0, r.d1, r.d2, r.d3, r.d6, r.a5);
        }
        r.unwound = false;
    }

    /// `func_00505C`, a task's routine: it follows task `FOLLOWED`
    /// (`func_005082`) while that one's `TASK_FOLLOWED` is set, else it
    /// ends.
    pub fn r_00505c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let byte = self.read8(r.a6 + FOLLOWED);
        let (d6, task, x) = self.r_0005f8(low_byte(r.d6, byte));
        r.d6 = d6;
        r.a6 = task;
        r.x = x;
        if self.read8(task + TASK_FLAGS) & TASK_FOLLOWED == 0 {
            (r.d6, r.x, r.c) = self.r_00509e(r.d5, r.d6, r.x);
        } else {
            r.d1 = low_byte(r.d1, self.read8(task + TASK_RATE));
            r.d2 = low_byte(r.d2, self.read8(task + TASK_OFFSET));
            r.d3 = low_byte(r.d3, self.read8(task + TASK_SPRITE));
            r.d0 = self.r_0015f8(task);
            (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) = self.r_005082(r.d0, r.d1, r.d2, r.d3, r.d6, r.a5);
        }
        r.unwound = false;
    }

    /// `func_005082` (`.L005082`): with the followed one's rate `d1`,
    /// height `d2` (low bytes) and sprite record `d3`, the running
    /// task's sprite (a5) put at that sprite, `d2` lower (6 more over a
    /// cell below `LOW_CELL`), its frame by the height.  When the other
    /// is down (`d2` 0, `d1` not negative) the running task counts down
    /// `COUNT` instead, and at 0 ends (`func_00509E`).  Gives back d0,
    /// d1, d6, a0 (the cell's address; 0 when the task ends), a6 (the
    /// sprite record followed), X and C.
    #[allow(clippy::type_complexity)]
    pub fn r_005082(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d6: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let height = d2 as u8;
        let mut d6 = d6;
        if (d1 as u8 as i8) >= 0 && height == 0 {
            let byte = self.read8(CURRENT_TASK);
            let (current, task, _) = self.r_0005f8(low_byte(d6, byte));
            let left = self.read8(task + COUNT).wrapping_sub(1);
            self.write8(task + COUNT, left);
            if left == 0 {
                // subq's X: no borrow from 1
                let (d6, x, c) = self.r_00509e(0, current, false);
                return (d0, d1, d6, 0, task, x, c);
            }
            d6 = current;
        }
        let mut lower = height.wrapping_neg();
        let (cell, _) = self.r_00167a();
        let byte = self.read8(cell);
        let mut d0 = low_byte(d0, byte);
        if byte < LOW_CELL {
            lower = lower.wrapping_sub(LOW_CELL_LESS);
        }
        let lower_word = lower as i8 as i16 as u16;
        let d1 = low_word(d1, lower_word);
        let (d6, sprite, _) = self.r_000b48(low_byte(d6, d3 as u8));
        let (y, mut x) = self.read16(sprite + SPRITE_Y).overflowing_add(lower_word);
        d0 = low_word(d0, y);
        self.write16(a5 + SPRITE_Y, y);
        let sprite_x = self.read16(sprite + SPRITE_X);
        self.write16(a5 + SPRITE_X, sprite_x);
        self.write8(a5 + SPRITE_FRAME, FRAME_HIGH);
        if lower < 0x30 {
            let step = lower >> 4;
            // the bit lsr #4 shifted out last
            x = lower & 0x08 != 0;
            d0 = low_word(d0, u16::from(step));
            let frame = self.read8(FRAMES + u32::from(step));
            self.write8(a5 + SPRITE_FRAME, frame);
        }
        (d0, d1, d6, cell, sprite, x, false)
    }

    /// `func_00509E` (`.L00509E`): the running task ends
    /// (`func_00121A`).  Gives back d6, X and C.
    pub fn r_00509e(&mut self, d5: u32, d6: u32, x: bool) -> (u32, bool, bool) {
        self.r_00121a(d5, d6, x)
    }

    /// `func_0054AE` (a routine of the cell kinds, for an object): unless
    /// `CHOSEN` is set, the cell's byte made again from its source
    /// (`func_00168C`), the cell cleared on the plane (`func_0052F0`)
    /// and a task made at it (`func_001222`: `func_005506`, its 0x20
    /// `CELL_TASK_20`), with a sound.  The registers it saves are kept.
    /// No script calls it: reviewed by reading.
    pub fn r_0054ae(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(CHOSEN) != 0 {
            r.c = false;
            return;
        }
        let (d0, cell) = self.r_00168c(r.d0);
        r.a0 = cell;
        self.r_001278();
        self.r_0052f0(d0);
        let (_, _, sprite, task, x, failed) = self.r_001222(d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, CELL_TASK_DEPTH, CELL_TASK_PRIORITY, CELL_TASK);
        if failed {
            (r.x, r.c) = (x, true);
            return;
        }
        self.r_001550(d0, task);
        self.write32(task + TASK_20, CELL_TASK_20);
        self.write8(sprite + SPRITE_FLAGS, CELL_TASK_SPRITE_FLAGS);
        let (d0, x) = self.r_00ac64(CELL_TASK_AC64);
        self.r_0034b6(low_byte(d0, CELL_TASK_SOUND));
        (r.x, r.c) = (x, false);
    }

    /// `func_005506`, the routine of `func_0054AE`'s task: `func_0013D6`,
    /// and its sprite placed (`func_001B7E`) until that says it is done
    /// (C), then the task ends (`func_00121A`).
    /// No script calls it: reviewed by reading.
    pub fn r_005506(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let done;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, done) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if done {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        } else {
            (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        }
        r.unwound = false;
    }
}
