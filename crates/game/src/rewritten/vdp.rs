//! The VDP: its registers as the game keeps them in RAM, the display
//! switched off, a VRAM fill by DMA, the backdrop colour.

use crate::Game;

/// The VDP's control port and data port.
const CONTROL: u32 = 0xC00004;
const DATA: u32 = 0xC00000;

/// Register 1 as the game keeps it (a control port word, `0x81xx`):
/// bit 6 of its low byte shows the picture, bit 4 lets DMA run.
const REG1: u32 = 0xFFF006;
/// Register 15 (the address step) as the game keeps it (`0x8Fxx`).
const REG15: u32 = 0xFFF022;
/// Register 1's low byte (where its bits are set and cleared).
const REG1_LOW: u32 = REG1 + 1;
const DISPLAY: u8 = 1 << 6;
const DMA_ON: u16 = 1 << 4;
/// The status bit that stays up while a DMA runs.
const DMA_BUSY: u16 = 1 << 1;

/// `func_000296`'s switch, a word in the ROM: 0, so the routine does
/// nothing.
const BACKDROP_SWITCH: u32 = 0x0002D8;
/// The colour `func_000296` writes for a negative argument.
const BACKDROP_DEFAULT: u32 = 0xFF5800;

impl Game {
    /// `func_008560`: register 1 as kept, to the VDP.
    pub fn r_008560(&mut self) {
        let reg1 = self.read16(REG1);
        self.write16(CONTROL, reg1);
    }

    /// `func_00855A`: the picture off (register 1's bit 6).
    pub fn r_00855a(&mut self) {
        let low = self.read8(REG1_LOW);
        self.write8(REG1_LOW, low & !DISPLAY);
        self.r_008560();
    }

    /// `func_0002E0`: the picture off (`func_00855A`).
    pub fn r_0002e0(&mut self) {
        self.r_00855a();
    }

    /// `func_00BAA8`: fills video memory by DMA with interrupts masked:
    /// `command` the VRAM write command of where it starts (as a long to
    /// the control port, the DMA bit added), `length` the DMA's length
    /// register, `value` the byte; waits for the fill to end.
    pub fn r_00baa8(&mut self, command: u32, length: u32, value: u32) {
        let ipl = self.ipl();
        self.set_ipl(7);
        // step 1, DMA on
        let reg15 = self.read16(REG15);
        self.write16(CONTROL, (reg15 & 0xFF00) | 1);
        let reg1 = self.read16(REG1);
        self.write16(CONTROL, reg1 | DMA_ON);
        // the length (registers 20 and 19), then the fill mode (23)
        let length = length as u16;
        self.write32(CONTROL, 0x9400_9300 | u32::from(length >> 8) << 16 | u32::from(length & 0xFF));
        self.write16(CONTROL, 0x9780);
        self.write32(CONTROL, command | 0x4000_0080);
        // the byte starts it
        self.write8(DATA, value as u8);
        while self.read16(CONTROL) & DMA_BUSY != 0 {
            self.pass(26);
        }
        let reg1 = self.read16(REG1);
        self.write16(CONTROL, reg1);
        let reg15 = self.read16(REG15);
        self.write16(CONTROL, reg15);
        self.set_ipl(ipl);
    }

    /// `func_0002E6`: VRAM from 0xE000 filled with 0 (a length of
    /// 0x1BFF).
    pub fn r_0002e6(&mut self) {
        self.r_00baa8(0x6000_0003, 0x1BFF, 0);
    }

    /// `func_000296`: the backdrop colour (CRAM 0), `colour` or, when
    /// it is negative, the word at 0xFF5800; only if the word at
    /// `BACKDROP_SWITCH` isn't 0 (it is). Gives back C (clear).
    pub fn r_000296(&mut self, colour: u16) -> bool {
        if self.read16(BACKDROP_SWITCH) == 0 {
            return false;
        }
        let ipl = self.ipl();
        self.set_ipl(7);
        self.write32(CONTROL, 0xC000_0000);
        let colour = if colour & 0x8000 != 0 { self.read16(BACKDROP_DEFAULT) } else { colour };
        self.write16(DATA, colour);
        self.set_ipl(ipl);
        false
    }
}

/// `func_00B9D0`'s DMA as `func_000302` sets it: the source (a long),
/// the VRAM address (a word) and the length in words (a word).
const DMA_FROM: u32 = 0xFFFF_A94E;
const DMA_TO: u32 = 0xFFFF_A952;
const DMA_LENGTH: u32 = 0xFFFF_A954;
/// What `func_000302` copies to each line of plane A: 0x40 words in the
/// ROM (`D_008776`).
const BLANK_LINE: u32 = 0x008776;
/// Plane A's first line in VRAM, a line's size and how many
/// `func_000302` blanks.
const PLANE_A: u16 = 0xE000;
const LINE: u16 = 0x80;
const PLANE_LINES: u16 = 0x38;

impl Game {
    /// `func_0002DA`: the picture on (`func_008552`).
    pub fn r_0002da(&mut self) {
        self.r_008552();
    }

    /// `func_000302`: plane A blanked: each of its `PLANE_LINES` lines
    /// (from `PLANE_A`, `LINE` bytes apart) a DMA (`func_00B9D0`) of
    /// 0x3F words of `BLANK_LINE`; then `func_000356`.  Gives back a0
    /// (the last DMA's source) and X (the carry of the last line's
    /// address step).
    pub fn r_000302(&mut self) -> (u32, bool) {
        self.write16(DMA_TO, PLANE_A);
        let mut a0 = 0;
        let mut x = false;
        for _ in 0..PLANE_LINES {
            self.write32(DMA_FROM, BLANK_LINE);
            self.write16(DMA_LENGTH, 0x3F);
            a0 = self.r_00b9d0();
            let to = self.read16(DMA_TO);
            let (next, carry) = to.overflowing_add(LINE);
            self.write16(DMA_TO, next);
            x = carry;
        }
        self.r_000356();
        (a0, x)
    }

    /// `func_00033A`: VRAM 0x0000 on filled with 0 (a length of 0x1F).
    pub fn r_00033a(&mut self) {
        self.r_00baa8(0x4000_0000, 0x1F, 0);
    }

    /// `func_000356`: VRAM 0xFDE0 on filled with 0 (a length of 0x1F).
    pub fn r_000356(&mut self) {
        self.r_00baa8(0x7DE0_0003, 0x1F, 0);
    }
}

/// A VRAM address as the control port's long has it: bits 0-13 in the
/// high word, bits 14-15 in the low word's bits 0-1.
fn vram_split(addr: u16) -> (u16, u16) {
    (addr & 0x3FFF, addr >> 14)
}

impl Game {
    /// `func_000268`: VRAM address `d0` (its low word) split: d0 = bits
    /// 14-15 in the high word, bits 0-13 in the low word.
    pub fn r_000268(&mut self, d0: u32) -> u32 {
        let (low, high) = vram_split(d0 as u16);
        u32::from(high) << 16 | u32::from(low)
    }

    /// `func_000282`: the VRAM write command for address `d0` (its low
    /// word).  Gives back C (clear).
    pub fn r_000282(&mut self, d0: u32) -> (u32, bool) {
        let (low, high) = vram_split(d0 as u16);
        (u32::from(low | 0x4000) << 16 | u32::from(high), false)
    }

    /// `func_00028E`: the VRAM read command for address `d0` (its low
    /// word).
    pub fn r_00028e(&mut self, d0: u32) -> u32 {
        let (low, high) = vram_split(d0 as u16);
        u32::from(low) << 16 | u32::from(high)
    }
}
