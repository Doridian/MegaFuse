//! The stage's start (0xE3E-0x10C8): the entry of `D_00A04A` it is
//! built from, its map unpacked and drawn on plane A, the cells' bytes
//! unpacked, the cell maps made from the map (the code at 0x50EE, which
//! only `func_00108C` reaches), and the two tasks made meanwhile.

use crate::{Game, Regs};

/// The VDP's control port and data port.
const CONTROL: u32 = 0xC00004;
const DATA: u32 = 0xC00000;
/// Register 15 (the address step) as the game keeps it (`0x8Fxx`).
const REG15: u32 = 0xFFFF_F022;

/// The index `func_001030` picks its entries by (a byte).
const INDEX: u32 = 0xFFFF_953A;
/// The entries `func_001030` picks from (longs in the ROM, `D_00A04A`):
/// each the address of four longs (to `ENTRY_LONGS`: the fourth is the
/// routine `func_000E98` runs) followed by the list `func_0094D8`
/// loads (its address to `LOAD_LIST`).
const ENTRIES: u32 = 0x00A04A;
const ENTRY_LONGS: [u32; 4] = [0xFFFF_9E78, 0xFFFF_9E7C, 0xFFFF_9E80, 0xFFFF_9E84];
const ENTRY_ROUTINE: u32 = 0xFFFF_9E84;
const LOAD_LIST: u32 = 0xFFFF_92FA;
/// A word by the same index (in the ROM, `D_00A4EC`), kept in
/// `PICKED_WORD`; `PICKED_BYTE` is 0xDE when it is 0x6000, else 0xDF.
const WORDS: u32 = 0x00A4EC;
const PICKED_WORD: u32 = 0xFFFF_9EC0;
const PICKED_BYTE: u32 = 0xFFFF_9EC2;

/// Cleared by `func_000E9E` first thing.
const CLEARED_A936: u32 = 0xFFFF_A936;
/// The index of `func_0096B0`'s entry `func_000E9E` picks (a byte), in
/// the table at `MAP_TABLE` (in the ROM, `D_0EB556`).
const MAP_INDEX: u32 = 0xFFFF_953F;
const MAP_TABLE: u32 = 0x0EB556;
/// The map's width and height in cells as the load leaves them, and
/// where they are kept (bytes).
const LOADED_WIDTH: u32 = 0xFFFF_9309;
const LOADED_HEIGHT: u32 = 0xFFFF_9308;
const WIDTH: u32 = 0xFFFF_9EB2;
const HEIGHT: u32 = 0xFFFF_9EB3;
/// A map this wide gets d2 = 0xF; this narrow gets its first column
/// drawn again and two columns of the plane cleared; this high gets
/// `func_008644`'s scroll offsets.
const WIDE: u8 = 0x20;
const NARROW: u8 = 0x10;
const HIGH: u8 = 0x1A;
/// The map: a byte a cell, rows of `WIDTH` (`map.rs`).
const MAP: u32 = 0xFFFF_955C;
/// The bytes `func_000E3E` clears: from `MAP`, and the 0x38 from
/// 0xFF9E88.
const MAP_BYTES: u32 = 0x810;
const CLEARED_9E88: u32 = 0xFFFF_9E88;
const CLEARED_9E88_BYTES: u32 = 0x38;
/// Set to 8 by `func_000E3E`.
const SET_9EBD: u32 = 0xFFFF_9EBD;
/// Where the cell drawn next goes (`map.rs`): its column x 2 (a byte)
/// and its row's offset (a word).
const COLUMN: u32 = 0xFFFF_9E74;
const ROW: u32 = 0xFFFF_9E76;
/// A row of cells on the plane (two lines of 0x40 words).
const ROW_STEP: u16 = 0x80;
/// Where a narrow map's first column is drawn again (column x 2).
const NARROW_COPY: u8 = 0x20;
/// The two columns of plane A a narrow map clears: 0x1C words down
/// (0x80 bytes apart) from each.
const NARROW_CLEAR: [u16; 2] = [0xE200, 0xE242];
const NARROW_CLEAR_WORDS: u16 = 0x1C;

/// The packed bytes of the cells (`func_00108C`, a long), and where
/// they are unpacked: 0x100 bytes, one by the map's byte of a cell.
const CELL_BYTES_SOURCE: u32 = 0xFFFF_9E70;
const CELL_BYTES: u32 = 0xFFFF_9D6C;
/// The packed data's address (`decode.rs`).
const SOURCE: u32 = 0xFFFF_930A;

/// The scroll's values and the offsets added to them (`h_scroll.rs`):
/// x, y; their sums go to `SUM_X` (and negated, the horizontal scroll
/// table's first word, `H_FIRST`) and `SUM_Y` (and the vertical one's,
/// `V_FIRST`).
const SCROLL_X: u32 = 0xFFFF_9EB4;
const SCROLL_Y: u32 = 0xFFFF_9EB8;
const ADD_X: u32 = 0xFFFF_9EBC;
const ADD_Y: u32 = 0xFFFF_9EBE;
const SUM_X: u32 = 0xFFFF_80E6;
const SUM_Y: u32 = 0xFFFF_80EA;
const H_FIRST: u32 = 0xFFFF_D002;
const V_FIRST: u32 = 0xFFFF_D402;

/// The tasks made: the scroll's (`func_000E6C`), the one that ends the
/// start (`func_0010C8`) and the one that one makes (`func_00BDD8`),
/// with their priorities.
const SCROLL_TASK: u32 = 0x000E6C;
const SCROLL_TASK_PRIORITY: u8 = 0x30;
const START_TASK: u32 = 0x0010C8;
const START_TASK_PRIORITY: u8 = 0x10;
const NEXT_TASK: u32 = 0x00BDD8;
const NEXT_TASK_PRIORITY: u8 = 0x70;

/// What `func_0010C8` clears: 0x340 bytes from 0xFF0000 and the long
/// after them; 0x50 longs from 0xFFAAC0.
const CLEARED_FF0000: u32 = 0xFFFF_0000;
const CLEARED_FF0000_BYTES: u32 = 0x340;
const CLEARED_AAC0: u32 = 0xFFFF_AAC0;
const CLEARED_AAC0_LONGS: u32 = 0x50;
/// Cleared once `NEXT_TASK` is made.
const CLEARED_AC7C: u32 = 0xFFFF_AC7C;

/// Bit 0 of this byte: the cell maps' marks are 0xF0, not 0x70, and
/// `func_0010C8` sends the Z80 command `START_COMMAND`.
const MODE: u32 = 0xFFFF_9540;
const START_COMMAND: u8 = 0x18;

/// What the code at 0x50EE sets up: counts cleared (a byte at
/// `MARKED`, the one the second cell map's marks are counted in; a word
/// at `DRAWN_INDEX`, whose high byte counts the numbers drawn, and a
/// byte at `CLEARED_ACE0`).
const MARKED: u32 = 0xFFFF_A9B8;
const DRAWN_INDEX: u32 = 0xFFFF_A9BA;
const CLEARED_ACE0: u32 = 0xFFFF_ACE0;
/// Where each number drawn goes: at `DRAWN` + the sign-extended
/// `DRAWN_INDEX` (which steps 0x100 at a time).
const DRAWN: u32 = 0xFFFF_A9BC;
/// The cells' routines by kind for a task and for an object (`e_cell.rs`),
/// 7 each, all set to `NO_ROUTINE` (`func_0093B8`); two more longs set
/// to `func_0054AE` and `func_00C886`.
const FOR_TASK: u32 = 0xFFFF_A97A;
const FOR_OBJECT: u32 = 0xFFFF_A996;
const NO_ROUTINE: u32 = 0x0093B8;
const ROUTINE_A9AA: u32 = 0xFFFF_A9AA;
const ROUTINE_A9AE: u32 = 0xFFFF_A9AE;
/// The bound a number drawn is compared with (`MODE` bit 0: 0xF0, else
/// 0x70).
const BOUND: u32 = 0xFFFF_9ED3;
/// The cell maps: from `CELL_AT` (`e_cell.rs`; a byte a cell, and
/// another `SECOND` on) and `CELL_AT_2`, and the map's byte, the
/// addresses (longs) moving on a cell at a time.
const CELL_AT: u32 = 0xFFFF_9ED6;
const CELL_MAP: u32 = 0xFFFF_989C;
const SECOND: u32 = 0x200;
const CELL_AT_2: u32 = 0xFFFF_A9B4;
const CELL_MAP_2: u32 = 0xFFFF_96FC;
const MAP_AT: u32 = 0xFFFF_9EDA;
/// The cell's position (words, x then y: 0x10 a cell) and the cells
/// still to go in the row and rows still to go (bytes).
const POSITION_X: u32 = 0xFFFF_9F04;
const POSITION_Y: u32 = 0xFFFF_9F06;
const CELLS_LEFT: u32 = 0xFFFF_9F08;
const ROWS_LEFT: u32 = 0xFFFF_9F09;
/// A cell's byte: bit 6, its kind's routine (`func_005336`) gives the
/// cell map's byte; else its bits 7 and 0-2 are kept; bit 4 or no bits
/// 0-2 and no number is drawn.
const BY_ROUTINE: u8 = 1 << 6;
const KEPT_BITS: u8 = 0x87;
const NO_DRAW: u8 = 1 << 4;
const DRAW_BITS: u8 = 0x07;
/// A cell with a number below `BOUND` and no object in it gets these
/// tiles (`func_005294`) and 2 in the cell map.
const MARK_TILES: u16 = 0x4304;
const MARK: u8 = 2;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// `dbra` on `reg`'s low word: true when it has run out (0xFFFF).
fn lo_dbra(reg: &mut u32) -> bool {
    let w = (*reg as u16).wrapping_sub(1);
    *reg = low_word(*reg, w);
    w == 0xFFFF
}

impl Game {
    /// `func_000E3E`: the map (`MAP_BYTES` from `MAP`) and 0x38 bytes
    /// from 0xFF9E88 cleared, `SET_9EBD` 8, the scroll's task
    /// (`func_000E6C`, priority 0x30) made; then it goes on into that
    /// task's routine.  Gives back d0, d1, d6, a6 and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_000e3e(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool) {
        let (_, d1, _, _) = self.r_0011d2(CLEARED_9E88_BYTES, d1, CLEARED_9E88);
        let (d0, d1, a0, _) = self.r_0011d2(MAP_BYTES, d1, MAP);
        self.write8(SET_9EBD, 8);
        let (d6, a6, x, c) = self.r_0003e8(d4, d5, d6, SCROLL_TASK_PRIORITY, SCROLL_TASK);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c, ..Regs::default() };
        self.r_000e6c(&mut r);
        (r.d0, r.d1, r.d6, r.a6, r.x)
    }

    /// `func_000E6C` (the scroll's task, a frame at a time): `SCROLL_X`
    /// + `ADD_X` to `SUM_X` (negated, to `H_FIRST`), `SCROLL_Y` +
    /// `ADD_Y` to `SUM_Y` and `V_FIRST`; then `func_008664` and
    /// `func_002CC0`.  Gives back d0, X, N, Z and C as they leave them.
    pub fn r_000e6c(&mut self, regs: &mut Regs) {
        let x = self.read16(SCROLL_X).wrapping_add(self.read16(ADD_X));
        self.write16(SUM_X, x);
        self.write16(H_FIRST, x.wrapping_neg());
        let (y, carry) = self.read16(SCROLL_Y).overflowing_add(self.read16(ADD_Y));
        self.write16(SUM_Y, y);
        self.write16(V_FIRST, y);
        regs.d0 = low_word(regs.d0, y);
        regs.x = carry;
        regs.c = carry;
        self.r_008664(regs);
        (regs.d0, regs.x, regs.c) = self.r_002cc0(regs.d0, regs.x, regs.c);
    }

    /// `func_000E98`: the routine in `ENTRY_ROUTINE` run, a0 its
    /// address.  Gives back the registers and X as it leaves them.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_000e98(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let a0 = self.read32(ENTRY_ROUTINE);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c, ..Regs::default() };
        self.call_family_4(a0, &mut r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x)
    }

    /// `func_000E9E`: `CLEARED_A936` cleared, the entry's routine run
    /// (`func_000E98`), the map's data picked (entry `MAP_INDEX` of
    /// `MAP_TABLE`, `func_0096B0`) and the scroll's offsets set
    /// (`func_008632`; `func_008644` too on a map `HIGH` cells high).
    /// The map is unpacked to `MAP` and drawn a cell at a time
    /// (`func_000FBE`); a `NARROW` map has its first column drawn again
    /// at column 16 and two columns of the plane cleared
    /// (`NARROW_CLEAR`).  Then `func_00108C`.  Gives back the registers
    /// and X as that leaves them.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_000e9e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        self.write8(CLEARED_A936, 0);
        let mut r = Regs { a1, a2, a3, a4, a5, a6, ..Regs::default() };
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_000e98(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, x, false);
        if self.restarted.is_some() {
            return (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        }
        r.a0 = MAP_TABLE;
        r.d0 = low_byte(r.d0, self.read8(MAP_INDEX));
        self.r_0096b0(r.d0, r.a0);
        self.r_008632(&mut r);
        let width = self.read8(LOADED_WIDTH);
        self.write8(WIDTH, width);
        r.d2 = if width == WIDE { 0xF } else { 0 };
        let height = self.read8(LOADED_HEIGHT);
        self.write8(HEIGHT, height);
        if height == HIGH {
            self.r_008644();
        }

        // the map, unpacked and drawn (the counts as dbra takes them)
        self.r_000cba();
        r.a1 = MAP;
        self.write16(ROW, 0);
        r.d4 = low_word(r.d4, u16::from(height).wrapping_sub(1));
        loop {
            self.write8(COLUMN, 0);
            let (cells, borrow) = u16::from(width).overflowing_sub(1);
            r.d3 = low_word(r.d3, cells);
            (r.x, r.c) = (borrow, borrow);
            loop {
                self.r_000cf4(&mut r);
                self.write8(r.a1, r.d0 as u8);
                r.a1 = r.a1.wrapping_add(1);
                (r.d0, r.x, r.c) = self.r_000fbe(r.d0);
                let column = self.read8(COLUMN);
                let (column, carry) = column.overflowing_add(2);
                self.write8(COLUMN, column);
                (r.x, r.c) = (carry, carry);
                if lo_dbra(&mut r.d3) {
                    break;
                }
            }
            let carry = self.lo_row_down();
            (r.x, r.c) = (carry, carry);
            if lo_dbra(&mut r.d4) {
                break;
            }
        }

        if self.read8(WIDTH) == NARROW {
            self.write16(ROW, 0);
            self.write8(COLUMN, NARROW_COPY);
            r.a1 = MAP;
            r.d4 = low_word(r.d4, u16::from(self.read8(HEIGHT)).wrapping_sub(1));
            loop {
                let cell = self.read8(r.a1);
                r.d0 = low_byte(r.d0, cell);
                (r.d0, r.x, r.c) = self.r_000fbe(r.d0);
                let carry = self.lo_row_down();
                (r.x, r.c) = (carry, carry);
                r.a1 = r.a1.wrapping_add(0x10);
                if lo_dbra(&mut r.d4) {
                    break;
                }
            }
            // (the status register saved and put back: X with it)
            let ipl = self.ipl();
            self.set_ipl(7);
            let step = (self.read16(REG15) & 0xFF00) | 0x80;
            self.write16(CONTROL, step);
            r.d1 = low_word(r.d1, 0);
            for at in NARROW_CLEAR {
                let (command, _) = self.r_000282(low_word(r.d0, at));
                r.d0 = command;
                self.write32(CONTROL, command);
                for _ in 0..NARROW_CLEAR_WORDS {
                    self.write16(DATA, 0);
                }
            }
            r.d2 = low_word(r.d2, 0xFFFF);
            self.set_ipl(ipl);
            let reg15 = self.read16(REG15);
            self.write16(CONTROL, reg15);
        }
        self.r_00108c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x)
    }

    /// `ROW` a row of cells down; gives back the add's carry.
    fn lo_row_down(&mut self) -> bool {
        let row = self.read16(ROW);
        let (row, carry) = row.overflowing_add(ROW_STEP);
        self.write16(ROW, row);
        carry
    }

    /// `func_001030`: entry `INDEX` of `ENTRIES`: its four longs to
    /// `ENTRY_LONGS`, its list (after them) loaded (`LOAD_LIST`,
    /// `func_0094D8`); then `PICKED_WORD` the index's word of `WORDS`,
    /// and `PICKED_BYTE` 0xDE if that is 0x6000, else 0xDF.  Gives back
    /// d0 (the word), a0 (`WORDS`), the other registers as the load
    /// leaves them, and X (the carry of the index x 2).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_001030(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let index = u16::from(self.read8(INDEX));
        let saved = low_word(d0, index);
        let by_two = index.wrapping_add(index);
        let (by_long, carry) = by_two.overflowing_add(by_two);
        let mut at = self.read32(ENTRIES.wrapping_add(by_long as i16 as u32));
        for to in ENTRY_LONGS {
            let long = self.read32(at);
            self.write32(to, long);
            at = at.wrapping_add(4);
        }
        self.write32(LOAD_LIST, at);
        // (X: the carry of the index x 4, a word)
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, _) =
            self.r_0094d8(low_word(d0, by_long), d1, d2, d3, d4, d5, d6, d7, at, a1, a2, a3, a4, a5, a6, carry);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x);
        }
        self.write8(PICKED_BYTE, 0xDE);
        let (by_word, carry) = (saved as u16).overflowing_add(saved as u16);
        let word = self.read16(WORDS.wrapping_add(by_word as i16 as u32));
        self.write16(PICKED_WORD, word);
        if word != 0x6000 {
            self.write8(PICKED_BYTE, 0xDF);
        }
        (low_word(saved, word), d1, d2, d3, d4, d5, d6, d7, WORDS, a1, a2, a3, a4, a5, a6, carry)
    }

    /// `func_00108C`: the cells' bytes unpacked (`CELL_BYTES_SOURCE` to
    /// `CELL_BYTES`), the task `func_0010C8` made (priority 0x10), the
    /// stage's objects (`func_00355E`); d0-d4, a0 and a4 put back.
    /// Then (the code at 0x50EE): the cells' routines set to none, and
    /// the cell maps made from the map, a cell at a time
    /// (`lo_cell_maps`).  Gives back the registers and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00108c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let source = self.read32(CELL_BYTES_SOURCE);
        self.write32(SOURCE, source);
        self.r_000cba();
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        r.d4 = low_word(d4, 0xFF);
        r.a4 = CELL_BYTES;
        r.c = false;
        loop {
            self.r_000cf4(&mut r);
            self.write8(r.a4, r.d0 as u8);
            r.a4 = r.a4.wrapping_add(1);
            r.c = false;
            if lo_dbra(&mut r.d4) {
                break;
            }
        }
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, START_TASK_PRIORITY, START_TASK);
        (r.d5, r.a5) = self.r_00355e(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        // d0-d4, a0 and a4 put back
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.a0, r.a4) = (d0, d1, d2, d3, d4, a0, a4);
        r.x = self.lo_cell_maps(&mut r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x)
    }

    /// The code at 0x50EE (`func_00108C`'s end): the counts cleared, the
    /// cells' routines `NO_ROUTINE`, `BOUND` set; then for each cell of
    /// the map its byte of `CELL_BYTES` (by the map's byte) goes to both
    /// cell maps (`CELL_AT`: its low nibble, and the high one `SECOND`
    /// on; `CELL_AT_2`: all of it): through its routine (`func_005336`)
    /// when it has `BY_ROUTINE`, else its `KEPT_BITS`.  A cell whose
    /// routine gives C, or without one, without `NO_DRAW` and with
    /// `DRAW_BITS`, gets a number drawn (`func_001BE2`, kept at `DRAWN`);
    /// one below `BOUND` with no object in the cell (`func_005252`) is
    /// marked (`MARK_TILES`, `MARK`, counted in `MARKED`).  Gives back
    /// X (the borrow of the last row count).
    fn lo_cell_maps(&mut self, r: &mut Regs) -> bool {
        self.write8(MARKED, 0);
        self.write16(DRAWN_INDEX, 0);
        self.write8(CLEARED_ACE0, 0);
        r.d0 = NO_ROUTINE;
        r.a0 = FOR_TASK;
        r.a1 = FOR_OBJECT;
        for _ in 0..7 {
            self.write32(r.a0, NO_ROUTINE);
            self.write32(r.a1, NO_ROUTINE);
            r.a0 += 4;
            r.a1 += 4;
        }
        r.d1 = 0xFFFF;
        self.write32(ROUTINE_A9AA, 0x0054AE);
        self.write32(ROUTINE_A9AE, 0x00C886);
        let bound = if self.read8(MODE) & 1 != 0 { 0xF0 } else { 0x70 };
        r.d0 = low_byte(r.d0, bound);
        self.write8(BOUND, bound);
        self.write32(CELL_AT, CELL_MAP);
        self.write32(CELL_AT_2, CELL_MAP_2);
        self.write32(MAP_AT, MAP);
        self.write16(ROW, 0);
        self.write16(POSITION_Y, 0);
        let height = self.read8(HEIGHT);
        self.write8(ROWS_LEFT, height);
        loop {
            self.write8(COLUMN, 0);
            self.write16(POSITION_X, 0);
            let width = self.read8(WIDTH);
            self.write8(CELLS_LEFT, width);
            loop {
                let map_at = self.read32(MAP_AT);
                let cell = self.read8(map_at);
                r.a0 = map_at;
                r.d1 = low_word(r.d1, u16::from(cell));
                r.a1 = CELL_BYTES;
                let byte = self.read8(CELL_BYTES + u32::from(cell));
                r.d0 = low_byte(r.d0, byte);
                let draw = if byte & BY_ROUTINE != 0 {
                    let c;
                    (r.d0, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, c) =
                        self.r_005336(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
                    if self.restarted.is_some() {
                        return r.x;
                    }
                    self.lo_cell_to_maps(r, r.d0 as u8);
                    c
                } else {
                    let kept = byte & KEPT_BITS;
                    r.d0 = low_byte(r.d0, kept);
                    self.lo_cell_to_maps(r, kept);
                    r.a0 = CELL_BYTES;
                    let byte = self.read8(CELL_BYTES + u32::from(cell));
                    r.d0 = low_byte(r.d0, byte);
                    if byte & NO_DRAW != 0 {
                        false
                    } else {
                        let bits = byte & DRAW_BITS;
                        r.d0 = low_byte(r.d0, bits);
                        bits != 0
                    }
                };
                if draw {
                    (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
                    let index = self.read16(DRAWN_INDEX);
                    r.a1 = index as i16 as u32;
                    self.write8(DRAWN.wrapping_add(r.a1), r.d0 as u8);
                    // (a byte add: the index's high byte)
                    let high = self.read8(DRAWN_INDEX);
                    self.write8(DRAWN_INDEX, high.wrapping_add(1));
                    if (r.d0 as u8) < self.read8(BOUND) {
                        let c;
                        (r.d0, r.d4, r.a4, c) = self.r_005252(r.d0);
                        if !c {
                            r.d0 = low_word(r.d0, MARK_TILES);
                            (r.x, _) = self.r_005294(r.d0, r.d1);
                            r.a0 = self.read32(CELL_AT);
                            self.write8(r.a0, MARK);
                            let marked = self.read8(MARKED);
                            self.write8(MARKED, marked.wrapping_add(1));
                        }
                    }
                }
                let column = self.read8(COLUMN);
                self.write8(COLUMN, column.wrapping_add(2));
                let x = self.read16(POSITION_X);
                self.write16(POSITION_X, x.wrapping_add(0x10));
                for at in [CELL_AT, CELL_AT_2, MAP_AT] {
                    let long = self.read32(at);
                    self.write32(at, long.wrapping_add(1));
                }
                let left = self.read8(CELLS_LEFT).wrapping_sub(1);
                self.write8(CELLS_LEFT, left);
                if left == 0 {
                    break;
                }
            }
            let row = self.read16(ROW);
            self.write16(ROW, row.wrapping_add(ROW_STEP));
            let y = self.read16(POSITION_Y);
            self.write16(POSITION_Y, y.wrapping_add(0x10));
            let (rows, borrow) = self.read8(ROWS_LEFT).overflowing_sub(1);
            self.write8(ROWS_LEFT, rows);
            if rows == 0 {
                return borrow;
            }
        }
    }

    /// A cell's byte `byte` into the cell maps: at `CELL_AT` its low
    /// nibble, `SECOND` on its high one, at `CELL_AT_2` all of it; a0
    /// left at `CELL_AT_2`'s cell.
    fn lo_cell_to_maps(&mut self, r: &mut Regs, byte: u8) {
        let at = self.read32(CELL_AT);
        self.write8(at, byte & 0x0F);
        self.write8(at.wrapping_add(SECOND), byte & 0xF0);
        r.a0 = self.read32(CELL_AT_2);
        self.write8(r.a0, byte);
    }

    /// `func_0010C8` (the task `func_00108C` makes; it runs once): unless
    /// the objects are stopped (`func_001BB4`; then it returns), the
    /// task ends (`func_000560`) and (the code at 0xBD8A) 0x340 bytes
    /// from 0xFF0000, the long after them and 0x50 longs from 0xFFAAC0
    /// are cleared, the rows' tables made (`func_0010EC`), on `MODE`
    /// bit 0 the Z80 sent `START_COMMAND`, and the task `func_00BDD8`
    /// made (priority 0x70), `CLEARED_AC7C` cleared.  Gives back d0,
    /// d1, d6, a0, a6, X and C (clear).
    pub fn r_0010c8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let (c, stopped) = self.r_001bb4();
        r.c = c;
        if stopped {
            return;
        }
        (r.d6, r.x, _) = self.r_000560(r.d5, r.d6, r.x);
        (r.d0, r.d1, _, r.x) = self.r_0011d2(CLEARED_FF0000_BYTES, r.d1, CLEARED_FF0000);
        self.write32(CLEARED_FF0000 + CLEARED_FF0000_BYTES, 0);
        for k in 0..CLEARED_AAC0_LONGS {
            self.write32(CLEARED_AAC0 + 4 * k, 0);
        }
        r.d0 = 0xFFFF;
        r.a0 = CLEARED_AAC0 + 4 * CLEARED_AAC0_LONGS;
        self.r_0010ec();
        if self.read8(MODE) & 1 != 0 {
            r.d0 = low_byte(r.d0, START_COMMAND);
            self.r_0034b6(r.d0);
        }
        (r.d6, r.a6, r.x, _) = self.r_0003e8(r.d4, r.d5, r.d6, NEXT_TASK_PRIORITY, NEXT_TASK);
        self.write8(CLEARED_AC7C, 0);
        r.c = false;
    }
}
