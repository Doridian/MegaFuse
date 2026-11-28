//! What the modules of 0x15000-0x16000 share (`t_*.rs`): the fields
//! and RAM they name, and the callees as the original calls them, on
//! the registers (a `Regs`): an object (a4) and its sprite record (a5),
//! a task (a6) and its sprite record (a5).
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read: each passes the
//! registers on as the 68000 does.

use crate::{Game, Regs};

/// An object's fields (`a_objects.rs`, `anim.rs`, `a_motion.rs`).
/// Byte 0: not 0 while it is in use.
pub(super) const OBJ_ID: u32 = 0x00;
/// The routine it runs (a long), and a long `func_006E28` keeps a
/// routine in, whose first byte's high nibble the routines here use as
/// a mark that they are under way.
pub(super) const OBJ_ROUTINE: u32 = 0x02;
pub(super) const OBJ_SAVED: u32 = 0x06;
/// Its position's low bytes (bytes of `OBJ_X`'s and `OBJ_Y`'s words
/// before them: written by the set-up of an object here).
pub(super) const OBJ_0F: u32 = 0x0F;
pub(super) const OBJ_10: u32 = 0x10;
pub(super) const OBJ_11: u32 = 0x11;
/// The direction it takes next (an angle for `func_00410C`, here), its
/// direction and the kind of its animation (`anim.rs`).
pub(super) const OBJ_NEXT_DIRECTION: u32 = 0x13;
pub(super) const OBJ_DIRECTION: u32 = 0x14;
pub(super) const OBJ_KIND: u32 = 0x15;
/// A byte the routines here count down.
pub(super) const OBJ_COUNT: u32 = 0x16;
/// Its position (words: x, y).
pub(super) const OBJ_X: u32 = 0x18;
pub(super) const OBJ_Y: u32 = 0x1A;
/// Its animation (a long) and the animation's frames to go and
/// position (`anim.rs`).
pub(super) const OBJ_ANIM: u32 = 0x20;
pub(super) const OBJ_TIMER: u32 = 0x24;
pub(super) const OBJ_STEP: u32 = 0x25;
/// Bytes (and a word at `OBJ_26`) of the routines here.
pub(super) const OBJ_26: u32 = 0x26;
pub(super) const OBJ_27: u32 = 0x27;
/// Its rate (a word, `a_motion.rs`) and offset (the high byte of a
/// word).
pub(super) const OBJ_RATE: u32 = 0x28;
pub(super) const OBJ_RATE_LOW: u32 = 0x29;
pub(super) const OBJ_OFFSET: u32 = 0x2A;

/// A task's fields (`anim.rs`, `a_motion.rs`).
/// Its position (words: x, y) and the low byte of its y.
pub(super) const TASK_X: u32 = 0x0E;
pub(super) const TASK_Y: u32 = 0x10;
pub(super) const TASK_Y_LOW: u32 = 0x11;
/// Bytes of the routines here.
pub(super) const TASK_12: u32 = 0x12;
pub(super) const TASK_13: u32 = 0x13;
pub(super) const TASK_15: u32 = 0x15;
pub(super) const TASK_COUNT: u32 = 0x16;
pub(super) const TASK_17: u32 = 0x17;
pub(super) const TASK_18: u32 = 0x18;
/// Its rate (a word) and its offset (the high byte of a word).
pub(super) const TASK_RATE: u32 = 0x1A;
pub(super) const TASK_RATE_LOW: u32 = 0x1B;
pub(super) const TASK_OFFSET: u32 = 0x20;
/// Its animation (a long; its first byte is `TASK_OFFSET`).
pub(super) const TASK_ANIM: u32 = 0x20;

/// A sprite record's frame and flags, position (`sprite.rs`).
pub(super) const SPRITE_FRAME: u32 = 0x04;
pub(super) const SPRITE_FLAGS: u32 = 0x05;
pub(super) const SPRITE_X: u32 = 0x08;
pub(super) const SPRITE_Y: u32 = 0x0A;

/// A position (words, x then y; their low bytes after each) that
/// `func_001542` sets from an object's and `func_00152A` gives a task.
pub(super) const POSITION_X: u32 = 0xFFFF_9F04;
pub(super) const POSITION_X_LOW: u32 = 0xFFFF_9F05;
pub(super) const POSITION_Y: u32 = 0xFFFF_9F06;
pub(super) const POSITION_Y_LOW: u32 = 0xFFFF_9F07;
/// A velocity (words, x then y; `a_motion.rs`).
pub(super) const VELOCITY_X: u32 = 0xFFFF_9EC4;
pub(super) const VELOCITY_Y: u32 = 0xFFFF_9EC6;
/// The speed (a word, its low byte after it) `func_001620` and
/// `func_00410C` make a velocity of.
pub(super) const SPEED: u32 = 0xFFFF_9F00;
pub(super) const SPEED_LOW: u32 = 0xFFFF_9F01;
/// A map cell (column, row; `a_cells.rs`) and the one `func_003E44`
/// keeps (object 0's).
pub(super) const CELL_X: u32 = 0xFFFF_9F08;
pub(super) const CELL_Y: u32 = 0xFFFF_9F09;
pub(super) const KEPT_X: u32 = 0xFFFF_A966;
pub(super) const KEPT_Y: u32 = 0xFFFF_A967;
/// What `func_0012A0` sets an object up with: its animations, and a
/// long for its routine.
pub(super) const OBJ_INIT_ANIMS: u32 = 0xFFFF_80C8;
pub(super) const OBJ_INIT_ROUTINE: u32 = 0xFFFF_80D0;
/// The object running (its index and address) and its sprite record's
/// (`a_objects.rs`): reloaded after making a task.
pub(super) const RUNNING: u32 = 0xFFFF_A42C;
pub(super) const RUNNING_ADDR: u32 = 0xFFFF_A42E;
pub(super) const SLOT_INDEX: u32 = 0xFFFF_8A57;
pub(super) const SLOT_ADDR: u32 = 0xFFFF_8A5A;
/// The running task's index (`task.rs`).
pub(super) const CURRENT_TASK: u32 = 0xFFFF_80EE;
/// Flags of the game: bit 6 set while the boss here is beaten
/// (`func_015432`), cleared when it starts (`func_0153B6`).
pub(super) const MODE: u32 = 0xFFFF_9540;
pub(super) const MODE_BIT_6: u8 = 1 << 6;
/// Bit 0: while set, objects lose nothing (`func_003032`); set and
/// cleared with `MODE_BIT_6` here.
pub(super) const SPARED: u32 = 0xFFFF_9548;
pub(super) const SPARED_BIT: u8 = 1 << 0;
/// Not 0 while the boss's helper tasks are under way: 2 when the boss
/// comes (`func_0150DA`), one more as `func_015394` makes two;
/// cleared when one of them (`func_01525A`, `func_0152FA`) is done.
/// `func_015394`, `func_0153B6` and `func_015854` wait for 0.
pub(super) const HELPERS: u32 = 0xFFFF_AD5A;
/// Whether `func_015884`'s two objects were made (cleared by
/// `func_015406` before its wait, `func_0157DA`).
pub(super) const TURNERS_MADE: u32 = 0xFFFF_AD5B;
/// Whether the main loop's step runs (`i_main.rs`): set by
/// `func_015068`, cleared by `func_015C52`.
pub(super) const STEP_RUNS: u32 = 0xFFFF_80DF;

pub(super) fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

pub(super) fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// Counts the byte at `addr` down (`subq.b #1`): whether it got to
    /// 0; X and C the borrow.
    pub(super) fn t_count_down(&mut self, r: &mut Regs, addr: u32) -> bool {
        let n = self.read8(addr);
        self.write8(addr, n.wrapping_sub(1));
        r.x = n == 0;
        r.c = n == 0;
        n == 1
    }

    /// `func_001222`: a task running `routine` with a sprite record
    /// (d5, d6, a5, a6 then the sprite's and the task's; C: none).
    pub(super) fn t_make_task(&mut self, r: &mut Regs, depth: u8, priority: u8, routine: u32) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, priority, routine);
    }

    /// `func_0003E8`: a task running `routine`, no sprite (d6, a6 then
    /// its; C: none).
    pub(super) fn t_make_bare_task(&mut self, r: &mut Regs, priority: u8, routine: u32) {
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, priority, routine);
    }

    /// `func_00053E`: the running task's routine from now on.
    pub(super) fn t_next_task(&mut self, r: &mut Regs, routine: u32) {
        (r.x, r.c) = self.r_00053e(r.d6, routine);
    }

    /// d4, a4 back to the object running.
    pub(super) fn t_reload_object(&mut self, r: &mut Regs) {
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
    }

    /// d5, a5 back to the running sprite record.
    pub(super) fn t_reload_sprite(&mut self, r: &mut Regs) {
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
    }

    /// d6, a6 back to the running task (`func_0005F8`), d5, a5 to its
    /// sprite record.
    pub(super) fn t_reload_task(&mut self, r: &mut Regs) {
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        self.t_reload_sprite(r);
    }

    /// `func_001336`: the object's animation a frame on (C: it ended).
    pub(super) fn t_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
    }

    /// `func_0013D6`: the task's animation a frame on (C: it ended).
    pub(super) fn t_task_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `func_001B7E`: the sprite placed for the task.
    pub(super) fn t_task_place(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_001B94`: the sprite placed for the task, y plus its offset.
    pub(super) fn t_task_place_offset(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
    }

    /// `func_00121A`: the task ends, its sprite record freed.
    pub(super) fn t_task_end(&mut self, r: &mut Regs) {
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
    }

    /// `func_000560`: the task ends.
    pub(super) fn t_task_end_bare(&mut self, r: &mut Regs) {
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }

    /// `func_001A88`: the task moved by the velocity's x.
    pub(super) fn t_move_x(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
    }

    /// `func_001AA2`: the task moved by the velocity's y.
    pub(super) fn t_move_y(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
    }

    /// `func_001B32`: the sprite placed for the object.
    pub(super) fn t_place(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
    }

    /// `func_001B50`: the sprite's depth, less the object's offset.
    pub(super) fn t_depth_offset(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b50(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_001B5C`: the sprite's depth.
    pub(super) fn t_depth(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_001BE2`: the next random number (d0's low byte).
    pub(super) fn t_random(&mut self, r: &mut Regs) -> u8 {
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        r.d0 as u8
    }

    /// `func_0034B6`: `sound` to the Z80's queue (d0's low byte).
    pub(super) fn t_sound(&mut self, r: &mut Regs, sound: u8) {
        r.d0 = low_byte(r.d0, sound);
        self.r_0034b6(r.d0);
    }
}
