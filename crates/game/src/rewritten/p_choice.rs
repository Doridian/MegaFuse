//! A choice counted 0-9 (`func_011294`, `func_0112AA`) and what it
//! picks (`func_0112BC`); two bytes taken from tables by the number
//! of records (`func_011A6E` on); and the records' sides: bit 6 of
//! their byte 0x2D, compared and counted (`func_0110CA`, `func_011F1C`,
//! `func_011FD4`).  The records are those `func_001CFA` looks up by
//! their index (`lookup.rs`).

use crate::Game;

/// The number of records `func_001CFA` gives that are counted (a byte).
const RECORDS: u32 = 0xFF9541;
/// Flags: bit 1 picks how the records are gone through (by side, or
/// one by one); bit 2 is set once by `func_011FD4`.
const FLAGS: u32 = 0xFF9540;
const BY_SIDE: u8 = 1 << 1;
const STARTED: u8 = 1 << 2;
/// The choice, 0-9 (a byte; `func_011294` counts it up, `func_0112AA`
/// down, both wrapping).
const CHOICE: u32 = 0xFFAD3F;
const CHOICES: u8 = 10;
/// What `func_0112BC` makes of the choice: the choice + 0x31, and the
/// byte `PICKS` has for it.
const CHOICE_CODE: u32 = 0xFF953F;
const CHOICE_PICK: u32 = 0xFF953A;
/// The bytes by choice (D_0112FE).
const PICKS: u32 = 0x0112FE;
/// Two bytes by a number (`func_011A72`), and their tables (the
/// first at func_011A82 + 8, inside D_011A92's run of bytes before it).
const BY_NUMBER_1: u32 = 0xFFAD3D;
const BY_NUMBER_2: u32 = 0xFFAD3E;
const TABLE_1: u32 = 0x011A8A;
const TABLE_2: u32 = 0x011A92;
/// The record `func_011F1C` places among those of its side (index,
/// address), and what it finds: its place, and how many there are.
const RECORD_INDEX: u32 = 0xFFA42C;
const RECORD_ADDR: u32 = 0xFFA42E;
const PLACE: u32 = 0xFFAC8A;
const COUNT: u32 = 0xFFAC8B;
/// The last record `func_011FD4` finds with bit 5 clear (its index).
const LAST_IN: u32 = 0xFFAD41;
/// A byte `func_011FD4` clears.
const CLEARED_80DF: u32 = 0xFF80DF;

/// A record's flags: bit 6 its side, bit 5 (set: not counted by
/// `func_011FD4`).
const REC_FLAGS: u32 = 0x2D;
/// The same flags of the record after it (records are 0x40 apart).
const NEXT_REC_FLAGS: u32 = 0x6D;
const SIDE: u8 = 1 << 6;
const OUT: u8 = 1 << 5;

/// What `func_011FD4` starts: a task (priority 0x10, routine
/// func_010BB6) with its count 0x80 (byte 0x16) and kept routine
/// func_01208E (0x1C); and the sound command it sends.
const START_PRIORITY: u8 = 0x10;
const START_ROUTINE: u32 = 0x010BB6;
const START_NEXT: u32 = 0x01208E;
const TASK_COUNT: u32 = 0x16;
const TASK_KEPT: u32 = 0x1C;
const START_SOUND: u8 = 0x2A;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_0110CA`: C set when every record from the first to the
    /// one before the last (`RECORDS` - 1 of them, at least one) is on
    /// the same side as the next one, else clear.  Gives back X and C.
    pub fn r_0110ca(&mut self, d4: u32) -> (bool, bool) {
        let last = self.read8(RECORDS).wrapping_sub(1);
        let mut d4 = low_word(d4, 0);
        loop {
            let (_, a4, x) = self.r_001cfa(d4);
            if self.read8(a4 + REC_FLAGS) & SIDE != self.read8(a4 + NEXT_REC_FLAGS) & SIDE {
                return (x, false);
            }
            let (index, x) = (d4 as u8).overflowing_add(1);
            d4 = low_byte(d4, index);
            if index == last {
                return (x, true);
            }
        }
    }

    /// `func_011294`: the choice counted up (9 to 0).  Gives back d0
    /// (its low byte the choice) and X.
    pub fn r_011294(&mut self, d0: u32) -> (u32, bool) {
        let (mut choice, x) = self.read8(CHOICE).overflowing_add(1);
        if choice == CHOICES {
            choice = 0;
        }
        self.write8(CHOICE, choice);
        (low_byte(d0, choice), x)
    }

    /// `func_0112AA`: the choice counted down (0 to 9; d0 then all 9).
    /// Gives back d0 and X.
    pub fn r_0112aa(&mut self, d0: u32) -> (u32, bool) {
        let (choice, x) = self.read8(CHOICE).overflowing_sub(1);
        let d0 = if (choice as i8) < 0 { u32::from(CHOICES - 1) } else { low_byte(d0, choice) };
        self.write8(CHOICE, d0 as u8);
        (d0, x)
    }

    /// `func_0112BC`: `CHOICE_CODE` and `CHOICE_PICK` by the choice.
    /// Gives back X (the carry of the choice + 0x31) and C (clear).
    pub fn r_0112bc(&mut self) -> (bool, bool) {
        let choice = self.read8(CHOICE);
        let (code, x) = choice.overflowing_add(0x31);
        self.write8(CHOICE_CODE, code);
        let pick = self.read8(PICKS.wrapping_add(choice as i8 as u32));
        self.write8(CHOICE_PICK, pick);
        (x, false)
    }

    /// `func_011A6E`: `func_011A72` by the number of records.
    pub fn r_011a6e(&mut self, d0: u32) -> (u32, bool) {
        let d0 = low_byte(d0, self.read8(RECORDS));
        self.r_011a72(d0)
    }

    /// `func_011A72`: `BY_NUMBER_1` and `BY_NUMBER_2` from their
    /// tables by d0's low byte (signed).  Gives back d0 (its low word
    /// that byte extended) and C (clear).
    pub fn r_011a72(&mut self, d0: u32) -> (u32, bool) {
        let index = d0 as u8 as i8;
        let d0 = low_word(d0, index as u16);
        let offset = i32::from(index) as u32;
        let first = self.read8(TABLE_1.wrapping_add(offset));
        self.write8(BY_NUMBER_1, first);
        let second = self.read8(TABLE_2.wrapping_add(offset));
        self.write8(BY_NUMBER_2, second);
        (d0, false)
    }

    /// `func_011A82`: both bytes of `func_011A72` cleared.  Gives back
    /// C (clear).
    pub fn r_011a82(&mut self) -> bool {
        self.write8(BY_NUMBER_2, 0);
        self.write8(BY_NUMBER_1, 0);
        false
    }

    /// `func_011F1C`: for the record at `RECORD_INDEX`, how many of the
    /// records are on its side (`COUNT`) and its place among them
    /// (`PLACE`; d1's low byte as it came if it isn't one of them).
    /// The registers are kept but d0 (which no caller reads).
    pub fn r_011f1c(&mut self, _d0: u32, d1: u32, _d4: u32) {
        let record = self.read32(RECORD_ADDR);
        let side = self.read8(record + REC_FLAGS);
        let mut place = d1 as u8;
        let mut count = 0u8;
        let mut index = 0u8;
        loop {
            let (_, a4, _) = self.r_001cfa(u32::from(index));
            if (self.read8(a4 + REC_FLAGS) ^ side) & SIDE == 0 {
                if index == self.read8(RECORD_INDEX) {
                    place = count;
                }
                count = count.wrapping_add(1);
            }
            index = index.wrapping_add(1);
            if index == self.read8(RECORDS) {
                break;
            }
        }
        self.write8(PLACE, place);
        self.write8(COUNT, count);
    }

    /// `func_011FD4`: whether to start: by side (`BY_SIDE`), when the
    /// records not out (bit 5 clear) are on both sides (`COUNT` those
    /// on side 0, `PLACE` those on side 1); else when two or more are
    /// not out (`LAST_IN` the last of them).  If not, and not started
    /// yet, it starts: `STARTED` set, the sound sent, a task made with
    /// its count and kept routine.  Gives back d0, d4, d6, a4 (the last
    /// record looked up), a6, X and C (set when too few are in, if
    /// started already).
    pub fn r_011fd4(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, a6: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let records = |game: &mut Self| game.read8(RECORDS);
        let mut d0 = d0;
        let mut d4 = low_word(d4, 0);
        let mut a4;
        let mut x;
        let c;
        if self.read8(FLAGS) & BY_SIDE != 0 {
            self.write8(COUNT, 0);
            self.write8(PLACE, 0);
            loop {
                (_, a4, _) = self.r_001cfa(d4);
                let flags = self.read8(a4 + REC_FLAGS);
                d0 = low_byte(d0, flags);
                if flags & OUT == 0 {
                    if flags & SIDE == 0 {
                        let count = self.read8(COUNT);
                        self.write8(COUNT, count.wrapping_add(1));
                    } else {
                        let place = self.read8(PLACE);
                        self.write8(PLACE, place.wrapping_add(1));
                    }
                }
                let index;
                (index, x) = (d4 as u8).overflowing_add(1);
                d4 = low_byte(d4, index);
                if index == records(self) {
                    break;
                }
            }
            d0 = low_byte(d0, self.read8(COUNT));
            c = false;
            if d0 as u8 != 0 {
                d0 = low_byte(d0, self.read8(PLACE));
                if d0 as u8 != 0 {
                    return (d0, d4, d6, a4, a6, x, c);
                }
            }
        } else {
            d0 = low_word(d0, 0);
            loop {
                (_, a4, _) = self.r_001cfa(d4);
                if self.read8(a4 + REC_FLAGS) & OUT == 0 {
                    d0 = low_byte(d0, (d0 as u8).wrapping_add(1));
                    self.write8(LAST_IN, d4 as u8);
                }
                let index;
                (index, x) = (d4 as u8).overflowing_add(1);
                d4 = low_byte(d4, index);
                if index == records(self) {
                    break;
                }
            }
            c = (d0 as u8) < 2;
            if !c {
                return (d0, d4, d6, a4, a6, x, c);
            }
        }
        let flags = self.read8(FLAGS);
        if flags & STARTED != 0 {
            return (d0, d4, d6, a4, a6, x, c);
        }
        self.write8(FLAGS, flags | STARTED);
        d0 = low_byte(d0, START_SOUND);
        self.r_0034b6(d0);
        self.write8(CLEARED_80DF, 0);
        let (d6, a6, x, _) = self.r_0003e8(d4, d5, d6, START_PRIORITY, START_ROUTINE);
        self.write8(a6 + TASK_COUNT, 0x80);
        self.write32(a6 + TASK_KEPT, START_NEXT);
        (d0, d4, d6, a4, a6, x, false)
    }
}
