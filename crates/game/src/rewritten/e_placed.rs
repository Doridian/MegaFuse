//! Tasks an object puts at cells (`func_0058F0`: their routine is
//! `func_005D46`, `e_placed_task.rs`), and the table that keeps them by
//! object (`PLACED`): finding one at a cell, taking one out.
//!
//! The object is `CHOSEN`'s; while these run, the object whose turn it
//! is (`RUNNING`, `RUNNING_ADDR`) is put back in d4 and a4 at the end.

use crate::{Game, Regs};

/// Each object's tasks put at cells: 0x40 bytes an object, entries of 8
/// bytes (the task's index, 0 for none, then its address at
/// `PLACED_TASK`), 0x28 entries in all.
const PLACED: u32 = 0xFFFF_AAC0;
const PLACED_TASK: u32 = 2;
const PLACED_ENTRY: u16 = 8;
const PLACED_END: u16 = 0x140;
/// `func_005A0C` packs 8 entries of an object's, or 10 (into the next
/// object's) when bit 0 of `SETTINGS` is clear.
const SETTINGS: u32 = 0xFF9540;

/// The object the routines here work for (its index, a byte), and the
/// one whose turn it is (`func_001CA0`; index and address).
const CHOSEN: u32 = 0xFFA93F;
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
/// Bits that `func_005776` tests: 0xFFA933's bit 6 (the first task
/// put gets `TASK_TIMER` 1 when the object's flag bit 2 is set too),
/// bit 5 (go on whatever the object's `OBJ_31`), and 0xFFA932's bit 5.
const BITS_A932: u32 = 0xFFA932;
const BITS_A933: u32 = 0xFFA933;
/// What a new task's `TASK_17` and `TASK_TIMER` (when its cell's byte
/// is below 7) are set to.
const NEW_TASK_17: u32 = 0xFFA934;
const NEW_TASK_TIMER: u32 = 0xFFA935;
/// A byte by object index: 8 or 9 sets a new task's flag bit 3 or 4.
const BY_OBJECT: u32 = 0xFFFF_9E92;
/// A direction (0 up, 1 right, 2 down, 3 left; a byte) the routines are
/// given in d0.
const DIRECTION: u32 = 0xFFAABD;

/// The running task's index.
const CURRENT_TASK: u32 = 0xFF80EE;
/// The slot (`func_000B5A`'s records) the running task has: index and
/// address.
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// A position (words, x then y) and a cell (column, row; one word).
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
const CELL: u32 = 0xFF9F08;
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
/// The cell's address in the cell map (`func_00167A`).
const CELL_AT: u32 = 0xFF9ED6;

/// An object's fields: flags, a direction (low 2 bits), its position
/// (words, x then y), how many tasks it has put and how many it may, a
/// byte set to `OBJ_31_SET` when it puts one.
const OBJ_FLAGS: u32 = 0x10;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_X: u32 = 0x18;
const OBJ_Y: u32 = 0x1A;
const OBJ_PUT: u32 = 0x2F;
const OBJ_MAY_PUT: u32 = 0x30;
const OBJ_31: u32 = 0x31;
const OBJ_31_SET: u8 = 0x0B;

/// A put task's fields: the object that moved it last (an index), a
/// direction, a count of frames, a byte for `0xFFAABE`, flags (bits
/// 0-2 its way of moving, `func_005E50`; bit 5: it is followed, bit 6:
/// its sprite clears the cell), the object that put it, its cell
/// (column, row) or a word of rate, and its offset.
const TASK_MOVED_BY: u32 = 0x14;
const TASK_DIRECTION: u32 = 0x15;
const TASK_TIMER: u32 = 0x16;
const TASK_17: u32 = 0x17;
const TASK_FLAGS: u32 = 0x18;
const TASK_OWNER: u32 = 0x19;
const TASK_CELL: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;
const TASK_OFFSET_X: u32 = 0x21;
/// A sprite record's flags.
const SPRITE_FLAGS: u32 = 0x05;

/// The task `func_0058F0` makes: routine, priority and depth; its
/// sprite's flags; the sound (`func_0034B6`).
const PUT_TASK: u32 = 0x005D46;
const PUT_TASK_PRIORITY: u8 = 0x41;
const PUT_TASK_DEPTH: u8 = 0xF8;
const PUT_TASK_SPRITE_FLAGS: u8 = 4;
const PUT_SOUND: u8 = 0x30;
/// The routines that move `CELL` and `POSITION` a cell by direction
/// (`D_0058B0`, longs in the ROM).
const STEPS: u32 = 0x0058B0;
/// What `func_005BD2` makes the task do next (`func_005C10`), and the
/// sound `func_005B8C` makes then.
const THROWN: u32 = 0x005C10;
const THROWN_RATE: u16 = 0xFC00;
const THROWN_SOUND: u8 = 0x40;
/// The routine `func_005E0E` gives the task (`func_005E02`).
const STOPPED: u32 = 0x005E02;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// d4 and a4: the object whose turn it is.
    fn e_running(&mut self, d4: u32) -> (u32, u32) {
        let index = self.read8(RUNNING);
        (low_byte(d4, index), self.read32(RUNNING_ADDR))
    }

    /// The `CHOSEN` object (`func_001CFA`).  Gives back d4 and a4.
    fn e_chosen(&mut self, d4: u32) -> (u32, u32) {
        let index = self.read8(CHOSEN);
        let (d4, object, _) = self.r_001cfa(low_byte(d4, index));
        (d4, object)
    }

    /// The running object's cell (by the middle of it) to `POSITION`,
    /// `CELL`.  Gives back d0 and a0 (the cell's address).
    fn e_running_cell(&mut self, d0: u32, object: u32) -> (u32, u32) {
        let mut d0 = d0;
        for (field, to) in [(OBJ_X, POSITION_X), (OBJ_Y, POSITION_Y)] {
            let middle = (self.read16(object + field) & 0xFFF0) | 8;
            d0 = low_word(d0, middle);
            self.write16(to, middle);
        }
        let d0 = self.r_0015d4();
        let (cell, _) = self.r_00167a();
        let byte = self.read8(cell);
        (low_byte(d0, byte), cell)
    }

    /// `func_005776`: the `CHOSEN` object puts a task at its cell
    /// (`func_0058F0`) if the cell's byte is not 1; on a 1, with
    /// `BITS_A933` bit 5 and its flag bit 3 set, a line of them a cell
    /// at a time in its direction (`func_005892`) until a cell has an
    /// object or none can be put.  Gives back d0, d4, d5, d6, a0, a4
    /// and a6.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_005776(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        let (mut d4, mut d6, mut a6) = (d4, d6, a6);
        if self.read8(BITS_A933) & 0x40 != 0 {
            let object;
            (d4, object) = self.e_chosen(d4);
            if self.read8(object + OBJ_FLAGS) & 0x04 != 0 && self.read8(object + OBJ_PUT) != 0 {
                let first = self.read8(PLACED);
                d6 = low_byte(d6, first);
                a6 = self.read32(PLACED + PLACED_TASK);
                self.write8(a6 + TASK_TIMER, 1);
            }
        }
        if self.read8(BITS_A933) & 0x20 == 0 {
            let object;
            (d4, object) = self.e_chosen(d4);
            if self.read8(object + OBJ_31) != 0 || self.read8(BITS_A932) & 0x20 == 0 {
                let (d4, a4) = self.e_running(d4);
                return (d0, d4, d5, d6, a0, a4, a6);
            }
        }
        let (d4, object) = self.e_running(d4);
        let (d0, cell) = self.e_running_cell(d0, object);
        if d0 as u8 != 1 {
            let slot = self.read8(SLOT_INDEX);
            let slot_addr = self.read32(SLOT_ADDR);
            let (d0, d4, d5, d6, a0, a4, _, a6, _) = self.r_0058f0(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a5, a6);
            self.write32(SLOT_ADDR, slot_addr);
            self.write8(SLOT_INDEX, slot);
            return (d0, d4, d5, d6, a0, a4, a6);
        }
        let mut d0 = d0;
        if self.read8(BITS_A933) & 0x20 != 0 {
            let (d4, object) = self.e_chosen(d4);
            let flag = self.read8(object + OBJ_FLAGS) & 0x08;
            d0 = low_byte(d0, flag);
            if flag != 0 {
                let (d4, a4) = self.e_running(d4);
                return self.e_put_line(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6);
            }
            let (d4, a4) = self.e_running(d4);
            return (d0, d4, d5, d6, cell, a4, a6);
        }
        let (d4, a4) = self.e_running(d4);
        (d0, d4, d5, d6, cell, a4, a6)
    }

    /// `func_005776`'s line of tasks: a cell on in the object's
    /// direction (`func_005892`) and one put there (`func_0058F0`) until
    /// the cell has an object (`func_0018F8`) or none can be put.  The
    /// cell and the slot are kept.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn e_put_line(&mut self, _d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        let cell = self.read16(CELL);
        let slot = self.read8(SLOT_INDEX);
        let slot_addr = self.read32(SLOT_ADDR);
        let (mut d4, mut d5, mut d6, mut a4, mut a5, mut a6) = (d4, d5, d6, a4, a5, a6);
        let (mut d0, mut a0);
        loop {
            let x;
            (d4, a0, a4, x) = self.r_005892(d4, a4);
            let taken;
            (d0, taken) = self.r_0018f8(x);
            if taken {
                break;
            }
            let failed;
            (d0, d4, d5, d6, a0, a4, a5, a6, failed) = self.r_0058f0(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a5, a6);
            if failed {
                break;
            }
        }
        self.write32(SLOT_ADDR, slot_addr);
        self.write8(SLOT_INDEX, slot);
        self.write16(CELL, cell);
        (d0, d4, d5, d6, a0, a4, a6)
    }

    /// `func_005892`: `CELL` and `POSITION` a cell on in the direction
    /// of the object at a4 (`STEPS`); then d4, a4 the running object
    /// again.  Gives back d4, a0 (the step's routine), a4 and X.
    /// No script calls it: reviewed by reading.
    pub fn r_005892(&mut self, d4: u32, a4: u32) -> (u32, u32, u32, bool) {
        let offset = (self.read8(a4 + OBJ_DIRECTION) & 3) * 4;
        let routine = self.read32(STEPS + u32::from(offset));
        let mut r = Regs { d0: u32::from(offset), d4, a0: routine, a4, ..Regs::default() };
        self.call_family_8(routine, &mut r);
        if r.unwound {
            return (r.d4, r.a0, r.a4, r.x);
        }
        let (d4, a4) = self.e_running(r.d4);
        (d4, r.a0, a4, r.x)
    }

    /// One of `STEPS`: `cell` moved by `by`, the position word at
    /// `position` by 16 pixels as much.  Gives back X and C (the add's
    /// carry).
    fn e_step(&mut self, regs: &mut Regs, cell: u32, position: u32, by: i8) {
        let at = self.read8(cell).wrapping_add(by as u8);
        self.write8(cell, at);
        let old = self.read16(position);
        let pixels = (i16::from(by) * 16) as u16;
        let (new, carry) = old.overflowing_add(pixels);
        self.write16(position, new);
        regs.x = carry;
        regs.c = carry;
        regs.unwound = false;
    }

    /// `func_0058C0`: a cell up.
    /// No script calls it: reviewed by reading.
    pub fn r_0058c0(&mut self, regs: &mut Regs) {
        self.e_step(regs, CELL_Y, POSITION_Y, -1);
    }

    /// `func_0058CC`: a cell right.
    /// No script calls it: reviewed by reading.
    pub fn r_0058cc(&mut self, regs: &mut Regs) {
        self.e_step(regs, CELL_X, POSITION_X, 1);
    }

    /// `func_0058D8`: a cell down.
    /// No script calls it: reviewed by reading.
    pub fn r_0058d8(&mut self, regs: &mut Regs) {
        self.e_step(regs, CELL_Y, POSITION_Y, 1);
    }

    /// `func_0058E4`: a cell left.
    /// No script calls it: reviewed by reading.
    pub fn r_0058e4(&mut self, regs: &mut Regs) {
        self.e_step(regs, CELL_X, POSITION_X, -1);
    }

    /// `func_0058F0`: a task put at the cell (`func_001222`: routine
    /// `PUT_TASK`) for the `CHOSEN` object, if the cell's byte is 6 or
    /// more and the object has put fewer than it may: counted in
    /// `OBJ_PUT`, kept in its first free entry of `PLACED`, the cell's
    /// byte made 1 when it was below 7.  When the task can't be made
    /// `OBJ_PUT` goes up once more all the same.  Then d4, a4 the
    /// running object.  Gives back d0, d4, d5, d6, a0, a4, a5, a6 and C
    /// (set when none was put).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0058f0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let (cell, _) = self.r_00167a();
        let byte = self.read8(cell);
        let mut d0 = low_byte(d0, byte);
        let mut d4 = d4;
        if byte >= 6 {
            let object;
            (d4, object) = self.e_chosen(d4);
            let put = self.read8(object + OBJ_PUT);
            d0 = low_byte(d0, put);
            if put < self.read8(object + OBJ_MAY_PUT) {
                self.write8(object + OBJ_PUT, put.wrapping_add(1));
                let (d5, d6, sprite, task, _, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, object, a5, PUT_TASK_DEPTH, PUT_TASK_PRIORITY, PUT_TASK);
                if failed {
                    let (d4, object) = self.e_chosen(d4);
                    let put = self.read8(object + OBJ_PUT);
                    self.write8(object + OBJ_PUT, put.wrapping_add(1));
                    let (d4, a4) = self.e_running(d4);
                    return (d0, d4, d5, d6, cell, a4, sprite, task, true);
                }
                return self.e_put_task(d0, d4, d5, d6, sprite, task);
            }
        }
        let (d4, a4) = self.e_running(d4);
        (d0, d4, d5, d6, cell, a4, a5, a6, true)
    }

    /// The rest of `func_0058F0` once the task (`task`, its sprite at
    /// `sprite`) is made.
    #[allow(clippy::type_complexity)]
    fn e_put_task(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, sprite: u32, task: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        self.write8(sprite + SPRITE_FLAGS, PUT_TASK_SPRITE_FLAGS);
        self.r_00152a(task);
        let cell = self.read16(CELL);
        self.write16(task + TASK_CELL, cell);
        self.write8(task + TASK_FLAGS, 0);
        self.write8(task + TASK_TIMER, 3);
        let at = self.read32(CELL_AT);
        let byte = self.read8(at);
        let mut d0 = low_byte(d0, byte);
        if byte < 7 {
            let timer = self.read8(NEW_TASK_TIMER);
            self.write8(task + TASK_TIMER, timer);
            (d0, _) = self.r_00168c(d0);
            self.r_00197a();
        }
        let t17 = self.read8(NEW_TASK_17);
        self.write8(task + TASK_17, t17);
        let owner = self.read8(CHOSEN);
        self.write8(task + TASK_OWNER, owner);
        let (d0, table, _) = self.r_0059f6(low_byte(d0, owner), 0);
        let entry = table.wrapping_add(d0 as u16 as i16 as u32);
        self.write8(entry, d6 as u8);
        self.write32(entry + PLACED_TASK, task);
        let (d4, chosen) = self.e_chosen(d4);
        self.write8(chosen + OBJ_31, OBJ_31_SET);
        let d4 = low_word(d4, d4 as u8 as i8 as i16 as u16);
        let kind = self.read8(BY_OBJECT.wrapping_add(d4 as u16 as i16 as u32));
        let flags = self.read8(task + TASK_FLAGS);
        if kind == 8 {
            self.write8(task + TASK_FLAGS, flags | 0x08);
        } else if kind == 9 {
            self.write8(task + TASK_FLAGS, flags | 0x10);
        }
        let (d4, a4) = self.e_running(d4);
        let d0 = low_byte(d0, PUT_SOUND);
        self.r_0034b6(d0);
        (d0, d4, d5, d6, BY_OBJECT, a4, sprite, task, false)
    }

    /// `func_0059F6`: object `d0`'s (low byte) first entry in `PLACED`
    /// whose index is `d6`'s low byte (there is one: no end is looked
    /// for).  Gives back d0 (the entry's offset), a0 (`PLACED`) and X.
    pub fn r_0059f6(&mut self, d0: u32, d6: u32) -> (u32, u32, bool) {
        let object = d0 as u8 as i8 as i16 as u16;
        // asl.w #6: the last bit out is bit 10
        let mut x = object & 0x0400 != 0;
        let mut offset = object << 6;
        while self.read8(PLACED.wrapping_add(offset as i16 as u32)) != d6 as u8 {
            let carry;
            (offset, carry) = offset.overflowing_add(PLACED_ENTRY);
            x = carry;
        }
        (low_word(d0, offset), PLACED, x)
    }

    /// `func_005A0C`: object `d0`'s (low byte) entries in `PLACED`
    /// packed to the front, those in use kept in order.  Registers are
    /// kept.  Gives back X.
    pub fn r_005a0c(&mut self, d0: u32) -> bool {
        let entries: u32 = if self.read8(SETTINGS) & 1 != 0 { 8 } else { 10 };
        let object = d0 as u8 as i8 as i16 as u16;
        let x = object & 0x0400 != 0;
        let start = PLACED.wrapping_add((object << 6) as i16 as u32);
        let mut to = start;
        for k in 0..entries {
            let from = start + k * u32::from(PLACED_ENTRY);
            let index = self.read8(from);
            if index != 0 {
                self.write8(from, 0);
                self.write8(to, index);
                let task = self.read32(from + PLACED_TASK);
                self.write32(to + PLACED_TASK, task);
                to += u32::from(PLACED_ENTRY);
            }
        }
        x
    }

    /// `func_005A4E`: the running task (a6) taken out of its owner's
    /// count and entries (`func_005A0C` packs them), and ended
    /// (`func_00121A`).  Gives back d0, d4, d6, a0, a4, X and C.
    #[allow(clippy::type_complexity)]
    pub fn r_005a4e(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, a6: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let owner = self.read8(a6 + TASK_OWNER);
        let d4 = low_byte(d4, owner);
        let (d4, object, _) = self.r_001cfa(d4);
        let put = self.read8(object + OBJ_PUT);
        self.write8(object + OBJ_PUT, put.wrapping_sub(1));
        let (offset, table, _) = self.r_0059f6(low_word(d0, d4 as u16), d6);
        self.write8(table.wrapping_add(offset as u16 as i16 as u32), 0);
        let x = self.r_005a0c(d4);
        let (d6, x, c) = self.r_00121a(d5, d6, x);
        (d4, d4, d6, table, object, x, c)
    }

    /// `func_005A76`: the put task (in any object's entries) at the cell
    /// `CELL` whose flag bit 5 is clear.  d1, d2 and a0 are kept.  Gives
    /// back d6 (the last index read), a6 (the last task looked at), X,
    /// Z (set) and C (set when there is one).
    /// No script calls it: reviewed by reading.
    pub fn r_005a76(&mut self, d2: u32, d6: u32, a6: u32) -> (u32, u32, bool, bool, bool) {
        let cell = self.read16(CELL);
        // asl.w #8 of d2's low word: the last bit out is bit 8
        let mut x = d2 & 0x100 != 0;
        let (mut d6, mut a6) = (d6, a6);
        let mut offset: u16 = 0;
        loop {
            let index = self.read8(PLACED + u32::from(offset));
            d6 = low_byte(d6, index);
            if index != 0 {
                a6 = self.read32(PLACED + u32::from(offset) + PLACED_TASK);
                if self.read8(a6 + TASK_FLAGS) & 0x20 == 0 && self.read16(a6 + TASK_CELL) == cell {
                    return (d6, a6, x, true, true);
                }
            }
            offset += PLACED_ENTRY;
            x = false;
            if offset == PLACED_END {
                return (d6, a6, x, true, false);
            }
        }
    }

    /// `func_005AC4`: when the `CHOSEN` object's flag bit 4 is set, the
    /// put task at `CELL` (`func_005A76`) is pushed in direction `d0`
    /// (kept in `DIRECTION`): if it is still, or moving across that
    /// direction, only when it is in the middle of its cell; and only
    /// if the next cell lets it (`func_005B62`).  It gets flag bit 0,
    /// the direction, the object (`TASK_MOVED_BY`).  Gives back d1, d4,
    /// d6, a0, a4, a6 and C (set when it is not pushed, clear too when
    /// the flag is clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_005ac4(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, bool) {
        let direction = d0 as u8;
        self.write8(DIRECTION, direction);
        let (d4, object) = self.e_chosen(d4);
        if self.read8(object + OBJ_FLAGS) & 0x10 == 0 {
            let (d4, a4) = self.e_running(d4);
            return (d1, d4, d6, a0, a4, a6, false);
        }
        let (d4, a4) = self.e_running(d4);
        let (d6, a6, _, _, found) = self.r_005a76(d2, d6, a6);
        if !found {
            return (d1, d4, d6, a0, a4, a6, true);
        }
        let flags = self.read8(a6 + TASK_FLAGS);
        let mut d0 = d0;
        let across = if flags & 0x01 != 0 {
            true
        } else {
            let turn = (self.read8(a6 + TASK_DIRECTION) ^ direction) & 1;
            d0 = low_byte(d0, turn);
            turn != 0
        };
        if across {
            let (_, middle) = self.r_001896(d0, a6);
            if !middle {
                return (d1, d4, d6, a0, a4, a6, true);
            }
        }
        let (a0, blocked) = self.r_005b62(a6);
        if blocked {
            let (d4, a4) = self.e_running(d4);
            return (d1, d4, d6, a0, a4, a6, true);
        }
        let by = self.read8(CHOSEN);
        self.write8(a6 + TASK_MOVED_BY, by);
        let flags = self.read8(a6 + TASK_FLAGS);
        self.write8(a6 + TASK_FLAGS, (flags & 0xF8) | 1);
        self.write8(a6 + TASK_DIRECTION, direction);
        let (d4, a4) = self.e_running(d4);
        (d1, d4, d6, a0, a4, a6, false)
    }

    /// `func_005B62`: whether the cell next to the task at a6 in
    /// `DIRECTION` (`func_001726`) is blocked: its byte below 5.
    /// `CELL` is kept.  Gives back a0 (its address) and C.
    /// No script calls it: reviewed by reading.
    pub fn r_005b62(&mut self, a6: u32) -> (u32, bool) {
        let cell = self.read16(CELL);
        let direction = self.read8(DIRECTION);
        self.r_001726(u32::from(direction), a6);
        self.write16(CELL, cell);
        let at = self.read32(CELL_AT);
        (at, self.read8(at) < 5)
    }

    /// `func_005B8C`: unless `func_002C36` says no, the put task at
    /// `CELL` (`func_005A76`) with `TASK_TIMER` 4 or more is thrown in
    /// direction `d0` (`func_005BD2`, as the running task), with a
    /// sound.  d4 and a4 are kept.  Gives back d0, d6, a0, a6 and X.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_005b8c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, bool) {
        self.write8(DIRECTION, d0 as u8);
        let (d0, a0, x, no) = self.r_002c36(d0, a4);
        if no {
            return (d0, d6, a0, a6, x);
        }
        let (d6, a6, x, _, found) = self.r_005a76(d2, d6, a6);
        if !found || self.read8(a6 + TASK_TIMER) < 4 {
            return (d0, d6, a0, a6, x);
        }
        self.r_00187c(a6);
        let current = self.read8(CURRENT_TASK);
        self.write8(CURRENT_TASK, d6 as u8);
        let (d0, a0, x) = self.r_005bd2(d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6);
        let d0 = low_byte(d0, THROWN_SOUND);
        self.r_0034b6(d0);
        self.write8(CURRENT_TASK, current);
        (d0, d6, a0, a6, x)
    }

    /// `func_005BD2`: the running task (a6) thrown in `DIRECTION`: its
    /// flags the direction with bit 5 (and bits 3-4 kept), its offset
    /// 0, `TASK_CELL` the rate `THROWN_RATE`, the cell's byte made
    /// again from its source (`func_00168C`); its routine `THROWN` from
    /// now on, and a task that follows it made (`func_004FF4`).  Gives
    /// back d0, a0 and X.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_005bd2(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, bool) {
        let direction = self.read8(DIRECTION);
        let flags = self.read8(a6 + TASK_FLAGS);
        self.write8(a6 + TASK_FLAGS, (flags & 0x38) | 0x20 | direction);
        self.write8(a6 + TASK_OFFSET, 0);
        self.write8(a6 + TASK_OFFSET_X, 0);
        self.write16(a6 + TASK_CELL, THROWN_RATE);
        let d0 = self.r_0015f8(a6);
        self.r_00167a();
        let (d0, a0) = self.r_00168c(d0);
        self.r_00053e(d6, THROWN);
        let (d0, x, _) = self.r_004ff4(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        (d0, a0, x)
    }

    /// `func_005E0E`: the put task at `CELL` (`func_005A76`) with
    /// `TASK_TIMER` 4 or more is stopped: the timer 3, in the middle of
    /// its cell, its flags 0, its routine `STOPPED` from now on.
    /// Registers are kept.  Gives back X, Z and C.
    /// No script calls it: reviewed by reading.
    pub fn r_005e0e(&mut self, d2: u32, d6: u32, a6: u32) -> (bool, bool, bool) {
        let (d6, a6, x, z, found) = self.r_005a76(d2, d6, a6);
        if !found {
            return (x, z, false);
        }
        let timer = self.read8(a6 + TASK_TIMER);
        if timer < 4 {
            return (x, timer == 4, true);
        }
        self.write8(a6 + TASK_TIMER, 3);
        self.r_00187c(a6);
        self.write8(a6 + TASK_FLAGS, 0);
        let current = self.read8(CURRENT_TASK);
        self.write8(CURRENT_TASK, d6 as u8);
        let (x, _) = self.r_00053e(d6, STOPPED);
        self.write8(CURRENT_TASK, current);
        (x, current == 0, false)
    }
}
