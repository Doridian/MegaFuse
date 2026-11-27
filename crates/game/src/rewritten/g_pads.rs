//! The pads' ports (0xA10003, 0xA10005: data; 6 bytes on, control):
//! set up (`func_007E08`) and read by the kind of pad found
//! (`func_007E92` picks a routine of the table at `loc_007EC8` by the
//! pad's ID, `func_007EE8`).  Each reader writes a word (what it found)
//! and then, for each byte of buttons read, the buttons held and those
//! newly pressed (`func_007F4A`).
//!
//! The readers here are the ones that read a 3- or 6-button pad
//! (`.L007EE0`) or nothing (the others).  `func_007E1E`, `func_007E92`,
//! `func_007EE8` and `func_007F04` read the ports in ways the check's
//! runs don't give back as the original's (the translation differs
//! there too), and are left translated.

use crate::{Game, Regs};

/// The ports' control registers (one byte each: the pads' 1 and 2, the
/// extension port), at 6 bytes from their data.
const CONTROL_1: u32 = 0xA10009;
const CONTROL_2: u32 = 0xA1000B;
const CONTROL_3: u32 = 0xA1000D;
const TO_CONTROL: u32 = 6;
/// The control value that makes the TH line (bit 6) an output.
const TH_OUT: u8 = 0x40;
/// The data written to set TH high and low.
const TH_HIGH: u8 = 0x40;
const TH_LOW: u8 = 0x00;

/// The word a reader writes first: a 3-button pad, a 6-button one,
/// nothing.
const FOUND_3: u16 = 0;
const FOUND_6: u16 = 1;
const FOUND_NONE: u16 = 0xF;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_007E08`: the three ports' TH lines made outputs.  Gives back
    /// d7 (`TH_OUT`).
    pub fn r_007e08(&mut self) -> u32 {
        for control in [CONTROL_1, CONTROL_2, CONTROL_3] {
            self.write8(control, TH_OUT);
        }
        u32::from(TH_OUT)
    }

    /// `loc_007EC8`: the reader for ID 0-1: nothing.  (No script runs
    /// it: reviewed by reading.)
    pub fn r_007ec8(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `.L007ECC`: the reader for ID 2-3: nothing.  (No script runs it:
    /// reviewed by reading.)
    pub fn r_007ecc(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `.L007ED0`: the reader for ID 4-5: nothing.  (No script runs it:
    /// reviewed by reading.)
    pub fn r_007ed0(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `.L007ED8`: the reader for ID 8-9: nothing.  (No script runs it:
    /// reviewed by reading.)
    pub fn r_007ed8(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `.L007EDC`: the reader for ID 0xA-0xB: nothing.  (No script runs
    /// it: reviewed by reading.)
    pub fn r_007edc(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `.L007EE0` (`.L007F56`), the reader for ID 0xC-0xD, a pad: TH
    /// pulsed until a read with TH low gives 0 in its low nibble (a
    /// 6-button pad: `FOUND_6`, the 8 buttons, then the 4 more of the
    /// next read with TH high), at most three times (a 3-button pad:
    /// `FOUND_3` and its 8 buttons, `func_007F26`).  d1 comes with the
    /// two reads of `func_007EE8` (TH high in the high word, TH low in
    /// the low).  Writes from a1; gives back d0.
    pub fn r_007ee0(&mut self, regs: &mut Regs) {
        let a0 = regs.a0;
        let mut d0: u32;
        let mut d1 = regs.d1;
        let mut a1 = regs.a1;
        self.write8(a0 + TO_CONTROL, TH_OUT);
        let mut tries = 3;
        loop {
            d0 = d1;
            if d1 & 0xF == 0 {
                break;
            }
            d1 = self.g_read_pair(a0);
            tries -= 1;
            if tries == 0 {
                self.write16(a1, FOUND_3);
                regs.d0 = self.r_007f26(d0, d1, a0, a1 + 2);
                regs.unwound = false;
                return;
            }
        }
        let more = self.g_read_pair(a0);
        self.write8(a0, TH_HIGH);
        self.write16(a1, FOUND_6);
        a1 += 2;
        d0 = low_word(d0, u16::from(g_buttons(d1)));
        (d1, a1, _) = self.r_007f4a(d0, d1, a1);
        let extra = !((more >> 16) as u8) & 0xF;
        d0 = low_word(d0, u16::from(extra));
        let _ = self.r_007f4a(d0, d1, a1);
        regs.d0 = d0;
        regs.unwound = false;
    }

    /// TH high then low, a read after each (the first in the high word).
    fn g_read_pair(&mut self, port: u32) -> u32 {
        self.write8(port, TH_HIGH);
        let high = self.read8(port);
        self.write8(port, TH_LOW);
        let low = self.read8(port);
        u32::from(high) << 16 | u32::from(low)
    }

    /// `.L007EE4`, the reader for ID 0xE-0xF: `FOUND_NONE`, then the
    /// buttons as a 3-button pad's (`func_007F26`).  Gives back d0.
    /// (No script runs it: reviewed by reading.)
    pub fn r_007ee4(&mut self, regs: &mut Regs) {
        self.write16(regs.a1, FOUND_NONE);
        regs.d0 = self.r_007f26(regs.d0, regs.d1, regs.a0, regs.a1 + 2);
        regs.unwound = false;
    }

    /// `.L007F26`: a 3-button pad's buttons from d1's two reads
    /// (`g_buttons`), TH left high, kept at a1 (`func_007F4A`).  Gives
    /// back d0 (its low word the buttons).
    pub fn r_007f26(&mut self, d0: u32, d1: u32, a0: u32, a1: u32) -> u32 {
        self.write8(a0 + TO_CONTROL, TH_OUT);
        self.write8(a0, TH_HIGH);
        let d0 = low_word(d0, u16::from(g_buttons(d1)));
        self.r_007f4a(d0, d1, a1);
        d0
    }

    /// `func_007F4A`: the buttons `d0` (low byte) kept at a1, those not
    /// held before (in the byte at a1) at a1 + 1.  Gives back d1 (its
    /// low byte those), a1 two bytes on and C (clear).
    pub fn r_007f4a(&mut self, d0: u32, d1: u32, a1: u32) -> (u32, u32, bool) {
        let now = d0 as u8;
        let before = self.read8(a1);
        self.write8(a1, now);
        let pressed = (before ^ now) & now;
        self.write8(a1 + 1, pressed);
        (low_byte(d1, pressed), a1 + 2, false)
    }
}

/// The 8 buttons of two reads (`d1`: TH high in the high word, TH low
/// in the low), 1 when held: the TH-low read's bits 4-5, then the
/// TH-high read's bits 0-5 (the ports give 0 for held).
fn g_buttons(d1: u32) -> u8 {
    let low = (d1 as u8) << 2 & 0xC0;
    let high = (d1 >> 16) as u8 & 0x3F;
    !(low | high)
}
