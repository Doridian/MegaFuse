//! An object that flies to a point and waits there, 0x1338A-0x13508:
//! `func_01338A` takes it to (0x88, 0x78) at speed 0x200; there it
//! bobs (`func_0133CA`, its offset from `func_012A8C`) until the
//! count `COUNT_AD57` is set, then goes through `loc_012B48` to
//! `func_013420` and its animations.  The task `func_0134E6` it makes
//! counts `COUNTDOWN` down (every 4th frame) and steps `COUNT_AD57`;
//! at 5 the list `D_0134DA` is loaded (`func_0094D8`) and, after its
//! last animation, the object is gone.  The task `func_013750`
//! (made with it, `func_012AF8`) watches object 0: once it is at
//! (0x88, 0xB8) `COUNT_AD57` goes up; until then `BYTE_954F` is the
//! bit (`D_009C68`) of the first way from its cell, from the way
//! toward the point on, that is not empty.
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read.

use crate::{Game, Regs};

/// An object's fields: byte 0 (in use; cleared when it is gone), its
/// routine and the routine `loc_012B48` goes on to (longs), its
/// position (words), its animation (a long), the animation's timer
/// and step, its offset (the high byte of a word, `a_motion.rs`).
const OBJ_USED: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_NEXT_ROUTINE: u32 = 0x06;
const OBJ_X: u32 = 0x18;
const OBJ_Y: u32 = 0x1A;
const OBJ_ANIM: u32 = 0x20;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_OFFSET: u32 = 0x2A;

/// Where it flies from and to (words; `math.rs`), and the speed.
const FROM_X: u32 = 0xFF9EFA;
const FROM_Y: u32 = 0xFF9EFE;
const TO_X: u32 = 0xFF9EE6;
const TO_Y: u32 = 0xFF9EE8;
const SPEED: u32 = 0xFF9F00;
const FLY_TO_X: u16 = 0x88;
const FLY_TO_Y: u16 = 0x78;
const FLY_SPEED: u16 = 0x200;
/// `func_004348`'s reach (pixels).
const REACH: u32 = 3;
/// Its offset (the high byte) when it is at the top of its bob.
const OFFSET_TOP: u8 = 0xF2;

/// A count: set elsewhere to send the object on; cleared, then stepped
/// by `func_0134E6`; at `COUNT_LOAD` the list is loaded.
const COUNT_AD57: u32 = 0xFFAD57;
const COUNT_LOAD: u8 = 5;
/// Counted down by `func_0134E6` every 4th frame.
const COUNTDOWN: u32 = 0xFF9EB9;
/// Stepped by `func_0134E6` each time its count runs out.
const COUNT_9663: u32 = 0xFF9663;
/// The frame count.
const FRAME: u32 = 0xFF80D8;
/// The list `func_0094D8` loads (a long).
const LIST: u32 = 0xFF92FA;
const LIST_0134DA: u32 = 0x0134DA;
/// Object 0's position (words), and where `func_013750` waits for it.
const OBJECT_0_X: u32 = 0xFFA44A;
const OBJECT_0_Y: u32 = 0xFFA44C;
const TO_X_LOW: u32 = 0xFF9EE7;
const WAIT_X: u8 = 0x88;
const WAIT_Y: u16 = 0xB8;
/// 1 when object 0 is there, else the bit of a way (`WAY_BITS`), 0
/// when the task ends.
const BYTE_954F: u32 = 0xFF954F;
const WAY_BITS: u32 = 0x009C68;
/// The way looked at (0-3; `func_00173E`'s direction).
const WAY: u32 = 0xFFAABD;
/// The cell's address in the cell map (`func_00167A`).
const CELL_AT: u32 = 0xFF9ED6;
/// The running object (index, address; `a_objects.rs`).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;

/// Routines and animations.
const ROUTINE_0133DA: u32 = 0x0133DA;
const ROUTINE_012B48: u32 = 0x012B48;
const ROUTINE_013420: u32 = 0x013420;
const ROUTINE_01343C: u32 = 0x01343C;
const ROUTINE_013466: u32 = 0x013466;
const ROUTINE_0134A0: u32 = 0x0134A0;
const TASK_0134E6: u32 = 0x0134E6;
const TASK_PRIORITY: u8 = 0x31;
const ROUTINE_0137CA: u32 = 0x0137CA;
const ANIM_E8526: u32 = 0x0E8526;
const ANIM_E856C: u32 = 0x0E856C;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_01338A`, an object's routine: it flies toward
    /// (`FLY_TO_X`, `FLY_TO_Y`) at `FLY_SPEED` (`func_00112C`,
    /// `func_0019C0`, `func_0019DA`); once within `REACH` it is put in
    /// the middle of its cell and its routine is `func_0133DA`.  Then
    /// `func_0133CA`.
    pub fn r_01338a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let x = self.read16(r.a4 + OBJ_X);
        self.write16(FROM_X, x);
        let y = self.read16(r.a4 + OBJ_Y);
        self.write16(FROM_Y, y);
        self.write16(TO_X, FLY_TO_X);
        self.write16(TO_Y, FLY_TO_Y);
        self.write16(SPEED, FLY_SPEED);
        self.r_00112c();
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
        (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        let there;
        (r.d0, r.x, there) = self.r_004348(REACH);
        if there {
            self.r_001852(r.a4);
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_0133DA);
        }
        (r.d0, r.d1, r.d5, r.a5, r.x, r.c) = self.r_0133ca(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4);
    }

    /// `func_0133CA`: the object bobs (`func_012A8C`) and its sprite is
    /// placed (`func_0133D0`).  Gives back d0, d1, d5, a5 (the
    /// routine's sprite record), X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0133ca(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        let (_, d5, a5, _) = self.r_012a8c(d0, d5, a4);
        let (d0, d1, x, c) = self.r_0133d0(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        (d0, d1, d5, a5, x, c)
    }

    /// `.L0133D0`: the object's sprite placed (`func_001B32`) and its
    /// depth (`func_001B5C`).  Gives back d0, d1, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0133d0(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        let (d0, x, c) = self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5);
        (d0, d1, x, c)
    }

    /// `func_0133DA`, an object's routine: it bobs (`func_0133CA`);
    /// when at the top with `COUNT_AD57` set, the Z80 gets 0xFE (d1 8),
    /// and the running object's timer is 0x40 and its routine
    /// `loc_012B48`, to go on to `func_013420`.
    pub fn r_0133da(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d5, r.a5, r.x, r.c) = self.r_0133ca(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4);
        let offset = self.read8(r.a4 + OBJ_OFFSET);
        r.d0 = low_byte(r.d0, offset);
        r.c = offset < OFFSET_TOP;
        if offset != OFFSET_TOP {
            return;
        }
        let count = self.read8(COUNT_AD57);
        r.d0 = low_byte(r.d0, count);
        r.c = false;
        if count == 0 {
            return;
        }
        r.d1 = 8;
        r.d0 = low_byte(r.d0, 0xFE);
        self.r_00349e(r.d0);
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        self.write8(r.a4 + OBJ_TIMER, 0x40);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_012B48);
        self.write32(r.a4 + OBJ_NEXT_ROUTINE, ROUTINE_013420);
        self.r_012b48(r);
    }

    /// `func_013420`, an object's routine: the animation `ANIM_E8526`
    /// from its start, offset 0, and on as `func_01343C`.
    pub fn r_013420(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write32(r.a4 + OBJ_ANIM, ANIM_E8526);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_OFFSET, 0);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01343C);
        r.c = false;
        self.r_01343c(r);
    }

    /// `func_01343C`, an object's routine: its animation run
    /// (`func_001336`) and its sprite placed; at the animation's end
    /// `COUNT_AD57` is cleared, the task `func_0134E6` made and it
    /// goes on as `func_013466`.
    pub fn r_01343c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if !r.c {
            (r.d0, r.d1, r.x, r.c) = self.r_0133d0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            return;
        }
        self.write8(COUNT_AD57, 0);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_013466);
        (r.d6, r.a6, r.x, _) = self.r_0003e8(r.d4, r.d5, r.d6, TASK_PRIORITY, TASK_0134E6);
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        r.c = false;
        self.r_013466(r);
    }

    /// `func_013466`, an object's routine: when `COUNT_AD57` comes to
    /// `COUNT_LOAD` the list `D_0134DA` is loaded (`func_0094D8`), the
    /// count cleared, and it goes on as `func_0134A0` with the
    /// animation `ANIM_E856C`.  Its sprite is placed (`func_0133D0`).
    pub fn r_013466(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(COUNT_AD57);
        r.d0 = low_byte(r.d0, count);
        r.c = count < COUNT_LOAD;
        if count == COUNT_LOAD {
            self.write32(LIST, LIST_0134DA);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
                self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
            if self.restarted.is_some() {
                return;
            }
            self.write8(COUNT_AD57, 0);
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_0134A0);
            self.write32(r.a4 + OBJ_ANIM, ANIM_E856C);
            self.write8(r.a4 + OBJ_STEP, 0);
            self.write8(r.a4 + OBJ_TIMER, 0);
            r.c = false;
        }
        (r.d0, r.d1, r.x, r.c) = self.r_0133d0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_0134A0`, an object's routine: while `COUNT_AD57` is set
    /// its animation runs (`func_001336`; the Z80 gets 0x80 at its
    /// step 0x20); at its end the object is gone (byte 0 cleared, its
    /// sprite record freed, `func_000890`) and the count cleared.
    pub fn r_0134a0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(COUNT_AD57);
        r.d0 = low_byte(r.d0, count);
        r.c = false;
        if count == 0 {
            return;
        }
        let timer = self.read8(r.a4 + OBJ_TIMER);
        r.d0 = low_byte(r.d0, timer);
        if timer == 0 {
            let step = self.read8(r.a4 + OBJ_STEP);
            r.d0 = low_byte(r.d0, step);
            if step == 0x20 {
                r.d0 = low_byte(r.d0, 0x80);
                self.r_0034b6(r.d0);
            }
        }
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if !r.c {
            return;
        }
        self.write8(r.a4 + OBJ_USED, 0);
        self.r_000890();
        self.write8(COUNT_AD57, 0);
        r.c = false;
    }

    /// `func_0134E6`, a task's routine: every 4th frame `COUNTDOWN`
    /// goes down; when it runs out `COUNT_AD57` and `COUNT_9663` go up
    /// and the task ends (`func_000560`).
    pub fn r_0134e6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let turn = self.read8(FRAME) & 3;
        r.d0 = low_byte(r.d0, turn);
        r.c = false;
        if turn != 0 {
            return;
        }
        let was = self.read8(COUNTDOWN);
        self.write8(COUNTDOWN, was.wrapping_sub(1));
        (r.x, r.c) = (was == 0, was == 0);
        if was != 1 {
            return;
        }
        let count = self.read8(COUNT_AD57).wrapping_add(1);
        self.write8(COUNT_AD57, count);
        let (count, carry) = self.read8(COUNT_9663).overflowing_add(1);
        self.write8(COUNT_9663, count);
        (r.x, r.c) = (carry, carry);
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }

    /// `func_013750`, a task's routine: `func_013756` with the point's
    /// y `WAIT_Y`.
    pub fn r_013750(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write16(TO_Y, WAIT_Y);
        r.c = false;
        (r.d0, r.d4, r.a0, r.a4, r.x, r.c) = self.r_013756(r.d4, r.d6, r.a0, r.a4);
    }

    /// `.L013756`: if object 0 is within `REACH` of the point
    /// (`WAIT_X`; `func_004348`), `BYTE_954F` is 1 and the task goes
    /// on as `func_0137CA`.  Otherwise, from the way toward the point
    /// on (`WAY`), the first way whose next cell from object 0's
    /// (`func_00173E`) is not empty: `BYTE_954F` its bit.  Gives back
    /// d0, d4, a0, a4 (object 0), X and C.
    pub fn r_013756(&mut self, d4: u32, d6: u32, a0: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        self.write8(TO_X_LOW, WAIT_X);
        let from_x = self.read16(OBJECT_0_X);
        self.write16(FROM_X, from_x);
        let from_y = self.read16(OBJECT_0_Y);
        self.write16(FROM_Y, from_y);
        let (mut d0, _, there) = self.r_004348(REACH);
        if there {
            self.write8(BYTE_954F, 1);
            let (x, c) = self.r_00053e(d6, ROUTINE_0137CA);
            return (d0, d4, a0, a4, x, c);
        }
        loop {
            self.write8(WAY, d0 as u8);
            let (d4, a4, _) = self.r_001cfa(d4 & 0xFFFF_FF00);
            (d0, _) = self.r_0015e6(a4);
            let way = self.read8(WAY);
            d0 = low_byte(d0, way);
            let x = self.r_00173e(d0, a4);
            let a0 = self.read32(CELL_AT);
            let cell = self.read8(a0);
            d0 = low_byte(d0, cell);
            if cell != 0 {
                let way = self.read8(WAY) as i8 as i16 as u16;
                d0 = (d0 & 0xFFFF_0000) | u32::from(way);
                let bit = self.read8(WAY_BITS.wrapping_add(way as i16 as u32));
                self.write8(BYTE_954F, bit);
                return (d0, d4, WAY_BITS, a4, x, false);
            }
            d0 = low_byte(d0, way.wrapping_add(1) & 3);
        }
    }

    /// `func_0137CA`, a task's routine: `BYTE_954F` cleared,
    /// `COUNT_AD57` up, and the task ends (`func_000560`).
    pub fn r_0137ca(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(BYTE_954F, 0);
        let (count, carry) = self.read8(COUNT_AD57).overflowing_add(1);
        self.write8(COUNT_AD57, count);
        (r.x, r.c) = (carry, carry);
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }
}
