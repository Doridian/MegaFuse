//! The Z80 and what it reaches: its 8 KB of RAM, the YM2612, the PSG and
//! the window onto the 68000's memory (32 KB at a time, the bank
//! register's).  The game uploads its sound driver into the RAM and
//! leaves the music to it.
//!
//! Time is the master clock from power-on, as the 68000's (docs/PORT.md,
//! "Time"): a Z80 cycle is 15 of its clocks, a YM2612 sample 1008 (7 x
//! 144), a PSG step 240.  The Z80 runs up to the 68000's time where the
//! two meet (a bus request, a reset, the frame's segments), as Genesis
//! Plus GX runs it; within an instruction its accesses happen after its
//! base time (and any wait on the 68000's bus).  The YM2612 makes its
//! samples as far as the time of each access to it (its timers, its busy
//! flag), and to each segment's end.

use crate::psg::Psg;
use crate::ym2612::Ym2612;
use crate::z80::{self, Z80};

const MASTER_PER_Z80: u64 = 15;
const MASTER_PER_SAMPLE: u64 = 1008;
const MASTER_PER_PSG: u64 = 240;
/// A Z80 access to the 68000's bus (the window, the VDP's ports) waits 3
/// cycles, on average (Genesis Plus GX).
const BUS_WAIT: u64 = 3 * MASTER_PER_Z80;
/// A write to the YM2612 keeps it busy 32 of its cycles (6 of its clock,
/// a 7th of the master's; a sample is 24 of them).
const YM_CYCLE: u64 = 42;
const BUSY_CYCLES: u64 = 32;
/// The YM2612's sample rate (NTSC): 53693175 / 1008.
pub const SAMPLE_RATE: u32 = 53_267;
/// The mix, of full scale: an FM channel's 14 bits are a quarter of it,
/// a PSG channel's swing 1/16 (Genesis Plus GX's balance, with its PSG
/// at its default 150%).
const FM_GAIN: f32 = 0.25;
const PSG_GAIN: f32 = 2100.0 / 32768.0;
/// The output is AC-coupled, as the console's is: a one-pole high-pass
/// at some 8 Hz takes off the DC (the YM2612's DAC offsets it,
/// docs/PORT.md "Sound").
const DC_POLE: f32 = 0.999;

/// Everything on the Z80's bus but the Z80.
#[derive(Clone)]
pub struct ZBus {
    pub ram: Box<[u8; 0x2000]>,
    pub ym: Ym2612,
    pub psg: Psg,
    /// Bits 15-23 of the 68000 address the window at 0x8000 shows.
    bank: u32,
    rom: std::sync::Arc<[u8]>,
    /// The time of the access being made.
    now: u64,
    /// The frame's start (the YM2612's busy time is counted from it).
    pub frame_clock: u64,
    /// The 68000's bus is the VDP's until then (a DMA from it).
    pub dma_until: u64,
    /// The next YM2612 sample's time, and when its busy flag drops.
    ym_next: u64,
    ym_busy: u64,
    psg_clock: u64,
    /// Stereo samples, interleaved, since the frontend last took them.
    pub samples: Vec<f32>,
    /// The high-pass's last input and output, left and right.
    dc: [(f32, f32); 2],
}

impl ZBus {
    /// A read of the 68000's memory through the window.  Only the ROM
    /// answers; work RAM and the I/O area read as 0xFF.
    fn window(&self, a: u16) -> u8 {
        let addr = self.bank | u32::from(a & 0x7FFF);
        self.rom.get(addr as usize).copied().unwrap_or(0xFF)
    }

    /// An access to the 68000's bus: it waits for a DMA to end, then
    /// for the bus.
    fn bus_wait(&mut self) {
        if self.now < self.dma_until {
            self.now = self.dma_until;
        }
        self.now += BUS_WAIT;
    }

    /// The YM2612's samples, as far as `t`.
    pub fn fm_update(&mut self, t: u64) {
        while self.ym_next < t {
            self.sample();
            self.ym_next += MASTER_PER_SAMPLE;
        }
    }

    fn sample(&mut self) {
        let (l, r) = self.ym.sample();
        self.psg_clock += MASTER_PER_SAMPLE;
        let steps = self.psg_clock / MASTER_PER_PSG;
        self.psg_clock %= MASTER_PER_PSG;
        let p = self.psg.run(steps as u32) * PSG_GAIN;
        for (k, x) in [l * FM_GAIN + p, r * FM_GAIN + p].into_iter().enumerate() {
            let (x0, y0) = self.dc[k];
            let y = x - x0 + DC_POLE * y0;
            self.dc[k] = (x, y);
            self.samples.push(y);
        }
    }

    /// The frame ends at `f` (the samples made as far as it): samples
    /// made past it (an access just after it) are dropped and their time
    /// taken again, though the chip has run them (its timers on); Genesis
    /// Plus GX's way, which shifts its samples' phase.
    pub fn frame_end(&mut self, f: u64) {
        if self.ym_next > f {
            let k = (self.ym_next - f) / MASTER_PER_SAMPLE;
            self.ym_next -= k * MASTER_PER_SAMPLE;
            let n = self.samples.len().saturating_sub(2 * k as usize);
            self.samples.truncate(n);
        }
    }

    /// The YM2612's status: only its first port has it (a Model 1's
    /// chip); busy for a while after a data write.
    pub fn ym_read(&mut self, port: u16, t: u64) -> u8 {
        if port & 3 != 0 {
            return 0;
        }
        self.fm_update(t);
        let busy = if t < self.ym_busy { 0x80 } else { 0 };
        self.ym.status | busy
    }

    pub fn ym_write(&mut self, port: u16, value: u8, t: u64) {
        if port & 1 != 0 {
            self.fm_update(t);
            // 32 cycles from its next, counted from the frame's start
            let rel = t - self.frame_clock;
            self.ym_busy = self.frame_clock + (rel.div_ceil(YM_CYCLE) + BUSY_CYCLES) * YM_CYCLE;
        }
        self.ym.write((port & 3) as u8, value);
    }

    /// The 68000 writes on the Z80's side at `t` (it has the bus).
    pub fn ym_write_at(&mut self, t: u64) {
        self.now = t;
    }

    /// The YM2612 reset (with the Z80): its samples up to then first.
    fn ym_reset(&mut self, t: u64) {
        self.fm_update(t);
        let model1 = self.ym.model1_dac;
        self.ym = Ym2612::new();
        self.ym.model1_dac = model1;
        self.ym_busy = 0;
    }
}

impl z80::Bus for ZBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.ram[usize::from(addr & 0x1FFF)],
            0x4000..=0x5FFF => self.ym_read(addr, self.now),
            0x7F00..=0x7FFF => {
                self.bus_wait();
                0xFF
            }
            0x8000..=0xFFFF => {
                self.bus_wait();
                self.window(addr)
            }
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x3FFF => self.ram[usize::from(addr & 0x1FFF)] = value,
            0x4000..=0x5FFF => self.ym_write(addr, value, self.now),
            // a bit at a time, into the top of nine
            0x6000..=0x60FF => {
                self.bank = ((self.bank >> 1) | (u32::from(value & 1) << 23)) & 0xFF_8000
            }
            0x7F00..=0x7FFF => {
                self.bus_wait();
                if matches!(addr, 0x7F11 | 0x7F13 | 0x7F15 | 0x7F17) {
                    self.psg.write(value);
                }
            }
            0x8000..=0xFFFF => self.bus_wait(),
            _ => {}
        }
    }
}

/// The Z80's RAM only, for looking at code without touching the bus.
struct Peek<'a>(&'a [u8; 0x2000]);

impl z80::Bus for Peek<'_> {
    fn read(&mut self, addr: u16) -> u8 {
        self.0[usize::from(addr & 0x1FFF)]
    }
    fn write(&mut self, _: u16, _: u8) {}
}

#[derive(Clone)]
pub struct Sound {
    pub z80: Z80,
    pub bus: ZBus,
    /// The 68000 holds the Z80's bus (it is stopped).
    pub busreq: bool,
    /// The reset line is held (the Z80 is stopped).
    pub reset: bool,
    /// The Z80's interrupt line (the VDP holds it from the vertical
    /// interrupt to the end of its line).
    pub irq: bool,
    /// Where the Z80 got to (master clocks; it may run a little past
    /// the time it was run to).
    pub z80_clock: u64,
    /// The Z80's time, instruction by instruction (`--ztimeline`; one
    /// for every copy of the state).
    pub timeline: Option<std::sync::Arc<std::sync::Mutex<game::Timeline>>>,
}

impl Sound {
    #[must_use]
    pub fn new(rom: Vec<u8>) -> Self {
        Sound {
            z80: Z80::new(),
            bus: ZBus {
                ram: Box::new([0; 0x2000]),
                ym: Ym2612::new(),
                psg: Psg::new(),
                bank: 0,
                rom: rom.into(),
                now: 0,
                frame_clock: 0,
                dma_until: 0,
                ym_next: 0,
                ym_busy: 0,
                psg_clock: 0,
                samples: Vec::new(),
                dc: [(0.0, 0.0); 2],
            },
            busreq: false,
            reset: true,
            irq: false,
            z80_clock: 0,
            timeline: None,
        }
    }

    /// Running: out of reset, its bus not taken.
    fn running(&self) -> bool {
        !self.reset && !self.busreq
    }

    /// The 68000 has the Z80's bus (requested, the Z80 out of reset).
    #[must_use]
    pub fn granted(&self) -> bool {
        !self.reset && self.busreq
    }

    /// The 68000 requests the Z80's bus (or lets it go) at `t`: the Z80
    /// runs up to then; let go, it goes on from then.
    pub fn set_busreq(&mut self, on: bool, t: u64) {
        if on && self.running() {
            self.run_until(t);
        }
        if !on && self.granted() {
            self.z80_clock = t.div_ceil(MASTER_PER_Z80) * MASTER_PER_Z80;
        }
        self.busreq = on;
    }

    /// The reset line at `t`: taken, the Z80 runs up to then; let go, it
    /// starts again from 0 and the YM2612 is reset.
    pub fn set_reset(&mut self, held: bool, t: u64) {
        if held && self.running() {
            self.run_until(t);
        }
        if !held && self.reset {
            if !self.busreq {
                self.z80_clock = t.div_ceil(MASTER_PER_Z80) * MASTER_PER_Z80;
            }
            self.z80.reset();
            self.bus.ym_reset(t);
        }
        self.reset = held;
    }

    /// The Z80 up to `t` (a segment's end); the YM2612's samples too.
    pub fn run_until(&mut self, t: u64) {
        if self.running() {
            while self.z80_clock < t {
                if self.irq && self.z80.takes_interrupt() {
                    let n = self.z80.interrupt(&mut self.bus);
                    self.z80_clock += u64::from(n) * MASTER_PER_Z80;
                    if self.z80_clock >= t {
                        break;
                    }
                }
                if let Some(tl) = &self.timeline {
                    tl.lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .record(u32::from(self.z80.pc), self.z80_clock);
                }
                let base = self.z80.base_cycles(&mut Peek(&self.bus.ram));
                let start = self.z80_clock;
                self.bus.now = start + u64::from(base) * MASTER_PER_Z80;
                let waited = self.bus.now;
                let n = self.z80.step(&mut self.bus);
                self.z80_clock = start + u64::from(n) * MASTER_PER_Z80 + (self.bus.now - waited);
            }
        }
        self.bus.fm_update(t);
    }
}
