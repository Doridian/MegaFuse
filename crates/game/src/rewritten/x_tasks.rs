//! Helpers of the screens at 0x18D00-0x19800 (0x1900E-0x19824): a task
//! with a sprite made every fifth call at the current task's place
//! (`func_01900E`) and its routine (`func_019060`), the tasks that end
//! once a long pair is 0 (`func_0193AC`) or a byte is (`func_0198C2`),
//! six colour cycles (`func_01914A`), a map cell drawn and emptied
//! (`func_0195FC`), a sprite dropped by a height (`func_01960E`), and
//! a routine of the horizontal interrupt's list (`func_0197DC`).  The
//! `rts`es the translation keeps as routines are here too.

use crate::{Game, Regs};

/// A task's fields: the bytes `func_01900E` and `func_01960E` copy to
/// a sprite's x and y low bytes, a sprite record's index
/// (`func_01960E`) and its animation (a long, `anim.rs`).
const TASK_X_BYTE: u32 = 0x0D;
const TASK_Y_BYTE: u32 = 0x0E;
const TASK_SPRITE: u32 = 0x0C;
const TASK_ANIM: u32 = 0x20;

/// A sprite record's frame and flags, its position (words, x then y)
/// and their bytes.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;

/// The current task's index (`func_0005F8`).
const CURRENT_TASK: u32 = 0xFFFF_80EE;
/// `func_01900E`'s count of calls, 0 to 4.
const CALLS: u32 = 0xFFFF_ADD6;
const EVERY: u8 = 5;
/// `func_01900E`'s task: its routine, priority and depth, the
/// animation (ROM) and the sprite's flags.
const MADE_ROUTINE: u32 = 0x019060;
const MADE_PRIORITY: u8 = 0x32;
const MADE_DEPTH: u8 = 0x28;
const MADE_ANIM: u32 = 0x0DE25C;
const MADE_FLAGS: u8 = 0x85;

/// Two longs `func_0193AC` waits on to be both 0.
const LONG_ADC6: u32 = 0xFFFF_ADC6;
const LONG_ADCA: u32 = 0xFFFF_ADCA;
/// A count `func_0198C2` waits on to be 0 (`x_object.rs` counts it
/// down).
const COUNT_ADE4: u32 = 0xFFFF_ADE4;

/// What an object is set up with (`anim.rs`: its routine, its
/// animations), used by `func_01914A` for other things: the routine's
/// long set to three addresses in the ROM before each colour cycle
/// (`func_00AA00` doesn't read it: its list is a0), and the
/// animations' first word, whose low byte is the colour cycled
/// (`func_00AA00`); its high byte is counted up.
const POINTER_80D0: u32 = 0xFFFF_80D0;
const CYCLE_WORD: u32 = 0xFFFF_80C8;
const POINTERS: [u32; 3] = [0x0D933E, 0x0D934C, 0x0D935A];
const CYCLE_FIRST: u16 = 0x00AC;
const CYCLE_SECOND: u16 = 0x01AC;

/// The cell drawn by `func_0195FC` (`func_000FBE`).
const CELL_DRAWN: u32 = 0x14;

/// `func_01960E`'s drop, kept, and the sprite record it moves (index,
/// address) first.
const DROP: u32 = 0xFFFF_80C5;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;
/// The frames by drop: under `DROP_LOW`, under `DROP_HIGH`, else.
const DROP_LOW: u8 = 0x0C;
const DROP_HIGH: u8 = 0x18;
const FRAME_LOW: u8 = 0xE0;
const FRAME_MIDDLE: u8 = 0xE1;
const FRAME_HIGH: u8 = 0xE2;

/// `func_0197DC`'s line (kept, and the list's first entry's line),
/// what it adds each time (signed) and the line it stops under.
const LINE: u32 = 0xFFFF_ADD9;
const STEP: u32 = 0xFFFF_ADD8;
const LIST_LINE: u32 = 0xFFFF_A40E;
const LINE_END: u8 = 0x1F;
/// The vertical scroll's value (`h_scroll.rs`), its low byte: 0xBF -
/// the line, and 0xA0 at the end.
const V_VALUE_LOW: u32 = 0xFFFF_D403;
const V_FROM: u8 = 0xBF;
const V_END: u8 = 0xA0;
/// Cleared when `func_0197DC` ends.
const BYTE_ADD7: u32 = 0xFFFF_ADD7;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_01905E`: an `rts`.
    pub fn r_01905e(&mut self) {}

    /// `func_0193AA`: an `rts`.
    pub fn r_0193aa(&mut self) {}

    /// `func_01924C`: an `rts`.
    pub fn r_01924c(&mut self) {}

    /// `func_0194A8`: an `rts`.
    pub fn r_0194a8(&mut self) {}

    /// `func_01900E`: `CALLS` counted; every fifth call (back to 0) a
    /// task `func_019060` with a sprite (`func_001222`) running
    /// `MADE_ANIM`, the sprite's flags `MADE_FLAGS` and its x and y
    /// (words) the current task's `TASK_X_BYTE`, `TASK_Y_BYTE`.  Gives
    /// back d5, d6, a5, a6 (the current task) and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_01900e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, bool) {
        let (calls, x) = self.read8(CALLS).overflowing_add(1);
        self.write8(CALLS, calls);
        if calls < EVERY {
            return (d5, d6, a5, a6, x);
        }
        self.write8(CALLS, 0);
        let (d5, d6, sprite, task, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, MADE_DEPTH, MADE_PRIORITY, MADE_ROUTINE);
        self.write32(task + TASK_ANIM, MADE_ANIM);
        self.write8(sprite + SPRITE_FLAGS, MADE_FLAGS);
        let d6 = low_byte(d6, self.read8(CURRENT_TASK));
        let (d6, current, x) = self.r_0005f8(d6);
        let x_byte = self.read8(current + TASK_X_BYTE);
        self.write8(sprite + SPRITE_X_LOW, x_byte);
        let y_byte = self.read8(current + TASK_Y_BYTE);
        self.write8(sprite + SPRITE_Y_LOW, y_byte);
        self.write8(sprite + SPRITE_X, 0);
        self.write8(sprite + SPRITE_Y, 0);
        (d5, d6, sprite, current, x)
    }

    /// `func_019060` (a task's routine, `func_01900E`'s): its animation
    /// a frame on (`func_0013D6`); when it ends, the task ends
    /// (`func_00121A`).
    pub fn r_019060(&mut self, regs: &mut Regs) {
        let r = regs;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        }
        r.unwound = false;
    }

    /// `func_0193AC` (a task's routine): ends (`func_00121A`) once
    /// `LONG_ADC6` and `LONG_ADCA` are both 0.  d0 their or.
    pub fn r_0193ac(&mut self, regs: &mut Regs) {
        let r = regs;
        r.d0 = self.read32(LONG_ADC6) | self.read32(LONG_ADCA);
        r.c = false;
        if r.d0 == 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        }
        r.unwound = false;
    }

    /// `func_0198C2` (a task's routine): ends (`func_00121A`) once
    /// `COUNT_ADE4` is 0.  d0's low byte the count.
    pub fn r_0198c2(&mut self, regs: &mut Regs) {
        let r = regs;
        let count = self.read8(COUNT_ADE4);
        r.d0 = low_byte(r.d0, count);
        r.c = false;
        if count == 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        }
        r.unwound = false;
    }

    /// `func_01914A`: six colour cycles (`func_00AA00`), the colour
    /// `CYCLE_WORD`'s low byte (0xAC each time); before each,
    /// `POINTER_80D0` one of `POINTERS` in turn and `CYCLE_WORD`
    /// `CYCLE_FIRST` or `CYCLE_SECOND` for the first of three, its high
    /// byte counted up for the others.  The list each cycle goes
    /// through is a0: the caller's first, then the address the last
    /// cycle gave back (the original never sets it from
    /// `POINTER_80D0`).  Gives back d0 and a0 (the last cycle's).
    pub fn r_01914a(&mut self, d4: u32, d5: u32, d6: u32, a0: u32) -> (u32, u32) {
        let mut d0 = 0;
        let mut a0 = a0;
        for start in [CYCLE_FIRST, CYCLE_SECOND] {
            for (i, &pointer) in POINTERS.iter().enumerate() {
                self.write32(POINTER_80D0, pointer);
                if i == 0 {
                    self.write16(CYCLE_WORD, start);
                } else {
                    let high = self.read8(CYCLE_WORD);
                    self.write8(CYCLE_WORD, high.wrapping_add(1));
                }
                (d0, a0, _, _) = self.r_00aa00(d4, d5, d6, a0);
            }
        }
        (d0, a0)
    }

    /// `func_0195FC`: the cell at `CELL_X`, `CELL_Y` (`func_001278`)
    /// drawn as cell `CELL_DRAWN` (`func_000FBE`) and emptied in the
    /// cell map (`func_00167A`, `func_0019B6`).  Gives back d0 and a0.
    pub fn r_0195fc(&mut self) -> (u32, u32) {
        self.r_001278();
        let (d0, _, _) = self.r_000fbe(CELL_DRAWN);
        self.r_00167a();
        let a0 = self.r_0019b6();
        (d0, a0)
    }

    /// `func_01960E`: the sprite at `SLOT_ADDR` put at the task's
    /// `TASK_X_BYTE` and `TASK_Y_BYTE` - `drop` (d0's low byte, kept in
    /// `DROP`; y's high byte 0xFF when that borrows), and the frame of
    /// the task's sprite `TASK_SPRITE` (`func_000B5A`) by the drop.
    /// Gives back d0 (its low byte the drop), d1 (its low byte 0), d5,
    /// a5 (the task's sprite), X and C (clear).
    pub fn r_01960e(&mut self, d0: u32, d1: u32, d5: u32, a6: u32) -> (u32, u32, u32, u32, bool, bool) {
        let drop = d0 as u8;
        self.write8(DROP, drop);
        let sprite = self.read32(SLOT_ADDR);
        let x_byte = self.read8(a6 + TASK_X_BYTE);
        self.write8(sprite + SPRITE_X_LOW, x_byte);
        let (y, borrow) = self.read8(a6 + TASK_Y_BYTE).overflowing_sub(drop);
        self.write8(sprite + SPRITE_Y_LOW, y);
        self.write8(sprite + SPRITE_Y, if borrow { 0xFF } else { 0 });
        let d5 = low_byte(d5, self.read8(a6 + TASK_SPRITE));
        let (d5, a5, x) = self.r_000b5a(d5);
        let drop = self.read8(DROP);
        let frame = if drop < DROP_LOW {
            FRAME_LOW
        } else if drop < DROP_HIGH {
            FRAME_MIDDLE
        } else {
            FRAME_HIGH
        };
        self.write8(a5 + SPRITE_FRAME, frame);
        (low_byte(d0, drop), low_byte(d1, 0), d5, a5, x, false)
    }

    /// `func_0197DC` (a routine of the horizontal interrupt's list,
    /// `j_lines.rs`): `LINE` + `STEP`; while that is positive and under
    /// `LINE_END` it is the line (and the list's first entry's) and the
    /// vertical scroll's value `V_FROM` - it (`func_008664`).  Else (at
    /// `LINE_END` or more after the window's table, `func_00B49A`) the
    /// value `V_END` (`func_008664`) and `BYTE_ADD7` 0.  No script
    /// calls it: reviewed by reading.
    pub fn r_0197dc(&mut self, regs: &mut Regs) {
        let r = regs;
        let (line, carry) = self.read8(LINE).overflowing_add(self.read8(STEP));
        r.d0 = low_byte(r.d0, line);
        r.x = carry;
        r.c = carry;
        r.n = line & 0x80 != 0;
        if !r.n {
            if line < LINE_END {
                self.write8(LINE, line);
                self.write8(LIST_LINE, line);
                let value = V_FROM.wrapping_sub(line);
                r.d0 = low_byte(r.d0, value);
                r.x = false;
                r.c = false;
                self.write8(V_VALUE_LOW, value);
                self.r_008664(r);
                r.unwound = false;
                return;
            }
            r.c = false;
            self.r_00b49a(r);
        }
        self.write8(V_VALUE_LOW, V_END);
        self.r_008664(r);
        self.write8(BYTE_ADD7, 0);
        r.n = false;
        r.z = true;
        r.c = false;
        r.unwound = false;
    }
}
