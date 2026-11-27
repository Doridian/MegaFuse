//! Map cells looked up by position: a cell (column, row) at `CELL_X`,
//! `CELL_Y` from a position (16 pixels a cell), its index in the map at
//! `MAP_AT` (`func_001652`), and the bytes kept for it in the cell map
//! from `CELL_MAP`.
//!
//! A position is a word of pixels with a byte of fraction before it:
//! an object (a4) has x at 0x18 (fraction 0x16), y at 0x1A (fraction
//! 0x17); a task (a6) x at 0x0E (fraction 0x0C), y at 0x10 (fraction
//! 0x0D).  The low byte of the pixel word is what the cell look-ups
//! work on: bits 4-5 change when a step takes it into the next cell.

use crate::Game;

/// A position (words, x then y; one long).
const POSITION: u32 = 0xFF9F04;
/// A cell (column, row).
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
/// Another position: x (a word, its low byte at `POS_X_LOW`), y (a
/// word, its low byte at `POS_Y_LOW`).
const POS_X: u32 = 0xFF9EFA;
const POS_X_LOW: u32 = 0xFF9EFB;
const POS_Y: u32 = 0xFF9EFE;
const POS_Y_LOW: u32 = 0xFF9EFF;
/// Bytes `func_0017B8` compares the moved `POS_X_LOW`, `POS_Y_LOW` with
/// (bits 4-5).
const FROM_X_LOW: u32 = 0xFF9F02;
const FROM_Y_LOW: u32 = 0xFF9F03;

/// The map's width (in cells, a byte).
const WIDTH: u32 = 0xFF9EB2;
/// The map's index of each row's first cell (a long a row, `map.rs`).
const ROW_STARTS: u32 = 0xFFA31A;
/// The map's index of the cell at `CELL_X`, `CELL_Y` (a long).
const MAP_AT: u32 = 0xFF9ECA;
/// The cell map: a byte a cell, by the map's index.  `func_00168C`
/// puts a cell's low nibble here and its high nibble 0x200 bytes on.
const CELL_MAP: u32 = 0xFFFF989C;
/// `func_00167A` keeps the cell's address in it (a long).
const CELL_AT: u32 = 0xFF9ED6;
/// A byte a cell, by the map's index, that `func_00168C` splits into
/// the cell map.
const CELL_SOURCE: u32 = 0xFFFF96FC;
/// Where the cell map's high nibbles are, from a cell's.
const HIGH_NIBBLES: u32 = 0x200;
/// Two more tables by the map's index (`func_0016B0` keeps the cell's
/// addresses in them: `TABLE_1A0_AT`, `TABLE_0_AT`).
const TABLE_1A0: u32 = 0xFFFF01A0;
const TABLE_0: u32 = 0xFFFF0000;
const TABLE_1A0_AT: u32 = 0xFF9EE2;
const TABLE_0_AT: u32 = 0xFF9EDE;

/// The 3x3 cells around `CELL_X`, `CELL_Y` (`func_00157A`), by row: the
/// cell itself is the fifth.
const AROUND: u32 = 0xFFFF9F0C;
const AROUND_MIDDLE: u16 = 4;
/// What `func_0017B8` finds in `AROUND`.
const FOUND: u32 = 0xFF9EF6;

/// `func_0016CE`'s steps by direction (0 up, 1 right, 2 down, 3 left):
/// columns (4 bytes, D_0016EE), then rows (4 bytes).
const STEPS_2: u32 = 0x0016EE;
/// `func_0016F6`'s (D_00171E), one cell a step.
const STEPS_1: u32 = 0x00171E;
/// Pixels moved by direction, x (D_0017A6) and y (D_0017AF), signed
/// bytes, 9 of each: 0 up 9 pixels, 1 right 8, 2 down 8, 3 left 9, 4
/// none, 5-8 up, right, down, left 16.
const MOVE_X: u32 = 0x0017A6;
const MOVE_Y: u32 = 0x0017AF;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// Whether moving the low byte of a position `low` by `step` pixels
/// changes its bits 4-5 (it crosses into another cell).
fn crosses(low: u8, step: u8) -> bool {
    (step.wrapping_add(low) ^ low) & 0x30 != 0
}

impl Game {
    /// `func_00157A`: the cells around `CELL_X`, `CELL_Y` (3 rows of 3,
    /// from the one up and left) copied from the cell map to `AROUND`.
    /// Where the first is is reckoned for a map 16 cells wide when
    /// `WIDTH` is 16, else 32 (the rows themselves are `WIDTH` apart).
    pub fn r_00157a(&mut self) {
        self.r_001652();
        let width = self.read8(WIDTH);
        let up_left: u32 = if width == 0x10 { 0x11 } else { 0x21 };
        let from = self.read32(MAP_AT).wrapping_add(CELL_MAP).wrapping_sub(up_left);
        let width = u32::from(width);
        for row in 0..3 {
            for column in 0..3 {
                let cell = self.read8(from.wrapping_add(row * width + column));
                self.write8(AROUND + 3 * row + column, cell);
            }
        }
    }

    /// The cell at `position` (x in the high word, y in the low; 16
    /// pixels a cell) to `CELL_X`, `CELL_Y`.  Gives back the long
    /// shifted (and swapped) as the routines leave d0, and X.
    fn cell_of(&mut self, position: u32) -> (u32, bool) {
        let shifted = position >> 4;
        self.write8(CELL_Y, shifted as u8);
        let swapped = shifted.rotate_left(16);
        self.write8(CELL_X, swapped as u8);
        (swapped, position & 0x8 != 0)
    }

    /// `func_0015D4`: the cell at `POSITION`.  Gives back d0.
    pub fn r_0015d4(&mut self) -> u32 {
        let position = self.read32(POSITION);
        self.cell_of(position).0
    }

    /// `func_0015E6`: the cell of the object at a4.  Gives back d0 and X.
    pub fn r_0015e6(&mut self, a4: u32) -> (u32, bool) {
        let position = self.read32(a4 + 0x18);
        self.cell_of(position)
    }

    /// `func_0015F8`: the cell of the task at a6.  Gives back d0.
    pub fn r_0015f8(&mut self, a6: u32) -> u32 {
        let position = self.read32(a6 + 0x0E);
        self.cell_of(position).0
    }

    /// `func_00160A`: the cell at `POS_X`, `POS_Y`.
    pub fn r_00160a(&mut self) {
        let column = self.read16(POS_X) >> 4;
        self.write8(CELL_X, column as u8);
        let row = self.read16(POS_Y) >> 4;
        self.write8(CELL_Y, row as u8);
    }

    /// `func_001652`: `MAP_AT`, the map's index of the cell at `CELL_X`,
    /// `CELL_Y`.  Gives back X (from the row x 4, which never carries:
    /// clear).
    pub fn r_001652(&mut self) -> bool {
        let row = u32::from(self.read8(CELL_Y));
        let start = self.read32(ROW_STARTS + 4 * row);
        let column = u32::from(self.read8(CELL_X));
        self.write32(MAP_AT, start.wrapping_add(column));
        false
    }

    /// `func_00167A`: the cell's address in the cell map, kept in
    /// `CELL_AT`.  Gives back a0 (it) and X.
    pub fn r_00167a(&mut self) -> (u32, bool) {
        let x = self.r_001652();
        let at = CELL_MAP.wrapping_add(self.read32(MAP_AT));
        self.write32(CELL_AT, at);
        (at, x)
    }

    /// `func_00168C`: the cell's byte from `CELL_SOURCE` split into the
    /// cell map at `CELL_AT`: its low nibble there, its high nibble
    /// `HIGH_NIBBLES` on.  Gives back d0 (the byte) and a0 (`CELL_AT`).
    pub fn r_00168c(&mut self, d0: u32) -> (u32, u32) {
        let index = self.read32(MAP_AT);
        let byte = self.read8(index.wrapping_add(CELL_SOURCE));
        let at = self.read32(CELL_AT);
        self.write8(at, byte & 0x0F);
        self.write8(at.wrapping_add(HIGH_NIBBLES), byte & 0xF0);
        (low_word(d0, u16::from(byte)), at)
    }

    /// `func_0016AC`: `MAP_AT` (`func_001652`), then `func_0016B0`.
    /// Gives back a0 and X.
    pub fn r_0016ac(&mut self) -> (u32, bool) {
        let x = self.r_001652();
        (self.r_0016b0(), x)
    }

    /// `func_0016B0`: the cell's addresses in `TABLE_1A0` and
    /// `TABLE_0`, kept in `TABLE_1A0_AT` and `TABLE_0_AT`.  Gives back
    /// a0 (the second).
    pub fn r_0016b0(&mut self) -> u32 {
        let index = self.read32(MAP_AT);
        self.write32(TABLE_1A0_AT, TABLE_1A0.wrapping_add(index));
        let at = TABLE_0.wrapping_add(index);
        self.write32(TABLE_0_AT, at);
        at
    }

    /// Moves `CELL_X`, `CELL_Y` by direction `d0` (its low byte, signed)
    /// in `steps`.  Gives back d0 (the direction made a word), d1 (the
    /// row's step in its low byte) and the new row with its carry.
    fn step_cell(&mut self, d0: u32, d1: u32, steps: u32) -> (u32, u32, u8, bool) {
        let direction = d0 as u8 as i8 as i16;
        let d0 = low_word(d0, direction as u16);
        let by = steps.wrapping_add(direction as i32 as u32);
        let column_step = self.read8(by);
        let column = self.read8(CELL_X).wrapping_add(column_step);
        self.write8(CELL_X, column);
        let row_step = self.read8(by + 4);
        let (row, carry) = self.read8(CELL_Y).overflowing_add(row_step);
        self.write8(CELL_Y, row);
        (d0, low_byte(d1, row_step), row, carry)
    }

    /// `func_0016CE`: the cell two cells away in direction `d0` (0 up,
    /// 1 right, 2 down, 3 left): `CELL_X`, `CELL_Y` moved, its address
    /// in the cell map (`func_00167A`).  Gives back d0 (its byte there in
    /// the low byte) and d1.
    pub fn r_0016ce(&mut self, d0: u32, d1: u32) -> (u32, u32) {
        let (d0, d1, ..) = self.step_cell(d0, d1, STEPS_2);
        let (at, _) = self.r_00167a();
        let cell = self.read8(at);
        (low_byte(d0, cell), d1)
    }

    /// `func_0016F6`: the next cell in direction `d0`, as `func_0016CE`;
    /// none (d0 0) when the row comes to 0.  Gives back d0, d1, a0 and X.
    pub fn r_0016f6(&mut self, d0: u32, d1: u32) -> (u32, u32, u32, bool) {
        let (d0, d1, row, carry) = self.step_cell(d0, d1, STEPS_1);
        if row == 0 {
            return (0, d1, STEPS_1, carry);
        }
        let (at, x) = self.r_00167a();
        let cell = self.read8(at);
        (low_byte(d0, cell), d1, at, x)
    }

    /// `func_001726`: `func_001756` for the task at a6.  Gives back X.
    pub fn r_001726(&mut self, d0: u32, a6: u32) -> bool {
        let x_low = self.read8(a6 + 0x0F);
        let y_low = self.read8(a6 + 0x11);
        self.r_001756(d0, u32::from(x_low), u32::from(y_low))
    }

    /// `func_00173E`: `func_001756` for the object at a4.  Gives back X.
    pub fn r_00173e(&mut self, d0: u32, a4: u32) -> bool {
        let x_low = self.read8(a4 + 0x19);
        let y_low = self.read8(a4 + 0x1B);
        self.r_001756(d0, u32::from(x_low), u32::from(y_low))
    }

    /// `func_001756`: `CELL_X`, `CELL_Y` moved a cell where the move of
    /// direction `d0` (`MOVE_X`, `MOVE_Y`) takes a position whose low
    /// bytes are `d2` (x) and `d3` (y) into another cell; then its
    /// address in the cell map (`func_00167A`).  Gives back X.
    pub fn r_001756(&mut self, d0: u32, d2: u32, d3: u32) -> bool {
        let direction = d0 as u8 as i8 as i32 as u32;
        for (moves, low, cell) in [(MOVE_X, d2, CELL_X), (MOVE_Y, d3, CELL_Y)] {
            let step = self.read8(moves.wrapping_add(direction));
            if crosses(low as u8, step) {
                let at = self.read8(cell);
                let at = if (step as i8) < 0 { at.wrapping_sub(1) } else { at.wrapping_add(1) };
                self.write8(cell, at);
            }
        }
        self.r_00167a().1
    }

    /// `func_0017B8`: the cell of `AROUND` that moving `POS_X`, `POS_Y`
    /// by `d0`, `d1` (low bytes, signed) takes it to: the middle one, a
    /// column or row over where the moved low byte's bits 4-5 differ
    /// from `FROM_X_LOW`'s, `FROM_Y_LOW`'s.  Kept in `FOUND`.  Gives back
    /// d0 (it in the low byte), X and C (set when it is 0).
    pub fn r_0017b8(&mut self, d0: u32, d1: u32) -> (u32, bool, bool) {
        let mut index = AROUND_MIDDLE;
        let moved_x = (d0 as u8).wrapping_add(self.read8(POS_X_LOW));
        if (moved_x ^ self.read8(FROM_X_LOW)) & 0x30 != 0 {
            index = if d0 & 0x80 != 0 { index - 1 } else { index + 1 };
        }
        let (moved_y, mut x) = (d1 as u8).overflowing_add(self.read8(POS_Y_LOW));
        if (moved_y ^ self.read8(FROM_Y_LOW)) & 0x30 != 0 {
            (index, x) = if d1 & 0x80 != 0 { index.overflowing_sub(3) } else { index.overflowing_add(3) };
        }
        let cell = self.read8(AROUND.wrapping_add(index as i16 as i32 as u32));
        self.write8(FOUND, cell);
        (low_byte(d0, cell), x, cell == 0)
    }

    /// Writes `value` to the cell at `CELL_AT`.  Gives back a0 (it).
    fn set_cell(&mut self, value: u8) -> u32 {
        let at = self.read32(CELL_AT);
        self.write8(at, value);
        at
    }

    /// `func_00197A`: 1 in the cell at `CELL_AT`.  Gives back a0.
    pub fn r_00197a(&mut self) -> u32 {
        self.set_cell(1)
    }

    /// `func_001984`: 8 in the cell at `CELL_AT`.  Gives back a0.
    pub fn r_001984(&mut self) -> u32 {
        self.set_cell(8)
    }

    /// `func_00198E`: 7 in the cell at `CELL_AT`.  Gives back a0.
    pub fn r_00198e(&mut self) -> u32 {
        self.set_cell(7)
    }

    /// `func_001998`: 3 in the cell at `CELL_AT`.  Gives back a0.
    pub fn r_001998(&mut self) -> u32 {
        self.set_cell(3)
    }

    /// `func_0019A2`: 5 in the cell at `CELL_AT`.  Gives back a0.
    pub fn r_0019a2(&mut self) -> u32 {
        self.set_cell(5)
    }

    /// `func_0019AC`: 2 in the cell at `CELL_AT`.  Gives back a0.
    pub fn r_0019ac(&mut self) -> u32 {
        self.set_cell(2)
    }

    /// `func_0019B6`: 0 in the cell at `CELL_AT`.  Gives back a0.
    pub fn r_0019b6(&mut self) -> u32 {
        self.set_cell(0)
    }
}
