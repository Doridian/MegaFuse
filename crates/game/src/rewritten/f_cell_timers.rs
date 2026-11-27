//! Marks on the map's cells that play out over time: a mark (a byte,
//! 0x80 plus its kind) in `MARKS` and a byte in `STEPS` (bits 5-7 the
//! step, bits 0-4 the frames to the next one), both by the map's index,
//! and a list of the cells marked (`LIST`) that `func_0062C0` goes
//! through each time, drawing each mark's step (`func_005294`) and
//! clearing it after its last.

use crate::Game;

/// The marks and their steps, a byte a cell by the map's index
/// (`func_0016B0` keeps the cell's addresses in them: `STEPS_AT`).
const MARKS: u32 = 0xFFFF_0000;
const STEPS: u32 = 0xFFFF_01A0;
const STEPS_AT: u32 = 0xFF9EE2;
/// A step's field in `STEPS`, and its frames to go.
const STEP_MASK: u8 = 0xE0;
const STEP_SHIFT: u32 = 5;
const STEP_ONE: u8 = 0x20;
const FRAMES_MASK: u8 = 0x1F;
/// The kind of mark that is always started over.
const RESTARTED: u8 = 6;
const MARKED: u8 = 0x80;
/// Added to a step the mark is at again before its fourth
/// (D_0062B8: the steps go 0, 3, 6, 9 ...).
const AGAIN: u32 = 0x0062B8;

/// The cells marked: 8 bytes each, the map's index (a long), its
/// column and row; a long 0 after the last.
const LIST: u32 = 0xFFFF_0340;
const ENTRY_SIZE: u32 = 8;
const ENTRY_COLUMN: u32 = 4;
const ENTRY_ROW: u32 = 5;

/// The map's index of the cell at `CELL_X`, `CELL_Y` (a long).
const MAP_AT: u32 = 0xFF9ECA;
/// The map's width (in cells, a byte).
const WIDTH: u32 = 0xFF9EB2;
/// A cell (column, row).
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;

/// By kind (its low 6 bits), 8 bytes each: the word the steps are
/// drawn from (D_006336) and a table of what each step adds to it
/// (D_00633A, a long).  In those tables a byte with bit 7 set has the
/// cell's byte split into the cell map (`func_00168C`) first, and 0xFF
/// ends the mark.
const KINDS_DRAWN: u32 = 0x006336;
const KINDS_STEPS: u32 = 0x00633A;
const KIND_SIZE_SHIFT: u32 = 3;
/// The frames each step lasts, by step (D_00632E).
const FRAMES: u32 = 0x00632E;
const STEP_ADDS_MASK: u8 = 0x7F;
const STEP_END: u8 = 0xFF;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

fn word_index(w: u16) -> u32 {
    w as i16 as i32 as u32
}

impl Game {
    /// `func_00625C`: the cell at `MAP_AT` marked with kind `d0` (its
    /// low byte), from its first step; a cell already marked so (and
    /// not of kind `RESTARTED`) is put further on instead, before its
    /// fourth step.  A cell not marked before is put in `LIST`
    /// (`func_006386`).  Keeps every register.
    pub fn r_00625c(&mut self, d0: u32) {
        let kind = d0 as u8;
        let mark_at = self.r_0016b0();
        let steps_at = self.read32(STEPS_AT);
        let mark = kind | MARKED;
        if kind != RESTARTED && self.read8(mark_at) != 0 {
            if self.read8(mark_at) == mark {
                let steps = self.read8(steps_at) & STEP_MASK;
                let step = steps >> STEP_SHIFT;
                if step < 4 {
                    let again = self.read8(AGAIN + u32::from(step));
                    self.write8(steps_at, step.wrapping_add(again) | steps);
                    return;
                }
            }
        } else {
            self.r_006386();
        }
        self.write8(mark_at, mark);
        self.write8(steps_at, 0);
    }

    /// `func_0062C0`: each cell in `LIST`: one whose mark is gone is
    /// taken out (`func_0063D2`); one whose step has frames to go has
    /// one fewer; the others go to their next step (`func_0062C4`).
    /// Gives back d0 (0), d1, d2, d3, d4, d5, a0, a1, a2 (the end of
    /// the list), a3 (`MARKS`), X and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0062c0(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, a0: u32, a1: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (mut d1, mut d2, mut d3, mut d4, mut d5, mut a0, mut a1, mut x) = (d1, d2, d3, d4, d5, a0, a1, x);
        let mut entry = LIST;
        loop {
            let index = self.read32(entry);
            if index == 0 {
                return (0, d1, d2, d3, d4, d5, a0, a1, entry, MARKS, x, false);
            }
            if self.read8(MARKS.wrapping_add(word_index(index as u16))) == 0 {
                self.r_0063d2(entry);
                continue;
            }
            let column = self.read8(entry + ENTRY_COLUMN);
            let row = self.read8(entry + ENTRY_ROW);
            d2 = low_word(d2, u16::from(column));
            d1 = low_word(d1, u16::from(row));
            // the row's first cell
            let row_start = word_index((index as u16).wrapping_sub(u16::from(column)));
            a0 = MARKS.wrapping_add(row_start);
            a1 = STEPS.wrapping_add(row_start);
            let steps_at = a1.wrapping_add(u32::from(column));
            let steps = self.read8(steps_at);
            let frames = steps & FRAMES_MASK;
            if frames != 0 {
                self.write8(steps_at, steps - 1);
                x = false;
            } else {
                let d0 = low_byte(low_word(index, row_start as u16), frames);
                (d3, d4, d5, a0, x) = self.r_0062c4(d0, d1, d2, d3, d4, d5, a0, a1);
            }
            entry += ENTRY_SIZE;
        }
    }

    /// `func_0062C4`: the mark at column `d2`, row `d1` (a0, a1 the
    /// row's first cell in `MARKS` and `STEPS`) drawn at its step
    /// (`func_005294`) and put at the next, with its frames; past its
    /// last the mark is cleared (`func_0052F0`).  Gives back d3 (the
    /// word drawn), d4 (the new byte in `STEPS`), d5 (the step), a0
    /// and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0062c4(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, a0: u32, a1: u32) -> (u32, u32, u32, u32, bool) {
        let column = word_index(d2 as u16);
        self.write8(CELL_X, d2 as u8);
        self.write8(CELL_Y, d1 as u8);
        self.r_001278();
        let kind = u16::from(self.read8(a0.wrapping_add(column))) & 0x3F;
        let at = u32::from(kind << KIND_SIZE_SHIFT);
        let drawn = self.read16(KINDS_DRAWN + at);
        let step_adds = self.read32(KINDS_STEPS + at);
        let steps = self.read8(a1.wrapping_add(column)) & STEP_MASK;
        let step = steps >> STEP_SHIFT;
        let d5 = low_word(d5, u16::from(step));
        let mut adds = self.read8(step_adds.wrapping_add(u32::from(step)));
        let d0 = low_byte(low_word(d0, kind << KIND_SIZE_SHIFT), adds);
        if adds & 0x80 != 0 {
            if adds == STEP_END {
                self.write8(a0.wrapping_add(column), 0);
                let (x, _) = self.r_0052f0(d0);
                let d4 = low_byte(d4, steps);
                return (low_word(d3, drawn), d4, d5, a0, x);
            }
            adds &= STEP_ADDS_MASK;
            self.r_00167a();
            self.r_00168c(low_byte(d0, adds));
        }
        let d3 = low_byte(low_word(d3, drawn), (drawn as u8).wrapping_add(adds));
        let next = (steps | self.read8(FRAMES + u32::from(step))).wrapping_add(STEP_ONE);
        let d4 = low_byte(d4, next);
        self.write8(a1.wrapping_add(column), next);
        let d0 = low_word(d0, d3 as u16);
        let (x, _) = self.r_005294(d0, d1);
        (d3, d4, d5, a0, x)
    }

    /// `func_006386`: the cell at `MAP_AT` put in `LIST` (its column
    /// and row from the map's width), unless it is there already; a new
    /// one at the end has the list's end moved after it.  Keeps every
    /// register.
    pub fn r_006386(&mut self) {
        let index = self.read32(MAP_AT);
        let mut entry = LIST;
        loop {
            let there = self.read32(entry);
            if there == 0 || there == index {
                break;
            }
            entry += ENTRY_SIZE;
        }
        let was = self.read32(entry);
        self.write32(entry, index);
        let width = u16::from(self.read8(WIDTH));
        let (quotient, _) = self.divu(index, width);
        self.write8(entry + ENTRY_ROW, quotient as u8);
        self.write8(entry + ENTRY_COLUMN, (quotient >> 16) as u8);
        if was == 0 {
            self.write32(entry + ENTRY_SIZE, 0);
        }
    }

    /// `func_0063D2`: the entry of `LIST` at a2 taken out, the ones
    /// after moved down.  Keeps a2.
    pub fn r_0063d2(&mut self, a2: u32) {
        let mut at = a2;
        loop {
            for _ in 0..2 {
                let long = self.read32(at + ENTRY_SIZE);
                self.write32(at, long);
                at += 4;
            }
            if self.read32(at) == 0 {
                break;
            }
        }
    }
}
