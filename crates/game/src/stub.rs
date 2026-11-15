//! Stands in for `translated/` when it hasn't been generated (`make port`
//! needs the ROM; CI has none): the crate builds, and checks that don't
//! run the game pass.  Running it panics.

use crate::Regs;

/// The reset routine's address (the translated game's).
pub const RESET: u32 = 0;

const NOT_TRANSLATED: &str = "the game isn't translated: make port (it needs the ROM)";

impl crate::Game {
    pub(crate) fn call_reset(&mut self, _: &mut Regs) {
        panic!("{NOT_TRANSLATED}");
    }

    pub(crate) fn call_step(&mut self, _: &mut Regs) {
        panic!("{NOT_TRANSLATED}");
    }

    pub(crate) fn call_interrupt(&mut self, _: u8) {
        panic!("{NOT_TRANSLATED}");
    }

    /// No routine is there.
    pub fn call_by_address(&mut self, _: u32, _: &mut Regs) -> bool {
        false
    }

    #[must_use]
    pub fn routine_outputs(_: u32) -> &'static [&'static str] {
        &[]
    }
}
