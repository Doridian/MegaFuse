//! The task `func_018D80` (made by `func_01884A`, `w_choice.rs`) and
//! the steps it yields to (0x18D80-0x19004), in the exit-* scripts:
//! with `ROW` below `LIMIT`, or the two nibbles of `ENTRY_BITS` the
//! same, it is the choice's task (`func_018876`, `yb_choice.rs`).
//! Else the screen is loaded (`func_018D1C`) and, unless `func_017952`
//! finds all four bits set, `func_018DDC` makes a sprite at object 0's
//! place (`OBJECT_0_X_LOW`, `OBJECT_0_Y_LOW`) that rises, waits, rises
//! faster (by `STEPS` / 8) to 0x40 and then turns in to the centre
//! about (0x80, 0x88) on a shrinking radius (`func_017770`,
//! `func_01776C`, a `func_01900E` task left behind each frame); then
//! the sprite freed, 0x20 bytes of `COPY_FROM` to `COPY_TO`, the
//! Z80's command 0x73, the bits of `ENTRY_BITS` kept in its high
//! nibble, and either the choice again (`func_018896`) or, with all
//! four, `LIMIT` up, a fade, and on to `func_019004`.
//!
//! The yield of `func_018D80` with all four bits set is an
//! instruction placed after `func_019060` (0x1906A; its resume
//! 0x1906E); `func_019060` never reaches it.

use crate::rewritten::y_screen::{low_byte, low_word};
use crate::rewritten::yb_wait::WAIT;
use crate::{Game, Regs};

/// The byte compared with `LIMIT` (`w_choice.rs`).
const ROW: u32 = 0xFFFF_953A;
const LIMIT: u32 = 0xFFFF_953B;
/// Bits of entries (`w_choice.rs`): its high nibble the low one as it
/// was.
const ENTRY_BITS: u32 = 0xFFFF_953D;
/// `ENTRY_BITS`'s high nibble, while `func_018D80` compares it.
const HIGH_80C5: u32 = 0xFFFF_80C5;
/// Two counts: `RISE` (`ACROSS`, `w_choice.rs`) up to 0xFF, its high
/// nibble + 1 taken off the angle each frame; `STEPS` (`DOWN`) the
/// rise's speed x 8, then the frames to the next step of the radius.
const RISE: u32 = 0xFFFF_ADD4;
const STEPS: u32 = 0xFFFF_ADD5;
/// Flags of the game (`b_control.rs`): bit 3 set by `func_018DDC`.
const MODE: u32 = 0xFFFF_9540;
const MODE_BIT3: u8 = 1 << 3;
/// Cleared by `func_018D80`.
const BYTE_80DF: u32 = 0xFFFF_80DF;
/// While it isn't 0 `func_018DCE` does nothing more.
const BYTE_954F: u32 = 0xFFFF_954F;
/// Object 0's place (`x_object.rs`): the low bytes of its x and y.
const OBJECT_0_X_LOW: u32 = 0xFFFF_A44B;
const OBJECT_0_Y_LOW: u32 = 0xFFFF_A44D;
/// The running task's sprite record (index and address).
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;
/// `func_018FA4` copies `COPY_SIZE` bytes from `COPY_FROM` to
/// `COPY_TO`.
const COPY_FROM: u32 = 0x00FF_5980;
const COPY_TO: u32 = 0xFFFF_9FDA;
const COPY_SIZE: u32 = 0x20;

/// A task's fields: its sprite record's index, its sprite's place
/// (across and down, bytes), the radius and angle it turns in by, its
/// frames and a byte (cleared) and its animation (a long).
const TASK_SLOT: u32 = 0x03;
const TASK_X: u32 = 0x0D;
const TASK_Y: u32 = 0x0E;
const TASK_RADIUS: u32 = 0x0F;
const TASK_ANGLE: u32 = 0x10;
const TASK_FRAMES: u32 = 0x12;
const TASK_13: u32 = 0x13;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's fields (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;
/// The flags of the sprite made here.
const SHOWN: u8 = 0x85;
/// The sprite rises fast until its y is below this.
const RISE_TOP: u8 = 0x40;

/// In the ROM: an animation.
const ANIM_DE276: u32 = 0x0DE276;

/// Routines: the task made, and where this one goes on.
const TASK_1966A: u32 = 0x01966A;
const ALL_SET: u32 = 0x01906E;
const AFTER_LOAD: u32 = 0x018DCE;
const AFTER_START: u32 = 0x018DF4;
const AFTER_MADE: u32 = 0x018E6E;
const AFTER_RISE: u32 = 0x018E9E;
const AFTER_PAUSE: u32 = 0x018EAE;
const AFTER_TOP: u32 = 0x018EEE;
const TURNING: u32 = 0x018F06;
const AFTER_TURN: u32 = 0x018FA4;
const AFTER_COPY: u32 = 0x018FD6;
const AFTER_FADE: u32 = 0x019004;

impl Game {
    /// `func_018D80`, a task: with `ROW` below `LIMIT`, or the high
    /// nibble of `ENTRY_BITS` (kept in `HIGH_80C5`) equal to its low
    /// one, on as `loc_018876` (`yb_choice.rs`).  Else `BYTE_80DF`
    /// cleared, the screen loaded (`func_018D1C`), a task `func_01966A`
    /// (priority 0x28), and a yield: to 0x1906E when `func_017952`
    /// finds all four bits set, else to `func_018DCE`.
    pub fn r_018d80(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let row = self.read8(ROW);
        r.d0 = low_byte(r.d0, row);
        if row < self.read8(LIMIT) {
            r.c = true;
            self.r_018876(r);
            return;
        }
        let bits = self.read8(ENTRY_BITS);
        r.d0 = low_byte(r.d0, bits >> 4);
        r.x = bits & 0x08 != 0;
        self.write8(HIGH_80C5, bits >> 4);
        let low = self.read8(ENTRY_BITS) & 0x0F;
        r.d0 = low_byte(r.d0, low);
        let high = self.read8(HIGH_80C5);
        if low == high {
            r.c = false;
            self.r_018876(r);
            return;
        }
        self.write8(BYTE_80DF, 0);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5) = self.r_018d1c(
            r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x,
        );
        if self.restarted.is_some() {
            return;
        }
        self.y_start_task(r, 0x28, TASK_1966A);
        let all;
        (r.d0, all) = self.r_017952(r.d0);
        self.yb_yield(r, if all { ALL_SET } else { AFTER_LOAD });
    }

    /// `func_018DCE`: `func_001BB4` (the task may stop there); unless
    /// `BYTE_954F` is set, on in `func_018DDC`.
    pub fn r_018dce(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_stopped(r) {
            return;
        }
        r.c = false;
        if self.read8(BYTE_954F) == 0 {
            self.r_018ddc(r);
        }
    }

    /// `loc_018DDC`: `MODE_BIT3` set and a wait of 0x30 frames (on in
    /// `func_018DF4`).
    pub fn r_018ddc(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let mode = self.read8(MODE);
        self.write8(MODE, mode | MODE_BIT3);
        self.yb_sleep(r, 0x30, AFTER_START);
    }

    /// `func_018DF4` (after the wait): a sprite (depth 0xFA, frame 0,
    /// flags `SHOWN`) as the task's, its frames and byte 0x13 cleared,
    /// `ANIM_DE276`; its place object 0's (x - 9, y) in the task and
    /// the sprite; angle 0, radius 0x80, `RISE` and `STEPS` 0; and a
    /// wait of 0x16 frames (on in `func_018E6E`).
    pub fn r_018df4(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_waiting(r) {
            return;
        }
        self.yb_own(r);
        r.d0 = low_byte(r.d0, 0xFA);
        (r.d5, r.a5, r.x, r.c) = self.r_000796(
            r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x,
        );
        self.write8(r.a6 + TASK_SLOT, r.d5 as u8);
        self.write8(r.a6 + TASK_FRAMES, 0);
        self.write8(r.a6 + TASK_13, 0);
        self.write32(r.a6 + TASK_ANIM, ANIM_DE276);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.write8(r.a5 + SPRITE_FLAGS, SHOWN);
        let x = self.read8(OBJECT_0_X_LOW).wrapping_sub(9);
        r.d0 = low_byte(r.d0, x);
        self.write8(r.a6 + TASK_X, x);
        self.write8(r.a5 + SPRITE_X_LOW, x);
        let y = self.read8(OBJECT_0_Y_LOW);
        r.d0 = low_byte(r.d0, y);
        self.write8(r.a6 + TASK_Y, y);
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        self.write8(r.a6 + TASK_ANGLE, 0);
        self.write8(r.a6 + TASK_RADIUS, 0x80);
        self.write8(RISE, 0);
        self.write8(STEPS, 0);
        self.yb_sleep(r, 0x16, AFTER_MADE);
    }

    /// `func_018E6E`, each frame of the wait: the sprite a line up,
    /// frame 0.  After it, a wait of 0x20 frames with the record at
    /// depth 0x32 (`func_0007D2`; on in `func_018E9E`).
    pub fn r_018e6e(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let up = self.read8(r.a6 + TASK_Y).wrapping_sub(1);
        self.write8(r.a6 + TASK_Y, up);
        self.write8(r.a5 + SPRITE_Y_LOW, up);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        if self.yb_waiting(r) {
            return;
        }
        self.yb_own(r);
        self.write16(r.a6 + WAIT, 0x20);
        r.d0 = low_byte(r.d0, 0x32);
        r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
        self.yb_yield(r, AFTER_RISE);
    }

    /// `func_018E9E`, each frame of the wait: the sprite's frame 0;
    /// after it a yield to `func_018EAE`.
    pub fn r_018e9e(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, 0);
        if self.yb_waiting(r) {
            return;
        }
        self.yb_yield(r, AFTER_PAUSE);
    }

    /// `func_018EAE`, each frame: the sprite up by `STEPS` / 8 (one
    /// more each frame), leaving a `func_01900E` task, frame 0; when it
    /// would be above `RISE_TOP`, a wait of 0x18 frames instead (on in
    /// `func_018EEE`).
    pub fn r_018eae(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let speed = self.read8(STEPS) >> 3;
        r.d0 = low_byte(r.d0, speed);
        let y = self.read8(r.a6 + TASK_Y).wrapping_sub(speed);
        r.d1 = low_byte(r.d1, y);
        if y < RISE_TOP {
            self.yb_sleep(r, 0x18, AFTER_TOP);
            return;
        }
        self.write8(r.a6 + TASK_Y, y);
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        let steps = self.read8(STEPS);
        self.write8(STEPS, steps.wrapping_add(1));
        self.yb_trail(r);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        r.c = false;
    }

    /// `func_01900E` with the registers (`x_tasks.rs`): a task left
    /// where the running one is, every fifth call.
    fn yb_trail(&mut self, r: &mut Regs) {
        (r.d5, r.d6, r.a5, r.a6, r.x) = self.r_01900e(
            r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6,
        );
    }

    /// `func_018EEE`, each frame of the wait: the sprite's frame 0;
    /// after it the Z80's command 0x74 and a yield to `func_018F06`.
    pub fn r_018eee(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, 0);
        if self.yb_waiting(r) {
            return;
        }
        self.y_sound(r, 0x74);
        self.yb_yield(r, TURNING);
    }

    /// `func_018F06`, each frame: `RISE` up (to 0xFF), the angle back
    /// by its high nibble + 1; every second frame (`STEPS`) the radius
    /// one less, and at 0 the sprite hidden (y 0xFF..) and freed
    /// (`func_000890`) and a wait of 0x30 frames (on in `func_018FA4`).
    /// Else a `func_01900E` task left, and the sprite at the angle and
    /// radius about the centre (`func_017770` across; `func_01776C` + 8
    /// down), frame 0.
    pub fn r_018f06(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let rise = self.read8(RISE);
        if rise != 0xFF {
            self.write8(RISE, rise + 1);
        }
        let turn = (self.read8(RISE) >> 4).wrapping_add(1);
        r.d0 = low_byte(r.d0, turn);
        let angle = self.read8(r.a6 + TASK_ANGLE).wrapping_sub(turn);
        r.d1 = low_byte(r.d1, angle);
        self.write8(r.a6 + TASK_ANGLE, angle);
        let steps = self.read8(STEPS).wrapping_add(1);
        self.write8(STEPS, steps);
        if steps >= 2 {
            self.write8(STEPS, 0);
            let radius = self.read8(r.a6 + TASK_RADIUS).wrapping_sub(1);
            self.write8(r.a6 + TASK_RADIUS, radius);
            r.d0 = low_byte(r.d0, radius);
            if radius == 0 {
                self.write8(r.a5 + SPRITE_Y, 0xFF);
                r.c = self.r_000890();
                self.yb_sleep(r, 0x30, AFTER_TURN);
                return;
            }
        }
        self.yb_trail(r);
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_ANGLE));
        r.d1 = low_byte(r.d1, self.read8(r.a6 + TASK_RADIUS));
        (r.d0, r.x) = self.r_017770(r.d0, r.d1, r.d3);
        self.write8(r.a6 + TASK_X, r.d0 as u8);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        self.write8(r.a5 + SPRITE_X_LOW, r.d0 as u8);
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_ANGLE));
        r.d1 = low_byte(r.d1, self.read8(r.a6 + TASK_RADIUS));
        (r.d0, r.x) = self.r_01776c(r.d0, r.d1, r.d3);
        let (y, carry) = (r.d0 as u8).overflowing_add(8);
        r.d0 = low_byte(r.d0, y);
        r.x = carry;
        self.write8(r.a6 + TASK_Y, y);
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        r.c = false;
    }

    /// `func_018FA4` (after the wait): `COPY_SIZE` bytes of `COPY_FROM`
    /// to `COPY_TO`, the Z80's command 0x73, and a wait of 0x80 frames
    /// (on in `func_018FD6`).
    pub fn r_018fa4(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_waiting(r) {
            return;
        }
        for i in 0..COPY_SIZE {
            let byte = self.read8(COPY_FROM + i);
            self.write8(COPY_TO + i, byte);
        }
        r.a0 = COPY_FROM + COPY_SIZE;
        r.a1 = COPY_TO + COPY_SIZE;
        r.d0 = low_word(0, 0xFFFF);
        self.y_sound(r, 0x73);
        self.yb_sleep(r, 0x80, AFTER_COPY);
    }

    /// `func_018FD6` (after the wait): `ENTRY_BITS`'s low nibble kept
    /// in its high one too; unless `func_017952` finds all four bits
    /// set, the choice again (`func_018896`, `yb_choice.rs`); else
    /// `ENTRY_BITS` cleared, `LIMIT` up, a fade (`func_00A51A` with 4)
    /// and a yield to `func_019004`.
    pub fn r_018fd6(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_waiting(r) {
            return;
        }
        let bits = self.read8(ENTRY_BITS);
        let high = bits << 4;
        r.d0 = low_byte(r.d0, high);
        r.x = bits & 0x10 != 0;
        self.write8(ENTRY_BITS, bits | high);
        r.c = false;
        let all;
        (r.d0, all) = self.r_017952(r.d0);
        if !all {
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
                self.r_018896(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            return;
        }
        self.write8(ENTRY_BITS, 0);
        let limit = self.read8(LIMIT);
        self.write8(LIMIT, limit.wrapping_add(1));
        self.y_fade(r, 4);
        self.yb_yield(r, AFTER_FADE);
    }
}
