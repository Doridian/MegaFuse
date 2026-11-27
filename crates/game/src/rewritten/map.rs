//! The stage's map on plane A (VRAM 0xE000 on): cells of 2x2 tiles,
//! drawn where `COLUMN` and `ROW` say.

use crate::Game;

/// The VDP's control port and data port.
const CONTROL: u32 = 0xC00004;
const DATA: u32 = 0xC00000;

/// Where the next cell goes: its column x 2 (a byte, in tiles) and its
/// row's offset (a word: 0x80, a row of cells, each line 0x40 words).
const COLUMN: u32 = 0xFF9E74;
const ROW: u32 = 0xFF9E76;
/// Where the cell is drawn from: 0xE100 on (two lines down).
const PLANE: u16 = 0xE100;
/// The cells' tiles: 8 bytes a cell (4 words: the top two, the bottom
/// two), from the address in this long.
const CELLS: u32 = 0xFF9E6C;
/// The map: a byte a cell, from 0xFF955C; `MAP_AT` (a long) the one
/// `func_0010D6` draws.
const MAP: u32 = 0xFFFF955C;
const MAP_AT: u32 = 0xFF9ECA;
/// Each row's offset (a word every 4 bytes) and the map's index of its
/// first cell (a long each), 26 rows; the map's width (a byte).
const ROW_OFFSETS: u32 = 0xFFA382;
const ROW_STARTS: u32 = 0xFFA31A;
const ROWS: u16 = 26;
const WIDTH: u32 = 0xFF9EB2;
/// Running sums while the tables are made.
const ROW_START: u32 = 0xFF9EDA;
/// A cell (column, row) to draw (`func_001278`).
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_000FBE`: map cell `d0` (its low byte) drawn (`func_000FDA`).
    /// Gives back d0, X and C as that leaves them.
    pub fn r_000fbe(&mut self, d0: u32) -> (u32, bool, bool) {
        let by_cell = ((d0 as u16) & 0xFF) << 3;
        let tiles = (by_cell as i16 as u32).wrapping_add(self.read32(CELLS));
        self.r_000fda(d0, tiles)
    }

    /// `func_000FDA`: a cell's 4 tile words from `tiles`, two on a line
    /// and two on the next, at `COLUMN` and `ROW`, interrupts masked.
    /// Gives back d0 (the second line's VRAM write command), X (the
    /// carry of its address) and C (clear).
    pub fn r_000fda(&mut self, d0: u32, tiles: u32) -> (u32, bool, bool) {
        let column = u16::from(self.read8(COLUMN));
        let at = column.wrapping_add(self.read16(ROW)).wrapping_mul(2).wrapping_add(PLANE);
        let mut x = false;
        let mut d0 = d0;
        for (line, from) in [(0u16, tiles), (0x80, tiles + 4)] {
            let (at, carry) = at.overflowing_add(line);
            if line != 0 {
                x = carry;
            }
            d0 = self.r_000282(low_word(d0, at)).0;
            let ipl = self.ipl();
            self.set_ipl(7);
            self.write32(CONTROL, d0);
            let first = self.read16(from);
            self.write16(DATA, first);
            let second = self.read16(from + 2);
            self.write16(DATA, second);
            self.set_ipl(ipl);
        }
        (d0, x, false)
    }

    /// `func_0010D6`: the map's cell at `MAP_AT` drawn.  Gives back X.
    pub fn r_0010d6(&mut self, d0: u32) -> bool {
        let at = self.read32(MAP_AT);
        let cell = self.read8(at.wrapping_add(MAP));
        self.r_000fbe((d0 & !0xFF) | u32::from(cell)).1
    }

    /// `func_0010EC`: the rows' tables: each row's offset (0x80 more
    /// each) and the map's index of its first cell (`WIDTH` more each).
    pub fn r_0010ec(&mut self) {
        self.write16(ROW, 0);
        self.write32(ROW_START, 0);
        let width = u32::from(self.read8(WIDTH));
        for row in 0..u32::from(ROWS) {
            let offset = self.read16(ROW);
            self.write16(ROW_OFFSETS + 4 * row, offset);
            let start = self.read32(ROW_START);
            self.write32(ROW_STARTS + 4 * row, start);
            self.write16(ROW, offset.wrapping_add(0x80));
            self.write32(ROW_START, start.wrapping_add(width));
        }
    }

    /// `func_001278`: `COLUMN` and `ROW` for the cell at `CELL_X`,
    /// `CELL_Y`.
    pub fn r_001278(&mut self) {
        let x = self.read8(CELL_X);
        self.write8(COLUMN, x.wrapping_add(x));
        let y = u32::from(self.read8(CELL_Y));
        let offset = self.read16(ROW_OFFSETS + 4 * y);
        self.write16(ROW, offset);
    }
}
