//! The VDP (Sega 315-5313) as Mega Bomberman drives it: registers,
//! VRAM/CRAM/VSRAM through the control and data ports, DMA, the status
//! and HV counter, and the picture, a line at a time.

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 224;
pub const LINES: u16 = 262;

/// A sprite on a line: its attributes (from VRAM), X, the row of it on
/// the line, its size.
#[derive(Clone, Copy)]
struct Obj {
    attr: u16,
    x: u16,
    row: u16,
    size: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dma {
    /// From 68000 memory: the source (a byte address) and its length in
    /// words; `Game` feeds the words through [`Vdp::dma_word`].
    Memory { source: u32, words: u32, vram: bool },
}

#[derive(Clone)]
pub struct Vdp {
    pub regs: [u8; 24],
    pub vram: Box<[u8; 0x10000]>,
    pub cram: [u16; 64],
    pub vsram: [u16; 40],
    addr: u32,
    code: u8,
    pending: bool,
    fill_pending: bool,
    /// A fill or copy under way: the byte a fill writes, a copy's source.
    fill: u8,
    copy_src: u16,
    dma: Option<Dma>,
    /// A VRAM copy's length, just done.
    pub copied: Option<u32>,
    /// The VDP's own copy of the sprite table (written along with VRAM
    /// there): the Y, size and link it goes by.
    pub sat: Box<[u8; 0x400]>,
    /// The sprites of a line, found on the line before (even, odd
    /// lines), and whether sprite masking carries over to the next line.
    objs: [Vec<Obj>; 2],
    /// How many of a line's sprites the console has (the rest are there
    /// only without the limit).
    kept: [usize; 2],
    spr_ovr: bool,
    /// No sprite limit (a port extra, docs/PORT.md): every sprite on a
    /// line is drawn, whole. The status flags and the masking stay the
    /// console's, so the game runs the same with it on or off.
    pub unlimited: bool,
    /// Sprite overflow (0x40) and collision (0x20), till the status is
    /// read.
    pub flags: u16,
    /// Sprite pixels a line (0: the mode's 256 or 320; a line the
    /// display came back on late in its blank gets 128).
    sprite_pixels: usize,
    /// The picture's width, and a change of it waiting for the frame.
    view_w: usize,
    view_pending: bool,
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
            fill: 0,
            copy_src: 0,
            dma: None,
            copied: None,
            sat: Box::new([0; 0x400]),
            objs: [Vec::new(), Vec::new()],
            kept: [0; 2],
            spr_ovr: false,
            unlimited: false,
            flags: 0,
            sprite_pixels: 0,
            view_w: 256,
            view_pending: false,
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
                    // code 1: VRAM (a word is two accesses); 3, 5: CRAM, VSRAM
                    let vram = self.code & 0x06 == 0;
                    self.dma = Some(Dma::Memory { source, words, vram });
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

    /// A VRAM copy, done at once; its length is kept for `Md` (the time
    /// it takes).
    /// A VRAM copy starts: its length is kept for `Md`, which runs it
    /// ([`Vdp::copy_step`]) at its pace.
    fn copy(&mut self) {
        self.copy_src = u16::from(self.regs[22]) << 8 | u16::from(self.regs[21]);
        self.copied = Some(self.dma_length());
    }

    fn dma_length(&self) -> u32 {
        let len = u32::from(self.regs[20]) << 8 | u32::from(self.regs[19]);
        if len == 0 { 0x10000 } else { len }
    }

    /// `n` bytes of a copy (Genesis Plus GX's: the byte at source ^ 1 to
    /// address ^ 1).
    pub fn copy_step(&mut self, n: u32) {
        for _ in 0..n {
            let b = self.vram[usize::from(self.copy_src ^ 1)];
            self.vram_put((self.addr ^ 1) as usize & 0xFFFF, b);
            self.copy_src = self.copy_src.wrapping_add(1);
            self.addr = (self.addr + self.autoinc()) & 0xFFFF;
        }
    }

    /// `n` bytes of a fill.
    pub fn fill_step(&mut self, n: u32) {
        for _ in 0..n {
            self.vram_put((self.addr ^ 1) as usize & 0xFFFF, self.fill);
            self.addr = (self.addr + self.autoinc()) & 0xFFFF;
        }
    }

    /// A fill or copy is done: the source moved on by the length, the
    /// length run out.
    pub fn fill_done(&mut self) {
        let src = u16::from(self.regs[22]) << 8 | u16::from(self.regs[21]);
        let len = u16::from(self.regs[20]) << 8 | u16::from(self.regs[19]);
        let end = src.wrapping_add(len);
        self.regs[21] = end as u8;
        self.regs[22] = (end >> 8) as u8;
        self.regs[19] = 0;
        self.regs[20] = 0;
    }

    /// A byte into VRAM; into the sprite table's copy too if it lands in
    /// the table (its first 512 bytes in H32, 1 KB in H40).
    fn vram_put(&mut self, a: usize, b: u8) {
        self.vram[a] = b;
        let (base, index) = self.sat_masks();
        if a & base == self.sat_base() {
            self.sat[a & index] = b;
        }
    }

    fn sat_masks(&self) -> (usize, usize) {
        if self.h40() { (0xFC00, 0x3FF) } else { (0xFE00, 0x1FF) }
    }

    fn sat_base(&self) -> usize {
        (usize::from(self.regs[5]) << 9) & self.sat_masks().0
    }

    fn write_target(&mut self, w: u16) {
        match self.code & 0x0F {
            1 => {
                let a = (self.addr & 0xFFFF) as usize;
                let [hi, lo] = w.to_be_bytes();
                if a & 1 == 0 {
                    self.vram_put(a, hi);
                    self.vram_put((a + 1) & 0xFFFF, lo);
                } else {
                    self.vram_put(a, lo);
                    self.vram_put(a - 1, hi);
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

    /// A write to the data port; a VRAM fill it starts returns its
    /// length (`Md` runs it, [`Vdp::fill_step`], at its pace).
    pub fn write_data(&mut self, w: u16) -> Option<u32> {
        self.pending = false;
        self.write_target(w);
        self.addr = (self.addr + self.autoinc()) & 0xFFFF;
        if self.fill_pending {
            self.fill_pending = false;
            self.fill = (w >> 8) as u8;
            return Some(self.dma_length());
        }
        None
    }

    /// In the vertical blank (the status flag): line 224 to 261's start.
    #[must_use]
    pub fn in_vblank(&self) -> bool {
        (HEIGHT as u16..LINES - 1).contains(&self.line)
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
        let mut s = self.flags;
        self.flags = 0;
        if self.in_vblank() || !self.display_on() {
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

    /// The frame starts (line 224): a change of width put off takes
    /// effect; the horizontal interrupt fires if its counter ran out on
    /// the last line.
    pub fn frame_start(&mut self) {
        if self.view_pending {
            self.view_pending = false;
            self.view_w = if self.h40() { 320 } else { 256 };
        }
        self.line = HEIGHT as u16;
        if self.hcount == 0 {
            self.hint_pending = true;
        }
    }

    /// A line starts (all but 224): drawn if it is a visible one; the
    /// horizontal interrupt's counter counts the visible lines down,
    /// fires and starts again from register 10 when it runs out; it is
    /// reloaded at line 261.
    pub fn line_start(&mut self, line: u16) {
        self.line = line;
        if (line as usize) < HEIGHT {
            self.draw_line(line as usize);
            // the sprites of the next line
            if self.display_on() && (line as usize) < HEIGHT - 1 {
                self.parse_sprites(i32::from(line));
            }
            if self.hcount == 0 {
                self.hcount = i16::from(self.regs[10]);
                self.hint_pending = true;
            } else {
                self.hcount -= 1;
            }
        } else if line == LINES - 1 {
            self.hcount = i16::from(self.regs[10]);
            if self.display_on() {
                self.parse_sprites(-1);
            }
        }
    }

    /// The interrupt raised: vertical (6) before horizontal (4), each if
    /// pending and enabled.
    #[must_use]
    pub fn irq_level(&self) -> u8 {
        if self.vint_pending && self.vint_enabled() {
            6
        } else if self.hint_pending && self.hint_enabled() {
            4
        } else {
            0
        }
    }

    /// The 68000 takes the interrupt: the vertical one if it is pending
    /// and enabled, else the horizontal one, is cleared.
    pub fn int_ack(&mut self) {
        if self.vint_pending && self.vint_enabled() {
            self.vint_pending = false;
        } else {
            self.hint_pending = false;
        }
    }

    /// A CRAM colour as RGB.  Each channel is a level of 16 (twice the
    /// CRAM's 3 bits; shadow and highlight take the ones between), made
    /// 8 bits as Genesis Plus GX's picture is (RGB565, widened), so the
    /// port's screens and the original's compare pixel for pixel.
    fn colour(&self, index: usize) -> u32 {
        let c = self.cram[index & 0x3F];
        let level = |v: u16| u32::from(v & 7) * 2;
        let five = |l: u32| ((l << 1) | (l >> 3)) * 255 / 31;
        let six = |l: u32| ((l << 2) | (l >> 2)) * 255 / 63;
        five(level(c >> 1)) << 16 | six(level(c >> 5)) << 8 | five(level(c >> 9))
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
    /// The picture's width: H40's 320 or H32's 256 as Genesis Plus GX
    /// takes a change of mode (in the vertical blank or the first three
    /// lines at once, else from the next frame).
    pub fn width(&self) -> usize {
        self.view_w
    }

    /// Register 12's H40 bit changed: when the picture's width follows.
    pub fn mode_switched(&mut self) {
        let w = if self.h40() { 320 } else { 256 };
        let line = usize::from(self.line);
        if line >= HEIGHT {
            self.view_w = w;
        } else if line <= 2 {
            self.view_w = w;
            for l in 0..=line {
                self.draw_line(l);
                if self.display_on() {
                    self.parse_sprites(l as i32);
                }
            }
        } else {
            self.view_pending = true;
        }
    }

    /// The display was turned on or off during a visible line (Genesis
    /// Plus GX's way): in its first 860 clocks (the blank before it), the
    /// line is drawn again as it now is (on after 360 clocks: 128 sprite
    /// pixels); later, from the pixel the beam was at, the rest is blank
    /// (off) or what came before was (on).
    pub fn display_switched(&mut self, into_line: u64) {
        let line = usize::from(self.line);
        if line >= HEIGHT {
            return;
        }
        let on = self.display_on();
        if into_line <= 860 {
            if on && into_line > 360 {
                self.sprite_pixels = 128;
            }
            self.draw_line(line);
            if on && line < HEIGHT - 1 {
                self.parse_sprites(line as i32);
            }
            self.sprite_pixels = 0;
            return;
        }
        let dot = if self.h40() { 8 } else { 10 };
        let x = ((into_line - 860) / dot + 16) as usize;
        let width = self.width();
        if x >= width {
            return;
        }
        let bg = self.colour(usize::from(self.regs[7] & 0x3F));
        if on {
            self.draw_line(line);
            if line < HEIGHT - 1 {
                self.parse_sprites(line as i32);
            }
            self.frame[line * WIDTH..line * WIDTH + x].fill(bg);
        } else {
            self.frame[line * WIDTH + x..line * WIDTH + width].fill(bg);
        }
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

    /// The sprites on the line after `line` (Genesis Plus GX's order):
    /// down the link list from the VDP's copy of the table (64 entries in
    /// H32, 80 in H40), the first 16 (20) on the line; overflow past
    /// them.
    fn parse_sprites(&mut self, line: i32) {
        // as many a line as the picture's width has 16-pixel columns; the
        // list read as far as the mode's entries, or the width's
        let max = self.view_w / 16;
        let entries = if self.h40() { 80 } else { 64 };
        let last = self.view_w / 4;
        let at = line + 0x81;
        let word = |m: &[u8], a: usize| u16::from_be_bytes([m[a], m[a + 1]]);
        let base = self.sat_base();
        let mut list = std::mem::take(&mut self.objs[((line + 1) & 1) as usize]);
        list.clear();
        let mut link = 0usize;
        let mut kept = None;
        for _ in 0..entries {
            let e = link * 8;
            let y = i32::from(word(&self.sat[..], e & 0x3FF) & 0x1FF);
            let sl = word(&self.sat[..], (e + 2) & 0x3FF);
            if at >= y {
                let size = sl >> 8;
                let height = 8 + (i32::from(size & 3) << 3);
                let row = at - y;
                if row < height {
                    if list.len() == max && kept.is_none() {
                        self.flags |= 0x40;
                        kept = Some(max);
                        if !self.unlimited {
                            break;
                        }
                    }
                    let v = |o: usize| word(&self.vram[..], (base + e + o) & 0xFFFE);
                    list.push(Obj { attr: v(4), x: v(6) & 0x1FF, row: row as u16, size: size & 0x0F });
                }
            }
            link = usize::from(sl & 0x7F);
            if link == 0 || link >= last {
                break;
            }
        }
        self.kept[((line + 1) & 1) as usize] = kept.unwrap_or(list.len());
        self.objs[((line + 1) & 1) as usize] = list;
    }

    /// The line's sprites, front to back: the first pixel down wins (two
    /// down: a collision).  A sprite at X 0 hides those after it once one
    /// further right has been seen (or the line before ran out of sprite
    /// pixels); a line has 256 (H32) or 320 (H40) sprite pixels, off
    /// screen ones too. Without the limit the sprites past it are drawn
    /// too (and the last one whole), but only the console's count for
    /// the flags and the masking.
    fn sprites(&mut self, line: usize, width: usize, out: &mut [(u8, bool)]) {
        let max_pixels = match self.sprite_pixels {
            0 => if self.h40() { 320 } else { 256 },
            n => n,
        };
        let list = std::mem::take(&mut self.objs[line & 1]);
        let kept = self.kept[line & 1];
        let mut pixels = 0;
        let mut masked = false;
        let mut done = false;
        // past the console's sprites: masking goes on as it would have
        let mut seen = self.spr_ovr;
        for (i, o) in list.iter().enumerate() {
            let console = !done && i < kept;
            if o.x != 0 {
                seen = true;
                if console {
                    self.spr_ovr = true;
                }
            } else if seen {
                masked = true;
            }
            let x0 = i32::from(o.x) - 0x80;
            let cw = usize::from((o.size >> 2) & 3) + 1;
            let ch = usize::from(o.size & 3) + 1;
            let mut w = cw * 8;
            if console {
                pixels += w;
            }
            if x0 + w as i32 > 0 && x0 < width as i32 && !masked {
                // the columns the console draws
                let mut shown = if console { w } else { 0 };
                if console && pixels > max_pixels {
                    shown -= pixels - max_pixels;
                    if !self.unlimited {
                        w = shown;
                    }
                }
                let row = usize::from(o.row);
                let row = if o.attr & 0x1000 != 0 { ch * 8 - 1 - row } else { row };
                for col in 0..w {
                    let px = x0 + col as i32;
                    if px < 0 || px as usize >= width {
                        continue;
                    }
                    let c = if o.attr & 0x0800 != 0 { cw * 8 - 1 - col } else { col };
                    let tile = usize::from(o.attr & 0x07FF) + (c / 8) * ch + row / 8;
                    let entry = (o.attr & 0xF800 & !0x1800) | (tile as u16 & 0x07FF);
                    let p = self.tile_pixel(entry, c & 7, row & 7);
                    if p == 0 {
                        continue;
                    }
                    let slot = &mut out[px as usize];
                    if slot.0 & 0x0F != 0 {
                        if col < shown {
                            self.flags |= 0x20;
                        }
                    } else {
                        *slot = ((((o.attr >> 13) & 3) as u8) << 4 | p, o.attr & 0x8000 != 0);
                    }
                }
            }
            if console && pixels >= max_pixels {
                // masking goes on to the next line if the whole width was
                self.spr_ovr = pixels >= width;
                done = true;
                if !self.unlimited {
                    break;
                }
            }
        }
        if !done {
            self.spr_ovr = false;
        }
        self.objs[line & 1] = list;
    }
}
