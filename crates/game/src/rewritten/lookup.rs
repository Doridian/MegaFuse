//! Look-ups of a pointer by an index in a table of longs.

use crate::Game;

/// `func_001CFA`'s table (D_001D1E).
const TABLE_1D1E: u32 = 0x001D1E;
/// `func_0005F8`'s table (D_00062E).
const TABLE_062E: u32 = 0x00062E;

/// The condition codes a look-up leaves: those of the index's low word
/// (the `move.w` that saves and restores it), V and C clear, X clear
/// (index x 4 doesn't carry out of the word).
fn index_flags(index: u32) -> (bool, bool, bool, bool, bool) {
    let w = index as u16;
    (false, w & 0x8000 != 0, w == 0, false, false)
}

impl Game {
    /// The long at `table` + 4 x the low byte of `index`.
    fn long_by_index(&mut self, table: u32, index: u32) -> u32 {
        self.read32(table + 4 * (index & 0xFF))
    }

    /// `func_001CFA`: a4 = D_001D1E[d4 & 0xFF]; d4 is kept.
    pub fn r_001cfa(&mut self, d4: u32) -> (u32, u32, bool, bool, bool, bool, bool) {
        let a4 = self.long_by_index(TABLE_1D1E, d4);
        let (x, n, z, v, c) = index_flags(d4);
        (d4, a4, x, n, z, v, c)
    }

    /// `func_0005F8`: a6 = D_00062E[d6 & 0xFF]; d6 is kept.
    pub fn r_0005f8(&mut self, d6: u32) -> (u32, u32, bool, bool, bool, bool, bool) {
        let a6 = self.long_by_index(TABLE_062E, d6);
        let (x, n, z, v, c) = index_flags(d6);
        (d6, a6, x, n, z, v, c)
    }
}
