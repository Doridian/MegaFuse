//! A list of lines with a routine each, set up for the VDP's
//! horizontal interrupt (`func_00AB12`, `func_00AB78`).  No script
//! calls either: they are checked by reading only.

use crate::Game;

/// The VDP's control port, and its H/V counter (the line in the high
/// byte).
const CONTROL: u32 = 0xC00004;
const HV_COUNTER: u32 = 0xC00008;
/// The bit of the status' high byte (read as a byte) that skips the
/// counter.
const STATUS_SKIP: u8 = 1 << 3;

/// Register 0 as the game keeps it (a control port word, `0x80xx`),
/// its low byte: bit 4 lets the horizontal interrupt in.
const REG0: u32 = 0xFFFF_F004;
const REG0_LOW: u32 = 0xFFFF_F005;
const HINT_ON: u8 = 1 << 4;
/// Register 10 as the game keeps it (`0x8Axx`), its low byte: the
/// lines between horizontal interrupts.
const REG10: u32 = 0xFFFF_F018;
const REG10_LOW: u32 = 0xFFFF_F019;

/// The list: entries of 6 bytes, a line (0xFF ends it), a byte not
/// read, a routine (a long).  `LIST` is where it is, `LIST_AT` a copy
/// `func_00AB12` reads it from, `LIST_OFFSET` the next entry's offset.
const LIST: u32 = 0xFFFF_A40A;
const LIST_AT: u32 = 0xFFFF_A406;
const LIST_OFFSET: u32 = 0xFFFF_A421;
const ENTRY_SIZE: u8 = 6;
const LIST_END: u8 = 0xFF;
/// The entry taken: its line and its routine (a long).
const LINE: u32 = 0xFFFF_80D9;
const ROUTINE: u32 = 0xFFFF_80DA;
/// The routine put there when the list is empty: an `rts`.
const NOTHING: u32 = 0x0093B8;

impl Game {
    /// `func_00AB12`: the horizontal interrupt on, and the list's first
    /// entry taken: its line in `LINE`, its routine in `ROUTINE`, the
    /// counter set for it (`func_00AB78`).  If the list is empty, the
    /// interrupt is off again and `ROUTINE` is `NOTHING`.
    pub fn r_00ab12(&mut self, d1: u32) {
        let reg0 = self.read8(REG0_LOW);
        self.write8(REG0_LOW, reg0 | HINT_ON);
        let reg0 = self.read16(REG0);
        self.write16(CONTROL, reg0);
        self.write8(LIST_OFFSET, 0);
        let list = self.read32(LIST);
        self.write32(LIST_AT, list);
        // the offset just cleared, as a signed word
        let offset = self.read8(LIST_OFFSET) as i8 as u32;
        let entry = self.read32(LIST_AT).wrapping_add(offset);
        let line = self.read8(entry);
        if line == LIST_END {
            let reg0 = self.read8(REG0_LOW);
            self.write8(REG0_LOW, reg0 & !HINT_ON);
            let reg0 = self.read16(REG0);
            self.write16(CONTROL, reg0);
            self.write32(ROUTINE, NOTHING);
            return;
        }
        self.write8(LINE, line);
        let routine = self.read32(entry + 2);
        self.write32(ROUTINE, routine);
        let next = self.read8(LIST_OFFSET);
        self.write8(LIST_OFFSET, next.wrapping_add(ENTRY_SIZE));
        self.r_00ab78(d1);
    }

    /// `func_00AB78`: the lines to the next horizontal interrupt
    /// (register 10): `LINE`, less the line the beam is on (the H/V
    /// counter) unless the status' `STATUS_SKIP` bit is set.
    pub fn r_00ab78(&mut self, _d1: u32) {
        let mut lines = self.read8(LINE);
        let status = self.read8(CONTROL);
        if status & STATUS_SKIP == 0 {
            let beam = (self.read16(HV_COUNTER) >> 8) as u8;
            lines = lines.wrapping_sub(beam);
        }
        self.write8(REG10_LOW, lines);
        let reg10 = self.read16(REG10);
        self.write16(CONTROL, reg10);
    }
}
