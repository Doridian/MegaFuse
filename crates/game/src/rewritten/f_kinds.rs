//! The routines by record kind (D_0069B2) that `func_006948`
//! (`f_records.rs`) runs when the record is taken, for the object at
//! `OBJECT` or the object being run; each leaves in d0 the command the
//! Z80 is sent after.  And the two tasks some of them make.

use crate::{Game, Regs};

/// An object's index (a byte) the routines work on (`func_001CFA`).
const OBJECT: u32 = 0xFFA93F;
/// The object being run (its index, a byte, and its address).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
/// Flags: bit 0 picks `LIMIT_30` of the two.
const FLAGS: u32 = 0xFF9540;
/// Bits and a count (bits 0-1) the routines set and count.
const BITS: u32 = 0xFFA442;
const BITS_COUNT: u8 = 3;
/// A count kept from 0 (it stays at 0xFF).
const COUNT_A443: u32 = 0xFFA443;
/// Set to 0x40 by `func_006A84`; its task counts down a byte each time
/// it is 1.
const SIGNAL: u32 = 0xFFA444;
/// A count kept below 0x80 (`func_006AAA`) and one counted with it.
const COUNT_9543: u32 = 0xFF9543;
const COUNT_AC7C: u32 = 0xFFAC7C;
/// The record taken (its offset; `func_006948`), and the records'
/// tasks' indexes (`f_records.rs`).
const FOUND: u32 = 0xFFAC7B;
const RECORDS: u32 = 0xFFFF_AC00;
const REC_TASK: u32 = 1;
/// How many objects there are to look at (`func_006AC8`), and tries.
const OBJECT_COUNT: u32 = 0xFF9541;
const TRIES: u8 = 5;
/// Bytes by object (`func_006AC8`): a number 1-10, 0x80, 4.
const BY_OBJECT_9E92: u32 = 0xFFFF_9E92;
const BY_OBJECT_9E8D: u32 = 0xFFFF_9E8D;
const BY_OBJECT_9E97: u32 = 0xFFFF_9E97;
/// Object indexes by an object's `OBJ_SLOT` (from 5 on).
const BY_SLOT: u32 = 0xFFFF_A968 - 5;
/// Set while `func_006DD4`'s task runs.
const MOVING: u32 = 0xFFA957;
/// A velocity's x (a word; `a_motion.rs`).
const VELOCITY_X: u32 = 0xFF9EC4;

/// An object's fields: in use; its routine and a kept one (longs);
/// flags; two bytes `func_006AC8` sets bit 6 of; their animation's
/// timer and step; position (words); bytes `func_006AC8` swaps; a
/// word `func_006D72` sets; flags (bit 5) and the slot of `BY_SLOT`;
/// counts that `func_006802` places by.
const OBJ_USED: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_KEPT: u32 = 0x06;
const OBJ_FLAGS: u32 = 0x10;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x15;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_X: u32 = 0x18;
const OBJ_Y: u32 = 0x1A;
const OBJ_SWAPPED: [u32; 7] = [0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x15];
const OBJ_RATE: u32 = 0x28;
const OBJ_FLAGS_2D: u32 = 0x2D;
const OBJ_COUNT_2E: u32 = 0x2E;
const OBJ_COUNT_30: u32 = 0x30;
const OBJ_SLOT: u32 = 0x32;
const OBJ_COUNT_36: u32 = 0x36;
const OBJ_COUNT_37: u32 = 0x37;
const OBJ_COUNT_38: u32 = 0x38;
/// Limits on `OBJ_COUNT_2E` and `OBJ_COUNT_30` (by bit 0 of `FLAGS`).
const LIMIT_2E: u8 = 5;
const LIMIT_30: u8 = 0x10;
const LIMIT_30_FLAG: u8 = 8;

/// A task's fields: a word and a byte its routine counts down, its
/// animation's timer and step, its animation.
const TASK_WORD: u32 = 0x0E;
const TASK_COUNT: u32 = 0x16;
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_ANIM: u32 = 0x20;

/// The tasks two routines make: routine (and priority 0x42).
const SIGNAL_TASK: u32 = 0x006DF2;
const MOVING_TASK: u32 = 0x006DD4;
const TASK_PRIORITY: u8 = 0x42;
/// The routine `func_006D72` gives the object being run, and the
/// animation for its record's task.
const FALL_ROUTINE: u32 = 0x00207C;
const FALL_ANIM: u32 = 0x0DA5BA;

/// The Z80's commands the routines leave in d0.
const COMMAND: u8 = 0x33;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// The object at `OBJECT` (`func_001CFA`): d4 and its address.
    fn object(&mut self, regs: &mut Regs) -> u32 {
        let object = self.read8(OBJECT);
        let (d4, a4, _) = self.r_001cfa(low_byte(regs.d4, object));
        regs.d4 = d4;
        a4
    }

    fn add8(&mut self, at: u32, by: u8) {
        let value = self.read8(at).wrapping_add(by);
        self.write8(at, value);
    }

    fn or8(&mut self, at: u32, bits: u8) {
        let value = self.read8(at) | bits;
        self.write8(at, value);
    }

    /// The end of each routine: d0 the command.
    fn command(regs: &mut Regs, command: u8) {
        regs.d0 = low_byte(regs.d0, command);
        regs.unwound = false;
    }

    /// `func_0069EE`: the object's `OBJ_COUNT_2E` one more (up to
    /// `LIMIT_2E`), and its `OBJ_COUNT_36` with it.
    pub fn r_0069ee(&mut self, regs: &mut Regs) {
        let a4 = self.object(regs);
        if self.read8(a4 + OBJ_COUNT_2E) != LIMIT_2E {
            self.add8(a4 + OBJ_COUNT_2E, 1);
            self.add8(a4 + OBJ_COUNT_36, 1);
        }
        Self::command(regs, COMMAND);
    }

    /// `func_006A0E`: the object's `OBJ_COUNT_30` one more (up to
    /// `LIMIT_30`, or `LIMIT_30_FLAG` with bit 0 of `FLAGS`), its
    /// `OBJ_COUNT_37` one more.
    pub fn r_006a0e(&mut self, regs: &mut Regs) {
        let a4 = self.object(regs);
        let limit = if self.read8(FLAGS) & 1 != 0 { LIMIT_30_FLAG } else { LIMIT_30 };
        if self.read8(a4 + OBJ_COUNT_30) != limit {
            self.add8(a4 + OBJ_COUNT_30, 1);
        }
        self.add8(a4 + OBJ_COUNT_37, 1);
        Self::command(regs, COMMAND);
    }

    /// `func_006A46`: bit 2 of `BITS` set.
    pub fn r_006a46(&mut self, regs: &mut Regs) {
        self.or8(BITS, 1 << 2);
        Self::command(regs, COMMAND);
    }

    /// `func_006A52`: `BITS`' count one more, up to 3.
    pub fn r_006a52(&mut self, regs: &mut Regs) {
        if self.read8(BITS) & BITS_COUNT != BITS_COUNT {
            self.add8(BITS, 1);
        }
        Self::command(regs, COMMAND);
    }

    /// `func_006A6C`: bit 5 of `BITS` set.
    pub fn r_006a6c(&mut self, regs: &mut Regs) {
        self.or8(BITS, 1 << 5);
        Self::command(regs, COMMAND);
    }

    /// `func_006A78`: bit 6 of `BITS` set.
    pub fn r_006a78(&mut self, regs: &mut Regs) {
        self.or8(BITS, 1 << 6);
        Self::command(regs, COMMAND);
    }

    /// `func_006A84`: a task running `func_006DF2` with a count of 4,
    /// `SIGNAL` 0x40.
    pub fn r_006a84(&mut self, regs: &mut Regs) {
        let failed;
        (regs.d6, regs.a6, _, failed) = self.r_0003e8(regs.d4, regs.d5, regs.d6, TASK_PRIORITY, SIGNAL_TASK);
        if !failed {
            self.write8(regs.a6 + TASK_COUNT, 4);
            self.write8(SIGNAL, 0x40);
        }
        Self::command(regs, COMMAND);
    }

    /// `func_006AAA`: the Z80 sent 0x1C; `COUNT_9543` one more (kept
    /// below 0x80) and `COUNT_AC7C` one more.  The command is 0x2F.
    pub fn r_006aaa(&mut self, regs: &mut Regs) {
        regs.d0 = low_byte(regs.d0, 0x1C);
        self.r_0034b6(regs.d0);
        let count = self.read8(COUNT_9543).wrapping_add(1);
        self.write8(COUNT_9543, if count & 0x80 != 0 { count.wrapping_sub(1) } else { count });
        self.add8(COUNT_AC7C, 1);
        Self::command(regs, 0x2F);
    }

    /// `func_006AC8`: a number from 1 to 10 at random.  Below 10 it goes
    /// to the object's byte in `BY_OBJECT_9E92` (with 0x80, 4 in the
    /// others; command 0x2B).  At 10 the object (its `OBJ_USED` 0, one
    /// more if it swaps) swaps with another in use and not bit 5 of
    /// `OBJ_FLAGS_2D`, looked for from a random one (`TRIES` at most):
    /// their places, routines, slots and other fields, each given a
    /// task (`func_006CD2`); command 0x2C.  None found, the number is
    /// 1-9 at random.
    pub fn r_006ac8(&mut self, regs: &mut Regs) {
        let mut d0 = self.r_001bc0(9, regs.d1);
        d0 = low_byte(d0, (d0 as u8).wrapping_add(1));
        if d0 as u8 == 0x0A {
            if self.swap_objects(regs, d0) {
                return;
            }
            d0 = self.r_001bc0(8, regs.d1);
            d0 = low_byte(d0, (d0 as u8).wrapping_add(1));
        }
        self.object(regs);
        let index = (regs.d4 as u8 as i8 as i16) as u16;
        regs.d4 = low_word(regs.d4, index);
        let at = index as i16 as i32 as u32;
        self.write8(BY_OBJECT_9E92.wrapping_add(at), d0 as u8);
        self.write8(BY_OBJECT_9E8D.wrapping_add(at), 0x80);
        self.write8(BY_OBJECT_9E97.wrapping_add(at), 4);
        regs.a0 = BY_OBJECT_9E97;
        regs.d4 = low_byte(regs.d4, self.read8(RUNNING));
        regs.d0 = d0;
        Self::command(regs, 0x2B);
    }

    /// `func_006AC8`'s swap (`FOUND` counts the tries).  Whether one was
    /// made; if not, d4 is the only register changed.
    fn swap_objects(&mut self, regs: &mut Regs, d0: u32) -> bool {
        let a4 = self.object(regs);
        let d4 = regs.d4;
        self.write8(a4 + OBJ_USED, 0);
        let count = self.read8(OBJECT_COUNT);
        let mut d0 = self.r_001bc0(low_byte(d0, count.wrapping_sub(1)), regs.d1);
        let mut d5 = low_byte(regs.d5, d0 as u8);
        let mut a5;
        self.write8(FOUND, TRIES);
        loop {
            (d5, a5) = self.r_001d0c(d5);
            if self.read8(a5 + OBJ_USED) != 0 {
                let flags = self.read8(a5 + OBJ_FLAGS_2D) & 0x20;
                d0 = low_byte(d0, flags);
                if flags == 0 {
                    break;
                }
            }
            let tries = self.read8(FOUND).wrapping_sub(1);
            self.write8(FOUND, tries);
            if tries == 0 {
                return false;
            }
            d5 = low_byte(d5, (d5 as u8).wrapping_add(1));
            if d5 as u8 == self.read8(OBJECT_COUNT) {
                d5 = low_byte(d5, 0);
            }
        }
        self.add8(a4 + OBJ_USED, 1);
        self.r_001542(a4);
        let value = self.read16(a5 + OBJ_X);
        self.write16(a4 + OBJ_X, value);
        let value = self.read16(a5 + OBJ_Y);
        self.write16(a4 + OBJ_Y, value);
        self.r_001300(a5);
        let slot = self.read8(a4 + OBJ_SLOT);
        let value = self.read8(a5 + OBJ_SLOT);
        self.write8(a4 + OBJ_SLOT, value);
        self.write8(a5 + OBJ_SLOT, slot);
        for (object, index) in [(a5, d5), (a4, d4)] {
            let slot = self.read8(object + OBJ_SLOT);
            d0 = low_byte(d0, slot);
            if slot & 0x80 == 0 {
                d0 = low_word(d0, u16::from(slot));
                self.write8(BY_SLOT + u32::from(slot), index as u8);
            }
        }
        for field in [OBJ_ROUTINE, OBJ_KEPT] {
            let long = self.read32(a4 + field);
            let value = self.read32(a5 + field);
            self.write32(a4 + field, value);
            self.write32(a5 + field, long);
        }
        for field in OBJ_SWAPPED {
            let byte = self.read8(a4 + field);
            let value = self.read8(a5 + field);
            self.write8(a4 + field, value);
            self.write8(a5 + field, byte);
        }
        let direction_4 = (self.read8(a4 + OBJ_DIRECTION) & 0x7F) | 0x40;
        let direction_5 = (self.read8(a5 + OBJ_DIRECTION) & 0x7F) | 0x40;
        self.write8(a5 + OBJ_DIRECTION, direction_4);
        self.write8(a4 + OBJ_DIRECTION, direction_5);
        for object in [a4, a5] {
            self.write8(object + OBJ_TIMER, 0);
            self.write8(object + OBJ_STEP, 0);
        }
        // bit 7 of the flags swapped
        let flags_4 = self.read8(a4 + OBJ_FLAGS);
        let flags_5 = self.read8(a5 + OBJ_FLAGS);
        self.write8(a5 + OBJ_FLAGS, (flags_5 & 0x7F) | (flags_4 & 0x80));
        self.write8(FOUND, flags_5 & 0x80);
        let flags = (flags_4 & 0x7F) | (flags_5 & 0x80);
        self.write8(a4 + OBJ_FLAGS, flags);
        d0 = low_byte(d0, flags);
        let mut d1 = low_byte(regs.d1, flags_5 & 0x80);
        let mut d2 = regs.d2;
        let (a0, _, _) = self.r_00130e(d0, a4);
        (d0, d1, d2) = self.r_006cd2(d0, d1, d2, regs.d3, d4, d5, regs.d6, regs.d7, a0, regs.a1, regs.a2, regs.a3, a4, a5);
        self.r_001542(a4);
        let (a0, _, _) = self.r_00130e(d0, a5);
        (_, d1, d2) = self.r_006cd2(d0, d1, d2, regs.d3, d5, d4, regs.d6, regs.d7, a0, regs.a1, regs.a2, regs.a3, a5, a4);
        regs.d4 = low_byte(d5, self.read8(RUNNING));
        let running = self.read32(RUNNING_ADDR);
        let (d0, _) = self.r_0015e6(running);
        let (a0, _) = self.r_00167a();
        regs.d0 = d0;
        regs.d1 = d1;
        regs.d2 = d2;
        regs.a0 = a0;
        Self::command(regs, 0x2C);
        true
    }

    /// `func_006CFC`: bit 3 of the object's flags set.
    pub fn r_006cfc(&mut self, regs: &mut Regs) {
        let a4 = self.object(regs);
        self.or8(a4 + OBJ_FLAGS, 1 << 3);
        Self::command(regs, COMMAND);
    }

    /// `func_006D10`: a task running `func_006DD4` for 600 frames.
    pub fn r_006d10(&mut self, regs: &mut Regs) {
        let failed;
        (regs.d6, regs.a6, _, failed) = self.r_0003e8(regs.d4, regs.d5, regs.d6, TASK_PRIORITY, MOVING_TASK);
        if !failed {
            self.write16(regs.a6 + TASK_WORD, 600);
        }
        Self::command(regs, COMMAND);
    }

    /// `func_006D30`: the object's `OBJ_COUNT_38` one more and bit 4 of
    /// its flags set.
    pub fn r_006d30(&mut self, regs: &mut Regs) {
        let a4 = self.object(regs);
        self.add8(a4 + OBJ_COUNT_38, 1);
        self.or8(a4 + OBJ_FLAGS, 1 << 4);
        Self::command(regs, COMMAND);
    }

    /// `func_006D48`: `BITS`' count one less, down to 0.
    pub fn r_006d48(&mut self, regs: &mut Regs) {
        if self.read8(BITS) & BITS_COUNT != 0 {
            let value = self.read8(BITS).wrapping_sub(1);
            self.write8(BITS, value);
        }
        Self::command(regs, COMMAND);
    }

    /// `func_006D5E`: `COUNT_A443` one more, kept at 0xFF.
    pub fn r_006d5e(&mut self, regs: &mut Regs) {
        let count = self.read8(COUNT_A443).wrapping_add(1);
        self.write8(COUNT_A443, if count == 0 { 0xFF } else { count });
        Self::command(regs, COMMAND);
    }

    /// `func_006D72`: the record's task (`FOUND`, `func_0005F8`) given
    /// `FALL_ANIM` from the start; the object being run set up afresh
    /// (flags bit 7, kind 8, `func_00130E`, `func_001336`, rate
    /// -0x380) and given `FALL_ROUTINE`, its routine kept.  Command
    /// 0x38.
    pub fn r_006d72(&mut self, regs: &mut Regs) {
        let found = self.read8(FOUND);
        regs.d1 = low_byte(low_word(regs.d1, 0), found);
        regs.a0 = RECORDS;
        let task = self.read8(RECORDS + u32::from(found) + REC_TASK);
        (regs.d6, regs.a6, _) = self.r_0005f8(low_byte(regs.d6, task));
        self.write32(regs.a6 + TASK_ANIM, FALL_ANIM);
        self.write8(regs.a6 + TASK_STEP, 0);
        self.write8(regs.a6 + TASK_TIMER, 0);
        regs.d4 = low_byte(regs.d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        self.or8(a4 + OBJ_FLAGS, 0x80);
        self.write8(a4 + OBJ_KIND, 8);
        let (_, x, _) = self.r_00130e(regs.d0, a4);
        self.write8(a4 + OBJ_TIMER, 0);
        self.write8(a4 + OBJ_STEP, 2);
        let r = &mut *regs;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, _, _, _) = self.r_001336(r.d0, r.d1, r.d2, r.d5, a4, x);
        self.write16(a4 + OBJ_RATE, 0xFC80);
        let value = self.read32(a4 + OBJ_ROUTINE);
        self.write32(a4 + OBJ_KEPT, value);
        self.write32(a4 + OBJ_ROUTINE, FALL_ROUTINE);
        Self::command(regs, 0x38);
    }

    /// `func_006DD4`, a task's routine: `MOVING` set and the task moved
    /// up (`func_001A88`, `VELOCITY_X` -1); when that says so (N), the
    /// task ends (`func_000560`) and `MOVING` is cleared.
    pub fn r_006dd4(&mut self, regs: &mut Regs) {
        self.write8(MOVING, 1);
        self.write16(VELOCITY_X, 0xFF00);
        let r = &mut *regs;
        let done;
        (r.d0, r.d1, r.x, done, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        if done {
            self.write8(MOVING, 0);
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
        }
        r.unwound = false;
    }

    /// `func_006DF2`, a task's routine: each time `SIGNAL` is 1 it is
    /// made 0xFF and the task's count goes down; at 0 the task ends
    /// (`func_000560`).
    pub fn r_006df2(&mut self, regs: &mut Regs) {
        let signal = self.read8(SIGNAL);
        regs.unwound = false;
        if signal != 1 {
            regs.c = signal < 1;
            return;
        }
        self.write8(SIGNAL, 0xFF);
        let count = self.read8(regs.a6 + TASK_COUNT);
        self.write8(regs.a6 + TASK_COUNT, count.wrapping_sub(1));
        regs.x = count == 0;
        regs.c = regs.x;
        if count == 1 {
            (regs.d6, regs.x, regs.c) = self.r_000560(regs.d5, regs.d6, regs.x);
        }
    }
}
