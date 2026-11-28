//! The first tasks of the screen's sequence (`y_screen.rs`;
//! 0x1A7EC-0x1AAE6): `func_01A7EC` scrolls the planes and, after
//! 0x160 frames, makes `func_01A850`, whose `func_01A864` makes a
//! task with a sprite (`func_01A8AE`) that moves left; then
//! `func_01A8E2` makes five tasks one by one (at the frames of
//! `APPEAR_AT`), each of which (`func_01A97C`) at its animation's end
//! makes two more (`func_01AA64`, `func_01AAAC`); the fifth's
//! `func_01AA64` passes to `func_01AAB6`, which loads the next lists
//! and goes on to `func_01AAE6` (`y_seq_b.rs`).

use crate::rewritten::y_screen::{low_byte, low_word, BYTE_AECB, OBJECTS_STOPPED, SCROLL_D002, SEQ_STATE};
use crate::{Game, Regs};

/// The words the task scrolls each frame (the second every other
/// frame), kept within 0x1FF (`h_scroll.rs`'s table).
const SCROLL_D000: u32 = 0xFFFF_D000;
/// Counted up each frame by `func_01A7EC`; bit 0 picks the frames.
const FRAMES: u32 = 0xFFFF_AEC0;
/// Words set when the scroll is done.
const WORD_AEC2: u32 = 0xFFFF_AEC2;
const WORD_AEC4: u32 = 0xFFFF_AEC4;
/// Set to 1 when `func_01A8AE`'s sprite has arrived: `func_01A8E2`
/// starts then.
const ARRIVED: u32 = 0xFFFF_AECC;
/// The low byte of `PLACE_X` (`y_screen.rs`), here carrying a count
/// from one task to the one it makes.
const BYTE_80C5: u32 = 0xFFFF_80C5;

/// In the ROM: the frames (by count) `func_01A8E2` makes its tasks
/// at, the y by count (bytes), the animations by count (longs).
const APPEAR_AT: u32 = 0x01A8DC;
const Y_BY_COUNT: u32 = 0x01AA46;
const ANIMS_BY_COUNT: u32 = 0x01AA4C;
/// The animations.
const ANIM_DC222: u32 = 0x0DC222;
const ANIM_DC22A: u32 = 0x0DC22A;
const ANIM_DC24C: u32 = 0x0DC24C;
const ANIM_DC246: u32 = 0x0DC246;
/// The list `func_01AAB6` loads.
const LOAD_1B6DE: u32 = 0x01B6DE;

/// The routines.
const SCROLL_DONE: u32 = 0x01A850;
const MAKING: u32 = 0x01A864;
const MOVING_LEFT: u32 = 0x01A8AE;
const APPEARING: u32 = 0x01A8E2;
const APPEARED: u32 = 0x01A97C;
const RISING: u32 = 0x01AA64;
const SECOND: u32 = 0x01AAAC;
const LOADING: u32 = 0x01AAB6;
const AFTER_LOAD: u32 = 0x01AAE6;

/// How many of `func_01A8E2`'s tasks there are.
const APPEAR_COUNT: u8 = 5;

/// A task's fields: its position (words, the y's bytes too),
/// its animation's step and timer, two counts and its animation.
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_Y_HIGH: u32 = 0x10;
const TASK_Y_LOW: u32 = 0x11;
const TASK_ANIM_STEP: u32 = 0x12;
const TASK_ANIM_TIMER: u32 = 0x13;
const TASK_COUNT: u32 = 0x18;
const TASK_MADE: u32 = 0x19;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's frame, flags, x (a word) and the x's low byte
/// (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;

impl Game {
    /// `func_001B7E` with the registers: the task's sprite placed.
    fn y_put(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_01A7EC`, a task: ends (`func_01A85E`) once `SEQ_STATE` is
    /// set.  `SCROLL_D000` one on each frame, `SCROLL_D002` every
    /// other, both within 0x1FF; the task's word at 0x18 counts up to
    /// 0x160, where `WORD_AEC2`/`WORD_AEC4` are set and the task
    /// `func_01A850` (priority 0x1E) is made.  Gives back the
    /// registers as the original leaves them.
    pub fn r_01a7ec(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        r.c = false;
        if self.read8(SEQ_STATE) != 0 {
            self.y_end(r);
            return;
        }
        let scroll = self.read16(SCROLL_D000).wrapping_add(1);
        self.write16(SCROLL_D000, scroll & 0x1FF);
        let frames = self.read8(FRAMES);
        let frames = frames.wrapping_add(1);
        self.write8(FRAMES, frames);
        r.x = frames == 0;
        if frames & 0x01 == 0 {
            let scroll = self.read16(SCROLL_D002);
            let next = scroll.wrapping_add(1);
            r.x = next == 0;
            self.write16(SCROLL_D002, next & 0x1FF);
        }
        let count = self.read16(r.a6 + TASK_COUNT);
        r.d0 = low_word(r.d0, count);
        r.c = false;
        if count == 0x160 {
            return;
        }
        let count = count.wrapping_add(1);
        r.x = count == 0;
        r.d0 = low_word(r.d0, count);
        self.write16(r.a6 + TASK_COUNT, count);
        r.c = count < 0x160;
        if count == 0x160 {
            self.write16(WORD_AEC2, 0x110);
            self.write16(WORD_AEC4, 0x136);
            self.y_start_task(r, 0x1E, SCROLL_DONE);
        }
    }

    /// `func_01A850`, a task: `func_01A864` next.
    pub fn r_01a850(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.y_next(regs, MAKING);
    }

    /// `loc_01A85C`: nothing.
    pub fn r_01a85c(&mut self) {}

    /// `loc_01A85E`: the running task ends (`func_000560`).  Gives back
    /// d6, X and C.
    pub fn r_01a85e(&mut self, d5: u32, d6: u32, x: bool) -> (u32, bool, bool) {
        self.r_000560(d5, d6, x)
    }

    /// `func_01A864`, a task: `func_01A8E2` next with its counts 0, and
    /// a task with a sprite `func_01A8AE` (depth 0xA, priority 0x82)
    /// placed at (0x180, 0x59), flags 0x80, animation `ANIM_DC222`.
    pub fn r_01a864(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_next(r, APPEARING);
        self.write8(r.a6 + TASK_COUNT, 0);
        self.write8(r.a6 + TASK_MADE, 0);
        self.y_make_task(r, 0xA, 0x82, MOVING_LEFT);
        self.y_place(r, 0x180, 0x59, 0x80, ANIM_DC222);
    }

    /// `func_01A8AE`, a task: its animation on (`func_0013D6`); it ends
    /// (`func_01A8CA`) once `SEQ_STATE` is set; its sprite's x down by
    /// one a frame until it is 0xFF, then `ARRIVED` is 1.  Gives back the
    /// registers as the original leaves them.
    pub fn r_01a8ae(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_animate(r);
        r.c = false;
        if self.read8(SEQ_STATE) != 0 {
            (r.d6, r.x, r.c) = self.r_01a8ca(r.d5, r.d6, r.x);
            return;
        }
        let x = self.read16(r.a5 + SPRITE_X);
        if x == 0xFF {
            self.write8(ARRIVED, 1);
            r.c = false;
            return;
        }
        self.write16(r.a5 + SPRITE_X, x.wrapping_sub(1));
        r.c = x == 0;
        r.x = r.c;
    }

    /// `func_01A8CA`: the task's sprite record (`func_000890`) and the
    /// task end (`func_000560`).  Gives back d6, X and C.
    pub fn r_01a8ca(&mut self, d5: u32, d6: u32, x: bool) -> (u32, bool, bool) {
        self.r_000890();
        self.r_000560(d5, d6, x)
    }

    /// `func_01A8E2`, a task: waits for `ARRIVED`; ends (`func_01A85E`)
    /// when it has made `APPEAR_COUNT` tasks.  It counts its frames,
    /// and at each of `APPEAR_AT` makes (sound 0x7B) a task with a
    /// sprite `func_01A97C` (depth 5, priority 0x32) at (0xB0, 0x59),
    /// its count the tasks made so far (through `BYTE_80C5`), flags
    /// 0x80, frame 0, its animation `ANIM_DC22A` for even counts and
    /// `ANIM_DC24C` for odd.  Gives back the registers as the original
    /// leaves them.
    pub fn r_01a8e2(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.read8(ARRIVED) & 0x01 == 0 {
            return;
        }
        let made = self.read8(r.a6 + TASK_MADE);
        r.d1 = low_byte(r.d1, made);
        if made == APPEAR_COUNT {
            r.c = false;
            (r.d6, r.x, r.c) = self.r_01a85e(r.d5, r.d6, r.x);
            return;
        }
        let count = self.read8(r.a6 + TASK_COUNT).wrapping_add(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = count == 0;
        r.d1 = low_word(r.d1, made as i8 as i16 as u16);
        let at = self.read8(APPEAR_AT.wrapping_add(made as i8 as i32 as u32));
        r.d0 = low_byte(r.d0, at);
        r.c = at < count;
        if at != count {
            return;
        }
        self.y_sound(r, 0x7B);
        let made = made.wrapping_add(1);
        self.write8(r.a6 + TASK_MADE, made);
        self.write8(BYTE_80C5, made);
        self.y_make_task(r, 5, 0x32, APPEARED);
        let made = self.read8(BYTE_80C5);
        self.write8(r.a6 + TASK_COUNT, made);
        self.write16(r.a6 + TASK_X, 0xB0);
        self.write16(r.a6 + TASK_Y, 0x59);
        self.write8(r.a5 + SPRITE_FLAGS, 0x80);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.y_put(r);
        let anim = if self.read8(r.a6 + TASK_COUNT) & 0x01 != 0 { ANIM_DC24C } else { ANIM_DC22A };
        self.write32(r.a6 + TASK_ANIM, anim);
        self.write8(r.a6 + TASK_ANIM_STEP, 0);
        self.write8(r.a6 + TASK_ANIM_TIMER, 0);
        r.c = false;
    }

    /// `func_01A97C`, a task: its animation on (`func_0013D6`); at its
    /// end (sound 0x7C) its count goes to `BYTE_AECB`, its sprite
    /// record and it end (`func_000890`, `func_000560`) and two tasks
    /// with sprites take its place, both with the count, the y from
    /// `Y_BY_COUNT`, flags 0x80: `func_01AA64` (depth 5,
    /// priority 0x32) at x 0xD0, frame 0, animation `ANIM_DC246`;
    /// `func_01AAAC` (9, 0x32) at x 0xA0, its animation from
    /// `ANIMS_BY_COUNT`.  Gives back the registers as the original
    /// leaves them.
    pub fn r_01a97c(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_animate(r);
        if !r.c {
            return;
        }
        self.y_sound(r, 0x7C);
        let count = self.read8(r.a6 + TASK_COUNT);
        self.write8(BYTE_AECB, count);
        self.r_000890();
        self.y_end(r);

        self.y_make_task(r, 5, 0x32, RISING);
        let count = self.read8(BYTE_AECB);
        self.write8(r.a6 + TASK_COUNT, count);
        self.write16(r.a6 + TASK_X, 0xD0);
        self.y_set_y(r, count);
        self.write8(r.a5 + SPRITE_FLAGS, 0x80);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.y_put(r);
        self.write32(r.a6 + TASK_ANIM, ANIM_DC246);
        self.write8(r.a6 + TASK_ANIM_STEP, 0);
        self.write8(r.a6 + TASK_ANIM_TIMER, 0);

        self.y_make_task(r, 9, 0x32, SECOND);
        self.write16(r.a6 + TASK_X, 0xA0);
        let count = self.read8(BYTE_AECB);
        self.y_set_y(r, count);
        self.write8(r.a5 + SPRITE_FLAGS, 0x80);
        self.y_put(r);
        let count = self.read8(BYTE_AECB);
        let twice = count.wrapping_add(count);
        let offset = twice.wrapping_add(twice);
        r.x = twice >= 0x80;
        r.d1 = low_word(r.d1, offset as i8 as i16 as u16);
        let anim = self.read32(ANIMS_BY_COUNT.wrapping_add(offset as i8 as i32 as u32));
        self.write32(r.a6 + TASK_ANIM, anim);
        self.write8(r.a6 + TASK_ANIM_STEP, 0);
        self.write8(r.a6 + TASK_ANIM_TIMER, 0);
        r.c = false;
    }

    /// The task's y (a word) from `Y_BY_COUNT` by `count`; d1 the
    /// count, a0 the table.
    fn y_set_y(&mut self, r: &mut Regs, count: u8) {
        let index = count as i8 as i16;
        r.d1 = low_word(r.d1, index as u16);
        r.a0 = Y_BY_COUNT;
        let y = self.read8(Y_BY_COUNT.wrapping_add(index as i32 as u32));
        self.write8(r.a6 + TASK_Y_LOW, y);
        self.write8(r.a6 + TASK_Y_HIGH, 0);
    }

    /// `func_01AA64`, a task: its animation on (`func_0013D6`), its
    /// sprite's x's low byte down by 0xA while it doesn't go below 0; then its sprite record ends (`func_000890`) and, but for the
    /// task of count 5, the task (`func_01A85E`); that one's sprite
    /// record again, `func_01AAB6` next and, unless the objects are
    /// stopped, a fade (`func_00A51A` with 6).  Gives back the
    /// registers as the original leaves them.
    pub fn r_01aa64(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_animate(r);
        let y = self.read8(r.a5 + SPRITE_X_LOW);
        let next = y.wrapping_sub(0xA);
        r.d0 = low_byte(r.d0, next);
        r.c = y < 0xA;
        r.x = r.c;
        if !r.c {
            self.write8(r.a5 + SPRITE_X_LOW, next);
            r.c = false;
            return;
        }
        self.r_000890();
        let count = self.read8(r.a6 + TASK_COUNT);
        if count != APPEAR_COUNT {
            (r.d6, r.x, r.c) = self.r_01a85e(r.d5, r.d6, r.x);
            return;
        }
        self.r_000890();
        self.y_next(r, LOADING);
        if self.read8(OBJECTS_STOPPED) & 0x01 == 0 {
            self.y_fade(r, 6);
        }
    }

    /// `func_01AAAC`, a task: its animation on (`func_0013D6`); at its
    /// end it ends (`func_01A8CA`).
    pub fn r_01aaac(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.y_animate(regs);
        if regs.c {
            (regs.d6, regs.x, regs.c) = self.r_01a8ca(regs.d5, regs.d6, regs.x);
        }
    }

    /// `func_01AAB6`, a task: unless the objects are stopped,
    /// `BYTE_AECB` 0, `func_01AAE6` next, `SEQ_STATE` 1 and the list
    /// `LOAD_1B6DE` loaded (`func_0094D8`).  Gives back the registers
    /// as the original leaves them.
    pub fn r_01aab6(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.read8(OBJECTS_STOPPED) & 0x01 != 0 {
            return;
        }
        self.write8(BYTE_AECB, 0);
        self.y_next(r, AFTER_LOAD);
        self.write8(SEQ_STATE, 1);
        self.y_load(r, LOAD_1B6DE);
    }

    /// `loc_01AAE4`: nothing.
    pub fn r_01aae4(&mut self) {}
}
