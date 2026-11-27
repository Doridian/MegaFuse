//! A queue of copies to VRAM (`func_009790` adds, `func_0097CE` does
//! them with `func_00B9F6`), and the sprite records whose patterns are
//! put in it each frame (`func_0096CE`).

use crate::Game;

/// The queue: 32 entries of 8 bytes (a ring, offsets 0-0xF8) from
/// `QUEUE`; `HEAD` the offset of the first, `TAIL` of the one after
/// the last (words).
const QUEUE: u32 = 0xFFFF_9436;
const HEAD: u32 = 0xFFFF_9536;
const TAIL: u32 = 0xFFFF_9538;
/// An entry: the source (a long), the VRAM address and the length
/// (words), as `func_00B9F6` takes them.
const ENTRY_SOURCE: u32 = 0;
const ENTRY_TARGET: u32 = 4;
const ENTRY_LENGTH: u32 = 6;
const ENTRY_SIZE: u16 = 8;

/// The entry `func_009790` adds (`i_load.rs` sets them too).
const SOURCE: u32 = 0xFFFF_A94E;
const TARGET: u32 = 0xFFFF_A952;
const LENGTH: u32 = 0xFFFF_A954;
/// The sprite record's first pattern x 0x20 (its VRAM address), a
/// word `func_009716` keeps.
const BASE: u32 = 0xFFFF_9434;

/// The sprite records (`sprite.rs`): the list's head, and a record's
/// fields.
const HEAD_RECORD: u32 = 0xFFFF_8AB4;
/// A record's kind: 1 the list's end.
const REC_KIND: u32 = 0x00;
const END: u8 = 1;
const REC_FRAME: u32 = 0x04;
/// Bits 0-2 the record's set of frames (`func_000C86`), bit 3 set when
/// its patterns are to be copied (`sprite.rs`'s `PAID`).
const REC_FLAGS: u32 = 0x05;
const PATTERNS_DUE: u8 = 1 << 3;
/// The record's first pattern (a word).
const REC_PATTERN: u32 = 0x06;
const REC_NEXT: u32 = 0x10;

/// The backdrop's colour while `func_0096CE` runs, then the default.
const BACKDROP_BUSY: u16 = 0x0E00;
const BACKDROP_DEFAULT: u16 = 0xFFFF;

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_0096CE`: each sprite record after the head whose
    /// `PATTERNS_DUE` is set has its patterns queued (`func_009716`).
    /// The backdrop is `BACKDROP_BUSY` meanwhile.  Gives back d0, d1,
    /// d2 and d5 (1, the end's kind).
    pub fn r_0096ce(&mut self, d0: u32, d1: u32, d2: u32) -> (u32, u32, u32, u32) {
        self.r_000296(BACKDROP_BUSY);
        let (mut d0, mut d1, mut d2) = (d0, d1, d2);
        let mut record = HEAD_RECORD;
        while self.read8(record + REC_KIND) != END {
            record = self.read32(record + REC_NEXT);
            if self.read8(record + REC_FLAGS) & PATTERNS_DUE != 0 {
                (d0, d1, d2) = self.r_009716(d0, d1, d2, record);
            }
        }
        self.r_000296(BACKDROP_DEFAULT);
        (d0, d1, d2, u32::from(END))
    }

    /// `func_009716`: the patterns of the frame of sprite record `a5`
    /// queued (`func_009790`), `PATTERNS_DUE` cleared.  The frame's
    /// list (from its set of frames' second long) has after a word
    /// entries of a source (a long; its high word 0 ends them), a
    /// length (a byte, x 4) and a first pattern (a byte, from the
    /// record's `REC_PATTERN`; x 0x20, the VRAM address).  Gives back d0, d1 and d2.
    pub fn r_009716(&mut self, d0: u32, d1: u32, d2: u32, a5: u32) -> (u32, u32, u32) {
        let flags = self.read8(a5 + REC_FLAGS) & !PATTERNS_DUE;
        self.write8(a5 + REC_FLAGS, flags);
        let set = self.r_000c86(low_word(d0, u16::from(flags & 7)));
        let frame = u16::from(self.read8(a5 + REC_FRAME)) * 4;
        let mut d0 = low_word(d0, frame);
        let frames = self.read32(set + 4);
        let mut entry = self.read32(frames.wrapping_add(u32::from(frame))) + 2;
        let base = self.read16(a5 + REC_PATTERN) << 5;
        let d1 = low_word(d1, base);
        self.write16(BASE, base);
        let mut d2 = d2;
        loop {
            let source = self.read32(entry);
            self.write32(SOURCE, source);
            if source >> 16 == 0 {
                return (d0, d1, d2);
            }
            let length = u16::from(self.read8(entry + 4)) * 4;
            d2 = low_word(d2, length);
            self.write16(LENGTH, length);
            let target = (u16::from(self.read8(entry + 5)) << 5).wrapping_add(self.read16(BASE));
            d0 = low_word(d0, target);
            self.write16(TARGET, target);
            self.r_009790();
            entry += 6;
        }
    }

    /// `func_00977E`: the queue emptied (interrupts masked).
    pub fn r_00977e(&mut self) {
        let ipl = self.ipl();
        self.set_ipl(7);
        self.write16(HEAD, 0);
        self.write16(TAIL, 0);
        self.set_ipl(ipl);
    }

    /// `func_009790`: `SOURCE`, `TARGET`, `LENGTH` added to the queue,
    /// unless it is full.
    pub fn r_009790(&mut self) {
        let tail = self.read16(TAIL);
        let next = tail.wrapping_add(ENTRY_SIZE) & 0xFF;
        if next == self.read16(HEAD) {
            return;
        }
        let entry = QUEUE + u32::from(tail);
        let length = self.read16(LENGTH);
        self.write16(entry + ENTRY_LENGTH, length);
        let target = self.read16(TARGET);
        self.write16(entry + ENTRY_TARGET, target);
        let source = self.read32(SOURCE);
        self.write32(entry + ENTRY_SOURCE, source);
        self.write16(TAIL, next);
    }

    /// `func_0097CE`: the queue's copies done (`func_00B9F6`), first to
    /// last; `HEAD` set once they all are.
    pub fn r_0097ce(&mut self) {
        let mut at = self.read16(HEAD);
        while at != self.read16(TAIL) {
            let entry = QUEUE + u32::from(at);
            let length = self.read16(entry + ENTRY_LENGTH);
            let target = self.read16(entry + ENTRY_TARGET);
            let source = self.read32(entry + ENTRY_SOURCE);
            self.r_00b9f6(source, target, (length >> 8) as u8, length as u8);
            at = at.wrapping_add(ENTRY_SIZE) & 0xFF;
        }
        self.write16(HEAD, at);
    }
}
