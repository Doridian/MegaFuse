//! The Z80's program loaded (`func_00B7F4`) and its reset
//! (`func_00B832`).

use crate::Game;

/// The Z80's bus request (bit 8 written: the 68000 wants the bus; bit
/// 0 read: set until the Z80 has let it go) and its reset (bit 8
/// written: let go).
const BUSREQ: u32 = 0xA11100;
const RESET: u32 = 0xA11200;
/// The Z80's RAM, as the 68000 sees it.
const Z80: u32 = 0xA00000;
/// The Z80's program in the ROM (`D_0F8000`): its length is the
/// little-endian word at 6 in it.
const PROGRAM: u32 = 0x0F8000;
const PROGRAM_LENGTH: u32 = PROGRAM + 6;

impl Game {
    /// `func_00B7F4`: the Z80's program (`PROGRAM`) copied into its
    /// RAM, with the bus taken and the Z80 reset before and after.
    /// Gives back a0 and a1 (past the copies).
    pub fn r_00b7f4(&mut self, _d0: u32) -> (u32, u32) {
        self.write16(BUSREQ, 0x100);
        self.r_00b832();
        while self.read8(BUSREQ) & 1 != 0 {
            self.pass(30);
        }
        let low = self.read8(PROGRAM_LENGTH);
        let high = self.read8(PROGRAM_LENGTH + 1);
        // a `dbra` loop: a length of 0 copies 0x10000 bytes
        let count = u32::from(u16::from_le_bytes([low, high]).wrapping_sub(1)) + 1;
        for k in 0..count {
            let byte = self.read8(PROGRAM + k);
            self.write8(Z80 + k, byte);
        }
        self.r_00b832();
        self.write16(BUSREQ, 0);
        (PROGRAM + count, Z80 + count)
    }

    /// `func_00B832`: the Z80 reset (held, then let go).
    pub fn r_00b832(&mut self) {
        self.write16(RESET, 0);
        // held a while: 11 registers pushed and popped
        self.pass(196);
        self.write16(RESET, 0x100);
    }
}
