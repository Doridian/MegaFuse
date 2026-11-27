//! A task that goes from cell to cell in one of four directions
//! (`func_00D07C`, `func_00D0A8`; stage-3-3 runs it): each 8 frames at a
//! cell it picks the next direction whose cell beyond holds a value
//! from 1 to 0x2F (`func_00D390`), its sprite's frame follows the
//! direction it turns to (`func_00D314`), and it moves the objects 0
//! (and 5, when its `TASK_OBJECT` is not 0) with it (`func_00D418`).
//! An object near it (`func_00D57C`) has its `OBJ_11` cleared; on a
//! cell holding 0x20 above it leaves for `func_00D28C`, and with no way
//! on it gives its object `TASK_DIRECTION` and the routines
//! `func_004E5C`/`func_001F7E` (or `func_004D7E`/`loc_0045E2`), and goes
//! on to `func_00CFF8`.  Bit 2 of `FLAGS` stops it (`func_00D554`).

use crate::{Game, Regs};

/// Flags: bit 2 stops the task's routines (`func_00D554`).
const FLAGS: u32 = 0xFF9540;
const STOP_BIT: u8 = 1 << 2;
/// The task being run (its index, a byte; `func_0005F8`).
const CURRENT_TASK: u32 = 0xFF80EE;
/// The frame count: bit 1 turns the sprite (`func_00D314`), bit 2 is
/// or'ed into the sprite's y (`OR_Y`).
const FRAME: u32 = 0xFF80D8;
/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// A velocity (words, x then y; `a_motion.rs`), set from the
/// direction's bytes (`STEP_X`, `STEP_Y`) as their high bytes.
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;
/// The cell looked at (column, row; a word together) and a position
/// (words, x then y) the objects are put at (`func_00D4DA`).
const CELL: u32 = 0xFF9F08;
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
/// The task's position (words) as `func_00D57C` compares the objects'
/// with it.
const NEAR_X: u32 = 0xFF9EFA;
const NEAR_Y: u32 = 0xFF9EFE;

/// `func_00D390`: the direction not to take (by `TASK_DIRECTION`, from
/// `NOT_BY_DIRECTION`; 0xFF for none), and the one looked at.
const NOT_DIRECTION: u32 = 0xFFAC8A;
const NOT_BY_DIRECTION: u32 = 0x00D38A;
const LOOKED_AT: u32 = 0xFFAC8B;
/// A cell's step by direction (columns, rows: bytes).
const STEP_X: u32 = 0x00572A;
const STEP_Y: u32 = 0x00572E;

/// What `func_00D418` hands `func_00D4DA`: the facing, `TASK_20`, the
/// sprite's depth byte, the direction, a frame base (`BASE`) and a bit
/// for the sprite's y (`OR_Y`).
const FACING: u32 = 0xFFAABD;
const BYTE_20: u32 = 0xFFAC8C;
const DEPTH: u32 = 0xFFAC8B;
const DIRECTION: u32 = 0xFF80C5;
const BASE: u32 = 0xFFAC8A;
const OR_Y: u32 = 0xFFAC8D;
/// The frame bases of object 0's and the other object's sprites.
const BASE_0: u8 = 0x47;
const BASE_OTHER: u8 = 0x31;
/// The object routines are run for (index, address), the object
/// worked on, and two bytes `func_00D418` sets for `func_005776`.
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const OBJECT: u32 = 0xFFA93F;
const BYTE_A932: u32 = 0xFFA932;
const BYTE_A933: u32 = 0xFFA933;
/// Records of 8 bytes; bit 6 of the first one's fourth byte goes to
/// `BYTE_A933`.
const RECORDS_AD0A: u32 = 0xFFFF_AD0A;
const RECORD_3: u32 = 3;
/// Bytes set when the object is left (`func_00D0A8`, the other object).
const BYTE_A446: u32 = 0xFFA446;
const BYTE_A447: u32 = 0xFFA447;

/// Sprite frames by facing (`func_00D314`) and by direction (`func_00D0A8`).
const FRAME_BY_FACING: u32 = 0x00D384;
const FRAME_BY_DIRECTION: u32 = 0x00D18A;
/// The velocity's high bytes by direction (`func_00D2DE`).
const VELOCITY_X_BY_DIRECTION: u32 = 0x00D30C;
const VELOCITY_Y_BY_DIRECTION: u32 = 0x00D310;

/// The task's routines, and the routines it gives the object.
const MOVING_ROUTINE: u32 = 0x00D0A8;
const RISING_ROUTINE: u32 = 0x00D28C;
const LEFT_ROUTINE: u32 = 0x00CFF8;
const OBJ_ROUTINE_0: u32 = 0x004E5C;
const OBJ_KEPT_0: u32 = 0x001F7E;
const OBJ_ROUTINE_OTHER: u32 = 0x004D7E;
const OBJ_KEPT_OTHER: u32 = 0x0045E2;
/// The animations of the tasks `func_00D5E0` makes.
const CELL_2_ANIM: u32 = 0x0DA504;
const EIGHTH_ANIM: u32 = 0x0DA4FC;
/// The routine of the tasks `func_00D5E0` makes, their depth and
/// priority.
const MADE_ROUTINE: u32 = 0x00D61C;
const MADE_DEPTH: u8 = 0x40;
const MADE_PRIORITY: u8 = 0xC2;

/// The task's fields: its position (words), direction (bits 0-1; bit 3
/// while rising), a step count, a frame count, its facing (0-7), the
/// object it carries (a byte; 0 for object 0 alone), its y's rate (a
/// word), a byte handed to the objects (`BYTE_20`), its animation.
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_DIRECTION: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_FACING: u32 = 0x17;
const TASK_OBJECT: u32 = 0x19;
const TASK_RATE: u32 = 0x1A;
const TASK_20: u32 = 0x20;
const TASK_ANIM: u32 = 0x20;
const RISING: u8 = 1 << 3;

/// An object's fields: in use, its routine and a kept one, its sprite
/// (address, index), flags, a byte cleared when near the task, two
/// bytes set to the direction, its kind, position (words), animation's
/// timer and step, two words and a byte `func_00D0A8` sets.
const OBJ_USED: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_KEPT: u32 = 0x06;
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_FLAGS: u32 = 0x10;
const OBJ_11: u32 = 0x11;
const OBJ_13: u32 = 0x13;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x15;
const OBJ_X: u32 = 0x18;
const OBJ_Y: u32 = 0x1A;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_26: u32 = 0x26;
const OBJ_RATE: u32 = 0x28;
const OBJ_2A: u32 = 0x2A;

/// A sprite record's frame, flags (bit 4: changed) and the low byte of
/// its y.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_Y_LOW: u32 = 0x0B;
const SPRITE_DEPTH: u32 = 0x02;

/// Cells' second plane (0x200 after a cell, `func_00167A`).
const PLANE_2: u32 = 0x200;
/// The cell value above which the task rises.
const RISE_ON: u8 = 0x20;
/// The objects the task looks at near it, from `NEAR_FIRST` to
/// `NEAR_END`, and how near (both ways).
const NEAR_FIRST: u8 = 6;
const NEAR_END: u8 = 0x14;
const NEAR: u16 = 8;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// A byte's index sign-extended (`ext.w`, then `(table,dn.w)`).
fn index(b: u8) -> u32 {
    b as i8 as i32 as u32
}

/// A word's distance (`sub.w`, `neg.w` when negative).
fn distance(a: u16, b: u16) -> u16 {
    let d = a.wrapping_sub(b);
    if (d as i16) < 0 { d.wrapping_neg() } else { d }
}

impl Game {
    fn m_add8(&mut self, at: u32, by: u8) -> u8 {
        let value = self.read8(at).wrapping_add(by);
        self.write8(at, value);
        value
    }

    fn m_or8(&mut self, at: u32, bits: u8) {
        let value = self.read8(at) | bits;
        self.write8(at, value);
    }

    /// `func_00D07C`, a task's routine: its count down; at 0 (count 1
    /// again, step 0) it goes on as `func_00D0A8` from now, its
    /// direction picked (`func_00D390`) and its facing made from it,
    /// and runs it.  Else on to `func_00D074` (`func_00CFF8`'s end).
    pub fn r_00d07c(&mut self, regs: &mut Regs) {
        let a6 = regs.a6;
        if self.m_add8(a6 + TASK_COUNT, 0xFF) != 0 {
            let r = &mut *regs;
            (r.d0, r.x, r.c) = self.r_00d074(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, a6);
            r.unwound = false;
            return;
        }
        self.write8(a6 + TASK_COUNT, 1);
        self.write8(a6 + TASK_STEP, 0);
        self.r_00053e(regs.d6, MOVING_ROUTINE);
        (regs.d0, regs.d1, regs.a0, _) = self.r_00d390(regs.d1, regs.a0, a6);
        let direction = self.read8(a6 + TASK_DIRECTION);
        let (facing, x) = direction.overflowing_add(direction);
        regs.d0 = low_byte(regs.d0, facing);
        self.write8(a6 + TASK_FACING, facing);
        regs.x = x;
        regs.c = false;
        self.r_00d0a8(regs);
    }

    /// `func_00D0A8`, a task's routine: an object near (`func_00D57C`)
    /// gets `OBJ_11` cleared; every 8 frames, at the cell's end, a new
    /// direction (`func_00D390`): with none, the task rises off a cell
    /// holding `RISE_ON` above (`func_00D28C`) or leaves the object
    /// (see the module).  Else it moves on (`func_00D2DE`): a cell
    /// holding 1 is handled by `func_005E0E`, one holding 2 is sent the
    /// Z80 0x60, cleared, and given a task (`CELL_2_ANIM`) before
    /// `func_0066FE`; its sprite turned and drawn, the objects moved
    /// with it (`func_00D418`), and its step counted: the Z80 0x4E each
    /// 4, a task each 8 (`func_00D564`).
    pub fn r_00d0a8(&mut self, regs: &mut Regs) {
        let unwound;
        (regs.d0, unwound) = self.r_00d554(regs.d0, 0xD0AE);
        regs.unwound = false;
        if unwound {
            return;
        }
        let (d4, a4, near) = self.r_00d57c(regs.d4, regs.a6);
        (regs.d4, regs.a4) = (d4, a4);
        if near {
            self.write8(a4 + OBJ_11, 0);
        }
        regs.d6 = low_byte(regs.d6, self.read8(CURRENT_TASK));
        (regs.d6, regs.a6, regs.x) = self.r_0005f8(regs.d6);
        let a6 = regs.a6;
        if self.m_add8(a6 + TASK_COUNT, 0xFF) == 0 {
            self.write8(a6 + TASK_COUNT, 8);
            let stuck;
            (regs.d0, regs.d1, regs.a0, stuck) = self.r_00d390(regs.d1, regs.a0, a6);
            if stuck {
                self.m_stuck(regs);
                return;
            }
        }
        (regs.d1, regs.d5, regs.a5) = self.r_00d2de(regs.d0, regs.d1, regs.d5, a6);
        regs.d0 = self.r_0015f8(a6);
        (regs.a0, _) = self.r_00167a();
        let cell = self.read8(regs.a0);
        regs.d0 = low_byte(regs.d0, cell);
        if cell == 1 {
            self.r_005e0e(regs.d2, regs.d6, a6);
            regs.d5 = low_byte(regs.d5, self.read8(SLOT_INDEX));
            regs.a5 = self.read32(SLOT_ADDR);
        } else {
            if cell == 2 {
                let r = &mut *regs;
                r.d0 = low_byte(r.d0, 0x60);
                self.r_0034b6(r.d0);
                (r.d0, _) = self.r_00168c(r.d0);
                self.r_001278();
                self.r_0052f0(r.d0);
                (r.d0, r.a0, _, _) = self.r_00d5e0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, CELL_2_ANIM, r.a1, r.a2, r.a3, r.a4, r.a5);
            }
            let r = &mut *regs;
            (r.d2, _) = self.r_0066fe(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
        (regs.d1, regs.d5, regs.a5) = self.r_00d314(regs.d1, regs.d5, regs.a5, a6);
        (regs.d0, _, _) = self.r_001b7e(regs.a5, a6);
        let or_y = (self.read8(FRAME) >> 2) & 1;
        regs.d0 = low_byte(regs.d0, or_y);
        self.write8(OR_Y, or_y);
        self.m_or8(regs.a5 + SPRITE_Y_LOW, or_y);
        let r = &mut *regs;
        (r.d0, _, _) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        (r.d0, r.d1, r.d4, r.d5, r.d6, r.a0, r.a1, r.a4, r.a5, r.a6, _, _) =
            self.r_00d418(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5, r.a6);
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, _) = self.r_0005f8(r.d6);
        let a6 = r.a6;
        let step = self.m_add8(a6 + TASK_STEP, 1);
        r.x = step == 0;
        if step & 3 == 0 {
            self.r_0034b6(low_byte(r.d0, 0x4E));
        }
        r.d0 = low_byte(r.d0, step & 7);
        r.c = false;
        if step & 7 == 0 {
            (r.d0, r.d6, r.a0, r.a6, r.x, r.c) =
                self.r_00d564(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
    }

    /// `func_00D0A8` with no way on: rising off a cell holding
    /// `RISE_ON` above, else the task's object left with the
    /// direction (`TASK_DIRECTION` 4 from now, on to `LEFT_ROUTINE`).
    fn m_stuck(&mut self, regs: &mut Regs) {
        let a6 = regs.a6;
        regs.d0 = self.r_0015f8(a6);
        (regs.a0, regs.x) = self.r_00167a();
        let above = self.read8(regs.a0 + PLANE_2);
        regs.d0 = low_byte(regs.d0, above);
        regs.c = false;
        if above == RISE_ON {
            self.write16(a6 + TASK_RATE, 0xFC00);
            (regs.x, _) = self.r_00053e(regs.d6, RISING_ROUTINE);
            regs.d0 = low_byte(regs.d0, 0x83);
            self.r_0034b6(regs.d0);
            return;
        }
        let direction = self.read8(a6 + TASK_DIRECTION);
        self.write8(FACING, direction);
        self.write8(a6 + TASK_DIRECTION, 4);
        self.r_00053e(regs.d6, LEFT_ROUTINE);
        regs.d4 = low_byte(regs.d4, self.read8(a6 + TASK_OBJECT));
        let a4;
        (regs.d4, a4, regs.x) = self.r_001cfa(regs.d4);
        regs.a4 = a4;
        regs.d0 = low_byte(regs.d0, direction);
        self.write8(a4 + OBJ_DIRECTION, direction);
        self.write8(a4 + OBJ_13, direction);
        if regs.d4 as u8 != 0 {
            self.write8(BYTE_A446, 0xFF);
            self.write8(BYTE_A447, 8);
            self.write16(a4 + OBJ_RATE, 0xFE00);
            self.write8(a4 + OBJ_STEP, 2);
            self.write8(a4 + OBJ_TIMER, 0);
            self.write8(a4 + OBJ_2A, 0);
            (regs.a0, regs.x, _) = self.r_00130e(regs.d0, a4);
            self.write16(a4 + OBJ_26, 0xA0);
            self.write32(a4 + OBJ_KEPT, OBJ_KEPT_OTHER);
            self.write32(a4 + OBJ_ROUTINE, OBJ_ROUTINE_OTHER);
            regs.d0 = low_byte(regs.d0, 0x3E);
        } else {
            regs.d0 = low_word(regs.d0, index(direction) as u16);
            regs.d5 = low_byte(regs.d5, self.read8(a4 + OBJ_SPRITE));
            let a5 = self.read32(a4 + OBJ_SPRITE_ADDR);
            regs.a5 = a5;
            let frame = self.read8(FRAME_BY_DIRECTION.wrapping_add(index(direction)));
            self.write8(a5 + SPRITE_FRAME, frame);
            self.m_or8(a5 + SPRITE_FLAGS, 0x10);
            self.write16(a4 + OBJ_RATE, 0xFE00);
            self.write8(a4 + OBJ_STEP, 0);
            self.write8(a4 + OBJ_TIMER, 0);
            self.write8(a4 + OBJ_KIND, 0);
            self.write8(a4 + OBJ_2A, 0);
            self.write16(a4 + OBJ_26, 0xA0);
            self.write32(a4 + OBJ_KEPT, OBJ_KEPT_0);
            self.write32(a4 + OBJ_ROUTINE, OBJ_ROUTINE_0);
            regs.d0 = low_byte(regs.d0, 0x38);
        }
        regs.c = false;
        self.r_0034b6(regs.d0);
    }

    /// `func_00D28C`, a task's routine: rising (bit 3 of its
    /// direction) at its rate (`func_00C742` with 0x30) until
    /// `func_001ACA` says it is down (not N): then on as
    /// `func_00D0A8` (direction kept, `TASK_20` 0, the Z80 0x4F).  Its
    /// sprite turned and drawn, the objects moved with it.  No pad
    /// script runs it: reviewed by reading.
    pub fn r_00d28c(&mut self, regs: &mut Regs) {
        let unwound;
        (regs.d0, unwound) = self.r_00d554(regs.d0, 0xD292);
        regs.unwound = false;
        if unwound {
            return;
        }
        let a6 = regs.a6;
        self.m_or8(a6 + TASK_DIRECTION, RISING);
        (regs.d1, regs.d5, regs.a5) = self.r_00d2de(regs.d0, regs.d1, regs.d5, a6);
        regs.d0 = 0x30;
        self.r_00c742(regs.d0, a6);
        let up;
        (regs.d0, regs.x, up) = self.r_001aca(regs.d0, a6);
        if !up {
            let direction = self.read8(a6 + TASK_DIRECTION) & 3;
            self.write8(a6 + TASK_DIRECTION, direction);
            self.write8(a6 + TASK_20, 0);
            self.r_00053e(regs.d6, MOVING_ROUTINE);
            regs.d0 = low_byte(regs.d0, 0x4F);
            self.r_0034b6(regs.d0);
        }
        let r = &mut *regs;
        (r.d1, r.d5, r.a5) = self.r_00d314(r.d1, r.d5, r.a5, a6);
        (r.d0, _, _) = self.r_001b94(r.a5, a6);
        (r.d0, _, _) = self.r_001b74(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, a6);
        self.write8(OR_Y, 0);
        (r.d0, r.d1, r.d4, r.d5, r.d6, r.a0, r.a1, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_00d418(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5, a6);
    }

    /// `func_00D2DE`: the task moved a step in its direction (the
    /// velocity's high bytes from `VELOCITY_X_BY_DIRECTION` and
    /// `VELOCITY_Y_BY_DIRECTION`, `func_001A88`, `func_001AA2`).  Gives
    /// back d1, d5 and a5 (the routine's sprite record).
    pub fn r_00d2de(&mut self, d0: u32, d1: u32, d5: u32, a6: u32) -> (u32, u32, u32) {
        self.write8(VELOCITY_X + 1, 0);
        self.write8(VELOCITY_Y + 1, 0);
        let direction = self.read8(a6 + TASK_DIRECTION) & 3;
        let d0 = low_word(d0, u16::from(direction));
        let x = self.read8(VELOCITY_X_BY_DIRECTION + u32::from(direction));
        self.write8(VELOCITY_X, x);
        let y = self.read8(VELOCITY_Y_BY_DIRECTION + u32::from(direction));
        self.write8(VELOCITY_Y, y);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        let (d0, d1, _, _, _) = self.r_001a88(d0, d1, a6);
        let (_, d1, _, _, _) = self.r_001aa2(d0, d1, a6);
        (d1, d5, a5)
    }

    /// `func_00D314`: on frames with bit 1 of `FRAME`, the sprite's
    /// frame by the task's facing (`FRAME_BY_FACING`), the facing
    /// turned a step (of 8, the shorter way) toward twice the
    /// direction plus 1; while rising, by half the direction.  Gives
    /// back d1 (the frame's index), d5 and a5 (the routine's sprite
    /// record), unchanged on other frames.
    pub fn r_00d314(&mut self, d1: u32, d5: u32, a5: u32, a6: u32) -> (u32, u32, u32) {
        if self.read8(FRAME) & (1 << 1) == 0 {
            return (d1, d5, a5);
        }
        let direction = self.read8(a6 + TASK_DIRECTION);
        let at = if direction >= 8 {
            direction >> 1
        } else {
            let facing = self.read8(a6 + TASK_FACING).wrapping_add(1);
            let toward = direction.wrapping_add(direction).wrapping_add(1);
            let (turn, borrow) = toward.overflowing_sub(facing);
            if turn != 0 {
                let up = if borrow { turn < 0xFC } else { turn < 4 };
                self.m_add8(a6 + TASK_FACING, if up { 1 } else { 0xFF });
            }
            let facing = self.read8(a6 + TASK_FACING) & 7;
            self.write8(a6 + TASK_FACING, facing);
            facing & 3
        };
        let d1 = low_word(d1, index(at) as u16);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        let frame = self.read8(FRAME_BY_FACING.wrapping_add(index(at)));
        self.write8(a5 + SPRITE_FRAME, frame);
        (d1, d5, a5)
    }

    /// `func_00D390`: the first direction (0-3) but `NOT_DIRECTION`
    /// whose next cell (`STEP_X`, `STEP_Y`) holds 1-0x2F in its second
    /// plane: the task's direction (and its facing, twice it, when
    /// there was none not to take); C clear.  None, C set.  Gives back
    /// d0, d1, a0 (the last cell looked at) and C.
    pub fn r_00d390(&mut self, d1: u32, a0: u32, a6: u32) -> (u32, u32, u32, bool) {
        let direction = self.read8(a6 + TASK_DIRECTION);
        let not = self.read8(NOT_BY_DIRECTION.wrapping_add(index(direction)));
        self.write8(NOT_DIRECTION, not);
        let mut d0 = low_byte(self.r_0015f8(a6), 0);
        let mut d1 = d1;
        let mut a0 = a0;
        loop {
            let looked = d0 as u8;
            self.write8(LOOKED_AT, looked);
            if looked != self.read8(NOT_DIRECTION) {
                let cell = self.read16(CELL);
                d0 = low_word(d0, u16::from(looked));
                let dx = self.read8(STEP_X + u32::from(looked));
                d1 = low_byte(d1, dx);
                self.m_add8(CELL_X, dx);
                let dy = self.read8(STEP_Y + u32::from(looked));
                d1 = low_byte(d1, dy);
                self.m_add8(CELL_Y, dy);
                (a0, _) = self.r_00167a();
                self.write16(CELL, cell);
                let value = self.read8(a0 + PLANE_2);
                d0 = low_byte(d0, value);
                if value != 0 && value < 0x30 {
                    if (self.read8(NOT_DIRECTION) as i8) < 0 {
                        let facing = self.read8(LOOKED_AT).wrapping_mul(2);
                        d0 = low_byte(d0, facing);
                        self.write8(a6 + TASK_FACING, facing);
                    }
                    let looked = self.read8(LOOKED_AT);
                    self.write8(a6 + TASK_DIRECTION, looked);
                    return (d0, d1, a0, false);
                }
            }
            let next = self.read8(LOOKED_AT).wrapping_add(1);
            d0 = low_byte(d0, next);
            if next == 4 {
                return (d0, d1, a0, true);
            }
        }
    }

    /// `func_00D418`: the objects moved with the task: object 0 (and,
    /// when the task has one, its `TASK_OBJECT` given object 5's place
    /// as the one run) put at the task's place (`func_00153A`,
    /// `func_005776`, `func_002F70`), their sprites framed from the
    /// facing (`func_00D4DA`); the other object's sprite is done first
    /// or last by the facing.  Gives back d0, d1, d4, d5, d6, a0, a1,
    /// a4, a5, a6, X and C.
    #[allow(clippy::type_complexity)]
    pub fn r_00d418(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let facing = self.read8(a6 + TASK_FACING);
        self.write8(FACING, facing);
        let byte = self.read8(a6 + TASK_20);
        self.write8(BYTE_20, byte);
        let depth = self.read8(a5 + SPRITE_DEPTH);
        self.write8(DEPTH, depth);
        let direction = self.read8(a6 + TASK_DIRECTION);
        self.write8(DIRECTION, direction);
        self.r_00153a(a6);
        self.write8(OBJECT, 0);
        self.write8(RUNNING, 0);
        let (d4, a4, _) = self.r_001cfa(low_byte(d4, 0));
        self.write32(RUNNING_ADDR, a4);
        let bit = self.read8(RECORDS_AD0A + RECORD_3) & 0x40;
        let d0 = low_byte(d0, bit);
        self.write8(BYTE_A933, bit);
        self.write8(BYTE_A932, 0);
        let (mut d0, d4, mut d5, d6, mut a0, _, _) =
            self.r_005776(d0, d1, d2, d3, d4, d5, d6, d7, RECORDS_AD0A, a1, a2, a3, a5, a6);
        let current = self.read8(CURRENT_TASK);
        let (d6, a6, _) = self.r_0005f8(low_byte(d6, current));
        let other = self.read8(a6 + TASK_OBJECT);
        let (mut d4, mut a4, _) = self.r_001cfa(low_byte(d4, other));
        let mut d1 = d1;
        let mut a1 = a1;
        if d4 as u8 != 0 {
            let (kept_d4, kept_a4) = (d4, a4);
            (d4, a4, _) = self.r_001cfa(low_byte(d4, 5));
            self.write8(RUNNING, d4 as u8);
            self.write32(RUNNING_ADDR, a4);
            (d0, d1, a0, a1, _) = self.r_002f70(d0, d1, d4, a0, a1);
            let facing = self.read8(FACING).wrapping_add(1) & 7;
            d0 = low_byte(d0, facing);
            if facing >= 3 {
                (d4, a4, _) = self.r_001cfa(low_byte(d4, 0));
                self.write8(BASE, BASE_0);
                (d0, d1, d5, _, _, _) = self.r_00d4da(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4);
                self.write8(BASE, BASE_OTHER);
                let (d0, d1, d5, a5, x, c) = self.r_00d4da(d0, d1, d2, d3, kept_d4, d5, d6, d7, a0, a1, a2, a3, kept_a4);
                return (d0, d1, kept_d4, d5, d6, a0, a1, kept_a4, a5, a6, x, c);
            }
            (d4, a4) = (kept_d4, kept_a4);
            self.write8(BASE, BASE_OTHER);
            (d0, d1, d5, _, _, _) = self.r_00d4da(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4);
        }
        (d0, d1, a0, a1, _) = self.r_002f70(d0, d1, d4, a0, a1);
        (d4, a4, _) = self.r_001cfa(low_byte(d4, 0));
        self.write8(BASE, BASE_0);
        let (d0, d1, d5, a5, x, c) = self.r_00d4da(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4);
        (d0, d1, d4, d5, d6, a0, a1, a4, a5, a6, x, c)
    }

    /// `func_00D4DA`: the object at a4 put at `POSITION_X`,
    /// `POSITION_Y`, given `BYTE_20`, its sprite's frame `BASE` plus
    /// the facing (or, rising, 8 plus half bit 1 of the direction),
    /// marked changed when it changes; `func_001B32`, `OR_Y` or'ed into
    /// its y, made the routine's sprite, and its depth from `DEPTH`
    /// less 1 (at least 2; `func_0007D2`).  Gives back d0, d1, d5, a5
    /// (the sprite record), X and C.
    #[allow(clippy::type_complexity)]
    pub fn r_00d4da(&mut self, _d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        let d5 = low_byte(d5, self.read8(a4 + OBJ_SPRITE));
        let a5 = self.read32(a4 + OBJ_SPRITE_ADDR);
        let x_pos = self.read16(POSITION_X);
        self.write16(a4 + OBJ_X, x_pos);
        let y_pos = self.read16(POSITION_Y);
        self.write16(a4 + OBJ_Y, y_pos);
        let byte = self.read8(BYTE_20);
        self.write8(a4 + OBJ_2A, byte);
        let direction = self.read8(DIRECTION);
        let frame = if direction & RISING != 0 { ((direction & 2) >> 1) | 8 } else { self.read8(FACING) };
        let frame = frame.wrapping_add(self.read8(BASE));
        if frame != self.read8(a5 + SPRITE_FRAME) {
            self.write8(a5 + SPRITE_FRAME, frame);
            self.m_or8(a5 + SPRITE_FLAGS, 0x10);
        }
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        let or_y = self.read8(OR_Y);
        self.m_or8(a5 + SPRITE_Y_LOW, or_y);
        self.write8(SLOT_INDEX, d5 as u8);
        self.write32(SLOT_ADDR, a5);
        let (depth, x) = self.read8(DEPTH).overflowing_sub(1);
        let d0 = low_byte(d0, if depth < 2 { 2 } else { depth });
        let c = self.r_0007d2(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, x);
        (d0, d1, d5, a5, x, c)
    }

    /// `func_00D554`: with bit 2 of `FLAGS` set, the caller's routine
    /// ends here (the return address taken off the stack into d0:
    /// `ret`, and the unwound flag).
    pub fn r_00d554(&mut self, d0: u32, ret: u32) -> (u32, bool) {
        if self.read8(FLAGS) & STOP_BIT != 0 { (ret, true) } else { (d0, false) }
    }

    /// `func_00D564`: the task being run (`CURRENT_TASK`) makes a task
    /// at its cell (`func_00D5E0`, `EIGHTH_ANIM`).  Gives back d0, d6,
    /// a0, a6, X and C.
    #[allow(clippy::type_complexity)]
    pub fn r_00d564(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        let current = self.read8(CURRENT_TASK);
        let (d6, a6, _) = self.r_0005f8(low_byte(d6, current));
        let d0 = self.r_0015f8(a6);
        let (d0, a0, x, c) = self.r_00d5e0(d0, d1, d2, d3, d4, d5, d6, d7, EIGHTH_ANIM, a1, a2, a3, a4, a5);
        (d0, d6, a0, a6, x, c)
    }

    /// `func_00D57C`: the first object from `NEAR_FIRST` (before
    /// `NEAR_END`) in use, not bit 7 of its flags, within `NEAR` of the
    /// task's position both ways: C set, d4 and a4 it.  None, C clear.
    pub fn r_00d57c(&mut self, d4: u32, a6: u32) -> (u32, u32, bool) {
        let x = self.read16(a6 + TASK_X);
        self.write16(NEAR_X, x);
        let y = self.read16(a6 + TASK_Y);
        self.write16(NEAR_Y, y);
        let mut d4 = low_byte(d4, NEAR_FIRST);
        loop {
            let a4;
            (d4, a4, _) = self.r_001cfa(d4);
            if self.read8(a4 + OBJ_USED) != 0
                && (self.read8(a4 + OBJ_FLAGS) as i8) >= 0
                && distance(self.read16(a4 + OBJ_X), self.read16(NEAR_X)) < NEAR
                && distance(self.read16(a4 + OBJ_Y), self.read16(NEAR_Y)) < NEAR
            {
                return (d4, a4, true);
            }
            let next = (d4 as u8).wrapping_add(1);
            d4 = low_byte(d4, next);
            if next == NEAR_END {
                return (d4, a4, false);
            }
        }
    }

    /// `func_00D5E0`: a task running `MADE_ROUTINE` (`func_001222`,
    /// depth `MADE_DEPTH`, priority `MADE_PRIORITY`), placed
    /// (`func_00CA06`, `func_001550`) with the animation at a0; C set
    /// when none could be made.  d4-d6 and a4-a6 are kept.  Gives back
    /// d0, a0, X and C.
    #[allow(clippy::type_complexity)]
    pub fn r_00d5e0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let (_, _, a5, a6, x, failed) =
            self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, MADE_DEPTH, MADE_PRIORITY, MADE_ROUTINE);
        if failed {
            return (d0, a0, x, true);
        }
        let (d0, _) = self.r_00ca06(d0, a5, a6);
        let (d0, x) = self.r_001550(d0, a6);
        self.write32(a6 + TASK_ANIM, a0);
        (d0, a0, x, false)
    }
}
