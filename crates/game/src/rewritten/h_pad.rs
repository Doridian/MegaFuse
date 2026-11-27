//! A pad port read nibble by nibble (0x8022-0x81C3): TR set or cleared
//! by turns, TL waited for to follow it (with a budget of tries), the
//! nibble read; and the records the nibbles make (`func_008114`).  C
//! set means a wait ran out.  No script calls these (none plugs in
//! what they read): reviewed by reading.

use crate::{Game, Regs};

/// The data port's TR (bit 5, written) and TL (bit 4, read).
const TR: u8 = 1 << 5;
const TL: u8 = 1 << 4;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// TR to `tr` on the data port `port`, then TL waited for to be
    /// `tl`: each try that finds it not costs one of `tries` (d7's low
    /// word), and when they run out (it goes to 0xFFFF) C is set.
    /// Else the port read again.  Gives back d0 (its low byte the last
    /// read), d7 and C.
    fn handshake(&mut self, d0: u32, tries: u32, port: u32, tr: u8, tl: u8) -> (u32, u32, bool) {
        self.write8(port, tr);
        let mut d0 = d0;
        let mut left = tries as u16;
        loop {
            let read = self.read8(port);
            d0 = low_byte(d0, read);
            if read & TL == tl {
                break;
            }
            left = left.wrapping_sub(1);
            if left == 0xFFFF {
                return (d0, low_word(tries, left), true);
            }
            self.pass(28);
        }
        let read = self.read8(port);
        (low_byte(d0, read), low_word(tries, left), false)
    }

    /// `func_00819A`: TR set, TL waited for to rise (`handshake`).
    pub fn r_00819a(&mut self, d0: u32, d7: u32, port: u32) -> (u32, u32, bool) {
        self.handshake(d0, d7, port, TR, TL)
    }

    /// `func_0081AE`: TR cleared, TL waited for to fall (`handshake`).
    pub fn r_0081ae(&mut self, d0: u32, d7: u32, port: u32) -> (u32, u32, bool) {
        self.handshake(d0, d7, port, 0, 0)
    }

    /// `func_0081C2`: C set (a wait ran out).
    pub fn r_0081c2(&mut self) -> bool {
        true
    }

    /// `func_008194`: the next nibble: d6's bit 0 flipped, TR set if it
    /// was clear (`func_00819A`), else cleared (`func_0081AE`).  Gives
    /// back d0 (the read), d6, d7 and C.
    pub fn r_008194(&mut self, d0: u32, d6: u32, d7: u32, port: u32) -> (u32, u32, u32, bool) {
        let d6 = d6 ^ 1;
        let (d0, d7, c) = if d6 & 1 == 0 { self.r_0081ae(d0, d7, port) } else { self.r_00819a(d0, d7, port) };
        (d0, d6, d7, c)
    }

    /// `func_008176`: two nibbles (`func_008194`), the first low, as a
    /// byte inverted (the pad's buttons are set when let go) into d1's
    /// low byte.  Gives back d0, d1, d6, d7 and C.
    pub fn r_008176(&mut self, d0: u32, d1: u32, d6: u32, d7: u32, port: u32) -> (u32, u32, u32, u32, bool) {
        let (d0, d6, d7, c) = self.r_008194(d0, d6, d7, port);
        if c {
            return (d0, d1, d6, d7, true);
        }
        let d0 = low_word(d0, d0 as u16 & 0xF);
        let d1 = low_word(d1, d0 as u16);
        let (d0, d6, d7, c) = self.r_008194(d0, d6, d7, port);
        if c {
            return (d0, d1, d6, d7, true);
        }
        let d0 = low_word(d0, (d0 as u16) << 4);
        let d1 = low_byte(d1, !(d1 as u8 | d0 as u8));
        (d0, d1, d6, d7, false)
    }

    /// `func_008022`: two nibbles (`func_008194`) onto d2: the first
    /// into its low byte, d2's low word shifted up 4, the second into
    /// its low byte.  Gives back d0, d2, d6, d7 and C.
    pub fn r_008022(&mut self, d0: u32, d2: u32, d6: u32, d7: u32, port: u32) -> (u32, u32, u32, u32, bool) {
        let (d0, d6, d7, c) = self.r_008194(d0, d6, d7, port);
        if c {
            return (d0, d2, d6, d7, true);
        }
        let d0 = low_byte(d0, d0 as u8 & 0xF);
        let d2 = low_byte(d2, d2 as u8 | d0 as u8);
        let (d0, d6, d7, c) = self.r_008194(d0, d6, d7, port);
        if c {
            return (d0, d2, d6, d7, true);
        }
        let d2 = low_word(d2, (d2 as u16) << 4);
        let d0 = low_word(d0, d0 as u16 & 0xF);
        let d2 = low_byte(d2, d2 as u8 | d0 as u8);
        (d0, d2, d6, d7, false)
    }

    /// `func_008114`: by the next byte at `kinds` (a2), a record at
    /// `to` (a1) read from the pad port: 0 a byte (`func_008126`), 1 a
    /// byte and a nibble (`func_00812A`), 2 `func_007FF4`'s; others
    /// skip the record (10 bytes, C clear).  Gives back d0 (the kind, or
    /// what the record's routine leaves), d1, d6, d7, a1, a2 and C.
    pub fn r_008114(&mut self, d1: u32, d6: u32, d7: u32, port: u32, to: u32, kinds: u32) -> (u32, u32, u32, u32, u32, u32, bool) {
        let kind = self.read8(kinds);
        let kinds = kinds.wrapping_add(1);
        let d0 = u32::from(kind);
        if kind > 2 {
            return (d0, d1, d6, d7, to.wrapping_add(10), kinds, false);
        }
        let d0 = d0 * 4;
        let (d0, d1, d6, d7, to, c) = match kind {
            0 => self.pad_byte(d0, d1, d6, d7, port, to),
            1 => self.pad_byte_nibble(d0, d1, d6, d7, port, to),
            _ => self.r_007ff4(d0, d1, d6, d7, port, to),
        };
        (d0, d1, d6, d7, to, kinds, c)
    }

    /// `func_008126`: `pad_byte` on the family's registers.
    pub fn r_008126(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.d0, r.d1, r.d6, r.d7, r.a1, r.c) = self.pad_byte(r.d0, r.d1, r.d6, r.d7, r.a0, r.a1);
    }

    /// `func_00812A`: `pad_byte_nibble` on the family's registers.
    pub fn r_00812a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.d0, r.d1, r.d6, r.d7, r.a1, r.c) = self.pad_byte_nibble(r.d0, r.d1, r.d6, r.d7, r.a0, r.a1);
    }

    /// `func_00812E`: `func_007FF4` on the family's registers.
    pub fn r_00812e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.d0, r.d1, r.d6, r.d7, r.a1, r.c) = self.r_007ff4(r.d0, r.d1, r.d6, r.d7, r.a0, r.a1);
    }

    /// A record of kind 0 (`func_008126`): a byte (`func_008176`); the
    /// word 0, then the byte through `func_007F4A`; 6 bytes skipped.
    fn pad_byte(&mut self, d0: u32, d1: u32, d6: u32, d7: u32, port: u32, to: u32) -> (u32, u32, u32, u32, u32, bool) {
        let (d0, d1, d6, d7, c) = self.r_008176(d0, d1, d6, d7, port);
        if c {
            return (d0, d1, d6, d7, to, true);
        }
        self.write16(to, 0);
        let d0 = low_byte(d0, d1 as u8);
        let (d1, to, c) = self.r_007f4a(d0, d1, to.wrapping_add(2));
        (d0, d1, d6, d7, to.wrapping_add(6), c)
    }

    /// A record of kind 1 (`func_00812A`): a byte (`func_008176`, its C
    /// not looked at) and a nibble, inverted; the word 1, then the byte
    /// and the nibble through `func_007F4A`; 4 bytes skipped.
    fn pad_byte_nibble(&mut self, d0: u32, d1: u32, d6: u32, d7: u32, port: u32, to: u32) -> (u32, u32, u32, u32, u32, bool) {
        let (d0, d1, d6, d7, _) = self.r_008176(d0, d1, d6, d7, port);
        let (d0, d6, d7, c) = self.r_008194(d0, d6, d7, port);
        if c {
            return (d0, d1, d6, d7, to, true);
        }
        let nibble = !(d0 as u8) & 0xF;
        let d0 = low_word(d0, u16::from(nibble));
        self.write16(to, 1);
        let d0 = low_byte(d0, d1 as u8);
        let (d1, to, _) = self.r_007f4a(d0, d1, to.wrapping_add(2));
        let d0 = low_byte(d0, nibble);
        let (d1, to, c) = self.r_007f4a(d0, d1, to);
        (d0, d1, d6, d7, to.wrapping_add(4), c)
    }
}
