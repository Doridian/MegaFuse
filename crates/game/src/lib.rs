//! Mega Bomberman's code, translated into Rust by tools/recomp
//! (docs/PORT.md).  The routines are methods of [`Game`]; `translated/`
//! is generated from the ROM (`make port`), this file is written by hand.

mod ops;
mod translated;

pub use translated::RESET;

/// What the game reaches on the bus besides work RAM and the ROM: the
/// VDP, the Z80's side (its RAM, the sound chips, the bus handshake), the
/// pads.  `crates/md` implements it.
pub trait Hardware {
    fn read8(&mut self, addr: u32) -> u8;
    fn read16(&mut self, addr: u32) -> u16;
    fn write8(&mut self, addr: u32, value: u8);
    fn write16(&mut self, addr: u32, value: u16);
}

/// The registers a family of routines called through a pointer share
/// (the scheduler's tasks, the objects' routines...: tools/recomp's
/// families.py).  Elsewhere registers are a routine's arguments and
/// results.
#[derive(Clone, Copy, Debug, Default)]
pub struct Regs {
    pub d0: u32,
    pub d1: u32,
    pub d2: u32,
    pub d3: u32,
    pub d4: u32,
    pub d5: u32,
    pub d6: u32,
    pub d7: u32,
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub a4: u32,
    pub a5: u32,
    pub a6: u32,
    pub x: bool,
    pub n: bool,
    pub z: bool,
    pub v: bool,
    pub c: bool,
    /// The routine returned from its caller too (docs/PORT.md).
    pub unwound: bool,
}

pub struct Game {
    /// Work RAM, 0xFF0000-0xFFFFFF (mirrored from 0xE00000).
    pub ram: Box<[u8; 0x10000]>,
    /// The cartridge.
    pub rom: Vec<u8>,
    pub hw: Box<dyn Hardware>,
}

impl Game {
    #[must_use]
    pub fn new(rom: Vec<u8>, hw: Box<dyn Hardware>) -> Self {
        Game { ram: Box::new([0; 0x10000]), rom, hw }
    }

    /// Power on: the reset routine, which runs the game.
    pub fn run(&mut self) {
        self.call_reset();
    }
}
