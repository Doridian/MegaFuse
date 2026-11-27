//! The planes' scrolling (0x8624-0x8775): the offsets added to the
//! scroll values, register 11's scroll modes, and the tables of scroll
//! values by line and by column made from a list of runs.

use crate::{Game, Regs};

/// The VDP's control port.
const CONTROL: u32 = 0xC00004;
/// Register 11 as the game keeps it (`0x8Bxx`): bits 0-1 of its low
/// byte the horizontal scroll's mode (3 by line), bit 2 the vertical's
/// (by column).
const REG11: u32 = 0xFFFF_F01A;
const REG11_LOW: u32 = REG11 + 1;
const H_MODE: u8 = 3;
const H_BY_CELL: u8 = 2;
const V_BY_COLUMN: u8 = 1 << 2;

/// The horizontal scroll table in RAM: by line, a word for plane A
/// and one for plane B.
const H_TABLE: u32 = 0xFFFF_D000;
/// The word `func_0086BA` starts the table from, and `func_008664`
/// adds `H_OFFSET` to into the table's first.
const H_VALUE: u32 = H_TABLE + 2;
/// The vertical scroll table in RAM: by column, plane A's and plane
/// B's.
const V_TABLE: u32 = 0xFFFF_D400;
const V_VALUE: u32 = V_TABLE + 2;
/// Offsets `func_008664` adds (set by `func_008624` ...
/// `func_008656`), and a third word they set.
const H_OFFSET: u32 = 0xFFFF_8064;
const V_OFFSET: u32 = 0xFFFF_8066;
const OFFSET_8068: u32 = 0xFFFF_8068;

/// The lines and columns the tables have.
const LINES: u16 = 0xE0;
const COLUMNS: u16 = 0x10;

impl Game {
    /// `func_008624`: the offsets (and `OFFSET_8068`) 0.
    pub fn r_008624(&mut self) {
        self.write16(H_OFFSET, 0);
        self.write16(V_OFFSET, 0);
        self.write16(OFFSET_8068, 0);
    }

    /// `func_008632`: offsets 0 and 0xD0, `OFFSET_8068` 0xEE00.  Gives
    /// back N set, Z and C clear (the last move).
    pub fn r_008632(&mut self, regs: &mut Regs) {
        self.write16(H_OFFSET, 0);
        self.write16(V_OFFSET, 0xD0);
        self.write16(OFFSET_8068, 0xEE00);
        regs.n = true;
        regs.z = false;
        regs.c = false;
    }

    /// `func_008644`: offsets 0x100 and 0, `OFFSET_8068` 0xE140.
    pub fn r_008644(&mut self) {
        self.write16(H_OFFSET, 0x100);
        self.write16(V_OFFSET, 0);
        self.write16(OFFSET_8068, 0xE140);
    }

    /// `func_008656`: offsets 0x100 and 0x20.
    /// No script calls it: reviewed by reading.
    pub fn r_008656(&mut self) {
        self.write16(H_OFFSET, 0x100);
        self.write16(V_OFFSET, 0x20);
    }

    /// `func_008664`: the tables' first words: `H_VALUE` + `H_OFFSET`,
    /// `V_VALUE` + `V_OFFSET`.  Gives back d0's low word (the second
    /// sum), X (its carry), N and Z (by it) and C clear.
    pub fn r_008664(&mut self, regs: &mut Regs) {
        let h = self.read16(H_VALUE).wrapping_add(self.read16(H_OFFSET));
        self.write16(H_TABLE, h);
        let value = self.read16(V_VALUE);
        let (v, carry) = value.overflowing_add(self.read16(V_OFFSET));
        self.write16(V_TABLE, v);
        regs.d0 = (regs.d0 & 0xFFFF_0000) | u32::from(v);
        regs.x = carry;
        regs.n = v & 0x8000 != 0;
        regs.z = v == 0;
        regs.c = false;
    }

    /// Register 11's low byte changed by `change`, the register to the
    /// VDP.
    fn set_reg11(&mut self, change: impl FnOnce(u8) -> u8) {
        let low = self.read8(REG11_LOW);
        self.write8(REG11_LOW, change(low));
        let reg11 = self.read16(REG11);
        self.write16(CONTROL, reg11);
    }

    /// `func_00867E`: the horizontal scroll by the whole screen.
    pub fn r_00867e(&mut self) {
        self.set_reg11(|r| r & !H_MODE);
    }

    /// `func_00868E`: the horizontal scroll by cell (8 lines).
    /// No script calls it: reviewed by reading.
    pub fn r_00868e(&mut self) {
        self.set_reg11(|r| (r & !H_MODE) | H_BY_CELL);
    }

    /// `func_0086A4`: the horizontal scroll table by line from the runs
    /// at `runs` (`func_0086BA`), the scroll by line.
    /// No script calls it: reviewed by reading.
    pub fn r_0086a4(&mut self, runs: u32) {
        self.r_0086ba(runs);
        self.set_reg11(|r| r | H_MODE);
    }

    /// `func_0086BA`: the horizontal scroll table's `LINES` lines from
    /// the runs at `runs` (`fill_runs`), each line's two words the
    /// value.
    /// No script calls it: reviewed by reading.
    pub fn r_0086ba(&mut self, runs: u32) {
        self.fill_runs(runs, LINES, H_VALUE, V_VALUE, H_TABLE, false);
    }

    /// `func_008706`: the vertical scroll by the whole screen.
    /// No script calls it: reviewed by reading.
    pub fn r_008706(&mut self) {
        self.set_reg11(|r| r & !V_BY_COLUMN);
    }

    /// `func_008716`: the vertical scroll table by column from the runs
    /// at `runs` (`func_00872C`), the scroll by column.
    /// No script calls it: reviewed by reading.
    pub fn r_008716(&mut self, runs: u32) {
        self.r_00872c(runs);
        self.set_reg11(|r| r | V_BY_COLUMN);
    }

    /// `func_00872C`: the vertical scroll table's `COLUMNS` columns from
    /// the runs at `runs` (`fill_runs`), each column's two words the
    /// value.
    /// No script calls it: reviewed by reading.
    pub fn r_00872c(&mut self, runs: u32) {
        self.fill_runs(runs, COLUMNS, H_VALUE, V_VALUE, V_TABLE, true);
    }

    /// A scroll table of `entries` entries (two words each, both the
    /// value) from runs: the first run's values the words at `h` and
    /// `v`; a run is a byte, its length (0xFF: to the end; 0: 0x10000),
    /// a byte skipped, then two pointers, `h` and `v` for the next run.
    /// The value is the word at `v` if `vertical`, else at `h`.
    fn fill_runs(&mut self, runs: u32, entries: u16, h: u32, v: u32, table: u32, vertical: bool) {
        let (mut h, mut v) = (h, v);
        let mut at = runs;
        let mut to = table;
        let mut left = entries;
        loop {
            let value = self.read16(if vertical { v } else { h });
            let length = self.read8(at);
            at = at.wrapping_add(2);
            let run = if length == 0xFF { u32::from(left) } else { u32::from(u16::from(length).wrapping_sub(1)) + 1 };
            for _ in 0..run {
                self.write16(to, value);
                self.write16(to + 2, value);
                to += 4;
                left -= 1;
                if left == 0 {
                    return;
                }
            }
            h = self.read32(at);
            v = self.read32(at.wrapping_add(4));
            at = at.wrapping_add(8);
        }
    }
}
