//! The tasks of the screen `func_01A598` sets up (0x1B01E-0x1B6C8):
//! sprites that swing in by an angle and a growing radius
//! (`func_01B01E`, `func_01B154`), the fades and lists loaded between
//! the steps (`func_01B1BE` to `func_01B3CC`), the scroll counted down
//! (`func_01B404`), a sprite that rises (`func_01B4BA`), four sprites
//! and a mark moved by `CHOICE_AE89` (`func_01B4E0` to `func_01B5F8`),
//! and the screen made again from the start (`func_01B616`).  Each
//! sprite-task is made by `func_001222` and placed by `func_01B684`
//! from the words at `PLACE_X` to `PLACE_ANIM`.

use crate::{Game, Regs};

/// A task's fields (`task.rs`, `a_motion.rs`, `anim.rs`): its
/// position (words), its animation's counters, a count or index
/// (0x18), a step (0x19), and its animation (a long).
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_COUNT: u32 = 0x18;
const TASK_STEPS: u32 = 0x19;
/// The 0x1B byte of the fade's task (`j_colours.rs`): not 0 while it
/// runs.
const FADE_RUNNING: u32 = 0x1B;
const TASK_ANIM: u32 = 0x20;

/// A sprite record's fields (`sprite.rs`): its palette (3), frame (4),
/// flags (5), x (a word at 8, 9 its low byte), y (a word at 0xA, 0xB
/// its low byte).
const SPRITE_PALETTE: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;

/// What `func_01B684` places a task with: its x and y (words), the
/// sprite's flags (the low byte of the word at `PLACE_FLAGS`), the
/// animation (a long).
const PLACE_X: u32 = 0xFFFF_80C4;
const PLACE_Y: u32 = 0xFFFF_80C8;
const PLACE_Y_HIGH: u32 = 0xFFFF_80C8;
const PLACE_Y_LOW: u32 = 0xFFFF_80C9;
const PLACE_FLAGS: u32 = 0xFFFF_80CC;
const PLACE_FLAGS_LOW: u32 = 0xFFFF_80CD;
const PLACE_ANIM: u32 = 0xFFFF_80D0;

/// The list `func_0094D8` loads graphics by (a long).
const LOAD_LIST: u32 = 0xFFFF_92FA;
/// While bit 0 is set (a fade runs, `j_colours.rs`) some steps wait.
const STOPPED: u32 = 0xFFFF_A3FC;
/// Set to 1 once (`func_01B2C2`); while it is set the swinging sprites
/// end early.
const ONCE_AECF: u32 = 0xFFFF_AECF;
/// Set to 0xFF by `func_01B01E`.
const BYTE_AECE: u32 = 0xFFFF_AECE;
/// Bytes and words the steps set: `BYTE_AEC9` is also `func_01B404`'s
/// count down; `BYTE_AECA` 1 then 2.
const BYTE_AEC3: u32 = 0xFFFF_AEC3;
const BYTE_AEC8: u32 = 0xFFFF_AEC8;
const BYTE_AEC9: u32 = 0xFFFF_AEC9;
const BYTE_AECA: u32 = 0xFFFF_AECA;
const WORD_AEBE: u32 = 0xFFFF_AEBE;
const WORD_AEC6: u32 = 0xFFFF_AEC6;
/// `func_01B404` counts it (and `BYTE_AEC3`) up to 0xEF, and
/// `BYTE_AE6A` down with its count.
const BYTE_AE70: u32 = 0xFFFF_AE70;
const BYTE_AE70_TOP: u8 = 0xEF;
const BYTE_AE6A: u32 = 0xFFFF_AE6A;
/// Bit 0 set: `func_01B5C2`'s sprites look at `CHOICE_AE89`.
const WATCH_AECB: u32 = 0xFFFF_AECB;
/// The sprite being made (`func_01B4E0`), then `CHOICE_AE89` + 1.
const INDEX_AECC: u32 = 0xFFFF_AECC;
/// A choice (a byte): the mark's row (`func_01B5AE`), and the sprite
/// that blinks (`func_01B5C2`).
const CHOICE_AE89: u32 = 0xFFFF_AE89;
/// A word set to 0x100 (`func_01B49A`).
const WORD_8064: u32 = 0xFFFF_8064;
/// The scroll tables' first words (`h_scroll.rs`).
const H_VALUE: u32 = 0xFFFF_D002;
const V_VALUE: u32 = 0xFFFF_D402;
/// The VDP's H/V counter: the line is its high byte.
const HV_COUNTER: u32 = 0xC00008;
/// The line the steps wait for.
const LINE: u8 = 0xC0;

/// The swinging sprites' radii by step (ROM, 0x13 steps), and the
/// angles of `func_01B154`'s three.
const RADII: u32 = 0x01B00A;
const RADIUS_STEPS: u8 = 0x13;
const ANGLES: u32 = 0x01B1BA;
/// `func_01B01E`'s angle that makes the next sprites.
const LAST_ANGLE: u8 = 0x7F;
/// `func_01B4E0`'s sprites' y by index, and their animations (ROM).
const ROW_Y: u32 = 0x01B590;
const ROW_ANIMS: u32 = 0x01B596;
const ROW_SPRITES: u8 = 4;

/// The lists loaded (ROM).
const LIST_1B714: u32 = 0x01B714;
const LIST_1B718: u32 = 0x01B718;
const LIST_1B720: u32 = 0x01B720;
const LIST_1B73C: u32 = 0x01B73C;
const LIST_1B74E: u32 = 0x01B74E;
const LIST_1B756: u32 = 0x01B756;
const LIST_1B77C: u32 = 0x01B77C;
const LIST_1B78A: u32 = 0x01B78A;
/// Animations (ROM).
const ANIM_DC280: u32 = 0x0DC280;
const ANIM_DC292: u32 = 0x0DC292;
const ANIM_DC2B4: u32 = 0x0DC2B4;
const ANIM_DC2C0: u32 = 0x0DC2C0;
const ANIM_DC2D8: u32 = 0x0DC2D8;
const ANIM_DC308: u32 = 0x0DC308;
const ANIM_DC326: u32 = 0x0DC326;

/// The routines.
const R_01A68C: u32 = 0x01A68C;
const R_01B122: u32 = 0x01B122;
const R_01B154: u32 = 0x01B154;
const R_01B1BE: u32 = 0x01B1BE;
const R_01B1F2: u32 = 0x01B1F2;
const R_01B270: u32 = 0x01B270;
const R_01B294: u32 = 0x01B294;
const R_01B2C2: u32 = 0x01B2C2;
const R_01B318: u32 = 0x01B318;
const R_01B3A6: u32 = 0x01B3A6;
const R_01B3CC: u32 = 0x01B3CC;
const R_01B3FE: u32 = 0x01B3FE;
const R_01B404: u32 = 0x01B404;
const R_01B43C: u32 = 0x01B43C;
const R_01B49A: u32 = 0x01B49A;
const R_01B4BA: u32 = 0x01B4BA;
const R_01B4E0: u32 = 0x01B4E0;
const R_01B5AE: u32 = 0x01B5AE;
const R_01B5C2: u32 = 0x01B5C2;
const R_01B5F8: u32 = 0x01B5F8;
const R_01B664: u32 = 0x01B664;

/// The Z80's commands sent (`func_0034B6`).
const SOUND_01: u8 = 0x01;
const SOUND_7E: u8 = 0x7E;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// A byte sign-extended into a word (`ext.w`) in `reg`'s low word.
fn ext(reg: u32, b: u8) -> u32 {
    low_word(reg, b as i8 as u16)
}

impl Game {
    /// `func_0094D8` with the list `list`, on all the registers.  True
    /// when the game restarted meanwhile.
    fn z_load(&mut self, r: &mut Regs, list: u32) -> bool {
        self.write32(LOAD_LIST, list);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        self.restarted.is_some()
    }

    /// `func_0013D6`: the task's animation a frame on.
    fn z_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `func_00053E`: `routine` the running task's routine.
    fn z_next(&mut self, r: &mut Regs, routine: u32) {
        (r.x, r.c) = self.r_00053e(r.d6, routine);
    }

    /// `func_01A8CA`: the running task's sprite record freed and the
    /// task ended.
    fn z_end(&mut self, r: &mut Regs) {
        (r.d6, r.x, r.c) = self.r_01a8ca(r.d5, r.d6, r.x);
    }

    /// `func_000560`: the running task ends.
    fn z_end_task(&mut self, r: &mut Regs) {
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }

    /// `func_008664` with the registers: d0's low word, X and C from it.
    fn z_scroll(&mut self, r: &mut Regs) {
        let mut copy = *r;
        self.r_008664(&mut copy);
        (r.d0, r.x, r.c) = (copy.d0, copy.x, copy.c);
    }

    /// `func_001222`: a task running `routine` with a sprite record.
    fn z_sprite_task(&mut self, r: &mut Regs, depth: u8, priority: u8, routine: u32) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, priority, routine);
    }

    /// `func_0003E8`: a task running `routine`.
    fn z_task(&mut self, r: &mut Regs, priority: u8, routine: u32) {
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, priority, routine);
    }

    /// The new sprite-task placed (`func_01B684`) at `x`, `y`, with
    /// the flags word `flags` and the animation `anim`.
    fn z_place(&mut self, r: &mut Regs, x: u16, y: u16, flags: u16, anim: u32) {
        self.write16(PLACE_X, x);
        self.write16(PLACE_Y, y);
        self.write16(PLACE_FLAGS, flags);
        self.write32(PLACE_ANIM, anim);
        (r.d0, r.x, r.c) = self.r_01b684(r.a5, r.a6);
    }

    /// Polls the VDP's line until it is `LINE` or more: d0's low word
    /// the counter shifted down 8, X the last bit out.
    fn z_wait_line(&mut self, r: &mut Regs) {
        loop {
            let hv = self.read16(HV_COUNTER);
            r.d0 = low_word(r.d0, hv >> 8);
            r.x = hv & 0x80 != 0;
            if (hv >> 8) as u8 >= LINE {
                break;
            }
            self.pass(56);
        }
    }

    /// Adds 1 to the task's byte at `field` and compares it with
    /// `limit`: whether it got there (X the add's carry, C the
    /// compare's).
    fn z_count(&mut self, r: &mut Regs, field: u32, limit: u8) -> bool {
        let (count, carry) = self.read8(r.a6 + field).overflowing_add(1);
        self.write8(r.a6 + field, count);
        r.x = carry;
        r.c = count < limit;
        count == limit
    }

    /// A point on the swing: d1 the radius of step `step` (the step
    /// sign-extended in d1's low word first), d0 `angle`, then `sine`
    /// (`func_017770` or `func_01776C`).  Gives back d0's low byte.
    fn z_swing(&mut self, r: &mut Regs, angle: u8, step: u8, sine: fn(&mut Self, u32, u32, u32) -> (u32, bool)) -> u8 {
        r.d1 = ext(r.d1, step);
        let radius = self.read8(RADII.wrapping_add(step as i8 as u32));
        r.d1 = low_byte(r.d1, radius);
        r.d0 = low_byte(r.d0, angle);
        (r.d0, r.x) = sine(self, r.d0, r.d1, r.d3);
        r.d0 as u8
    }

    /// `func_01B01E`, a task's routine: its sprite swings out by the
    /// angle at `TASK_COUNT`, a radius from `RADII` a step
    /// (`func_017770` for x, `func_01776C` less 8 for y).  After 0x13
    /// steps it ends (`func_01A8CA`), but the one at angle 0x7F (unless
    /// `ONCE_AECF`): its sprite freed, its routine `func_01B122`,
    /// `LIST_1B714` loaded, then three tasks `func_01B154` (angles by
    /// index) and one `func_01B1BE` made.
    pub fn r_01b01e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let angle = self.read8(r.a6 + TASK_COUNT);
        let step = self.read8(r.a6 + TASK_STEPS);
        let x = self.z_swing(r, angle, step, Self::r_017770);
        self.write8(r.a5 + SPRITE_X_LOW, x);
        let step = self.read8(r.a6 + TASK_STEPS);
        let angle = self.read8(r.a6 + TASK_COUNT);
        let y = self.z_swing(r, angle, step, Self::r_01776c).wrapping_sub(8);
        r.d0 = low_byte(r.d0, y);
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        if !self.z_count(r, TASK_STEPS, RADIUS_STEPS) {
            return;
        }
        if self.read8(r.a6 + TASK_COUNT) != LAST_ANGLE {
            self.z_end(r);
            return;
        }
        if self.read8(ONCE_AECF) != 0 {
            r.c = false;
            self.z_end(r);
            return;
        }
        self.r_000890();
        self.z_next(r, R_01B122);
        if self.z_load(r, LIST_1B714) {
            return;
        }
        self.write8(BYTE_AECE, 0xFF);
        r.d2 = low_byte(r.d2, 0);
        loop {
            self.z_sprite_task(r, 0x0A, 0x3C, R_01B154);
            self.write8(r.a6 + TASK_COUNT, r.d2 as u8);
            self.write8(r.a6 + TASK_STEPS, 0);
            self.z_place(r, 0x80, 0x58, 0x400, ANIM_DC280);
            let index = (r.d2 as u8).wrapping_add(1);
            r.d2 = low_byte(r.d2, index);
            if index == 3 {
                break;
            }
        }
        self.z_sprite_task(r, 0x0A, 0x3C, R_01B1BE);
        self.z_place(r, 0x60, 0x59, 0x1080, ANIM_DC2B4);
    }

    /// `func_01B122`, a task's routine: once the VDP's line is 0xC0 or
    /// more, `BYTE_AEC8` 1, `BYTE_AEC9` 0x30, the vertical scroll 0xE0
    /// (`func_008664`), `func_0094B2`, and the task ends.
    pub fn r_01b122(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_wait_line(r);
        self.write8(BYTE_AEC8, 1);
        self.write8(BYTE_AEC9, 0x30);
        self.write16(V_VALUE, 0xE0);
        self.z_scroll(r);
        self.r_0094b2();
        self.z_end_task(r);
    }

    /// `func_01B154`, a task's routine: its animation, and its sprite
    /// swung by the angle of its index (`ANGLES`), as `func_01B01E`'s
    /// but y + 0x30; ended after 0x13 steps (`func_01A8CA`).  a0 is
    /// left `RADII`.
    pub fn r_01b154(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_animate(r);
        for (field, sine, offset) in [(SPRITE_X_LOW, Self::r_017770 as fn(&mut Self, u32, u32, u32) -> (u32, bool), 0u8), (SPRITE_Y_LOW, Self::r_01776c, 0x30)] {
            let index = self.read8(r.a6 + TASK_COUNT);
            r.d1 = ext(r.d1, index);
            let angle = self.read8(ANGLES.wrapping_add(index as i8 as u32));
            let step = self.read8(r.a6 + TASK_STEPS);
            r.a0 = RADII;
            let value = self.z_swing(r, angle, step, sine).wrapping_add(offset);
            r.d0 = low_byte(r.d0, value);
            self.write8(r.a5 + field, value);
        }
        if self.z_count(r, TASK_STEPS, RADIUS_STEPS) {
            self.z_end(r);
        }
    }

    /// `func_01B1BE`, a task's routine: ended at once if `ONCE_AECF`
    /// (`func_01A8CA`); else its animation, `LIST_1B718` loaded,
    /// `STOPPED` 0, a fade to white (`func_00A52E`, speed 6), and its
    /// routine `func_01B1F2`.
    pub fn r_01b1be(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.c = false;
        if self.read8(ONCE_AECF) != 0 {
            self.z_end(r);
            return;
        }
        self.z_animate(r);
        if self.z_load(r, LIST_1B718) {
            return;
        }
        self.write8(STOPPED, 0);
        r.d0 = 6;
        (r.d0, r.d6, r.a0, r.a1, r.a6, r.x, r.c) =
            self.r_00a52e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a2, r.a3, r.a4, r.a5, r.a6);
        self.z_next(r, R_01B1F2);
    }

    /// `func_01B1F2`, a task's routine: once the fade is over (`STOPPED`
    /// bit 0 clear) it ends (`func_01A8CA`), `LIST_1B720` is loaded and
    /// a task `func_01B270` made; then unless `ONCE_AECF`, a fade from
    /// white (`func_00A542`, speed 4) whose task's index goes to the new
    /// task's `TASK_COUNT`, `BYTE_AEC9` 0x30, `BYTE_AEC8` 0 and the
    /// vertical scroll 0.
    pub fn r_01b1f2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(STOPPED) & 1 != 0 {
            return;
        }
        self.z_end(r);
        if self.z_load(r, LIST_1B720) {
            return;
        }
        self.z_sprite_task(r, 0x1E, 0x50, R_01B270);
        self.z_place(r, 0x80, 0x58, 0x480, ANIM_DC292);
        r.c = false;
        if self.read8(ONCE_AECF) != 0 {
            return;
        }
        r.d0 = 4;
        (r.d0, r.d6, r.a0, r.a1, r.a6, r.x) =
            self.r_00a542(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a2, r.a3, r.a4, r.a5, r.a6);
        self.write8(r.a6 + TASK_COUNT, r.d0 as u8);
        self.write8(BYTE_AEC9, 0x30);
        self.write8(BYTE_AEC8, 0);
        self.write16(V_VALUE, 0);
        self.z_scroll(r);
    }

    /// `func_01B26E`: an `rts` (the end of `func_01B1F2`).
    pub fn r_01b26e(&mut self) {}

    /// `func_01B270`, a task's routine: its animation; once the task
    /// whose index is at `TASK_COUNT` (the fade's, `func_0005F8`) no
    /// longer runs, its routine `func_01B294` and that task's
    /// `TASK_COUNT` 0.  a6 is left that task's.
    pub fn r_01b270(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_animate(r);
        r.d6 = low_byte(r.d6, self.read8(r.a6 + TASK_COUNT));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        r.c = false;
        if self.read8(r.a6 + FADE_RUNNING) != 0 {
            return;
        }
        self.z_next(r, R_01B294);
        self.write8(r.a6 + TASK_COUNT, 0);
        r.c = false;
    }

    /// `func_01B294`, a task's routine: its animation; after 0x3C
    /// calls, its routine `func_01B2C2` with the animation
    /// `ANIM_DC2D8`.
    pub fn r_01b294(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_animate(r);
        if !self.z_count(r, TASK_COUNT, 0x3C) {
            return;
        }
        self.z_next(r, R_01B2C2);
        self.write32(r.a6 + TASK_ANIM, ANIM_DC2D8);
        self.write8(r.a6 + TASK_TIMER, 0);
        self.write8(r.a6 + TASK_STEP, 0);
        r.c = false;
    }

    /// `func_01B2C2`, a task's routine: its animation; when it ends (C),
    /// the task ends if `ONCE_AECF` (`func_01A8CA`), else `ONCE_AECF`
    /// set, its routine `func_01B318`, `LIST_1B73C` loaded, the words
    /// `WORD_AEBE`, `WORD_AEC6` and the horizontal scroll 0
    /// (`func_008664`), and the Z80's commands 1 and 0x7E.
    pub fn r_01b2c2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_animate(r);
        if !r.c {
            return;
        }
        r.c = false;
        if self.read8(ONCE_AECF) != 0 {
            self.z_end(r);
            return;
        }
        self.write8(ONCE_AECF, 1);
        self.z_next(r, R_01B318);
        self.write8(r.a6 + TASK_COUNT, 0);
        if self.z_load(r, LIST_1B73C) {
            return;
        }
        self.write16(WORD_AEBE, 0);
        self.write16(WORD_AEC6, 0);
        self.write16(H_VALUE, 0);
        self.z_scroll(r);
        r.d0 = low_byte(r.d0, SOUND_01);
        self.r_0034b6(r.d0);
        r.d0 = low_byte(r.d0, SOUND_7E);
        r.c = false;
        self.r_0034b6(r.d0);
    }

    /// `func_01B318`, a task's routine: `LIST_1B74E` loaded, the task
    /// ended (`func_01A8CA`) and a task `func_01B3A6` made (its
    /// `TASK_COUNT` and `TASK_STEPS` 0); once the VDP's line is 0xC0 or
    /// more, `func_0094B2`, `BYTE_AEC9` 0x30, `BYTE_AEC8` 0,
    /// `BYTE_AEC3` 0xA0, `WORD_AEBE` 0x100, the scroll at (0x100, 0).
    pub fn r_01b318(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.z_load(r, LIST_1B74E) {
            return;
        }
        self.z_end(r);
        self.z_sprite_task(r, 0x0A, 0x50, R_01B3A6);
        self.z_place(r, 0xA0, 0x46, 0x180, ANIM_DC2C0);
        self.write8(r.a6 + TASK_COUNT, 0);
        self.write8(r.a6 + TASK_STEPS, 0);
        self.z_wait_line(r);
        self.r_0094b2();
        self.write8(BYTE_AEC9, 0x30);
        self.write8(BYTE_AEC8, 0);
        self.write8(BYTE_AEC3, 0xA0);
        self.write16(WORD_AEBE, 0x100);
        self.write16(H_VALUE, 0x100);
        self.write16(V_VALUE, 0);
        self.z_scroll(r);
    }

    /// `func_01B3A4`: an `rts` (the end of `func_01B318`).
    pub fn r_01b3a4(&mut self) {}

    /// `func_01B3A6`, a task's routine: its animation; after 0x28
    /// calls its sprite record freed, its routine `func_01B3CC`.
    pub fn r_01b3a6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_animate(r);
        if !self.z_count(r, TASK_COUNT, 0x28) {
            return;
        }
        self.r_000890();
        self.z_next(r, R_01B3CC);
        self.write8(r.a6 + TASK_COUNT, 0);
        r.c = false;
    }

    /// `func_01B3CC`, a task's routine: after 0x1E calls the task ends,
    /// `LIST_1B756` is loaded, `func_0094B2`, and `func_01B6B6`.
    pub fn r_01b3cc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.z_count(r, TASK_COUNT, 0x1E) {
            return;
        }
        self.z_end_task(r);
        if self.z_load(r, LIST_1B756) {
            return;
        }
        self.r_0094b2();
        self.r_01b6b4();
        (r.d6, r.a6, r.x, r.c) = self.r_01b6b6(r.d4, r.d5, r.d6);
    }

    /// `func_01B3FE`, a task's routine: its animation only.
    pub fn r_01b3fe(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_animate(r);
    }

    /// `func_01B404`, a task's routine: `BYTE_AE70` and `BYTE_AEC3`
    /// counted up while `BYTE_AE70` is short of 0xEF; `BYTE_AEC9` and
    /// `BYTE_AE6A` counted down until `BYTE_AEC9` is 1; then
    /// `BYTE_AEC9` 0 and its routine `func_01B43C`.
    pub fn r_01b404(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(BYTE_AE70) != BYTE_AE70_TOP {
            let b = self.read8(BYTE_AEC3).wrapping_add(1);
            self.write8(BYTE_AEC3, b);
            let b = self.read8(BYTE_AE70).wrapping_add(1);
            self.write8(BYTE_AE70, b);
        }
        let count = self.read8(BYTE_AEC9);
        if count != 1 {
            self.write8(BYTE_AEC9, count.wrapping_sub(1));
            let (b, borrow) = self.read8(BYTE_AE6A).overflowing_sub(1);
            self.write8(BYTE_AE6A, b);
            r.x = borrow;
            r.c = borrow;
            return;
        }
        self.z_next(r, R_01B43C);
        self.write8(BYTE_AEC9, 0);
        r.c = false;
    }

    /// `func_01B43C`, a task's routine: the vertical scroll 0 and the
    /// horizontal by the whole screen (`func_00867E`), the task ended;
    /// a task `func_01B49A` and a sprite-task `func_01B4BA` made;
    /// `BYTE_AECA` 1.
    pub fn r_01b43c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write16(V_VALUE, 0);
        self.z_scroll(r);
        self.r_00867e();
        self.z_end_task(r);
        self.z_task(r, 0x3C, R_01B49A);
        self.z_sprite_task(r, 0x0A, 0x3C, R_01B4BA);
        self.z_place(r, 0x60, 0xE0, 0, ANIM_DC308);
        self.write8(BYTE_AECA, 1);
        r.c = false;
    }

    /// `func_01B49A`, a task's routine: the task ends, `LIST_1B77C` is
    /// loaded, `WORD_8064` 0x100 and `func_008664`.
    pub fn r_01b49a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_end_task(r);
        if self.z_load(r, LIST_1B77C) {
            return;
        }
        self.write16(WORD_8064, 0x100);
        self.z_scroll(r);
    }

    /// `func_01B4BA`, a task's routine: its animation, its sprite a
    /// pixel right and 4 up, until its y's low byte is 0x7C: then its
    /// routine `func_01B4E0`.
    pub fn r_01b4ba(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_animate(r);
        let x = self.read8(r.a5 + SPRITE_X_LOW).wrapping_add(1);
        self.write8(r.a5 + SPRITE_X_LOW, x);
        let (y, borrow) = self.read16(r.a5 + SPRITE_Y).overflowing_sub(4);
        self.write16(r.a5 + SPRITE_Y, y);
        let y = self.read8(r.a5 + SPRITE_Y_LOW);
        r.d0 = low_byte(r.d0, y);
        r.x = borrow;
        r.c = y < 0x7C;
        if y == 0x7C {
            self.z_next(r, R_01B4E0);
        }
    }

    /// `func_01B4E0`, a task's routine: its animation, its routine
    /// `func_01B3FE`; `WATCH_AECB` 0 and four sprite-tasks
    /// `func_01B5C2` (their `TASK_COUNT` their index, y from `ROW_Y`,
    /// animation from `ROW_ANIMS`), one `func_01B5AE`, and a task
    /// `func_01A68C`.  d1's low word is left the last index x 4.
    pub fn r_01b4e0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_animate(r);
        self.z_next(r, R_01B3FE);
        self.write8(WATCH_AECB, 0);
        self.write8(INDEX_AECC, 0);
        loop {
            self.z_sprite_task(r, 0x05, 0x64, R_01B5C2);
            let index = self.read8(INDEX_AECC);
            self.write8(r.a6 + TASK_COUNT, index);
            self.write16(PLACE_X, 0x80);
            self.write16(PLACE_FLAGS, 0x380);
            r.d1 = ext(r.d1, index);
            self.write8(PLACE_Y_HIGH, 0);
            let y = self.read8(ROW_Y.wrapping_add(index as i8 as u32));
            self.write8(PLACE_Y_LOW, y);
            let offset = (index as i8 as u16).wrapping_mul(4);
            r.d1 = low_word(r.d1, offset);
            let anim = self.read32(ROW_ANIMS.wrapping_add(offset as i16 as u32));
            self.write32(PLACE_ANIM, anim);
            (r.d0, r.x, r.c) = self.r_01b684(r.a5, r.a6);
            let index = self.read8(INDEX_AECC).wrapping_add(1);
            self.write8(INDEX_AECC, index);
            if index == ROW_SPRITES {
                break;
            }
        }
        self.z_sprite_task(r, 0x0A, 0x3C, R_01B5AE);
        self.z_place(r, 0x4A, 0xA8, 0x380, ANIM_DC326);
        self.z_task(r, 0x32, R_01A68C);
    }

    /// `func_01B5AE`, a task's routine: its animation, its sprite's y's
    /// low byte 0x98 + 0x10 x `CHOICE_AE89`.
    pub fn r_01b5ae(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_animate(r);
        let (y, carry) = (self.read8(CHOICE_AE89) << 4).overflowing_add(0x98);
        r.d0 = low_byte(r.d0, y);
        r.x = carry;
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        r.c = false;
    }

    /// `func_01B5C2`, a task's routine: its animation; while
    /// `WATCH_AECB` bit 0 is set, `INDEX_AECC` is `CHOICE_AE89` + 1, and
    /// the task whose `TASK_COUNT` is that takes the routine
    /// `func_01B5F8` (its count 0).
    pub fn r_01b5c2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.z_animate(r);
        if self.read8(WATCH_AECB) & 1 == 0 {
            return;
        }
        let (chosen, carry) = self.read8(CHOICE_AE89).overflowing_add(1);
        r.x = carry;
        self.write8(INDEX_AECC, chosen);
        let index = self.read8(r.a6 + TASK_COUNT);
        r.d0 = low_byte(r.d0, index);
        r.c = index < chosen;
        if index != chosen {
            return;
        }
        self.z_next(r, R_01B5F8);
        self.write8(r.a6 + TASK_COUNT, 0);
        r.c = false;
    }

    /// `func_01B5F8`, a task's routine: its count up, and its sprite's
    /// y's high byte 1 (off the screen) in the half of every 16 calls
    /// where bit 3 is set, else 0.
    pub fn r_01b5f8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (count, carry) = self.read8(r.a6 + TASK_COUNT).overflowing_add(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = carry;
        r.d0 = if count & 8 != 0 { 1 } else { low_byte(r.d0, 0) };
        self.write8(r.a5 + SPRITE_Y, r.d0 as u8);
        r.c = false;
    }

    /// `func_01B616`, a task's routine: the tasks and sprites cleared
    /// (`func_0003DA`, `func_000704`), `LIST_1B78A` loaded, the scroll
    /// at (0xFF00, 0) and the horizontal by the whole screen, a task
    /// `func_01B664`, a fade in (`func_00A506`, speed 1) and the Z80's
    /// command 1.
    pub fn r_01b616(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d0, r.d1, r.a0, r.x) = self.r_000704(r.d0);
        if self.z_load(r, LIST_1B78A) {
            return;
        }
        self.write16(H_VALUE, 0xFF00);
        self.write16(V_VALUE, 0);
        self.z_scroll(r);
        self.r_00867e();
        self.z_task(r, 0x32, R_01B664);
        r.d0 = 1;
        (r.d0, r.a1, r.x, r.c) = self.r_00a506(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        r.d0 = low_byte(r.d0, SOUND_01);
        r.c = false;
        self.r_0034b6(r.d0);
    }

    /// `func_01B664`, a task's routine: once the fade is over
    /// (`STOPPED` bit 0 clear) the task ends, `func_01B6B6`, and
    /// `BYTE_AEC8` 0.
    pub fn r_01b664(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(STOPPED) & 1 != 0 {
            return;
        }
        self.z_end_task(r);
        self.r_01b6b4();
        (r.d6, r.a6, r.x, r.c) = self.r_01b6b6(r.d4, r.d5, r.d6);
        self.write8(BYTE_AEC8, 0);
        r.c = false;
    }

    /// `func_01B684`: the task (a6) at `PLACE_X`, `PLACE_Y`, its
    /// sprite's (a5) flags `PLACE_FLAGS`' low byte, palette and frame
    /// 0, placed (`func_001B7E`), its animation `PLACE_ANIM` from the
    /// start.  Gives back d0, X and C (clear).
    pub fn r_01b684(&mut self, a5: u32, a6: u32) -> (u32, bool, bool) {
        let value = self.read16(PLACE_X);
        self.write16(a6 + TASK_X, value);
        let value = self.read16(PLACE_Y);
        self.write16(a6 + TASK_Y, value);
        let value = self.read8(PLACE_FLAGS_LOW);
        self.write8(a5 + SPRITE_FLAGS, value);
        self.write8(a5 + SPRITE_PALETTE, 0);
        self.write8(a5 + SPRITE_FRAME, 0);
        let (d0, x, _) = self.r_001b7e(a5, a6);
        let value = self.read32(PLACE_ANIM);
        self.write32(a6 + TASK_ANIM, value);
        self.write8(a6 + TASK_TIMER, 0);
        self.write8(a6 + TASK_STEP, 0);
        (d0, x, false)
    }

    /// `func_01B6B4`: an `rts`.
    pub fn r_01b6b4(&mut self) {}

    /// `func_01B6B6`: a task `func_01B404`, `BYTE_AECA` 2 and
    /// `BYTE_AEC9` 0x31.  Gives back d6, a6 (the task's), X and C
    /// (clear).
    pub fn r_01b6b6(&mut self, d4: u32, d5: u32, d6: u32) -> (u32, u32, bool, bool) {
        let (d6, a6, x, _) = self.r_0003e8(d4, d5, d6, 0x32, R_01B404);
        self.write8(BYTE_AECA, 2);
        self.write8(BYTE_AEC9, 0x31);
        (d6, a6, x, false)
    }
}
