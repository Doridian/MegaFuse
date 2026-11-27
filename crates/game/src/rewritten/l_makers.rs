//! Routines of the lists by world of D_00A04A (D_00A0D0, D_00A158):
//! each makes a task with a sprite where the position (`POSITION`) is
//! (`func_00CA06`, its frame `MADE_FRAME`) and leaves a number in d0
//! and C clear (`func_00CFB6`: C set); and the routines of the tasks
//! they make.  `func_00C93C` also points the longs `HOOK_A996`,
//! `HOOK_A99A`, `HOOK_A99E` at `func_00CA38`, which counts an object's
//! nearness into `NEAREST` and `NEAR_BITS` for its tasks.

use crate::{Game, Regs};

/// A task's fields: its position (words; the y's low byte), its byte
/// 0x12, a count, a byte 0x19, a flag (0x16: not 0 for the first of
/// `func_00C93C`'s tasks) and its offset (the high byte: the sprite
/// drawn so many pixels up; `a_motion.rs`).
const TASK_Y: u32 = 0x10;
const TASK_Y_LOW: u32 = 0x11;
const TASK_12: u32 = 0x12;
const TASK_COUNT: u32 = 0x16;
const TASK_19: u32 = 0x19;
const TASK_OFFSET: u32 = 0x20;
/// A sprite record's frame and flags, its y's low byte.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_Y_LOW: u32 = 0x0B;
/// An object's fields: its routine (a long), flags, a kind, a byte
/// whose low nibble `func_00CA38` reads, the animation's timer and
/// step, a word `func_00CFF8` sets, and its nearness (`func_00CA38`).
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_FLAGS: u32 = 0x10;
const OBJ_KIND: u32 = 0x15;
const OBJ_19: u32 = 0x19;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_RATE: u32 = 0x28;
const OBJ_NEAR: u32 = 0x2A;

/// The running task's index.
const CURRENT_TASK: u32 = 0xFF80EE;
/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// The object being run (its index and address).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
/// An object's index `func_00CFF8` sets.
const OTHER: u32 = 0xFFA93F;
/// A position (words, x then y): the made tasks are put there.
const POSITION_X: u32 = 0xFF9F04;
/// The frame the made task's sprite gets.
const MADE_FRAME: u32 = 0xFFAC91;
/// Flags: bit 5 starts `func_00C80C`'s task; bit 3 set by
/// `func_00C886`.
const FLAGS: u32 = 0xFF9540;
/// A byte that picks the Z80's command (`func_00C822`, `RISE_SOUNDS`)
/// and a byte `func_00C89E` compares with 5.
const BYTE_953A: u32 = 0xFF953A;
const BYTE_953E: u32 = 0xFF953E;
/// Set to 0x7F by `func_00C89E` unless `BYTE_953E` is 5.
const BYTE_A3F9: u32 = 0xFFA3F9;
/// The highest nearness (`func_00CA38`) and the bits for it.
const NEAREST: u32 = 0xFFAC88;
const NEAR_BITS: u32 = 0xFFAC89;
/// Where `func_00CA38` keeps its row (d0 - 1, times 4), and the count
/// of `func_00C93C`'s third kind of tasks.
const ROW: u32 = 0xFFAC8B;
const LEFT: u32 = 0xFFAC90;
/// Longs pointed at `func_00CA38` (by `func_00C93C`, `func_00CA0E`,
/// `func_00CA1E`).
const HOOK_A996: u32 = 0xFFA996;
const HOOK_A99A: u32 = 0xFFA99A;
const HOOK_A99E: u32 = 0xFFA99E;
const NEARNESS: u32 = 0x00CA38;
/// The nearness by row and the object's quarter (D_00CAAA) and the
/// bits for it (D_00CAB6).
const NEAR_TABLE: u32 = 0x00CAAA;
const NEAR_BITS_TABLE: u32 = 0x00CAB6;
/// For `func_00C93C`'s four tasks of `ROUTINE_CAF0` (by `LEFT`, 1-4):
/// their flag bits (from 0x00CA30, bytes in the ROM after
/// `func_00CA1E`) and their highest offset (D_00CA33 from 1).
const BAR_BITS: u32 = 0x00CA2F;
const BAR_MOST: u32 = 0x00CA33;
/// The Z80's commands by `BYTE_953A` (D_00C868).
const RISE_SOUNDS: u32 = 0x00C868;

/// The cell (column, row) and the cell's second byte (0x200 on).
const CELL_Y: u32 = 0xFF9F09;
const SECOND: u32 = 0x200;

/// The routines, depths and priorities of the tasks made here.
const ROUTINE_C7C0: u32 = 0x00C7C0;
const ROUTINE_C80C: u32 = 0x00C80C;
const ROUTINE_C822: u32 = 0x00C822;
const ROUTINE_C89E: u32 = 0x00C89E;
const ROUTINE_C8C8: u32 = 0x00C8C8;
const ROUTINE_C938: u32 = 0x00C938;
const ROUTINE_CAC2: u32 = 0x00CAC2;
const ROUTINE_CAE8: u32 = 0x00CAE8;
const ROUTINE_CAF0: u32 = 0x00CAF0;
const ROUTINE_CFF8: u32 = 0x00CFF8;
const ROUTINE_D07C: u32 = 0x00D07C;
const PRIORITY: u8 = 0x42;
const PRIORITY_CAC2: u8 = 0x43;
const C886_PRIORITY: u8 = 0x10;
/// The routine an object gets from `func_00CFF8`.
const OBJECT_ROUTINE: u32 = 0x002038;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_001222` on the family's registers: a task of `routine`.
    fn l_make(&mut self, r: &mut Regs, depth: u8, priority: u8, routine: u32) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, priority, routine);
    }

    /// A task of `routine` made (`func_001222`) and put at the position
    /// (`func_00CA06`).
    fn l_make_at(&mut self, r: &mut Regs, depth: u8, priority: u8, routine: u32) {
        self.l_make(r, depth, priority, routine);
        (r.d0, r.c) = self.r_00ca06(r.d0, r.a5, r.a6);
    }

    /// `func_00C796`: `func_00C79C` with frame 0x35.
    pub fn r_00c796(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(MADE_FRAME, 0x35);
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00c79c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00C79C`: a task of `func_00C7C0` (depth 3) made at the
    /// position.  Gives back d0 (0), d5, d6, a5, a6, X and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00c79c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let (d5, d6, a5, a6, x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 3, PRIORITY, ROUTINE_C7C0);
        let (d0, _) = self.r_00ca06(d0, a5, a6);
        (low_byte(d0, 0), d5, d6, a5, a6, x, false)
    }

    /// `func_00C7C0`, a task's routine: its sprite placed
    /// (`func_001B7E`) and its depth from its y less 8 (`func_0007D2`).
    pub fn r_00c7c0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        let y = self.read8(r.a5 + SPRITE_Y_LOW);
        let (less, borrow) = y.overflowing_sub(8);
        r.x = borrow;
        r.d0 = low_byte(r.d0, (!less & 0xFE) | 2);
        r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
    }

    /// `func_00C7DA`: `func_00C7E0` with frame 0x34.
    pub fn r_00c7da(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(MADE_FRAME, 0x34);
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00c7e0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00C7E0` (`.L00C7E0`): a task of `func_00C80C` (depth 0xFE)
    /// made at the position, its offset and count 0.  Gives back d0
    /// (0x74), d5, d6, a5, a6, X and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00c7e0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let (d5, d6, a5, a6, x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0xFE, PRIORITY, ROUTINE_C80C);
        let (d0, _) = self.r_00ca06(d0, a5, a6);
        self.write8(a6 + TASK_OFFSET, 0);
        self.write8(a6 + TASK_COUNT, 0);
        (low_byte(d0, 0x74), d5, d6, a5, a6, x, false)
    }

    /// `func_00C80C`, a task's routine: its sprite placed
    /// (`func_00C864`) until bit 5 of `FLAGS` is set; then it goes on
    /// as `func_00C822`.
    pub fn r_00c80c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(FLAGS) & 0x20 == 0 {
            (r.d0, r.x, r.c) = self.r_00c864(r.a5, r.a6);
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C822);
        self.r_00c822(r);
    }

    /// `func_00C822`, a task's routine: it rises a pixel each fourth
    /// frame of 16 out of 32 (its count), the Z80 getting the command
    /// for `BYTE_953A` (`RISE_SOUNDS`), and its sprite is placed
    /// (`func_00C864`); at offset 0xE0 the cell two rows under it
    /// becomes 4 (its second byte 0) and it ends (`func_00C86E`).
    pub fn r_00c822(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(r.a6 + TASK_COUNT).wrapping_add(1);
        self.write8(r.a6 + TASK_COUNT, count);
        let offset = self.read8(r.a6 + TASK_OFFSET);
        r.d0 = low_byte(r.d0, offset);
        if offset == 0xE0 {
            (r.d0, r.d6, r.a0, r.x, r.c) = self.r_00c86e(r.d5, r.d6, r.a6);
            return;
        }
        r.d0 = low_byte(r.d0, count);
        if count & 0x10 == 0 {
            r.d0 = low_byte(r.d0, count & 3);
            if count & 3 == 3 {
                self.write8(r.a6 + TASK_OFFSET, offset.wrapping_sub(1));
                let which = self.read8(BYTE_953A) as i8 as i16;
                r.d0 = low_word(r.d0, which as u16);
                let command = self.read8(RISE_SOUNDS.wrapping_add(which as u32));
                r.d0 = low_byte(r.d0, command);
                self.r_0034b6(r.d0);
                r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
                r.a5 = self.read32(SLOT_ADDR);
            }
        }
        (r.d0, r.x, r.c) = self.r_00c864(r.a5, r.a6);
    }

    /// `func_00C864` (`.L00C864`): `func_001B94`.  Gives back d0, X and
    /// C.
    pub fn r_00c864(&mut self, a5: u32, a6: u32) -> (u32, bool, bool) {
        self.r_001b94(a5, a6)
    }

    /// `func_00C86E` (`.L00C86E`): the cell two rows under the task's
    /// becomes 4, its second byte 0, and the task ends
    /// (`func_00121A`).  Gives back d0, d6, a0, X and C.
    pub fn r_00c86e(&mut self, d5: u32, d6: u32, a6: u32) -> (u32, u32, u32, bool, bool) {
        let d0 = self.r_0015f8(a6);
        let row = self.read8(CELL_Y);
        self.write8(CELL_Y, row.wrapping_add(2));
        let (a0, x) = self.r_00167a();
        self.write8(a0, 4);
        self.write8(a0.wrapping_add(SECOND), 0);
        let (d6, x, c) = self.r_00121a(d5, d6, x);
        (d0, d6, a0, x, c)
    }

    /// `func_00C886`: a task of `func_00C89E` made (`func_0003E8`) and
    /// bit 3 of `FLAGS` set.
    pub fn r_00c886(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d6, r.a6, r.x, _) = self.r_0003e8(r.d4, r.d5, r.d6, C886_PRIORITY, ROUTINE_C89E);
        let flags = self.read8(FLAGS);
        self.write8(FLAGS, flags | 0x08);
        r.c = false;
    }

    /// `func_00C89E`, a task's routine: its routine `func_00C8C8` from
    /// the next frame, `func_00A556`, `BYTE_A3F9` 0x7F unless
    /// `BYTE_953E` is 5, and `loc_00A686` with 4.
    pub fn r_00c89e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C8C8);
        self.r_00a556();
        if self.read8(BYTE_953E) != 5 {
            self.write8(BYTE_A3F9, 0x7F);
        }
        (r.d0, r.a1, r.x, r.c) = self.r_00a686(4, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00C8F4`: `func_00C904` with frame 0x37.
    pub fn r_00c8f4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(MADE_FRAME, 0x37);
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00c904(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00C8FE`: `func_00C904` with frame 0x36.
    pub fn r_00c8fe(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(MADE_FRAME, 0x36);
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00c904(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00C904` (`.L00C904`): a task of `func_00C938` (depth 3) at
    /// the position, its y 0x80.  Gives back d0 (0), d5, d6, a5, a6, X
    /// and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00c904(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let (d5, d6, a5, a6, x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 3, PRIORITY, ROUTINE_C938);
        self.r_00152a(a6);
        self.write16(a6 + TASK_Y, 0x80);
        let (d0, c) = self.r_00c922(d0, a5);
        (d0, d5, d6, a5, a6, x, c)
    }

    /// `func_00C922` (`.L00C922`): the sprite at a5's frame
    /// `MADE_FRAME`, its flags 4.  Gives back d0 (0) and C (clear).
    pub fn r_00c922(&mut self, d0: u32, a5: u32) -> (u32, bool) {
        let frame = self.read8(MADE_FRAME);
        self.write8(a5 + SPRITE_FRAME, frame);
        self.write8(a5 + SPRITE_FLAGS, 4);
        (low_byte(d0, 0), false)
    }

    /// `func_00C938`, a task's routine: its sprite placed
    /// (`func_001B7E`).
    pub fn r_00c938(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_00C93C`: `NEAREST` and `NEAR_BITS` cleared; two tasks of
    /// `func_00CAC2` (frame 0x31) at the position, the first 2 pixels
    /// up (its flag 0), the second 0x0F down (its flag 1); four of
    /// `func_00CAF0` (frame 0x33) 12 pixels apart to the right, each
    /// with its bit and highest offset (`BAR_BITS`, `BAR_MOST`); and
    /// `HOOK_A996` pointed at `func_00CA38`.  Gives back d0 (0x14) and C
    /// (clear).
    pub fn r_00c93c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(NEAREST, 0);
        self.write8(NEAR_BITS, 0);
        self.write8(MADE_FRAME, 0x31);
        self.l_make_at(r, 3, PRIORITY_CAC2, ROUTINE_CAC2);
        self.write8(r.a6 + TASK_COUNT, 0);
        let y = self.read8(r.a6 + TASK_Y_LOW);
        self.write8(r.a6 + TASK_Y_LOW, y.wrapping_sub(2));
        self.write8(MADE_FRAME, 0x31);
        self.l_make_at(r, 3, PRIORITY_CAC2, ROUTINE_CAC2);
        self.write8(r.a6 + TASK_COUNT, 1);
        let y = self.read8(r.a6 + TASK_Y_LOW);
        self.write8(r.a6 + TASK_Y_LOW, y.wrapping_add(0x11));
        r.d0 = low_byte(r.d0, 4);
        let kept_x = self.read16(POSITION_X);
        self.write8(LEFT, 4);
        self.write8(MADE_FRAME, 0x33);
        loop {
            self.l_make_at(r, 3, PRIORITY, ROUTINE_CAF0);
            let x = self.read16(POSITION_X);
            self.write16(POSITION_X, x.wrapping_add(0x0C));
            let left = self.read8(LEFT);
            r.d1 = low_byte(r.d1, left);
            let at = r.d1 as u16 as i16 as u32;
            let bit = self.read8(BAR_BITS.wrapping_add(at));
            self.write8(r.a6 + TASK_COUNT, bit);
            let most = self.read8(BAR_MOST.wrapping_add(at));
            self.write8(r.a6 + TASK_12, most);
            let left = self.read8(LEFT);
            self.write8(LEFT, left.wrapping_sub(1));
            (r.x, r.c) = (left == 0, left == 0);
            if left == 1 {
                break;
            }
        }
        self.write16(POSITION_X, kept_x);
        self.write32(HOOK_A996, NEARNESS);
        r.d0 = 0x14;
        r.c = false;
    }

    /// `func_00CA06`: the task at a6 put at the position
    /// (`func_00152A`) and its sprite given its frame
    /// (`func_00C922`).  Gives back d0 (0) and C (clear).
    pub fn r_00ca06(&mut self, d0: u32, a5: u32, a6: u32) -> (u32, bool) {
        self.r_00152a(a6);
        self.r_00c922(d0, a5)
    }

    /// `func_00CA0E`: `HOOK_A99A` pointed at `func_00CA38`.  Gives back
    /// d0 (0x24) and C (clear).
    pub fn r_00ca0e(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.write32(HOOK_A99A, NEARNESS);
        regs.d0 = 0x24;
        regs.c = false;
    }

    /// `func_00CA1E`: `HOOK_A99E` pointed at `func_00CA38`.  Gives back
    /// d0 (0x34 in its low byte) and C (clear).
    pub fn r_00ca1e(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.write32(HOOK_A99E, NEARNESS);
        regs.d0 = low_byte(regs.d0, 0x34);
        regs.c = false;
    }

    /// `func_00CA38`: for the object being run, unless bit 7 of its
    /// flags is set, its nearness (`OBJ_NEAR`) a step toward
    /// `NEAR_TABLE`'s for row `d0` (1-3) and the quarter of its byte
    /// 0x19's low nibble (0 at once when that is 0); a nearness as high
    /// as `NEAREST` becomes it, its bits (`NEAR_BITS_TABLE`) put in
    /// `NEAR_BITS`.  d1 is kept.  Gives back d0, d4, a4, X and C (set
    /// when lower than `NEAREST`).
    /// No script calls it: reviewed by reading.
    pub fn r_00ca38(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        let row = (r.d0 as u8).wrapping_sub(1).wrapping_mul(4);
        r.d0 = low_byte(r.d0, row);
        self.write8(ROW, row);
        let quarter = (self.read8(r.a4 + OBJ_19) & 0xF) >> 2;
        let (entry, carry) = quarter.overflowing_add(row);
        r.x = carry;
        let at = entry as i8 as i16 as u32;
        r.c = false;
        if (self.read8(r.a4 + OBJ_FLAGS) as i8) < 0 {
            return;
        }
        let want = self.read8(NEAR_TABLE.wrapping_add(at));
        r.d0 = low_byte(r.d0, want);
        if want == 0 {
            self.write8(r.a4 + OBJ_NEAR, 0);
        } else {
            let near = self.read8(r.a4 + OBJ_NEAR);
            if want > near {
                self.write8(r.a4 + OBJ_NEAR, near.wrapping_add(1));
                r.x = near == 0xFF;
            } else if want < near {
                self.write8(r.a4 + OBJ_NEAR, near.wrapping_sub(1));
                r.x = near == 0;
            }
        }
        let near = self.read8(r.a4 + OBJ_NEAR);
        r.d0 = low_byte(r.d0, near);
        if near < self.read8(NEAREST) {
            r.c = true;
            return;
        }
        self.write8(NEAREST, near);
        let bits = self.read8(NEAR_BITS_TABLE.wrapping_add(at));
        r.d0 = low_byte(r.d0, bits);
        let have = self.read8(NEAR_BITS);
        self.write8(NEAR_BITS, have | bits);
        r.c = false;
    }

    /// `func_00CAC2`, a task's routine: frame 0x31, or 0x32 when
    /// `NEAREST` is 4 or more; the first task (flag not 0) clears
    /// `NEAREST` and `NEAR_BITS`; then `loc_00CAE8`.
    pub fn r_00cac2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, 0x31);
        if self.read8(NEAREST) >= 4 {
            self.write8(r.a5 + SPRITE_FRAME, 0x32);
        }
        if self.read8(r.a6 + TASK_COUNT) != 0 {
            self.write8(NEAREST, 0);
            self.write8(NEAR_BITS, 0);
        }
        r.c = false;
        self.r_00cae8(r);
    }

    /// `loc_00CAE8`, a task's routine: its sprite placed
    /// (`func_001B7E`) and its depth by its y (`func_001B5C`).
    pub fn r_00cae8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_00CAF0`, a task's routine: its offset `NEAREST` (at most
    /// its byte 0x12) while its bit is in `NEAR_BITS`, else 0; its
    /// sprite placed (`func_001B94`, `func_001B5C`).
    pub fn r_00caf0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a6 + TASK_OFFSET, 0);
        let bit = self.read8(r.a6 + TASK_COUNT) & self.read8(NEAR_BITS);
        r.d0 = low_byte(r.d0, bit);
        if bit != 0 {
            let near = self.read8(NEAREST);
            r.d0 = low_byte(r.d0, near);
            let most = self.read8(r.a6 + TASK_12);
            self.write8(r.a6 + TASK_OFFSET, near.min(most));
        }
        (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_00CB22`: returns.
    pub fn r_00cb22(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `func_00CB2C`: `func_00C7E0` with frame 0x3A.
    pub fn r_00cb2c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(MADE_FRAME, 0x3A);
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00c7e0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00CB36`: `func_00C79C` with frame 0x3B.
    pub fn r_00cb36(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(MADE_FRAME, 0x3B);
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00c79c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00CB40`: nothing made.  Gives back d0 (0) and C (clear).
    pub fn r_00cb40(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        regs.d0 = low_byte(regs.d0, 0);
        regs.c = false;
    }

    /// `func_00CF82`: a task of `loc_00CAE8` (depth 0x10) at the
    /// position, its frame 0x39 for an even `d0`, 0x38 for an odd.
    /// Gives back d0 (0x26) and C (clear).
    pub fn r_00cf82(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let frame = ((r.d0 as u8 & 1) ^ 1).wrapping_add(0x38);
        r.d0 = low_byte(r.d0, frame);
        self.write8(MADE_FRAME, frame);
        self.l_make_at(r, 0x10, PRIORITY, ROUTINE_CAE8);
        r.d0 = low_byte(r.d0, 0x26);
        r.c = false;
    }

    /// `func_00CFB6`: nothing made.  Gives back d0 (0x16) and C (set).
    pub fn r_00cfb6(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        regs.d0 = low_byte(regs.d0, 0x16);
        regs.c = true;
    }

    /// `func_00CFC0`: a task of `func_00CFF8` (depth 0xF0, frame 0x4F)
    /// at the position, in the middle of its cell, its byte 0x12 4 and
    /// offset 0.  Gives back d0 (0x16) and C (clear).
    pub fn r_00cfc0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(MADE_FRAME, 0x4F);
        self.l_make_at(r, 0xF0, PRIORITY, ROUTINE_CFF8);
        self.r_00187c(r.a6);
        self.write8(r.a6 + TASK_12, 4);
        self.write8(r.a6 + TASK_OFFSET, 0);
        r.d0 = low_byte(r.d0, 0x16);
        r.c = false;
    }

    /// `func_00CFF8`, a task's routine: when an object of index below 6
    /// is at its cell (`func_00191C`) it is kept (`TASK_19`, `OTHER`)
    /// and set going: bit 7 of its flags, kind 8, its animation
    /// (`func_00130E`), timer 0 and step 2, its word 0x28 0xFC80, its
    /// routine `OBJECT_ROUTINE`; the Z80 gets 0x38 for object 0, 0x3E
    /// for another, and the task's count is 0x1A, its routine
    /// `func_00D07C` from the next frame.  Then `func_00D074`.
    pub fn r_00cff8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let found;
        (r.d0, found) = self.r_00191c(r.a6, r.x);
        r.c = found;
        let object = r.d0 as u8;
        if found && object < 6 {
            self.write8(r.a6 + TASK_19, object);
            self.write8(OTHER, object);
            r.d4 = low_byte(r.d4, object);
            (r.d4, r.a4, r.x) = self.r_001cfa(r.d4);
            let flags = self.read8(r.a4 + OBJ_FLAGS);
            self.write8(r.a4 + OBJ_FLAGS, flags | 0x80);
            self.write8(r.a4 + OBJ_KIND, 8);
            (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
            self.write8(r.a4 + OBJ_TIMER, 0);
            self.write8(r.a4 + OBJ_STEP, 2);
            self.write16(r.a4 + OBJ_RATE, 0xFC80);
            self.write32(r.a4 + OBJ_ROUTINE, OBJECT_ROUTINE);
            let command = if object == 0 { 0x38 } else { 0x3E };
            r.d0 = low_byte(r.d0, command);
            self.r_0034b6(r.d0);
            let current = self.read8(CURRENT_TASK);
            (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
            self.write8(r.a6 + TASK_COUNT, 0x1A);
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_D07C);
        }
        (r.d0, r.x, r.c) = self.r_00d074(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
    }
}
