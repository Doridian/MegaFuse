//! Runs of words made in the buffer at `BUFFER` and handed to
//! `func_00B9D0` (which sends `QUEUED_COUNT` words of the buffer to the
//! VRAM address `QUEUED_ADDRESS`), a run at a time: `RUNS` runs of
//! `RUN_LENGTH` words, each run's address `STEP` further on.  The words
//! are made by `func_00956C` (`func_0100AE`), are one word counted up
//! (`func_0101C0`), or are read back from the VRAM (`func_0107B6`).

use crate::Game;

/// Where the runs start (a VRAM address, a word; its low byte is the
/// column part) and the word `func_0101C2` writes (counted up in its
/// low byte; `func_0107B6` makes its high bits).
const START: u32 = 0xFF80C8;
const START_LOW: u32 = 0xFF80C9;
const WORD: u32 = 0xFF80D0;
const WORD_LOW: u32 = 0xFF80D1;

/// The run's length (bit 7 set on entry: `OR_FLAG`), the words left in
/// the run, the runs and the runs left.
const RUN_LENGTH: u32 = 0xFFACF4;
const RUN_LEFT: u32 = 0xFFACF5;
const RUNS: u32 = 0xFFACF6;
const RUNS_LEFT: u32 = 0xFFACF7;
/// 0xFF when the run's length had bit 7 set: `func_0100AE` ors the
/// word at `OR_WORD` (whose high byte is `WORD_LOW`) into each word.
const OR_FLAG: u32 = 0xFFACF8;
const OR_WORD: u32 = 0xFFACFA;
/// By `func_01018C`'s d0 (`SHAPES`): the mask of the run's address
/// kept from `START` (a word), the mask of `START_LOW` (a byte) and the
/// step between runs (a word).
const ADDRESS_MASK: u32 = 0xFFACFC;
const STEP: u32 = 0xFFACFE;
const COLUMN_MASK: u32 = 0xFFAD00;
const SHAPES: u32 = 0x0101A6;

/// The run's address: its row part (a word) and column part (a byte).
const ROW: u32 = 0xFF9E76;
const COLUMN: u32 = 0xFF9E74;
/// What `func_00B9D0` is handed: the buffer, the VRAM address and the
/// count of words.
const QUEUED_BUFFER: u32 = 0xFFA94E;
const QUEUED_ADDRESS: u32 = 0xFFA952;
const QUEUED_COUNT: u32 = 0xFFA954;
const BUFFER: u32 = 0x00FF_6000;

/// `func_0107B6`: the line to read back (one more than the line's
/// index), the place (`PLACE`) it is read from and the attribute bits
/// kept of the words read.
const LINE: u32 = 0xFFAD3B;
const PLACE: u32 = 0xFF9ECE;
const KEPT_BITS: u16 = 0x87FF;

/// The VDP's ports.
const DATA: u32 = 0xC00000;
const CONTROL: u32 = 0xC00004;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_0100AE`: runs of the words `func_00956C` makes (d1: the
    /// run's length in the low byte, the runs in the high byte; bit 7
    /// of the length: each word or'd with `OR_WORD`, whose high byte
    /// is `WORD_LOW`).  d1 and a6 come back as they were.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0100ae(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let (kept_d1, kept_a6) = (d1, a6);
        let mut d0 = low_word(d0, 0);
        self.r_01018c(d0);
        let (mut d1, mut x) = self.r_0101ae(d1);
        let word_low = self.read8(WORD_LOW);
        self.write8(OR_WORD, word_low);
        let mut a6;
        (d0, a6) = self.r_010136(d0);
        let or_word = self.read8(OR_FLAG) & 0x80 != 0;
        let (mut d3, mut d4, mut d5, mut d6, mut d7) = (d3, d4, d5, d6, d7);
        let mut a0: u32;
        let (mut a1, mut a2, mut a3, mut a4, mut a5) = (a1, a2, a3, a4, a5);
        loop {
            (d0, d1, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x) = self.r_00956c(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, x, false);
            if self.restarted.is_some() {
                return (d0, d1, d3, d4, d5, d6, d7, a1, a2, a3, a5, a6, x);
            }
            if or_word {
                d0 = low_word(d0, d0 as u16 | self.read16(OR_WORD));
            }
            self.write16(a6, d0 as u16);
            a6 = a6.wrapping_add(2);
            let done;
            (d0, _, a6, x, done) = self.r_010102(d0, a0, a6);
            if done {
                return (d0, kept_d1, d3, d4, d5, d6, d7, a1, a2, a3, a5, kept_a6, x);
            }
        }
    }

    /// `func_010102`, after each word: one word less in the run; at
    /// the run's end it is handed to `func_00B9D0` (`RUN_LENGTH` words),
    /// the address moves on by `STEP` and, if runs are left, the next
    /// run starts (`func_010156`).  C: the last run ended.  X: the
    /// borrow of the last count.
    pub fn r_010102(&mut self, d0: u32, a0: u32, a6: u32) -> (u32, u32, u32, bool, bool) {
        let left = self.read8(RUN_LEFT);
        self.write8(RUN_LEFT, left.wrapping_sub(1));
        if left != 1 {
            return (d0, a0, a6, left == 0, false);
        }
        let length = self.read8(RUN_LENGTH);
        self.write16(QUEUED_COUNT, u16::from(length));
        let a0 = self.r_00b9d0();
        let step = self.read16(STEP);
        let d0 = low_word(d0, step);
        let row = self.read16(ROW).wrapping_add(step);
        self.write16(ROW, row);
        let runs = self.read8(RUNS_LEFT);
        self.write8(RUNS_LEFT, runs.wrapping_sub(1));
        if runs == 1 {
            return (d0, a0, a6, false, true);
        }
        let (d0, a6, _) = self.r_010156(d0);
        (d0, a0, a6, runs == 0, false)
    }

    /// `func_010136`: the runs set up: the address from `START` (by
    /// `ADDRESS_MASK`), `OR_FLAG` from bit 7 of the length (cleared),
    /// the runs left, then the first run (`func_010156`).
    pub fn r_010136(&mut self, d0: u32) -> (u32, u32) {
        let row = self.read16(START) & self.read16(ADDRESS_MASK);
        self.write16(ROW, row);
        let length = self.read8(RUN_LENGTH);
        self.write8(OR_FLAG, if length & 0x80 != 0 { 0xFF } else { 0 });
        self.write8(RUN_LENGTH, length & 0x7F);
        let runs = self.read8(RUNS);
        self.write8(RUNS_LEFT, runs);
        let (d0, a6, _) = self.r_010156(low_word(d0, row));
        (d0, a6)
    }

    /// `func_010156`: a run started: its words left, its column part
    /// (`START_LOW` by `COLUMN_MASK`) and its address (`func_010174`).
    /// C clear.
    pub fn r_010156(&mut self, d0: u32) -> (u32, u32, bool) {
        let length = self.read8(RUN_LENGTH);
        self.write8(RUN_LEFT, length);
        let mask = self.read8(COLUMN_MASK);
        let column = self.read8(START_LOW) & mask;
        self.write8(COLUMN, column);
        let (d0, a6) = self.r_010174(low_byte(d0, mask));
        (d0, a6, false)
    }

    /// `func_010174`: the run's address (`ROW` or'd with `COLUMN`) to
    /// `QUEUED_ADDRESS`, and the buffer: a6 and `QUEUED_BUFFER`.
    pub fn r_010174(&mut self, d0: u32) -> (u32, u32) {
        let row = self.read16(ROW);
        let address = row | u16::from(self.read8(COLUMN));
        self.write16(QUEUED_ADDRESS, address);
        self.write32(QUEUED_BUFFER, BUFFER);
        (low_word(d0, address), BUFFER)
    }

    /// `func_01018C`: `ADDRESS_MASK`, `COLUMN_MASK` and `STEP` from
    /// `SHAPES` by d0 (0 or 4).
    pub fn r_01018c(&mut self, d0: u32) {
        let at = SHAPES.wrapping_add(d0 as u16 as i16 as u32);
        let mask = self.read16(at);
        self.write16(ADDRESS_MASK, mask);
        let column = self.read8(at + 2);
        self.write8(COLUMN_MASK, column);
        let step = self.read8(at + 3);
        self.write16(STEP, u16::from(step));
    }

    /// `func_0101AE`: d1's low byte the run's length, its high byte
    /// the runs.  Gives back d1's low word shifted down 8 (and X, its
    /// bit 7).
    pub fn r_0101ae(&mut self, d1: u32) -> (u32, bool) {
        self.write8(RUN_LENGTH, d1 as u8);
        let high = (d1 as u16) >> 8;
        self.write8(RUNS, high as u8);
        (low_word(d1, high), d1 & 0x80 != 0)
    }

    /// `func_0101BA`: `func_0101C2` with the shape 4.
    pub fn r_0101ba(&mut self, d1: u32, a0: u32, a6: u32) -> (u32, u32, u32, u32) {
        let (d0, d1, a0, a6, _) = self.r_0101c2(4, d1, a0, a6);
        (d0, d1, a0, a6)
    }

    /// `func_0101C0`: `func_0101C2` with the shape 0.
    pub fn r_0101c0(&mut self, d0: u32, d1: u32, a0: u32, a6: u32) -> (u32, u32, u32, u32, bool) {
        self.r_0101c2(low_word(d0, 0), d1, a0, a6)
    }

    /// `func_0101C2`: runs (d1, as `func_0100AE`'s) of the word at
    /// `WORD`, its low byte one more after each word; the shape by d0
    /// (`func_01018C`).  `ROW`, d1 and a6 come back as they were.
    pub fn r_0101c2(&mut self, d0: u32, d1: u32, a0: u32, a6: u32) -> (u32, u32, u32, u32, bool) {
        let (kept_d1, kept_a6) = (d1, a6);
        self.r_01018c(d0);
        self.r_0101ae(d1);
        let row = self.read16(ROW);
        let (mut d0, mut a6) = self.r_010136(d0);
        let mut a0 = a0;
        loop {
            let word = self.read16(WORD);
            self.write16(a6, word);
            a6 = a6.wrapping_add(2);
            let low = self.read8(WORD_LOW).wrapping_add(1);
            self.write8(WORD_LOW, low);
            let (x, done);
            (d0, a0, a6, x, done) = self.r_010102(d0, a0, a6);
            if done {
                self.write16(ROW, row);
                return (d0, kept_d1, a0, kept_a6, x);
            }
        }
    }

    /// `func_0107B6`: the line `LINE` - 1 (none if `LINE` is 0) read
    /// back from the VRAM: 12 runs of 0x1B words from `PLACE` (0x686
    /// on, as a VRAM address), each word's bits `KEPT_BITS` given the
    /// line's number in bits 13-15 (`WORD`), with the interrupts
    /// masked.  d1's low word comes back 0x0C1B; X as the address left
    /// it.
    pub fn r_0107b6(&mut self, d0: u32, d1: u32, a0: u32, x: bool) -> (u32, u32, u32, bool) {
        let d1 = low_word(d1, 0x0C1B);
        let line = self.read8(LINE);
        if line == 0 {
            return (low_byte(d0, 0), d1, a0, x);
        }
        self.write8(WORD_LOW, line.wrapping_sub(1));
        let place = self.read16(PLACE).wrapping_sub(0x1000).wrapping_mul(2);
        let (start, x) = place.overflowing_add(0x686);
        self.write16(START, start);
        let (d0, a0) = self.o_read_back(d1, a0);
        (d0, d1, a0, x)
    }

    /// `func_0107B6`'s reading (from 0x101F2): runs (d1) from `START`,
    /// each read from the VRAM by its address and written to the
    /// buffer with the bits of `WORD_LOW` << 13 (in `WORD` meanwhile).
    /// `ROW`, `WORD` and the interrupt mask come back as they were.
    fn o_read_back(&mut self, d1: u32, a0: u32) -> (u32, u32) {
        let row = self.read16(ROW);
        let word = self.read16(WORD);
        let ipl = self.ipl();
        self.set_ipl(ipl | 7);
        let bits = u16::from(self.read8(WORD_LOW)) << 13;
        self.write16(WORD, bits);
        self.r_01018c(0);
        self.r_0101ae(d1);
        let (mut d0, mut a6) = self.r_010136(0);
        let mut a0 = a0;
        'run: loop {
            let address = self.read16(QUEUED_ADDRESS);
            d0 = self.r_00028e(low_word(d0, address));
            self.write32(CONTROL, d0);
            loop {
                let read = (self.read16(DATA) & KEPT_BITS) | self.read16(WORD);
                self.write16(a6, read);
                a6 = a6.wrapping_add(2);
                let done;
                (d0, a0, a6, _, done) = self.r_010102(low_word(d0, read), a0, a6);
                if done {
                    break 'run;
                }
                if self.read8(RUN_LENGTH) == self.read8(RUN_LEFT) {
                    continue 'run;
                }
            }
        }
        self.set_ipl(ipl);
        self.write16(WORD, word);
        self.write16(ROW, row);
        (d0, a0)
    }
}
