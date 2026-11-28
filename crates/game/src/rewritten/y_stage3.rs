//! An object's routines (a4) of 0x1A022-0x1A40A, run in
//! tools/inputs/stage-3-boss.txt (no other script reaches them): it
//! waits, counting `WAIT_ADE4` down, until hit for the last time
//! (`func_006E7C`), then plays two animations (`func_01A082`,
//! `func_01A09A`) and walks (`func_01A0BC`, `func_004096`), now and
//! then stopping to make eight tasks (`func_01A18C`: `func_019ACE`);
//! hit for the last time again it ends (`func_01A0FC`, `func_01A126`).
//! The tasks it makes: `func_01A228`'s (after `func_01A202`'s yields)
//! fly off from `CENTRE` and fall (`func_01A29E`); `func_01A328`'s
//! (after `func_01A30A`'s) spiral out from it (`func_01A392`).  Left out
//! (they yield, `func_000522`): `func_01A202`, `func_01A20C`,
//! `func_01A30A`, `func_01A314`.

use crate::rewritten::y_screen::{low_byte, low_word};
use crate::{Game, Regs};

/// A count: the waiting routine's frames to go (and, by its parity,
/// how `func_01A18C` places its tasks).
const WAIT_ADE4: u32 = 0xFFFF_ADE4;
/// The speed `func_004096` walks by (a word, `c_chase.rs`).
const SPEED: u32 = 0xFFFF_A960;
/// `func_01A18C`'s tasks still to make, and the next one's angle (the
/// two bytes of `PLACE_X`, `y_screen.rs`, used as counters here).
const TO_MAKE: u32 = 0xFFFF_80C4;
const NEXT_ANGLE: u32 = 0xFFFF_80C5;
/// The position's low bytes (`a_cells.rs`).
const POS_X_LOW: u32 = 0xFFFF_9EFB;
const POS_Y_LOW: u32 = 0xFFFF_9EFF;
/// The point the tasks start from (words; `q_object.rs`), and its low
/// bytes.
const CENTRE_X: u32 = 0xFFFF_A5CA;
const CENTRE_Y: u32 = 0xFFFF_A5CC;
const CENTRE_X_LOW: u32 = 0xFFFF_A5CB;
const CENTRE_Y_LOW: u32 = 0xFFFF_A5CD;
/// The velocity's y (a word, `a_motion.rs`).
const VELOCITY_Y: u32 = 0xFFFF_9EC6;
/// The running task, the running sprite record (`sprite.rs`).
const CURRENT_TASK: u32 = 0xFFFF_80EE;
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// The object's fields: its kind byte, routine (a long), a timer
/// (high nibble tested), a byte and a step for `func_004096`, a
/// table (a long), its facing (bit 0), its animation (a long) and the
/// animation's step and timer, a byte cleared before each walk.
const OBJ_KIND: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_TIMER: u32 = 0x06;
const OBJ_11: u32 = 0x11;
const OBJ_STEP: u32 = 0x12;
const OBJ_FACING: u32 = 0x14;
const OBJ_TABLE: u32 = 0x1C;
const OBJ_ANIM: u32 = 0x20;
const OBJ_ANIM_STEP: u32 = 0x24;
const OBJ_ANIM_TIMER: u32 = 0x25;
const OBJ_26: u32 = 0x26;

/// A task's fields: its position (words, and the y's high byte), a
/// frame count, a kind (0-3), an angle, a count or distance, the low
/// bytes `func_01A18C` gives its tasks, a rate (a word; its high byte
/// counted by `func_01A392`), and at 0x20 a byte whose sign ends the
/// fall (or its animation, a long).
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_Y_HIGH: u32 = 0x10;
const TASK_FRAMES: u32 = 0x12;
const TASK_KIND: u32 = 0x13;
const TASK_ANGLE: u32 = 0x16;
const TASK_COUNT: u32 = 0x17;
const TASK_X_LOW_19: u32 = 0x19;
const TASK_Y_LOW_1A: u32 = 0x1A;
const TASK_RATE: u32 = 0x1A;
const TASK_RATE_HIGH: u32 = 0x1A;
const TASK_20: u32 = 0x20;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's frame and flags.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// In the ROM: the table `func_01A09A` gives the object, the frames
/// of `func_01A29E`'s tasks (by kind and frame count), the
/// animations.
const TABLE_1A41A: u32 = 0x01A41A;
const FALL_FRAMES: u32 = 0x01A302;
const ANIM_DE2F4: u32 = 0x0DE2F4;
const ANIM_DE372: u32 = 0x0DE372;
const ANIM_DE306: u32 = 0x0DE306;
const ANIM_DE384: u32 = 0x0DE384;
const ANIM_DE2C6: u32 = 0x0DE2C6;

/// The routines.
const WAITING: u32 = 0x019988;
const FIRST_ANIM: u32 = 0x01A082;
const SECOND_ANIM: u32 = 0x01A09A;
const WALKING: u32 = 0x01A0BC;
const ENDING: u32 = 0x01A126;
const STOPPED: u32 = 0x01A15C;
const TASK_1A202: u32 = 0x01A202;
const TASK_1A30A: u32 = 0x01A30A;
const TASK_19262: u32 = 0x019262;
const TASK_19ACE: u32 = 0x019ACE;
const FALLING: u32 = 0x01A29E;
const SPIRALLING: u32 = 0x01A392;

impl Game {
    /// `func_001336` with the registers (the object's animation; C:
    /// it ended).
    fn y_obj_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
    }

    /// `func_0015E6` and `func_006E7C` with the registers: C when the
    /// object took its last hit.
    fn y_obj_hit(&mut self, r: &mut Regs) {
        (r.d0, r.x) = self.r_0015e6(r.a4);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e7c(r.d1, r.d2, r.a0, r.a4, r.a5);
    }

    /// The running task's record (`func_0005F8`), its angle one down
    /// (X and C the borrow); at 0 it ends (`func_000560`).
    fn y_count_down_end(&mut self, r: &mut Regs) {
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        let count = self.read8(r.a6 + TASK_ANGLE);
        self.write8(r.a6 + TASK_ANGLE, count.wrapping_sub(1));
        r.c = count == 0;
        r.x = r.c;
        if count == 1 {
            self.y_end(r);
        }
    }

    /// A random number (`func_001BE2`) with the registers.
    fn y_random(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
    }

    /// `func_01A022`, the object waiting: when hit for the last time,
    /// `func_01A04C`; otherwise `WAIT_ADE4` down, and at 0 the routine
    /// `func_019988` again with the timer 0xF0 and the count 1.  Gives
    /// back the registers as the original leaves them.
    pub fn r_01a022(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_obj_hit(r);
        if r.c {
            (r.d6, r.a6, r.x, r.c) = self.r_01a04c(r.d4, r.d5, r.d6, r.a4);
            return;
        }
        let count = self.read8(WAIT_ADE4);
        self.write8(WAIT_ADE4, count.wrapping_sub(1));
        r.c = count == 0;
        r.x = r.c;
        if count != 1 {
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, WAITING);
        self.write8(r.a4 + OBJ_TIMER, 0xF0);
        self.write8(WAIT_ADE4, 1);
        r.c = false;
    }

    /// `loc_01A04C`: the object's kind 1, `ANIM_DE2F4` from its start,
    /// the timer 0x70, `func_01A082` its routine, and the task
    /// `func_01A202` (priority 0x32, `func_0003E8`).  Gives back d6, a6,
    /// X and C.
    pub fn r_01a04c(&mut self, d4: u32, d5: u32, d6: u32, a4: u32) -> (u32, u32, bool, bool) {
        self.write8(a4 + OBJ_KIND, 1);
        self.write8(a4 + OBJ_ANIM_STEP, 0);
        self.write8(a4 + OBJ_ANIM_TIMER, 0);
        self.write32(a4 + OBJ_ANIM, ANIM_DE2F4);
        self.write8(a4 + OBJ_TIMER, 0x70);
        self.write32(a4 + OBJ_ROUTINE, FIRST_ANIM);
        self.r_0003e8(d4, d5, d6, 0x32, TASK_1A202)
    }

    /// `func_01A082`: the animation on (`func_001336`); at its end
    /// `ANIM_DE372` and `func_01A09A`, run at once.  Gives back the
    /// registers as the original leaves them.
    pub fn r_01a082(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.y_obj_animate(regs);
        if !regs.c {
            return;
        }
        self.write32(regs.a4 + OBJ_ROUTINE, SECOND_ANIM);
        self.write32(regs.a4 + OBJ_ANIM, ANIM_DE372);
        regs.c = false;
        self.r_01a09a(regs);
    }

    /// `func_01A09A`: the animation on (`func_001336`); at its end
    /// `func_01A0BC` (the step 0, the table `TABLE_1A41A`, the byte at
    /// 0x11 4), run at once.  Gives back the registers as the original
    /// leaves them.
    pub fn r_01a09a(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.y_obj_animate(regs);
        if !regs.c {
            return;
        }
        self.write32(regs.a4 + OBJ_ROUTINE, WALKING);
        self.write8(regs.a4 + OBJ_STEP, 0);
        self.write32(regs.a4 + OBJ_TABLE, TABLE_1A41A);
        self.write8(regs.a4 + OBJ_11, 4);
        regs.c = false;
        self.r_01a0bc(regs);
    }

    /// `func_01A0BC`, the object walking: `SPEED` 0xD0 or 0x90 by
    /// bit 0 of its facing, a frame of it (`func_004096`); hit for the
    /// last time, `func_01A0FC`.  Otherwise its sprite at depth 8
    /// (`func_0007D2`), and when the timer's high nibble is out,
    /// `ANIM_DE384`, sound 0x68 and `func_01A15C`, run at once.  Gives
    /// back the registers as the original leaves them.
    pub fn r_01a0bc(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let speed = if self.read8(r.a4 + OBJ_FACING) & 0x01 != 0 { 0xD0 } else { 0x90 };
        self.write16(SPEED, speed);
        self.write8(r.a4 + OBJ_26, 0);
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c) =
            self.r_004096(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        if r.c {
            (r.d0, r.d6, r.a6, r.x, r.c) = self.r_01a0fc(r.d0, r.d4, r.d5, r.d6, r.a4);
            return;
        }
        r.d0 = 8;
        r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
        let timer = self.read8(r.a4 + OBJ_TIMER) & 0xF0;
        r.d0 = low_byte(r.d0, timer);
        r.c = false;
        if timer != 0 {
            return;
        }
        self.write32(r.a4 + OBJ_ANIM, ANIM_DE384);
        self.write32(r.a4 + OBJ_ROUTINE, STOPPED);
        self.y_sound(r, 0x68);
        self.r_01a15c(r);
    }

    /// `loc_01A0FA`: nothing.
    pub fn r_01a0fa(&mut self) {}

    /// `loc_01A0FC`: `ANIM_DE306`, `func_01A126` the routine, the task
    /// `func_01A30A` (priority 0x32) and sound 0x5F.  Gives back d0,
    /// d6, a6, X and C.
    pub fn r_01a0fc(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, a4: u32) -> (u32, u32, u32, bool, bool) {
        self.write32(a4 + OBJ_ANIM, ANIM_DE306);
        self.write32(a4 + OBJ_ROUTINE, ENDING);
        let (d6, a6, x, _) = self.r_0003e8(d4, d5, d6, 0x32, TASK_1A30A);
        let d0 = low_byte(d0, 0x5F);
        self.r_0034b6(d0);
        (d0, d6, a6, x, false)
    }

    /// `func_01A126`: the animation on (`func_001336`); at its end the
    /// object's kind 0, the task `func_019262` (priority 0x32) and the
    /// sprite record ended (`func_000890`).  Gives back the registers
    /// as the original leaves them.
    pub fn r_01a126(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_obj_animate(r);
        if !r.c {
            return;
        }
        self.write8(r.a4 + OBJ_KIND, 0);
        self.y_start_task(r, 0x32, TASK_19262);
        r.c = self.r_000890();
    }

    /// `func_01A15C`, the object stopped: hit for the last time,
    /// `func_01A0FC`; otherwise the animation on (`func_001336`), and at
    /// its end sound 0x84, the tasks (`func_01A18C`), the timer 0x70
    /// and `func_01A0BC` again.  Gives back the registers as the
    /// original leaves them.
    pub fn r_01a15c(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_obj_hit(r);
        if r.c {
            (r.d0, r.d6, r.a6, r.x, r.c) = self.r_01a0fc(r.d0, r.d4, r.d5, r.d6, r.a4);
            return;
        }
        self.y_obj_animate(r);
        if !r.c {
            return;
        }
        r.d0 = low_byte(r.d0, 0x84);
        self.r_0034b6(r.d0);
        (r.d0, r.x) = self.r_01a18c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.write32(r.a4 + OBJ_ROUTINE, WALKING);
        self.write8(r.a4 + OBJ_TIMER, 0x70);
        r.c = false;
    }

    /// `func_01A18C`: the object's position (`func_001A10`, `func_001A18`)
    /// and eight tasks with sprites `func_019ACE` (depth 6, priority
    /// 0x32; flags 0x81), at its position's low bytes, their angles
    /// 0x20 apart from 0 or 0x10 by the parity of `WAIT_ADE4` (one up
    /// first), until one can't be had.  d4-d6, a4-a6 kept (movem).
    /// Gives back d0 and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_01a18c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, bool) {
        self.write8(TO_MAKE, 8);
        self.r_001a10(a4);
        self.r_001a18(a4);
        let count = self.read8(WAIT_ADE4).wrapping_add(1);
        self.write8(WAIT_ADE4, count);
        let d0 = low_byte(d0, count & 1);
        self.write8(NEXT_ANGLE, if count & 1 != 0 { 0 } else { 0x10 });
        let (mut d5, mut d6, mut a5) = (d5, d6, a5);
        loop {
            let (next_d5, next_d6, sprite, task, x, failed) =
                self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 6, 0x32, TASK_19ACE);
            (d5, d6, a5) = (next_d5, next_d6, sprite);
            if failed {
                return (d0, x);
            }
            self.write8(sprite + SPRITE_FLAGS, 0x81);
            self.write8(task + TASK_COUNT, 0);
            let angle = self.read8(NEXT_ANGLE);
            self.write8(task + TASK_ANGLE, angle);
            self.write8(NEXT_ANGLE, angle.wrapping_add(0x20));
            let x_low = self.read8(POS_X_LOW);
            self.write8(task + TASK_X_LOW_19, x_low);
            let y_low = self.read8(POS_Y_LOW);
            self.write8(task + TASK_Y_LOW_1A, y_low);
            let left = self.read8(TO_MAKE);
            self.write8(TO_MAKE, left.wrapping_sub(1));
            if left == 1 {
                return (d0, false);
            }
        }
    }

    /// `func_01A228` (`func_01A202` after its second yield): a task
    /// with a sprite `func_01A29E` (depth 6, priority 0x81; flags
    /// 0x81) of a random kind (0-3), at `CENTRE` plus random 0-15 each
    /// way, its byte at 0x20 0, rate 0xFB00, a random angle; then this
    /// task's count down (`TASK_ANGLE`), ending at 0.  Gives back the
    /// registers as the original leaves them.
    pub fn r_01a228(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_make_task(r, 6, 0x81, FALLING);
        self.write8(r.a5 + SPRITE_FLAGS, 0x81);
        self.y_random(r);
        r.d0 = low_byte(r.d0, r.d0 as u8 & 3);
        self.write8(r.a6 + TASK_KIND, r.d0 as u8);
        let x = self.y_random_near(r, CENTRE_X_LOW);
        self.write16(r.a6 + TASK_X, x);
        let y = self.y_random_near(r, CENTRE_Y_LOW);
        self.write16(r.a6 + TASK_Y, y);
        self.write8(r.a6 + TASK_20, 0);
        self.write16(r.a6 + TASK_RATE, 0xFB00);
        self.y_random(r);
        self.write8(r.a6 + TASK_ANGLE, r.d0 as u8);
        self.y_count_down_end(r);
    }

    /// A random 0-15 plus the byte at `at` (d0's and d1's low words).
    fn y_random_near(&mut self, r: &mut Regs, at: u32) -> u16 {
        self.y_random(r);
        let offset = r.d0 as u16 & 0xF;
        let base = u16::from(self.read8(at));
        r.d1 = low_word(r.d1, base);
        let value = offset.wrapping_add(base);
        r.d0 = low_word(r.d0, value);
        value
    }

    /// `func_01A29E`, a task: moved by `func_017780`/`func_01777C` of its
    /// angle at distance 7, its sprite's frame from `FALL_FRAMES` by its
    /// kind and frame count, `func_00C742` with 0x28, its offset by its
    /// rate (`func_001ACA`), placed (`func_001B94`); it ends
    /// (`func_00121A`) unless its byte at 0x20 is negative.  Gives back
    /// the registers as the original leaves them.
    pub fn r_01a29e(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let angle = self.read8(r.a6 + TASK_ANGLE);
        r.d0 = low_byte(r.d0, angle);
        r.d1 = 7;
        r.d0 = self.r_017780(r.d0, r.d1, r.d3);
        let x = self.read16(r.a6 + TASK_X).wrapping_add(r.d0 as u16);
        self.write16(r.a6 + TASK_X, x);
        r.d0 = low_byte(r.d0, angle);
        r.d1 = 7;
        r.d0 = self.r_01777c(r.d0, r.d1, r.d3);
        let y = self.read16(r.a6 + TASK_Y).wrapping_add(r.d0 as u16);
        self.write16(r.a6 + TASK_Y, y);
        let kind = self.read8(r.a6 + TASK_KIND);
        r.d1 = low_byte(r.d1, kind.wrapping_add(kind));
        let frames = self.read8(r.a6 + TASK_FRAMES).wrapping_add(1);
        self.write8(r.a6 + TASK_FRAMES, frames);
        let index = ((frames >> 2) & 1).wrapping_add(r.d1 as u8);
        r.d0 = low_word(r.d0, index as i8 as i16 as u16);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        let frame = self.read8(FALL_FRAMES.wrapping_add(index as i8 as i32 as u32));
        self.write8(r.a5 + SPRITE_FRAME, frame);
        r.d0 = 0x28;
        self.r_00c742(r.d0, r.a6);
        (r.d0, r.x, _) = self.r_001aca(r.d0, r.a6);
        (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
        r.c = false;
        if self.read8(r.a6 + TASK_20) & 0x80 == 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        }
    }

    /// `loc_01A31C`: nothing.
    pub fn r_01a31c(&mut self) {}

    /// `func_01A328` (`func_01A30A` after its second yield): its
    /// count (`TASK_COUNT`) up; from 6 on, sound 0x68 and a task with
    /// a sprite `func_01A392` (depth 6, priority 0x80; flags 0x81,
    /// `ANIM_DE2C6`) at `CENTRE` with a random angle, count 0xC and
    /// rate 0; then this task's `TASK_ANGLE` down, ending at 0.  Gives
    /// back the registers as the original leaves them.
    pub fn r_01a328(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let count = self.read8(r.a6 + TASK_COUNT).wrapping_add(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = count == 0;
        r.c = count < 6;
        if r.c {
            return;
        }
        self.y_sound(r, 0x68);
        self.y_make_task(r, 6, 0x80, SPIRALLING);
        self.write32(r.a6 + TASK_ANIM, ANIM_DE2C6);
        self.write8(r.a5 + SPRITE_FLAGS, 0x81);
        self.y_random(r);
        self.write8(r.a6 + TASK_ANGLE, r.d0 as u8);
        self.write8(r.a6 + TASK_COUNT, 0xC);
        self.write16(r.a6 + TASK_RATE, 0);
        let x = self.read16(CENTRE_X);
        self.write16(r.a6 + TASK_X, x);
        let y = self.read16(CENTRE_Y);
        self.write16(r.a6 + TASK_Y, y);
        self.y_count_down_end(r);
    }

    /// `func_01A392`, a task: its animation on (`func_0013D6`), the
    /// velocity's y 0xFC60 less its rate's high byte (one up each frame),
    /// moved (`func_001AA2`), then out along its angle by
    /// `func_017780` (x, at its count) and `func_01777C` (y, at half
    /// of it), its count down to 0; placed (`func_001B7E`), it ends
    /// (`func_00121A`) once its y's high byte isn't 0.  Gives back the
    /// registers as the original leaves them.
    pub fn r_01a392(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_animate(r);
        let rate = self.read8(r.a6 + TASK_RATE_HIGH).wrapping_add(1);
        self.write8(r.a6 + TASK_RATE_HIGH, rate);
        let velocity = 0xFC60u16.wrapping_sub(u16::from(rate));
        self.write16(VELOCITY_Y, velocity);
        r.d0 = low_byte(r.d0, (velocity >> 8) as u8);
        r.d1 = low_byte(r.d1, 0);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        let angle = self.read8(r.a6 + TASK_ANGLE);
        let count = self.read8(r.a6 + TASK_COUNT);
        r.d0 = low_byte(r.d0, angle);
        r.d1 = low_byte(r.d1, count);
        r.d0 = self.r_017780(r.d0, r.d1, r.d3);
        let x = self.read16(r.a6 + TASK_X).wrapping_add(r.d0 as u16);
        self.write16(r.a6 + TASK_X, x);
        r.d1 = low_byte(r.d1, count >> 1);
        r.d0 = low_byte(r.d0, angle);
        r.d0 = self.r_01777c(r.d0, r.d1, r.d3);
        let y = self.read16(r.a6 + TASK_Y).wrapping_add(r.d0 as u16);
        self.write16(r.a6 + TASK_Y, y);
        let next = count.wrapping_sub(1);
        r.d0 = low_byte(r.d0, next);
        if next & 0x80 == 0 {
            self.write8(r.a6 + TASK_COUNT, next);
        }
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        r.c = false;
        if self.read8(r.a6 + TASK_Y_HIGH) != 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        }
    }
}
