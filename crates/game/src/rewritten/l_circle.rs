//! The circle of 12 sprites around object 0 (`l_sequence.rs` leads to
//! it): `func_00C292` rises over object 0, `func_00C2F2` waits for
//! `STAGE` 8 and makes the sprites, then they spin out
//! (`func_00C380`), in (`func_00C3B6`) and with a growing square of
//! cells (`func_00C3E4`, `func_0053EC`) until they are freed with
//! objects 6-19's sprites (`func_00C496`); each frame `func_00C5AE`
//! places them by their angles.

use crate::{Game, Regs};

/// A task's fields: its sprite's index; its position (words; the x's
/// low byte), a count, a second count, its rate (a word) and its offset
/// (the high byte: the sprite drawn so many pixels up; `a_motion.rs`).
const TASK_SPRITE: u32 = 0x03;
const TASK_X: u32 = 0x0E;
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y: u32 = 0x10;
const TASK_COUNT: u32 = 0x16;
const TASK_COUNT_2: u32 = 0x17;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;
/// A sprite record's frame, flags and position (words).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_Y: u32 = 0x0A;
/// An object's fields: in use (5 when it is in a cell, `func_00C496`),
/// its sprite record (address, index).
const OBJ_USED: u32 = 0x00;
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;

/// The running task's index.
const CURRENT_TASK: u32 = 0xFF80EE;
/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// A byte whose bit 4 paces `func_00C292` and whose bit 1 turns the
/// sprites further (`func_00C5AE`).
const BYTE_80D8: u32 = 0xFF80D8;
/// Object 0's routine (a long) and object 5's, its position (words),
/// its byte 0x32 (negative: the circle lower), its sprite record
/// (address, index).
const OBJECT_0_ROUTINE: u32 = 0xFFA434;
const OBJECT_5_ROUTINE: u32 = 0xFFA574;
const OBJECT_0_X: u32 = 0xFFA44A;
const OBJECT_0_Y: u32 = 0xFFA44C;
const OBJECT_0_32: u32 = 0xFFA464;
const OBJECT_0_SPRITE_ADDR: u32 = 0xFFA43C;
const OBJECT_0_SPRITE: u32 = 0xFFA440;
/// The routine objects 0 and 5 get.
const OBJECT_ROUTINE: u32 = 0x0032AE;
/// A velocity's y (a word; `a_motion.rs`).
const VELOCITY_Y: u32 = 0xFF9EC6;
/// A stage `func_00C2F2` waits for 8 in (set to 2 by `func_00C292`);
/// its half picks the offset (`OFFSETS`).
const STAGE: u32 = 0xFFA457;
const OFFSETS: u32 = 0x00C313;
/// The sprites: their indexes (0xFF: none), angles, the turn a frame,
/// how far out, and a count (also the number being made).
const SPRITES: u32 = 0xFFFF_AC9E;
const ANGLES: u32 = 0xFFFF_ACBE;
const TURN: u32 = 0xFFAC8A;
const RADIUS: u32 = 0xFFAC8D;
const LEFT: u32 = 0xFFAC8B;
const COUNT: u8 = 12;
/// The sprites' depth, and the angle between them.
const DEPTH: u32 = 0x10;
const APART: u8 = 0x15;
/// The cell (column, row) and the map's size; the cells
/// `func_00C4DA` looks through (`l_sequence.rs`).
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
const WIDTH: u32 = 0xFF9EB2;
const HEIGHT: u32 = 0xFF9EB3;
const LOOK_AT: u32 = 0xFFAC96;
const LOOK_LAST: u32 = 0xFFAC9A;
/// Flags: bit 6 cleared at the circle's end.
const FLAGS: u32 = 0xFF9540;
/// Set at the circle's end (0, 0xF, 0, 0x50).
const BYTE_A426: u32 = 0xFFA426;
const BYTE_A427: u32 = 0xFFA427;
const BYTE_A428: u32 = 0xFFA428;
const BYTE_A429: u32 = 0xFFA429;
/// The objects whose sprites are freed (`func_00C496`).
const FIRST_FREED: u8 = 6;
const OBJECTS: u8 = 0x14;
const IN_CELL: u8 = 5;

/// The routines the task goes on with.
const ROUTINE_C2F2: u32 = 0x00C2F2;
const ROUTINE_C380: u32 = 0x00C380;
const ROUTINE_C3B6: u32 = 0x00C3B6;
const ROUTINE_C3E4: u32 = 0x00C3E4;
const ROUTINE_C4DA: u32 = 0x00C4DA;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_00C292`, a task's routine: on frames with bit 4 of
    /// `BYTE_80D8` set it rises (its offset up by 1); at 0x18 objects 0
    /// and 5 get `OBJECT_ROUTINE`, the task goes to object 0's position
    /// (raised 11 pixels unless `OBJECT_0_32` is negative), to the left
    /// of its cell, `STAGE` is 2 and it goes on as `func_00C2F2`.
    /// Otherwise its sprite is placed (`func_00C310`).
    pub fn r_00c292(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(BYTE_80D8) & 0x10 == 0 {
            (r.d0, r.x, r.c) = self.r_00c310(r.a5, r.a6);
            return;
        }
        let offset = self.read8(r.a6 + TASK_OFFSET).wrapping_add(1);
        self.write8(r.a6 + TASK_OFFSET, offset);
        if offset != 0x18 {
            (r.d0, r.x, r.c) = self.r_00c310(r.a5, r.a6);
            return;
        }
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        r.d0 = OBJECT_ROUTINE;
        self.write32(OBJECT_0_ROUTINE, OBJECT_ROUTINE);
        self.write32(OBJECT_5_ROUTINE, OBJECT_ROUTINE);
        let x = self.read16(OBJECT_0_X);
        self.write16(r.a6 + TASK_X, x);
        let y = self.read16(OBJECT_0_Y);
        self.write16(r.a6 + TASK_Y, y);
        if (self.read8(OBJECT_0_32) as i8) >= 0 {
            self.write16(VELOCITY_Y, 0xF500);
            (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        }
        let low = self.read8(r.a6 + TASK_X_LOW);
        self.write8(r.a6 + TASK_X_LOW, low & 0xF0);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C2F2);
        self.write8(STAGE, 2);
        r.c = false;
        self.r_00c2f2(r);
    }

    /// `func_00C2F2`, a task's routine: until `STAGE` is 8 its offset
    /// is `OFFSETS` by half the stage and its sprite placed
    /// (`func_00C310`).  Then it goes on as `func_00C380` (count 0x30,
    /// rate and offset 0) and makes the `COUNT` sprites (frame 0,
    /// flags 4, at `DEPTH`; their angles `APART` apart), and the Z80
    /// gets 0x0B.
    pub fn r_00c2f2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let stage = self.read8(STAGE);
        r.d0 = low_byte(r.d0, stage);
        if stage != 8 {
            let half = stage >> 1;
            r.d0 = low_word(r.d0, u16::from(half));
            let offset = self.read8(OFFSETS + u32::from(half));
            self.write8(r.a6 + TASK_OFFSET, offset);
            r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
            r.a5 = self.read32(SLOT_ADDR);
            (r.d0, r.x, r.c) = self.r_00c310(r.a5, r.a6);
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x30);
        self.write16(r.a6 + TASK_RATE, 0);
        self.write8(r.a6 + TASK_OFFSET, 0);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C380);
        r.a0 = SPRITES;
        r.a1 = ANGLES;
        self.write8(LEFT, COUNT);
        self.write8(TURN, 0);
        loop {
            self.write8(r.a0, 0xFF);
            r.d0 = DEPTH;
            let failed;
            (r.d5, r.a5, r.x, failed) = self.r_000796(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
            if !failed {
                self.write8(r.a0, r.d5 as u8);
                self.write8(r.a5 + SPRITE_FRAME, 0);
                self.write8(r.a5 + SPRITE_FLAGS, 4);
                let angle = self.read8(TURN);
                self.write8(r.a1, angle);
            }
            r.a0 = r.a0.wrapping_add(1);
            r.a1 = r.a1.wrapping_add(1);
            let angle = self.read8(TURN);
            self.write8(TURN, angle.wrapping_add(APART));
            let left = self.read8(LEFT);
            self.write8(LEFT, left.wrapping_sub(1));
            r.x = left == 0;
            if left == 1 {
                break;
            }
        }
        r.d0 = low_byte(r.d0, 0x0B);
        r.c = false;
        self.r_0034b6(r.d0);
    }

    /// `func_00C310` (`.L00C310`): `func_001B94`.  Gives back d0, X and
    /// C.
    pub fn r_00c310(&mut self, a5: u32, a6: u32) -> (u32, bool, bool) {
        self.r_001b94(a5, a6)
    }

    /// `func_00C380`, a task's routine: after 0x30 frames it goes on as
    /// `func_00C3B6` (count 0x30); it falls (`func_00C742` with 0x10,
    /// `func_001ACA`), and the sprites are out as far as its offset,
    /// turning 1 a frame (`func_00C5AE`).
    pub fn r_00c380(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let left = self.read8(r.a6 + TASK_COUNT).wrapping_sub(1);
        self.write8(r.a6 + TASK_COUNT, left);
        if left == 0 {
            self.write8(r.a6 + TASK_COUNT, 0x30);
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C3B6);
        }
        r.d0 = 0x10;
        self.r_00c742(r.d0, r.a6);
        (r.d0, r.x, _) = self.r_001aca(r.d0, r.a6);
        let offset = self.read8(r.a6 + TASK_OFFSET);
        self.write8(RADIUS, offset);
        self.write8(TURN, 1);
        self.l_circle(r);
    }

    /// `func_00C5AE` on the family's registers.
    fn l_circle(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.a0, r.a1, r.a4, r.a5, r.x, r.c) = self.r_00c5ae(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.a4, r.a5, r.a6);
    }

    /// `func_00C3B6`, a task's routine: after 0x30 frames it goes on as
    /// `func_00C3E4` (count 0x20); its offset comes down by 1 and the
    /// sprites are out as far as it, turning back 1 a frame
    /// (`func_00C5AE`).
    pub fn r_00c3b6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let left = self.read8(r.a6 + TASK_COUNT).wrapping_sub(1);
        self.write8(r.a6 + TASK_COUNT, left);
        if left == 0 {
            self.write8(r.a6 + TASK_COUNT, 0x20);
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C3E4);
        }
        let offset = self.read8(r.a6 + TASK_OFFSET).wrapping_sub(1);
        self.write8(r.a6 + TASK_OFFSET, offset);
        self.write8(RADIUS, offset);
        self.write8(TURN, 0xFF);
        self.l_circle(r);
    }

    /// `func_00C3E4`, a task's routine: its offset goes up by 6 a frame
    /// and the cells within an eighth-of-32 of it around object 0 are
    /// turned (`func_0053EC`), the sprites out as far as it, not turning
    /// (`func_00C5AE`).  When it comes round past 0xFF: the sprites,
    /// objects 6-19's (`func_00C496`) and its own freed, the cells
    /// within 0x20 turned, the bytes `BYTE_A426`-`BYTE_A429` set,
    /// `func_00ADF2`, and it goes on as `func_00C4DA` with the whole
    /// map to look through (`LOOK_AT`, `LOOK_LAST`); bit 6 of `FLAGS`
    /// cleared.
    pub fn r_00c3e4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let was = self.read8(r.a6 + TASK_OFFSET);
        let offset = was.wrapping_add(6);
        self.write8(r.a6 + TASK_OFFSET, offset);
        if offset >= was {
            r.d0 = low_byte(r.d0, offset >> 5);
            r.a6 = self.r_0053ec(r.d0, r.d4, r.a4, r.a6);
            let offset = self.read8(r.a6 + TASK_OFFSET);
            self.write8(RADIUS, offset);
            self.write8(TURN, 0);
            r.c = false;
            self.l_circle(r);
            return;
        }
        r.d2 = low_word(r.d2, 0);
        loop {
            r.a0 = SPRITES;
            let sprite = self.read8(SPRITES.wrapping_add(r.d2 as u16 as i16 as u32));
            r.d0 = low_byte(r.d0, sprite);
            if (sprite as i8) >= 0 {
                r.x = self.r_000870(r.d0, r.d5);
            }
            r.d2 = low_word(r.d2, (r.d2 as u16).wrapping_add(1));
            if r.d2 as u8 == COUNT {
                break;
            }
        }
        (r.d0, r.d4, r.d5, r.a0, r.a4, r.a5) = self.r_00c496(r.d0, r.d5, r.a0);
        r.d0 = low_byte(r.d0, 0x20);
        r.a6 = self.r_0053ec(r.d0, r.d4, r.a4, r.a6);
        self.write8(BYTE_A429, 0);
        self.write8(BYTE_A428, 0x0F);
        self.write8(BYTE_A427, 0);
        self.write8(BYTE_A426, 0x50);
        (r.d0, r.d5, r.d6, r.a5, r.a6) = self.r_00adf2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
        let sprite = self.read8(r.a6 + TASK_SPRITE);
        r.d0 = low_byte(r.d0, sprite);
        r.x = self.r_000870(r.d0, r.d5);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_C4DA);
        self.write8(CELL_Y, 0);
        self.write8(CELL_X, 0);
        (r.a0, r.x) = self.r_00167a();
        self.write32(LOOK_AT, r.a0);
        let width = self.read8(WIDTH);
        self.write8(CELL_X, width.wrapping_sub(1));
        let height = self.read8(HEIGHT);
        self.write8(CELL_Y, height.wrapping_sub(1));
        (r.a0, r.x) = self.r_00167a();
        self.write32(LOOK_LAST, r.a0);
        let flags = self.read8(FLAGS);
        self.write8(FLAGS, flags & 0xBF);
        r.c = false;
    }

    /// `func_00C496`: objects 6-19 (`func_001CFA`) in use are taken out:
    /// their sprite record made the routine's (`SLOT_INDEX`,
    /// `SLOT_ADDR`) and freed (`func_000890`), and for one in a cell
    /// (`IN_CELL`) the cell's byte made again from its source
    /// (`func_00168C`).  Gives back d0, d4, d5, a0, a4 and a5.
    pub fn r_00c496(&mut self, d0: u32, d5: u32, a0: u32) -> (u32, u32, u32, u32, u32, u32) {
        let (mut d0, mut d5, mut a0) = (d0, d5, a0);
        let mut d4: u32 = u32::from(FIRST_FREED);
        let mut a4;
        let mut a5;
        loop {
            (d4, a4, _) = self.r_001cfa(d4);
            d5 = low_byte(d5, self.read8(a4 + OBJ_SPRITE));
            a5 = self.read32(a4 + OBJ_SPRITE_ADDR);
            self.write8(SLOT_INDEX, d5 as u8);
            self.write32(SLOT_ADDR, a5);
            let used = self.read8(a4 + OBJ_USED);
            d0 = low_byte(d0, used);
            if used != 0 {
                self.write8(a4 + OBJ_USED, 0);
                if used == IN_CELL {
                    (d0, _) = self.r_0015e6(a4);
                    self.r_00167a();
                    (d0, a0) = self.r_00168c(d0);
                }
                self.r_000890();
            }
            d4 = low_byte(d4, (d4 as u8).wrapping_add(1));
            if d4 as u8 == OBJECTS {
                return (d0, d4, d5, a0, a4, a5);
            }
        }
    }

    /// `func_00C5AE` (`.L00C5AE`): the task's second count stepped (the
    /// sprites' frame 0x22-0x25 by it) and the `COUNT` sprites each
    /// turned by `TURN` (and 10 more on frames with bit 1 of
    /// `BYTE_80D8`) and placed `RADIUS` out from object 0's sprite at
    /// their angle (`func_01777C` for y, `func_00C66C`; `func_017780`
    /// for x); they show only from radius 0x12, but for the first two.
    /// Gives back d0, d1, d2, d3, d4, d5, a0, a1, a4, a5, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00c5ae(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (mut d0, mut d1, mut d2, mut d3, mut d4, mut d5, mut a4, mut a5) = (d0, d1, d2, d3, d4, d5, a4, a5);
        self.write8(LEFT, COUNT);
        let step = self.read8(a6 + TASK_COUNT_2).wrapping_add(1);
        self.write8(a6 + TASK_COUNT_2, step);
        d3 = low_byte(d3, ((step >> 1) & 3) + 0x22);
        d2 = low_word(d2, 0);
        loop {
            let i = (d2 as u16 as i16) as u32;
            let sprite = self.read8(SPRITES.wrapping_add(i));
            d5 = low_byte(d5, sprite);
            if (sprite as i8) >= 0 {
                (d5, a5, _) = self.r_000b5a(d5);
                self.write8(SLOT_INDEX, d5 as u8);
                self.write32(SLOT_ADDR, a5);
                let angle = self.read8(ANGLES.wrapping_add(i)).wrapping_add(self.read8(TURN));
                d0 = low_byte(d0, angle);
                self.write8(ANGLES.wrapping_add(i), angle);
                if self.read8(BYTE_80D8) & 0x02 != 0 {
                    d0 = low_byte(d0, angle.wrapping_add(10));
                }
                let kept = d0;
                d1 = low_byte(d1, self.read8(RADIUS));
                d0 = self.r_01777c(d0, d1, d3);
                (d4, a4, d5, a5) = self.l_sprites(d4, d5);
                self.r_00c66c(d0, a4, a5);
                d0 = kept;
                d1 = low_byte(d1, self.read8(RADIUS));
                d0 = self.r_017780(d0, d1, d3);
                (d4, a4, d5, a5) = self.l_sprites(d4, d5);
                let x = (d0 as u16).wrapping_add(self.read16(a5 + SPRITE_X));
                d0 = low_word(d0, x);
                self.write16(a4 + SPRITE_X, x);
                self.write8(a4 + SPRITE_FRAME, 0);
                if self.read8(RADIUS) >= 0x12 || d2 as u8 >= 2 {
                    self.write8(a4 + SPRITE_FRAME, d3 as u8);
                }
            }
            d2 = low_byte(d2, (d2 as u8).wrapping_add(1));
            let left = self.read8(LEFT);
            self.write8(LEFT, left.wrapping_sub(1));
            if left == 1 {
                let borrow = left == 0;
                return (d0, d1, d2, d3, d4, d5, SPRITES, ANGLES, a4, a5, borrow, borrow);
            }
        }
    }

    /// The routine's sprite record (d4, a4) and object 0's (d5, a5).
    fn l_sprites(&mut self, d4: u32, d5: u32) -> (u32, u32, u32, u32) {
        let d4 = low_byte(d4, self.read8(SLOT_INDEX));
        let a4 = self.read32(SLOT_ADDR);
        let d5 = low_byte(d5, self.read8(OBJECT_0_SPRITE));
        let a5 = self.read32(OBJECT_0_SPRITE_ADDR);
        (d4, a4, d5, a5)
    }

    /// `func_00C66C`: the sprite at a4's y that of the sprite at a5 plus
    /// `d0`, less 0x22 (0x14 when `OBJECT_0_32` is negative).
    pub fn r_00c66c(&mut self, d0: u32, a4: u32, a5: u32) {
        let y = (d0 as u16).wrapping_add(self.read16(a5 + SPRITE_Y));
        self.write16(a4 + SPRITE_Y, y);
        let up = if (self.read8(OBJECT_0_32) as i8) < 0 { 0x14 } else { 0x22 };
        self.write16(a4 + SPRITE_Y, y.wrapping_sub(up));
    }
}
