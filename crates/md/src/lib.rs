//! The Mega Drive as the translated game sees it (docs/PORT.md): the VDP,
//! the pads, the Z80's side; the 68000 is the translated code itself.

pub mod vdp;

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
}

pub struct Md {
    pub vdp: Vdp,
    pub pads: [u16; 4],
    /// The data ports' output bits (TH selects the pad's halves).
    data_out: [u8; 3],
    ctrl: [u8; 3],
    pub z80_ram: Box<[u8; 0x2000]>,
    z80_busreq: bool,
    z80_reset: bool,
    frontend: Box<dyn Frontend>,
    pub frames: u64,
}

impl Md {
    #[must_use]
    pub fn new(frontend: Box<dyn Frontend>) -> Self {
        Md {
            vdp: Vdp::new(),
            pads: [0; 4],
            data_out: [0x40; 3],
            ctrl: [0; 3],
            z80_ram: Box::new([0; 0x2000]),
            z80_busreq: false,
            z80_reset: true,
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
        (v & !self.ctrl[port]) | (self.data_out[port] & self.ctrl[port]) | 0x80 & 0
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
            0xA0_0000..=0xA0_FFFF => self.z80_ram[(addr & 0x1FFF) as usize],
            0xA1_0000..=0xA1_001F => self.io_read(addr),
            // the Z80's bus is granted at once (bit 0 clear)
            0xA1_1100 => u8::from(!self.z80_busreq),
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
            0xA0_0000..=0xA0_FFFF => {
                if addr & 0xF000 < 0x2000 {
                    self.z80_ram[(addr & 0x1FFF) as usize] = value;
                }
            }
            0xA1_0000..=0xA1_001F => self.io_write(addr, value),
            0xA1_1100 => self.z80_busreq = value & 1 != 0,
            0xA1_1200 => self.z80_reset = value & 1 == 0,
            0xC0_0000..=0xC0_000F => self.write16(addr & !1, u16::from(value) << 8 | u16::from(value)),
            _ => {}
        }
    }

    fn write16(&mut self, addr: u32, value: u16) {
        match addr {
            0xC0_0000 | 0xC0_0002 => self.vdp.write_data(value),
            0xC0_0004 | 0xC0_0006 => self.vdp.write_control(value),
            0xC0_0011..=0xC0_0017 => {} // the PSG
            0xA1_1100 => self.z80_busreq = value & 0x100 != 0,
            0xA1_1200 => self.z80_reset = value & 0x100 == 0,
            _ => {
                self.write8(addr, (value >> 8) as u8);
                self.write8(addr + 1, value as u8);
            }
        }
    }

    fn take_dma(&mut self) -> Option<(u32, u32)> {
        match self.vdp.take_dma() {
            Some(Dma::Memory { source, words }) => Some((source, words)),
            None => None,
        }
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
        self.vdp.run_line(line)
    }

    fn vint(&mut self) -> bool {
        self.vdp.line = vdp::HEIGHT as u16;
        self.vdp.vint_pending = true;
        self.vdp.vint_enabled()
    }

    fn end_frame(&mut self, ram: &mut [u8; 0x10000]) {
        self.vdp.vint_pending = false;
        self.vdp.line = 0;
        self.frames += 1;
        let pads = &mut self.pads;
        let width = self.vdp.width();
        self.frontend.frame(&self.vdp.frame, width, ram, pads);
    }

}
