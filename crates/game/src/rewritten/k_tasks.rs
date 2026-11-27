//! Tasks with a sprite that move across the screen, from 0xB1A2: one
//! that flies in and walks a path of map cells, making a task at each
//! (`func_00B1A2` on; `func_00B3BC`, `func_00B414`), and two that
//! come down and then end the play (`func_00B2A0`, `func_00B346`, with
//! `loc_00B312`).

use crate::{Game, Regs};

/// The tasks' routines (code addresses).
const FLY_IN: u32 = 0x00B1DC;
const WALK: u32 = 0x00B21A;
const FALL: u32 = 0x00B2DE;
const FALLEN: u32 = 0x00B300;
const DROP: u32 = 0x00B378;
const DROPPED: u32 = 0x00B39E;
const CELL_TASK: u32 = 0x00B414;
/// What a task runs at last (`loc_00B312`).
const LAST: u32 = 0x001212;
/// The routines kept for `LAST` (`FALLEN`'s, `DROPPED`'s).
const KEPT_FALLEN: u32 = 0x01151A;
const KEPT_DROPPED: u32 = 0x00320A;

/// `WALK`'s path (`D_00B286`, in the ROM): pairs of a way (0-3) and a
/// number of cells, to a negative way.
const PATH: u32 = 0x00B286;
/// A cell's step by way: columns, rows (`D_00B298`, `D_00B29C`).
const STEP_COLUMN: u32 = 0x00B298;
const STEP_ROW: u32 = 0x00B29C;

/// Flags of the game (`b_control.rs`): bit 0 the effects' mode,
/// bit 2 `WALK` waits, bit 3 set by `loc_00B312`.
const MODE: u32 = 0xFFFF_9540;
const MODE_EFFECTS: u8 = 1 << 0;
const MODE_WAIT: u8 = 1 << 2;
const MODE_ENDED: u8 = 1 << 3;
/// One more for each `DROP` that lands (`k_vdp.rs`: the play stops).
const STOPPED: u32 = 0xFFFF_A3FC;
/// A velocity (words, x then y; `a_motion.rs`).
const VELOCITY_X: u32 = 0xFFFF_9EC4;
const VELOCITY_Y: u32 = 0xFFFF_9EC6;
/// A map cell, column then row (bytes; `a_cells.rs`).
const CELL_X: u32 = 0xFFFF_9F08;
/// The frame `CELL_TASK`'s sprite shows.
const CELL_FRAME: u32 = 0xFFFF_9EC2;
/// `a_cells.rs`: the cell's address `func_00167A` keeps.
const CELL_AT: u32 = 0xFFFF_9ED6;
/// Where the next cell goes on the plane (`map.rs`): its column x 2
/// (a byte) and its row's offset (a word).
const COLUMN: u32 = 0xFFFF_9E74;
const ROW: u32 = 0xFFFF_9E76;
/// Bits or'd into the tile word `CELL_TASK` writes.
const TILE_BITS: u32 = 0xFFFF_9EC0;
/// `func_0101C0`'s arguments: where on the plane (a word) and what
/// (a word).
const PLANE_AT: u32 = 0xFFFF_80C8;
const PLANE_WORD: u32 = 0xFFFF_80D0;

/// A task's fields.
/// Its position (words: x, y; `a_motion.rs`).
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
/// `func_00B3BC`'s task: the byte of its y it lowers.
const TASK_Y_HIGH: u32 = 0x10;
/// The frames left (a byte).
const TIMER: u32 = 0x16;
/// `WALK`'s: the cells left on the way, the way, the path's entry.
const CELLS_LEFT: u32 = 0x12;
const WAY: u32 = 0x15;
const PATH_AT: u32 = 0x21;
/// A map cell (bytes: column, row).
const CELL: u32 = 0x1A;
const CELL_ROW: u32 = 0x1B;
/// The routine kept for later (`task.rs`, `loc_000558`).
const KEPT: u32 = 0x1C;

/// A sprite record's fields (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_Y: u32 = 0x0A;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// A table's entry by a register's low word (signed).
fn word_at(table: u32, index: u32) -> u32 {
    table.wrapping_add(index as u16 as i16 as u32)
}

impl Game {
    /// A task's byte at `field` one less; true if it is 0 now, and the
    /// borrow.
    fn k_count_down(&mut self, task: u32, field: u32) -> (bool, bool) {
        let old = self.read8(task + field);
        let new = old.wrapping_sub(1);
        self.write8(task + field, new);
        (new == 0, old == 0)
    }

    /// `func_00B1A2`: a task running `FLY_IN` (priority 0x40, sprite
    /// depth 4) at 0x120, 0x60, 0x60 frames; frame 0x18; sound 0x28;
    /// then its first frame.  Gives back d0, d1, d5, d6, a0, a5, a6.
    /// No script runs it (checked by reading).
    #[allow(clippy::too_many_arguments)]
    pub fn r_00b1a2(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        let (d5, d6, sprite, task, x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 4, 0x40, FLY_IN);
        self.write8(task + TIMER, 0x60);
        self.write16(task + TASK_X, 0x120);
        self.write16(task + TASK_Y, 0x60);
        self.write8(sprite + SPRITE_FRAME, 0x18);
        self.write8(sprite + SPRITE_FLAGS, 0xC4);
        let d0 = low_byte(d0, 0x28);
        self.r_0034b6(d0);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5: sprite, a6: task, x, ..Regs::default() };
        self.r_00b1dc(&mut r);
        (r.d0, r.d1, r.d5, r.d6, r.a0, r.a5, r.a6)
    }

    /// `func_00B1DC`, `FLY_IN`: the task moves left (x velocity
    /// -4); when its time is up, its sprite record is freed and it
    /// walks the path from cell (2, 1) (`WALK`, at once).  Gives back
    /// d0, d1, X and C, or `WALK`'s.
    /// No script runs it (checked by reading).
    pub fn r_00b1dc(&mut self, regs: &mut Regs) {
        let task = regs.a6;
        let (up, _) = self.k_count_down(task, TIMER);
        if !up {
            self.write16(VELOCITY_X, 0xFC00);
            (regs.d0, regs.d1, _, _, _) = self.r_001a88(regs.d0, regs.d1, task);
            (regs.d0, regs.x, regs.c) = self.r_001b7e(regs.a5, task);
            regs.unwound = false;
            return;
        }
        self.r_000890();
        self.write8(task + CELL, 2);
        self.write8(task + PATH_AT, 0);
        regs.d0 = 1;
        self.write8(task + TIMER, 1);
        self.write8(task + CELL_ROW, 1);
        self.write8(task + CELLS_LEFT, 1);
        (regs.x, regs.c) = self.r_00053e(regs.d6, WALK);
        self.r_00b21a(regs);
    }

    /// `func_00B21A`, `WALK` (not while `MODE_WAIT`): every 0x1C
    /// frames a cell task (`func_00B3BC`) at its cell, which then moves
    /// a cell its way; when a way's cells are done, the next pair of
    /// `PATH`; at its end the task ends.  Gives back d0, d1, d5, d6,
    /// a0, a5, a6, X and C as they are left.
    /// No script runs it (checked by reading).
    pub fn r_00b21a(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let task = regs.a6;
        if self.read8(MODE) & MODE_WAIT != 0 {
            return;
        }
        let (up, borrow) = self.k_count_down(task, TIMER);
        if !up {
            regs.x = borrow;
            regs.c = borrow;
            return;
        }
        loop {
            self.write8(task + TIMER, 0x1C);
            let (done, borrow) = self.k_count_down(task, CELLS_LEFT);
            regs.x = borrow;
            if !done {
                break;
            }
            // the entry as a word (ext.w), its low byte stepped on
            let entry = self.read8(task + PATH_AT);
            regs.d0 = low_word(regs.d0, entry as i8 as u16);
            let way = self.read8(word_at(PATH, regs.d0));
            regs.d1 = low_byte(regs.d1, way);
            if (way as i8) < 0 {
                (regs.d6, regs.x, regs.c) = self.r_000560(regs.d5, regs.d6, regs.x);
                return;
            }
            self.write8(task + WAY, way);
            regs.d0 = low_byte(regs.d0, entry.wrapping_add(1));
            let cells = self.read8(word_at(PATH, regs.d0));
            self.write8(task + CELLS_LEFT, cells);
            regs.d0 = low_byte(regs.d0, entry.wrapping_add(2));
            self.write8(task + PATH_AT, entry.wrapping_add(2));
        }
        let cell = self.read16(task + CELL);
        self.write16(CELL_X, cell);
        let r = &mut *regs;
        (r.d0, r.d5, r.d6, r.a0, r.a5, r.a6) = self.r_00b3bc(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        let task = regs.a6;
        let way = self.read8(task + WAY);
        regs.d0 = low_word(regs.d0, way as i8 as u16);
        let column = self.read8(word_at(STEP_COLUMN, regs.d0));
        let cell = self.read8(task + CELL).wrapping_add(column);
        self.write8(task + CELL, cell);
        let step = self.read8(word_at(STEP_ROW, regs.d0));
        regs.d1 = low_byte(regs.d1, step);
        let (row, carry) = self.read8(task + CELL_ROW).overflowing_add(step);
        self.write8(task + CELL_ROW, row);
        regs.x = carry;
        regs.c = carry;
    }

    /// `func_00B2A0`: in the effects' mode, a task running `FALL`
    /// (priority 0x30, sprite depth 4) at 0x80, -0x80, 0x25 frames;
    /// then its first frame.  Gives back d0, d1, d5, d6, a1, a5, a6.
    /// No script runs it (checked by reading).
    #[allow(clippy::too_many_arguments)]
    pub fn r_00b2a0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        if self.read8(MODE) & MODE_EFFECTS == 0 {
            return (d0, d1, d5, d6, a1, a5, a6);
        }
        let (d5, d6, sprite, task, x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 4, 0x30, FALL);
        self.write16(task + TASK_X, 0x80);
        self.write16(task + TASK_Y, 0xFF80);
        self.write8(sprite + SPRITE_FLAGS, 0x84);
        self.write8(sprite + SPRITE_FRAME, 0x17);
        self.write8(task + TIMER, 0x25);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5: sprite, a6: task, x, ..Regs::default() };
        self.r_00b2de(&mut r);
        (r.d0, r.d1, r.d5, r.d6, r.a1, r.a5, r.a6)
    }

    /// `func_00B2DE`, `FALL`: the task moves down (y velocity 6) for
    /// its time; then sound 0x29 and `FALLEN` (at once), 0x80 frames.
    /// Gives back d0, d1, X and C, or `FALLEN`'s.
    /// No script runs it (checked by reading).
    pub fn r_00b2de(&mut self, regs: &mut Regs) {
        let task = regs.a6;
        let (up, _) = self.k_count_down(task, TIMER);
        if !up {
            self.write16(VELOCITY_Y, 0x600);
            (regs.d0, regs.d1, _, _, _) = self.r_001aa2(regs.d0, regs.d1, task);
            (regs.d0, regs.x, regs.c) = self.r_001b7e(regs.a5, task);
            regs.unwound = false;
            return;
        }
        regs.d0 = low_byte(regs.d0, 0x29);
        self.r_0034b6(regs.d0);
        self.write8(task + TIMER, 0x80);
        (regs.x, regs.c) = self.r_00053e(regs.d6, FALLEN);
        self.r_00b300(regs);
    }

    /// `func_00B300`, `FALLEN`: when its time is up, `loc_00B312`
    /// with `KEPT_FALLEN` kept.  Gives back X and C (the count's
    /// borrow), or d0, d1, a1, X and C.
    /// No script runs it (checked by reading).
    pub fn r_00b300(&mut self, regs: &mut Regs) {
        self.k_ended(regs, KEPT_FALLEN);
    }

    /// `FALLEN`'s and `DROPPED`'s end: when the task's time is up,
    /// `kept` is kept, then `loc_00B312`.
    fn k_ended(&mut self, regs: &mut Regs, kept: u32) {
        regs.unwound = false;
        let task = regs.a6;
        let (up, borrow) = self.k_count_down(task, TIMER);
        if !up {
            regs.x = borrow;
            regs.c = borrow;
            return;
        }
        self.write32(task + KEPT, kept);
        let r = &mut *regs;
        (r.d0, r.d1, r.a1, r.x, r.c) = self.r_00b312(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// `loc_00B312`: the task runs `LAST` from now on, `MODE_ENDED` is
    /// set, `func_00A51A` with 4, and 0xFE to the Z80 (d1 0x10).
    /// Gives back d0, d1, a1, X and C (clear).
    #[allow(clippy::too_many_arguments)]
    pub fn r_00b312(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, bool, bool) {
        self.r_00053e(d6, LAST);
        let mode = self.read8(MODE);
        self.write8(MODE, mode | MODE_ENDED);
        let (d0, a1, x, _) = self.r_00a51a(4, d1, d2, d3, d4, d5, d6, d7, a2, a3, a4, a5);
        let d0 = low_byte(d0, 0xFE);
        self.r_00349e(d0);
        (d0, 0x10, a1, x, false)
    }

    /// `func_00B346`: a task running `DROP` (priority 0x30, sprite
    /// depth 1), its sprite at 0x80, -0x60, 0x25 frames; then its
    /// first frame.  Gives back d0, d1, d5, d6, a1, a5, a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00b346(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        let (d5, d6, sprite, task, x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 1, 0x30, DROP);
        self.write16(sprite + SPRITE_X, 0x80);
        self.write16(sprite + SPRITE_Y, 0xFFA0);
        self.write8(sprite + SPRITE_FLAGS, 0x83);
        self.write8(sprite + SPRITE_FRAME, 0x17);
        self.write8(task + TIMER, 0x25);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5: sprite, a6: task, x, ..Regs::default() };
        self.r_00b378(&mut r);
        (r.d0, r.d1, r.d5, r.d6, r.a1, r.a5, r.a6)
    }

    /// `func_00B378`, `DROP`: the sprite moves down 6 a frame for the
    /// task's time; then sound 0x29, `STOPPED` one more and `DROPPED`
    /// (at once), 0x80 frames.  Gives back d0, X and C (the add's), or
    /// `DROPPED`'s.
    pub fn r_00b378(&mut self, regs: &mut Regs) {
        let task = regs.a6;
        let (up, _) = self.k_count_down(task, TIMER);
        if !up {
            regs.d0 = 6;
            let sprite = regs.a5;
            let (y, carry) = self.read16(sprite + SPRITE_Y).overflowing_add(6);
            self.write16(sprite + SPRITE_Y, y);
            regs.x = carry;
            regs.c = carry;
            regs.unwound = false;
            return;
        }
        regs.d0 = low_byte(regs.d0, 0x29);
        self.r_0034b6(regs.d0);
        self.write8(task + TIMER, 0x80);
        self.r_00053e(regs.d6, DROPPED);
        let (stopped, carry) = self.read8(STOPPED).overflowing_add(1);
        self.write8(STOPPED, stopped);
        regs.x = carry;
        regs.c = carry;
        self.r_00b39e(regs);
    }

    /// `func_00B39E`, `DROPPED`: when its time is up, `loc_00B312`
    /// with `KEPT_DROPPED` kept.  Gives back X and C (the count's
    /// borrow), or d0, d1, a1, X and C.
    pub fn r_00b39e(&mut self, regs: &mut Regs) {
        self.k_ended(regs, KEPT_DROPPED);
    }

    /// `func_00B3BC`: if the cell at `CELL_X` (`func_00167A`) isn't 0,
    /// a task running `CELL_TASK` (priority 0x30, sprite depth 4) in
    /// its middle (the high byte of its y one less), frame
    /// `CELL_FRAME`, its cell
    /// kept, 0x40 frames; the caller's d5, d6, a5, a6 kept.  Else the
    /// caller's task (a6) has 1 frame left.  Gives back d0, d5, d6,
    /// a0 (the cell's address), a5, a6.
    /// No script runs it (checked by reading).
    #[allow(clippy::too_many_arguments)]
    pub fn r_00b3bc(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32) {
        let (cell_at, _) = self.r_00167a();
        let cell = self.read8(cell_at);
        let d0 = low_byte(d0, cell);
        if cell == 0 {
            self.write8(a6 + TIMER, 1);
            return (d0, d5, d6, cell_at, a5, a6);
        }
        let (_, _, sprite, task, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 4, 0x30, CELL_TASK);
        let frame = self.read8(CELL_FRAME);
        self.write8(sprite + SPRITE_FRAME, frame);
        self.write8(sprite + SPRITE_FLAGS, 0x84);
        let cell = self.read16(CELL_X);
        self.write16(task + CELL, cell);
        self.r_001550(d0, task);
        let y = self.read8(task + TASK_Y_HIGH);
        self.write8(task + TASK_Y_HIGH, y.wrapping_sub(1));
        let (d0, _, _) = self.r_001b7e(sprite, task);
        self.write8(task + TIMER, 0x40);
        (d0, d5, d6, cell_at, a5, a6)
    }

    /// `func_00B414`, `CELL_TASK`: the task moves down (y velocity 4)
    /// for its time; then sound 0x2D, the task ends (`func_00121A`)
    /// and its cell is cleared: `func_0066FE`, the cell map's byte and
    /// `func_0016B0`'s, and the plane's cell (`func_0101C0`, 2x2 tiles
    /// 0x320 with `TILE_BITS`), then `func_0052F0`.  Gives back d0,
    /// d1, d2, d6, a0, a6, X and C as they are left.
    /// No script runs it (checked by reading).
    pub fn r_00b414(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let task = regs.a6;
        let cell = self.read16(task + CELL);
        self.write16(CELL_X, cell);
        let time = self.read8(task + TIMER).wrapping_sub(1);
        regs.d0 = low_byte(regs.d0, time);
        if time != 0 {
            self.write8(task + TIMER, time);
            self.write16(VELOCITY_Y, 0x400);
            (regs.d0, regs.d1, _, _, _) = self.r_001aa2(regs.d0, regs.d1, task);
            (regs.d0, regs.x, regs.c) = self.r_001b7e(regs.a5, task);
            return;
        }
        regs.d0 = low_byte(regs.d0, 0x2D);
        self.r_0034b6(regs.d0);
        // the borrow of the count (none) goes in as X
        (regs.d6, _, _) = self.r_00121a(regs.d5, regs.d6, false);
        self.r_00167a();
        let r = &mut *regs;
        (r.d2, _) = self.r_0066fe(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        let cell_at = self.read32(CELL_AT);
        self.write8(cell_at, 0);
        let other = self.r_0016b0();
        self.write8(other, 0);
        self.r_001278();
        let column = self.read8(COLUMN);
        let row = self.read16(ROW);
        regs.d1 = low_word(regs.d1, 0x202);
        let tiles = 0x320 | self.read16(TILE_BITS);
        self.write16(PLANE_WORD, tiles);
        let offset = (row & 0xFF00) | u16::from(row as u8 | column);
        let at = offset.wrapping_mul(2).wrapping_add(0xE100);
        self.write16(PLANE_AT, at);
        let r = &mut *regs;
        (r.d0, r.d1, r.a0, r.a6, _) = self.r_0101c0(low_word(r.d0, at), r.d1, other, task);
        self.write16(ROW, row);
        self.write8(COLUMN, column);
        (regs.x, regs.c) = self.r_0052f0(regs.d0);
    }
}
