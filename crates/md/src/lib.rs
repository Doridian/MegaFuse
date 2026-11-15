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
    /// The time of the access being made (master clocks) and the rest of
    /// the instruction making it.
    clock: u64,
    rest: u64,
    /// The line being run started, and the frame (master clocks).
    line_clock: u64,
    frame_clock: u64,
    /// The VDP's FIFO: when each of its 4 entries is (or was) written
    /// out, and the next to fill.
    fifo: [u64; 4],
    fifo_idx: usize,
    /// A fill runs until then (the status's DMA busy flag).
    busy_until: u64,
    /// The vertical interrupt was raised this frame.
    vint_raised: bool,
    /// The 68000 has to wait until then (a write to a full FIFO).
    stall: u64,
}

/// Master clocks: a line, a frame; the vertical blank's lines from the
/// frame's start (224) to 261.
const LINE: u64 = 3420;
const FRAME: u64 = LINE * 262;
const VBLANK: u64 = 37 * LINE;
/// When the VDP takes a FIFO entry, from the line's start, into the next
/// line (H40, H32; Genesis Plus GX's tables of its access slots).
const FIFO_H40: [u64; 30] = [
    352, 820, 948, 1076, 1332, 1460, 1588, 1844, 1972, 2100, 2356, 2484, 2612, 2868, 2996, 3124,
    3364, 3380, LINE + 352, LINE + 820, LINE + 948, LINE + 1076, LINE + 1332, LINE + 1460,
    LINE + 1588, LINE + 1844, LINE + 1972, LINE + 2100, LINE + 2356, LINE + 2484,
];
const FIFO_H32: [u64; 28] = [
    230, 510, 810, 970, 1130, 1450, 1610, 1770, 2090, 2250, 2410, 2730, 2890, 3050, 3350, 3370,
    LINE + 230, LINE + 510, LINE + 810, LINE + 970, LINE + 1130, LINE + 1450, LINE + 1610,
    LINE + 1770, LINE + 2090, LINE + 2250, LINE + 2410, LINE + 2730,
];

/// What a DMA moves.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DmaKind {
    /// From 68000 memory to VRAM (two accesses a word).
    Vram,
    /// From 68000 memory to CRAM or VSRAM.
    Other,
    Fill,
    /// VRAM to VRAM (a read and a write a byte).
    Copy,
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
            clock: 0,
            rest: 0,
            line_clock: 0,
            frame_clock: 0,
            fifo: [0; 4],
            fifo_idx: 0,
            busy_until: 0,
            vint_raised: false,
            stall: 0,
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

impl Md {
    /// The status register, as of the end of the instruction reading
    /// it: DMA busy, the FIFO empty or full, the blanks, the vertical
    /// interrupt (its flag up from its time, even before it is raised).
    fn status(&mut self) -> u16 {
        let t = self.clock + self.rest;
        let mut s = self.vdp.read_status();
        if t < self.busy_until {
            s |= 0x0002;
        }
        if t >= self.fifo[(self.fifo_idx + 3) & 3] {
            s |= 0x0200;
        } else if t < self.fifo[self.fifo_idx] {
            s |= 0x0100;
        }
        let x = t.saturating_sub(self.line_clock);
        let h40 = self.vdp.regs[12] & 1 != 0;
        if self.vdp.line == vdp::HEIGHT as u16 && !self.vint_raised && x >= game::Hardware::vint_clock(self) {
            s |= 0x0080;
        }
        let (from, to) = if h40 { (228, 872) } else { (280, 860) };
        if (from..to).contains(&x) {
            s |= 0x0004;
        }
        s
    }

    /// A write to the data port.  While the picture is drawn it goes
    /// through the FIFO, taken out at the VDP's access slots: with four
    /// waiting the 68000 waits for the oldest.  A fill it starts runs on
    /// (the DMA busy flag).
    fn data_write(&mut self, w: u16) {
        if !self.vdp.in_vblank() && self.vdp.display_on() {
            let mut t = self.clock;
            let last = self.fifo[(self.fifo_idx + 3) & 3];
            if t < last {
                if t < self.fifo[self.fifo_idx] {
                    // whole 68000 cycles from the frame's start
                    let f = self.frame_clock;
                    self.stall = f + (self.fifo[self.fifo_idx] - f).div_ceil(7) * 7;
                }
                t = last;
            }
            let x = t - self.line_clock;
            let h40 = self.vdp.regs[12] & 1 != 0;
            let slots: &[u64] = if h40 { &FIFO_H40 } else { &FIFO_H32 };
            let at = slots.iter().copied().find(|&s| x < s).unwrap_or(slots[slots.len() - 1]);
            self.fifo[self.fifo_idx] = self.line_clock + at;
        }
        self.fifo_idx = (self.fifo_idx + 1) & 3;
        if let Some(len) = self.vdp.write_data(w) {
            let start = self.clock.max(self.stall);
            self.busy_until = self.dma_time(start, u64::from(len), DmaKind::Fill).0;
        }
    }

    /// When a DMA started at `start` ends: so many accesses a line by
    /// the mode, to the line's end (the vertical blank's, line 261, in
    /// it), then on from each line's start (Genesis Plus GX's model);
    /// and whether it ran past the first.
    fn dma_time(&self, start: u64, len: u64, kind: DmaKind) -> (u64, bool) {
        let h40 = self.vdp.regs[12] & 1 != 0;
        let display = self.vdp.display_on();
        let mut t = start as i64;
        let mut left = len as i64;
        let mut first = true;
        loop {
            let rel = t.rem_euclid(FRAME as i64) as u64;
            let vblank = if first { self.vdp.in_vblank() } else { rel < VBLANK };
            let blank = vblank || !display;
            let base: i64 = match (blank, h40) {
                (false, false) => 16,
                (false, true) => 18,
                (true, false) => 166,
                (true, true) => 204,
            };
            let rate = match kind {
                DmaKind::Vram | DmaKind::Copy => base >> 1,
                DmaKind::Other => match base {
                    166 => 161,
                    204 => 198,
                    r => r,
                },
                DmaKind::Fill => base,
            };
            if first && kind == DmaKind::Fill {
                t += 2 * (LINE as i64 / rate);
            }
            let end = if vblank {
                t - t.rem_euclid(FRAME as i64) + VBLANK as i64
            } else if first {
                self.line_clock as i64 + LINE as i64
            } else {
                t + LINE as i64
            };
            let can = (end - t) * rate / LINE as i64;
            if left < can {
                return ((t + left * LINE as i64 / rate).max(0) as u64, !first);
            }
            if left == can {
                // done with the window: no more updates
                return (end.max(0) as u64, !first);
            }
            first = false;
            left -= can;
            t = end;
        }
    }
}

impl game::Hardware for Md {
    fn read8(&mut self, addr: u32) -> u8 {
        match addr {
            // the Z80's side answers while the 68000 has its bus
            0xA0_0000..=0xA0_FFFF if !self.sound.granted() => 0xFF,
            0xA0_0000..=0xA0_3FFF => self.sound.bus.ram[(addr & 0x1FFF) as usize],
            0xA0_4000..=0xA0_5FFF => self.sound.bus.ym_read((addr & 3) as u16, self.clock),
            0xA1_0000..=0xA1_001F => self.io_read(addr),
            // the Z80's bus is granted at once (bit 0 clear)
            // bit 0 clear: the bus is the 68000's
            0xA1_1100 => u8::from(!self.sound.granted()),
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
            0xC0_0004 | 0xC0_0006 => self.status(),
            0xC0_0008..=0xC0_000F => self.vdp.read_hv(),
            // the Z80's side is 8 bits wide: the byte, twice
            0xA0_0000..=0xA0_FFFF => u16::from(self.read8(addr)) * 0x0101,
            _ => u16::from(self.read8(addr)) << 8 | u16::from(self.read8(addr + 1)),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        match addr {
            0xA0_0000..=0xA0_FFFF => {
                if self.sound.granted() {
                    self.sound.bus.ym_write_at(self.clock);
                    z80::Bus::write(&mut self.sound.bus, (addr & 0x7FFF) as u16, value);
                }
            }
            0xA1_0000..=0xA1_001F => self.io_write(addr, value),
            0xA1_1100 => self.sound.set_busreq(value & 1 != 0, self.clock),
            0xA1_1200 => self.sound.set_reset(value & 1 == 0, self.clock),
            0xC0_0000..=0xC0_000F => self.write16(addr & !1, u16::from(value) << 8 | u16::from(value)),
            0xC0_0011 | 0xC0_0013 | 0xC0_0015 | 0xC0_0017 => self.sound.bus.psg.write(value),
            _ => {}
        }
    }

    fn write16(&mut self, addr: u32, value: u16) {
        match addr {
            0xC0_0000 | 0xC0_0002 => self.data_write(value),
            0xC0_0004 | 0xC0_0006 => {
                self.vdp.write_control(value);
                if let Some(len) = self.vdp.copied.take() {
                    self.busy_until = self.dma_time(self.clock, u64::from(len), DmaKind::Copy).0;
                }
            }
            0xC0_0010..=0xC0_0017 => self.sound.bus.psg.write(value as u8),
            0xA1_1100 => self.sound.set_busreq(value & 0x100 != 0, self.clock),
            0xA1_1200 => self.sound.set_reset(value & 0x100 == 0, self.clock),
            // the Z80's side is 8 bits wide: the high byte, at the address
            0xA0_0000..=0xA0_FFFF => self.write8(addr, (value >> 8) as u8),
            _ => {
                self.write8(addr, (value >> 8) as u8);
                self.write8(addr + 1, value as u8);
            }
        }
    }

    fn set_time(&mut self, clock: u64, rest: u64) {
        self.clock = clock;
        self.rest = rest;
    }

    fn take_dma(&mut self) -> Option<(u32, u32, bool)> {
        self.vdp.take_dma().map(|Dma::Memory { source, words, vram }| (source, words, vram))
    }

    fn dma_word(&mut self, w: u16) {
        self.vdp.dma_word(w);
    }

    fn dma_done(&mut self, source: u32, words: u32) {
        self.vdp.dma_done(source, words);
    }

    /// Accesses a line (Genesis Plus GX's table): VRAM takes two a word;
    /// to CRAM or VSRAM in the blank a refresh slot is lost every so often.
    fn dma_end(&mut self, start: u64, words: u64, vram: bool) -> (u64, bool) {
        let end = self.dma_time(start, words, if vram { DmaKind::Vram } else { DmaKind::Other });
        // the Z80 waits for the 68000's bus meanwhile
        self.sound.bus.dma_until = end.0;
        end
    }

    fn take_stall(&mut self) -> u64 {
        std::mem::take(&mut self.stall)
    }

    fn display_on(&self) -> bool {
        self.vdp.display_on()
    }

    fn frame_start(&mut self) {
        self.line_clock = self.clock;
        self.frame_clock = self.clock;
        self.fifo = [self.clock; 4];
        self.vint_raised = false;
        self.sound.bus.frame_clock = self.clock;
        self.vdp.frame_start();
    }

    fn vint(&mut self) {
        self.vdp.vint_pending = true;
        self.vint_raised = true;
        self.sound.irq = true;
    }

    fn vint_clock(&self) -> u64 {
        if self.vdp.regs[12] & 1 != 0 { 788 } else { 770 }
    }

    fn line_start(&mut self, line: u16) {
        self.line_clock = self.clock;
        if line == vdp::HEIGHT as u16 + 1 {
            // the Z80's interrupt is held through line 224
            self.sound.irq = false;
        }
        self.vdp.line_start(line);
    }

    fn irq_level(&self) -> u8 {
        self.vdp.irq_level()
    }

    fn int_ack(&mut self, _level: u8) {
        self.vdp.int_ack();
    }

    fn sync(&mut self, t: u64) {
        self.clock = t;
        self.rest = 0;
        self.sound.run_until(t);
    }

    fn z80_granted(&self) -> bool {
        self.sound.granted()
    }

    fn end_frame(&mut self, ram: &mut [u8; 0x10000]) {
        self.sound.bus.frame_end(self.clock);
        if let Some(t) = &mut self.sound.timeline {
            t.end_frame();
        }
        if std::env::var("MEGAFUSE_VDP").ok().and_then(|v| v.parse::<u64>().ok()) == Some(self.frames) {
            eprintln!("regs {:02X?}", self.vdp.regs);
            eprintln!("cram {:03X?}", &self.vdp.cram[..32]);
        }
        self.frames += 1;
        self.frontend.audio(&self.sound.bus.samples);
        self.sound.bus.samples.clear();
        let pads = &mut self.pads;
        let width = self.vdp.width();
        self.frontend.frame(&self.vdp.frame, width, ram, pads);
    }

}
