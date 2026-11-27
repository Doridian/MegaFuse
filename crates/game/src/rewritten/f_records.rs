//! The records at `RECORDS`: 30 of 4 bytes, each a kind (0x80 plus
//! its number; 0 when free), a task's index (kind `WITH_TASK` only) and
//! a cell (column, row; a word).  `func_00644A` puts them under the
//! cells holding 2 at random; a cell holding 5 shows its record's kind
//! (`func_00667E`), which `func_006948` hands to one of the routines of
//! `KIND_ROUTINES` (`f_kinds.rs`).

use crate::{Game, Regs};

/// The records, `RECORD_SIZE` bytes each, `RECORDS_END` bytes in all.
const RECORDS: u32 = 0xFFFF_AC00;
const RECORD_SIZE: u16 = 4;
const RECORDS_END: u16 = 0x78;
const REC_TASK: u32 = 1;
const REC_CELL: u32 = 2;
/// A kind's mark, and the kind whose record keeps a task.
const KIND_MARK: u8 = 0x80;
const WITH_TASK: u8 = 0x8E;

/// The cells holding 2 (`func_00644A`): columns and rows, a byte each,
/// and how many (also the records' count as they are placed).
const FREE_X: u32 = 0xFFFF_9F1A;
const FREE_Y: u32 = 0xFFFF_A01A;
const FREE_COUNT: u32 = 0xFFAC78;
/// The list of kinds to place: pairs of a kind and how many, ended by a
/// byte with bit 7 set (a long; `func_00644A`).
const TO_PLACE: u32 = 0xFF80D0;
/// Where the list is made (from `DEFAULT_LIST`), its fifth byte (the
/// third kind) from `PLACE_BY_SETTING`.
const LIST: u32 = 0xFFFF_9550;
const LIST_THIRD_KIND: u32 = 0xFF9554;
const DEFAULT_LIST: u32 = 0x0065AE;
const DEFAULT_LIST_LEN: u32 = 7;
/// A byte (`SETTING`) and a table by it (D_015C26) for the list's
/// third kind.
const SETTING: u32 = 0xFF953F;
const PLACE_BY_SETTING: u32 = 0x015C26;
/// Lists of kinds to place by `LIST_INDEX` (D_006578, longs), used
/// when bit 0 of `FLAGS` is set.  (Its eighth entry, 0x65AC, is written
/// as bytes in the disassembly: the end of D_0065A2, an empty list.)
const LISTS: u32 = 0x006578;
const LIST_INDEX: u32 = 0xFFAD3F;
/// Flags: bit 0 picks the lists.
const FLAGS: u32 = 0xFF9540;
/// The random numbers' state's high byte (`a_random.rs`), stirred
/// after a number is taken.
const RANDOM_STATE: u32 = 0xFFA400;

/// A cell (column, row; a word together), and the map's size.
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
const WIDTH: u32 = 0xFF9EB2;
const HEIGHT: u32 = 0xFF9EB3;

/// The routine kind `WITH_TASK`'s task runs, and its sprite's
/// animation (D_00667A).
const WITH_TASK_ROUTINE: u32 = 0x006652;
const WITH_TASK_ANIM: u32 = 0x00667A;
/// Bytes by kind for the word `func_00667E` draws (the table is read
/// with the kind's mark, so its entries are from 0x66B0).
const KIND_DRAWN: u32 = 0x006630;

/// The routine of the task `func_006766` makes, and its animations
/// (by `MODE`).
const GONE_ROUTINE: u32 = 0x0067B4;
const GONE_ANIM: u32 = 0x0DA5C6;
const GONE_ANIM_MODE_2: u32 = 0x0DA51E;
const MODE: u32 = 0xFF953A;

/// What `func_006802` places from an object's counts (kinds 0, 1, 11;
/// kind 9 for bit 3 of its flags).
const OBJ_FLAGS: u32 = 0x10;
const OBJ_FLAG_9: u8 = 1 << 3;
const OBJ_COUNTS: [(u32, u8); 3] = [(0x36, 0), (0x37, 1), (0x38, 0x0B)];
const FLAG_KIND: u8 = 9;
/// The kind `func_00685A` places, and how many cells it has still to
/// look at.
const PLACING: u32 = 0xFFAC7A;
/// The cell value `func_00685A` places on.
const PLACE_ON: u8 = 6;
/// The routine of the task `func_00685A` makes, and the animation and
/// Z80 command of `func_006918`.
const PLACED_ROUTINE: u32 = 0x00693A;
const PLACED_ANIM: u32 = 0x0DA4BA;
const PLACED_COMMAND: u8 = 0x2C;

/// The record `func_006948` found (its offset), the object being run
/// (its index and address) and the routines by kind (D_0069B2).
const FOUND: u32 = 0xFFAC7B;
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const KIND_ROUTINES: u32 = 0x0069B2;
/// From which object being run the kind `WITH_TASK` is left.
const RUNNING_LIMIT: u8 = 5;

/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;

/// A task's fields: its sprite's index, its animation's timer and
/// step, its animation (a long).
const TASK_SPRITE: u32 = 0x03;
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's flags.
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_Y_LOW: u32 = 0x0B;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

fn record(offset: u32) -> u32 {
    RECORDS.wrapping_add((offset as u16) as i16 as i32 as u32)
}

/// The depth byte the tasks' sprites get from their y (`func_0007D2`).
fn depth_from_y(y: u8, by: u8) -> (u8, bool) {
    let (less, borrow) = y.overflowing_sub(by);
    ((!less & 0xFE) | 2, borrow)
}

impl Game {
    /// `func_00644A`: the records cleared, the cells holding 2 listed
    /// (`FREE_X`, `FREE_Y`), and the kinds of the list at `TO_PLACE`
    /// (made from `DEFAULT_LIST` unless bit 0 of `FLAGS` picks one of
    /// `LISTS`) placed in records, each under one of the listed cells
    /// taken at random (`func_006540`).  Gives back d0, d1, d2, d3, a0
    /// (after the list), a1 (after the last record), a2 and X.
    #[allow(clippy::type_complexity)]
    pub fn r_00644a(&mut self, d1: u32, d2: u32, d3: u32) -> (u32, u32, u32, u32, u32, u32, u32, bool) {
        for i in 0..u32::from(RECORDS_END) {
            self.write8(RECORDS + i, 0);
        }
        let mut d0: u32 = 0xFFFF;
        self.r_0010ec();
        let mut count: u16 = 0;
        let mut x;
        let mut row = 0u8;
        loop {
            self.write8(CELL_Y, row);
            let mut column = 0u8;
            loop {
                self.write8(CELL_X, column);
                let (cell, _) = self.r_00167a();
                if self.read8(cell) == 2 {
                    let at = count as i16 as i32 as u32;
                    let value = self.read8(CELL_X);
                    self.write8(FREE_X.wrapping_add(at), value);
                    let value = self.read8(CELL_Y);
                    self.write8(FREE_Y.wrapping_add(at), value);
                    count = count.wrapping_add(1);
                }
                column = self.read8(CELL_X).wrapping_add(1);
                if column == self.read8(WIDTH) {
                    break;
                }
            }
            let (next, carry) = self.read8(CELL_Y).overflowing_add(1);
            row = next;
            x = carry;
            if row == self.read8(HEIGHT) {
                break;
            }
        }
        d0 = low_byte(d0, row);
        self.write8(FREE_COUNT, count as u8);
        let mut d2 = low_word(d2, count);
        let mut a0;
        if self.read8(FLAGS) & 1 == 0 {
            for i in 0..DEFAULT_LIST_LEN {
                let value = self.read8(DEFAULT_LIST + i);
                self.write8(LIST + i, value);
            }
            d0 = 0xFFFF;
            self.write32(TO_PLACE, LIST);
            let setting = self.read8(SETTING);
            d2 = low_word(d2, u16::from(setting));
            let value = self.read8(PLACE_BY_SETTING + u32::from(setting));
            self.write8(LIST_THIRD_KIND, value);
        } else {
            let (index, carry) = self.read8(LIST_INDEX).wrapping_mul(2).overflowing_add(self.read8(LIST_INDEX).wrapping_mul(2));
            x = carry;
            let index = index as i8 as i16;
            d2 = low_word(d2, index as u16);
            let value = self.read32(LISTS.wrapping_add(i32::from(index) as u32));
            self.write32(TO_PLACE, value);
        }
        let mut d1 = d1;
        let mut d3 = d3;
        let mut at = RECORDS;
        a0 = self.read32(TO_PLACE);
        loop {
            let kind = self.read8(a0);
            a0 = a0.wrapping_add(1);
            d2 = low_byte(d2, kind);
            if kind & 0x80 != 0 {
                return (d0, d1, d2, d3, a0, at, FREE_Y, x);
            }
            let mark = kind | KIND_MARK;
            d2 = low_byte(d2, mark);
            let mut times = self.read8(a0);
            a0 = a0.wrapping_add(1);
            loop {
                let free = self.read8(FREE_COUNT);
                if free != 0 {
                    (d0, d1, _) = self.r_001be2(d1);
                    let value = self.read8(RANDOM_STATE).wrapping_add(1);
                    self.write8(RANDOM_STATE, value);
                    let mut pick = d0 as u8;
                    loop {
                        pick = pick.wrapping_sub(self.read8(FREE_COUNT));
                        if pick < self.read8(FREE_COUNT) {
                            break;
                        }
                    }
                    d0 = low_byte(d0, pick);
                }
                d0 = self.r_006540(d0);
                let value = self.read16(CELL_X);
                self.write16(at + REC_CELL, value);
                self.write8(at, mark);
                at += u32::from(RECORD_SIZE);
                x = times == 0;
                times = times.wrapping_sub(1);
                d3 = low_byte(d3, times);
                if times == 0 {
                    break;
                }
            }
        }
    }

    /// `func_006540`: the listed cell `d0` (its low byte) taken out of
    /// `FREE_X`, `FREE_Y` to `CELL_X`, `CELL_Y`, the entry at
    /// `FREE_COUNT` (one past the last) put in its place and the count
    /// one less.  Gives
    /// back d0 (the low byte, a word).
    pub fn r_006540(&mut self, d0: u32) -> u32 {
        let d0 = d0 & 0xFFFF_00FF;
        let taken = d0 & 0xFF;
        let value = self.read8(FREE_X + taken);
        self.write8(CELL_X, value);
        let value = self.read8(FREE_Y + taken);
        self.write8(CELL_Y, value);
        let last = u32::from(self.read8(FREE_COUNT));
        let value = self.read8(FREE_X + last);
        self.write8(FREE_X + taken, value);
        let value = self.read8(FREE_Y + last);
        self.write8(FREE_Y + taken, value);
        let value = self.read8(FREE_COUNT).wrapping_sub(1);
        self.write8(FREE_COUNT, value);
        d0
    }

    /// `func_0065B6`: whether a record is at `CELL_X`, `CELL_Y` (C;
    /// `func_0066BE`).  If so the cell shows it: kind `WITH_TASK` gets
    /// a task running `WITH_TASK_ROUTINE` (run once now, as the slot's
    /// routine) whose index the record keeps, the cell 5
    /// (`func_0019A2`); the others are drawn (`func_00667E`).  Gives
    /// back d0, d2, d5, d6, a5, a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0065b6(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        let (offset, x, found) = self.r_0066be(d1, d2);
        if !found {
            return (d0, d2, d5, d6, a5, a6, x, false);
        }
        if self.read8(record(offset)) != WITH_TASK {
            let (d0, _, _, x) = self.r_00667e(d0, offset, RECORDS);
            return (d0, d2, d5, d6, a5, a6, x, true);
        }
        let a0 = self.r_0019a2();
        self.r_001278();
        self.r_0052f0(d0);
        // whether a task could be had isn't looked at
        let (d5, d6, a5, a6, _, _) = self.r_001222(d0, offset, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0x80, 0x3F, WITH_TASK_ROUTINE);
        let (d0, x) = self.r_001550(d0, a6);
        self.write32(a6 + TASK_ANIM, WITH_TASK_ANIM);
        self.write8(a5 + SPRITE_FLAGS, 4);
        let slot_index = self.read8(SLOT_INDEX);
        let slot_addr = self.read32(SLOT_ADDR);
        self.write8(SLOT_INDEX, d5 as u8);
        self.write32(SLOT_ADDR, a5);
        let mut regs = Regs { d0, d1: offset, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c: false, ..Regs::default() };
        self.r_006652(&mut regs);
        self.write32(SLOT_ADDR, slot_addr);
        self.write8(SLOT_INDEX, slot_index);
        self.write8(record(offset) + REC_TASK, regs.d6 as u8);
        (regs.d0, regs.d2, regs.d5, regs.d6, regs.a5, regs.a6, regs.x, true)
    }

    /// `func_006652`, a task's routine: the task's sprite moved on
    /// (`func_0013D6`) and placed by its y (`func_0007D2`), or the task
    /// ended (`func_00121A`) when that is over.
    pub fn r_006652(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, ended) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if ended {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        } else {
            (r.d0, _, _) = self.r_001b7e(r.a5, r.a6);
            let (depth, borrow) = depth_from_y(self.read8(r.a5 + SPRITE_Y_LOW), 0x11);
            r.d0 = low_byte(r.d0, depth);
            r.x = borrow;
            r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
        }
        r.unwound = false;
    }

    /// `func_00667E`: the record at offset `d1`'s kind drawn at its cell
    /// (`func_005294`; the word from `KIND_DRAWN` and the kind), the
    /// cell 5 (`func_0019A2`).  Gives back d0 (the word), d1, a0 and X.
    pub fn r_00667e(&mut self, d0: u32, d1: u32, a0: u32) -> (u32, u32, u32, bool) {
        self.r_0019a2();
        let kind = self.read8(record(d1));
        let low = kind.wrapping_mul(4) | 0x80;
        let high = self.read8(KIND_DRAWN + u32::from(kind));
        let d0 = low_word(d0, u16::from_be_bytes([high, low]));
        self.r_001278();
        let (x, _) = self.r_005294(d0, d1);
        (d0, d1, a0, x)
    }

    /// `func_0066BE`: the record at `CELL_X`, `CELL_Y` (C set when there
    /// is one).  Gives back d1 (its offset, or `RECORDS_END`), X and C.
    pub fn r_0066be(&mut self, d1: u32, d2: u32) -> (u32, bool, bool) {
        let cell = self.read16(CELL_X);
        // asl.w #8 of the column's word
        let x = d2 & 0x100 != 0;
        let mut offset: u16 = 0;
        loop {
            let at = record(u32::from(offset));
            if self.read8(at) != 0 && self.read16(at + REC_CELL) == cell {
                return (low_word(d1, offset), offset == 0 && x, true);
            }
            offset += RECORD_SIZE;
            if offset == RECORDS_END {
                return (low_word(d1, offset), false, false);
            }
        }
    }

    /// `func_0066FE`: the record at `CELL_X`, `CELL_Y`, if any, taken:
    /// its task (kind `WITH_TASK`) ended with its sprite
    /// (`func_000870`, `func_000564`), the cell's byte split into the
    /// cell map again and drawn as it is (`func_00168C`, `func_0052F0`),
    /// and a task made to show it gone (`func_006766`).  Gives back d2
    /// and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0066fe(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, bool) {
        let (offset, x, found) = self.r_0066be(d1, d2);
        if !found {
            return (d2, x);
        }
        let at = record(offset);
        let kind = self.read8(at);
        let mut d0 = low_byte(d0, kind);
        let mut d6 = d6;
        self.write8(at, 0);
        if kind == WITH_TASK {
            let task = self.read8(at + REC_TASK);
            d6 = low_byte(d6, task);
            if task != 0 {
                let (index, a6, _) = self.r_0005f8(d6);
                d6 = index;
                self.write8(at + REC_TASK, 0);
                d0 = low_byte(d0, self.read8(a6 + TASK_SPRITE));
                let slot_index = self.read8(SLOT_INDEX);
                let slot_addr = self.read32(SLOT_ADDR);
                let x = self.r_000870(d0, d5);
                self.write32(SLOT_ADDR, slot_addr);
                self.write8(SLOT_INDEX, slot_index);
                self.r_000564(d5, d6, x);
            }
        }
        let (d0, a0) = self.r_00168c(d0);
        self.r_001278();
        self.r_0052f0(d0);
        let (_, _, d2, _, x, _) = self.r_006766(d0, offset, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        (d2, x)
    }

    /// `func_006766`: a task running `GONE_ROUTINE` with a sprite
    /// (`func_001222`), its animation `GONE_ANIM` (`GONE_ANIM_MODE_2`
    /// when `MODE` is 2), run on once (`func_0013D6`).  Keeps d5, d6,
    /// a5, a6; gives back d0, d1, d2, a0, X and C (set when no task
    /// could be had).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_006766(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        let (d5, _, a5, a6, x, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0x20, 0xD0, GONE_ROUTINE);
        if failed {
            return (d0, d1, d2, a0, x, true);
        }
        self.r_001550(d0, a6);
        self.write8(a5 + SPRITE_FLAGS, 4);
        let anim = if self.read8(MODE) == 2 { GONE_ANIM_MODE_2 } else { GONE_ANIM };
        self.write32(a6 + TASK_ANIM, anim);
        let (d0, _, _) = self.r_001b7e(a5, a6);
        let (d0, d1, d2, _, a0, _, x, c) = self.r_0013d6(d0, d1, d2, d5, a6);
        (d0, d1, d2, a0, x, c)
    }

    /// `func_0067B4`, a task's routine: the sprite moved on
    /// (`func_0013D6`, `func_001B7E`, `func_001B5C`), or the task ended
    /// (`func_00121A`) when that is over.
    pub fn r_0067b4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, ended) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if ended {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        } else {
            (r.d0, _, _) = self.r_001b7e(r.a5, r.a6);
            (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        }
        r.unwound = false;
    }

    /// `func_0067C8`: the record at `CELL_X`, `CELL_Y` taken unless it is
    /// of kind `WITH_TASK` (C set then), the cell's byte split into the
    /// cell map again and drawn (`func_00168C`, `func_0052F0`).  With no
    /// record there it looks at the byte after the records
    /// (`FREE_COUNT`) instead.  Gives back d0, X and C.
    pub fn r_0067c8(&mut self, d0: u32, d1: u32, d2: u32) -> (u32, bool, bool) {
        let (offset, x, _) = self.r_0066be(d1, d2);
        let at = record(offset);
        if self.read8(at) == WITH_TASK {
            return (d0, x, true);
        }
        self.write8(at, 0);
        let (d0, _) = self.r_00168c(d0);
        self.r_001278();
        let (x, _) = self.r_0052f0(d0);
        (d0, x, false)
    }

    /// `func_006802`: kinds placed for the object at a4's counts (each
    /// counted down to 0, `func_00685A` for each), and kind `FLAG_KIND`
    /// when bit 3 of its flags is set.
    #[allow(clippy::too_many_arguments)]
    pub fn r_006802(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) {
        for (field, kind) in OBJ_COUNTS {
            while self.read8(a4 + field) != 0 {
                let value = self.read8(a4 + field) - 1;
                self.write8(a4 + field, value);
                self.r_00685a(u32::from(kind), d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
            }
        }
        if self.read8(a4 + OBJ_FLAGS) & OBJ_FLAG_9 != 0 {
            self.r_00685a(u32::from(FLAG_KIND), d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        }
    }

    /// `func_00685A`: kind `d0` placed on a cell holding `PLACE_ON`,
    /// looked for from a random one (columns 2-14, rows 1-11) on, at
    /// most 255 cells: a record for it in the first free one, the kind
    /// drawn (`func_00667E`) and a task made for it (`func_006918`).
    /// Keeps every register.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00685a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) {
        self.write8(PLACING, d0 as u8);
        self.write8(FREE_COUNT, 0);
        let d0 = self.r_001bc0(0x0C, d1);
        let value = self.read8(RANDOM_STATE).wrapping_add(1);
        self.write8(RANDOM_STATE, value);
        self.write8(CELL_X, (d0 as u8).wrapping_add(2));
        let mut d0 = self.r_001bc0(0x0A, d1);
        d0 = low_byte(d0, (d0 as u8).wrapping_add(1));
        'rows: loop {
            self.write8(CELL_Y, d0 as u8);
            d0 = low_byte(d0, self.read8(CELL_X));
            loop {
                self.write8(CELL_X, d0 as u8);
                let (cell, _) = self.r_00167a();
                let held = self.read8(cell);
                d0 = low_byte(d0, held);
                if held == PLACE_ON {
                    break 'rows;
                }
                let left = self.read8(FREE_COUNT).wrapping_sub(1);
                self.write8(FREE_COUNT, left);
                if left == 0 {
                    return;
                }
                let column = self.read8(CELL_X).wrapping_add(1);
                d0 = low_byte(d0, column);
                if column == 0x0F {
                    self.write8(CELL_X, 2);
                    let row = self.read8(CELL_Y).wrapping_add(1);
                    d0 = if row == 0x0C { 1 } else { low_byte(d0, row) };
                    continue 'rows;
                }
            }
        }
        let mut offset: u16 = 0;
        while self.read8(record(u32::from(offset))) != 0 {
            offset = offset.wrapping_add(RECORD_SIZE);
        }
        let at = record(u32::from(offset));
        let d0 = low_byte(d0, self.read8(PLACING) | KIND_MARK);
        self.write8(at, d0 as u8);
        let value = self.read16(CELL_X);
        self.write16(at + REC_CELL, value);
        let d1 = low_word(d1, offset);
        let (d0, d1, a0, _) = self.r_00667e(d0, d1, RECORDS);
        let (d5, d6, a5, a6, _, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0xF0, 0xC0, PLACED_ROUTINE);
        if !failed {
            self.r_001550(d0, a6);
            self.r_006918(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6);
        }
    }

    /// `func_006918`: the Z80 sent `PLACED_COMMAND`, the task's sprite
    /// flags 0x84, its animation `PLACED_ANIM` from the start, then run
    /// once (`func_00693A`).  Gives back d0, d1, d2.
    #[allow(clippy::too_many_arguments)]
    pub fn r_006918(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32) {
        self.r_0034b6(u32::from(PLACED_COMMAND));
        self.write8(a5 + SPRITE_FLAGS, 0x84);
        self.write8(a6 + TASK_TIMER, 0);
        self.write8(a6 + TASK_STEP, 0);
        self.write32(a6 + TASK_ANIM, PLACED_ANIM);
        let (d0, x, c) = self.r_001b7e(a5, a6);
        let mut regs = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c, ..Regs::default() };
        self.r_00693a(&mut regs);
        (regs.d0, regs.d1, regs.d2)
    }

    /// `func_00693A`, a task's routine: the sprite moved on
    /// (`func_0013D6`), the task ended (`func_00121A`) when that is
    /// over.
    pub fn r_00693a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, ended) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        r.c = ended;
        if ended {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        }
        r.unwound = false;
    }

    /// `func_006948`: if the cell at `CELL_X`, `CELL_Y` holds 5, its
    /// record taken (kind `WITH_TASK` is left while the object being
    /// run is `RUNNING_LIMIT` or more), the cell's byte split into the
    /// cell map again and drawn, and the kind's routine of
    /// `KIND_ROUTINES` run for the object being run, the Z80 sent the
    /// command it gives back (d0).  Gives back d0, d1, d2, d4 and a4
    /// (the object being run), d5, d6, a0, a5, a6.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_006948(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let (cell, _) = self.r_00167a();
        let held = self.read8(cell);
        let d0 = low_byte(d0, held);
        if held != 5 {
            return (d0, d1, d2, d4, d5, d6, cell, a4, a5, a6);
        }
        let (offset, x, _) = self.r_0066be(d1, d2);
        self.write8(FOUND, offset as u8);
        let at = record(offset);
        let mut r = Regs { d0, d1: offset, d2, d3, d4, d5, d6, d7, a0: RECORDS, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        if self.read8(at) != WITH_TASK || self.read8(RUNNING) < RUNNING_LIMIT {
            let (d0, _) = self.r_00168c(r.d0);
            self.r_001278();
            (r.x, _) = self.r_0052f0(d0);
            let kind = self.read8(at);
            self.write8(at, 0);
            let index = kind.wrapping_mul(4) as i8 as i16;
            r.d0 = low_word(d0, index as u16);
            let routine = self.read32(KIND_ROUTINES.wrapping_add(i32::from(index) as u32));
            r.a0 = routine;
            r.d4 = low_byte(r.d4, self.read8(RUNNING));
            r.a4 = self.read32(RUNNING_ADDR);
            self.call_family_6(routine, &mut r);
            if r.unwound {
                return (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, a4, r.a5, r.a6);
            }
            self.r_0034b6(r.d0);
        }
        let d4 = low_byte(r.d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        (r.d0, r.d1, r.d2, d4, r.d5, r.d6, r.a0, a4, r.a5, r.a6)
    }

    /// `func_006CD2`: a task running `PLACED_ROUTINE` with a sprite
    /// (`func_001222`, `func_00152A`), set up by `func_006918`.  Keeps
    /// d4, d5, d6, a4, a5, a6; gives back d0, d1, d2.
    #[allow(clippy::too_many_arguments)]
    pub fn r_006cd2(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32) {
        let (d5, d6, a5, a6, _, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0x10, 0xC0, PLACED_ROUTINE);
        if failed {
            return (d0, d1, d2);
        }
        self.r_00152a(a6);
        self.r_006918(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6)
    }
}
