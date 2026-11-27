//! The cell at `CELL_X`, `CELL_Y`: the object there, its 2x2 tiles on
//! the plane, the routines run by the kind in its byte of the cell map,
//! the cells around an object put right.
//!
//! The cell map (`func_00167A` keeps the cell's address in `CELL_AT`)
//! has a byte a cell, and 0x200 bytes on a second byte: bits 4-6 of that
//! one choose one of seven routines (1-7) from a table.

use crate::{Game, Regs};

/// The object records (`func_001CFA`): the byte that says one is in use,
/// its position (words, x then y; the cell is the high 12 bits of each).
const OBJ_USED: u32 = 0x00;
const OBJ_X: u32 = 0x18;
const OBJ_Y: u32 = 0x1A;
/// Cleared by `func_0053A6` when the cell has no routine.
const OBJ_3A: u32 = 0x3A;
/// How many objects there are.
const OBJECTS: u8 = 20;

/// A position (words, x then y) and a cell (column, row; one word).
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
/// The map's width and height (in cells, bytes).
const WIDTH: u32 = 0xFF9EB2;
const HEIGHT: u32 = 0xFF9EB3;
/// Where the cell is on the plane: its column x 2 (a byte, in tiles)
/// and its row's offset (a word); `func_001278` sets them from the cell.
const COLUMN: u32 = 0xFF9E74;
const ROW: u32 = 0xFF9E76;
/// The plane's address in VRAM (a word) the offsets are from.
const PLANE: u32 = 0xFF8068;
/// The VDP's control port and data port.
const CONTROL: u32 = 0xC00004;
const DATA: u32 = 0xC00000;
/// A VRAM write's command: one line of the plane (0x40 tiles) further.
const NEXT_LINE: u32 = 0x0080_0000;

/// The cell's address in the cell map (`func_00167A`), and where its
/// second byte is from it.
const CELL_AT: u32 = 0xFF9ED6;
const SECOND: u32 = 0x200;
/// Bits 4-6 of the second byte: which routine (1-7, none if 0).
const KIND: u8 = 0x70;
/// The routines by kind for a task (`func_005378`; kind 1 is the
/// first) and for an object (`func_0053A6`; from the long after this
/// one, kind 1 at `FOR_OBJECT + 4`).
const FOR_TASK: u32 = 0xFFFF_A97A;
const FOR_OBJECT: u32 = 0xFFFF_A992;
/// A table of routines (a long: its address) `func_005336` runs one of
/// by `d0`'s low 6 bits.
const ROUTINES: u32 = 0xFF9E80;

/// The cell where `func_005356` puts a position (words, x then y), in
/// the middle of the cell.
const MIDDLE_X: u32 = 0xFF954A;
const MIDDLE_Y: u32 = 0xFF954C;

/// The object whose turn it is (its index and address, `func_001CA0`)
/// and the one `func_0053A6` clears `OBJ_3A` of when the cell has no
/// routine (an index, a byte).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const CHOSEN: u32 = 0xFFA93F;
/// The running task's index.
const CURRENT_TASK: u32 = 0xFF80EE;

/// `func_0053EC`'s square of cells: its first column, and how many
/// cells a side (bytes).
const SQUARE_X: u32 = 0xFFA9B2;
const SQUARE_SIDE: u32 = 0xFFA9B3;
/// What the cells of the square changed from 2 to 6 get: a long for
/// `OBJ_INIT_2`, an address on the plane (`OBJ_INIT_AT`), for
/// `func_0101C0`.
const OBJ_INIT_2: u32 = 0xFF80D0;
const OBJ_INIT_AT: u32 = 0xFF80C8;

/// The task list's head and a task record's fields: the next task's
/// index (0xFF: the tail) and address, its priority, and the cell it
/// is at (column, row; a word).
const HEAD: u32 = 0xFFFF_80F2;
const TASK_NEXT: u32 = 0x00;
const TASK_PRIORITY: u32 = 0x02;
const TASK_NEXT_ADDR: u32 = 0x08;
const TASK_CELL: u32 = 0x1A;
/// The priority `func_005516` looks for.
const CELL_TASK_PRIORITY: u8 = 0x42;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_005252`: the first object in use whose position is in the
    /// cell of `POSITION_X`, `POSITION_Y` (both already at the cell's
    /// corner).  Gives back d0 (the last word compared), d4 and a4 (the
    /// object's index and address, or the last one's) and C (set when
    /// there is one).
    pub fn r_005252(&mut self, d0: u32) -> (u32, u32, u32, bool) {
        let mut d0 = d0;
        let mut index = 0u32;
        loop {
            let (d4, object, _) = self.r_001cfa(index);
            if self.read8(object + OBJ_USED) != 0 {
                let x = self.read16(object + OBJ_X) & 0xFFF0;
                d0 = low_word(d0, x);
                if x == self.read16(POSITION_X) {
                    let y = self.read16(object + OBJ_Y) & 0xFFF0;
                    d0 = low_word(d0, y);
                    if y == self.read16(POSITION_Y) {
                        return (d0, d4, object, true);
                    }
                }
            }
            index = d4 + 1;
            if index as u8 == OBJECTS {
                return (d0, index, object, false);
            }
        }
    }

    /// The VRAM write command of the cell at `COLUMN`, `ROW` on the
    /// plane (`func_000282`).  Gives back X (of the last add) and C.
    fn e_cell_command(&mut self) -> (u32, bool, bool) {
        let column = u16::from(self.read8(COLUMN));
        let offset = column.wrapping_add(self.read16(ROW)).wrapping_mul(2);
        let base = self.read16(PLANE);
        let (address, x) = offset.overflowing_add(base);
        let (command, c) = self.r_000282(u32::from(address));
        (command, x, c)
    }

    /// The cell's two lines of two tiles: `top` (two words) at
    /// `command`, then `bottom` a line down, interrupts masked.
    fn e_cell_tiles(&mut self, command: u32, top: u32, bottom: u32) {
        let ipl = self.ipl();
        self.set_ipl(7);
        self.write32(CONTROL, command);
        self.write32(DATA, top);
        self.write32(CONTROL, command.wrapping_add(NEXT_LINE));
        self.write32(DATA, bottom);
        self.set_ipl(ipl);
    }

    /// `func_005294`: the cell at `COLUMN`, `ROW` drawn with the tiles
    /// from `d0` (its low word): that one and the next on the top line,
    /// the two after on the bottom one.  d0 and d1 are kept.  Gives back
    /// X and C.
    pub fn r_005294(&mut self, d0: u32, _d1: u32) -> (bool, bool) {
        let tile = d0 as u16;
        let top = u32::from(tile) << 16 | u32::from(tile & 0xFF00 | u16::from((tile as u8).wrapping_add(1)));
        let (command, x, c) = self.e_cell_command();
        self.e_cell_tiles(command, top, top.wrapping_add(0x0002_0002));
        (x, c)
    }

    /// `func_0052F0`: the cell at `COLUMN`, `ROW` cleared (tile 0 in
    /// all four).  d0 is kept.  Gives back X and C.
    pub fn r_0052f0(&mut self, _d0: u32) -> (bool, bool) {
        let (command, x, c) = self.e_cell_command();
        self.e_cell_tiles(command, 0, 0);
        (x, c)
    }

    /// `func_0052E6` (one of the routines by kind): d0's low byte 6, C
    /// clear (`func_0052EA`).
    /// No script calls it: reviewed by reading.
    pub fn r_0052e6(&mut self, regs: &mut Regs) {
        regs.d0 = low_byte(regs.d0, 6);
        self.r_0052ea(regs);
    }

    /// `func_0052EA` (one of the routines by kind): C clear.
    pub fn r_0052ea(&mut self, regs: &mut Regs) {
        regs.c = false;
        regs.unwound = false;
    }

    /// `func_005336`: routine `d0` (its low 6 bits) of the table at
    /// `ROUTINES` run, with d0 those bits; d1 and a0 are kept.  Gives
    /// back the other registers and C as the routine leaves them.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_005336(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let which = d0 as u8 & 0x3F;
        let offset = u16::from(which) * 4;
        let table = self.read32(ROUTINES);
        let routine = self.read32(table.wrapping_add(u32::from(offset)));
        let mut r = Regs {
            d0: low_byte(d0, which),
            d1: low_word(d1, offset),
            d2, d3, d4, d5, d6, d7,
            a0: routine,
            a1, a2, a3, a4, a5, a6,
            ..Regs::default()
        };
        self.call_family_16(routine, &mut r);
        (r.d0, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.c)
    }

    /// `func_005356` (one of the routines by kind): the middle of the
    /// cell at `POSITION_X`, `POSITION_Y` to `MIDDLE_X`, `MIDDLE_Y`;
    /// d0's low byte 6, C clear.
    pub fn r_005356(&mut self, regs: &mut Regs) {
        let x = self.read16(POSITION_X) | 8;
        self.write16(MIDDLE_X, x);
        let y = self.read16(POSITION_Y);
        self.write16(MIDDLE_Y, y | 8);
        regs.d0 = low_word(regs.d0, (y & 0xFF00) | 6);
        regs.c = false;
        regs.unwound = false;
    }

    /// The kind in the second byte of the cell at `at` (0: none), and
    /// the offset of its routine in the tables (kind x 4).
    fn e_kind(&mut self, at: u32) -> (u8, u8) {
        let bits = self.read8(at + SECOND) & KIND;
        (bits >> 4, bits >> 2)
    }

    /// `func_005378`: the routine for a task of the kind of the cell at
    /// `CELL_AT` run (`FOR_TASK`), then a6 the running task's record
    /// again.  Gives back d1, d6, a6 and X.
    pub fn r_005378(&mut self, d0: u32, d1: u32, d6: u32, a6: u32, x: bool) -> (u32, u32, u32, bool) {
        let cell = self.read32(CELL_AT);
        let (kind, offset) = self.e_kind(cell);
        if kind == 0 {
            return (d1, d6, a6, x);
        }
        let routine = self.read32(FOR_TASK - 4 + u32::from(offset));
        let mut r = Regs {
            d0: low_byte(d0, kind),
            d1: low_word(d1, u16::from(offset)),
            d6,
            a0: routine,
            a6,
            // the bit lsr #2 shifted out last
            // the bit the second lsr #2 shifts out: bit 3, outside the kind
            x: false,
            ..Regs::default()
        };
        self.call_family_3(routine, &mut r);
        if r.unwound {
            return (r.d1, r.d6, r.a6, r.x);
        }
        let byte = self.read8(CURRENT_TASK);
        let (d6, a6, x) = self.r_0005f8(low_byte(r.d6, byte));
        (r.d1, d6, a6, x)
    }

    /// `func_0053A6`: the routine for an object of the kind of the cell
    /// of the object at a4 run (`FOR_OBJECT`); with none, the `CHOSEN`
    /// object's `OBJ_3A` cleared.  Then d4 and a4 the running object
    /// again.  Gives back every register.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0053a6(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let (d0, _) = self.r_0015e6(a4);
        let (cell, _) = self.r_00167a();
        let (kind, offset) = self.e_kind(cell);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0: cell, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        if kind == 0 {
            r.d0 = low_byte(d0, 0);
            let byte = self.read8(CHOSEN);
            let (_, chosen, _) = self.r_001cfa(low_byte(d4, byte));
            self.write8(chosen + OBJ_3A, 0);
        } else {
            let routine = self.read32(FOR_OBJECT + u32::from(offset));
            r.d0 = low_byte(d0, kind);
            r.d1 = low_word(d1, u16::from(offset));
            r.a0 = routine;
            r.x = false;
            self.call_family_4(routine, &mut r);
            if r.unwound || self.restarted.is_some() {
                return (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            }
        }
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6)
    }

    /// `func_0053EC`: the cells within `d0` (its low byte) cells of
    /// object 0's, a square cut off at the map's top and left (and
    /// at its width and height): each whose byte in the cell map is 2
    /// becomes 6 (its second byte 0x60), and `func_0101C0` is run for it
    /// (`OBJ_INIT_2` 0x21F8, `OBJ_INIT_AT` from `COLUMN`, `ROW`).  The
    /// registers are kept but a6.  Gives back a6.
    pub fn r_0053ec(&mut self, d0: u32, d4: u32, _a4: u32, a6: u32) -> u32 {
        let reach_reg = d0;
        let reach = d0 as u8;
        let (_, first, _) = self.r_001cfa(d4 & 0xFFFF_0000);
        // d0's high word goes on to `func_0101C0` from here
        let (mut d0, _) = self.r_0015e6(first);
        // the corner, kept from going negative (as a signed byte)
        let from = |v: u8| {
            let v = v.wrapping_sub(reach);
            if (v as i8) < 0 { 0 } else { v }
        };
        let y = from(self.read8(CELL_Y));
        self.write8(CELL_Y, y);
        let x = from(self.read8(CELL_X));
        self.write8(SQUARE_X, x);
        let side = reach.wrapping_add(reach) | 1;
        self.write8(SQUARE_SIDE, side);
        let mut rows = (side as i8 as i16 as u16).wrapping_sub(1);
        // the high word of d1 is d0's from the start
        let mut d1 = low_word(reach_reg, 0x202);
        let mut a6 = a6;
        loop {
            let byte = self.read8(SQUARE_X);
            self.write8(CELL_X, byte);
            let mut columns = u16::from(self.read8(SQUARE_SIDE)).wrapping_sub(1);
            loop {
                let (cell, _) = self.r_00167a();
                if self.read8(cell) == 2 {
                    self.write8(cell, 6);
                    self.write8(cell + SECOND, 0x60);
                    self.r_001278();
                    self.write16(OBJ_INIT_2, 0x21F8);
                    let byte = self.read8(COLUMN);
                    let at = self.read16(ROW) | u16::from(byte);
                    let at = at.wrapping_mul(2).wrapping_add(self.read16(PLANE));
                    self.write16(OBJ_INIT_AT, at);
                    (d0, d1, _, a6, _) = self.r_0101c0(low_word(d0, at), d1, cell, a6);
                }
                let x = self.read8(CELL_X).wrapping_add(1);
                self.write8(CELL_X, x);
                if x == self.read8(WIDTH) || columns == 0 {
                    break;
                }
                columns -= 1;
            }
            let y = self.read8(CELL_Y).wrapping_add(1);
            self.write8(CELL_Y, y);
            if y == self.read8(HEIGHT) || rows == 0 {
                break;
            }
            rows -= 1;
        }
        a6
    }

    /// `func_005516`: whether a task of priority `CELL_TASK_PRIORITY` is
    /// at the cell `CELL_X`, `CELL_Y` (`TASK_CELL`).  The list is in
    /// order of priority, so the search stops past it.  Registers are
    /// kept.  Gives back X (bit 8 of d0, shifted out making the word to
    /// compare) and C (set when there is one).
    /// No script calls it: reviewed by reading.
    pub fn r_005516(&mut self, d0: u32) -> (bool, bool) {
        let x = d0 & 0x100 != 0;
        let cell = self.read16(CELL_X);
        let mut task = HEAD;
        loop {
            if self.read8(task + TASK_NEXT) == 0xFF {
                return (x, false);
            }
            task = self.read32(task + TASK_NEXT_ADDR);
            let priority = self.read8(task + TASK_PRIORITY);
            if priority > CELL_TASK_PRIORITY {
                return (x, false);
            }
            if priority == CELL_TASK_PRIORITY && self.read16(task + TASK_CELL) == cell {
                return (x, true);
            }
        }
    }
}
