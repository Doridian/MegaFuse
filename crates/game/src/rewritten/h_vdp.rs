//! The VDP set up and cleared (0x841E-0x8604): its registers from a
//! list, video memory filled by DMA, the palette kept in RAM and sent
//! to CRAM by DMA, a rectangle of cells written to a plane; and a
//! palette that steps through a table of colours (`func_008286`).

use crate::Game;

/// The VDP's control port and data port.
const CONTROL: u32 = 0xC00004;
const DATA: u32 = 0xC00000;
/// The status bit that stays up while a DMA runs.
const DMA_BUSY: u16 = 1 << 1;

/// The registers as the game keeps them: register `n`'s control word
/// (`0x8nxx`) at `REGS + 2n`.
const REGS: u32 = 0xFFFF_F004;
/// Register 1 as kept: bit 6 of its low byte shows the picture, bit 4
/// lets DMA run.
const REG1: u32 = REGS + 2;
const REG1_LOW: u32 = REG1 + 1;
const DISPLAY: u8 = 1 << 6;
const DMA_ON: u16 = 1 << 4;
/// `func_00841E` keeps the registers up to 0x12 (0x92xx); later ones
/// only go to the VDP.
const LAST_KEPT: u8 = 0x92;
/// `func_00841E`'s list of control words (bytes in the ROM, a word
/// with bit 15 clear ends it).
const SETUP: u32 = 0x00844A;
/// Set to 0x80 by `func_00841E`.
const SETUP_80: u32 = 0xFFFF_8062;

/// The Z80's bus request (bit 8 written: the 68000 wants the bus; bit 0
/// read: set until the Z80 has let it go).
const BUSREQ: u32 = 0xA11100;

/// The palette as kept: 0x40 colours, sent to CRAM by `func_00858A`.
const PALETTE: u32 = 0x00FF_5800;
/// Bit 0: the palette was changed (`func_00856A`, `func_008286`), for
/// `func_00858A` to send.
const PALETTE_FLAGS: u32 = 0xFFFF_805B;
const PALETTE_CHANGED: u8 = 1 << 0;

/// `func_008286`'s colours: 11 from `CYCLE + CYCLE_AT` to the palette's
/// colours 2-12, every other call while `CYCLE_DELAY` runs out, with
/// `CYCLE_AT` going down by 2 to below 0.
const CYCLE: u32 = 0x0082BC;
const CYCLE_AT: u32 = 0xFFFF_805E;
/// Counted down each call; below 0, set to 1 and the colours step.
const CYCLE_DELAY: u32 = 0xFFFF_8060;
const CYCLE_COLOURS: u32 = 11;
const CYCLE_TO: u32 = PALETTE + 4;

/// Mode 1 and its DMA fill: register 15 (the address step) to 1, the
/// fill mode (23), and back to a step of 2.
const STEP_1: u16 = 0x8F01;
const STEP_2: u16 = 0x8F02;
const FILL_MODE: u16 = 0x9780;
/// A VRAM write's command with its DMA bit, as `or`ed into a fill's.
const DMA_WRITE: u32 = 0x4000_0080;

impl Game {
    /// `func_008508`: fills VRAM by DMA with 0 (interrupts as they
    /// are): `command` the VRAM write command of where it starts (the
    /// DMA bit added), `length` (its low word) the DMA's length; waits
    /// for it to end.  Gives back d0 (the command sent), d2 (0), d3
    /// (0x94 and the length's high byte over the last status read) and
    /// a6 (the control port).
    pub fn r_008508(&mut self, command: u32, length: u32) -> (u32, u32, u32, u32) {
        self.write16(CONTROL, STEP_1);
        let reg1 = self.read16(REG1);
        self.write16(CONTROL, reg1 | DMA_ON);
        let length = length as u16;
        let [high, low] = length.to_be_bytes();
        // registers 20 and 19
        let lengths = 0x9400_9300 | u32::from(high) << 16 | u32::from(low);
        self.write32(CONTROL, lengths);
        self.write16(CONTROL, FILL_MODE);
        let command = command | DMA_WRITE;
        self.write32(CONTROL, command);
        // the byte starts it
        self.write8(DATA, 0);
        let mut status;
        loop {
            status = self.read16(CONTROL);
            if status & DMA_BUSY == 0 {
                break;
            }
            self.pass(28);
        }
        let reg1 = self.read16(REG1);
        self.write16(CONTROL, reg1);
        self.write16(CONTROL, STEP_2);
        (command, 0, lengths & 0xFFFF_0000 | u32::from(status), CONTROL)
    }

    /// `func_008504`: `func_008508` with a length of 0x1FFF.
    pub fn r_008504(&mut self, command: u32) {
        self.r_008508(command, 0x1FFF);
    }

    /// `func_0084F6`: VRAM from 0xE000 filled with 0 (a length of
    /// 0x1FFF).
    pub fn r_0084f6(&mut self) {
        self.r_008504(0x6000_0003);
    }

    /// `func_0084FE`: the same as `func_0084F6`.
    pub fn r_0084fe(&mut self) {
        self.r_008504(0x6000_0003);
    }

    /// `func_0084CE`: VSRAM's 0x28 words set to 0.  Gives back d1
    /// (0xFFFF, the loop's end).
    pub fn r_0084ce(&mut self) -> u32 {
        self.write32(CONTROL, 0x4000_0010);
        for _ in 0..0x28 {
            self.write16(DATA, 0);
        }
        0xFFFF
    }

    /// `func_00846C`: CRAM's colour 0 set to 0, VSRAM cleared
    /// (`func_0084CE`), all of VRAM filled with 0 (`func_008508`, a
    /// length of 0xFFFF).  Gives back d1, d2, d3 and a6 as
    /// `func_008508` leaves them.
    pub fn r_00846c(&mut self) -> (u32, u32, u32, u32) {
        self.write32(CONTROL, 0xC000_0000);
        self.write16(DATA, 0);
        self.r_0084ce();
        let (_, d2, d3, a6) = self.r_008508(0x4000_0000, 0xFFFF);
        (0xFFFF, d2, d3, a6)
    }

    /// `func_00849E`: a long of 0 to VRAM 0xFC00.
    /// No script calls it: reviewed by reading.
    pub fn r_00849e(&mut self) {
        self.write32(CONTROL, 0x7C00_0003);
        self.write32(DATA, 0);
    }

    /// `func_0084B4`: the long at 0xFFD800 and a long of VRAM at 0xFE00
    /// set to 0.
    pub fn r_0084b4(&mut self) {
        self.write32(0xFFFF_D800, 0);
        self.write32(CONTROL, 0x7E00_0003);
        self.write32(DATA, 0);
    }

    /// `func_008490`: `func_0084B4`, VRAM from 0xE000 filled with 0
    /// twice (`func_0084F6`, `func_0084FE`) and again with a length of
    /// 0xDFF.  Gives back d0 (the fill's command), d1 (`d1` with 0xDFF
    /// in its low word) and d2 (0).
    pub fn r_008490(&mut self, d1: u32) -> (u32, u32, u32) {
        self.r_0084b4();
        self.r_0084f6();
        self.r_0084fe();
        let d1 = (d1 & 0xFFFF_0000) | 0xDFF;
        let (d0, d2, _, _) = self.r_008508(0x6000_0003, d1);
        (d0, d1, d2)
    }

    /// `func_00841E`: the registers from the list at `SETUP` to the VDP,
    /// those up to 0x12 kept at `REGS` too; `SETUP_80` set to 0x80.
    /// Gives back a2 (`REGS`).
    pub fn r_00841e(&mut self) -> u32 {
        self.write16(SETUP_80, 0x80);
        let mut at = SETUP;
        loop {
            let word = self.read16(at);
            let [register, _] = word.to_be_bytes();
            if register & 0x80 == 0 {
                return REGS;
            }
            at += 2;
            if register <= LAST_KEPT {
                // (0x8n doubled, as a byte: 2n)
                let offset = register.wrapping_add(register);
                self.write16(REGS + u32::from(offset), word);
            }
            self.write16(CONTROL, word);
        }
    }

    /// `func_008552`: the picture on (register 1's bit 6).
    pub fn r_008552(&mut self) {
        let low = self.read8(REG1_LOW);
        self.write8(REG1_LOW, low | DISPLAY);
        self.r_008560();
    }

    /// `func_00856A`: colours from the list at `list` into the palette
    /// (marked changed): a byte the offset into the palette, a byte the
    /// count less 1, the colours.  Gives back d0 (0xFFFF, the loop's
    /// end) and a1 (past the list).
    pub fn r_00856a(&mut self, list: u32) -> (u32, u32) {
        let flags = self.read8(PALETTE_FLAGS);
        self.write8(PALETTE_FLAGS, flags | PALETTE_CHANGED);
        let offset = self.read8(list);
        let count = self.read8(list + 1);
        let mut from = list + 2;
        let mut to = PALETTE + u32::from(offset);
        for _ in 0..=count {
            let colour = self.read16(from);
            self.write16(to, colour);
            from += 2;
            to += 2;
        }
        (0xFFFF, from)
    }

    /// `func_00858A`: if the palette was changed, the change cleared and
    /// the palette sent to CRAM by DMA (with interrupts masked and the
    /// Z80's bus held), then its first colour again by hand.
    pub fn r_00858a(&mut self, _d0: u32) {
        let flags = self.read8(PALETTE_FLAGS);
        self.write8(PALETTE_FLAGS, flags & !PALETTE_CHANGED);
        if flags & PALETTE_CHANGED == 0 {
            return;
        }
        let reg1 = self.read16(REG1);
        self.write16(CONTROL, reg1 | DMA_ON);
        let ipl = self.ipl();
        self.set_ipl(7);
        // the Z80's bus asked for (what to write back: the request
        // as it was, flipped)
        let busreq = self.read16(BUSREQ) ^ 0x100;
        self.write16(BUSREQ, 0x100);
        // 0x40 words from the palette (registers 20, 19, 22, 21, 23),
        // to CRAM 0: the command's second word starts it, once the Z80
        // has let the bus go
        self.write32(CONTROL, 0x9400_9340);
        self.write32(CONTROL, 0x96AC_9500);
        self.write16(CONTROL, 0x977F);
        self.write16(CONTROL, 0xC000);
        while self.read8(BUSREQ) & 1 != 0 {
            self.pass(30);
        }
        self.write16(CONTROL, 0x0080);
        while self.read16(CONTROL) & DMA_BUSY != 0 {
            self.pass(28);
        }
        self.write16(BUSREQ, busreq);
        let reg1 = self.read16(REG1);
        self.write16(CONTROL, reg1);
        self.write32(CONTROL, 0xC000_0000);
        let colour = self.read16(PALETTE);
        self.write16(DATA, colour);
        self.set_ipl(ipl);
    }

    /// `func_008604`: `rows` + 1 rows of `columns` + 1 words from
    /// `from` to the VDP, the first at the write command `command`, each
    /// next 0x80 bytes on.  Gives back d2 (`d2` with 0xFFFF in its low
    /// word), d3 (0x800000), d5 (the command after the last row), d6
    /// (`rows` with 0xFFFF in its low word), a5 (the data port) and a6
    /// (past the words).
    pub fn r_008604(&mut self, d2: u32, command: u32, rows: u32, columns: u32, from: u32) -> (u32, u32, u32, u32, u32, u32) {
        const ROW: u32 = 0x80_0000;
        let mut command = command;
        let mut from = from;
        for _ in 0..=(rows as u16) {
            self.write32(CONTROL, command);
            for _ in 0..=(columns as u16) {
                let word = self.read16(from);
                self.write16(DATA, word);
                from = from.wrapping_add(2);
            }
            command = command.wrapping_add(ROW);
        }
        (d2 | 0xFFFF, ROW, command, rows | 0xFFFF, DATA, from)
    }

    /// `func_008286`: the palette marked changed; `CYCLE_DELAY` counted
    /// down, and when it goes below 0, set to 1 and, while `CYCLE_AT`
    /// isn't below 0, the 11 colours at `CYCLE + CYCLE_AT` to the
    /// palette's colours 2-12 and `CYCLE_AT` down by 2.  Gives back d0,
    /// a0 and a1 (`CYCLE_AT` in d0's low word; once copied, d0 0xFFFF
    /// and a0, a1 past the colours).
    pub fn r_008286(&mut self, d0: u32, a0: u32, a1: u32) -> (u32, u32, u32) {
        let flags = self.read8(PALETTE_FLAGS);
        self.write8(PALETTE_FLAGS, flags | PALETTE_CHANGED);
        let delay = self.read8(CYCLE_DELAY).wrapping_sub(1);
        self.write8(CYCLE_DELAY, delay);
        if delay & 0x80 == 0 {
            return (d0, a0, a1);
        }
        self.write8(CYCLE_DELAY, 1);
        let at = self.read16(CYCLE_AT);
        let d0 = (d0 & 0xFFFF_0000) | u32::from(at);
        if at & 0x8000 != 0 {
            return (d0, a0, a1);
        }
        self.write16(CYCLE_AT, at.wrapping_sub(2));
        let mut from = CYCLE.wrapping_add(u32::from(at));
        let mut to = CYCLE_TO;
        for _ in 0..CYCLE_COLOURS {
            let colour = self.read16(from);
            self.write16(to, colour);
            from += 2;
            to += 2;
        }
        (0xFFFF, from, to)
    }
}
