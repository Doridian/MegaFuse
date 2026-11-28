//! Tiles cleared a pixel at a time (0x017828-0x01793A): `func_017828`
//! makes a task of `func_017870`, which each frame clears one pixel (a
//! nibble, `func_0178CE` picks it from the order D_0178FA) in each row
//! of four of a run of tiles in VRAM, 64 frames.

use crate::{Game, Regs};

/// The task's fields: the frames left (a byte, it ends below 0), the
/// VRAM address of its first tile (a word) and the number of tiles (a
/// byte).
const LEFT: u32 = 0x0C;
const FIRST: u32 = 0x0E;
const TILES: u32 = 0x10;
/// The frames: 0x3F down to 0.
const FRAMES: u8 = 0x3F;

/// Longs set to -1 by `func_017828`; the first one's word is the frame
/// `func_0178CE` was last given.
const LAST_FRAME: u32 = 0xFFFF_ADC6;
const LONG_ADCA: u32 = 0xFFFF_ADCA;
/// The tile `func_017828` starts at (a word): its VRAM address is
/// twice it less 0x2000.
const START: u32 = 0xFFFF_80C4;
const START_LESS: u16 = 0x1000;
/// The pixel cleared on each frame (D_0178FA, 64 bytes): bits 0-1 the
/// nibble of the word, bits 3-7 the word of the row (bytes, over 2).
const ORDER: u32 = 0x0178FA;
/// The task's routine and priority.
const ROUTINE_017870: u32 = 0x017870;
const PRIORITY: u8 = 0x32;
/// The VDP's ports.
const VDP_DATA: u32 = 0xC0_0000;
const VDP_CONTROL: u32 = 0xC0_0004;
/// A VRAM read command made a write command, the next row of a tile,
/// the rows of a tile cleared (less one, `dbra`'s).
const WRITE: u32 = 1 << 30;
const ROW: u16 = 0x20;
const ROWS: u16 = 3;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_017828`: `LAST_FRAME` and `LONG_ADCA` -1, and a task of
    /// `func_017870` (`func_0003E8`) for `d0` (its low byte) tiles from
    /// the tile `START`.  d0, d1, d6, a0 and a6 are kept.  Gives back X
    /// (the doubling's carry) and C (clear).
    pub fn r_017828(&mut self, d0: u32, d4: u32, d5: u32, d6: u32) -> (bool, bool) {
        self.write32(LAST_FRAME, 0xFFFF_FFFF);
        self.write32(LONG_ADCA, 0xFFFF_FFFF);
        let (_, task, _, _) = self.r_0003e8(d4, d5, d6, PRIORITY, ROUTINE_017870);
        self.write8(task + LEFT, FRAMES);
        let start = self.read16(START).wrapping_sub(START_LESS);
        let (first, carry) = start.overflowing_add(start);
        self.write16(task + FIRST, first);
        self.write8(task + TILES, d0 as u8);
        (carry, false)
    }

    /// `func_017870`, a task's routine: until `LEFT` goes below 0 (then
    /// it ends), the pixel of frame `LEFT` (`func_0178CE`) cleared in
    /// each row of each tile, read back from VRAM and written with the
    /// interrupts masked.
    pub fn r_017870(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let left = self.read8(r.a6 + LEFT);
        r.d0 = low_byte(r.d0, left);
        if left & 0x80 != 0 {
            r.c = false;
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
            return;
        }
        (r.d0, r.d1) = self.r_0178ce(r.d0, r.d1);
        let keep = r.d0 as u16;
        r.d3 = low_word(r.d3, keep);
        let mut address = u16::from(r.d1 as u8).wrapping_add(self.read16(r.a6 + FIRST));
        self.write8(r.a6 + LEFT, left.wrapping_sub(1));
        let borrow = left == 0;
        let ipl = self.ipl();
        self.set_ipl(ipl | 7);
        let mut tiles = u16::from(self.read8(r.a6 + TILES)).wrapping_sub(1);
        loop {
            for _ in 0..=ROWS {
                let read = self.r_00028e(low_word(r.d0, address));
                self.write32(VDP_CONTROL, read);
                let word = self.read16(VDP_DATA) & keep;
                r.d4 = low_word(r.d4, word);
                r.d0 = read | WRITE;
                self.write32(VDP_CONTROL, r.d0);
                self.write16(VDP_DATA, word);
                address = address.wrapping_add(ROW);
            }
            tiles = tiles.wrapping_sub(1);
            if tiles == 0xFFFF {
                break;
            }
        }
        r.d1 = 0xFFFF;
        r.d2 = low_word(r.d2, tiles);
        r.d5 = low_word(r.d5, address);
        self.set_ipl(ipl);
        // the status saved after the count's subq
        r.x = borrow;
        r.c = borrow;
    }

    /// `func_0178CE`: frame `d0` (its low byte, its word kept in
    /// `LAST_FRAME`) in `ORDER`: gives back d0 (its low word the mask
    /// that keeps all but the nibble) and d1 (its low byte the byte of
    /// the row).  d2-d5 and a0 are kept.
    pub fn r_0178ce(&mut self, d0: u32, d1: u32) -> (u32, u32) {
        self.write16(LAST_FRAME, d0 as u16);
        let pixel = self.read8(ORDER + (d0 & 0xFF));
        let d1 = low_byte(d1, (pixel >> 1) & 0xFE);
        let mask = 0xFFF0u16.rotate_left(u32::from(pixel & 3) * 4);
        (low_word(d0, mask), d1)
    }
}
