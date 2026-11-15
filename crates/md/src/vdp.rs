//! The VDP (Sega 315-5313) as Mega Bomberman drives it: registers,
//! VRAM/CRAM/VSRAM through the control and data ports, DMA, the status
//! and HV counter, and the picture, a line at a time.

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 224;
pub const LINES: u16 = 262;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dma {
    /// From 68000 memory: the source (a byte address) and its length in
    /// words; `Game` feeds the words through [`Vdp::dma_word`].
    Memory { source: u32, words: u32 },
}

pub struct Vdp {
    pub regs: [u8; 24],
    pub vram: Box<[u8; 0x10000]>,
    pub cram: [u16; 64],
    pub vsram: [u16; 40],
    addr: u32,
    code: u8,
    pending: bool,
    fill_pending: bool,
    dma: Option<Dma>,
    /// The line being drawn (0-261).
    pub line: u16,
    /// The horizontal interrupt's line counter.
    hcount: i16,
    pub vint_pending: bool,
    pub hint_pending: bool,
    /// The picture, RGB, 320x224 (a 256-wide mode is drawn centred).
    pub frame: Vec<u32>,
}

impl Default for Vdp {
    fn default() -> Self {
        Self::new()
    }
}

impl Vdp {
    #[must_use]
    pub fn new() -> Self {
        Vdp {
            regs: [0; 24],
            vram: Box::new([0; 0x10000]),
            cram: [0; 64],
            vsram: [0; 40],
            addr: 0,
            code: 0,
            pending: false,
            fill_pending: false,
            dma: None,
            line: 0,
            hcount: 0,
            vint_pending: false,
            hint_pending: false,
            frame: vec![0; WIDTH * HEIGHT],
        }
    }

    fn autoinc(&self) -> u32 {
        u32::from(self.regs[15])
    }

    pub fn display_on(&self) -> bool {
        self.regs[1] & 0x40 != 0
    }

    pub fn vint_enabled(&self) -> bool {
        self.regs[1] & 0x20 != 0
    }

    pub fn hint_enabled(&self) -> bool {
        self.regs[0] & 0x10 != 0
    }

    fn h40(&self) -> bool {
        self.regs[12] & 0x01 != 0
    }

    // --- the ports -----------------------------------------------------------

    pub fn write_control(&mut self, w: u16) {
        if !self.pending && w & 0xC000 == 0x8000 {
            let r = ((w >> 8) & 0x1F) as usize;
            if r < self.regs.len() {
                self.regs[r] = w as u8;
            }
            self.code = 0;
            return;
        }
        if !self.pending {
            self.addr = (self.addr & 0xC000) | u32::from(w & 0x3FFF);
            self.code = (self.code & 0x3C) | (w >> 14) as u8;
            self.pending = true;
            return;
        }
        self.pending = false;
        self.addr = (self.addr & 0x3FFF) | (u32::from(w & 3) << 14);
        self.code = (self.code & 0x03) | ((w >> 2) & 0x3C) as u8;
        if std::env::var_os("MEGAFUSE_DMA").is_some() && self.code & 0x20 != 0 {
            eprintln!("dma code {:02X} addr {:04X} mode {:02X} len {:02X}{:02X} src {:02X}{:02X}{:02X} reg1 {:02X}",
                self.code, self.addr, self.regs[23] >> 6, self.regs[20], self.regs[19], self.regs[23], self.regs[22], self.regs[21], self.regs[1]);
        }
        if self.code & 0x20 != 0 && self.regs[1] & 0x10 != 0 {
            match self.regs[23] >> 6 {
                0 | 1 => {
                    let source = (u32::from(self.regs[23] & 0x7F) << 17)
                        | (u32::from(self.regs[22]) << 9)
                        | (u32::from(self.regs[21]) << 1);
                    let mut words =
                        u32::from(self.regs[20]) << 8 | u32::from(self.regs[19]);
                    if words == 0 {
                        words = 0x10000;
                    }
                    self.dma = Some(Dma::Memory { source, words });
                }
                2 => self.fill_pending = true,
                _ => self.copy(),
            }
        }
    }

    /// A DMA from 68000 memory the last control write started.
    pub fn take_dma(&mut self) -> Option<Dma> {
        self.dma.take()
    }

    /// A word of a DMA from 68000 memory.
    pub fn dma_word(&mut self, w: u16) {
        self.write_target(w);
        self.addr = (self.addr + self.autoinc()) & 0xFFFF;
    }

    /// After a DMA from memory: the length counter and the source as
    /// they are left.
    pub fn dma_done(&mut self, source: u32, words: u32) {
        let end = (source >> 1).wrapping_add(words);
        self.regs[19] = 0;
        self.regs[20] = 0;
        self.regs[21] = end as u8;
        self.regs[22] = (end >> 8) as u8;
    }

    fn copy(&mut self) {
        let mut src = u32::from(self.regs[22]) << 8 | u32::from(self.regs[21]);
        let mut len = u32::from(self.regs[20]) << 8 | u32::from(self.regs[19]);
        if len == 0 {
            len = 0x10000;
        }
        for _ in 0..len {
            let b = self.vram[(src & 0xFFFF) as usize];
            self.vram[(self.addr & 0xFFFF) as usize] = b;
            src = src.wrapping_add(1);
            self.addr = (self.addr + self.autoinc()) & 0xFFFF;
        }
    }

    fn write_target(&mut self, w: u16) {
        match self.code & 0x0F {
            1 => {
                let a = (self.addr & 0xFFFF) as usize;
                let [hi, lo] = w.to_be_bytes();
                if a & 1 == 0 {
                    self.vram[a] = hi;
                    self.vram[(a + 1) & 0xFFFF] = lo;
                } else {
                    self.vram[a] = lo;
                    self.vram[a - 1] = hi;
                }
            }
            3 => self.cram[((self.addr >> 1) & 0x3F) as usize] = w & 0x0EEE,
            5 => {
                let i = ((self.addr >> 1) & 0x3F) as usize;
                if i < self.vsram.len() {
                    self.vsram[i] = w & 0x07FF;
                }
            }
            _ => {}
        }
    }

    pub fn write_data(&mut self, w: u16) {
        self.pending = false;
        if self.fill_pending {
            self.fill_pending = false;
            self.write_target(w);
            let mut len = u32::from(self.regs[20]) << 8 | u32::from(self.regs[19]);
            if len == 0 {
                len = 0x10000;
            }
            let fill = (w >> 8) as u8;
            for _ in 0..len {
                self.addr = (self.addr + self.autoinc()) & 0xFFFF;
                self.vram[(self.addr ^ 1) as usize & 0xFFFF] = fill;
            }
            return;
        }
        self.write_target(w);
        self.addr = (self.addr + self.autoinc()) & 0xFFFF;
    }

    pub fn read_data(&mut self) -> u16 {
        self.pending = false;
        let v = match self.code & 0x0F {
            0 => {
                let a = (self.addr & 0xFFFE) as usize;
                u16::from_be_bytes([self.vram[a], self.vram[a + 1]])
            }
            4 => self
                .vsram
                .get(((self.addr >> 1) & 0x3F) as usize)
                .copied()
                .unwrap_or(0),
            8 => self.cram[((self.addr >> 1) & 0x3F) as usize],
            _ => 0,
        };
        self.addr = (self.addr + self.autoinc()) & 0xFFFF;
        v
    }

    /// The status register: FIFO empty, in the vertical blank, the
    /// vertical interrupt pending; DMA is never busy (it is done at once).
    pub fn read_status(&mut self) -> u16 {
        self.pending = false;
        let mut s = 0x3400 | 0x0200;
        if self.line >= HEIGHT as u16 || !self.display_on() {
            s |= 0x0008;
        }
        if self.vint_pending {
            s |= 0x0080;
        }
        s
    }

    /// The HV counter: the line (NTSC numbering) and a horizontal
    /// position that moves on with each read.  A read in a loop waiting
    /// for a line advances the beam: [`Vdp::advance_on_hv_read`].
    pub fn read_hv(&self) -> u16 {
        let v = if self.line <= 0xEA { self.line } else { self.line - 6 };
        (v & 0xFF) << 8 | 0x80
    }

    // --- the frame ---------------------------------------------------------

    /// Draws `line` (if it is a visible one) and counts the horizontal
    /// interrupt's lines; returns whether it fires after this line.
    pub fn run_line(&mut self, line: u16) -> bool {
        self.line = line;
        if (line as usize) < HEIGHT {
            self.draw_line(line as usize);
        }
        let mut fire = false;
        if line <= HEIGHT as u16 {
            if self.hcount <= 0 {
                self.hcount = i16::from(self.regs[10]);
                fire = true;
            } else {
                self.hcount -= 1;
            }
        } else {
            self.hcount = i16::from(self.regs[10]);
        }
        if fire && self.hint_enabled() {
            self.hint_pending = true;
            return true;
        }
        false
    }

    fn colour(&self, index: usize) -> u32 {
        let c = self.cram[index & 0x3F];
        let r = u32::from((c >> 1) & 7);
        let g = u32::from((c >> 5) & 7);
        let b = u32::from((c >> 9) & 7);
        let s = |v: u32| (v * 255) / 7;
        s(r) << 16 | s(g) << 8 | s(b)
    }

    fn tile_pixel(&self, entry: u16, x: usize, y: usize) -> u8 {
        let tile = usize::from(entry & 0x07FF);
        let px = if entry & 0x0800 != 0 { 7 - x } else { x };
        let py = if entry & 0x1000 != 0 { 7 - y } else { y };
        let byte = self.vram[(tile * 32 + py * 4 + px / 2) & 0xFFFF];
        if px & 1 == 0 { byte >> 4 } else { byte & 0x0F }
    }

    fn plane_size(&self) -> (usize, usize) {
        let w = match self.regs[16] & 3 {
            0 => 32,
            1 => 64,
            _ => 128,
        };
        let h = match (self.regs[16] >> 4) & 3 {
            0 => 32,
            1 => 64,
            _ => 128,
        };
        (w, h)
    }

    fn hscroll(&self, line: usize) -> (u16, u16) {
        let base = (usize::from(self.regs[13] & 0x3F)) << 10;
        let off = match self.regs[11] & 3 {
            0 => 0,
            2 => (line & !7) * 4,
            3 => line * 4,
            _ => (line & 7) * 4,
        };
        let a = (base + off) & 0xFFFF;
        let word = |a: usize| u16::from_be_bytes([self.vram[a], self.vram[(a + 1) & 0xFFFF]]);
        (word(a) & 0x3FF, word(a + 2) & 0x3FF)
    }

    fn plane_line(&self, base: usize, hs: u16, column_vs: &dyn Fn(usize) -> u16, line: usize, width: usize, out: &mut [(u8, bool)]) {
        let (pw, ph) = self.plane_size();
        for (x, o) in out.iter_mut().enumerate().take(width) {
            let sx = (x as i32 - i32::from(hs)).rem_euclid((pw * 8) as i32) as usize;
            let vs = usize::from(column_vs(x));
            let sy = (line + vs) % (ph * 8);
            let a = (base + ((sy / 8) * pw + sx / 8) * 2) & 0xFFFF;
            let entry = u16::from_be_bytes([self.vram[a], self.vram[(a + 1) & 0xFFFF]]);
            let p = self.tile_pixel(entry, sx & 7, sy & 7);
            *o = if p == 0 {
                (0, entry & 0x8000 != 0)
            } else {
                ((((entry >> 13) & 3) as u8) << 4 | p, entry & 0x8000 != 0)
            };
        }
    }

    /// The picture's width in the current mode (H40: 320, H32: 256).
    #[must_use]
    pub fn width(&self) -> usize {
        if self.h40() { 320 } else { 256 }
    }

    fn draw_line(&mut self, line: usize) {
        let width = self.width();
        // the picture is kept left-aligned, `width` pixels of each line
        let offset = 0;
        let bg = self.colour(usize::from(self.regs[7] & 0x3F));
        if !self.display_on() {
            for x in 0..WIDTH {
                self.frame[line * WIDTH + x] = bg;
            }
            return;
        }
        let (hs_a, hs_b) = self.hscroll(line);
        let per_column = self.regs[11] & 4 != 0;
        let vsram = self.vsram;
        let vs_a = move |x: usize| {
            if per_column { vsram[(x / 16 * 2).min(38)] } else { vsram[0] }
        };
        let vs_b = move |x: usize| {
            if per_column { vsram[(x / 16 * 2 + 1).min(39)] } else { vsram[1] }
        };
        let mut a = [(0u8, false); WIDTH];
        let mut b = [(0u8, false); WIDTH];
        let base_a = usize::from(self.regs[2] & 0x38) << 10;
        let base_b = usize::from(self.regs[4] & 0x07) << 13;
        self.plane_line(base_a, hs_a, &vs_a, line, width, &mut a);
        self.plane_line(base_b, hs_b, &vs_b, line, width, &mut b);
        self.window(line, width, &mut a);
        let mut s = [(0u8, false); WIDTH];
        self.sprites(line, width, &mut s);
        for x in 0..width {
            let mut px = None;
            for (layer, hi) in [(&s, true), (&a, true), (&b, true), (&s, false), (&a, false), (&b, false)] {
                let (c, h) = layer[x];
                if c & 0x0F != 0 && h == hi {
                    px = Some(c);
                    break;
                }
            }
            self.frame[line * WIDTH + offset + x] = match px {
                Some(c) => self.colour(usize::from(c)),
                None => bg,
            };
        }
        for x in width..WIDTH {
            self.frame[line * WIDTH + x] = 0;
        }
    }

    fn window(&self, line: usize, width: usize, a: &mut [(u8, bool)]) {
        let wx = usize::from(self.regs[17] & 0x1F) * 16;
        let right = self.regs[17] & 0x80 != 0;
        let wy = usize::from(self.regs[18] & 0x1F) * 8;
        let down = self.regs[18] & 0x80 != 0;
        let in_rows = if down { line >= wy } else { line < wy };
        let base = if self.h40() {
            usize::from(self.regs[3] & 0x3C) << 10
        } else {
            usize::from(self.regs[3] & 0x3E) << 10
        };
        let pw = if self.h40() { 64 } else { 32 };
        for (x, o) in a.iter_mut().enumerate().take(width) {
            let in_cols = if right { x >= wx } else { x < wx };
            if !(in_rows || in_cols) || (wx == 0 && !right && !in_rows) {
                continue;
            }
            let ad = (base + ((line / 8) * pw + x / 8) * 2) & 0xFFFF;
            let entry = u16::from_be_bytes([self.vram[ad], self.vram[(ad + 1) & 0xFFFF]]);
            let p = self.tile_pixel(entry, x & 7, line & 7);
            *o = if p == 0 {
                (0, entry & 0x8000 != 0)
            } else {
                ((((entry >> 13) & 3) as u8) << 4 | p, entry & 0x8000 != 0)
            };
        }
    }

    fn sprites(&self, line: usize, width: usize, out: &mut [(u8, bool)]) {
        let base = if self.h40() {
            usize::from(self.regs[5] & 0x7E) << 9
        } else {
            usize::from(self.regs[5] & 0x7F) << 9
        };
        let max = if self.h40() { 80 } else { 64 };
        let per_line = if self.h40() { 20 } else { 16 };
        let mut link = 0usize;
        let mut shown = 0;
        let y = line as i32 + 128;
        for _ in 0..max {
            let e = (base + link * 8) & 0xFFFF;
            let w = |o: usize| u16::from_be_bytes([self.vram[(e + o) & 0xFFFF], self.vram[(e + o + 1) & 0xFFFF]]);
            let sy = i32::from(w(0) & 0x3FF);
            let size = w(2) >> 8;
            let next = usize::from(w(2) & 0x7F);
            let attr = w(4);
            let sx = i32::from(w(6) & 0x1FF);
            let cw = usize::from((size >> 2) & 3) + 1;
            let ch = usize::from(size & 3) + 1;
            if y >= sy && y < sy + ch as i32 * 8 {
                shown += 1;
                if shown > per_line {
                    break;
                }
                let row = (y - sy) as usize;
                let row = if attr & 0x1000 != 0 { ch * 8 - 1 - row } else { row };
                for col in 0..cw * 8 {
                    let px = sx - 128 + col as i32;
                    if px < 0 || px as usize >= width {
                        continue;
                    }
                    let c = if attr & 0x0800 != 0 { cw * 8 - 1 - col } else { col };
                    let tile = usize::from(attr & 0x07FF) + (c / 8) * ch + row / 8;
                    let entry = (attr & 0xF800 & !0x1800) | (tile as u16 & 0x07FF);
                    let p = self.tile_pixel(entry, c & 7, row & 7);
                    let o = &mut out[px as usize];
                    if p != 0 && o.0 & 0x0F == 0 {
                        *o = ((((attr >> 13) & 3) as u8) << 4 | p, attr & 0x8000 != 0);
                    }
                }
            }
            if next == 0 || next >= max {
                break;
            }
            link = next;
        }
    }
}
