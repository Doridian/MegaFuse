//! A walk out from the cell at `CELL_X`, `CELL_Y` in each of the four
//! directions (0 up, 1 right, 2 down, 3 left), up to `REACH` cells, the
//! cells met marked through `func_00625C` (`f_cell_timers.rs`) and
//! handed on by what is in them (`func_0060CC`, `func_006112`).

use crate::{Game, Regs};

/// Flags: while bit 3 is set, `func_0060CC` does nothing.
const FLAGS: u32 = 0xFF9540;
const FLAGS_NO_WALK: u8 = 1 << 3;
/// The cell at `CELL_AT`'s address in the cell map, kept by
/// `func_00167A` (a long; `a_cells.rs`).
const CELL_AT: u32 = 0xFF9ED6;
/// A cell (column, row; a word together).
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
/// The map's width (in cells, a byte).
const WIDTH: u32 = 0xFF9EB2;
/// The direction being walked (0 to 3), how many cells a walk goes,
/// and how many it has still to go.
const DIRECTION: u32 = 0xFFAABD;
const REACH: u32 = 0xFFAABE;
const TO_GO: u32 = 0xFFAABF;
/// Compared with 2 after a cell holding 2 is marked (`func_00D8BE`).
const MODE: u32 = 0xFF953A;

/// Steps by direction, column (D_00620E) and row (D_006212), signed
/// bytes.
const STEP_X: u32 = 0x00620E;
const STEP_Y: u32 = 0x006212;
/// The routines that move `CELL_AT` a cell in each direction (a long
/// each: `func_006238`, `func_006244`, `func_00624A`, `func_006256`).
const MOVES: u32 = 0x006228;

/// The command `func_0060CC` sends the Z80 (`func_0034B6`).
const WALK_COMMAND: u8 = 0x31;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_0060CC`: unless `FLAGS` says not to, the cell at `CELL_AT`
    /// marked with 6 (8 put in it first, if it wasn't empty), the Z80
    /// sent `WALK_COMMAND`, and the walk made in each direction
    /// (`func_006112`).  Gives back the registers the walks leave, X
    /// and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0060cc(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        if self.read8(FLAGS) & FLAGS_NO_WALK != 0 {
            return (d0, d1, d2, d5, d6, a0, a5, a6, x, c);
        }
        let mut d0 = d0;
        let cell = self.read32(CELL_AT);
        let held = self.read8(cell);
        d0 = low_byte(d0, held);
        if held != 0 {
            self.r_001984();
            d0 = low_byte(d0, 6);
            self.r_00625c(d0);
        }
        d0 = low_byte(d0, WALK_COMMAND);
        self.r_0034b6(d0);
        let (mut d1, mut d2, mut d5, mut d6, mut a0, mut a5, mut a6) = (d1, d2, d5, d6, a0, a5, a6);
        self.write8(DIRECTION, 0);
        for direction in 1..=4u8 {
            (d0, d1, d2, d5, d6, a0, a5, a6) = self.r_006112(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6);
            self.write8(DIRECTION, direction);
        }
        (d0, d1, d2, d5, d6, a0, a5, a6, false, false)
    }

    /// `func_006112`: the walk in direction `DIRECTION` from `CELL_X`,
    /// `CELL_Y` (put back after): each cell on until `REACH` of them,
    /// marked (`func_00625C`) with 7 in it (`func_00198E`), the mark
    /// the direction's axis plus 4, or the direction itself at the
    /// last.  It stops at a cell holding 0, 4 or 8; one holding 1 goes
    /// to `func_005E0E`, 2 to `func_0065B6` (and is marked with 3 in
    /// it, unless that says not to), 5 to `func_0066FE`.  A last cell
    /// holding 7 has the next one looked at too.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_006112(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        let (mut d0, mut d1, mut d2, mut d5, mut d6, mut a5, mut a6) = (d0, d1, d2, d5, d6, a5, a6);
        let mut a0;
        let value = self.read8(REACH);
        self.write8(TO_GO, value);
        let start = self.read16(CELL_X);
        loop {
            (d0, d1) = self.r_0061f6(d0, d1);
            self.r_00167a();
            a0 = self.read32(CELL_AT);
            let held = self.read8(a0);
            d0 = low_byte(d0, held);
            match held {
                0 | 4 | 8 => break,
                1 => {
                    self.r_005e0e(d2, d6, a6);
                    break;
                }
                2 => {
                    let stop;
                    (d0, d2, d5, d6, a5, a6, _, stop) = self.r_0065b6(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6);
                    if !stop {
                        a0 = self.r_001998();
                        d0 = low_byte(d0, 7);
                        self.r_00625c(d0);
                        if self.read8(MODE) == 2 {
                            (d0, d1, d2, d5, d6, a0, a5, a6) = self.r_00d8be(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
                        }
                    }
                    break;
                }
                5 => {
                    (d2, _) = self.r_0066fe(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
                    break;
                }
                _ => {}
            }
            let to_go = self.read8(TO_GO).wrapping_sub(1);
            self.write8(TO_GO, to_go);
            let along = (self.read8(DIRECTION) & 1) | 4;
            if to_go != 0 {
                self.r_00198e();
                d0 = low_byte(d0, along);
                self.r_00625c(d0);
                continue;
            }
            // the last cell
            a0 = self.read32(CELL_AT);
            let held = self.read8(a0);
            d0 = low_byte(d0, held);
            if held == 7 {
                d0 = self.r_006216(d0);
                a0 = self.read32(CELL_AT);
                let next = self.read8(a0);
                d0 = low_byte(d0, next);
                if next >= 7 {
                    a0 = self.r_00198e();
                    d0 = low_byte(d0, along);
                    self.r_00625c(d0);
                    break;
                }
            } else {
                a0 = self.r_00198e();
            }
            d0 = low_byte(d0, self.read8(DIRECTION));
            self.r_00625c(d0);
            break;
        }
        self.write16(CELL_X, start);
        (d0, d1, d2, d5, d6, a0, a5, a6)
    }

    /// `func_0061F6`: `CELL_X`, `CELL_Y` a step in direction
    /// `DIRECTION`.  Gives back d0 (the direction, a word) and d1 (the
    /// row's step).
    pub fn r_0061f6(&mut self, d0: u32, d1: u32) -> (u32, u32) {
        let direction = self.read8(DIRECTION) as i8 as i16;
        let d0 = low_word(d0, direction as u16);
        let at = i32::from(direction) as u32;
        let step_x = self.read8(STEP_X.wrapping_add(at));
        let value = self.read8(CELL_X).wrapping_add(step_x);
        self.write8(CELL_X, value);
        let step_y = self.read8(STEP_Y.wrapping_add(at));
        let value = self.read8(CELL_Y).wrapping_add(step_y);
        self.write8(CELL_Y, value);
        (d0, low_byte(d1, step_y))
    }

    /// `func_006216`: `CELL_AT` moved a cell in direction `DIRECTION`
    /// (through `MOVES`).  Gives back d0.
    pub fn r_006216(&mut self, d0: u32) -> u32 {
        let index = self.read8(DIRECTION).wrapping_mul(4) as i8 as i16;
        let d0 = low_word(d0, index as u16);
        let routine = self.read32(MOVES.wrapping_add(i32::from(index) as u32));
        let mut regs = Regs { d0, ..Regs::default() };
        self.call_family_9(routine, &mut regs);
        regs.d0
    }

    /// `CELL_AT` moved by `by`.
    fn move_cell_at(&mut self, by: i32) {
        let at = self.read32(CELL_AT);
        self.write32(CELL_AT, at.wrapping_add(by as u32));
    }

    /// `func_006238`: `CELL_AT` a row up; d0 the map's width.
    pub fn r_006238(&mut self, regs: &mut Regs) {
        let width = self.read8(WIDTH);
        regs.d0 = u32::from(width);
        self.move_cell_at(-i32::from(width));
        regs.unwound = false;
    }

    /// `func_006244`: `CELL_AT` a cell right.
    pub fn r_006244(&mut self, regs: &mut Regs) {
        self.move_cell_at(1);
        regs.unwound = false;
    }

    /// `func_00624A`: `CELL_AT` a row down; d0 the map's width.
    pub fn r_00624a(&mut self, regs: &mut Regs) {
        let width = self.read8(WIDTH);
        regs.d0 = u32::from(width);
        self.move_cell_at(i32::from(width));
        regs.unwound = false;
    }

    /// `func_006256`: `CELL_AT` a cell left.
    pub fn r_006256(&mut self, regs: &mut Regs) {
        self.move_cell_at(-1);
        regs.unwound = false;
    }
}
