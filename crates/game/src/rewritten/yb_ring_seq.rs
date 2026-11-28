//! The yielding parts of two tasks of the screen `func_018772` sets up
//! (`w_ring.rs`), 0x18008-0x18562.
//!
//! A task of 0x17Fxx from `func_018008` on (where its yield in
//! `func_017FF6`, 0x18004, goes on; in exit-5-boss): each
//! frame the planes' scroll (`func_01859A`) and a count down, and the
//! task of index 0 that `func_0184A8` finds grows its radius by
//! 0xB0/0x100 a frame until it is `RADIUS_FULL`; then a wait of 0x80
//! frames, the Z80's command 0x79, and (`func_01808A`) its sprite
//! moved by `func_018564` until the count is 0, when `PICKED`,
//! `CHOSEN` and `CHOICE` are set and a fade starts (on in
//! `func_0180CA`, `w_ring.rs`).
//!
//! `func_0180DA`'s task (in ending): the screen set up and loaded, the
//! ring of tasks made (`func_01865E`), a task `func_0175A4`, a fade;
//! then waits and sprites made at random (`func_0183B8`, five times),
//! the task `func_01836C` (a sprite rising), the spiral of
//! `func_018314`, the scroll until `ACTIVE` is cleared, a fade and on
//! to `func_018306` (`w_ring.rs`).  `func_0184FE` (a task of
//! `func_0185EC`'s kind) no script runs: reviewed by reading.

use crate::rewritten::y_screen::low_byte;
use crate::{Game, Regs};

/// Set to 1 by `func_0180DA`; `func_0182AE` scrolls until it is 0
/// (`func_018314` clears it, `w_ring.rs`).
const ACTIVE: u32 = 0xFFFF_ADCF;
/// `func_0184A8`'s search: the priority looked for (`w_ring.rs`).
const SEARCH_PRIORITY: u32 = 0xFFFF_80C4;
/// Set by `func_01808A` at its end: 0xFF, 5 and 0 (`i_stage.rs`,
/// `w_choice.rs`).
const PICKED: u32 = 0xFFFF_954E;
const CHOSEN: u32 = 0xFFFF_953A;
const CHOICE: u32 = 0xFFFF_953C;
/// Plane A's offsets across and down (words, `h_scroll.rs`).
const H_OFFSET: u32 = 0xFFFF_D002;
const V_OFFSET: u32 = 0xFFFF_D402;
/// The velocity's y (a word, `a_motion.rs`) and the byte after it.
const VELOCITY_Y: u32 = 0xFFFF_9EC6;
const BYTE_9EC8: u32 = 0xFFFF_9EC8;
/// Compared with 0xFF by `func_01823A`: the spiral's animation.
const BYTE_A464: u32 = 0xFFFF_A464;

/// A task's fields: its angle and radius (`w_ring.rs`; for
/// `func_0180DA`'s own task the radius byte is a count, and its byte
/// at 0x0F the times `func_01817E` runs), its index, the radius's
/// fraction, its frames (`func_0184FE` compares them with 4), the
/// spiral's angle, radius and fraction (`func_018314`), a byte
/// `func_0184FE` sets, the rate `func_01836C` rises by (a word, with
/// `WAIT`'s place) and its animation (a long).
const TASK_ANGLE: u32 = 0x0C;
const TASK_RADIUS: u32 = 0x0D;
const TASK_INDEX: u32 = 0x0E;
const TASK_TIMES: u32 = 0x0F;
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_FRACTION: u32 = 0x10;
const TASK_FRAMES: u32 = 0x12;
const TASK_13: u32 = 0x13;
const SPIRAL_ANGLE: u32 = 0x16;
const SPIRAL_RADIUS: u32 = 0x17;
const SPIRAL_FRACTION: u32 = 0x18;
const TASK_19: u32 = 0x19;
const TASK_RATE: u32 = 0x1A;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's fields (`sprite.rs`).
const SPRITE_03: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y_LOW: u32 = 0x0B;

/// The radius `func_018008`'s task grows to.
const RADIUS_FULL: u8 = 0xAA;
/// The flags of the sprites made here.
const SHOWN: u8 = 0x81;

/// In the ROM: the list `func_0180DA` loads, animations.
const LIST_187F2: u32 = 0x0187F2;
const ANIM_DE238: u32 = 0x0DE238;
const ANIM_DE240: u32 = 0x0DE240;
const ANIM_DE24C: u32 = 0x0DE24C;

/// Routines: the tasks made, and where the tasks go on.
const TASK_175A4: u32 = 0x0175A4;
const RISING: u32 = 0x01836C;
const SPIRAL: u32 = 0x018314;
const AFTER_GROWN: u32 = 0x018078;
const AFTER_SOUND: u32 = 0x01808A;
const AFTER_MOVED: u32 = 0x0180CA;
const AFTER_FADE: u32 = 0x018138;
const AFTER_LOADED: u32 = 0x018154;
const RANDOM_LOOP: u32 = 0x01816C;
const AFTER_SHORT: u32 = 0x01817E;
const AFTER_RANDOM: u32 = 0x0181BE;
const AFTER_RISING: u32 = 0x018222;
const AFTER_SECOND: u32 = 0x01823A;
const SCROLLING: u32 = 0x0182AE;
const AFTER_SCROLL: u32 = 0x0182DE;
const AFTER_FADE_OUT: u32 = 0x0182F0;
const LAST_WAIT: u32 = 0x018306;
const RISE_ON: u32 = 0x018378;
const ANGLE_END: u32 = 0x018562;

impl Game {
    /// `func_01859A` with the registers: the planes' scroll by the
    /// running task's angle.
    fn yb_scroll(&mut self, r: &mut Regs) {
        (r.d0, r.d1) =
            self.r_01859a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `func_018008` (`func_017FF6` after its yield): the scroll
    /// (`func_01859A`), the count at `TASK_RADIUS` down to 0, and the
    /// task of index 0 found (`func_0184A8`, priority 0x64): until its
    /// radius is `RADIUS_FULL` it grows by 0xB0/0x100 (the carry of its
    /// fraction).  Then this task's angle 0x4D and count 0xFF,
    /// `func_017B46`, and a wait of 0x80 frames (on in `func_018078`).
    /// Gives back the registers as the original leaves them.
    pub fn r_018008(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.yb_scroll(r);
        let count = self.read8(r.a6 + TASK_RADIUS);
        if count != 0 {
            self.write8(r.a6 + TASK_RADIUS, count - 1);
        }
        self.write8(SEARCH_PRIORITY, 0x64);
        r.d0 = low_byte(r.d0, 0);
        (r.d0, r.d6, r.a6) = self.r_0184a8(r.d0, r.d6);
        let found = r.a6;
        if self.read8(found + TASK_RADIUS) != RADIUS_FULL {
            let (fraction, carry) = self.read8(found + TASK_FRACTION).overflowing_add(0xB0);
            self.write8(found + TASK_FRACTION, fraction);
            r.d1 = low_byte(r.d1, 0);
            let (radius, over) = self.read8(found + TASK_RADIUS).overflowing_add(u8::from(carry));
            r.d0 = low_byte(r.d0, radius);
            self.write8(found + TASK_RADIUS, radius);
            r.x = over;
            r.c = false;
            return;
        }
        self.yb_own(r);
        self.write8(r.a6 + TASK_ANGLE, 0x4D);
        self.write8(r.a6 + TASK_RADIUS, 0xFF);
        r.c = self.r_017b46(r.a6);
        self.yb_sleep(r, 0x80, AFTER_GROWN);
    }

    /// `func_018078` (after `func_018008`'s wait): the Z80's command
    /// 0x79 and a yield to `func_01808A`.
    pub fn r_018078(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_waiting(r) {
            return;
        }
        self.y_sound(r, 0x79);
        self.yb_yield(r, AFTER_SOUND);
    }

    /// `func_01808A`, each frame: the sprite placed (`func_018564`),
    /// then moved 0x20 up and 0x20 right; while the count at
    /// `TASK_RADIUS` isn't 0 it goes down and the animation goes on
    /// (`func_0013D6`).  At 0: `PICKED` 0xFF, `CHOSEN` 5, `CHOICE` 0,
    /// a fade (`func_00A51A` with 4) and a yield to `func_0180CA`.
    pub fn r_01808a(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.r_018564(r);
        let y = (r.d0 as u8).wrapping_sub(0x20);
        r.d0 = low_byte(r.d0, y);
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        let x = self.read8(r.a5 + SPRITE_X_LOW);
        self.write8(r.a5 + SPRITE_X_LOW, x.wrapping_add(0x20));
        let count = self.read8(r.a6 + TASK_RADIUS);
        if count != 0 {
            self.write8(r.a6 + TASK_RADIUS, count - 1);
            self.y_animate(r);
            return;
        }
        self.write8(PICKED, 0xFF);
        self.write8(CHOSEN, 5);
        self.write8(CHOICE, 0);
        self.y_fade(r, 4);
        self.yb_yield(r, AFTER_MOVED);
    }

    /// `func_0180DA`, a task: `ACTIVE` set, the screen set up
    /// (`func_018772`) and `LIST_187F2` loaded (`func_0094D8`), plane
    /// A's offsets (0xFF80, 0x80, `func_008664`), `func_0002DA`, the
    /// ring of tasks (`func_01865E`; the last one's radius
    /// `RADIUS_FULL`), a task `func_0175A4` (priority 0x32) with angle
    /// 0x10, a fade (`func_00A542` with 4), and a yield to
    /// `func_018138`.
    pub fn r_0180da(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(ACTIVE, 1);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_018772(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return;
        }
        if self.y_load(r, LIST_187F2) {
            return;
        }
        self.write16(H_OFFSET, 0xFF80);
        self.write16(V_OFFSET, 0x80);
        self.r_008664(r);
        self.r_0002da();
        (r.d5, r.d6, r.a5, r.a6) =
            self.r_01865e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.write8(r.a6 + TASK_RADIUS, RADIUS_FULL);
        self.y_start_task(r, 0x32, TASK_175A4);
        self.write8(r.a6 + TASK_ANGLE, 0x10);
        r.d0 = 4;
        (r.d0, r.d6, r.a0, r.a1, r.a6, r.x) =
            self.r_00a542(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a2, r.a3, r.a4, r.a5, r.a6);
        self.yb_yield(r, AFTER_FADE);
    }

    /// `func_018138`: `func_001BB4` (the task may stop there),
    /// `func_0185CC`, and a wait of 0x80 frames (on in `func_018154`).
    pub fn r_018138(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_stopped(r) {
            return;
        }
        self.r_0185cc();
        self.yb_sleep(r, 0x80, AFTER_LOADED);
    }

    /// `func_018154` (after the wait): the task's routine
    /// `func_01816C` from now on, `TASK_TIMES` 4, and on into it.
    pub fn r_018154(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_waiting(r) {
            return;
        }
        self.y_next(r, RANDOM_LOOP);
        self.write8(r.a6 + TASK_TIMES, 4);
        r.c = false;
        self.r_01816c(r);
    }

    /// `func_01816C`: a wait of 0x10 frames (on in `func_01817E`).
    pub fn r_01816c(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.yb_sleep(regs, 0x10, AFTER_SHORT);
    }

    /// `func_01817E` (after the wait): sprites made at random places
    /// (`func_0183B8`); while `TASK_TIMES` isn't 0 it goes down and the
    /// task goes back to `func_01816C`, else a wait of 0x80 frames (on
    /// in `func_0181BE`).
    pub fn r_01817e(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_waiting(r) {
            return;
        }
        (r.d0, r.d1, r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_0183b8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.yb_own(r);
        let times = self.read8(r.a6 + TASK_TIMES);
        if times == 0 {
            self.yb_sleep(r, 0x80, AFTER_RANDOM);
        } else {
            self.write8(r.a6 + TASK_TIMES, times - 1);
            self.y_next(r, RANDOM_LOOP);
        }
    }

    /// `func_0181BE` (after the wait): the Z80's command 0x14, a task
    /// `func_01836C` (depth 0x32, priority 0xB4) with animation
    /// `ANIM_DE238`, flags `SHOWN`, rate 0 and place (0xC0, 0x48); this
    /// task's angle 0x4D and radius 0, and a wait of 0x60 frames (on in
    /// `func_018222`).
    pub fn r_0181be(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_waiting(r) {
            return;
        }
        self.y_sound(r, 0x14);
        self.y_make_task(r, 0x32, 0xB4, RISING);
        self.write32(r.a6 + TASK_ANIM, ANIM_DE238);
        self.write8(r.a5 + SPRITE_FLAGS, SHOWN);
        self.write8(r.a6 + TASK_RATE, 0);
        self.write16(r.a6 + TASK_X, 0xC0);
        self.write16(r.a6 + TASK_Y, 0x48);
        self.yb_own(r);
        self.write8(r.a6 + TASK_ANGLE, 0x4D);
        self.write8(r.a6 + TASK_RADIUS, 0);
        self.yb_sleep(r, 0x60, AFTER_RISING);
    }

    /// `func_018222` (after the wait): another wait of 0x60 frames (on
    /// in `func_01823A`).
    pub fn r_018222(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_waiting(r) {
            return;
        }
        self.yb_sleep(r, 0x60, AFTER_SECOND);
    }

    /// `func_01823A`, each frame of its wait: the scroll
    /// (`func_01859A`) and the radius up to 0xFF.  After it: the spiral
    /// task `func_018314` (depth 0xA, priority 0xB4), its animation
    /// `ANIM_DE240` (`ANIM_DE24C` unless `BYTE_A464` is 0xFF), flags
    /// `SHOWN`, place (0xE0, 0x44), angle 0xC0, radius and fraction 0;
    /// and a yield to `func_0182AE`.
    pub fn r_01823a(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.yb_scroll(r);
        let radius = self.read8(r.a6 + TASK_RADIUS);
        if radius != 0xFF {
            self.write8(r.a6 + TASK_RADIUS, radius + 1);
        }
        if self.yb_waiting(r) {
            return;
        }
        self.y_make_task(r, 0xA, 0xB4, SPIRAL);
        self.write8(r.a6 + TASK_13, 0);
        self.write8(r.a6 + TASK_FRAMES, 0);
        self.write32(r.a6 + TASK_ANIM, ANIM_DE240);
        if self.read8(BYTE_A464) != 0xFF {
            self.write32(r.a6 + TASK_ANIM, ANIM_DE24C);
        }
        self.write8(r.a5 + SPRITE_FLAGS, SHOWN);
        self.write16(r.a6 + TASK_X, 0xE0);
        self.write16(r.a6 + TASK_Y, 0x44);
        self.write8(r.a6 + SPIRAL_ANGLE, 0xC0);
        self.write8(r.a6 + SPIRAL_RADIUS, 0);
        self.write8(r.a6 + SPIRAL_FRACTION, 0);
        self.yb_yield(r, SCROLLING);
    }

    /// `func_0182AE`, each frame: the scroll (`func_01859A`); while
    /// `ACTIVE` is set the radius goes up (to 0xFF, then the task goes
    /// on).  Then a wait of 0x100 frames (on in `func_0182DE`).
    pub fn r_0182ae(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.yb_scroll(r);
        let radius = self.read8(r.a6 + TASK_RADIUS);
        if radius != 0xFF && self.read8(ACTIVE) != 0 {
            let (more, carry) = radius.overflowing_add(1);
            self.write8(r.a6 + TASK_RADIUS, more);
            r.x = carry;
            r.c = carry;
            return;
        }
        self.yb_sleep(r, 0x100, AFTER_SCROLL);
    }

    /// `func_0182DE` (after the wait): a fade (`func_00A51A` with 0xA)
    /// and a yield to `func_0182F0`.
    pub fn r_0182de(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_waiting(r) {
            return;
        }
        self.y_fade(r, 0xA);
        self.yb_yield(r, AFTER_FADE_OUT);
    }

    /// `func_0182F0`: `func_001BB4` (the task may stop there), and a
    /// wait of 0x80 frames (on in `func_018306`, `w_ring.rs`).
    pub fn r_0182f0(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.yb_stopped(r) {
            return;
        }
        self.yb_sleep(r, 0x80, LAST_WAIT);
    }

    /// `func_01836C`, `func_0181BE`'s task: its animation on
    /// (`func_0013D6`; when it ends the task yields, to go on at
    /// 0x18378, a place the translation has no routine for); its rate
    /// up by 0x20 as the velocity's y (`func_001AA2`), and its sprite
    /// placed (`func_001B7E`), or, once its y's high byte isn't 0, the
    /// task ends (`func_00121A`).
    pub fn r_01836c(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_animate(r);
        if r.c {
            self.yb_yield(r, RISE_ON);
            return;
        }
        let rate = self.read16(r.a6 + TASK_RATE).wrapping_add(0x20);
        r.d0 = (r.d0 & 0xFFFF_0000) | u32::from(rate);
        self.write16(r.a6 + TASK_RATE, rate);
        self.write16(VELOCITY_Y, rate);
        self.write8(BYTE_9EC8, 0);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        r.c = false;
        if self.read8(r.a6 + TASK_Y) != 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        } else {
            (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        }
    }

    /// `func_0184FE`, a task of `func_0185EC`'s kind (no script runs
    /// it; reviewed by reading): the task of its index found
    /// (`func_0184A8`, priority 0x64), that one's radius plus 0x6E as
    /// this one's, the sprite placed (`func_018564`) with palette
    /// 0x7A + index, the Z80's command 0x78 on frame 0x1C at frames 4;
    /// its animation on (`func_0013D6`), and when that ends its byte at
    /// 0x19 set and a yield (to 0x18562, an `rts`).
    pub fn r_0184fe(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(SEARCH_PRIORITY, 0x64);
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_INDEX));
        (r.d0, r.d6, r.a6) = self.r_0184a8(r.d0, r.d6);
        let radius = self.read8(r.a6 + TASK_RADIUS).wrapping_add(0x6E);
        r.d0 = low_byte(r.d0, radius);
        self.yb_own(r);
        self.write8(r.a6 + TASK_RADIUS, radius);
        r.c = false;
        self.r_018564(r);
        let palette = self.read8(r.a6 + TASK_INDEX).wrapping_add(0x7A);
        r.d0 = low_byte(r.d0, palette);
        self.write8(r.a5 + SPRITE_03, palette);
        if self.read8(r.a5 + SPRITE_FRAME) == 0x1C && self.read8(r.a6 + TASK_FRAMES) == 4 {
            self.y_sound(r, 0x78);
        }
        self.y_animate(r);
        if r.c {
            self.write8(r.a6 + TASK_19, 1);
            self.yb_yield(r, ANGLE_END);
        }
    }
}
