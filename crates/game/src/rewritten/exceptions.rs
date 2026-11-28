//! The exception vectors' handlers (0x200-0x267).  Each fault's
//! handler (bus and address errors, illegal instructions, a division
//! by zero, `chk`, `trapv`, privilege, trace, the line A and F codes,
//! the spurious interrupt, the reserved vectors and the interrupt
//! levels the game never lets through)
//! saves a0, reads the faulting pc from the frame, and loops for ever:
//! a crash, which the port reports as a panic.  After each loop comes
//! an `rte` no path reaches (0x208, 0x214, ...).  The traps (vectors
//! 32-47) return at once.

use crate::Game;

impl Game {
    /// A handler's loop: the game has crashed.
    fn crashed(&mut self, handler: u32) -> ! {
        panic!("exception: the game crashed (its handler at {handler:#08X})");
    }

    /// `func_000200`: a bus error.
    pub fn r_000200(&mut self) {
        self.crashed(0x200);
    }

    /// `func_000208`: the `rte` after 0x200's loop.
    pub fn r_000208(&mut self) {
        self.crashed(0x208);
    }

    /// `func_00020C`: an address error.
    pub fn r_00020c(&mut self) {
        self.crashed(0x20C);
    }

    /// `func_000214`: the `rte` after 0x20C's loop.
    pub fn r_000214(&mut self) {
        self.crashed(0x214);
    }

    /// `func_000218`: an illegal instruction.
    pub fn r_000218(&mut self) {
        self.crashed(0x218);
    }

    /// `func_000220`: the `rte` after 0x218's loop.
    pub fn r_000220(&mut self) {
        self.crashed(0x220);
    }

    /// `func_000224`: a division by zero.
    pub fn r_000224(&mut self) {
        self.crashed(0x224);
    }

    /// `func_00022C`: the `rte` after 0x224's loop.
    pub fn r_00022c(&mut self) {
        self.crashed(0x22C);
    }

    /// `func_000230`: privilege, trace, the line A and F codes,
    /// vectors 12-23 and the spurious interrupt.
    pub fn r_000230(&mut self) {
        self.crashed(0x230);
    }

    /// `func_000238`: the `rte` after 0x230's loop.
    pub fn r_000238(&mut self) {
        self.crashed(0x238);
    }

    /// `func_00023C`: `chk`.
    pub fn r_00023c(&mut self) {
        self.crashed(0x23C);
    }

    /// `func_000244`: the `rte` after 0x23C's loop.
    pub fn r_000244(&mut self) {
        self.crashed(0x244);
    }

    /// `func_000248`: `trapv`.
    pub fn r_000248(&mut self) {
        self.crashed(0x248);
    }

    /// `func_000250`: the `rte` after 0x248's loop.
    pub fn r_000250(&mut self) {
        self.crashed(0x250);
    }

    /// `func_000254`: the reserved vectors 48-63.
    pub fn r_000254(&mut self) {
        self.crashed(0x254);
    }

    /// `func_00025C`: the `rte` after 0x254's loop.
    pub fn r_00025c(&mut self) {
        self.crashed(0x25C);
    }

    /// `func_000260`: the interrupt levels the game never lets through
    /// (1-5 and 7; the H-interrupt, level 4, included): a `nop` and a
    /// loop for ever.
    pub fn r_000260(&mut self) {
        panic!("an interrupt the game never lets through (its handler at 0x260)");
    }

    /// `func_000264`: an `rte` no vector points to.
    pub fn r_000264(&mut self) {}

    /// `func_000266`: the traps (vectors 32-47): `rte`.
    pub fn r_000266(&mut self) {}
}
