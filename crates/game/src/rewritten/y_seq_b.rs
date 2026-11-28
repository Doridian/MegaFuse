//! The screen's sequence, on from `func_01AAE6` (0x1AAE6-0x1B000):
//! five tasks with sprites made at the frames of `MAKE_AT`
//! (`func_01AB7A`), each turning in to the centre by
//! `func_01776C`/`func_017770` (`func_01AC44`); after the fifth
//! `func_01ACB6` loads the next lists and makes two more tasks
//! (`func_01AD66`, `func_01ADAC`).  `func_01ADAC` waits for the VDP's
//! line `LINE`; two tasks then turn out (`func_01AF30`) and the
//! routine made by `func_01ADAE`'s part (`func_01AEF0`) starts
//! `func_01AF90`, whose three tasks (`func_01B01E`) follow.  Left out
//! (they yield, `func_000522`): `func_01AD66`, `func_01AD76`,
//! `func_01AD90` and `func_01AEE0`.

use crate::rewritten::y_screen::{low_byte, low_word, BYTE_AECB, BYTE_AECD, OBJECTS_STOPPED, SCROLL_D002, SCROLL_D402, SEQ_STATE, SKIPPED, TASK_SPRITE};
use crate::{Game, Regs};

/// How many tasks `func_01AB7A` has made (and, the first time,
/// cleared with `BYTE_AECB`).
const MADE: u32 = 0xFFFF_AECC;
/// A byte the tasks pass on: 1 when `func_01AF30`'s tasks are in,
/// 0xFF when `func_01AF7A`'s are to end; `func_01AF90` counts its
/// tasks in it.
const STATE_AECE: u32 = 0xFFFF_AECE;
/// A word (its high byte set to 1 by `func_01ADAC`), cleared by
/// `func_01AEF0`.
const WORD_AEC8: u32 = 0xFFFF_AEC8;
const BYTE_AEC8: u32 = 0xFFFF_AEC8;
/// A long cleared before `func_0094B2`.
const LONG_FF580A: u32 = 0xFF_580A;
/// The task, a sprite record's index and address (`sprite.rs`).
const CURRENT_TASK: u32 = 0xFFFF_80EE;
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;
/// The VDP's H/V counter: its high byte the line.
const HV_COUNTER: u32 = 0xC00008;
/// The line `func_01ADAC` waits for.
const LINE: u8 = 0xC0;

/// In the ROM: the frames (by count) `func_01AB7A` makes its tasks at,
/// their first distances, the angles of `func_01AF90`'s tasks, the
/// lists loaded.
const MAKE_AT: u32 = 0x01B6F2;
const DISTANCES: u32 = 0x01B6EC;
const ANGLES: u32 = 0x01B006;
const LOAD_1B6D4: u32 = 0x01B6D4;
/// The animations.
const ANIM_DC292: u32 = 0x0DC292;
const ANIM_DC280: u32 = 0x0DC280;
const ANIM_DC29C: u32 = 0x0DC29C;
const ANIM_DC296: u32 = 0x0DC296;

/// The routines.
const COUNTING: u32 = 0x01AB52;
const MAKING: u32 = 0x01AB7A;
const TURNING_IN: u32 = 0x01AC44;
const AFTER_IN: u32 = 0x01ACB6;
const WAITING_AD56: u32 = 0x01AD56;
const TASK_AD66: u32 = 0x01AD66;
const TASK_ADAC: u32 = 0x01ADAC;
const AFTER_OUT: u32 = 0x01AF7A;
const LAST: u32 = 0x01AF90;
const FOLLOWING: u32 = 0x01B01E;

/// How many tasks `func_01AB7A` makes; the angle added between two.
const MAKE_COUNT: u8 = 5;
const ANGLE_STEP: u8 = 0x33;

/// A task's fields: its position (words, with their bytes), its
/// animation's step and timer, a count or angle, a distance, its
/// animation.
const TASK_X_HIGH: u32 = 0x0E;
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y_HIGH: u32 = 0x10;
const TASK_Y_LOW: u32 = 0x11;
const TASK_ANIM_STEP: u32 = 0x12;
const TASK_ANIM_TIMER: u32 = 0x13;
const TASK_ANGLE: u32 = 0x18;
const TASK_DISTANCE: u32 = 0x19;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's fields (`sprite.rs`): its palette, frame, flags
/// and position (words, and their low bytes).
const SPRITE_PALETTE: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;

impl Game {
    /// The task's count at 0x18 one up (X its carry); whether it is
    /// now `to` (C: it is below).
    fn y_count_to(&mut self, r: &mut Regs, to: u8) -> bool {
        let count = self.read8(r.a6 + TASK_ANGLE).wrapping_add(1);
        self.write8(r.a6 + TASK_ANGLE, count);
        r.x = count == 0;
        r.c = count < to;
        count == to
    }

    /// `func_01776C` on (d0's low byte `angle`, d1's `distance`).
    fn y_1776c(&mut self, r: &mut Regs, angle: u8, distance: u8) {
        r.d1 = low_byte(r.d1, distance);
        r.d0 = low_byte(r.d0, angle);
        (r.d0, r.x) = self.r_01776c(r.d0, r.d1, r.d3);
    }

    /// `func_017770` on (d0's low byte `angle`, d1's `distance`).
    fn y_17770(&mut self, r: &mut Regs, angle: u8, distance: u8) {
        r.d1 = low_byte(r.d1, distance);
        r.d0 = low_byte(r.d0, angle);
        (r.d0, r.x) = self.r_017770(r.d0, r.d1, r.d3);
    }

    /// `func_01AAE6`, a task: `SEQ_STATE` 0, `func_01AB52` next with
    /// its counts 0, a task with a sprite `func_01AD56` (depth 0x1E,
    /// priority 0x7D) placed at (0x80, 0x58), flags 0x80 (`ANIM_DC292`);
    /// unless the objects are stopped, a fade in (`func_00A506` with 1),
    /// the scroll words 0 and `func_008664`.  Gives back the registers
    /// as the original leaves them.
    pub fn r_01aae6(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(SEQ_STATE, 0);
        self.y_next(r, COUNTING);
        self.write8(r.a6 + TASK_ANGLE, 0);
        self.write8(r.a6 + TASK_DISTANCE, 0);
        self.y_make_task(r, 0x1E, 0x7D, WAITING_AD56);
        self.y_place(r, 0x80, 0x58, 0x480, ANIM_DC292);
        if self.read8(OBJECTS_STOPPED) & 0x01 != 0 {
            return;
        }
        self.y_fade_in(r, 1);
        self.write16(SCROLL_D002, 0);
        self.write16(SCROLL_D402, 0);
        self.r_008664(r);
    }

    /// `func_01AB52`, a task: after 0xA frames `func_01AB7A` next, its
    /// count, `MADE` and `BYTE_AECB` 0.  Gives back X and C.
    pub fn r_01ab52(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if !self.y_count_to(r, 0xA) {
            return;
        }
        self.y_next(r, MAKING);
        self.write8(r.a6 + TASK_ANGLE, 0);
        self.write8(MADE, 0);
        self.write8(BYTE_AECB, 0);
        r.c = false;
    }

    /// `func_01AB7A`, a task: ends (`func_01A85E`) once `MADE` is
    /// `MAKE_COUNT`; at the frames of `MAKE_AT` it makes a task with a
    /// sprite `func_01AC44` (depth 0xA, priority 0x1E) with the angle
    /// `BYTE_AECB` and the distance from `DISTANCES`, its position's
    /// high bytes by them (`func_017770` the x; `func_01776C` less 8 the
    /// y), flags 0x80, frame 0, `ANIM_DC280`; `MADE` one more, the
    /// angle `ANGLE_STEP` on.  Gives back the registers as the
    /// original leaves them.
    pub fn r_01ab7a(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let count = self.read8(r.a6 + TASK_ANGLE).wrapping_add(1);
        self.write8(r.a6 + TASK_ANGLE, count);
        r.x = count == 0;
        let made = self.read8(MADE);
        r.d1 = low_byte(r.d1, made);
        if made == MAKE_COUNT {
            r.c = false;
            (r.d6, r.x, r.c) = self.r_01a85e(r.d5, r.d6, r.x);
            return;
        }
        r.d0 = low_byte(r.d0, count);
        let index = made as i8 as i16;
        r.d1 = low_word(r.d1, index as u16);
        r.a0 = MAKE_AT;
        let at = self.read8(MAKE_AT.wrapping_add(index as i32 as u32));
        r.c = count < at;
        if count != at {
            return;
        }
        r.d2 = low_word(r.d2, index as u16);
        r.a0 = DISTANCES;
        let distance = self.read8(DISTANCES.wrapping_add(index as i32 as u32));
        let angle = self.read8(BYTE_AECB);
        // d0 and d1 kept (movem) over func_01776C
        let d0_kept = low_byte(r.d0, angle);
        self.y_1776c(r, angle, distance);
        r.d2 = low_byte(r.d2, (r.d0 as u8).wrapping_sub(8));
        let y = r.d2;
        r.d0 = d0_kept;
        (r.d0, r.x) = self.r_017770(r.d0, r.d1, r.d3);
        let x = r.d0;
        self.y_make_task(r, 0xA, 0x1E, TURNING_IN);
        let angle = self.read8(BYTE_AECB);
        self.write8(r.a6 + TASK_ANGLE, angle);
        let made = self.read8(MADE) as i8 as i16;
        r.d2 = low_word(r.d2, made as u16);
        let distance = self.read8(DISTANCES.wrapping_add(made as i32 as u32));
        self.write8(r.a6 + TASK_DISTANCE, distance);
        r.d0 = x;
        self.write8(r.a6 + TASK_X_LOW, x as u8);
        self.write8(r.a6 + TASK_X_HIGH, 0);
        r.d0 = y;
        self.write8(r.a6 + TASK_Y_LOW, y as u8);
        self.write8(r.a6 + TASK_Y_HIGH, 0);
        self.write8(r.a5 + SPRITE_FLAGS, 0x80);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        self.write32(r.a6 + TASK_ANIM, ANIM_DC280);
        self.write8(r.a6 + TASK_ANIM_STEP, 0);
        self.write8(r.a6 + TASK_ANIM_TIMER, 0);
        let made = self.read8(MADE).wrapping_add(1);
        self.write8(MADE, made);
        let angle = self.read8(BYTE_AECB);
        let next = angle.wrapping_add(ANGLE_STEP);
        self.write8(BYTE_AECB, next);
        r.c = next < angle;
        r.x = r.c;
    }

    /// `func_01AC44`, a task: its animation on (`func_0013D6`); its
    /// distance down by 5 and its sprite (the running task's record,
    /// `SLOT_ADDR`) at `func_017770` and `func_01776C` less 8 of its
    /// angle and distance (the y kept in `BYTE_AECD` meanwhile; the
    /// task's record found again through `func_0005F8` after each).
    /// At the animation's end, the task with angle 0xCC (its sprite
    /// record ended, `func_000890`) goes on to `func_01ACB6` with its
    /// count 0; the others end (`func_01A8CA`).  Gives back the
    /// registers as the original leaves them.
    pub fn r_01ac44(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_animate(r);
        if r.c {
            let angle = self.read8(r.a6 + TASK_ANGLE);
            if angle != 0xCC {
                r.c = angle < 0xCC;
                (r.d6, r.x, r.c) = self.r_01a8ca(r.d5, r.d6, r.x);
                return;
            }
            self.r_000890();
            self.y_next(r, AFTER_IN);
            self.write8(r.a6 + TASK_ANGLE, 0);
            r.c = false;
            return;
        }
        let distance = self.read8(r.a6 + TASK_DISTANCE).wrapping_sub(5);
        self.write8(r.a6 + TASK_DISTANCE, distance);
        let angle = self.read8(r.a6 + TASK_ANGLE);
        self.y_1776c(r, angle, distance);
        r.d0 = low_byte(r.d0, (r.d0 as u8).wrapping_sub(8));
        self.write8(BYTE_AECD, r.d0 as u8);
        self.y_find_task(r);
        let distance = self.read8(r.a6 + TASK_DISTANCE);
        let angle = self.read8(r.a6 + TASK_ANGLE);
        self.y_17770(r, angle, distance);
        self.y_find_task(r);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        self.write8(r.a5 + SPRITE_X_LOW, r.d0 as u8);
        let y = self.read8(BYTE_AECD);
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        r.c = false;
    }

    /// The running task's record (`func_0005F8` with `CURRENT_TASK`).
    fn y_find_task(&mut self, r: &mut Regs) {
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
    }

    /// `func_01ACB6`, a task: after 0x3C frames `SEQ_STATE` 1 and it
    /// ends; unless `SKIPPED`, the list `LOAD_1B6D4` loaded,
    /// `LONG_FF580A` cleared, `func_0094B2`, and two tasks with sprites:
    /// `func_01AD66` (depth 0x14, priority 0x32) placed at (0xD0, 0),
    /// flags 0 (`ANIM_DC280`), and `func_01ADAC` (0xA, 0x3C) at
    /// (0x60, 0x59), flags 0x80 (`ANIM_DC29C`).  Gives back the
    /// registers as the original leaves them.
    pub fn r_01acb6(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if !self.y_count_to(r, 0x3C) {
            return;
        }
        self.write8(SEQ_STATE, 1);
        self.y_end(r);
        r.c = false;
        if self.read8(SKIPPED) != 0 {
            return;
        }
        if self.y_load(r, LOAD_1B6D4) {
            return;
        }
        self.write32(LONG_FF580A, 0);
        r.c = self.r_0094b2();
        self.y_make_task(r, 0x14, 0x32, TASK_AD66);
        self.y_place(r, 0xD0, 0, 0, ANIM_DC280);
        self.y_make_task(r, 0xA, 0x3C, TASK_ADAC);
        self.y_place(r, 0x60, 0x59, 0x1080, ANIM_DC29C);
    }

    /// `loc_01AD54`: nothing.
    pub fn r_01ad54(&mut self) {}

    /// `func_01AD56`, a task: its animation on (`func_0013D6`); it ends
    /// (`func_01A8CA`) once bit 0 of `SEQ_STATE` is set.
    pub fn r_01ad56(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.y_animate(regs);
        if self.read8(SEQ_STATE) & 0x01 != 0 {
            (regs.d6, regs.x, regs.c) = self.r_01a8ca(regs.d5, regs.d6, regs.x);
        }
    }

    /// `loc_01ADAA`: nothing.
    pub fn r_01adaa(&mut self) {}

    /// `func_01ADAC`, a task: waits for the VDP's line `LINE` (ending,
    /// `func_01A8CA`, if `SKIPPED` is set meanwhile); then the scroll
    /// words 0 and 0xE0, `func_008664`, `BYTE_AEC8` 1, its animation
    /// on (`func_0013D6`), and it ends unless bit 0 of `SEQ_STATE` is
    /// set.  Gives back the registers as the original leaves them.
    pub fn r_01adac(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        loop {
            if self.read8(SKIPPED) != 0 {
                r.c = false;
                (r.d6, r.x, r.c) = self.r_01a8ca(r.d5, r.d6, r.x);
                return;
            }
            let hv = self.read16(HV_COUNTER);
            r.d0 = low_word(r.d0, hv >> 8);
            r.x = hv & 0x80 != 0;
            if (hv >> 8) as u8 >= LINE {
                break;
            }
            self.pass(78);
        }
        self.write16(SCROLL_D002, 0);
        self.write16(SCROLL_D402, 0xE0);
        self.r_008664(r);
        self.write8(BYTE_AEC8, 1);
        self.y_animate(r);
        if self.read8(SEQ_STATE) & 0x01 == 0 {
            (r.d6, r.x, r.c) = self.r_01a8ca(r.d5, r.d6, r.x);
        }
    }

    /// `func_01AEE4` (the rest of `func_01AD76`'s part after its second
    /// yield): `func_008552` and the task ends (`func_000560`).
    pub fn r_01aee4(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.r_008552();
        self.y_end(regs);
    }

    /// `func_01AEF0`, a task: `WORD_AEC8` and the scroll word 0,
    /// `func_008664`; once bit 0 of `STATE_AECE` is set its animation
    /// is `ANIM_DC296`, `func_01AF90` next and sound 0x7D.  Gives back
    /// the registers as the original leaves them.
    pub fn r_01aef0(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write16(WORD_AEC8, 0);
        self.write16(SCROLL_D402, 0);
        self.r_008664(r);
        if self.read8(STATE_AECE) & 0x01 == 0 {
            return;
        }
        self.write32(r.a6 + TASK_ANIM, ANIM_DC296);
        self.write8(r.a6 + TASK_ANIM_STEP, 0);
        self.write8(r.a6 + TASK_ANIM_TIMER, 0);
        self.y_next(r, LAST);
        self.y_sound(r, 0x7D);
    }

    /// `func_01AF30`, a task: its animation on (`func_0013D6`), its
    /// distance down by 8 and its sprite's low bytes at `func_017770`
    /// and `func_01776C` less 8 of its angle and distance; at distance
    /// 0, `func_01AF7A` next and `STATE_AECE` 1.  Gives back the
    /// registers as the original leaves them.
    pub fn r_01af30(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_animate(r);
        let distance = self.read8(r.a6 + TASK_DISTANCE).wrapping_sub(8);
        self.write8(r.a6 + TASK_DISTANCE, distance);
        let angle = self.read8(r.a6 + TASK_ANGLE);
        self.y_17770(r, angle, distance);
        self.write8(r.a5 + SPRITE_X_LOW, r.d0 as u8);
        self.y_1776c(r, angle, distance);
        let y = r.d0 as u8;
        r.d0 = low_byte(r.d0, y.wrapping_sub(8));
        r.x = y < 8;
        self.write8(r.a5 + SPRITE_Y_LOW, r.d0 as u8);
        r.c = false;
        if distance == 0 {
            self.y_next(r, AFTER_OUT);
            self.write8(STATE_AECE, 1);
            r.c = false;
        }
    }

    /// `func_01AF7A`, a task: its animation on (`func_0013D6`); it ends
    /// (`func_01A8CA`) when `STATE_AECE` is 0xFF.  Gives back the
    /// registers as the original leaves them.
    pub fn r_01af7a(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_animate(r);
        let state = self.read8(STATE_AECE);
        r.c = state < 0xFF;
        if state == 0xFF {
            (r.d6, r.x, r.c) = self.r_01a8ca(r.d5, r.d6, r.x);
        }
    }

    /// `func_01AF90`, a task: its animation on (`func_0013D6`); at its
    /// end it ends (`func_01A8CA`) and makes three tasks `func_01B01E`
    /// (priority 0x64) with the angles of `ANGLES` and distance 0, each
    /// with a sprite record (`func_000796`, depth 0xA) of frame 0x33 +
    /// its number at (0x80, 0x70), flags 0x80; `STATE_AECE` counts
    /// them (ends at 3).  Gives back the registers as the original
    /// leaves them.
    pub fn r_01af90(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_animate(r);
        if !r.c {
            return;
        }
        (r.d6, r.x, r.c) = self.r_01a8ca(r.d5, r.d6, r.x);
        self.write8(STATE_AECE, 0);
        loop {
            self.y_start_task(r, 0x64, FOLLOWING);
            let number = self.read8(STATE_AECE);
            let index = number as i8 as i16;
            r.d1 = low_word(r.d1, index as u16);
            let angle = self.read8(ANGLES.wrapping_add(index as i32 as u32));
            self.write8(r.a6 + TASK_ANGLE, angle);
            self.write8(r.a6 + TASK_DISTANCE, 0);
            r.d0 = 0xA;
            (r.d5, r.a5, r.x, r.c) =
                self.r_000796(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
            self.write8(r.a6 + TASK_SPRITE, r.d5 as u8);
            let number = self.read8(STATE_AECE);
            r.d0 = low_byte(r.d0, 0x33u8.wrapping_add(number));
            self.write8(r.a5 + SPRITE_FRAME, r.d0 as u8);
            self.write16(r.a5 + SPRITE_X, 0x80);
            self.write16(r.a5 + SPRITE_Y, 0x70);
            self.write8(r.a5 + SPRITE_PALETTE, 0);
            self.write8(r.a5 + SPRITE_FLAGS, 0x80);
            let number = number.wrapping_add(1);
            self.write8(STATE_AECE, number);
            r.x = number == 0;
            r.c = number < 3;
            if number == 3 {
                break;
            }
        }
    }
}
