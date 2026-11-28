//! The two tasks `func_01908A` makes (`yc_screen.rs`):
//!
//! - `func_0194AA`-`func_01953C`: five times 0x36 frames and the Z80
//!   command `SOUND`, then it ends;
//! - `func_0193BA`-`func_019484`: over the map's cells (columns 3-0xD,
//!   rows 2-0xA, every other) a task `func_01954E` a frame, a row at a
//!   time with 0x30 frames between rows.  Each such task has two
//!   sprites and comes down from `TASK_DROP` above its place
//!   (`func_01960E`), bounces (`func_01956A`), flickers 8 frames
//!   (`func_0195A6`) and ends with its cell drawn and emptied
//!   (`func_0195C2`).

use crate::{Game, Regs};

/// The index of the task running (`task.rs`).
const CURRENT_TASK: u32 = 0xFFFF_80EE;

/// A map cell (column, row; `anim.rs`).
const CELL_X: u32 = 0xFFFF_9F08;
const CELL_Y: u32 = 0xFFFF_9F09;
/// The sprite record kept (index, address; `func_01960E` moves it).
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// `func_0193BA`'s task's fields: the column and row counted (0-5,
/// 0-4).
const TASK_COLUMN: u32 = 0x0D;
const TASK_ROW: u32 = 0x0E;
const COLUMNS: u8 = 6;
const ROWS: u8 = 5;

/// `func_01954E`'s task's fields: its first and second sprite records'
/// indexes, its place (x, y; pixels), the height it is above it and
/// what that changes by each frame.
const TASK_SPRITE: u32 = 0x03;
const TASK_SPRITE_2: u32 = 0x0C;
const TASK_X: u32 = 0x0D;
const TASK_Y: u32 = 0x0E;
const TASK_DROP: u32 = 0x0F;
const TASK_STEP: u32 = 0x10;
/// The height it starts at, what it falls by each frame, the step it
/// bounces with.
const DROP_START: u8 = 0xC8;
const DROP_FALL: u8 = 4;
const BOUNCE: u8 = 0x0C;
/// The cell's y in pixels: row x 16 + `CELL_TOP`.
const CELL_TOP: u8 = 0x19;

/// `func_01954E`'s task: routine and priority, and its sprites
/// (`func_000796`): the first's depth is made from its y, the
/// second's is `SECOND_DEPTH`; frames, flags and the first's y.
const SQUARE_TASK: u32 = 0x01954E;
const SQUARE_PRIORITY: u8 = 0x32;
const SECOND_DEPTH: u8 = 0xFB;
const FIRST_FRAME: u8 = 0xFF;
const SECOND_FRAME: u8 = 0xE0;
const SPRITE_FLAGS_84: u8 = 0x84;
const FIRST_Y: u16 = 0xFFF6;

/// A sprite record's fields (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;

/// The Z80 command `func_0194AA` sends (`func_0034B6`), and the frames
/// between.
const SOUND: u8 = 0x64;
const SOUND_WAIT: u16 = 0x36;
/// The frames between rows and the flicker's.
const ROW_WAIT: u16 = 0x30;
const FLICKER_WAIT: u16 = 8;

/// Where the tasks go on (the yields' resume addresses, and
/// `func_0193BE` as the routine again).
const SOUND_2: u32 = 0x0194BC;
const SOUND_3: u32 = 0x0194DC;
const SOUND_4: u32 = 0x0194FC;
const SOUND_5: u32 = 0x01951C;
const SOUND_LAST: u32 = 0x01953C;
const ROW_START: u32 = 0x0193BE;
const NEXT_SQUARE: u32 = 0x0193C6;
const NEXT_ROW: u32 = 0x019484;
const BOUNCING: u32 = 0x01956A;
const FLICKERING: u32 = 0x0195A6;
const LANDED: u32 = 0x0195C2;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// After the wait (`func_01793A`), the command `SOUND` and
    /// `SOUND_WAIT` frames more, to `resume`.
    fn yc_sound(&mut self, r: &mut Regs, resume: u32) {
        if self.yc_waiting(r) {
            return;
        }
        r.d0 = low_byte(r.d0, SOUND);
        self.r_0034b6(r.d0);
        self.yc_wait(r, SOUND_WAIT, resume);
    }

    /// `func_0194AA` (a task's routine): `SOUND_WAIT` frames.
    pub fn r_0194aa(&mut self, regs: &mut Regs) {
        self.yc_wait(regs, SOUND_WAIT, SOUND_2);
    }

    /// `func_0194BC` (a task's routine): the first command.
    pub fn r_0194bc(&mut self, regs: &mut Regs) {
        self.yc_sound(regs, SOUND_3);
    }

    /// `func_0194DC` (a task's routine): the second command.
    pub fn r_0194dc(&mut self, regs: &mut Regs) {
        self.yc_sound(regs, SOUND_4);
    }

    /// `func_0194FC` (a task's routine): the third command.
    pub fn r_0194fc(&mut self, regs: &mut Regs) {
        self.yc_sound(regs, SOUND_5);
    }

    /// `func_01951C` (a task's routine): the fourth command.
    pub fn r_01951c(&mut self, regs: &mut Regs) {
        self.yc_sound(regs, SOUND_LAST);
    }

    /// `func_01953C` (a task's routine): after the wait, the fifth
    /// command and the task ends (`func_000560`).
    pub fn r_01953c(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.yc_waiting(r) {
            return;
        }
        r.d0 = low_byte(r.d0, SOUND);
        self.r_0034b6(r.d0);
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }

    /// `func_0193BA` (a task's routine): the row cleared, and on
    /// (`func_0193BE`).
    pub fn r_0193ba(&mut self, regs: &mut Regs) {
        self.write8(regs.a6 + TASK_ROW, 0);
        regs.c = false;
        self.r_0193be(regs);
    }

    /// `func_0193BE` (a task's routine, a row's start): the column
    /// cleared and a frame on (`func_0193C6`).
    pub fn r_0193be(&mut self, regs: &mut Regs) {
        self.write8(regs.a6 + TASK_COLUMN, 0);
        self.yc_yield(regs, NEXT_SQUARE);
    }

    /// `func_0193C6` (a task's routine): a task `SQUARE_TASK`
    /// (`func_0003E8`) at the cell of the column and row (x 2, + 3 and
    /// + 2; in `CELL_X`, `CELL_Y`), with two sprites (`func_000796`),
    /// at its place (x: column x 16; y: row x 16 + `CELL_TOP`) and
    /// `DROP_START` above it; the column counted, and after the last
    /// `ROW_WAIT` frames and the next row (`func_019484`).
    pub fn r_0193c6(&mut self, regs: &mut Regs) {
        let r = regs;
        let column = self.read8(r.a6 + TASK_COLUMN);
        let cell_x = column.wrapping_add(column).wrapping_add(3);
        r.d0 = low_byte(r.d0, cell_x);
        self.write8(CELL_X, cell_x);
        let row = self.read8(r.a6 + TASK_ROW);
        let cell_y = row.wrapping_add(row).wrapping_add(2);
        r.d0 = low_byte(r.d0, cell_y);
        self.write8(CELL_Y, cell_y);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, SQUARE_PRIORITY, SQUARE_TASK);
        // the first sprite's depth: !(y + CELL_TOP) & 0xFE | 2
        let (y, carry) = (self.read8(CELL_Y) << 4).overflowing_add(CELL_TOP);
        r.x = carry;
        r.d0 = low_byte(r.d0, (!y & 0xFE) | 2);
        (r.d5, r.a5, r.x, r.c) = self.r_000796(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
        self.write8(r.a6 + TASK_SPRITE, r.d5 as u8);
        self.write8(r.a5 + SPRITE_FRAME, FIRST_FRAME);
        self.write8(r.a5 + SPRITE_FLAGS, SPRITE_FLAGS_84);
        self.write16(r.a5 + SPRITE_Y, FIRST_Y);
        r.d0 = low_byte(r.d0, SECOND_DEPTH);
        (r.d5, r.a5, r.x, r.c) = self.r_000796(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
        self.write8(r.a6 + TASK_SPRITE_2, r.d5 as u8);
        self.write8(r.a5 + SPRITE_FRAME, SECOND_FRAME);
        self.write8(r.a5 + SPRITE_FLAGS, SPRITE_FLAGS_84);
        self.write8(r.a6 + TASK_DROP, DROP_START);
        let x = self.read8(CELL_X) << 4;
        r.d0 = low_byte(r.d0, x);
        self.write8(r.a6 + TASK_X, x);
        self.write8(r.a5 + SPRITE_X_LOW, x);
        let y = (self.read8(CELL_Y) << 4).wrapping_add(CELL_TOP);
        r.d0 = low_byte(r.d0, y);
        self.write8(r.a6 + TASK_Y, y);
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        let (column, carry) = self.read8(r.a6 + TASK_COLUMN).overflowing_add(1);
        self.write8(r.a6 + TASK_COLUMN, column);
        r.x = carry;
        r.unwound = false;
        if column != COLUMNS {
            r.c = column < COLUMNS;
            return;
        }
        self.yc_wait(r, ROW_WAIT, NEXT_ROW);
    }

    /// `func_019484` (a task's routine): after the wait, the routine
    /// `func_0193BE` again (`func_00053E`) and the row counted; after the
    /// last the task ends (`func_000560`).
    pub fn r_019484(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.yc_waiting(r) {
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, ROW_START);
        let (row, carry) = self.read8(r.a6 + TASK_ROW).overflowing_add(1);
        self.write8(r.a6 + TASK_ROW, row);
        r.x = carry;
        if row != ROWS {
            r.c = row < ROWS;
            return;
        }
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }

    /// `func_01954E` (a task's routine): while `TASK_DROP` isn't 0 it
    /// goes down by `DROP_FALL` and the sprites are placed by it
    /// (`func_01960E`); at 0, `TASK_STEP` `BOUNCE` and a frame on
    /// (`func_01956A`).
    pub fn r_01954e(&mut self, regs: &mut Regs) {
        let r = regs;
        let drop = self.read8(r.a6 + TASK_DROP);
        r.d0 = low_byte(r.d0, drop);
        if drop == 0 {
            self.write8(r.a6 + TASK_STEP, BOUNCE);
            self.yc_yield(r, BOUNCING);
            return;
        }
        let drop = drop.wrapping_sub(DROP_FALL);
        r.d0 = low_byte(r.d0, drop);
        self.write8(r.a6 + TASK_DROP, drop);
        (r.d0, r.d1, r.d5, r.a5, r.x, r.c) = self.r_01960e(r.d0, r.d1, r.d5, r.a6);
        r.unwound = false;
    }

    /// `func_01956A` (a task's routine): `TASK_STEP` added to
    /// `TASK_DROP` and the sprites placed by a quarter of it
    /// (`func_01960E`); the step goes down by 1, until the drop is back
    /// to 0: then `FLICKER_WAIT` frames (`func_0195A6`).
    pub fn r_01956a(&mut self, regs: &mut Regs) {
        let r = regs;
        let drop = self.read8(r.a6 + TASK_DROP).wrapping_add(self.read8(r.a6 + TASK_STEP));
        self.write8(r.a6 + TASK_DROP, drop);
        r.d0 = low_byte(r.d0, drop >> 2);
        (r.d0, r.d1, r.d5, r.a5, r.x, r.c) = self.r_01960e(r.d0, r.d1, r.d5, r.a6);
        r.unwound = false;
        if drop == 0 {
            self.yc_wait(r, FLICKER_WAIT, FLICKERING);
            return;
        }
        let (step, borrow) = self.read8(r.a6 + TASK_STEP).overflowing_sub(1);
        self.write8(r.a6 + TASK_STEP, step);
        r.x = borrow;
        r.c = borrow;
    }

    /// `func_0195A6` (a task's routine): `TASK_STEP` counted and the
    /// sprites placed by its bit 0 (`func_01960E`); after the wait, a
    /// frame on (`func_0195C2`).
    pub fn r_0195a6(&mut self, regs: &mut Regs) {
        let r = regs;
        let step = self.read8(r.a6 + TASK_STEP).wrapping_add(1);
        self.write8(r.a6 + TASK_STEP, step);
        r.d0 = low_byte(r.d0, step & 1);
        (r.d0, r.d1, r.d5, r.a5, r.x, r.c) = self.r_01960e(r.d0, r.d1, r.d5, r.a6);
        if self.yc_waiting(r) {
            return;
        }
        self.yc_yield(r, LANDED);
    }

    /// `func_0195C2` (a task's routine): the cell under the task's place
    /// (`CELL_X`, `CELL_Y`) drawn and emptied (`func_0195FC`), the
    /// second sprite freed (`func_000870`), the first kept in
    /// `SLOT_INDEX`, `SLOT_ADDR` (`func_000B5A`), and the task ends
    /// (`func_00121A`).
    pub fn r_0195c2(&mut self, regs: &mut Regs) {
        let r = regs;
        let cell_x = self.read8(r.a6 + TASK_X) >> 4;
        r.d0 = low_byte(r.d0, cell_x);
        self.write8(CELL_X, cell_x);
        let cell_y = self.read8(r.a6 + TASK_Y).wrapping_sub(CELL_TOP) >> 4;
        r.d0 = low_byte(r.d0, cell_y);
        self.write8(CELL_Y, cell_y);
        (r.d0, r.a0) = self.r_0195fc();
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_SPRITE_2));
        r.x = self.r_000870(r.d0, r.d5);
        r.d5 = low_byte(r.d5, self.read8(r.a6 + TASK_SPRITE));
        (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
        self.write8(SLOT_INDEX, r.d5 as u8);
        self.write32(SLOT_ADDR, r.a5);
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        r.unwound = false;
    }
}
