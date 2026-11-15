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
    /// A DMA from 68000 memory the last write started: (source, words).
    fn take_dma(&mut self) -> Option<(u32, u32)>;
    fn dma_word(&mut self, w: u16);
    fn dma_done(&mut self, source: u32, words: u32);
    /// Draws a line; true when the horizontal interrupt fires after it.
    fn run_line(&mut self, line: u16) -> bool;
    /// The vertical blank starts; true when its interrupt is enabled.
    fn vint(&mut self) -> bool;
    /// Is the picture on (a DMA is slow while it is drawn)?
    fn display_on(&self) -> bool;
    /// The frame is over: the picture and RAM out, the pads in.
    fn end_frame(&mut self, ram: &mut [u8; 0x10000]);
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
    /// 68000 cycles since power-on, as the translated code counts them
    /// (`tick`, docs/PORT.md "Time").
    pub cycles: u64,
    /// When the line being drawn ends.
    next_line: u64,
    /// The line being drawn (0-261).
    line: u16,
    /// The interrupt mask (SR bits 8-10).
    ipl: u8,
    /// Interrupts that came while masked: bit 4 horizontal, bit 6
    /// vertical.
    pending: u8,
}

/// 68000 cycles a line takes (NTSC: 3420 master clocks over 7).
pub const CYCLES_PER_LINE: u64 = 488;
pub const LINES: u16 = 262;
/// The first line of the vertical blank (the vertical interrupt's).
pub const VBLANK_LINE: u16 = 224;

impl Game {
    #[must_use]
    pub fn new(rom: Vec<u8>, hw: Box<dyn Hardware>) -> Self {
        Game {
            ram: Box::new([0; 0x10000]),
            rom,
            hw,
            cycles: 0,
            next_line: CYCLES_PER_LINE,
            line: 0,
            ipl: 7,
            pending: 0,
        }
    }

    /// Power on: the reset routine, which runs the game; it never
    /// returns, the frames pass as it runs ([`Game::tick`]).
    pub fn run(&mut self) {
        self.call_reset();
    }

    /// Time passes: n cycles of the 68000's.  The lines they cover are
    /// drawn, and the interrupts come as the VDP raises them (between
    /// two blocks of the game's code, as between two instructions on
    /// the console) unless the mask holds them back.
    pub fn tick(&mut self, n: u32) {
        self.cycles += u64::from(n);
        while self.cycles >= self.next_line {
            self.next_line += CYCLES_PER_LINE;
            self.line_done();
        }
    }

    fn line_done(&mut self) {
        let line = self.line;
        if self.hw.run_line(line) {
            self.interrupt(4);
        }
        self.line += 1;
        if self.line == VBLANK_LINE && self.hw.vint() {
            self.interrupt(6);
        }
        if self.line == LINES {
            self.line = 0;
            self.hw.end_frame(&mut self.ram);
        }
    }

    fn interrupt(&mut self, level: u8) {
        if level <= self.ipl {
            self.pending |= 1 << level;
            return;
        }
        self.pending &= !(1 << level);
        let saved = self.ipl;
        self.ipl = level;
        self.call_interrupt(level);
        self.ipl = saved;
    }

    /// The interrupt mask (SR bits 8-10).
    #[must_use]
    pub fn ipl(&self) -> u8 {
        self.ipl
    }

    /// The interrupt mask written: what waited for it comes now.
    pub fn set_ipl(&mut self, level: u8) {
        self.ipl = level & 7;
        for l in [6, 4] {
            if self.pending & (1 << l) != 0 && l > self.ipl {
                self.interrupt(l);
            }
        }
    }

    /// A DMA from memory the write just made started.  The 68000 waits
    /// while it runs: some 18 bytes go a line while the picture is drawn,
    /// 205 in the vertical blank or with the display off (H40).
    pub(crate) fn dma(&mut self) {
        while let Some((source, words)) = self.hw.take_dma() {
            for k in 0..words {
                let a = (source & 0xFF_0000) | (source.wrapping_add(2 * k) & 0xFFFF);
                let w = self.read16(a);
                self.hw.dma_word(w);
            }
            self.hw.dma_done(source, words);
            let fast = self.line >= VBLANK_LINE || !self.hw.display_on();
            let per_line = if fast { 205 } else { 18 };
            let cycles = u64::from(words) * 2 * CYCLES_PER_LINE / per_line;
            self.tick(u32::try_from(cycles).unwrap_or(u32::MAX));
        }
    }
}
