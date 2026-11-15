//! The Z80 and what it reaches: its 8 KB of RAM, the YM2612, the PSG and
//! the window onto the 68000's memory (32 KB at a time, the bank
//! register's).  The game uploads its sound driver into the RAM and
//! leaves the music to it.
//!
//! Time goes by the master clock (53.69 MHz NTSC): a line is 3420 of its
//! clocks, a Z80 cycle 15, a YM2612 sample 1008 (7 x 144), a PSG step
//! 240 (15 x 16).  The Z80 and the YM2612 take turns within a line, so
//! the samples a driver plays through the DAC come out at its pace.

use crate::psg::Psg;
use crate::ym2612::Ym2612;
use crate::z80::{self, Z80};

pub const MASTER_PER_LINE: u32 = 3420;
const MASTER_PER_Z80: u32 = 15;
const MASTER_PER_SAMPLE: u32 = 1008;
const MASTER_PER_PSG: u32 = 240;
/// The YM2612's sample rate (NTSC): 53693175 / 1008.
pub const SAMPLE_RATE: u32 = 53_267;
/// The mix, of full scale: an FM channel's 14 bits are a quarter of it,
/// a PSG channel's swing 1/16 (Genesis Plus GX's balance, with its PSG
/// at its default 150%).
const FM_GAIN: f32 = 0.25;
const PSG_GAIN: f32 = 2100.0 / 32768.0;

/// Everything on the Z80's bus but the Z80.
pub struct ZBus {
    pub ram: Box<[u8; 0x2000]>,
    pub ym: Ym2612,
    pub psg: Psg,
    /// Bits 15-23 of the 68000 address the window at 0x8000 shows.
    bank: u32,
    rom: Vec<u8>,
}

impl ZBus {
    /// A read of the 68000's memory through the window.  Only the ROM
    /// answers; work RAM and the I/O area read as 0xFF.
    fn window(&self, a: u16) -> u8 {
        let addr = self.bank | u32::from(a & 0x7FFF);
        self.rom.get(addr as usize).copied().unwrap_or(0xFF)
    }
}

impl z80::Bus for ZBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.ram[usize::from(addr & 0x1FFF)],
            0x4000..=0x5FFF => self.ym.status,
            0x8000..=0xFFFF => self.window(addr),
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x3FFF => self.ram[usize::from(addr & 0x1FFF)] = value,
            0x4000..=0x5FFF => self.ym.write((addr & 3) as u8, value),
            // a bit at a time, into the top of nine
            0x6000..=0x60FF => self.bank = ((self.bank >> 1) | (u32::from(value & 1) << 23)) & 0xFF_8000,
            0x7F11 | 0x7F13 | 0x7F15 | 0x7F17 => self.psg.write(value),
            _ => {}
        }
    }
}

pub struct Sound {
    pub z80: Z80,
    pub bus: ZBus,
    /// The 68000 holds the Z80's bus (it is stopped).
    pub busreq: bool,
    /// The reset line is held (the Z80 and the YM2612 are stopped).
    pub reset: bool,
    /// The Z80's interrupt line (the VDP raises it for a line from the
    /// vertical blank's start).
    pub irq: bool,
    /// Master clocks into the line: where the Z80 got to (it may run a
    /// little past the line's end) and where the next sample is due.
    z80_clock: u32,
    sample_clock: u32,
    psg_clock: u32,
    /// Stereo samples, interleaved, since the frontend last took them.
    pub samples: Vec<f32>,
}

impl Sound {
    #[must_use]
    pub fn new(rom: Vec<u8>) -> Self {
        Sound {
            z80: Z80::new(),
            bus: ZBus { ram: Box::new([0; 0x2000]), ym: Ym2612::new(), psg: Psg::new(), bank: 0, rom },
            busreq: false,
            reset: true,
            irq: false,
            z80_clock: 0,
            sample_clock: 0,
            psg_clock: 0,
            samples: Vec::new(),
        }
    }

    /// The reset line: the Z80 starts again from 0 when it is let go;
    /// the YM2612's registers are cleared when it is taken.
    pub fn set_reset(&mut self, held: bool) {
        if held && !self.reset {
            self.bus.ym = Ym2612::new();
        }
        if !held && self.reset {
            self.z80.reset();
        }
        self.reset = held;
    }

    /// A line's worth of time.
    pub fn run_line(&mut self) {
        while self.sample_clock < MASTER_PER_LINE {
            self.run_z80(self.sample_clock);
            self.sample();
            self.sample_clock += MASTER_PER_SAMPLE;
        }
        self.run_z80(MASTER_PER_LINE);
        self.z80_clock -= MASTER_PER_LINE;
        self.sample_clock -= MASTER_PER_LINE;
        self.irq = false;
    }

    /// The Z80 up to `until` master clocks into the line; the time
    /// passes all the same while it is stopped.
    fn run_z80(&mut self, until: u32) {
        while self.z80_clock < until {
            if self.reset || self.busreq {
                self.z80_clock = until;
                return;
            }
            if self.irq {
                let n = self.z80.interrupt(&mut self.bus);
                if n > 0 {
                    self.irq = false;
                    self.z80_clock += n * MASTER_PER_Z80;
                    continue;
                }
            }
            self.z80_clock += self.z80.step(&mut self.bus) * MASTER_PER_Z80;
        }
    }

    fn sample(&mut self) {
        // held in reset the YM2612 is silent, but for its DAC's offset
        let (l, r) = self.bus.ym.sample();
        self.psg_clock += MASTER_PER_SAMPLE;
        let steps = self.psg_clock / MASTER_PER_PSG;
        self.psg_clock %= MASTER_PER_PSG;
        let p = self.bus.psg.run(steps) * PSG_GAIN;
        self.samples.push(l * FM_GAIN + p);
        self.samples.push(r * FM_GAIN + p);
    }
}
