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
