//! A pseudo-random generator: three bytes of state, `MADE` (which the
//! task list counts with too, `task.rs`), `STATE_A`, `STATE_B`, mixed
//! by shifts through X and exclusive ors (`func_001BE2`).

use crate::Game;

/// The state: a byte the task list counts the tasks made in, then
/// two bytes (a word, `STATE_A` high).
const MADE: u32 = 0xFFA3FE;
const STATE_A: u32 = 0xFFA400;
const STATE_B: u32 = 0xFFA401;
/// Where `func_001C3A` keeps a copy of it (`MADE`, then the word).
const SAVED_MADE: u32 = 0xFFA402;
const SAVED_STATE: u32 = 0xFFA404;
/// The byte `func_001C16` seeds it from.
const SEED: u32 = 0xFF80D4;
/// What `func_001C48` sets it to.
const START_MADE: u8 = 0x25;
const START_STATE: u16 = 0x892F;
/// How many numbers `func_001C16` draws after seeding.
const WARM_UP: u32 = 24;

impl Game {
    /// `func_001BC0`: a number from 0 to `d0`'s low byte (`func_001BE2`,
    /// less that plus 1 until it is no more); d1 is kept.  Gives back d0.
    pub fn r_001bc0(&mut self, d0: u32, d1: u32) -> u32 {
        let limit = d0 as u8;
        let (drawn, ..) = self.r_001be2((d1 & 0xFFFF_FF00) | u32::from(limit));
        let mut number = drawn as u8;
        while number > limit {
            number = number.wrapping_sub(limit).wrapping_sub(1);
        }
        (drawn & 0xFFFF_FF00) | u32::from(number)
    }

    /// `func_001BE2`: the next number (a byte): the state stepped, then
    /// `STATE_B` ^ `STATE_A`.  d1 is kept.  Gives back d0 (the number),
    /// d1 and X (the bit shifted out of `STATE_B`).
    pub fn r_001be2(&mut self, d1: u32) -> (u32, u32, bool) {
        let a = self.read8(STATE_A);
        let b = self.read8(STATE_B);
        // bit 0 of roxl.b #3 is bit 6 of b, whatever X was
        let bit = ((b >> 6) ^ a) & 1 != 0;
        let b = b ^ a;
        self.write8(STATE_B, b);
        let mixed = a ^ self.read8(MADE);
        let a = mixed << 1 | u8::from(bit);
        self.write8(STATE_A, a);
        let carry = mixed & 0x80 != 0;
        let x = b & 0x80 != 0;
        let b = b << 1 | u8::from(carry);
        self.write8(STATE_B, b);
        (u32::from(b ^ a), d1, x)
    }

    /// `func_001C16`: the state seeded from `SEED` (all three bytes),
    /// then `WARM_UP` numbers drawn.
    pub fn r_001c16(&mut self) {
        let seed = self.read8(SEED);
        for at in [MADE, STATE_A, STATE_B] {
            self.write8(at, seed);
        }
        for _ in 0..WARM_UP {
            self.r_001be2(0);
        }
    }

    /// `func_001C3A`: the state copied to `SAVED_MADE`, `SAVED_STATE`.
    /// Gives back C (clear).
    pub fn r_001c3a(&mut self) -> bool {
        let made = self.read8(MADE);
        self.write8(SAVED_MADE, made);
        let state = self.read16(STATE_A);
        self.write16(SAVED_STATE, state);
        false
    }

    /// `func_001C48`: the state set to `START_MADE`, `START_STATE`.
    pub fn r_001c48(&mut self) {
        self.write8(MADE, START_MADE);
        self.write16(STATE_A, START_STATE);
    }
}
