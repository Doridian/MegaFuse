//! Mega Bomberman's code, translated into Rust by tools/recomp
//! (docs/PORT.md).  The routines are methods of [`Game`]; `translated/`
//! is generated (`make port`), this file is written by hand.

mod ops;
// the exact build (feature `exact`) runs the translation of the
// rewritten routines too: the console's time to the cycle
#[cfg(not(feature = "exact"))]
mod rewritten;
mod translated;

pub use translated::RESET;

/// What the game reaches on the bus besides work RAM and the ROM: the
/// VDP, the Z80's side (its RAM, the sound chips, the bus handshake), the
/// pads.  `crates/md` implements it.
///
/// Time is in master clocks (53.69 MHz NTSC, 7 a 68000 cycle) from
/// power-on, frames of 262 lines of 3420 starting with the vertical
/// blank's first line (224), as Genesis Plus GX counts them.
pub trait Hardware {
    /// The time of the access about to be made, and how much the
    /// instruction making it has still to spend (its base time: the
    /// VDP's status read counts it).
    fn set_time(&mut self, clock: u64, rest: u64);
    fn read8(&mut self, addr: u32) -> u8;
    fn read16(&mut self, addr: u32) -> u16;
    fn write8(&mut self, addr: u32, value: u8);
    fn write16(&mut self, addr: u32, value: u16);
    /// A DMA from 68000 memory the last write started: (source, words,
    /// to VRAM).
    fn take_dma(&mut self) -> Option<(u32, u32, bool)>;
    /// A DMA from 68000 memory starts at `start` with these words (the
    /// 68000 waits; the VDP takes them a window at a time): when it ends,
    /// and whether it ran on past the line (or the blank) it started in.
    fn dma_run(&mut self, start: u64, source: u32, data: Vec<u16>, vram: bool) -> (u64, bool);
    /// When the 68000 may go on after the access just made (0: at once).
    fn take_stall(&mut self) -> u64;
    /// The frame starts: line 224, the vertical blank's first.
    fn frame_start(&mut self);
    /// The vertical interrupt is raised (line 224, `vint_clock` in).
    fn vint(&mut self);
    /// Where in line 224 the vertical interrupt comes.
    fn vint_clock(&self) -> u64;
    /// A line starts (all but 224): drawn if it is a visible one.
    fn line_start(&mut self, line: u16);
    /// The interrupt the VDP is raising (6, 4 or none).
    fn irq_level(&self) -> u8;
    /// The 68000 takes the interrupt.
    fn int_ack(&mut self, level: u8);
    /// Is the picture on (a DMA is slow while it is drawn)?
    fn display_on(&self) -> bool;
    /// The Z80 and the sound chips run up to `t` (a segment's end).
    fn sync(&mut self, t: u64);
    /// Does the 68000 have the Z80's bus (its side answers, a cycle
    /// late)?
    fn z80_granted(&self) -> bool;
    /// The frame is over: the picture and RAM out, the pads in.
    fn end_frame(&mut self, ram: &mut [u8; 0x10000]);
    /// The hardware's state (a copy: [`Game::save`]).
    fn save(&self) -> Box<dyn std::any::Any + Send>;
    /// The hardware's state back.
    fn restore(&mut self, state: &dyn std::any::Any);
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
    /// The 68000's side of the state between steps (plain data: a copy
    /// is a snapshot, [`Game::save`]).
    pub state: State,
    /// The cartridge.
    pub rom: Vec<u8>,
    pub hw: Box<dyn Hardware>,
    /// The 68000's time, instruction by instruction, for
    /// tools/timeline_check.py.
    pub timeline: Option<Timeline>,
    /// A soft reset in this step (a jump to the reset routine): the
    /// registers it left for the main loop; the routines between return
    /// at once (they never go on in the original, whose stack the reset
    /// starts again).
    pub(crate) restarted: Option<Regs>,
    /// Tests: the clock past which a routine run on its own is taken to
    /// wait for ever (a wait whose end the snapshot can't bring).
    pub deadline: Option<u64>,
}

/// What the 68000's side keeps: work RAM, the time, the interrupt mask,
/// the registers between steps.
#[derive(Clone)]
pub struct State {
    /// Work RAM, 0xFF0000-0xFFFFFF (mirrored from 0xE00000).
    pub ram: Box<[u8; 0x10000]>,
    /// Master clocks since power-on (docs/PORT.md, "Time"): where the
    /// instruction being run started, but for its base time, `pending`,
    /// spent when the next one starts.
    pub clock: u64,
    pending: u64,
    /// When the next bus refresh is due.
    refresh_due: u64,
    /// The frame's start, the segment of it being run and where that
    /// ends (segments: line 224 to the vertical interrupt, the rest of
    /// it, then a line each).
    frame_base: u64,
    seg: usize,
    seg_end: u64,
    /// The interrupt mask (SR bits 8-10).
    ipl: u8,
    /// An interrupt enabled while pending (a VDP register write) comes
    /// an instruction late: 1 while the next instruction is to start
    /// (it runs as it is, no refresh, no interrupt), 2 while it runs.
    delayed: u8,
    /// A 32-bit write is being made (its interrupt comes at once).
    pub(crate) long_write: bool,
    /// The registers between steps.
    pub regs: Regs,
}

impl std::ops::Deref for Game {
    type Target = State;
    fn deref(&self) -> &State {
        &self.state
    }
}

impl std::ops::DerefMut for Game {
    fn deref_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

/// The whole game between two steps: the 68000's side and the
/// hardware's.
pub struct Snapshot {
    pub state: State,
    pub hw: Box<dyn std::any::Any + Send>,
}

/// Master clocks: a line, a frame.
pub const LINE: u64 = 3420;
pub const LINES: u64 = 262;
pub const FRAME: u64 = LINE * LINES;
/// Segments a frame: line 224 in two, the 261 others.
const SEGMENTS: usize = 263;
/// Power-on: where in the frame the 68000 starts (line 191, Genesis Plus
/// GX's measurement of a Model 1) and the reset's 40 cycles.
const POWER_ON: u64 = (LINES - 192 + 159) * LINE + 1004;
const RESET_TIME: u64 = 40 * 7;
/// A bus refresh: 2 cycles every 128.
const REFRESH_EVERY: u64 = 128 * 7;
const REFRESH_TAKES: u64 = 2 * 7;
/// An interrupt's processing: 44 cycles and a wait for the E clock (a
/// tenth of the 68000's), by where in its period it starts.
const INTERRUPT: [u64; 10] = [50, 59, 58, 57, 56, 55, 54, 53, 52, 51];

/// The 68000's time per frame: how many instructions and an FNV-1a hash
/// of their (PC, clock), as tools/mdtrace --timeline writes it; one frame
/// in full.
pub struct Timeline {
    pub out: Box<dyn std::io::Write + Send>,
    pub full: Option<u64>,
    frame: u64,
    count: u64,
    hash: u64,
}

const FNV: u64 = 0xCBF2_9CE4_8422_2325;

impl Timeline {
    #[must_use]
    pub fn new(out: Box<dyn std::io::Write + Send>, full: Option<u64>) -> Self {
        Timeline {
            out,
            full,
            frame: 0,
            count: 0,
            hash: FNV,
        }
    }

    /// The frame is over: its line, and the file up to date (the
    /// frontend may end the run now).
    pub fn end_frame(&mut self) {
        let _ = writeln!(
            self.out,
            "FRAME {} {} {:016X}",
            self.frame, self.count, self.hash
        );
        let _ = self.out.flush();
        self.frame += 1;
        self.count = 0;
        self.hash = FNV;
    }

    pub fn record(&mut self, pc: u32, clock: u64) {
        let frame = clock / FRAME;
        let v = u64::from(pc) << 40 ^ clock;
        for k in 0..8 {
            self.hash ^= (v >> (8 * k)) & 0xFF;
            self.hash = self.hash.wrapping_mul(0x0000_0100_0000_01B3);
        }
        self.count += 1;
        if self.full == Some(frame) {
            let _ = writeln!(self.out, "{pc:06X} {clock}");
        }
    }
}

use std::io::Write as _;

impl Game {
    #[must_use]
    pub fn new(rom: Vec<u8>, hw: Box<dyn Hardware>) -> Self {
        let clock = POWER_ON / 7 * 7 + RESET_TIME;
        Game {
            state: State {
                ram: Box::new([0; 0x10000]),
                clock,
                pending: 0,
                refresh_due: clock / REFRESH_EVERY * REFRESH_EVERY + REFRESH_EVERY,
                frame_base: 0,
                seg: SEGMENTS,
                seg_end: 0,
                ipl: 7,
                delayed: 0,
                long_write: false,
                regs: Regs::default(),
            },
            rom,
            hw,
            timeline: None,
            restarted: None,
            deadline: None,
        }
    }

    /// Power on and play: [`Game::boot`], then [`Game::step`] for ever;
    /// the frames pass as it runs ([`Game::insn`]).
    pub fn run(&mut self) -> ! {
        self.boot();
        loop {
            self.step();
        }
    }

    /// Power on: the reset routine, to the main loop.
    pub fn boot(&mut self) {
        let mut regs = Regs::default();
        self.call_reset(&mut regs);
        self.regs = regs;
    }

    /// The game as it is between two steps.
    #[must_use]
    pub fn save(&self) -> Snapshot {
        Snapshot {
            state: self.state.clone(),
            hw: self.hw.save(),
        }
    }

    /// The game back as it was.
    pub fn restore(&mut self, s: &Snapshot) {
        self.state = s.state.clone();
        self.hw.restore(&*s.hw);
    }

    /// A pass of the main loop (docs/PORT.md, "A step"): from its wait
    /// for the vertical blank to the next, a frame (more when the game
    /// lags).  Between steps the game's whole state is plain data: this
    /// struct and the hardware's.
    pub fn step(&mut self) {
        let mut regs = self.regs;
        self.call_step(&mut regs);
        // a soft reset: the next pass starts where it left the registers
        self.regs = self.restarted.take().unwrap_or(regs);
    }

    /// An instruction starts (at `pc`; `base` its time but what depends
    /// on the data): the last one's time is spent, the beam moves on,
    /// interrupts may come, the bus may be refreshed.
    #[inline]
    pub fn insn(&mut self, pc: u32, base: u32) {
        if self.delayed != 0 {
            self.delayed_insn(pc, base);
            return;
        }
        loop {
            self.clock += self.pending;
            self.pending = 0;
            if self.clock < self.seg_end || !self.boundary() {
                break;
            }
        }
        if let Some(t) = &mut self.timeline {
            t.record(pc, self.state.clock);
        }
        if self.clock >= self.refresh_due {
            self.refresh_due = self.clock + REFRESH_EVERY;
            self.clock += REFRESH_TAKES;
        }
        self.pending = u64::from(base) * 7;
    }

    /// The instruction after one that enabled a pending interrupt runs
    /// first, as it is; the interrupt comes after it (before its base
    /// time is spent).
    #[cold]
    fn delayed_insn(&mut self, pc: u32, base: u32) {
        if self.delayed == 1 {
            self.clock += self.pending;
            self.pending = u64::from(base) * 7;
            if let Some(t) = &mut self.timeline {
                t.record(pc, self.state.clock);
            }
            self.delayed = 2;
            return;
        }
        self.delayed = 0;
        self.check_interrupts();
        self.insn(pc, base);
    }

    /// A write to the VDP's control port that raised its interrupt
    /// line (an interrupt enabled while pending): it comes an
    /// instruction late, or at once in a 32-bit write.
    pub(crate) fn irq_raised(&mut self) {
        if self.long_write {
            self.check_interrupts();
        } else {
            self.delayed = 1;
        }
    }

    /// What an instruction's data makes it take beyond its base time
    /// (cycles).
    #[inline]
    pub fn extra(&mut self, cycles: u32) {
        self.clock += u64::from(cycles) * 7;
    }

    /// A long instruction (a shift by a register, MOVEM.L, a division)
    /// skips the bus refresh that came due during it.
    #[inline]
    pub fn skip_refresh(&mut self) {
        if self.clock >= self.refresh_due {
            self.refresh_due += REFRESH_EVERY;
        }
    }

    /// The segments that ended: their events; true when an interrupt
    /// was taken at the start of the last (the 68000 is let run again).
    fn boundary(&mut self) -> bool {
        while self.clock >= self.seg_end {
            let start = self.seg_end;
            // the Z80 runs the segment after the 68000, to its end
            self.hw.sync(start);
            self.seg += 1;
            if self.seg >= SEGMENTS {
                if self.frame_base > 0 || self.seg_end > 0 {
                    self.frame_base += FRAME;
                    if let Some(t) = &mut self.timeline {
                        t.end_frame();
                    }
                    self.hw.end_frame(&mut self.state.ram);
                }
                self.seg = 0;
            }
            self.seg_end = self.frame_base
                + match self.seg {
                    0 => self.hw.vint_clock(),
                    s => (s as u64) * LINE,
                };
            self.hw.set_time(start, 0);
            match self.seg {
                0 => self.hw.frame_start(),
                1 => self.hw.vint(),
                s => self.hw.line_start(((224 + s - 1) % 262) as u16),
            }
        }
        self.check_interrupts()
    }

    fn check_interrupts(&mut self) -> bool {
        let level = self.hw.irq_level();
        if level > self.ipl {
            self.interrupt(level);
            return true;
        }
        false
    }

    /// An interrupt: its processing, the instruction it came in (if it
    /// came in one) finished, the handler; returning from it lets in
    /// what waited.
    fn interrupt(&mut self, level: u8) {
        // the E clock's phase: Genesis Plus GX counts it from the frame's
        // start (a frame isn't a whole number of its periods)
        let rel = self.clock - self.frame_base;
        self.clock += INTERRUPT[((rel / 7) % 10) as usize] * 7;
        self.clock += self.pending;
        self.pending = 0;
        let saved = self.ipl;
        self.ipl = level;
        self.hw.int_ack(level);
        self.call_interrupt(level);
        self.set_ipl(saved);
    }

    /// The 68000 waits (a rewritten wait, docs/PORT.md "Rewriting"):
    /// time passes to the next of the frame's events (a line, the
    /// vertical interrupt), which come as they would.
    pub fn idle(&mut self) {
        self.clock += self.pending;
        self.pending = 0;
        self.clock = self.clock.max(self.seg_end);
        self.boundary();
        self.check_deadline();
    }

    /// The 68000 polls the hardware (a rewritten wait for the Z80's bus,
    /// a DMA, ...): `cycles` pass, and the frame's events in them.
    pub fn pass(&mut self, cycles: u32) {
        self.clock += self.pending + u64::from(cycles) * 7;
        self.pending = 0;
        while self.clock >= self.seg_end && self.boundary() {}
        self.check_deadline();
    }

    fn check_deadline(&self) {
        if let Some(d) = self.deadline {
            assert!(self.clock < d, "still waiting at the deadline");
        }
    }

    /// Tests: the interrupt mask as it is, for a routine run on its own.
    pub fn mask_interrupts(&mut self) {
        self.ipl = 7;
    }

    /// The interrupt mask (SR bits 8-10).
    #[must_use]
    pub fn ipl(&self) -> u8 {
        self.ipl
    }

    /// The interrupt mask written: what waited for it comes now.
    pub fn set_ipl(&mut self, level: u8) {
        self.ipl = level & 7;
        self.check_interrupts();
    }

    /// The time of a hardware access, for the hardware.
    #[inline]
    pub(crate) fn sync(&mut self) {
        self.hw.set_time(self.clock, self.pending);
    }

    /// A DMA from memory the write just made started.  The 68000 waits
    /// while it runs: so many words a line (fewer while the picture is
    /// drawn), to the line's end, or the vertical blank's (line 261),
    /// then on from each line's start.
    pub(crate) fn dma(&mut self) {
        while let Some((source, words, vram)) = self.hw.take_dma() {
            // the 68000 is held meanwhile: its memory can be read now
            let data = (0..words)
                .map(|k| self.read16((source & 0xFF_0000) | (source.wrapping_add(2 * k) & 0xFFFF)))
                .collect();
            let (end, on) = self.hw.dma_run(self.clock, source, data, vram);
            self.clock = end;
            if on {
                // Genesis Plus GX sets the 68000's clock to the DMA's end
                // again at each line it runs on: the instruction's base
                // time, spent after the first, is lost
                self.pending = 0;
            }
        }
    }
}
