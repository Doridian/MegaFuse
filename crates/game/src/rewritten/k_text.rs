//! Reading a string of bytes (`func_00BE40`, `func_00BE74`) and a
//! position on plane 0xE000 (`func_00BE5A`).

use crate::Game;

/// Where the next byte is read (a long), and the byte last read.
const READ_AT: u32 = 0xFFFF_AC82;
const LAST: u32 = 0xFFFF_AC7F;
/// The bits 5-6 `func_00BE74` reads (a hex digit's low two bits).
const DIGIT_BITS: u32 = 0xFFFF_AC7E;
/// A VRAM address on the table at 0xE000 (`func_00BE5A`; 32 cells a
/// row, 2 bytes a cell).
const CELL: u32 = 0xFFFF_AC86;
const TABLE: u16 = 0xE000;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_00BE40`: the next byte (at `READ_AT`, which moves on),
    /// kept in `LAST`.  Gives back d0 (its low byte) and N, Z by it.
    /// No script runs it (checked by reading).
    pub fn r_00be40(&mut self, d0: u32) -> (u32, bool, bool) {
        let at = self.read32(READ_AT);
        let byte = self.read8(at);
        self.write8(LAST, byte);
        self.write32(READ_AT, at.wrapping_add(1));
        (low_byte(d0, byte), (byte as i8) < 0, byte == 0)
    }

    /// `func_00BE5A`: `CELL` the address of column `d1` (5 bits), row
    /// `d2` + 6 (5 bits).  Gives back d1 and d2 (the column, the
    /// address: low words) and X.
    /// No script runs it (checked by reading).
    pub fn r_00be5a(&mut self, d1: u32, d2: u32) -> (u32, u32, bool) {
        let column = d1 as u16 & 0x1F;
        let row = (d2 as u16).wrapping_add(6) & 0x1F;
        let offset = (row << 5 | column) * 2;
        let (cell, x) = offset.overflowing_add(TABLE);
        self.write16(CELL, cell);
        (low_word(d1, column), low_word(d2, cell), x)
    }

    /// `func_00BE74`: the next byte read as a hex digit ('0'-'9',
    /// 'A'-'F'), its low two bits put in bits 5-6 of `DIGIT_BITS`.
    /// Gives back X (the digit's bit 3, shifted out).
    /// No script runs it (checked by reading).
    pub fn r_00be74(&mut self, d0: u32) -> bool {
        let (d0, _, _) = self.r_00be40(d0);
        let byte = d0 as u8;
        let digit = if byte < 0x3A { byte.wrapping_sub(0x30) } else { byte.wrapping_sub(0x37) };
        self.write8(DIGIT_BITS, (digit << 5) & 0x60);
        digit & 0x08 != 0
    }
}
