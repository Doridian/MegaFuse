//! The frame: the main loop waits for the vertical blank.

use crate::Game;

/// Set by the main loop before it waits; the vertical interrupt's
/// handler (`func_007C5A`) clears it.
const WAITING: u32 = 0xFF805A;

impl Game {
    /// `func_007DFA`: waits for the vertical interrupt, the main loop's
    /// one wait.  The 68000 does nothing meanwhile: the hardware runs to
    /// the interrupt and its handler runs.
    pub fn r_007dfa(&mut self) {
        self.write8(WAITING, 1);
        while self.read8(WAITING) != 0 {
            self.idle();
        }
    }
}
