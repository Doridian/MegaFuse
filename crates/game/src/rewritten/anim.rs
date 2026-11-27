//! Objects and their animations.
//!
//! An animation is a list of byte pairs, (how many frames, the sprite
//! frame), ended by 0 (it stops) or 0xFF (it starts again).  Two kinds
//! of record run one:
//!
//! - an object (a4): its sprite record's index and address at
//!   `OBJ_SPRITE`, `OBJ_SPRITE_ADDR`, a table of animations at
//!   `OBJ_ANIMS` (by direction and kind), the one running at `OBJ_ANIM`
//!   (`func_001336`);
//! - a task (a6): its sprite record's index in the task's byte 3, the
//!   animation at `TASK_ANIM` (`func_0013D6`).

use crate::Game;

/// An object's fields.
/// Counts the times it was set up.
const OBJ_COUNT: u32 = 0x00;
/// Set up from `OBJ_INIT_2` (a long).
const OBJ_2: u32 = 0x02;
/// Its sprite record's address (a long) and index.
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
/// Its direction (bits 0-1) and what its animation is by (added).
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x15;
/// Its position (words).
const OBJ_X: u32 = 0x18;
const OBJ_Y: u32 = 0x1A;
/// Its animations (a table of longs) and the one running.
const OBJ_ANIMS: u32 = 0x1C;
const OBJ_ANIM: u32 = 0x20;
/// The animation's frames to go and position (the next pair).
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;

/// A task's fields (beyond `task.rs`'s).
const TASK_SPRITE: u32 = 0x03;
/// A long: a position (words, x then y; `TASK_Y` set apart) and two
/// bytes before it.
const TASK_C: u32 = 0x0C;
const TASK_D: u32 = 0x0D;
const TASK_POS: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_ANIM: u32 = 0x20;

/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const BUDGETED: u8 = 1 << 4;
/// A sprite record's position's high bytes (`func_0013C0`).
const SPRITE_X_HIGH: u32 = 0x08;
const SPRITE_Y_HIGH: u32 = 0x0A;

/// What an object is set up with: a long for `OBJ_2`, its animations;
/// a position (words, x then y; one long).
const OBJ_INIT_2: u32 = 0xFF80D0;
const OBJ_INIT_ANIMS: u32 = 0xFF80C8;
const POSITION: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
/// A map cell (column, row).
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
/// While this isn't 0, `func_001486` finds nothing.
const STOPPED: u32 = 0xFFA3FC;
/// The map's width (in cells).
const WIDTH: u32 = 0xFF9EB2;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_0012A0`: the object at a4 gets a sprite record (at depth
    /// 0x80, `func_000796`; whether it got one isn't looked at), frame
    /// 0, and is set up (`func_0012B6`).  Gives back d0, d5 and a5 (the
    /// sprite record's index and address) and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0012a0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, obj: u32, a5: u32, x: bool) -> (u32, u32, u32, bool) {
        let d0 = low_byte(d0, 0x80);
        let (d5, sprite, _, _) = self.r_000796(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, obj, a5, x);
        self.write8(obj + OBJ_SPRITE, d5 as u8);
        self.write32(obj + OBJ_SPRITE_ADDR, sprite);
        self.write8(sprite + SPRITE_FRAME, 0);
        let x = self.r_0012b6(obj);
        (d0, d5, sprite, x)
    }

    /// `func_0012B6`: the object at `obj` set up: counted, its
    /// animations from `OBJ_INIT_ANIMS`, its state cleared, its position
    /// from `POSITION` (`func_001300`).  Gives back X (the count's
    /// carry).
    pub fn r_0012b6(&mut self, obj: u32) -> bool {
        let count = self.read8(obj + OBJ_COUNT);
        self.write8(obj + OBJ_COUNT, count.wrapping_add(1));
        let init = self.read32(OBJ_INIT_2);
        self.write32(obj + OBJ_2, init);
        let anims = self.read32(OBJ_INIT_ANIMS);
        self.write32(obj + OBJ_ANIMS, anims);
        for field in [0x14, 0x13, 0x15, 0x10, 0x0F, 0x16, 0x17, 0x24, 0x25] {
            self.write8(obj + field, 0);
        }
        for field in [0x26, 0x28, 0x2A] {
            self.write16(obj + field, 0);
        }
        self.write8(obj + 0x12, 0);
        self.write8(obj + 0x11, 1);
        self.r_001300(obj);
        count == 0xFF
    }

    /// `func_001300`: the object's position from `POSITION`.
    pub fn r_001300(&mut self, obj: u32) {
        let x = self.read16(POSITION);
        self.write16(obj + OBJ_X, x);
        let y = self.read16(POSITION_Y);
        self.write16(obj + OBJ_Y, y);
    }

    /// `func_00130E`: the object's animation by its direction and kind
    /// (`func_00132A`).  Gives back a0 (its table of animations), X and
    /// C (clear).
    pub fn r_00130e(&mut self, _d0: u32, obj: u32) -> (u32, bool, bool) {
        let direction = self.read8(obj + OBJ_DIRECTION) & 3;
        let which = direction.wrapping_add(self.read8(obj + OBJ_KIND));
        let a0 = self.r_00132a(u32::from(which) * 4, obj);
        (a0, false, false)
    }

    /// `func_00132A`: the object's animation, the long `d0` (its low
    /// word) into its table.  Gives back a0, the table.
    pub fn r_00132a(&mut self, d0: u32, obj: u32) -> u32 {
        let anims = self.read32(obj + OBJ_ANIMS);
        let anim = self.read32(anims.wrapping_add(d0 as u16 as i16 as u32));
        self.write32(obj + OBJ_ANIM, anim);
        anims
    }

    /// `loc_001334`: an `rts`.
    pub fn r_001334(&mut self) {}

    /// `func_001336`: the object's animation, a frame on: while a
    /// pair's frames last its sprite frame is shown again; then the next
    /// pair's, its sprite record `BUDGETED`.  C set when the animation
    /// ended (its position back to the start).  Gives back d0, d1, d2,
    /// d5 (the sprite record's index), a0 (the animation), a5 (the
    /// sprite record), X, Z and C as the original leaves them.
    pub fn r_001336(&mut self, d0: u32, d1: u32, d2: u32, d5: u32, obj: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, bool, bool, bool) {
        let anim = self.read32(obj + OBJ_ANIM);
        let timer = self.read8(obj + OBJ_TIMER);
        if timer != 0 {
            self.write8(obj + OBJ_TIMER, timer - 1);
            let (step, borrow) = self.read8(obj + OBJ_STEP).overflowing_sub(1);
            let d0 = low_word(d0, u16::from(step));
            let frame = self.read8(anim.wrapping_add(u32::from(step)));
            let d1 = low_byte(d1, frame);
            let d5 = low_byte(d5, self.read8(obj + OBJ_SPRITE));
            let sprite = self.read32(obj + OBJ_SPRITE_ADDR);
            self.write8(sprite + SPRITE_FRAME, frame);
            return (d0, d1, d2, d5, anim, sprite, borrow, frame == 0, false);
        }
        let mut d0 = low_word(d0, u16::from(self.read8(obj + OBJ_STEP)));
        let mut d1 = d1;
        loop {
            let frames = self.read8(anim.wrapping_add(d0 as u16 as i16 as u32));
            d1 = low_byte(d1, frames);
            match frames {
                0 => {
                    self.write8(obj + OBJ_STEP, 0);
                    self.write8(obj + OBJ_TIMER, 0);
                    let index = self.read8(obj + OBJ_SPRITE);
                    let sprite = self.read32(obj + OBJ_SPRITE_ADDR);
                    return (d0, d1, d2, low_byte(d5, index), anim, sprite, x, index == 0, true);
                }
                0xFF => d0 = low_word(d0, 0),
                _ => {
                    self.write8(obj + OBJ_TIMER, frames);
                    d0 = low_word(d0, (d0 as u16).wrapping_add(1));
                    let (next, carry) = (d0 as u8).overflowing_add(1);
                    let d2 = low_byte(d0, next);
                    self.write8(obj + OBJ_STEP, next);
                    let frame = self.read8(anim.wrapping_add(d0 as u16 as i16 as u32));
                    let d1 = low_byte(d1, frame);
                    let d5 = low_byte(d5, self.read8(obj + OBJ_SPRITE));
                    let sprite = self.read32(obj + OBJ_SPRITE_ADDR);
                    self.write8(sprite + SPRITE_FRAME, frame);
                    let flags = self.read8(sprite + SPRITE_FLAGS);
                    self.write8(sprite + SPRITE_FLAGS, flags | BUDGETED);
                    return (low_byte(d0, BUDGETED), d1, d2, d5, anim, sprite, carry, false, false);
                }
            }
        }
    }

    /// `func_0013C0`: sound `d0` (`func_0034B6`) if the sprite record at
    /// `sprite` is on the screen (the high bytes of its x and y 0).
    /// Gives back C (clear).
    pub fn r_0013c0(&mut self, d0: u32, sprite: u32) -> bool {
        if self.read8(sprite + SPRITE_Y_HIGH) == 0 && self.read8(sprite + SPRITE_X_HIGH) == 0 {
            self.r_0034b6(d0);
        }
        false
    }

    /// `func_0013D6`: the task's animation a frame on, as `func_001336`
    /// (the sprite record not made `BUDGETED`).  Gives back d0, d1, d2,
    /// d5 (the sprite record's index), a0 (the animation), a5 (the
    /// sprite record), X and C.
    pub fn r_0013d6(&mut self, d0: u32, d1: u32, d2: u32, d5: u32, task: u32) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        let anim = self.read32(task + TASK_ANIM);
        let timer = self.read8(task + TASK_TIMER);
        if timer != 0 {
            self.write8(task + TASK_TIMER, timer - 1);
            let step = self.read8(task + TASK_STEP);
            let d0 = low_word(d0, u16::from(step));
            let frame = self.read8(anim.wrapping_add(u32::from(step)).wrapping_sub(1));
            let d1 = low_byte(d1, frame);
            let (d5, sprite, x) = self.task_sprite(d5, task);
            self.write8(sprite + SPRITE_FRAME, frame);
            return (d0, d1, d2, d5, anim, sprite, x, false);
        }
        let mut d0 = low_word(d0, u16::from(self.read8(task + TASK_STEP)));
        let mut d1 = d1;
        loop {
            let frames = self.read8(anim.wrapping_add(d0 as u16 as i16 as u32));
            d1 = low_byte(d1, frames);
            match frames {
                0 => {
                    self.write8(task + TASK_STEP, 0);
                    self.write8(task + TASK_TIMER, 0);
                    let (d5, sprite, x) = self.task_sprite(d5, task);
                    return (d0, d1, d2, d5, anim, sprite, x, true);
                }
                0xFF => d0 = 0,
                _ => {
                    self.write8(task + TASK_TIMER, frames);
                    d0 = low_word(d0, (d0 as u16).wrapping_add(1));
                    let d2 = low_byte(d0, (d0 as u8).wrapping_add(1));
                    self.write8(task + TASK_STEP, d2 as u8);
                    let frame = self.read8(anim.wrapping_add(d0 as u16 as i16 as u32));
                    let d1 = low_byte(d1, frame);
                    let (d5, sprite, x) = self.task_sprite(d5, task);
                    self.write8(sprite + SPRITE_FRAME, frame);
                    return (d0, d1, d2, d5, anim, sprite, x, false);
                }
            }
        }
    }

    /// The task's sprite record: d5 with its index, its address, X
    /// (`func_000B5A`'s).
    fn task_sprite(&mut self, d5: u32, task: u32) -> (u32, u32, bool) {
        let d5 = low_byte(d5, self.read8(task + TASK_SPRITE));
        self.r_000b5a(d5)
    }

    /// `func_001456`: the first free entry of `func_001CFA`'s table from
    /// 6 to 0x13 (its record's first byte 0), its index to d0; C set if
    /// none is free.  Gives back d0, X and C.
    pub fn r_001456(&mut self, d0: u32) -> (u32, bool, bool) {
        for index in 6..0x14 {
            let (_, record, _) = self.r_001cfa(index);
            if self.read8(record) == 0 {
                return (index, false, false);
            }
        }
        (d0, false, true)
    }

    /// `func_001486`: C set if the map's cell above `CELL_X`, `CELL_Y`
    /// (`func_0016AC`) has a cell of kind 5 or 6 to its left or right
    /// or of kind 4 or 6 above or below it (bit 7 aside; the right one
    /// compared as a word, d1's bits 8-15 in it).  Nothing while
    /// `STOPPED`.  Gives back X and C.
    pub fn r_001486(&mut self, _d0: u32, d1: u32, _a0: u32, x: bool) -> (bool, bool) {
        if self.read8(STOPPED) != 0 {
            return (x, false);
        }
        let y = self.read8(CELL_Y);
        self.write8(CELL_Y, y.wrapping_sub(1));
        let (cell, _) = self.r_0016ac();
        let y = self.read8(CELL_Y);
        self.write8(CELL_Y, y.wrapping_add(1));
        let x = y == 0xFF;
        let width = u32::from(self.read8(WIDTH));
        let kind = |b: u8| b & 0x7F;
        let left = self.read8(cell.wrapping_add(width).wrapping_sub(1));
        if left != 0 && matches!(kind(left), 5 | 6) {
            return (x, true);
        }
        let right = self.read8(cell.wrapping_add(width).wrapping_add(1));
        if right != 0 {
            let word = (d1 as u16 & 0xFF00) | u16::from(kind(right));
            if matches!(word, 5 | 6) {
                return (x, true);
            }
        }
        let up = self.read8(cell);
        if up != 0 && matches!(kind(up), 4 | 6) {
            return (x, true);
        }
        let down = self.read8(cell.wrapping_add(2 * width));
        if down != 0 && matches!(kind(down), 4 | 6) {
            return (x, true);
        }
        (x, false)
    }

    /// `func_00152A`: the task's position from `POSITION`, bytes 0xC
    /// and 0xD cleared.
    pub fn r_00152a(&mut self, task: u32) {
        let pos = self.read32(POSITION);
        self.write32(task + TASK_POS, pos);
        self.write8(task + TASK_C, 0);
        self.write8(task + TASK_D, 0);
    }

    /// `func_00153A`: `POSITION` from the task's.
    pub fn r_00153a(&mut self, task: u32) {
        let pos = self.read32(task + TASK_POS);
        self.write32(POSITION, pos);
    }

    /// `func_001542`: `POSITION` from the object's.
    pub fn r_001542(&mut self, obj: u32) {
        let x = self.read16(obj + OBJ_X);
        self.write16(POSITION, x);
        let y = self.read16(obj + OBJ_Y);
        self.write16(POSITION_Y, y);
    }

    /// `func_001550`: the task's position the centre of the map cell
    /// `CELL_X`, `CELL_Y` (16 pixels a cell), bytes 0xC and 0xD
    /// cleared.  Gives back d0 (the y) and X (the row's sign).
    pub fn r_001550(&mut self, d0: u32, task: u32) -> (u32, bool) {
        self.write8(task + TASK_C, 0);
        self.write8(task + TASK_D, 0);
        let centre = |cell: u8| ((cell as i8 as u16) << 4) | 8;
        let x = centre(self.read8(CELL_X));
        self.write16(task + TASK_POS, x);
        let row = self.read8(CELL_Y);
        let y = centre(row);
        self.write16(task + TASK_Y, y);
        (low_word(d0, y), row & 0x80 != 0)
    }
}
