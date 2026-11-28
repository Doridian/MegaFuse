//! The screen of four digits (`loc_01BBE0`, whose task `func_01BD36`
//! moves the chosen digit with the pad): the digits' sum as a number,
//! a digit's sprite, the screen's set-up, and the two tasks with
//! sprites (`func_01C106`, `func_01C1C0`) that go from animation to
//! animation by the bytes `STATE_6B`-`STATE_6D` the pad's task sets.

use crate::{Game, Regs};

/// The four digits (bytes 0-9; the first counts ones, the last
/// thousands in `func_01C024`).
const DIGITS: u32 = 0xFF9564;
const DIGIT_TENS: u32 = 0xFF9565;
const DIGIT_HUNDREDS: u32 = 0xFF9566;
const DIGIT_THOUSANDS: u32 = 0xFF9567;
/// The work word `func_01C024` leaves the number in.
const NUMBER: u32 = 0xFF80C4;
/// Bytes the screen's tasks go by: set to 1 by `func_01C1A6` and when
/// `func_01C212` is done (`func_01C0D8` waits for it); set by the pad's
/// task (`func_01BD36`) and cleared by `func_01C212`/`func_01C23C`.
const STATE_6B: u32 = 0xFF956B;
const STATE_6C: u32 = 0xFF956C;
const STATE_6D: u32 = 0xFF956D;
/// What `func_01C082` clears: `CLEARED_LEN` bytes from 0xFF955C, a
/// byte, two words, a long and a word.
const CLEARED: u32 = 0xFFFF955C;
const CLEARED_LEN: u32 = 0x1E;
const BYTE_AE89: u32 = 0xFFAE89;
const WORD_D002: u32 = 0xFFD002;
const WORD_D402: u32 = 0xFFD402;
const LONG_80E6: u32 = 0xFF80E6;
const WORD_80EA: u32 = 0xFF80EA;
/// What `func_01C258` sets (bytes, but the last two words); bit 0 of
/// `SPARED` cleared.
const BYTE_A937: u32 = 0xFFA937;
const BYTE_A938: u32 = 0xFFA938;
const BYTE_A939: u32 = 0xFFA939;
const BYTE_A93A: u32 = 0xFFA93A;
const SPARED: u32 = 0xFF9548;
const BYTE_9543: u32 = 0xFF9543;
const BYTE_954E: u32 = 0xFF954E;
const WORD_9544: u32 = 0xFF9544;
const WORD_9546: u32 = 0xFF9546;

/// A task's fields: its animation's counters (`anim.rs`), the digit it
/// shows (`func_01C06A`), a byte `func_01C082` sets, its animation.
const TASK_C: u32 = 0x0C;
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_DIGIT: u32 = 0x18;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's frame.
const SPRITE_FRAME: u32 = 0x04;
/// A digit's frame: the digit plus this.
const DIGIT_FRAME: u8 = 0x74;
/// The frame `func_01C212` leaves its sprite with.
const DONE_FRAME: u8 = 0x81;

/// `func_01C082`'s sprites' frames (`func_000776`, set 1), its task (`SCREEN_TASK`,
/// priority `SCREEN_PRIORITY`) and the arguments of `func_00A506`.
const FRAMES: u32 = 0x0DC3D8;
const SCREEN_TASK: u32 = 0x0175A4;
const SCREEN_PRIORITY: u8 = 0x1E;

/// The tasks' routines and animations: the first (`func_01C106`) goes to
/// `FIRST_B` (`ANIM_FIRST_B`) or to `FIRST_C` (`ANIM_FIRST_C`), each
/// back to `FIRST_A` (`ANIM_FIRST_A`) or on to `FIRST_D` (`ANIM_FIRST_D`)
/// then `SECOND_IDLE`; the second (`func_01C1C0`) to `SECOND_B`
/// (`ANIM_SECOND_B`) or `SECOND_C` (`ANIM_SECOND_C`) and back.
const FIRST_A: u32 = 0x01C106;
const FIRST_B: u32 = 0x01C15C;
const FIRST_C: u32 = 0x01C182;
const FIRST_D: u32 = 0x01C1A6;
const SECOND_IDLE: u32 = 0x01C1C0;
const SECOND_B: u32 = 0x01C212;
const SECOND_C: u32 = 0x01C23C;
const ANIM_FIRST_A: u32 = 0x0DC34A;
const ANIM_FIRST_B: u32 = 0x0DC350;
const ANIM_FIRST_C: u32 = 0x0DC36C;
const ANIM_FIRST_D: u32 = 0x0DC3A4;
const ANIM_SECOND_B: u32 = 0x0DC35A;
const ANIM_SECOND_C: u32 = 0x0DC3C4;
/// `func_01C0D8`'s next routine and `func_01C0F6`'s.
const WAIT_DONE: u32 = 0x01C0F6;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_01C014`: a bare `rts` (`func_01BD36`'s way out).
    pub fn r_01c014(&mut self) {}

    /// `func_01C016`, a task's routine (`func_01BD36`'s on bit 4 of the
    /// pad): unless the objects are stopped (`func_001BB4`), its sprite
    /// freed (`func_000890`) and on to `loc_01BA12`.
    pub fn r_01c016(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        self.r_000890();
        (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) = self.r_01ba12(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
    }

    /// `func_01C024`: the digits as a number into `NUMBER` (a word:
    /// thousands, hundreds, tens and ones, the first three counted up
    /// one at a time).  Gives back d0 (the ones, its low word) and d3
    /// (its low byte 0).
    pub fn r_01c024(&mut self, d0: u32, d3: u32) -> (u32, u32) {
        let mut number: u16 = 0;
        for (addr, unit) in [(DIGIT_THOUSANDS, 1000u16), (DIGIT_HUNDREDS, 100), (DIGIT_TENS, 10)] {
            let n = self.read8(addr);
            number = number.wrapping_add(unit.wrapping_mul(u16::from(n)));
        }
        let ones = self.read8(DIGITS);
        self.write16(NUMBER, number.wrapping_add(u16::from(ones)));
        (low_word(d0, u16::from(ones)), low_byte(d3, 0))
    }

    /// `func_01C06A`, a digit's task: its sprite shows the digit
    /// `TASK_DIGIT` picks (`DIGIT_FRAME` on).  Gives back d0 (the
    /// index's sign above the frame), a0 (`DIGITS`), X (the frame's
    /// carry) and C (clear).
    pub fn r_01c06a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let index = self.read8(r.a6 + TASK_DIGIT) as i8 as i16;
        r.a0 = 0xFFFF0000 | DIGITS;
        let digit = self.read8(r.a0.wrapping_add(index as i32 as u32));
        let (frame, carry) = digit.overflowing_add(DIGIT_FRAME);
        r.d0 = low_byte(low_word(r.d0, index as u16), frame);
        self.write8(r.a5 + SPRITE_FRAME, frame);
        r.x = carry;
        r.c = false;
    }

    /// `func_01C082`: the screen's set-up: `func_0002DA`, the frames set 1
    /// from `FRAMES`, the bytes from `CLEARED` and the other words cleared,
    /// `func_008664`, `func_00A506` (4), and a task `SCREEN_TASK` whose
    /// `TASK_C` is 0xA.  Gives back d0, d6, a0, a1, a6 and X.
    pub fn r_01c082(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, bool) {
        self.r_0002da();
        self.r_000776(1, FRAMES, a1);
        for i in 0..CLEARED_LEN {
            self.write8(CLEARED + i, 0);
        }
        self.write8(BYTE_AE89, 0);
        self.write16(WORD_D002, 0);
        self.write16(WORD_D402, 0);
        // dbra leaves d0's low word 0xFFFF, a0 past the bytes.
        let mut r = Regs { d0: 0xFFFF, d1, d2, d3, d4, d5, d6, d7, a0: CLEARED + CLEARED_LEN, a1, a2, a3, a4, a5, a6, x, n: false, z: false, v: false, c: false, unwound: false };
        self.r_008664(&mut r);
        self.write32(LONG_80E6, 0);
        self.write16(WORD_80EA, 0);
        let (d0, a1, _, _) = self.r_00a506(4, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        let (d6, a6, x, _) = self.r_0003e8(r.d4, r.d5, r.d6, SCREEN_PRIORITY, SCREEN_TASK);
        self.write8(a6 + TASK_C, 0xA);
        (d0, d6, r.a0, a1, a6, x)
    }

    /// `func_01C0D8`, a task's routine: once `STATE_6B` is set, on as
    /// `WAIT_DONE` and `func_00A51A` (3).  C clear when it waits.
    pub fn r_01c0d8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(STATE_6B) == 0 {
            r.c = false;
            return;
        }
        self.r_00053e(r.d6, WAIT_DONE);
        (r.d0, r.a1, r.x, r.c) = self.r_00a51a(3, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_01C0F6`, `WAIT_DONE`: unless the objects are stopped
    /// (`func_001BB4`), the sprite freed, the task ended
    /// (`func_000560`) and on to `loc_01797A`.
    pub fn r_01c0f6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
        (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) = self.r_01797a(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// The task's animation a frame on (`func_0013D6`); true (and C) at
    /// its end.
    fn zc_task_animate(&mut self, r: &mut Regs) -> bool {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        r.c
    }

    /// The task on as `routine` (`func_00053E`) with `anim` from its
    /// start.  Leaves C clear.
    fn zc_task_next(&mut self, r: &mut Regs, routine: u32, anim: u32) {
        (r.x, _) = self.r_00053e(r.d6, routine);
        self.write32(r.a6 + TASK_ANIM, anim);
        self.write8(r.a6 + TASK_TIMER, 0);
        self.write8(r.a6 + TASK_STEP, 0);
        r.c = false;
    }

    /// The task's sprite placed (`func_001B7E`), then: `STATE_6D` set,
    /// on as `b` with `anim_b`; else `STATE_6C` set, on as `c` with
    /// `anim_c`.  Leaves C clear.
    fn zc_task_choose(&mut self, r: &mut Regs, b: (u32, u32), c: (u32, u32)) {
        (r.d0, r.x, _) = self.r_001b7e(r.a5, r.a6);
        if self.read8(STATE_6D) != 0 {
            self.zc_task_next(r, b.0, b.1);
        } else if self.read8(STATE_6C) != 0 {
            self.zc_task_next(r, c.0, c.1);
        } else {
            r.c = false;
        }
    }

    /// `func_01C106`, `FIRST_A`: animated, placed, and on by
    /// `STATE_6D`/`STATE_6C` (`zc_task_choose`).
    pub fn r_01c106(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_task_animate(r);
        self.zc_task_choose(r, (FIRST_B, ANIM_FIRST_B), (FIRST_C, ANIM_FIRST_C));
    }

    /// `func_01C15C`, `FIRST_B`: at its animation's end back to
    /// `FIRST_A`.  C clear.
    pub fn r_01c15c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.zc_task_animate(r) {
            self.zc_task_next(r, FIRST_A, ANIM_FIRST_A);
        }
    }

    /// `func_01C180`: a bare `rts` (`func_01C15C`'s way out).
    pub fn r_01c180(&mut self) {}

    /// `func_01C182`, `FIRST_C`: at its animation's end on to `FIRST_D`.
    /// C clear.
    pub fn r_01c182(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.zc_task_animate(r) {
            self.zc_task_next(r, FIRST_D, ANIM_FIRST_D);
        }
    }

    /// `func_01C1A6`, `FIRST_D`: at its animation's end on as
    /// `SECOND_IDLE`, `STATE_6B` set.  C clear.
    pub fn r_01c1a6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.zc_task_animate(r) {
            (r.x, _) = self.r_00053e(r.d6, SECOND_IDLE);
            self.write8(STATE_6B, 1);
            r.c = false;
        }
    }

    /// `func_01C1C0`, `SECOND_IDLE`: placed, and on by
    /// `STATE_6D`/`STATE_6C` (`zc_task_choose`).
    pub fn r_01c1c0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_task_choose(r, (SECOND_B, ANIM_SECOND_B), (SECOND_C, ANIM_SECOND_C));
    }

    /// `func_01C212`, `SECOND_B`: at its animation's end its sprite's
    /// frame `DONE_FRAME`, back to `SECOND_IDLE`, `STATE_6D` and
    /// `STATE_6C` cleared and `STATE_6B` set.  C clear.
    pub fn r_01c212(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.zc_task_animate(r) {
            self.write8(r.a5 + SPRITE_FRAME, DONE_FRAME);
            (r.x, _) = self.r_00053e(r.d6, SECOND_IDLE);
            self.write8(STATE_6D, 0);
            self.write8(STATE_6C, 0);
            self.write8(STATE_6B, 1);
            r.c = false;
        }
    }

    /// `func_01C23A`: a bare `rts` (`func_01C212`'s way out).
    pub fn r_01c23a(&mut self) {}

    /// `func_01C23C`, `SECOND_C`: at its animation's end `STATE_6C` and
    /// `STATE_6D` cleared and back to `SECOND_IDLE`.  C as
    /// `func_00053E` leaves it then, else clear.
    pub fn r_01c23c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.zc_task_animate(r) {
            self.write8(STATE_6C, 0);
            self.write8(STATE_6D, 0);
            (r.x, r.c) = self.r_00053e(r.d6, SECOND_IDLE);
        }
    }

    /// `func_01C258`: bytes set for what follows the screen: 2, 1, 0, 1
    /// from `BYTE_A937`, `SPARED`'s bit 0 cleared, `BYTE_9543` 3,
    /// `BYTE_954E` 0xFF, the words 0 and 0xC8.
    pub fn r_01c258(&mut self) {
        self.write8(BYTE_A937, 2);
        self.write8(BYTE_A938, 1);
        self.write8(BYTE_A939, 0);
        self.write8(BYTE_A93A, 1);
        let spared = self.read8(SPARED);
        self.write8(SPARED, spared & 0xFE);
        self.write8(BYTE_9543, 3);
        self.write8(BYTE_954E, 0xFF);
        self.write16(WORD_9544, 0);
        self.write16(WORD_9546, 0xC8);
    }
}
