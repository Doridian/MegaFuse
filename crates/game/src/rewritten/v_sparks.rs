//! Pairs of sprites sent out from the middle (0x0175A4-0x01776C): the
//! task of `func_0175A4` makes a task of `func_01768A` every
//! `EVERY` frames, with two sprites, four random directions and
//! speeds; `func_01768A` moves them out along those directions
//! (`v_wave.rs`), one pair of directions a frame.

use crate::{Game, Regs};

/// The making task's fields: a frame count (bytes), and a count less
/// 2 for each task made (it ends below 0).
const FRAMES: u32 = 0x0D;
const MADE_LEFT: u32 = 0x0C;
/// The made task's fields: its sprite record (the task's own, as the
/// scheduler takes it) and its second one; four directions, four
/// speeds, four lengths (bytes, from 0x0C, 0x10, 0x14), and a frame
/// count (whose bit 0 picks the directions moved).
const SPRITE: u32 = 0x03;
const SPRITE_2: u32 = 0x18;
const DIRECTIONS: u32 = 0x0C;
const SPEEDS: u32 = 0x10;
const LENGTHS: u32 = 0x14;
const MOVED: u32 = 0x19;
/// A sprite record's frame, flags, and the low bytes of its x and y.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y_LOW: u32 = 0x0B;

/// The running task's index.
const CURRENT_TASK: u32 = 0xFFFF_80EE;
/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// How often a task is made (frames), the made task's routine and
/// priority, its sprites' depth, frame and flags.
const EVERY: u8 = 5;
const ROUTINE_01768A: u32 = 0x01768A;
const PRIORITY: u8 = 0xF0;
const DEPTH: u8 = 0xFA;
const FRAME: u8 = 0x11;
const FLAGS: u8 = 0x01;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_000796` on the family's registers: a sprite record at
    /// `DEPTH`; true if there was none.
    fn v_new_sprite(&mut self, r: &mut Regs) -> bool {
        r.d0 = low_byte(r.d0, DEPTH);
        (r.d5, r.a5, r.x, r.c) = self.r_000796(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
        r.c
    }

    /// Sprite record `field` of the task (`func_000B5A`) gets
    /// `FRAME` and `FLAGS`.
    fn v_show(&mut self, r: &mut Regs, field: u32) {
        r.d5 = low_byte(r.d5, self.read8(r.a6 + field));
        (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
        self.write8(r.a5 + SPRITE_FRAME, FRAME);
        self.write8(r.a5 + SPRITE_FLAGS, FLAGS);
    }

    /// The sprite at a5 put at length `which` along direction `which`
    /// (x `func_017770`, y `func_01776C`).
    fn v_put(&mut self, r: &mut Regs, which: u32) {
        for field in [SPRITE_X_LOW, SPRITE_Y_LOW] {
            r.d1 = low_byte(r.d1, self.read8(r.a6 + LENGTHS + which));
            r.d0 = low_byte(r.d0, self.read8(r.a6 + DIRECTIONS + which));
            (r.d0, r.x) = if field == SPRITE_X_LOW { self.r_017770(r.d0, r.d1, r.d3) } else { self.r_01776c(r.d0, r.d1, r.d3) };
            self.write8(r.a5 + field, r.d0 as u8);
        }
    }

    /// `func_0175A4`, a task's routine: every `EVERY` frames a task of
    /// `func_01768A` (`PRIORITY`) with two sprite records (`DEPTH`,
    /// `FRAME`, `FLAGS`), four random directions and four speeds of
    /// 1-4 (`func_001BE2`), its lengths 0; when there is no record the
    /// made task ends.  Once its `MADE_LEFT` goes below 0 the task
    /// ends.
    pub fn r_0175a4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let frames = self.read8(r.a6 + FRAMES).wrapping_add(1);
        self.write8(r.a6 + FRAMES, frames);
        r.x = frames == 0;
        r.c = frames < EVERY;
        if r.c {
            return;
        }
        self.write8(r.a6 + FRAMES, 0);
        r.a0 = ROUTINE_01768A;
        r.d0 = low_byte(r.d0, PRIORITY);
        (r.d6, r.a6, r.x, r.c) = self.r_000404(r.d0, r.d4, r.d5, r.d6, r.a0);
        if r.c {
            return;
        }
        for field in [SPRITE, SPRITE_2] {
            if self.v_new_sprite(r) {
                (r.x, r.c) = self.r_000564(r.d5, r.d6, r.x);
                return;
            }
            self.write8(r.a6 + field, r.d5 as u8);
        }
        for i in 0..4 {
            (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
            self.write8(r.a6 + DIRECTIONS + i, r.d0 as u8);
        }
        for i in 0..4 {
            (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
            r.d0 = low_byte(r.d0, (r.d0 as u8 & 3) + 1);
            // addq.b #1 on 0-3, which never carries
            r.x = false;
            self.write8(r.a6 + SPEEDS + i, r.d0 as u8);
        }
        for i in 0..4 {
            self.write8(r.a6 + LENGTHS + i, 0);
        }
        self.v_show(r, SPRITE);
        self.v_show(r, SPRITE_2);
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        let left = self.read8(r.a6 + MADE_LEFT);
        self.write8(r.a6 + MADE_LEFT, left.wrapping_sub(2));
        r.x = left < 2;
        r.c = r.x;
        if left.wrapping_sub(2) & 0x80 == 0 {
            return;
        }
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }

    /// `func_01768A`, a task's routine: each length grows by its speed;
    /// on odd frames (`MOVED`) the second sprite goes along direction
    /// 0 and the task's own along 1, on even ones along 2 and 3.
    pub fn r_01768a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        for i in 0..4 {
            let speed = self.read8(r.a6 + SPEEDS + i);
            r.d0 = low_byte(r.d0, speed);
            let length = self.read8(r.a6 + LENGTHS + i);
            self.write8(r.a6 + LENGTHS + i, length.wrapping_add(speed));
        }
        let moved = self.read8(r.a6 + MOVED).wrapping_add(1);
        self.write8(r.a6 + MOVED, moved);
        let first = if moved & 1 != 0 { 0 } else { 2 };
        r.d5 = low_byte(r.d5, self.read8(r.a6 + SPRITE_2));
        (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
        self.v_put(r, first);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        self.v_put(r, first + 1);
        r.c = false;
    }
}
