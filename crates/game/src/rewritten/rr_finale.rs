//! 0x13CB0-0x13F14: a task that comes in from the right
//! (`func_013CB0`, `func_013D00`) and makes `func_013E82`, which runs
//! object 5 (or 0, when object 0's byte 0x32 is negative) as the
//! running object (`func_013EAE`).  Then (`func_013D20`,
//! `func_013D46`) it holds objects 0 and 5 at its position while it
//! falls, gives them their routines `func_01237C`/`func_0124A8`
//! (`func_013ECE`, `func_013EF0`) and, at its count's end
//! (`func_013DB0`), leaves to the left; a fade to white
//! (`func_013E02`), every task ended (`func_013E1C`), and after two
//! counts of 0x40 frames (`func_013E3E`, `func_013E6A`) `func_0179B0`
//! with 6.
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read.

use crate::{Game, Regs};

/// An object's fields: its routine (a long), its sprite record's
/// address (a long) and index, its rate (a word, `a_motion.rs`).
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_RATE: u32 = 0x28;
/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
/// A task's fields: its x (a word) and y's low byte, a count, the
/// object it runs (`func_013E82`), its rate (a word) and offset (the
/// high byte of a word at 0x20; `a_motion.rs`).
const TASK_X: u32 = 0x0E;
const TASK_Y_LOW: u32 = 0x11;
const TASK_COUNT: u32 = 0x16;
const TASK_OBJECT: u32 = 0x17;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;

/// Object 0's byte 0x32: negative, object 0 is run instead of 5, it
/// doesn't fall, and the Z80 doesn't get 0x3C.
const OBJECT_0_32: u32 = 0xFFA464;
/// Objects 0's and 5's routines (longs) and offsets (`OBJ_OFFSET`'s
/// high bytes), set from the task's.
const OBJECT_0_ROUTINE: u32 = 0xFFA434;
const OBJECT_5_ROUTINE: u32 = 0xFFA574;
const OBJECT_0_OFFSET: u32 = 0xFFA45C;
const OBJECT_5_OFFSET: u32 = 0xFFA59C;
const OBJECT_0: u8 = 0;
const OBJECT_5: u8 = 5;
/// Counts the steps of the finale: `func_013E82` ends once it is set,
/// `func_013ECE` goes on at 2.
const STEP: u32 = 0xFFAD58;
/// Set to 4 while the task comes in, cleared when it holds.
const BYTE_954F: u32 = 0xFF954F;
/// Cleared when the task comes in, stepped when it holds; while it is
/// 0 the run object falls (`func_013E82`).
const COUNT_AD57: u32 = 0xFFAD57;
/// Bit 3 set at the fade (`func_00A52E`).
const FLAGS: u32 = 0xFF9540;
/// What `func_0179B0` goes on to (6 here).
const NEXT_953B: u32 = 0xFF953B;
/// The velocity (words, x then y; `a_motion.rs`).
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;
/// The frame count.
const FRAME: u32 = 0xFF80D8;
/// The running task's index; the running object (index, address) and
/// the sprite record of the routine being run (index, address).
const CURRENT_TASK: u32 = 0xFF80EE;
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// A byte of the ROM: the original clears it (`clr.b (0x0).l`), which
/// does nothing.
const ROM_0000: u32 = 0x000000;

/// Routines.
const ROUTINE_013D20: u32 = 0x013D20;
const ROUTINE_013D46: u32 = 0x013D46;
const ROUTINE_013E02: u32 = 0x013E02;
const ROUTINE_013E1C: u32 = 0x013E1C;
const ROUTINE_013E3E: u32 = 0x013E3E;
const ROUTINE_013E6A: u32 = 0x013E6A;
const ROUTINE_013E82: u32 = 0x013E82;
const ROUTINE_013ECE: u32 = 0x013ECE;
const ROUTINE_013EF0: u32 = 0x013EF0;
const ROUTINE_01237C: u32 = 0x01237C;
const ROUTINE_0124A8: u32 = 0x0124A8;
const PRIORITY: u8 = 0x42;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// A byte at `at` less 1; gives back the byte it was.
    fn rr_count_down(&mut self, at: u32) -> u8 {
        let was = self.read8(at);
        self.write8(at, was.wrapping_sub(1));
        was
    }

    /// `func_013CB0`, a task's routine: it moves left (`func_013D00`)
    /// for its count; then (count 0x30) it goes on as `func_013D20`
    /// and makes the task `func_013E82` (priority 0x42) for object 5
    /// (0 when `OBJECT_0_32` is negative), whose rate is 0xFC40;
    /// `BYTE_954F` 4, `COUNT_AD57` cleared.
    pub fn r_013cb0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let was = self.rr_count_down(r.a6 + TASK_COUNT);
        (r.x, r.c) = (was == 0, was == 0);
        if was != 1 {
            (r.d0, r.d1, r.x, r.c) = self.r_013d00(r.d0, r.d1, r.a5, r.a6);
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x30);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_013D20);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, PRIORITY, ROUTINE_013E82);
        r.d4 = if (self.read8(OBJECT_0_32) as i8) < 0 { low_byte(r.d4, OBJECT_0) } else { u32::from(OBJECT_5) };
        self.write8(r.a6 + TASK_OBJECT, r.d4 as u8);
        (r.d4, r.a4, r.x) = self.r_001cfa(r.d4);
        self.write16(r.a4 + OBJ_RATE, 0xFC40);
        self.write8(BYTE_954F, 4);
        self.write8(COUNT_AD57, 0);
        r.c = false;
    }

    /// `.L013D00`: the task moves 2 pixels left (`func_001A88`) with
    /// frame 0xD5 or 0xD6 (`FRAME`'s bit 1), its sprite placed with
    /// its offset (`func_001B94`).  Gives back d0, d1, X and C.
    pub fn r_013d00(&mut self, d0: u32, d1: u32, a5: u32, a6: u32) -> (u32, u32, bool, bool) {
        let frame = ((self.read8(FRAME) & 2) >> 1).wrapping_add(0xD5);
        self.write8(a5 + SPRITE_FRAME, frame);
        self.write16(VELOCITY_X, 0xFE00);
        let (_, d1, _, _, _) = self.r_001a88(low_byte(d0, frame), d1, a6);
        let (d0, x, c) = self.r_001b94(a5, a6);
        (d0, d1, x, c)
    }

    /// `func_013D20`, a task's routine: `BYTE_954F` cleared; it moves
    /// left (`func_013D00`) for its count; then `COUNT_AD57` goes up,
    /// its rate 0, count 0xA0, and it goes on as `func_013D46`.
    pub fn r_013d20(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(BYTE_954F, 0);
        let was = self.rr_count_down(r.a6 + TASK_COUNT);
        (r.x, r.c) = (was == 0, was == 0);
        if was != 1 {
            (r.d0, r.d1, r.x, r.c) = self.r_013d00(r.d0, r.d1, r.a5, r.a6);
            return;
        }
        let count = self.read8(COUNT_AD57).wrapping_add(1);
        self.write8(COUNT_AD57, count);
        self.write16(r.a6 + TASK_RATE, 0);
        self.write8(r.a6 + TASK_COUNT, 0xA0);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_013D46);
        self.r_013d46(r);
    }

    /// `func_013D46`, a task's routine: objects 0 and 5 put at its
    /// position (`func_001300`); unless `OBJECT_0_32` is negative it
    /// falls (`func_00C742`, 0x10) and once down (offset 0) `STEP`
    /// goes up, it goes on as `func_013ECE` and objects 0 and 5 get
    /// the routines `func_01237C` and `func_0124A8`.  Object 5's offset
    /// is its, then `func_013DB0`.
    pub fn r_013d46(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.r_00153a(r.a6);
        (r.d4, r.a4, r.x) = self.r_001cfa(low_byte(r.d4, OBJECT_0));
        self.r_001300(r.a4);
        (r.d4, r.a4, r.x) = self.r_001cfa(u32::from(OBJECT_5));
        self.r_001300(r.a4);
        let byte = self.read8(OBJECT_0_32);
        r.d0 = low_byte(r.d0, byte);
        if (byte as i8) >= 0 {
            let current = self.read8(CURRENT_TASK);
            (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
            r.d0 = 0x10;
            self.r_00c742(r.d0, r.a6);
            let up;
            (r.d0, r.x, up) = self.r_001aca(r.d0, r.a6);
            if !up {
                self.write8(r.a6 + TASK_OFFSET, 0);
                let step = self.read8(STEP).wrapping_add(1);
                self.write8(STEP, step);
                (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_013ECE);
                self.write32(OBJECT_0_ROUTINE, ROUTINE_01237C);
                self.write32(OBJECT_5_ROUTINE, ROUTINE_0124A8);
            }
        }
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
        let offset = self.read8(r.a6 + TASK_OFFSET);
        self.write8(OBJECT_5_OFFSET, offset);
        r.c = false;
        (r.d0, r.d1, r.d6, r.a0, r.a1, r.a6, r.x, r.c) = self.r_013db0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `.L013DB0`: object 0's offset is the task's; it moves left
    /// (`func_013D00`) for its count; then `STEP` goes up and it goes
    /// on as `func_013E02`: flags 1, y's low byte 0xF0, x -0x20, offset
    /// and rate 0, count 0xA0, depth 0xFC (`func_0007D2`).  Gives back
    /// d0, d1, d6, a0, a1, a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_013db0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        let offset = self.read8(a6 + TASK_OFFSET);
        self.write8(OBJECT_0_OFFSET, offset);
        if self.rr_count_down(a6 + TASK_COUNT) != 1 {
            let (d0, d1, x, c) = self.r_013d00(d0, d1, a5, a6);
            return (d0, d1, d6, a0, a1, a6, x, c);
        }
        self.write8(ROM_0000, 0);
        let step = self.read8(STEP).wrapping_add(1);
        self.write8(STEP, step);
        let (x, _) = self.r_00053e(d6, ROUTINE_013E02);
        self.write8(a5 + SPRITE_FLAGS, 1);
        self.write8(a6 + TASK_Y_LOW, 0xF0);
        self.write16(a6 + TASK_X, 0xFFE0);
        self.write8(a6 + TASK_OFFSET, 0);
        self.write8(a6 + TASK_COUNT, 0xA0);
        self.write16(a6 + TASK_RATE, 0);
        let d0 = low_byte(d0, 0xFC);
        let c = self.r_0007d2(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, x);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c, ..Regs::default() };
        self.r_013e02(&mut r);
        (r.d0, r.d1, r.d6, r.a0, r.a1, r.a6, r.x, r.c)
    }

    /// `func_013E02`, a task's routine: on as `func_013E1C`, bit 3 of
    /// `FLAGS` set, and a fade to white at speed 8 (`func_00A52E`).
    pub fn r_013e02(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_013E1C);
        let flags = self.read8(FLAGS);
        self.write8(FLAGS, flags | 8);
        r.d0 = 8;
        r.c = false;
        (r.d0, r.d6, r.a0, r.a1, r.a6, r.x, r.c) = self.r_00a52e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `func_013E1C`, a task's routine (unless the objects are stopped,
    /// `func_001BB4`): every other task ended (`func_0003DA`), the
    /// objects cleared and their task made again (`func_001C56`), and
    /// a task of `func_013E3E` (priority 0x42, count 0x40) made and
    /// run.
    pub fn r_013e1c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d6, r.a6, r.x) = self.r_001c56(r.d4, r.d5, r.d6);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, PRIORITY, ROUTINE_013E3E);
        self.write8(r.a6 + TASK_COUNT, 0x40);
        r.c = false;
        self.r_013e3e(r);
    }

    /// `func_013E3E`, a task's routine: when its count runs out the
    /// Z80 gets 0x3C (unless `OBJECT_0_32` is negative) and it goes on
    /// as `func_013E6A` (count 0x40).
    pub fn r_013e3e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let was = self.rr_count_down(r.a6 + TASK_COUNT);
        (r.x, r.c) = (was == 0, was == 0);
        if was != 1 {
            return;
        }
        if (self.read8(OBJECT_0_32) as i8) >= 0 {
            r.d0 = low_byte(r.d0, 0x3C);
            self.r_0034b6(r.d0);
        }
        self.write8(r.a6 + TASK_COUNT, 0x40);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_013E6A);
        self.r_013e6a(r);
    }

    /// `func_013E6A`, a task's routine: when its count runs out,
    /// `NEXT_953B` 6 and `func_0179B0`.
    pub fn r_013e6a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let was = self.rr_count_down(r.a6 + TASK_COUNT);
        (r.x, r.c) = (was == 0, was == 0);
        if was != 1 {
            return;
        }
        self.write8(NEXT_953B, 6);
        (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) = self.r_0179b0(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_013E82`, a task's routine: once `STEP` is set it ends
    /// (`func_000560`); until then its object (`TASK_OBJECT`) falls
    /// while `COUNT_AD57` is 0 (`func_001AF6`, `func_001ABC`) and is
    /// run (`func_013EAE`).
    pub fn r_013e82(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(STEP) != 0 {
            r.c = false;
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
            return;
        }
        let object = self.read8(r.a6 + TASK_OBJECT);
        (r.d4, r.a4, r.x) = self.r_001cfa(low_byte(r.d4, object));
        if self.read8(COUNT_AD57) == 0 {
            r.d0 = self.r_001af6(r.a4);
            (r.d0, r.x, _) = self.r_001abc(r.d0, r.a4);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_013eae(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a6, r.x);
    }

    /// `func_013EAE`: the object at a4 (index d4) run: made the running
    /// object, its sprite record the routine's, and its routine called
    /// (family 4).  Gives back every register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_013eae(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let d5 = low_byte(d5, self.read8(a4 + OBJ_SPRITE));
        let a5 = self.read32(a4 + OBJ_SPRITE_ADDR);
        self.write8(SLOT_INDEX, d5 as u8);
        self.write32(SLOT_ADDR, a5);
        self.write8(RUNNING, d4 as u8);
        self.write32(RUNNING_ADDR, a4);
        let a0 = self.read32(a4 + OBJ_ROUTINE);
        // the flags of move.l a4,(RUNNING_ADDR).w
        let (n, z) = (a4 & 0x8000_0000 != 0, a4 == 0);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, n, z, ..Regs::default() };
        self.call_family_4(a0, &mut r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_013ECE`, a task's routine: its sprite placed with its
    /// offset (`func_001B94`) until `STEP` is 2; then it goes on as
    /// `func_013EF0` (count 0x40).
    /// No script calls it: reviewed by reading.
    pub fn r_013ece(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(STEP) != 2 {
            (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_013EF0);
        self.write8(r.a6 + TASK_COUNT, 0x40);
        r.c = false;
        self.r_013ef0(r);
    }

    /// `func_013EF0`, a task's routine: it rises a pixel a frame
    /// (`func_001AA2`) with object 0 put at its position
    /// (`func_001300`), then `func_013DB0`.
    /// No script calls it: reviewed by reading.
    pub fn r_013ef0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write16(VELOCITY_Y, 0xFF00);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        self.r_00153a(r.a6);
        (r.d4, r.a4, r.x) = self.r_001cfa(low_byte(r.d4, OBJECT_0));
        self.r_001300(r.a4);
        let current = self.read8(CURRENT_TASK);
        r.c = false;
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
        (r.d0, r.d1, r.d6, r.a0, r.a1, r.a6, r.x, r.c) = self.r_013db0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }
}
