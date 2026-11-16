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
    /// (`width` x `height` of it: the mode at the frame's start; `frame`,
    /// the frames ended so far, this one included: the machine's count,
    /// which a restored state takes back.)
    fn frame(&mut self, frame: u64, picture: &[u32], width: usize, height: usize, ram: &mut [u8; 0x10000], pads: &mut [u16; 4]);

    /// The VDP as the frame ends, before [`Frontend::frame`] (debugging:
    /// `megafuse --vram`).
    fn vdp(&mut self, _frame: u64, _vdp: &Vdp) {}

    /// The frame's sound, before [`Frontend::frame`]: stereo samples,
    /// interleaved, at [`sound::SAMPLE_RATE`].
    fn audio(&mut self, _samples: &[f32]) {}
}

pub struct Md {
    /// The hardware's state between steps (plain data: a copy is a
    /// snapshot).
    pub state: State,
    frontend: Box<dyn Frontend>,
}

/// Everything the hardware keeps.
#[derive(Clone)]
pub struct State {
    pub vdp: Vdp,
    pub pads: [u16; 4],
    /// The data ports' output bits (TH selects the pad's halves).
    data_out: [u8; 3],
    ctrl: [u8; 3],
    pub sound: Sound,
    pub frames: u64,
    /// What is plugged into the pad ports.
    plug: Plug,
    /// The picture's width and height this frame.
    picture_size: (usize, usize),
    /// The 3-button pads' TH (as last written; 0x40 high) and when, let
    /// go high, it reads high: pads 1-4, and port B's behind a Team
    /// Player.
    pad_th: [u8; 5],
    pad_latency: [u64; 5],
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
    /// A fill or copy runs until then (the status's DMA busy flag), and
    /// is done only once the line it ends in has started (Genesis Plus
    /// GX moves it on at each line's start).
    busy_until: u64,
    busy_last: u64,
    /// The DMA under way (its windows after the first).
    dma: Option<DmaRun>,
    /// The vertical interrupt was raised this frame.
    vint_raised: bool,
    /// The 68000 has to wait until then (a write to a full FIFO).
    stall: u64,
}

impl std::ops::Deref for Md {
    type Target = State;
    fn deref(&self) -> &State {
        &self.state
    }
}

impl std::ops::DerefMut for Md {
    fn deref_mut(&mut self) -> &mut State {
        &mut self.state
    }
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

/// What is plugged into the pad ports.
#[derive(Clone, Copy)]
enum Plug {
    /// A 3-button pad in each (pads 1 and 2).
    Pads,
    /// Sega's Team Player on port A: where its TH/TR are and how far into
    /// its sequence it is.
    TeamPlayer { state: u8, counter: u8 },
    /// EA's 4 Way Play: port B's latched TH, TR, TL pick the pad port A
    /// reads (TH high: its ID).
    WayPlay { latch: u8 },
}

/// A DMA under way: what it moves in each window still to come (when
/// the window starts, how much), and from 68000 memory, the words.
#[derive(Clone)]
struct DmaRun {
    kind: DmaKind,
    windows: std::collections::VecDeque<(u64, u32)>,
    words: Vec<u16>,
    next: usize,
    source: u32,
}

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
            state: State {
            vdp: Vdp::new(),
            pads: [0; 4],
            data_out: [0; 3],
            ctrl: [0; 3],
            sound: Sound::new(rom),
            frames: 0,
            plug: Plug::Pads,
            picture_size: (256, 192),
            pad_th: [0x40; 5],
            pad_latency: [0; 5],
            clock: 0,
            rest: 0,
            line_clock: 0,
            frame_clock: 0,
            fifo: [0; 4],
            fifo_idx: 0,
            busy_until: 0,
            busy_last: 0,
            dma: None,
            vint_raised: false,
            stall: 0,
            },
            frontend,
        }
    }

}

impl State {
    /// A Team Player on port A, pads 1-4 behind it.
    pub fn plug_teamplayer(&mut self) {
        self.plug = Plug::TeamPlayer { state: 0x60, counter: 0 };
    }

    /// EA's 4 Way Play on both ports, pads 1-4 behind it.
    pub fn plug_wayplay(&mut self) {
        self.plug = Plug::WayPlay { latch: 0 };
    }

    /// A pad port's data: what the device puts out, but for the output
    /// bits and bit 7, which read back as written.
    fn pad_read(&self, port: usize) -> u8 {
        let v = match (self.plug, port) {
            (Plug::Pads, p) => self.pad3_read(p, self.pads[p]),
            (Plug::TeamPlayer { state, counter }, 0) => self.teamplayer_read(state, counter),
            (Plug::TeamPlayer { .. }, _) => self.pad3_read(4, 0),
            // the 4 Way Play's ID (port B's TH high), else the pad port B
            // picks
            (Plug::WayPlay { latch }, 0) => {
                if latch & 4 != 0 {
                    0x7C
                } else {
                    let k = usize::from(latch & 3);
                    self.pad3_read(k, self.pads[k])
                }
            }
            (Plug::WayPlay { .. }, _) => 0x7F,
        };
        let out = 0x80 | self.ctrl[port];
        (v & !out) | (self.data_out[port] & out)
    }

    /// A port's output bits change (a data write, or a control write
    /// that changes the directions): the device sees them.
    fn pad_write(&mut self, port: usize) {
        let (data, mask) = (self.data_out[port], self.ctrl[port]);
        match (self.plug, port) {
            (Plug::Pads, p) => self.pad3_write(p, data, mask),
            (Plug::TeamPlayer { state, counter }, 0) => {
                let new = (state & !mask) | (data & mask);
                let counter = if new & 0x40 != 0 {
                    0
                } else if (state ^ new) & 0x60 != 0 {
                    counter.saturating_add(1)
                } else {
                    counter
                };
                self.plug = Plug::TeamPlayer { state: new, counter };
            }
            (Plug::TeamPlayer { .. }, _) => self.pad3_write(4, data, mask),
            (Plug::WayPlay { latch }, 0) => self.pad3_write(usize::from(latch & 3), data, mask),
            (Plug::WayPlay { .. }, _) => {
                // inputs read high; Up and Down low latch TH, TR, TL: the
                // pad port A reads
                let d = data | !mask;
                if d & 3 == 0 {
                    self.plug = Plug::WayPlay { latch: (d >> 4) & 7 };
                }
            }
        }
    }

    /// A 3-button pad (`pad` of the five Genesis Plus GX keeps: four, and
    /// port B's behind a Team Player): TH high gives C B Right Left Down
    /// Up, low Start A 0 0 Down Up (active low).  TH is what was last
    /// written to it; let go as an input it rises only a little later.
    fn pad3_read(&self, pad: usize, buttons: u16) -> u8 {
        let p = !buttons;
        let th = self.pad_th[pad];
        let high = th != 0 && self.clock >= self.pad_latency[pad];
        let low = if high {
            (p & 0x3F) as u8
        } else {
            (((p >> 2) & 0x30) as u8) | (p & 0x03) as u8
        };
        th | low
    }

    fn pad3_write(&mut self, pad: usize, data: u8, mask: u8) {
        if mask & 0x40 != 0 {
            self.pad_th[pad] = data & 0x40;
            self.pad_latency[pad] = 0;
        } else {
            // pulled high, 172 clocks on
            if self.pad_th[pad] == 0 {
                self.pad_latency[pad] = self.clock + 172;
            }
            self.pad_th[pad] = 0x40;
        }
    }

    /// The Team Player, four 3-button pads behind it: TH high starts
    /// over; each change of TH or TR the 68000 makes moves it on through
    /// its header (0011, 1111, 0000, 0000), the pads' types (0000: 3
    /// buttons) and each pad's two nibbles (RLDU, SACB; active low); TL
    /// follows TR.
    fn teamplayer_read(&self, state: u8, counter: u8) -> u8 {
        let tl = (state & 0x20) >> 1;
        let low = match counter {
            0 => 0x03,
            1 => 0x0F,
            2..=7 => 0x00,
            n => {
                let k = usize::from(n - 8);
                if k >= 8 {
                    0x0F
                } else {
                    let pad = self.pads[k / 2];
                    0x0F & !((pad >> (4 * (k % 2))) as u8)
                }
            }
        };
        tl | low
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
            0x03 => {
                self.data_out[0] = v;
                self.pad_write(0);
            }
            0x05 => {
                self.data_out[1] = v;
                self.pad_write(1);
            }
            0x07 => self.data_out[2] = v,
            0x09 | 0x0B => {
                let port = usize::from((a & 0x1F) as u8 - 0x09) / 2;
                if v != self.ctrl[port] {
                    self.ctrl[port] = v;
                    self.pad_write(port);
                }
            }
            0x0D => self.ctrl[2] = v,
            _ => {}
        }
    }
}

impl State {
    /// The status register, as of the end of the instruction reading
    /// it: DMA busy, the FIFO empty or full, the blanks, the vertical
    /// interrupt (its flag up from its time, even before it is raised).
    fn status(&mut self) -> u16 {
        let t = self.clock + self.rest;
        let mut s = self.vdp.read_status();
        if t < self.busy_until || self.line_clock < self.busy_last {
            s |= 0x0002;
        }
        if t >= self.fifo[(self.fifo_idx + 3) & 3] {
            s |= 0x0200;
        } else if t < self.fifo[self.fifo_idx] {
            s |= 0x0100;
        }
        let x = t.saturating_sub(self.line_clock);
        let h40 = self.vdp.regs[12] & 1 != 0;
        if self.vdp.line == vdp::HEIGHT as u16 && !self.vint_raised && x >= self.vint_clock() {
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
            let (k, at) = (self.fifo_idx, self.line_clock + at);
            self.fifo[k] = at;
        }
        self.fifo_idx = (self.fifo_idx + 1) & 3;
        if let Some(len) = self.vdp.write_data(w) {
            let start = self.clock.max(self.stall);
            let (end, last, _, windows) = self.dma_plan(start, u64::from(len), DmaKind::Fill);
            (self.busy_until, self.busy_last) = (end, last);
            self.dma_start(DmaRun { kind: DmaKind::Fill, windows: windows.into(), words: Vec::new(), next: 0, source: 0 });
        }
    }

    /// A DMA starts: its first window now, the rest as their lines start
    /// (each before the line is drawn).
    fn dma_start(&mut self, run: DmaRun) {
        self.dma = Some(run);
        self.dma_progress(0);
    }

    /// The DMA's windows that have started by `t`.
    fn dma_progress(&mut self, t: u64) {
        while let Some(run) = &mut self.dma {
            let Some(&(at, n)) = run.windows.front() else { break };
            if at > t {
                break;
            }
            run.windows.pop_front();
            match run.kind {
                DmaKind::Vram | DmaKind::Other => {
                    for _ in 0..n {
                        if let Some(&w) = run.words.get(run.next) {
                            self.vdp.dma_word(w);
                        }
                        run.next += 1;
                    }
                }
                DmaKind::Fill => self.vdp.fill_step(n),
                DmaKind::Copy => self.vdp.copy_step(n),
            }
            if run.windows.is_empty() {
                let run = self.dma.take().unwrap_or_else(|| unreachable!());
                match run.kind {
                    DmaKind::Vram | DmaKind::Other => self.vdp.dma_done(run.source, run.words.len() as u32),
                    _ => self.vdp.fill_done(),
                }
            }
        }
    }

    /// When a DMA started at `start` ends: so many accesses a line by
    /// the mode, to the line's end (the vertical blank's, line 261, in
    /// it), then on from each line's start (Genesis Plus GX's model);
    /// where its last window starts (0: the first) and whether it ran
    /// past the first.
    /// And how much it moves in each of its windows: (when the window
    /// starts, 0 for the first; how many words or bytes).
    #[allow(clippy::type_complexity)]
    fn dma_plan(&self, start: u64, len: u64, kind: DmaKind) -> (u64, u64, bool, Vec<(u64, u32)>) {
        let mut windows = Vec::new();
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
            let last = if first { 0 } else { t.max(0) as u64 };
            if left < can {
                windows.push((last, left as u32));
                return ((t + left * LINE as i64 / rate).max(0) as u64, last, !first, windows);
            }
            windows.push((last, can.max(0) as u32));
            if left == can {
                // done with the window: no more updates
                return (end.max(0) as u64, last, !first, windows);
            }
            first = false;
            left -= can;
            t = end;
        }
    }
}

impl State {
    pub(crate) fn read8(&mut self, addr: u32) -> u8 {
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

    pub(crate) fn read16(&mut self, addr: u32) -> u16 {
        match addr {
            0xC0_0000 | 0xC0_0002 => self.vdp.read_data(),
            0xC0_0004 | 0xC0_0006 => self.status(),
            0xC0_0008..=0xC0_000F => self.vdp.read_hv(),
            // the Z80's side is 8 bits wide: the byte, twice
            0xA0_0000..=0xA0_FFFF => u16::from(self.read8(addr)) * 0x0101,
            _ => u16::from(self.read8(addr)) << 8 | u16::from(self.read8(addr + 1)),
        }
    }

    pub(crate) fn write8(&mut self, addr: u32, value: u8) {
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

    pub(crate) fn write16(&mut self, addr: u32, value: u16) {
        match addr {
            0xC0_0000 | 0xC0_0002 => self.data_write(value),
            0xC0_0004 | 0xC0_0006 => {
                let display = self.vdp.display_on();
                let mode = self.vdp.regs[12] & 1;
                self.vdp.write_control(value);
                if self.vdp.display_on() != display {
                    self.vdp.display_switched(self.clock - self.line_clock);
                }
                if self.vdp.regs[12] & 1 != mode {
                    self.vdp.mode_switched();
                }
                if let Some(len) = self.vdp.copied.take() {
                    let (end, last, _, windows) = self.dma_plan(self.clock, u64::from(len), DmaKind::Copy);
                    (self.busy_until, self.busy_last) = (end, last);
                    self.dma_start(DmaRun {
                        kind: DmaKind::Copy,
                        windows: windows.into(),
                        words: Vec::new(),
                        next: 0,
                        source: 0,
                    });
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

    pub(crate) fn set_time(&mut self, clock: u64, rest: u64) {
        self.clock = clock;
        self.rest = rest;
    }

    pub(crate) fn take_dma(&mut self) -> Option<(u32, u32, bool)> {
        self.vdp.take_dma().map(|Dma::Memory { source, words, vram }| (source, words, vram))
    }

    /// Accesses a line (Genesis Plus GX's table): VRAM takes two a word;
    /// to CRAM or VSRAM in the blank a refresh slot is lost every so often.
    pub(crate) fn dma_run(&mut self, start: u64, source: u32, data: Vec<u16>, vram: bool) -> (u64, bool) {
        let kind = if vram { DmaKind::Vram } else { DmaKind::Other };
        let (end, _, on, windows) = self.dma_plan(start, data.len() as u64, kind);
        self.dma_start(DmaRun { kind, windows: windows.into(), words: data, next: 0, source });
        // the Z80 waits for the 68000's bus meanwhile
        self.sound.bus.dma_until = end;
        (end, on)
    }

    pub(crate) fn take_stall(&mut self) -> u64 {
        std::mem::take(&mut self.stall)
    }

    pub(crate) fn display_on(&self) -> bool {
        self.vdp.display_on()
    }

    pub(crate) fn frame_start(&mut self) {
        self.dma_progress(self.clock);
        // the picture's lines are the mode's at the frame's start (mode 4,
        // the VDP's at power-on, has 192); its width the VDP's at its end
        let lines = if self.vdp.regs[1] & 4 != 0 { vdp::HEIGHT } else { 192 };
        self.picture_size.1 = lines;
        self.line_clock = self.clock;
        self.frame_clock = self.clock;
        self.fifo = [self.clock; 4];
        self.vint_raised = false;
        self.sound.bus.frame_clock = self.clock;
        self.vdp.frame_start();
    }

    pub(crate) fn vint(&mut self) {
        self.vdp.vint_pending = true;
        self.vint_raised = true;
        self.sound.irq = true;
    }

    pub(crate) fn vint_clock(&self) -> u64 {
        if self.vdp.regs[12] & 1 != 0 { 788 } else { 770 }
    }

    pub(crate) fn line_start(&mut self, line: u16) {
        self.line_clock = self.clock;
        self.dma_progress(self.clock);
        if line == vdp::HEIGHT as u16 + 1 {
            // the Z80's interrupt is held through line 224
            self.sound.irq = false;
        }
        self.vdp.line_start(line);
    }

    pub(crate) fn irq_level(&self) -> u8 {
        self.vdp.irq_level()
    }

    pub(crate) fn int_ack(&mut self, _level: u8) {
        self.vdp.int_ack();
    }



    pub(crate) fn sync(&mut self, t: u64) {
        self.clock = t;
        self.rest = 0;
        self.sound.run_until(t);
    }

    pub(crate) fn z80_granted(&self) -> bool {
        self.sound.granted()
    }


}

impl game::Hardware for Md {
    fn read8(&mut self, addr: u32) -> u8 {
        self.state.read8(addr)
    }

    fn read16(&mut self, addr: u32) -> u16 {
        self.state.read16(addr)
    }

    fn write8(&mut self, addr: u32, value: u8) {
        self.state.write8(addr, value)
    }

    fn write16(&mut self, addr: u32, value: u16) {
        self.state.write16(addr, value)
    }

    fn set_time(&mut self, clock: u64, rest: u64) {
        self.state.set_time(clock, rest)
    }

    fn take_dma(&mut self) -> Option<(u32, u32, bool)> {
        self.state.take_dma()
    }

    fn dma_run(&mut self, start: u64, source: u32, data: Vec<u16>, vram: bool) -> (u64, bool) {
        self.state.dma_run(start, source, data, vram)
    }

    fn take_stall(&mut self) -> u64 {
        self.state.take_stall()
    }

    fn display_on(&self) -> bool {
        self.state.display_on()
    }

    fn frame_start(&mut self) {
        self.state.frame_start()
    }

    fn vint(&mut self) {
        self.state.vint()
    }

    fn vint_clock(&self) -> u64 {
        self.state.vint_clock()
    }

    fn line_start(&mut self, line: u16) {
        self.state.line_start(line)
    }

    fn irq_level(&self) -> u8 {
        self.state.irq_level()
    }

    fn int_ack(&mut self, _level: u8) {
        self.state.int_ack(_level)
    }

    fn sync(&mut self, t: u64) {
        self.state.sync(t)
    }

    fn z80_granted(&self) -> bool {
        self.state.z80_granted()
    }

    fn save(&self) -> Box<dyn std::any::Any + Send> {
        Box::new(self.state.clone())
    }

    fn restore(&mut self, state: &dyn std::any::Any) {
        if let Some(s) = state.downcast_ref::<State>() {
            self.state = s.clone();
        }
    }

    fn end_frame(&mut self, ram: &mut [u8; 0x10000]) {
        self.state.sound.bus.frame_end(self.state.clock);
        if let Some(t) = &self.state.sound.timeline {
            t.lock().unwrap_or_else(std::sync::PoisonError::into_inner).end_frame();
        }
        if std::env::var("MEGAFUSE_VDP").ok().and_then(|v| v.parse::<u64>().ok()) == Some(self.state.frames) {
            eprintln!("regs {:02X?}", self.state.vdp.regs);
            eprintln!("cram {:03X?}", &self.state.vdp.cram[..32]);
        }
        self.state.frames += 1;
        self.frontend.vdp(self.state.frames, &self.state.vdp);
        self.frontend.audio(&self.state.sound.bus.samples);
        self.state.sound.bus.samples.clear();
        let pads = &mut self.state.pads;
        let (width, height) = (self.state.vdp.width(), self.state.picture_size.1);
        self.frontend.frame(self.state.frames, &self.state.vdp.frame, width, height, ram, pads);
    }
}
