//! The Mega Drive as the translated game sees it (docs/PORT.md): the VDP,
//! the pads, the Z80 and the sound chips; the 68000 is the translated
//! code itself.

pub mod psg;
pub mod sound;
pub mod vdp;
pub mod ym2612;
pub mod z80;

use sound::Sound;
use vdp::{Dma, Vdp};

/// A pad's buttons (set = pressed).
pub mod button {
    pub const UP: u16 = 1 << 0;
    pub const DOWN: u16 = 1 << 1;
    pub const LEFT: u16 = 1 << 2;
    pub const RIGHT: u16 = 1 << 3;
    pub const B: u16 = 1 << 4;
    pub const C: u16 = 1 << 5;
    pub const A: u16 = 1 << 6;
    pub const START: u16 = 1 << 7;
}

/// Called at the end of each frame with the picture and work RAM; it
/// sets the pads for the next one.
pub trait Frontend {
    /// picture: 320 pixels a line, of which the first `width` are the
    /// picture (H40 320, H32 256).
    fn frame(&mut self, picture: &[u32], width: usize, ram: &mut [u8; 0x10000], pads: &mut [u16; 4]);

    /// The frame's sound, before [`Frontend::frame`]: stereo samples,
    /// interleaved, at [`sound::SAMPLE_RATE`].
    fn audio(&mut self, _samples: &[f32]) {}
}

pub struct Md {
    pub vdp: Vdp,
    pub pads: [u16; 4],
    /// The data ports' output bits (TH selects the pad's halves).
    data_out: [u8; 3],
    ctrl: [u8; 3],
    pub sound: Sound,
    frontend: Box<dyn Frontend>,
    pub frames: u64,
}

impl Md {
    #[must_use]
    /// The ROM is for the Z80, which reads it through its window.
    pub fn new(rom: Vec<u8>, frontend: Box<dyn Frontend>) -> Self {
        Md {
            vdp: Vdp::new(),
            pads: [0; 4],
            data_out: [0x40; 3],
            ctrl: [0; 3],
            sound: Sound::new(rom),
            frontend,
            frames: 0,
        }
    }

    /// A 3-button pad's port: TH high gives C B Right Left Down Up, low
    /// Start A 0 0 Down Up (active low).
    fn pad_read(&self, port: usize) -> u8 {
        let p = !self.pads.get(port).copied().unwrap_or(0);
        let th = self.data_out[port] & 0x40 != 0 || self.ctrl[port] & 0x40 == 0;
        let v = if th {
            0x40 | (p & 0x3F) as u8
        } else {
            (((p >> 2) & 0x30) as u8) | (p & 0x03) as u8
        };
        // the output bits read back as written
        (v & !self.ctrl[port]) | (self.data_out[port] & self.ctrl[port])
    }

    fn io_read(&self, a: u32) -> u8 {
        let v = self.io_read_inner(a);
        if std::env::var("MEGAFUSE_IO").ok().and_then(|v| v.parse::<u64>().ok()).is_some_and(|n| self.frames < n) {
            eprintln!("io read {a:06X} = {v:02X}");
        }
        v
    }

    fn io_read_inner(&self, a: u32) -> u8 {
        match a & 0x1F {
            0x01 => 0xA0, // overseas, NTSC, no expansion unit
            0x03 => self.pad_read(0),
            0x05 => self.pad_read(1),
            0x07 => self.data_out[2],
            0x09 => self.ctrl[0],
            0x0B => self.ctrl[1],
            0x0D => self.ctrl[2],
            _ => 0,
        }
    }

    fn io_write(&mut self, a: u32, v: u8) {
        if std::env::var("MEGAFUSE_IO").ok().and_then(|v| v.parse::<u64>().ok()).is_some_and(|n| self.frames < n) {
            eprintln!("io write {a:06X} = {v:02X}");
        }
        match a & 0x1F {
            0x03 => self.data_out[0] = v,
            0x05 => self.data_out[1] = v,
            0x07 => self.data_out[2] = v,
            0x09 => self.ctrl[0] = v,
            0x0B => self.ctrl[1] = v,
            0x0D => self.ctrl[2] = v,
            _ => {}
        }
    }
}

impl game::Hardware for Md {
    fn read8(&mut self, addr: u32) -> u8 {
        match addr {
            0xA0_0000..=0xA0_3FFF => self.sound.bus.ram[(addr & 0x1FFF) as usize],
            0xA0_4000..=0xA0_5FFF => self.sound.bus.ym.status,
            0xA1_0000..=0xA1_001F => self.io_read(addr),
            // the Z80's bus is granted at once (bit 0 clear)
            0xA1_1100 => u8::from(!self.sound.busreq),
            0xA1_1101 => 0,
            0xC0_0000..=0xC0_001F => {
                let w = self.read16(addr & !1);
                if addr & 1 == 0 { (w >> 8) as u8 } else { w as u8 }
            }
            _ => 0xFF,
        }
    }

    fn read16(&mut self, addr: u32) -> u16 {
        match addr {
            0xC0_0000 | 0xC0_0002 => self.vdp.read_data(),
            0xC0_0004 | 0xC0_0006 => self.vdp.read_status(),
            0xC0_0008..=0xC0_000F => self.vdp.read_hv(),
            _ => u16::from(self.read8(addr)) << 8 | u16::from(self.read8(addr + 1)),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        match addr {
            0xA0_0000..=0xA0_FFFF => z80::Bus::write(&mut self.sound.bus, (addr & 0x7FFF) as u16, value),
            0xA1_0000..=0xA1_001F => self.io_write(addr, value),
            0xA1_1100 => self.sound.busreq = value & 1 != 0,
            0xA1_1200 => self.sound.set_reset(value & 1 == 0),
            0xC0_0000..=0xC0_000F => self.write16(addr & !1, u16::from(value) << 8 | u16::from(value)),
            0xC0_0011 | 0xC0_0013 | 0xC0_0015 | 0xC0_0017 => self.sound.bus.psg.write(value),
            _ => {}
        }
    }

    fn write16(&mut self, addr: u32, value: u16) {
        match addr {
            0xC0_0000 | 0xC0_0002 => self.vdp.write_data(value),
            0xC0_0004 | 0xC0_0006 => self.vdp.write_control(value),
            0xC0_0010..=0xC0_0017 => self.sound.bus.psg.write(value as u8),
            0xA1_1100 => self.sound.busreq = value & 0x100 != 0,
            0xA1_1200 => self.sound.set_reset(value & 0x100 == 0),
            _ => {
                self.write8(addr, (value >> 8) as u8);
                self.write8(addr + 1, value as u8);
            }
        }
    }

    fn take_dma(&mut self) -> Option<(u32, u32)> {
        self.vdp.take_dma().map(|Dma::Memory { source, words }| (source, words))
    }

    fn dma_word(&mut self, w: u16) {
        self.vdp.dma_word(w);
    }

    fn dma_done(&mut self, source: u32, words: u32) {
        self.vdp.dma_done(source, words);
    }

    fn display_on(&self) -> bool {
        self.vdp.display_on()
    }

    fn run_line(&mut self, line: u16) -> bool {
        self.sound.run_line();
        self.vdp.run_line(line)
    }

    fn vint(&mut self) -> bool {
        self.vdp.line = vdp::HEIGHT as u16;
        self.vdp.vint_pending = true;
        self.sound.irq = true;
        self.vdp.vint_enabled()
    }

    fn end_frame(&mut self, ram: &mut [u8; 0x10000]) {
        if std::env::var("MEGAFUSE_VDP").ok().and_then(|v| v.parse::<u64>().ok()) == Some(self.frames) {
            eprintln!("regs {:02X?}", self.vdp.regs);
            eprintln!("cram {:03X?}", &self.vdp.cram[..32]);
        }
        self.vdp.vint_pending = false;
        self.vdp.line = 0;
        self.frames += 1;
        self.frontend.audio(&self.sound.samples);
        self.sound.samples.clear();
        let pads = &mut self.pads;
        let width = self.vdp.width();
        self.frontend.frame(&self.vdp.frame, width, ram, pads);
    }

}
