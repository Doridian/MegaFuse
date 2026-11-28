//! An object that throws sparks about and walks to a cell,
//! 0x139DC-0x13CB0: `func_0139DC` waits `TIMER` frames with sparks
//! (`func_013B1C`), then walks (`func_013A04`) cell by cell toward
//! `TARGET` (`func_003F1A`, `func_003B28`) with sparks
//! (`func_013BDC`, `func_013C4E`); there the list `D_0139D4` is
//! loaded, and after its animation (`func_013AB0`, `func_013AD8`) it
//! makes the task `func_013CB0`.  The sparks are tasks of
//! `func_013BB2` (made by `func_013B56` at `POSITION`) and
//! `func_013BC8` (rising).
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read.

use crate::{Game, Regs};

/// An object's fields: its routine, its next direction and direction
/// (bits 0-1), its animation (a long), the animation's timer and step.
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_NEXT_DIRECTION: u32 = 0x13;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_ANIM: u32 = 0x20;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
/// A task's fields: its position (words), a count (0x11, counted down
/// by `func_013BB2`; 0x16), its animation (a long) or offset (byte
/// 0x20).
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_11: u32 = 0x11;
const TASK_COUNT: u32 = 0x16;
const TASK_ANIM: u32 = 0x20;
const TASK_OFFSET: u32 = 0x20;

/// Object 6's byte 0x26: the frames to wait, then (0x80) to walk with
/// sparks.
const TIMER: u32 = 0xFFA5D8;
/// Cleared when the walk starts.
const BYTE_AD58: u32 = 0xFFAD58;
/// The cell walked to (a word: column, row; `c_chase.rs`) and the
/// object's cell (`func_0015E6`).
const TARGET: u32 = 0xFFA966;
const TARGET_CELL: u16 = 0x0807;
const CELL: u32 = 0xFF9F08;
/// The way picked (`func_003F1A`'s direction).
const PICKED: u32 = 0xFFAC8B;
/// The speed `func_001620` makes a velocity of (a word).
const SPEED: u32 = 0xFF9F00;
const WALK_SPEED: u16 = 0x33;
/// A position (words; `a_objects.rs`): where the next spark is made.
const POSITION_X: u32 = 0xFF9F04;
const POSITION_X_LOW: u32 = 0xFF9F05;
const POSITION_Y: u32 = 0xFF9F06;
const POSITION_Y_LOW: u32 = 0xFF9F07;
/// The velocity (bytes: pixels and fraction, x then y; `a_motion.rs`).
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_X_FRACTION: u32 = 0xFF9EC5;
const VELOCITY_Y: u32 = 0xFF9EC6;
/// The rising sparks' velocity by `FRAME`'s bit 4 (D_013C4A x,
/// D_013C4B y).
const RISING_X: u32 = 0x013C4A;
const RISING_Y: u32 = 0x013C4B;
/// Sparks made by `func_013B56`; every other one the Z80 gets 0x5E.
const SPARKS: u32 = 0xFF9667;
/// The list `func_0094D8` loads (a long).
const LIST: u32 = 0xFF92FA;
const LIST_0139D4: u32 = 0x0139D4;
/// A byte of the ROM: the original takes 1 from it while `TIMER` is
/// set (`subq.b #1,(0x2C).l`), which does nothing.
const ROM_002C: u32 = 0x00002C;
/// The frame count.
const FRAME: u32 = 0xFF80D8;
/// The running object (index, address), the sprite record of the
/// routine being run (index, the one after, address), the task made
/// last.
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_8A59: u32 = 0xFF8A59;
const SLOT_ADDR: u32 = 0xFF8A5A;
const NEW_TASK: u32 = 0xFF80EF;

/// Routines and animations.
const ROUTINE_013A04: u32 = 0x013A04;
const ROUTINE_013AB0: u32 = 0x013AB0;
const ROUTINE_013ABC: u32 = 0x013ABC;
const ROUTINE_013AD8: u32 = 0x013AD8;
const ROUTINE_013BB2: u32 = 0x013BB2;
const ROUTINE_013BC8: u32 = 0x013BC8;
const ROUTINE_013CB0: u32 = 0x013CB0;
const ANIM_E877A: u32 = 0x0E877A;
const ANIM_SPARK: u32 = 0x0E860C;
const ANIM_SPARK_2: u32 = 0x0E861A;
const ANIM_RISING: u32 = 0x0E86FC;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// The running object and the routine's sprite record taken back.
    fn rr_take_back(&mut self, d4: u32, d5: u32) -> (u32, u32, u32, u32) {
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        (d4, d5, a4, a5)
    }

    /// `func_0139DC`, an object's routine: sparks (`func_013B1C`) until
    /// `TIMER` runs out; then it walks (`func_013A04`, its direction's
    /// bits 0-1 kept), `TIMER` 0x80 and `BYTE_AD58` cleared.
    pub fn r_0139dc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6) = self.r_013b1c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        let was = self.read8(TIMER);
        self.write8(TIMER, was.wrapping_sub(1));
        (r.x, r.c) = (was == 0, was == 0);
        if was != 1 {
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_013A04);
        let direction = self.read8(r.a4 + OBJ_DIRECTION) & 3;
        self.write8(r.a4 + OBJ_DIRECTION, direction);
        self.write8(TIMER, 0x80);
        self.write8(BYTE_AD58, 0);
        r.c = false;
    }

    /// `func_013A04`, an object's routine: sparks (`func_013BDC`,
    /// `func_013C4E`) and its animation (`func_001336`).  In the middle
    /// of a cell it picks its way toward `TARGET` (`func_003AEA`,
    /// `func_003F1A`, `func_003B28`, its animation by it,
    /// `func_00130E`); once on `TARGET` the list `D_0139D4` is loaded
    /// and it goes on as `func_013AB0` (animation `ANIM_E877A`, frame
    /// 0xA5).  Otherwise it walks (speed `WALK_SPEED`) and its sprite
    /// is placed.
    pub fn r_013a04(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x) = self.r_013bdc(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        (r.d0, r.d1, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x) = self.r_013c4e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        let mut middle;
        (r.d0, middle, r.c) = self.r_00182e(r.d0, r.a4);
        if middle {
            (r.d0, middle, r.c) = self.r_001840(r.d0, r.a4);
        }
        if middle {
            self.write16(TARGET, TARGET_CELL);
            self.r_003aea(r.a4);
            (r.d0, r.x) = self.r_0015e6(r.a4);
            self.r_003f1a(r);
            r.unwound = false;
            self.write8(PICKED, r.d0 as u8);
            (r.d0, _) = self.r_003b28(r.d1, r.a4, r.a5);
            self.write8(r.a4 + OBJ_NEXT_DIRECTION, r.d0 as u8);
            self.write8(r.a4 + OBJ_DIRECTION, r.d0 as u8);
            (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
            let cell = self.read16(CELL);
            r.d0 = low_word(r.d0, cell);
            if cell == self.read16(TARGET) {
                self.write32(LIST, LIST_0139D4);
                (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
                    self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
                if self.restarted.is_some() {
                    return;
                }
                self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_013AB0);
                self.write32(r.a4 + OBJ_ANIM, ANIM_E877A);
                self.write8(r.a4 + OBJ_STEP, 0);
                self.write8(r.a4 + OBJ_TIMER, 0);
                self.write8(r.a5 + SPRITE_FRAME, 0xA5);
                r.c = false;
                return self.r_013ab0(r);
            }
        }
        let timer = self.read8(TIMER);
        r.d0 = low_byte(r.d0, timer);
        if timer != 0 {
            let byte = self.read8(ROM_002C).wrapping_sub(1);
            self.write8(ROM_002C, byte);
        }
        self.write16(SPEED, WALK_SPEED);
        r.d0 = low_byte(r.d0, self.read8(r.a4 + OBJ_DIRECTION));
        self.r_001620(r.d0);
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
        (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_013A82`: nothing (an `rts`).
    /// No script calls it: reviewed by reading.
    pub fn r_013a82(&mut self) {}

    /// `func_013AB0`, an object's routine: sparks and its sprite placed
    /// (`loc_013ABC`) while `TIMER` runs out; then `func_013AD8`.
    pub fn r_013ab0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let timer = self.read8(TIMER);
        r.d0 = low_byte(r.d0, timer);
        if timer == 0 {
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_013AD8);
            r.c = false;
            return self.r_013ad8(r);
        }
        self.write8(TIMER, timer - 1);
        (r.x, r.c) = (false, false);
        self.r_013abc(r);
    }

    /// `loc_013ABC`, an object's routine: sparks (`func_013BDC`,
    /// `func_013C4E`) and its sprite placed.
    pub fn r_013abc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x) = self.r_013bdc(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        (r.d0, r.d1, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x) = self.r_013c4e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_013AD8`, an object's routine: its animation runs
    /// (`func_001336`, else `loc_013ABC`); at its end it goes on as
    /// `loc_013ABC` and makes a task of `func_013CB0` (depth 1,
    /// priority 0x42; not checked) at (0x120, 0xB8), count 0x20, flags
    /// 0x81, offset 0xE0, and `func_004FF4`.
    pub fn r_013ad8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if !r.c {
            return self.r_013abc(r);
        }
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_013ABC);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 1, 0x42, ROUTINE_013CB0);
        self.write16(r.a6 + TASK_X, 0x120);
        self.write16(r.a6 + TASK_Y, 0xB8);
        self.write8(r.a6 + TASK_COUNT, 0x20);
        self.write8(r.a5 + SPRITE_FLAGS, 0x81);
        self.write8(r.a6 + TASK_OFFSET, 0xE0);
        r.c = false;
        (r.d0, r.x, r.c) = self.r_004ff4(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_013B1C`: every 8th frame a spark (`func_013B56`) at the
    /// object's position moved by -0x10 + 0-0x1F in x and -0x20 +
    /// 0-0x1F in y (random, `func_001BE2`).  Gives back d0, d1, d4,
    /// d5, d6, a4, a5, a6.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_013b1c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        let turn = self.read8(FRAME) & 7;
        let d0 = low_byte(d0, turn);
        if turn != 0 {
            return (d0, d1, d4, d5, d6, a4, a5, a6);
        }
        self.r_001542(a4);
        let x = self.read8(POSITION_X_LOW).wrapping_sub(0x10);
        self.write8(POSITION_X_LOW, x);
        let (d0, d1, _) = self.r_001be2(d1);
        let d0 = low_byte(d0, d0 as u8 & 0x1F);
        self.write8(POSITION_X_LOW, x.wrapping_add(d0 as u8));
        let y = self.read8(POSITION_Y_LOW).wrapping_sub(0x20);
        self.write8(POSITION_Y_LOW, y);
        let (d0, d1, _) = self.r_001be2(d1);
        let d0 = low_byte(d0, d0 as u8 & 0x1F);
        self.write8(POSITION_Y_LOW, y.wrapping_add(d0 as u8));
        let (d0, d4, d5, d6, a4, a5, a6) = self.r_013b56(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        (d0, d1, d4, d5, d6, a4, a5, a6)
    }

    /// `func_013B56`: a spark, a task of `func_013BB2` (depth 1,
    /// priority 0xC2) at `POSITION` with the animation `ANIM_SPARK`,
    /// flags 0x81, placed; every other one (`SPARKS`) the Z80 gets
    /// 0x5E.  The running object and the routine's sprite record are
    /// taken back.  Gives back d0, d4, d5, d6, a4, a5, a6.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_013b56(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        let (d5, d6, a5, a6, _, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 1, 0xC2, ROUTINE_013BB2);
        let mut d0 = d0;
        if !failed {
            self.r_00152a(a6);
            self.write32(a6 + TASK_ANIM, ANIM_SPARK);
            self.write8(a5 + SPRITE_FLAGS, 0x81);
            (d0, _, _) = self.r_001b7e(a5, a6);
            let sparks = self.read8(SPARKS).wrapping_add(1);
            self.write8(SPARKS, sparks);
            d0 = low_byte(d0, sparks & 1);
            if sparks & 1 == 0 {
                d0 = low_byte(d0, 0x5E);
                self.r_0034b6(d0);
            }
        }
        let (d4, d5, a4, a5) = self.r_013ba0(d4, d5);
        (d0, d4, d5, d6, a4, a5, a6)
    }

    /// `.L013BA0`: the running object and the routine's sprite record
    /// taken back.  Gives back d4, d5, a4, a5.
    pub fn r_013ba0(&mut self, d4: u32, d5: u32) -> (u32, u32, u32, u32) {
        self.rr_take_back(d4, d5)
    }

    /// `func_013BB2`, a spark's routine: its byte 0x11 (its y's low
    /// byte) a pixel up, its animation run (`func_0013D6`) and its
    /// sprite placed; at the animation's end it ends (`func_013BC4`).
    pub fn r_013bb2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let y = self.read8(r.a6 + TASK_11).wrapping_sub(1);
        self.write8(r.a6 + TASK_11, y);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            (r.d6, r.x, r.c) = self.r_013bc4(r.d5, r.d6, r.x);
            return;
        }
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `.L013BC4`: the running task ends (`func_00121A`).  Gives back
    /// d6, X and C.
    pub fn r_013bc4(&mut self, d5: u32, d6: u32, x: bool) -> (u32, bool, bool) {
        self.r_00121a(d5, d6, x)
    }

    /// `func_013BC8`, a rising spark's routine: its animation run
    /// (`func_0013D6`), it rises 1.5 pixels a frame and its sprite is
    /// placed; at the animation's end it ends (`func_013BC4`).
    pub fn r_013bc8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            (r.d6, r.x, r.c) = self.r_013bc4(r.d5, r.d6, r.x);
            return;
        }
        self.write16(VELOCITY_Y, 0xFE80);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_013BDC`: every 16th frame a rising spark, a task of
    /// `func_013BC8` (depth 1, priority 0x42) at the object's position,
    /// moved a frame by the velocity `RISING_X`/`RISING_Y` picks by
    /// `FRAME`'s bit 4, on the sprite record after the routine's, with
    /// the animation `ANIM_RISING` and flags 0x81.  The running object
    /// and the routine's sprite record are taken back.  Gives back d0,
    /// d1, d4, d5, d6, a4, a5, a6 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_013bdc(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let turn = self.read8(FRAME) & 0xF;
        let d0 = low_byte(d0, turn);
        if turn != 0 {
            return (d0, d1, d4, d5, d6, a4, a5, a6, x);
        }
        self.r_001542(a4);
        let (d5, d6, _, a6, x, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 1, 0x42, ROUTINE_013BC8);
        if failed {
            let (d4, d5, a4, a5) = self.r_013ba0(d4, d5);
            return (d0, d1, d4, d5, d6, a4, a5, a6, x);
        }
        self.r_00152a(a6);
        let pick = (self.read8(FRAME) & 0x10) >> 3;
        let d0 = low_word(d0, u16::from(pick));
        self.write8(VELOCITY_X_FRACTION, 0);
        let vx = self.read8(RISING_X + u32::from(pick));
        self.write8(VELOCITY_X, vx);
        let vy = self.read8(RISING_Y + u32::from(pick));
        self.write8(VELOCITY_Y, vy);
        let (d0, d1, _, _, _) = self.r_001a88(d0, d1, a6);
        let (d0, d1, _, _, _) = self.r_001aa2(d0, d1, a6);
        let other = self.read8(SLOT_8A59);
        let (d5, a5, x) = self.r_000b5a(low_byte(d5, other));
        self.write32(a6 + TASK_ANIM, ANIM_RISING);
        self.write8(a5 + SPRITE_FLAGS, 0x81);
        let (d4, d5, a4, a5) = self.r_013ba0(d4, d5);
        (d0, d1, d4, d5, d6, a4, a5, a6, x)
    }

    /// `func_013C4E`: every 64th frame `func_0043A8` (0x30 frames);
    /// every 4th frame a spark (`func_013B56`) at a random position
    /// (bytes 0-0xFF), which half the time (a random number's sign)
    /// takes the animation `ANIM_SPARK_2` (with the running object
    /// taken back).  Gives back d0, d1, d4, d5, d6, a4, a5, a6 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_013c4e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let (mut d0, mut x) = (low_byte(d0, self.read8(FRAME) & 0x3F), x);
        if d0 as u8 == 0 {
            (d0, x) = self.r_0043a8(0x30, d4, d5, d6);
        }
        let turn = self.read8(FRAME) & 3;
        d0 = low_byte(d0, turn);
        if turn != 0 {
            return (d0, d1, d4, d5, d6, a4, a5, a6, x);
        }
        self.write8(POSITION_X, 0);
        self.write8(POSITION_Y, 0);
        let (d0, d1, _) = self.r_001be2(d1);
        self.write8(POSITION_X_LOW, d0 as u8);
        let (d0, d1, _) = self.r_001be2(d1);
        self.write8(POSITION_Y_LOW, d0 as u8);
        let (_, mut d4, d5, mut d6, mut a4, a5, mut a6) = self.r_013b56(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        let (d0, d1, mut x) = self.r_001be2(d1);
        if (d0 as u8 as i8) < 0 {
            let new = self.read8(NEW_TASK);
            (d6, a6, x) = self.r_0005f8(low_byte(d6, new));
            self.write32(a6 + TASK_ANIM, ANIM_SPARK_2);
            d4 = low_byte(d4, self.read8(RUNNING));
            a4 = self.read32(RUNNING_ADDR);
        }
        (d0, d1, d4, d5, d6, a4, a5, a6, x)
    }
}
