//! Two decoders, each giving the next byte of data packed in the ROM
//! (the one in use is the routine at 0xFF9E84, `func_000E98`; the
//! stage's map is read through it, `func_000E9E`).  `SOURCE` is where
//! the packed data is.
//!
//! - An LZ decoder (`func_000CBA` starts it, `func_000CF4` a byte): a
//!   bit stream of a flag then either a literal byte (8 bits) or a copy
//!   (an 8-bit position in the window of the last 256 bytes, a 4-bit
//!   length less 2).
//! - A run decoder (`func_000D7A` starts it, `func_000D8E` a byte):
//!   blocks of 16 bytes, the even ones then the odd ones, each half a
//!   byte of flags then the bytes, a flag set repeating the last byte
//!   of the half rather than reading one.

use crate::{Game, Regs};

/// The packed data's address (a long).
const SOURCE: u32 = 0xFF930A;

/// The LZ decoder.
/// The next byte of the bit stream (a long).
const BIT_SOURCE: u32 = 0xFF930E;
/// The window's write position: a word, the position its low byte.
const WRITE: u32 = 0xFF9312;
const WRITE_LOW: u32 = WRITE + 1;
/// Bytes of a copy still to go.
const RUN: u32 = 0xFF9314;
/// A copy's position: a word, the position its low byte.
const READ: u32 = 0xFF9316;
const READ_LOW: u32 = READ + 1;
/// The bits of the stream not yet read, and how many until the next
/// byte is read.
const BITS: u32 = 0xFF9318;
const BITS_LEFT: u32 = 0xFF9319;
/// The last 256 bytes (the window), written from 0xEF on.
const WINDOW: u32 = 0xFFFF931A;
const WINDOW_LONGS: u32 = 0x40;
const WINDOW_START: u16 = 0xEF;

/// The run decoder.
/// Where it is in the packed data (from `SOURCE`): a word, the low
/// byte counting (`SOURCE` moves on 0x100 when it wraps).
const RUN_SOURCE: u32 = 0xFF942A;
const RUN_SOURCE_LOW: u32 = RUN_SOURCE + 1;
/// The next byte of the block: a word, 0x10 in the low byte when the
/// block is used up.
const BLOCK_POS: u32 = 0xFF942C;
const BLOCK_POS_LOW: u32 = BLOCK_POS + 1;
/// The flags of the half being decoded (the word's high byte, shifted
/// out from the top).
const FLAGS: u32 = 0xFF942E;
/// The last byte of each half.
const LAST_EVEN: u32 = 0xFF9430;
const LAST_ODD: u32 = 0xFF9431;
/// The block.
const BLOCK: u32 = 0xFFFF941A;
const BLOCK_LEN: u8 = 0x10;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_000CBA`: the LZ decoder starts on the data at `SOURCE`
    /// (its first byte the first bits), the window clear.
    pub fn r_000cba(&mut self) {
        for k in 0..WINDOW_LONGS {
            self.write32(WINDOW + 4 * k, 0);
        }
        self.write8(RUN, 0);
        self.write16(WRITE, WINDOW_START);
        let source = self.read32(SOURCE);
        let first = self.read8(source);
        self.write8(BITS, first);
        self.write32(BIT_SOURCE, source.wrapping_add(1));
        self.write8(BITS_LEFT, 8);
        self.write16(READ, 0);
    }

    /// The next `count` bits of the LZ stream, the first the highest.
    fn lz_bits(&mut self, count: u16) -> u16 {
        let mut w = u16::from(self.read8(BITS));
        for _ in 0..=count.wrapping_sub(1) {
            w = w.wrapping_shl(1);
            let left = self.read8(BITS_LEFT).wrapping_sub(1);
            self.write8(BITS_LEFT, left);
            if left == 0 {
                let at = self.read32(BIT_SOURCE);
                let next = self.read8(at);
                self.write32(BIT_SOURCE, at.wrapping_add(1));
                w = (w & 0xFF00) | u16::from(next);
                self.write8(BITS_LEFT, 8);
            }
        }
        self.write8(BITS, w as u8);
        w >> 8
    }

    /// `func_000D4C`: the next `d1` + 1 bits (d1's low word) to d0's
    /// low word; d1's low word left 0xFFFF.
    pub fn r_000d4c(&mut self, d0: u32, d1: u32) -> (u32, u32) {
        let bits = self.lz_bits((d1 as u16).wrapping_add(1));
        (low_word(d0, bits), low_word(d1, 0xFFFF))
    }

    /// `func_000CF4`: the LZ decoder's next byte, to d0's low byte (and
    /// to the window).  Leaves d1 0xFFFF if it read bits, a0 the
    /// window's write position (sign-extended), X and C its carry.
    pub fn r_000cf4(&mut self, regs: &mut Regs) {
        let byte = if self.read8(RUN) == 0 {
            regs.d1 = 0xFFFF;
            let literal = self.lz_bits(1) != 0;
            if literal {
                let byte = self.lz_bits(8) as u8;
                regs.d0 = low_word(regs.d0, u16::from(byte));
                Some(byte)
            } else {
                let position = self.lz_bits(8) as u8;
                self.write8(READ_LOW, position);
                let length = (self.lz_bits(4) as u8).wrapping_add(2);
                regs.d0 = low_word(regs.d0, u16::from(length));
                self.write8(RUN, length);
                None
            }
        } else {
            None
        };
        let byte = match byte {
            Some(byte) => byte,
            None => {
                // the copy goes on
                let read = self.read16(READ);
                let byte = self.read8(WINDOW.wrapping_add(read as i16 as u32));
                regs.d0 = low_byte(regs.d0, byte);
                let low = self.read8(READ_LOW);
                self.write8(READ_LOW, low.wrapping_add(1));
                let run = self.read8(RUN);
                self.write8(RUN, run.wrapping_sub(1));
                byte
            }
        };
        let write = self.read16(WRITE);
        regs.a0 = write as i16 as u32;
        self.write8(WINDOW.wrapping_add(write as i16 as u32), byte);
        let low = self.read8(WRITE_LOW);
        self.write8(WRITE_LOW, low.wrapping_add(1));
        regs.x = low == 0xFF;
        regs.c = regs.x;
    }

    /// `func_000D7A`: the run decoder starts at the data at `SOURCE`
    /// (the block used up: the next byte starts one).
    pub fn r_000d7a(&mut self) {
        self.write16(RUN_SOURCE, 0);
        self.write16(BLOCK_POS, BLOCK_LEN.into());
        self.write8(LAST_EVEN, 0);
        self.write8(LAST_ODD, 0);
    }

    /// `func_000E26`: the packed byte `d1` (its low word) on from
    /// `SOURCE`, to d0's low byte; d1's low byte one more (`SOURCE`
    /// 0x100 on when it wraps).
    pub fn r_000e26(&mut self, d0: u32, d1: u32) -> (u32, u32) {
        let source = self.read32(SOURCE);
        let byte = self.read8(source.wrapping_add(d1 as u16 as i16 as u32));
        let next = (d1 as u8).wrapping_add(1);
        if next == 0 {
            self.write32(SOURCE, source.wrapping_add(0x100));
        }
        (low_byte(d0, byte), low_byte(d1, next))
    }

    /// Half of a block: `first` (0 the even bytes, 1 the odd ones) on,
    /// from a byte of flags; `last` keeps the half's last byte.  `d0`
    /// and `d1` are the registers `func_000E26` works in.
    fn run_half(&mut self, d0: &mut u32, d1: &mut u32, first: u8, last: u32) {
        (*d0, *d1) = self.r_000e26(*d0, *d1);
        self.write8(FLAGS, *d0 as u8);
        *d0 = low_byte(*d0, self.read8(last));
        let mut at = first;
        while at != BLOCK_LEN + first {
            let flags = self.read16(FLAGS);
            self.write16(FLAGS, flags << 1);
            if flags & 0x8000 == 0 {
                (*d0, *d1) = self.r_000e26(*d0, *d1);
            }
            self.write8(BLOCK + u32::from(at), *d0 as u8);
            self.write8(last, *d0 as u8);
            at += 2;
        }
    }

    /// `func_000D8E`: the run decoder's next byte, to d0's low byte (a
    /// new block decoded first if this one is used up).  X and C the
    /// position's carry; the other registers are kept.
    pub fn r_000d8e(&mut self, regs: &mut Regs) {
        if self.read8(BLOCK_POS_LOW) == BLOCK_LEN {
            let mut d1 = u32::from(self.read16(RUN_SOURCE));
            self.run_half(&mut regs.d0, &mut d1, 0, LAST_EVEN);
            self.run_half(&mut regs.d0, &mut d1, 1, LAST_ODD);
            self.write8(RUN_SOURCE_LOW, d1 as u8);
            self.write16(BLOCK_POS, 0);
        }
        let pos = self.read16(BLOCK_POS);
        let byte = self.read8(BLOCK.wrapping_add(pos as i16 as u32));
        regs.d0 = low_byte(regs.d0, byte);
        self.write16(BLOCK_POS, pos.wrapping_add(1));
        regs.x = pos == 0xFFFF;
        regs.c = regs.x;
    }
}
