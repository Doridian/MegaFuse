//! The task `func_01AD66` of the screen's sequence (`y_seq_b.rs`
//! makes it; 0x1AD66-0x1AEE4, in most scripts, which pass the title):
//! it counts 0x90 frames of its animation, then moves its sprite
//! diagonally (4 left, 4 down a frame) until it is at 0xD2 down; then,
//! unless the sequence was skipped, it frees the sprite, loads
//! `LIST_1B6F8` and makes two tasks `func_01AF30` and the task
//! `func_01AEF0` with a sprite of its own, and ends after two more
//! frames (`func_01AEE4`, `y_seq_b.rs`).

use crate::{Game, Regs};

/// Set once the pad skipped the sequence (`y_screen.rs`).
const SKIPPED: u32 = 0xFFFF_AECF;
/// The sequence's state (`y_screen.rs`), cleared here.
const SEQ_STATE: u32 = 0xFFFF_AECA;
/// A word (`BYTE_AEC8` and the byte after it, `y_seq_b.rs`) cleared.
const WORD_AEC8: u32 = 0xFFFF_AEC8;
/// A word of the horizontal scroll table (`y_screen.rs`).
const SCROLL_D402: u32 = 0xFFFF_D402;

/// A task's fields: its sprite record's index, its frames (a word;
/// for the tasks `func_01AF30` two bytes it starts from).
const TASK_SLOT: u32 = 0x03;
const TASK_FRAMES: u32 = 0x18;
const TASK_18: u32 = 0x18;
const TASK_19: u32 = 0x19;
/// A sprite record's fields (`sprite.rs`).
const SPRITE_03: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;

/// The frames before the sprite moves, and where it stops.
const FRAMES_BEFORE: u16 = 0x90;
const STOP_Y: u8 = 0xD2;

/// In the ROM: the list loaded, an animation.
const LIST_1B6F8: u32 = 0x01B6F8;
const ANIM_DC2C4: u32 = 0x0DC2C4;

/// Routines: the tasks made, and where this one goes on.
const COUNTING: u32 = 0x01AD76;
const MOVING: u32 = 0x01AD90;
const TASK_1AF30: u32 = 0x01AF30;
const TASK_1AEF0: u32 = 0x01AEF0;
const LAST_FRAME: u32 = 0x01AEE0;
const ENDING: u32 = 0x01AEE4;

impl Game {
    /// `func_01AD66`, a task: `TASK_FRAMES` 0, its routine
    /// `func_01AD76` from now on, and on into it.
    pub fn r_01ad66(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write16(r.a6 + TASK_FRAMES, 0);
        self.y_next(r, COUNTING);
        self.r_01ad76(r);
    }

    /// `func_01AD76`, each frame: its animation on (`func_0013D6`),
    /// `TASK_FRAMES` up; at `FRAMES_BEFORE` a yield to `func_01AD90`.
    /// Gives back X (the count's carry) and C (below
    /// `FRAMES_BEFORE`), or the yield's.
    pub fn r_01ad76(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_animate(r);
        let (frames, carry) = self.read16(r.a6 + TASK_FRAMES).overflowing_add(1);
        r.d0 = (r.d0 & 0xFFFF_0000) | u32::from(frames);
        r.x = carry;
        self.write16(r.a6 + TASK_FRAMES, frames);
        if frames != FRAMES_BEFORE {
            r.c = frames < FRAMES_BEFORE;
            return;
        }
        self.yb_yield(r, MOVING);
    }

    /// `func_01AD90`, each frame: its animation on (`func_0013D6`), its
    /// sprite 4 left and 4 down, until it is at `STOP_Y` down.  Then the
    /// sprite freed (`func_000890`); unless `SKIPPED`: `func_00855A`,
    /// `LIST_1B6F8` loaded (`func_0094D8`), `SEQ_STATE` cleared, two
    /// tasks `func_01AF30` (depth 0xA, priority 0x3C; bytes 0xE6/0xC0
    /// and 0xF8, placed at (0xFF, 0) with flags 0x580 and 0x680,
    /// `ANIM_DC2C4`), the task `func_01AEF0` (priority 0xB) with a
    /// sprite (depth 0x1E: frame 0x31 at (0x80, 0x78), flags 0x80),
    /// `WORD_AEC8` and the scroll word cleared (`func_008664`),
    /// `func_0094B2`, and a yield to 0x1AEE0.
    pub fn r_01ad90(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.y_animate(r);
        let x = self.read8(r.a5 + SPRITE_X_LOW);
        self.write8(r.a5 + SPRITE_X_LOW, x.wrapping_sub(4));
        let (y, carry) = self.read8(r.a5 + SPRITE_Y_LOW).overflowing_add(4);
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        r.x = carry;
        if y < STOP_Y {
            r.c = true;
            return;
        }
        self.r_000890();
        r.c = false;
        if self.read8(SKIPPED) != 0 {
            return;
        }
        self.r_00855a();
        if self.y_load(r, LIST_1B6F8) {
            return;
        }
        self.write8(SEQ_STATE, 0);
        for (start, flags) in [(0xE6, 0x580), (0xC0, 0x680)] {
            self.y_make_task(r, 0xA, 0x3C, TASK_1AF30);
            self.write8(r.a6 + TASK_18, start);
            self.write8(r.a6 + TASK_19, 0xF8);
            self.y_place(r, 0xFF, 0, flags, ANIM_DC2C4);
        }
        self.y_start_task(r, 0xB, TASK_1AEF0);
        self.yb_new_sprite(r, 0x1E);
        self.write8(r.a6 + TASK_SLOT, r.d5 as u8);
        self.write8(r.a5 + SPRITE_FRAME, 0x31);
        self.write8(r.a5 + SPRITE_X_LOW, 0x80);
        self.write8(r.a5 + SPRITE_X, 0);
        self.write8(r.a5 + SPRITE_Y_LOW, 0x78);
        self.write8(r.a5 + SPRITE_Y, 0);
        self.write8(r.a5 + SPRITE_03, 0);
        self.write8(r.a5 + SPRITE_FLAGS, 0x80);
        self.write16(WORD_AEC8, 0);
        self.write16(SCROLL_D402, 0);
        self.r_008664(r);
        r.c = self.r_0094b2();
        self.yb_yield(r, LAST_FRAME);
    }

    /// `func_01AEE0` (the frame after): a yield to `func_01AEE4`.
    pub fn r_01aee0(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.yb_yield(regs, ENDING);
    }
}
