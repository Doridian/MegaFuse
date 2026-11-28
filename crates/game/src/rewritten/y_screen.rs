//! A screen of three choices (0x1A598-0x1A7EC): `func_01A598` sets it
//! up (the tasks `func_01A7EC` and `func_01A772`, `y_seq_a.rs`); its
//! choice routine `func_01A68C` moves `CHOICE` with bits 0 and 1 of the
//! pad's byte and takes it with bit 7 (`func_01A712` then goes on by
//! it: `func_010450`, `func_0179B0` or `loc_01BBE0`), or gives up after
//! `IDLE_LIMIT` frames (`func_01A768`: `func_007AAE`).  `func_01A772`
//! skips the screen's sequence when any of the pad's bits 4-7 is set.

use crate::{Game, Regs};

/// Cleared by the set-up (`k_vdp.rs`, `i_stage.rs`).
const COPY_ON: u32 = 0xFFFF_A945;
const BYTE_80DF: u32 = 0xFFFF_80DF;
/// A long the set-up sets to 0xFE00.
const LONG_8A5E: u32 = 0xFFFF_8A5E;
/// The view (a long and a word, `a_motion.rs`), cleared.
const VIEW: u32 = 0xFFFF_80E6;
const VIEW_Y_MORE: u32 = 0xFFFF_80EA;
/// The screen's bytes, cleared by the set-up (`STATE_SIZE` of them).
const STATE: u32 = 0xFFFF_AE6A;
const STATE_SIZE: u32 = 0x68;
/// Where the set-up copies `COPIED_SIZE` bytes of `COPIED` to.
const COPY: u32 = 0xFFFF_AE8A;
const COPIED_SIZE: u32 = 0x15;
/// The choice (0-2).
pub(crate) const CHOICE: u32 = 0xFFFF_AE89;
/// Set to 0x30 and 1 by the set-up (0x30 and 1 again by `func_01B122`).
const BYTE_AEC9: u32 = 0xFFFF_AEC9;
const BYTE_AEC6: u32 = 0xFFFF_AEC6;
/// Set to 1 when the choice is taken.
pub(crate) const BYTE_AECB: u32 = 0xFFFF_AECB;
/// Set to 1 by `func_01A712`.
pub(crate) const BYTE_AECD: u32 = 0xFFFF_AECD;
/// The frames the choice routine has waited (a word), and the most.
const IDLE_COUNT: u32 = 0xFFFF_AED0;
const IDLE_LIMIT: u16 = 0x4B0;
/// Set once the pad skipped the sequence (`func_01A772`).
pub(crate) const SKIPPED: u32 = 0xFFFF_AECF;
/// The sequence's state: bit 1 makes `func_01A772` end, any bit
/// `func_01A7EC` (`y_seq_a.rs`).
pub(crate) const SEQ_STATE: u32 = 0xFFFF_AECA;
/// The pad's byte (`i_main.rs`): bit 0 and bit 1 move the choice,
/// bit 7 takes it, bits 4-7 skip the sequence.
pub(crate) const PAD: u32 = 0xFFFF_AD02;
/// Bit 0 set: the objects are stopped (`j_colours.rs`); the screen's
/// tasks then do nothing.
pub(crate) const OBJECTS_STOPPED: u32 = 0xFFFF_A3FC;
/// Bytes the set-up sets (0xFF, 0, 0, and `OBJECTS_STOPPED` 0).
const LINES_DEFAULT: u32 = 0xFFFF_A3F9;
const BYTE_A3FA: u32 = 0xFFFF_A3FA;
const BYTE_A3FB: u32 = 0xFFFF_A3FB;
/// Words of the horizontal scroll table (`h_scroll.rs`) the set-up
/// sets.
pub(crate) const SCROLL_D002: u32 = 0xFFFF_D002;
pub(crate) const SCROLL_D400: u32 = 0xFFFF_D400;
pub(crate) const SCROLL_D402: u32 = 0xFFFF_D402;
/// The lists `func_0094D8` (`LOAD_LIST`) and the H-interrupt
/// (`LINES_LIST`, `j_lines.rs`) read.
pub(crate) const LOAD_LIST: u32 = 0xFFFF_92FA;
const LINES_LIST: u32 = 0xFFFF_A40A;
/// Bytes `func_01A712` clears before `func_0179B0`.
const UPTO: u32 = 0xFFFF_953B;
const BYTE_953D: u32 = 0xFFFF_953D;
/// What `func_01B684` places a task with: x, y (words), the sprite's
/// flags (the low byte of a word) and the animation (a long).
pub(crate) const PLACE_X: u32 = 0xFFFF_80C4;
pub(crate) const PLACE_Y: u32 = 0xFFFF_80C8;
pub(crate) const PLACE_FLAGS: u32 = 0xFFFF_80CC;
pub(crate) const PLACE_ANIM: u32 = 0xFFFF_80D0;

/// In the ROM: the tiles' set (`func_000776`), the bytes copied to
/// `COPY`, the lists.
const TILE_SET: u32 = 0x0DA7CC;
const COPIED: u32 = 0x01B7AE;
const LINES_1B7E8: u32 = 0x01B7E8;
const LOAD_1A670: u32 = 0x01A670;
/// By choice: the next one up and down.
const CHOICE_UP: u32 = 0x01A6E8;
const CHOICE_DOWN: u32 = 0x01A6EB;
/// `loc_01BBE0`'s lists and its tasks' routines and animation.
const LINES_ABA4: u32 = 0x00ABA4;
const LOAD_1BD12: u32 = 0x01BD12;
const ROUTINE_1BD36: u32 = 0x01BD36;
const ROUTINE_1C06A: u32 = 0x01C06A;
const ROUTINE_1C106: u32 = 0x01C106;
const ROUTINE_1C1C0: u32 = 0x01C1C0;
const ANIM_DC34A: u32 = 0x0DC34A;

/// The tasks' routines.
const SCROLLING: u32 = 0x01A7EC;
const WATCHING: u32 = 0x01A772;
const WATCHING_2: u32 = 0x01A7BC;
const AFTER_SKIP: u32 = 0x01B616;
const TAKEN: u32 = 0x01A712;
const GAVE_UP: u32 = 0x01A768;

/// A sprite record's fields (`sprite.rs`): its palette, frame, flags,
/// x and y (words).
const SPRITE_PALETTE: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_Y: u32 = 0x0A;
/// A task's sprite record's index, position (words) and a byte.
pub(crate) const TASK_SPRITE: u32 = 0x03;
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_18: u32 = 0x18;

pub(crate) fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

pub(crate) fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_001222` with the registers: a task with a sprite record.
    pub(crate) fn y_make_task(&mut self, r: &mut Regs, depth: u8, priority: u8, routine: u32) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, priority, routine);
    }

    /// `func_0003E8` with the registers: a task.
    pub(crate) fn y_start_task(&mut self, r: &mut Regs, priority: u8, routine: u32) {
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, priority, routine);
    }

    /// `func_01B684` with `PLACE_*` set: the task placed.
    pub(crate) fn y_place(&mut self, r: &mut Regs, x: u16, y: u16, flags: u16, anim: u32) {
        self.write16(PLACE_X, x);
        self.write16(PLACE_Y, y);
        self.write16(PLACE_FLAGS, flags);
        self.write32(PLACE_ANIM, anim);
        (r.d0, r.x, r.c) = self.r_01b684(r.a5, r.a6);
    }

    /// `func_000560` with the registers: the running task ends.
    pub(crate) fn y_end(&mut self, r: &mut Regs) {
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }

    /// `func_00053E` with the registers.
    pub(crate) fn y_next(&mut self, r: &mut Regs, routine: u32) {
        (r.x, r.c) = self.r_00053e(r.d6, routine);
    }

    /// `func_0013D6` with the registers: the task's animation a frame
    /// on (C: it ended).
    pub(crate) fn y_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `func_0094D8` with the registers; true when the game restarted.
    pub(crate) fn y_load(&mut self, r: &mut Regs, list: u32) -> bool {
        self.write32(LOAD_LIST, list);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        self.restarted.is_some()
    }

    /// `func_00A51A` (a fade, `j_colours.rs`) with `d0` (all of it).
    pub(crate) fn y_fade(&mut self, r: &mut Regs, d0: u32) {
        r.d0 = d0;
        (r.d0, r.a1, r.x, r.c) = self.r_00a51a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00A506` with `d0` (all of it).
    pub(crate) fn y_fade_in(&mut self, r: &mut Regs, d0: u32) {
        r.d0 = d0;
        (r.d0, r.a1, r.x, r.c) = self.r_00a506(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// The Z80's command `command` (`func_0034B6`, d0's low byte).
    pub(crate) fn y_sound(&mut self, r: &mut Regs, command: u8) {
        r.d0 = low_byte(r.d0, command);
        r.c = false;
        self.r_0034b6(r.d0);
    }

    /// `func_01A598`: the screen set up.  The tasks and sprites
    /// cleared (`func_0003DA`, `func_000704`), `TILE_SET` loaded
    /// (`func_000776`, `func_000752`), `STATE` cleared, the scroll
    /// words, the tasks `func_01A7EC` (priority 0x96) and `func_01A772`
    /// (0x14), the view cleared, `COPIED` copied to `COPY`, the lists
    /// loaded (`func_0094D8`), `func_00A63A` with 5 and the Z80's
    /// command 0.  Gives back the registers as the original leaves
    /// them.
    pub fn r_01a598(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(COPY_ON, 0);
        self.write8(BYTE_80DF, 0);
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.write32(LONG_8A5E, 0xFE00);
        r.a0 = TILE_SET;
        r.d0 = low_word(r.d0, 0);
        r.d0 = self.r_000776(r.d0, r.a0, r.a1);
        r.d0 = low_word(r.d0, 0);
        r.d1 = low_word(r.d1, 0);
        r.d2 = low_byte(r.d2, 7);
        self.r_000752(r.d0, r.d1, r.d2);
        (r.d0, r.d1, r.a0, r.x) = self.r_000704(r.d0);
        for i in 0..STATE_SIZE {
            self.write8(STATE + i, 0);
        }
        r.a0 = STATE + STATE_SIZE;
        r.d0 = low_word(r.d0, 0xFFFF);
        self.write8(BYTE_AEC9, 0x30);
        self.write8(BYTE_AEC6, 1);
        self.r_0002da();
        self.write16(SCROLL_D002, 0xFF00);
        self.write16(SCROLL_D402, 0);
        self.write16(SCROLL_D400, 3);
        self.y_start_task(r, 0x96, SCROLLING);
        self.y_start_task(r, 0x14, WATCHING);
        self.write32(VIEW, 0);
        self.write16(VIEW_Y_MORE, 0);
        for i in 0..COPIED_SIZE {
            let b = self.read8(COPIED + i);
            self.write8(COPY + i, b);
        }
        r.a1 = COPY + COPIED_SIZE;
        r.d0 = low_word(r.d0, 0xFFFF);
        r.a0 = COPY;
        self.write32(LINES_LIST, LINES_1B7E8);
        if self.y_load(r, LOAD_1A670) {
            return;
        }
        self.write8(LINES_DEFAULT, 0xFF);
        self.write8(BYTE_A3FA, 0);
        self.write8(BYTE_A3FB, 0);
        self.write8(OBJECTS_STOPPED, 0);
        r.d0 = 5;
        (r.d0, r.a1, r.x, r.c) = self.r_00a63a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        self.y_sound(r, 0);
    }

    /// `func_01A68C`, the choice's routine: `IDLE_COUNT` up; at
    /// `IDLE_LIMIT` a fade (`func_00A51A` with 4) and `func_01A768`
    /// next.  Otherwise the pad's bit 0 or 1 moves `CHOICE` up or down
    /// (`CHOICE_UP`, `CHOICE_DOWN`; sound 0x20, the count back to 0),
    /// bit 7 takes it: sound 0x21, `BYTE_AECB` 1, the fade and
    /// `func_01A712` next.  Gives back the registers as the original
    /// leaves them.
    pub fn r_01a68c(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let count = self.read16(IDLE_COUNT);
        let count = count.wrapping_add(1);
        self.write16(IDLE_COUNT, count);
        r.c = count == 0;
        r.x = r.c;
        if count == IDLE_LIMIT {
            self.y_fade(r, 4);
            self.y_next(r, GAVE_UP);
            return;
        }
        let choice = self.read8(CHOICE) as i8;
        r.d1 = low_word(r.d1, choice as i16 as u16);
        r.c = false;
        let pad = self.read8(PAD);
        let index = choice as i32 as u32;
        if pad & 0x01 != 0 {
            self.y_sound(r, 0x20);
            let next = self.read8(CHOICE_UP.wrapping_add(index));
            self.write8(CHOICE, next);
            self.write16(IDLE_COUNT, 0);
        } else if pad & 0x02 != 0 {
            self.y_sound(r, 0x20);
            let next = self.read8(CHOICE_DOWN.wrapping_add(index));
            self.write8(CHOICE, next);
            self.write16(IDLE_COUNT, 0);
        } else if pad & 0x80 != 0 {
            self.y_sound(r, 0x21);
            self.write8(BYTE_AECB, 1);
            self.y_fade(r, 4);
            self.y_next(r, TAKEN);
        }
    }

    /// `func_01A712`: unless the objects are stopped (`func_001BB4`:
    /// then it returns), `func_008624`, `BYTE_AECD` 1, and by `CHOICE`:
    /// 0 `func_01C258`, `UPTO` and `BYTE_953D` cleared and
    /// `func_0179B0`; 1 `func_010450`; 2 `loc_01BBE0`.  Gives back the
    /// registers as the original leaves them.
    pub fn r_01a712(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        self.r_008624();
        self.write8(BYTE_AECD, 1);
        let choice = self.read8(CHOICE);
        if choice == 1 {
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
                self.r_010450(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5, r.a6);
        } else if choice == 0 {
            self.r_01c258();
            self.write8(UPTO, 0);
            self.write8(BYTE_953D, 0);
            (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) =
                self.r_0179b0(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        } else {
            self.y_bbe0(r);
        }
    }

    /// `loc_01BBE0` (reached only from `func_01A712`): the tasks and
    /// sprites cleared (`func_0003DA`, `func_000704`), `func_01C082`,
    /// `func_00B4C6`, the lists loaded; a task `func_01BD36` (priority
    /// 0x1E) with a sprite (depth 0xA, frame 0x7E at (0x44, 0x5C)),
    /// four `func_01C06A` (0x28; `TASK_18` 0-3, frame 0x74 at
    /// (0x44 + 0x28 * i, 0x5C)), the tasks with sprites `func_01C106`
    /// (depth 0x1E, priority 0x64, placed at (0x50, 0x90)) and
    /// `func_01C1C0` (0x28, 0x82; at (0xB8, 0xA0), frame 0x81), a fade
    /// in (`func_00A506` with 4) and sound 0x15.
    fn y_bbe0(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d0, r.d1, r.a0, r.x) = self.r_000704(r.d0);
        (r.d0, r.d6, r.a0, r.a1, r.a6, r.x) =
            self.r_01c082(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        self.r_00b4c6(r);
        self.write32(LINES_LIST, LINES_ABA4);
        if self.y_load(r, LOAD_1BD12) {
            return;
        }
        r.a0 = ROUTINE_1BD36;
        r.d0 = 0x1E;
        (r.d6, r.a6, r.x, r.c) = self.r_000404(r.d0, r.d4, r.d5, r.d6, r.a0);
        r.d0 = 0xA;
        (r.d5, r.a5, r.x, r.c) = self.r_000796(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
        self.write8(r.a6 + TASK_SPRITE, r.d5 as u8);
        self.write8(r.a5 + SPRITE_FRAME, 0x7E);
        self.write16(r.a5 + SPRITE_X, 0x44);
        self.write16(r.a5 + SPRITE_Y, 0x5C);
        self.write8(r.a5 + SPRITE_PALETTE, 0);
        self.write8(r.a5 + SPRITE_FLAGS, 0x80);
        r.d3 = low_word(r.d3, 0);
        for i in 0..4u8 {
            r.a0 = ROUTINE_1C06A;
            r.d0 = 0x28;
            (r.d6, r.a6, r.x, r.c) = self.r_000404(r.d0, r.d4, r.d5, r.d6, r.a0);
            self.write8(r.a6 + TASK_18, i);
            r.d0 = 0x14;
            (r.d5, r.a5, r.x, r.c) = self.r_000796(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
            self.write8(r.a6 + TASK_SPRITE, r.d5 as u8);
            self.write8(r.a5 + SPRITE_FRAME, 0x74);
            r.d1 = low_byte(r.d1, i << 3);
            r.d0 = low_byte(r.d0, (i << 5).wrapping_add(i << 3).wrapping_add(0x44));
            self.write16(r.a5 + SPRITE_X, u16::from(r.d0 as u8));
            self.write16(r.a5 + SPRITE_Y, 0x5C);
            self.write8(r.a5 + SPRITE_FLAGS, 0x80);
            self.write8(r.a5 + SPRITE_PALETTE, 0);
            r.d3 = low_byte(r.d3, i + 1);
        }
        self.y_make_task(r, 0x1E, 0x64, ROUTINE_1C106);
        self.y_place(r, 0x50, 0x90, 0x3280, ANIM_DC34A);
        self.y_make_task(r, 0x28, 0x82, ROUTINE_1C1C0);
        self.write16(r.a6 + TASK_X, 0xB8);
        self.write16(r.a6 + TASK_Y, 0xA0);
        self.write8(r.a5 + SPRITE_FLAGS, 0x80);
        self.write8(r.a5 + SPRITE_PALETTE, 0);
        self.write8(r.a5 + SPRITE_FRAME, 0x81);
        self.y_fade_in(r, 4);
        self.y_sound(r, 0x15);
    }

    /// `func_01A768`: unless the objects are stopped (`func_001BB4`),
    /// the game starts over (`func_007AAE`).
    pub fn r_01a768(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        let mut next = *r;
        self.r_007aae(&mut next);
        self.restarted = Some(next);
    }

    /// `func_01A772`, a task: ends (`func_01A85E`) once `SKIPPED` or
    /// bit 1 of `SEQ_STATE` is set; nothing while the objects are
    /// stopped; when the pad's bits 4-7 (`func_01A7D4`) skip the
    /// sequence: `func_01A7BC` next, a fade (`func_00A51A` with 5),
    /// `SKIPPED` 1, the Z80's command 0xFE with d1 0x40.  Gives back
    /// the registers as the original leaves them.
    pub fn r_01a772(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        r.c = false;
        if self.read8(SKIPPED) != 0 {
            self.y_end(r);
            return;
        }
        if self.read8(OBJECTS_STOPPED) & 0x01 != 0 {
            return;
        }
        if self.read8(SEQ_STATE) & 0x02 != 0 {
            self.y_end(r);
            return;
        }
        (r.d0, r.c) = self.r_01a7d4(r.d0);
        if !r.c {
            return;
        }
        self.y_next(r, WATCHING_2);
        self.y_fade(r, 5);
        self.write8(SKIPPED, 1);
        r.d1 = 0x40;
        r.d0 = low_byte(r.d0, 0xFE);
        r.c = false;
        self.r_00349e(r.d0);
    }

    /// `func_01A7BC`, a task: unless the objects are stopped,
    /// `func_01B616` next.  Gives back X and C (as they came while
    /// stopped).
    pub fn r_01a7bc(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        if self.read8(OBJECTS_STOPPED) & 0x01 == 0 {
            self.y_next(regs, AFTER_SKIP);
        }
    }

    /// `loc_01A7D2`: nothing.
    pub fn r_01a7d2(&mut self) {}

    /// `func_01A7D4`: d0's low byte the pad's bits 4-7; C if any is
    /// set.
    pub fn r_01a7d4(&mut self, d0: u32) -> (u32, bool) {
        let bits = self.read8(PAD) & 0xF0;
        (low_byte(d0, bits), bits != 0)
    }
}
