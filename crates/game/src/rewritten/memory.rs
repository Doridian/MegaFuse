//! Memory cleared.

use crate::Game;

impl Game {
    /// `func_0011D2`: `d0` bytes (its low word) from `a0` cleared, by
    /// longs, words or bytes as the address and the length allow (a
    /// length of 0 by longs clears 0x40000 bytes).  Gives back d0 (its
    /// low word 0xFFFF), d1, a0 (past the end) and X.
    pub fn r_0011d2(&mut self, d0: u32, d1: u32, a0: u32) -> (u32, u32, u32, bool) {
        let len = d0 as u16;
        let d0_out = (d0 & 0xFFFF_0000) | 0xFFFF;
        let mut at = a0;
        if a0 & 1 != 0 || len & 1 != 0 {
            let n = len.wrapping_sub(1);
            for _ in 0..=u32::from(n) {
                self.write8(at, 0);
                at = at.wrapping_add(1);
            }
            return (d0_out, (d1 & 0xFFFF_0000) | (a0 & 0xFF00), at, len == 0);
        }
        let (size, n) = if len & 2 != 0 { (2, len >> 1) } else { (4, len >> 2) };
        for _ in 0..=u32::from(n.wrapping_sub(1)) {
            if size == 2 {
                self.write16(at, 0);
            } else {
                self.write32(at, 0);
            }
            at = at.wrapping_add(size);
        }
        (d0_out, 0, at, n == 0)
    }
}
