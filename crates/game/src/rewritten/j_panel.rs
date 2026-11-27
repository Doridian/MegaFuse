//! What is drawn in plane A's rows from 0xE000 (a tile at a time,
//! `func_00ADCA`, or blocks of them, `func_0101BA`) and in sprites
//! beside it: a clock (`CLOCK_TENTHS`, `CLOCK_SECONDS`,
//! `CLOCK_MINUTES`, counted down by `func_00ACA4`, drawn by
//! `func_00AD66`; a large one in sprites when its minutes run out,
//! `func_00ADF2`, `func_00AE6E`), a sum and a count (`func_00AC64`,
//! drawn by `func_00AF26`), and two helpers that place a sprite record
//! by a task's position (`func_00A030`, `func_00AEFA`).

use crate::{Game, Regs};

/// The VDP's control port and data port.
const CONTROL: u32 = 0xC00004;
const DATA: u32 = 0xC00000;

/// Flags of the game (`b_control.rs`).  Bit 0 a mode: the clock has
/// tenths, starts at `MODE_0_MINUTES`, and `func_00AF26` draws the
/// objects' marks instead of the sum.  Bit 1: `func_00AF26` compares
/// an object with the next.  Bit 2: set when the clock runs out.  Bit
/// 4: the large clock sounds.  Bit 7: no clock from the tables, no
/// large clock and no task from `func_00AF26`.  `CLOCK_HELD`: either
/// bit stops the clock; `NO_END_CALL`: either bit and `func_00B346`
/// isn't called when the clock runs out.
const MODE: u32 = 0xFFFF_9540;
const MODE_0: u8 = 1 << 0;
const MODE_COMPARE: u8 = 1 << 1;
const MODE_RAN_OUT: u8 = 1 << 2;
const MODE_SOUND: u8 = 1 << 4;
const MODE_7: u8 = 1 << 7;
const CLOCK_HELD: u8 = 0x44;
const NO_END_CALL: u8 = 0x50;
/// Compared with `func_00AF26`'s count (a byte; less 1).
const MODE_COUNT: u32 = 0xFFFF_9541;

/// The clock's counts (bytes): frames to the next tick (from
/// `TICK_FRAMES`), then tenths (9 to 0), seconds (59 to 0) and
/// minutes.
const CLOCK_TICK: u32 = 0xFFFF_A426;
const CLOCK_TENTHS: u32 = 0xFFFF_A427;
const CLOCK_SECONDS: u32 = 0xFFFF_A428;
const CLOCK_MINUTES: u32 = 0xFFFF_A429;
const TICK_FRAMES: u8 = 6;
const LAST_TENTH: u8 = 9;
const LAST_SECOND: u8 = 0x3B;
/// The minutes the clock starts at in mode 0, else at 5, or by
/// `SELECT_ROW` and `SELECT_ENTRY` from the ROM's tables
/// (`D_00AC2E`, the rows' offsets; `D_00AC34`).
const MODE_0_MINUTES: u8 = 3;
const DEFAULT_MINUTES: u8 = 5;
const SELECT_ROW: u32 = 0xFFFF_953A;
const SELECT_ENTRY: u32 = 0xFFFF_953C;
const ROW_OFFSETS: u32 = 0x00AC2E;
const MINUTES_TABLE: u32 = 0x00AC34;
/// Set when the large clock's task is made (once).
const LARGE_CLOCK_MADE: u32 = 0xFFFF_A42A;
/// Two words `func_00ABA6` sets.
const WORD_A422: u32 = 0xFFFF_A422;
const WORD_A424: u32 = 0xFFFF_A424;
/// What `func_00ABA6` sets them to.
const WORD_A424_START: u16 = 0x1A0;
/// The list `func_0094D8` loads (a long), and the two `func_00ABA6`
/// gives it (in the ROM: `D_00AC48` in mode 0, else `D_00AC56`).
const LOAD_LIST: u32 = 0xFFFF_92FA;
const LOAD_MODE_0: u32 = 0x00AC48;
const LOAD_OTHER: u32 = 0x00AC56;
/// The objects stop while this isn't 0 (`a_objects.rs`).
const OBJECTS_STOPPED: u32 = 0xFFFF_A3FC;

/// The sum `func_00AC64` adds to (a word, 0xFFFF at most), what is left
/// to the next `STEP` (a word), the steps (a byte, 0x7F at most), and
/// a byte it sets.
const SUM: u32 = 0xFFFF_9544;
const STEP_LEFT: u32 = 0xFFFF_9546;
const STEP_COUNT: u32 = 0xFFFF_9543;
const SUMMED: u32 = 0xFFFF_AC7C;
const STEP: u16 = 0x1F4;
/// The Z80's command (`func_0034B6`) at each step, at the clock's 59th
/// second in `func_00ADF2`, and the large clock's.
const STEP_SOUND: u8 = 0x1C;
const LAST_MINUTE_SOUND: u8 = 0x28;
const LARGE_CLOCK_SOUND: u8 = 0x81;

/// Where `func_00ADCA` draws a tile: a VRAM address (a word), its low
/// byte moved along the row.
const DRAW_AT: u32 = 0xFFFF_9E76;
const DRAW_AT_LOW: u32 = 0xFFFF_9E77;
/// Where the clock, the step count and the sum are drawn (in mode 0
/// the clock's first digit two on), and the objects' marks.
const CLOCK_AT: u16 = 0xE046;
const STEP_COUNT_AT: u16 = 0xE07A;
const SUM_AT: u16 = 0xE064;
const MARKS_AT: u16 = 0xE056;
/// The highest step count drawn.
const STEP_COUNT_TOP: u8 = 9;
/// A digit's tile: the digit + 1, with the priority bit.
const DIGIT_PRIORITY: u32 = 1 << 15;
/// The place values of the sum's digits (words in the ROM, `D_00AF1A`,
/// 0 ends them).
const PLACES: u32 = 0x00AF1A;

/// `func_0101BA`'s block of tiles: where it goes (a word, `BLOCK_AT_LOW`
/// its low byte) and its first tile (a word: the attributes in
/// `BLOCK_ATTR`, the tile in `BLOCK_TILE`).
const BLOCK_AT: u32 = 0xFFFF_80C8;
const BLOCK_AT_LOW: u32 = 0xFFFF_80C9;
const BLOCK_ATTR: u32 = 0xFFFF_80D0;
const BLOCK_TILE: u32 = 0xFFFF_80D1;
/// A count of the blocks `func_00AF26` draws in a row (a byte).
const BLOCKS_LEFT: u32 = 0xFFFF_9ED3;
const BLOCK_ROW: u8 = 7;
/// How many of the row's blocks are marks (`MARKS`) and how many of
/// those are the other mark (`MARKS_OTHER`, a byte, 0 or negative
/// none); `func_00AF26` takes 1 from `MARKS_OTHER` when it is 0.
const MARKS: u32 = 0xFFFF_A95A;
const MARKS_OTHER: u32 = 0xFFFF_A959;
/// The blocks' tiles: a mark, the other mark, none.
const TILE_MARK: u8 = 0xE;
const TILE_MARK_OTHER: u8 = 0xC;
const TILE_NONE: u8 = 0x2C;
/// The blocks' sizes (`func_0101BA`'s d1).
const BLOCK_ROW_SIZE: u16 = 0x201;
const BLOCK_OBJECT_SIZE: u16 = 0x303;
const BLOCK_SIDE_SIZE: u16 = 0x301;
/// The task `func_00AF26` makes (`func_006FC6`, priority 0x42) and the
/// byte 0x16 it sets in it.
const AF26_TASK: u32 = 0x006FC6;
const AF26_TASK_PRIORITY: u8 = 0x42;
const AF26_TASK_FIELD: u32 = 0x16;
const AF26_TASK_VALUE: u8 = 0x40;

/// In mode 0, `func_00AF26` goes through 4 objects, by the bytes from
/// `MARK_OBJECTS` (`func_001CFA`; the next byte's by `func_001D0C`),
/// the one at hand in `MARK_INDEX`, and the side of the last in
/// `LAST_SIDE`.
const MARK_OBJECTS: u32 = 0xFFFF_9E88;
const MARK_INDEX: u32 = 0xFFFF_80CD;
const LAST_SIDE: u32 = 0xFFFF_80CC;
const MARK_COUNT: u8 = 4;
/// An object's byte 0 (its use), and its flags: bit 7 marked, bit 6
/// the side, bits 0-2 a number drawn.
const OBJ_USED: u32 = 0x00;
const OBJ_FLAGS: u32 = 0x2D;
const FLAG_MARKED: u8 = 0x80;
const FLAG_SIDE: u8 = 0x40;
const FLAG_NUMBER: u8 = 0x07;
/// The objects' blocks: the tiles, in use or not; the attributes
/// (from the object's index); the side's block.
const TILE_USED: u8 = 0x40;
const TILE_UNUSED: u8 = 0x50;
const SIDE_BLOCK: u16 = 0x8019;
const MARKS_ATTR: u8 = 0x80;
const SUM_ATTR: u8 = 0xA0;
const MARKS_BLOCK_AT: u16 = 0xE016;
const SUM_BLOCK_AT: u16 = 0xE04E;

/// The large clock's task: made by `func_001222` (depth 2, priority
/// 0x42) running `func_00AE6E`, with two more sprite records (depth 2);
/// its fields: the three records' indices (bytes; the seconds' ones,
/// the mark, the tenths), its x and y (words).
const LARGE_CLOCK: u32 = 0x00AE6E;
const LARGE_CLOCK_DEPTH: u8 = 2;
const LARGE_CLOCK_PRIORITY: u8 = 0x42;
const LARGE_ONES: u32 = 0x0C;
const LARGE_MARK: u32 = 0x0D;
const LARGE_TENTHS: u32 = 0x12;
const LARGE_X: u32 = 0xC0;
const LARGE_Y: u32 = 0xC8;
/// The digits' frames: 0x26 + the digit; the mark's, by the tenths (0xA
/// + 0x26 below 5, else none), and the digits' offsets in x.
const DIGIT_FRAME: u8 = 0x26;
const MARK_SHOWN: u32 = 0xA;
const MARK_HIDDEN: u32 = 0xFFFF_FFDA;
const HALF: u8 = 5;

/// The running task's sprite record (index and address).
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// A task's fields: a position (words) and a word `func_00A040` adds
/// to.
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_1A: u32 = 0x1A;
/// A sprite record's fields: frame, attributes (bytes), x and y
/// (words).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_ATTR: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_Y: u32 = 0x0A;
const DIGIT_ATTR: u8 = 0x84;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// divu.w: the quotient in the low word, the remainder in the high
/// (never overflowing here).
fn divu(value: u32, by: u16) -> u32 {
    let by = u32::from(by);
    (value % by) << 16 | (value / by)
}

impl Game {
    /// `func_00A030`: sprite record `a5` at task `a6`'s position, `d0`
    /// (its low word) to the right.  Gives back d0 (its low word the x),
    /// X (the carry) and C (clear).
    pub fn r_00a030(&mut self, d0: u32, a5: u32, a6: u32) -> (u32, bool, bool) {
        let (x, carry) = (d0 as u16).overflowing_add(self.read16(a6 + TASK_X));
        self.write16(a5 + SPRITE_X, x);
        let y = self.read16(a6 + TASK_Y);
        self.write16(a5 + SPRITE_Y, y);
        (low_word(d0, x), carry, false)
    }

    /// `func_00A040`: `d0` (its low word) added to task `a6`'s word
    /// 0x1A.  Gives back d0, its low word the sum.
    pub fn r_00a040(&mut self, d0: u32, a6: u32) -> u32 {
        let sum = self.read16(a6 + TASK_1A).wrapping_add(d0 as u16);
        self.write16(a6 + TASK_1A, sum);
        low_word(d0, sum)
    }

    /// `func_00ABA6`: `WORD_A422` and `WORD_A424` set, the mode's list
    /// loaded (`func_0094D8`) and the clock drawn (`func_00AD66`).
    /// Gives back every register and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00aba6(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        self.write16(WORD_A422, 0);
        self.write16(WORD_A424, WORD_A424_START);
        let list = if self.read8(MODE) & MODE_0 != 0 { LOAD_MODE_0 } else { LOAD_OTHER };
        self.write32(LOAD_LIST, list);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, _) =
            self.r_0094d8(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x);
        }
        let (d0, x) = self.r_00ad66(d0);
        (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x)
    }

    /// `func_00ABE0`: the clock set: `MODE_0_MINUTES` in mode 0, else
    /// `DEFAULT_MINUTES`, or (without `MODE_7`) the minutes from the
    /// tables by `SELECT_ROW` and `SELECT_ENTRY`; `LARGE_CLOCK_MADE`
    /// cleared (not in mode 0), the rest 0, the tick full.  Gives back
    /// d0, d1 (their low words the bytes read) and X.
    pub fn r_00abe0(&mut self, d0: u32, d1: u32, x: bool) -> (u32, u32, bool) {
        let (mut d0, mut d1, mut x) = (d0, d1, x);
        let mode = self.read8(MODE);
        if mode & MODE_0 != 0 {
            self.write8(CLOCK_MINUTES, MODE_0_MINUTES);
        } else {
            self.write8(LARGE_CLOCK_MADE, 0);
            self.write8(CLOCK_MINUTES, DEFAULT_MINUTES);
            if self.read8(MODE) & MODE_7 == 0 {
                let row = self.read8(SELECT_ROW);
                d0 = low_word(d0, u16::from(row));
                let entry = self.read8(SELECT_ENTRY);
                let offset = self.read8(ROW_OFFSETS + u32::from(row));
                let (index, carry) = entry.overflowing_add(offset);
                d1 = low_word(d1, u16::from(index));
                x = carry;
                let minutes = self.read8(MINUTES_TABLE + u32::from(index));
                self.write8(CLOCK_MINUTES, minutes);
            }
        }
        self.write8(CLOCK_TENTHS, 0);
        self.write8(CLOCK_SECONDS, 0);
        self.write8(CLOCK_TICK, TICK_FRAMES);
        (d0, d1, x)
    }

    /// `func_00AC64`: `d0` (its low byte) added to `SUM` (0xFFFF at most)
    /// and taken from `STEP_LEFT`; when that is 0 or below, a `STEP`
    /// more there, a sound (`STEP_SOUND`) and a step counted
    /// (`STEP_COUNT`, 0x7F at most).  `SUMMED` set.  Gives back d0 and
    /// X.
    pub fn r_00ac64(&mut self, d0: u32) -> (u32, bool) {
        let amount = u16::from(d0 as u8);
        let mut d0 = low_word(d0, amount);
        let (sum, carry) = self.read16(SUM).overflowing_add(amount);
        self.write16(SUM, if carry { 0xFFFF } else { sum });
        let left = self.read16(STEP_LEFT);
        let now = left.wrapping_sub(amount);
        self.write16(STEP_LEFT, now);
        let mut x = left < amount;
        if now as i16 <= 0 {
            self.write16(STEP_LEFT, now.wrapping_add(STEP));
            d0 = low_byte(d0, STEP_SOUND);
            self.r_0034b6(d0);
            let count = self.read8(STEP_COUNT).wrapping_add(1);
            self.write8(STEP_COUNT, count);
            x = count == 0;
            if count & 0x80 != 0 {
                self.write8(STEP_COUNT, count - 1);
                x = false;
            }
        }
        self.write8(SUMMED, 1);
        (d0, x)
    }

    /// `func_00ACA4`: unless the objects are stopped (`func_001BB4`) or
    /// the clock held (`CLOCK_HELD`), a tick every `TICK_FRAMES` frames:
    /// the clock goes down a tenth.  When it runs out, it stays at 0,
    /// `MODE_RAN_OUT` is set, the objects stop and `func_00B2A0` is
    /// called in mode 0, else `func_00B346` (not with `NO_END_CALL`,
    /// and then the objects don't stop).  Then, in mode 0, at 1:01.0
    /// `func_00B1A2`; else with 0 minutes the large clock
    /// (`func_00ADF2`); and the clock drawn (`func_00AD66`).  Gives back
    /// d0, d1, d5, d6, a0, a1, a5, a6 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00aca4(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let (_, stopped) = self.r_001bb4();
        if stopped {
            return (d0, d1, d5, d6, a0, a1, a5, a6, x);
        }
        let (mut d0, mut d1, mut d5, mut d6, mut a0, mut a1, mut a5, mut a6, mut x) = (d0, d1, d5, d6, a0, a1, a5, a6, x);
        let held = self.read8(MODE) & CLOCK_HELD;
        d0 = low_byte(d0, held);
        if held != 0 {
            return (d0, d1, d5, d6, a0, a1, a5, a6, x);
        }
        let tick = self.read8(CLOCK_TICK);
        self.write8(CLOCK_TICK, tick.wrapping_sub(1));
        x = tick == 0;
        if tick != 1 {
            return (d0, d1, d5, d6, a0, a1, a5, a6, x);
        }
        self.write8(CLOCK_TICK, TICK_FRAMES);
        if self.j_clock_down(CLOCK_TENTHS, &mut x)
            && self.j_reload(CLOCK_TENTHS, LAST_TENTH)
            && self.j_clock_down(CLOCK_SECONDS, &mut x)
            && self.j_reload(CLOCK_SECONDS, LAST_SECOND)
            && self.j_clock_down(CLOCK_MINUTES, &mut x)
        {
            self.write8(CLOCK_TENTHS, 0);
            self.write8(CLOCK_SECONDS, 0);
            self.write8(CLOCK_MINUTES, 0);
            let mode = self.read8(MODE) | MODE_RAN_OUT;
            self.write8(MODE, mode);
            if mode & MODE_0 != 0 {
                self.j_stop_objects();
                (d0, d1, d5, d6, a1, a5, a6) = self.r_00b2a0(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6);
            } else {
                let no_call = self.read8(MODE) & NO_END_CALL;
                d0 = low_byte(d0, no_call);
                if no_call == 0 {
                    self.j_stop_objects();
                    (d0, d1, d5, d6, a1, a5, a6) = self.r_00b346(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
                }
            }
        }
        if self.read8(MODE) & MODE_0 == 0 {
            if self.read8(CLOCK_MINUTES) == 0 {
                (d0, d5, d6, a5, a6) = self.r_00adf2(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6);
            }
        } else if self.read8(CLOCK_TENTHS) == 0 && self.read8(CLOCK_SECONDS) == 1 && self.read8(CLOCK_MINUTES) == 1 {
            (d0, d1, d5, d6, a0, a5, a6) = self.r_00b1a2(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        }
        (d0, x) = self.r_00ad66(d0);
        (d0, d1, d5, d6, a0, a1, a5, a6, x)
    }

    /// A count of the clock less 1 (X its borrow).  Gives back whether
    /// it went below 0.
    fn j_clock_down(&mut self, at: u32, x: &mut bool) -> bool {
        let count = self.read8(at);
        let now = count.wrapping_sub(1);
        self.write8(at, now);
        *x = count == 0;
        now & 0x80 != 0
    }

    /// A count of the clock that went below 0 starts again at `last`.
    fn j_reload(&mut self, at: u32, last: u8) -> bool {
        self.write8(at, last);
        true
    }

    /// The objects stopped (one more in `OBJECTS_STOPPED`; the X of
    /// its addq is drawn over by `func_00AD66`'s).
    fn j_stop_objects(&mut self) {
        let stopped = self.read8(OBJECTS_STOPPED).wrapping_add(1);
        self.write8(OBJECTS_STOPPED, stopped);
    }

    /// `func_00AD66`: the clock drawn at `CLOCK_AT` (two tiles to the
    /// left but in mode 0): the minutes (10 drawn as 0), the seconds'
    /// two digits and in mode 0 the tenths (`func_00ADC0`).  Gives back
    /// d0 and X.
    pub fn r_00ad66(&mut self, d0: u32) -> (u32, bool) {
        self.write16(DRAW_AT, CLOCK_AT);
        let mode_0 = self.read8(MODE) & MODE_0 != 0;
        if !mode_0 {
            self.j_move_draw(-2);
        }
        let minutes = match self.read8(CLOCK_MINUTES) {
            10 => 0,
            m => m,
        };
        self.r_00adc0(low_byte(d0, minutes));
        self.j_move_draw(4);
        let seconds = self.read8(CLOCK_SECONDS);
        let (d0, _) = self.r_00adc0(divu(u32::from(seconds), 10));
        let d0 = d0.rotate_left(16);
        self.j_move_draw(2);
        let (d0, x) = self.r_00adc0(d0);
        if self.read8(MODE) & MODE_0 == 0 {
            return (d0, x);
        }
        self.j_move_draw(4);
        let tenths = self.read8(CLOCK_TENTHS);
        self.r_00adc0(low_byte(d0, tenths))
    }

    /// `DRAW_AT`'s low byte moved by `bytes`.
    fn j_move_draw(&mut self, bytes: i8) {
        let at = self.read8(DRAW_AT_LOW).wrapping_add(bytes as u8);
        self.write8(DRAW_AT_LOW, at);
    }

    /// `func_00ADC0`: digit `d0` (its low byte) drawn at `DRAW_AT`
    /// (`func_00ADCA`): its tile the digit + 1 with `DIGIT_PRIORITY`.
    /// Gives back d0 (the tile in its low word) and X (the + 1's carry).
    pub fn r_00adc0(&mut self, d0: u32) -> (u32, bool) {
        let digit = d0 as u8;
        let tile = (d0 & 0xFFFF_0000) | u32::from(digit.wrapping_add(1)) | DIGIT_PRIORITY;
        let x = self.r_00adca(tile, digit == 0xFF);
        (tile, x)
    }

    /// `func_00ADCA`: tile `d0` (its low word) written at `DRAW_AT`, with
    /// interrupts masked.  Gives back X as it was.
    pub fn r_00adca(&mut self, d0: u32, x: bool) -> bool {
        let tile = d0 as u16;
        let at = self.read16(DRAW_AT);
        let (command, _) = self.r_000282(low_word(d0, at));
        let ipl = self.ipl();
        self.set_ipl(7);
        self.write32(CONTROL, command);
        self.write16(DATA, tile);
        self.set_ipl(ipl);
        x
    }

    /// `func_00ADF2`: the large clock, once (`LARGE_CLOCK_MADE`) and not
    /// with `MODE_7`: its task (`func_001222`, running `func_00AE6E`)
    /// and three sprite records (`func_000796`), their frames 0, at
    /// `LARGE_X`, `LARGE_Y`; at 59 seconds a sound first
    /// (`LAST_MINUTE_SOUND`).  Gives back d0, d5, a5 (the last record's
    /// index and address), d6 and a6 (the task's).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00adf2(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32) {
        if self.read8(LARGE_CLOCK_MADE) != 0 || self.read8(MODE) & MODE_7 != 0 {
            return (d0, d5, d6, a5, a6);
        }
        let mut d0 = d0;
        if self.read8(CLOCK_SECONDS) == LAST_SECOND {
            d0 = low_byte(d0, LAST_MINUTE_SOUND);
            self.r_0034b6(d0);
        }
        let made = self.read8(LARGE_CLOCK_MADE);
        self.write8(LARGE_CLOCK_MADE, made.wrapping_add(1));
        let (mut d5, d6, mut a5, task, mut x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, LARGE_CLOCK_DEPTH, LARGE_CLOCK_PRIORITY, LARGE_CLOCK);
        let depth = u32::from(LARGE_CLOCK_DEPTH);
        for field in [LARGE_ONES, LARGE_MARK, LARGE_TENTHS] {
            (d5, a5, x, _) = self.r_000796(depth, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
            self.write8(task + field, d5 as u8);
            self.write8(a5 + SPRITE_FRAME, 0);
        }
        self.write16(task + TASK_X, LARGE_X as u16);
        self.write16(task + TASK_Y, LARGE_Y as u16);
        (depth, d5, d6, a5, task)
    }

    /// `func_00AE6E` (a task's routine), the large clock: with
    /// `MODE_SOUND`, a sound at each half second (`LARGE_CLOCK_SOUND`;
    /// not at 0 seconds); the seconds' tens in the task's own sprite
    /// record, their ones in `LARGE_ONES`'s, the mark (shown for the
    /// first half of each second) in `LARGE_MARK`'s, the tenths in
    /// `LARGE_TENTHS`'s (`func_00AEFA`).  Gives back d0, d3, d5, a5, X
    /// and C.
    pub fn r_00ae6e(&mut self, regs: &mut Regs) {
        let task = regs.a6;
        if self.read8(MODE) & MODE_SOUND != 0 && self.read8(CLOCK_TICK) == TICK_FRAMES {
            let tenths = self.read8(CLOCK_TENTHS);
            regs.d0 = low_byte(regs.d0, tenths);
            if (tenths == 0 || tenths == HALF) && self.read8(CLOCK_SECONDS) != 0 {
                regs.d0 = low_byte(regs.d0, LARGE_CLOCK_SOUND);
                self.r_0034b6(regs.d0);
            }
        }
        let seconds = divu(u32::from(self.read8(CLOCK_SECONDS)), 10);
        let index = self.read8(SLOT_INDEX);
        let mut d5 = low_byte(regs.d5, index);
        let mut a5 = self.read32(SLOT_ADDR);
        // each digit's x offset (d3's low byte)
        self.r_00aefa(seconds, 0, a5, task);
        let record = self.read8(task + LARGE_ONES);
        (d5, a5, _) = self.r_000b5a(low_byte(d5, record));
        self.r_00aefa(seconds.rotate_left(16), 0xB, a5, task);
        let record = self.read8(task + LARGE_MARK);
        (d5, a5, _) = self.r_000b5a(low_byte(d5, record));
        let mark = if self.read8(CLOCK_TENTHS) < HALF { MARK_SHOWN } else { MARK_HIDDEN };
        self.r_00aefa(mark, 0x18, a5, task);
        let record = self.read8(task + LARGE_TENTHS);
        (d5, a5, _) = self.r_000b5a(low_byte(d5, record));
        let tenths = self.read8(CLOCK_TENTHS);
        let (d0, d3, x, c) = self.r_00aefa(low_byte(mark, tenths), 0x24, a5, task);
        (regs.d0, regs.d3, regs.d5, regs.a5, regs.x, regs.c) = (d0, d3, d5, a5, x, c);
    }

    /// `func_00AEFA`: sprite record `a5` shows digit `d0` (its low byte;
    /// frame `DIGIT_FRAME` + it) at task `a6`'s position, `d3` (its low
    /// byte, signed) to the right.  Gives back d0 (the frame in its low
    /// byte), d3 (the x in its low word), X (the carry) and C (clear).
    pub fn r_00aefa(&mut self, d0: u32, d3: u32, a5: u32, a6: u32) -> (u32, u32, bool, bool) {
        let frame = (d0 as u8).wrapping_add(DIGIT_FRAME);
        self.write8(a5 + SPRITE_FRAME, frame);
        self.write8(a5 + SPRITE_ATTR, DIGIT_ATTR);
        let offset = d3 as u8 as i8 as u16;
        let (x, carry) = offset.overflowing_add(self.read16(a6 + TASK_X));
        self.write16(a5 + SPRITE_X, x);
        let y = self.read16(a6 + TASK_Y);
        self.write16(a5 + SPRITE_Y, y);
        (low_byte(d0, frame), low_word(d3, x), carry, false)
    }

    /// `func_00AF26`: in mode 0 the objects' marks (`j_marks`), else: the step count (`STEP_COUNT_TOP` at most) and
    /// the sum's digits without its leading zeroes (`func_00ADC0`), then
    /// a row of `BLOCK_ROW` blocks (`func_0101BA`): `MARKS` marks
    /// (`MARKS_OTHER` of them the other), the rest none.  With
    /// `MARKS_OTHER` 0 it becomes 0xFF and (without `MODE_7`) a task is
    /// made (`AF26_TASK`).  Gives back d0-d6, a0, a4, a5, a6 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00af26(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        if self.read8(MODE) & MODE_0 != 0 {
            return self.j_marks(d0, d1, d2, d3, d4, d5, d6, a5, a6);
        }
        self.write16(DRAW_AT, STEP_COUNT_AT);
        let count = self.read8(STEP_COUNT).min(STEP_COUNT_TOP);
        let (mut d0, _) = self.r_00adc0(low_byte(d0, count));
        self.write16(DRAW_AT, SUM_AT);
        let mut leading = true;
        let mut a0 = PLACES;
        let mut d1 = d1;
        let mut d2 = u32::from(self.read16(SUM));
        loop {
            let place = self.read16(a0);
            a0 += 2;
            d1 = low_word(d1, place);
            if place == 0 {
                break;
            }
            let digit = d2 / u32::from(place);
            d0 = low_word(d0, digit as u16);
            d2 %= u32::from(place);
            if !leading || digit as u8 != 0 {
                (d0, _) = self.r_00adc0(d0);
                leading = false;
            }
            self.j_move_draw(2);
        }
        // the row of blocks
        let mut d4: u32 = if leading { 0 } else { 1 };
        let mut a6 = a6;
        self.write16(BLOCK_AT, SUM_BLOCK_AT);
        self.write8(BLOCK_ATTR, SUM_ATTR);
        d1 = low_word(d1, BLOCK_ROW_SIZE);
        self.write8(BLOCKS_LEFT, BLOCK_ROW);
        let marks = self.read8(MARKS);
        let mut d3 = low_word(d3, u16::from(marks));
        if marks != 0 {
            let mut other = self.read8(MARKS_OTHER);
            for _ in 0..marks {
                let mut tile = TILE_MARK;
                if other != 0 && other & 0x80 == 0 {
                    other -= 1;
                    tile = TILE_MARK_OTHER;
                }
                self.write8(BLOCK_TILE, tile);
                (_, d1, a0, a6) = self.r_0101ba(d1, a0, a6);
                let left = self.read8(BLOCKS_LEFT);
                self.write8(BLOCKS_LEFT, left.wrapping_sub(1));
                let at = self.read16(BLOCK_AT);
                self.write16(BLOCK_AT, at.wrapping_add(2));
            }
            d4 = low_byte(d4, other);
            // dbra's end
            d3 = low_word(d3, 0xFFFF);
        }
        let mut x;
        loop {
            self.write8(BLOCK_TILE, TILE_NONE);
            (d0, d1, a0, a6) = self.r_0101ba(d1, a0, a6);
            let at = self.read16(BLOCK_AT);
            self.write16(BLOCK_AT, at.wrapping_add(2));
            let left = self.read8(BLOCKS_LEFT);
            self.write8(BLOCKS_LEFT, left.wrapping_sub(1));
            x = left == 0;
            if left == 1 {
                break;
            }
        }
        let mut d6 = d6;
        if self.read8(MARKS_OTHER) == 0 {
            self.write8(MARKS_OTHER, 0xFF);
            x = true;
            if self.read8(MODE) & MODE_7 == 0 {
                let (index, task, task_x, _) = self.r_0003e8(d4, d5, d6, AF26_TASK_PRIORITY, AF26_TASK);
                (d6, a6, x) = (index, task, task_x);
                self.write8(a6 + AF26_TASK_FIELD, AF26_TASK_VALUE);
            }
        }
        (d0, d1, d2, d3, d4, d5, d6, a0, a4, a5, a6, x)
    }

    /// `func_00AF26` in mode 0: for each of `MARK_COUNT` objects marked
    /// (`FLAG_MARKED`), a block (`func_0101BA`) in use or not, after the
    /// side's block where its side differs from the last one's, then its
    /// number (`FLAG_NUMBER` + 1, `func_00ADCA`): with `MODE_COMPARE`,
    /// only if it is the last (`MODE_COUNT` less 1) or its side differs
    /// from the next object's.  The marks' area cleared first
    /// (`func_00B126`).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn j_marks(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let (mut d0, mut d1, mut d4, mut d5, mut a5, mut a6) = (d0, d1, d4, d5, a5, a6);
        let (mut a0, mut a4);
        self.r_00b126(d1, d4);
        self.write8(LAST_SIDE, 0);
        d1 = low_word(d1, BLOCK_OBJECT_SIZE);
        self.write16(DRAW_AT, MARKS_AT);
        self.write16(BLOCK_AT, MARKS_BLOCK_AT);
        self.write8(BLOCK_ATTR, MARKS_ATTR);
        d0 = low_word(d0, 0);
        let x = loop {
            let k = d0 as u8;
            self.write8(MARK_INDEX, k);
            d0 = low_word(d0, k as i8 as u16);
            a0 = MARK_OBJECTS;
            let index = self.read8(MARK_OBJECTS + u32::from(k));
            (d4, a4, _) = self.r_001cfa(low_byte(d4, index));
            let flags = self.read8(a4 + OBJ_FLAGS);
            d0 = low_byte(d0, flags);
            if flags & FLAG_MARKED != 0 {
                let last = self.read8(LAST_SIDE);
                let side = self.read8(a4 + OBJ_FLAGS) & FLAG_SIDE;
                self.write8(LAST_SIDE, side);
                if last != side {
                    d1 = low_word(d1, BLOCK_SIDE_SIZE);
                    self.write16(BLOCK_ATTR, SIDE_BLOCK);
                    (_, d1, a0, a6) = self.r_0101ba(d1, a0, a6);
                    self.j_move_draw(4);
                    let at = self.read8(BLOCK_AT_LOW).wrapping_add(4);
                    self.write8(BLOCK_AT_LOW, at);
                }
                d1 = low_word(d1, BLOCK_OBJECT_SIZE);
                let attr = (d4 as u8) << 5 & 0x60 | 0x80;
                self.write8(BLOCK_ATTR, attr);
                let used = self.read8(a4 + OBJ_USED) != 0;
                self.write8(BLOCK_TILE, if used { TILE_USED } else { TILE_UNUSED });
                (d0, d1, a0, a6) = self.r_0101ba(d1, a0, a6);
                let at = self.read8(BLOCK_AT_LOW).wrapping_add(6);
                self.write8(BLOCK_AT_LOW, at);
                self.j_move_draw(6);
                let mut draw = true;
                if self.read8(MODE) & MODE_COMPARE != 0 {
                    let last_index = self.read8(MODE_COUNT).wrapping_sub(1);
                    d0 = low_byte(d0, last_index);
                    let k = self.read8(MARK_INDEX);
                    if last_index != k {
                        d0 = low_word(d0, k as i8 as u16);
                        a0 = MARK_OBJECTS;
                        let next = self.read8(MARK_OBJECTS + 1 + u32::from(k));
                        (d5, a5) = self.r_001d0c(low_byte(d5, next));
                        let theirs = self.read8(a5 + OBJ_FLAGS);
                        let ours = self.read8(a4 + OBJ_FLAGS);
                        d1 = low_byte(d1, ours);
                        let differ = u16::from(theirs ^ ours) & u16::from(FLAG_SIDE);
                        d0 = low_word(d0, differ);
                        draw = differ != 0;
                    }
                }
                if draw {
                    let number = self.read8(a4 + OBJ_FLAGS) & FLAG_NUMBER;
                    d0 = low_word(d0, u16::from(number + 1) | DIGIT_PRIORITY as u16);
                    self.r_00adca(d0, false);
                    let at = self.read8(BLOCK_AT_LOW).wrapping_add(4);
                    self.write8(BLOCK_AT_LOW, at);
                    self.j_move_draw(4);
                }
            }
            let next = self.read8(MARK_INDEX) + 1;
            d0 = low_byte(d0, next);
            if next == MARK_COUNT {
                // addq.b's carry: none
                break false;
            }
        };
        (d0, d1, d2, d3, d4, d5, d6, a0, a4, a5, a6, x)
    }
}
