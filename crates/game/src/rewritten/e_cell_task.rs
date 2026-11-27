//! A task made at a cell whose byte in the cell map is 2
//! (`func_00555E`, `func_0055A0`): it moves a cell at a time in a
//! direction (`func_00560A` while it moves, `func_0056EC` once it is
//! stuck), its sprite frame `MOVING_FRAME`; when it comes to rest the
//! cell is drawn with tiles `REST_TILES` and, if the next cell that way
//! is a 2 too, another such task is made there.

use crate::{Game, Regs};

/// A direction (0 up, 1 right, 2 down, 3 left; a byte).
const DIRECTION: u32 = 0xFFAABD;
/// A cell (column, row; one word) and its address in the cell map.
const CELL: u32 = 0xFF9F08;
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
const CELL_AT: u32 = 0xFF9ED6;
/// The cell's addresses in two more tables (`func_0016B0`).
const TABLE_1A0_AT: u32 = 0xFF9EE2;
const TABLE_0_AT: u32 = 0xFF9EDE;
/// Words at 0xFFAC02 plus `func_0066BE`'s index: the cell the task with
/// that index is at.
const TABLE_AC00: u32 = 0xFFFF_AC00;
/// The running task's index; the slot (`func_000B5A`'s records) it
/// has: index and address.
const CURRENT_TASK: u32 = 0xFF80EE;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;

/// The task: its routine, priority and depth; its sprite's flags and
/// the sound when it is made.
const TASK_ROUTINE: u32 = 0x00560A;
const TASK_PRIORITY: u8 = 0x42;
const TASK_DEPTH: u8 = 0xF8;
const SPRITE_FLAGS_SET: u8 = 0x84;
const MADE_SOUND: u8 = 0x3D;
/// Its routine once stuck, and the frames it stays stuck.
const STUCK: u32 = 0x0056EC;
const STUCK_FRAMES: u8 = 5;
/// Its fields: position (words, x then y), the index from
/// `func_0066BE` (0xFF: none), its direction, a count of frames (3 at
/// the start: the cell is cleared on the plane when it runs out), its
/// cell (column, row).
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_INDEX: u32 = 0x12;
const TASK_DIRECTION: u32 = 0x15;
const TASK_COUNT: u32 = 0x16;
const TASK_CELL: u32 = 0x1A;
/// A sprite record's frame, flags, the low bytes of its x and y.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y_LOW: u32 = 0x0B;
const MOVING_FRAME: u8 = 0xDD;
/// The velocities by direction it moves at (`D_005EEC`, longs in the
/// ROM: x, y).
const PUSHED: u32 = 0x005EEC;
/// A step by direction (signed bytes in the ROM: `D_00572A` x,
/// `D_00572E` y).
const STEP_X: u32 = 0x00572A;
const STEP_Y: u32 = 0x00572E;
/// The tiles of the cell where it comes to rest (`func_005294`).
const REST_TILES: u16 = 0x4304;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_00555E`: unless `func_002C36` says no, the task made
    /// (`func_0055A0`, with a sound) at the cell of the object at a4,
    /// to move in direction `d0` (kept in `DIRECTION`), if the cell's
    /// byte is 2 and no such task is there (`func_005516`).  The
    /// registers but d7 are kept.  Gives back X.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00555e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> bool {
        self.write8(DIRECTION, d0 as u8);
        let (d0, _) = self.r_0015e6(a4);
        let direction = self.read8(DIRECTION);
        let (d0, _, x, no) = self.r_002c36(low_byte(d0, direction), a4);
        if no {
            return x;
        }
        let (cell, x) = self.r_00167a();
        let byte = self.read8(cell);
        let d0 = low_byte(d0, byte);
        if byte != 2 {
            return x;
        }
        let (x, there) = self.r_005516(d0);
        if there {
            return x;
        }
        let (d0, _, _, _, _, _, x) = self.r_0055a0(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        self.r_0034b6(low_byte(d0, MADE_SOUND));
        x
    }

    /// `func_0055A0`: the task made (`func_001222`) at the cell `CELL`,
    /// in its middle, to move in `DIRECTION`; its index from
    /// `func_0066BE` if it gives one.  Gives back d0, d1, d5, d6, a5,
    /// a6 (the task's slot and record) and X.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0055a0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, bool) {
        let (d5, d6, sprite, task, x, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, TASK_DEPTH, TASK_PRIORITY, TASK_ROUTINE);
        if failed {
            return (d0, d1, d5, d6, sprite, task, x);
        }
        self.write8(task + TASK_INDEX, 0xFF);
        let (d1, _, given) = self.r_0066be(d1, d2);
        if given {
            self.write8(task + TASK_INDEX, d1 as u8);
        }
        let direction = self.read8(DIRECTION);
        self.write8(task + TASK_DIRECTION, direction);
        self.write8(task + TASK_COUNT, 3);
        let x_middle = u16::from(self.read8(CELL_X)) << 4 | 8;
        self.write16(task + TASK_X, x_middle);
        let y_middle = u16::from(self.read8(CELL_Y)) << 4 | 8;
        self.write16(task + TASK_Y, y_middle);
        let cell = self.read16(CELL);
        self.write16(task + TASK_CELL, cell);
        self.write8(sprite + SPRITE_FRAME, 0);
        self.write8(sprite + SPRITE_FLAGS, SPRITE_FLAGS_SET);
        // asl.w #4 of a byte: no bit out
        (low_word(d0, y_middle), d1, d5, d6, sprite, task, false)
    }

    /// `func_00560A`, the task's routine while it moves: its cell
    /// cleared from the tables and the task ended when the cell's byte
    /// is 3 or 7 or more (unless `func_0065B6` says otherwise, the
    /// cell made a 7 and 3), ended on a 5; else it moves at `PUSHED`
    /// (`func_005F3A`): its cell kept (and in `TABLE_AC00` by its
    /// index), the cell made a 2, its sprite drawn; when it can't move
    /// it is stuck (`STUCK`).
    /// No script calls it: reviewed by reading.
    pub fn r_00560a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = self.r_0015f8(r.a6);
        (r.a0, r.x) = self.r_00167a();
        let byte = self.read8(r.a0);
        r.d0 = low_byte(r.d0, byte);
        if byte == 5 {
            return self.e_cell_task_ends(r);
        }
        if byte < 7 && byte != 3 {
            r.a2 = PUSHED;
            let stuck;
            (r.d0, r.d1, r.d6, r.a6, stuck) = self.r_005f3a(r.d0, r.d1, r.d2, r.d6, r.a2);
            if stuck {
                return self.e_cell_task_stuck(r);
            }
            let index = self.read8(r.a6 + TASK_INDEX);
            r.d0 = low_word(r.d0, u16::from(index));
            if (index as i8) >= 0 {
                r.a0 = TABLE_AC00;
                let cell = self.read16(CELL);
                self.write16(TABLE_AC00.wrapping_add(2).wrapping_add(u32::from(index)), cell);
            }
            let cell = self.read16(CELL);
            self.write16(r.a6 + TASK_CELL, cell);
            r.a0 = self.read32(CELL_AT);
            let byte = self.read8(r.a0);
            r.d0 = low_byte(r.d0, byte);
            if byte < 7 {
                r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
                r.a5 = self.read32(SLOT_ADDR);
                let count = self.read8(r.a6 + TASK_COUNT);
                if count != 0 {
                    let left = count - 1;
                    self.write8(r.a6 + TASK_COUNT, left);
                    if left == 0 {
                        self.r_001278();
                        self.r_0052f0(r.d0);
                    }
                }
                r.a0 = self.r_0019ac();
                self.write8(r.a5 + SPRITE_FRAME, MOVING_FRAME);
                (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
                return;
            }
        }
        // the cell taken out of the tables
        let at = self.r_0016b0();
        self.write8(at, 0);
        let at = self.read32(TABLE_1A0_AT);
        self.write8(at, 0);
        r.a0 = at;
        let c;
        (r.d0, r.d2, r.d5, r.d6, r.a5, r.a6, r.x, c) = self.r_0065b6(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        if !c {
            let at = self.read32(TABLE_0_AT);
            self.write8(at, 7);
            r.a0 = self.r_001998();
        }
        self.e_cell_task_ends(r);
    }

    /// The task ends (`func_00121A`) on the family's registers.
    fn e_cell_task_ends(&mut self, r: &mut Regs) {
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
    }

    /// `func_00560A` when the task can't move: stuck for `STUCK_FRAMES`
    /// frames (`STUCK` from now on, run at once).
    fn e_cell_task_stuck(&mut self, r: &mut Regs) {
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        self.write8(r.a5 + SPRITE_FRAME, MOVING_FRAME);
        r.d0 = self.r_0015f8(r.a6);
        (r.a0, r.x) = self.r_00167a();
        r.a0 = self.r_0019ac();
        self.write8(r.a6 + TASK_COUNT, STUCK_FRAMES);
        (r.x, r.c) = self.r_00053e(r.d6, STUCK);
        self.r_0056ec(r);
    }

    /// `func_0056EC`, the task's routine once stuck: while its cell's
    /// byte is 2 it shakes (its sprite a pixel off by the count, from
    /// `STEP_X`, `STEP_Y`) for its count of frames; then the cell is
    /// drawn with `REST_TILES` and, if the next cell in its direction
    /// is a 2 with no such task, one is made there (`func_0055A0`).
    /// The task ends then, or when its cell is not a 2.
    /// No script calls it: reviewed by reading.
    pub fn r_0056ec(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = self.r_0015f8(r.a6);
        (r.a0, r.x) = self.r_00167a();
        let byte = self.read8(r.a0);
        r.d0 = low_byte(r.d0, byte);
        if byte != 2 {
            return self.e_cell_task_ends(r);
        }
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        let left = self.read8(r.a6 + TASK_COUNT).wrapping_sub(1);
        self.write8(r.a6 + TASK_COUNT, left);
        if left != 0 {
            let k = u16::from(left) & 3;
            r.d0 = low_word(r.d0, k);
            let dx = self.read8(STEP_X + u32::from(k));
            let x = self.read8(r.a5 + SPRITE_X_LOW).wrapping_add(dx);
            self.write8(r.a5 + SPRITE_X_LOW, x);
            let dy = self.read8(STEP_Y + u32::from(k));
            r.d1 = low_byte(r.d1, dy);
            let y = self.read8(r.a5 + SPRITE_Y_LOW);
            let (y, carry) = y.overflowing_add(dy);
            self.write8(r.a5 + SPRITE_Y_LOW, y);
            r.x = carry;
            self.write8(r.a5 + SPRITE_FRAME, MOVING_FRAME);
            r.c = false;
            return;
        }
        self.r_001278();
        r.d0 = low_word(r.d0, REST_TILES);
        (r.x, r.c) = self.r_005294(r.d0, r.d1);
        let direction = self.read8(r.a6 + TASK_DIRECTION);
        self.write8(DIRECTION, direction);
        r.d0 = low_word(r.d0, direction as i8 as i16 as u16);
        let at = (direction as i8 as i16 as u16) as i16 as u32;
        let dx = self.read8(STEP_X.wrapping_add(at));
        let column = self.read8(CELL_X).wrapping_add(dx);
        self.write8(CELL_X, column);
        let dy = self.read8(STEP_Y.wrapping_add(at));
        r.d1 = low_byte(r.d1, dy);
        let row = self.read8(CELL_Y).wrapping_add(dy);
        self.write8(CELL_Y, row);
        (r.a0, r.x) = self.r_00167a();
        let byte = self.read8(r.a0);
        r.d0 = low_byte(r.d0, byte);
        if byte == 2 {
            let there;
            (r.x, there) = self.r_005516(r.d0);
            r.c = there;
            if !there {
                (r.d0, r.d1, r.d5, r.d6, r.a5, r.a6, r.x) = self.r_0055a0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            }
        }
        self.e_cell_task_ends(r);
    }
}
