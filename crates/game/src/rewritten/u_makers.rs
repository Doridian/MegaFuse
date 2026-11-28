//! The ending's makers of tasks (the routines the script's command 0x0F
//! calls: `func_01611E`, family 10) and the routines of the tasks they
//! make: a task that makes a task with a sprite each eighth frame
//! (`func_016240`, `func_016392`), a pair that rise and slow
//! (`func_01661E`), one that moves the scroll's y (`func_0166D0`), and
//! the like.  `STOP` set (`func_016D8A`) ends the makers.

use crate::{Game, Regs};

/// Not 0: `func_016256` and `func_0163BE` end (`func_016D8A` adds 1;
/// `func_016240`, `func_016392` clear it).
const STOP: u32 = 0xFFAD80;
/// The scripted task's index (`u_scene.rs`), the task running's.
const SCRIPTED: u32 = 0xFFAD81;
const CURRENT_TASK: u32 = 0xFF80EE;
/// A count of frames: the makers make a task when its low 3 bits are 0.
const BYTE_80D8: u32 = 0xFF80D8;
/// The place a task is made at (words, x then y; `func_00152A`): its
/// bytes.
const POSITION_X: u32 = 0xFF9F04;
const POSITION_X_LOW: u32 = 0xFF9F05;
const POSITION_Y: u32 = 0xFF9F06;
const POSITION_Y_LOW: u32 = 0xFF9F07;
/// The speed `func_00410C` makes the velocity for; the velocity (words,
/// x then y, and their bytes).
const SPEED: u32 = 0xFF9F00;
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_X_LOW: u32 = 0xFF9EC5;
const VELOCITY_Y: u32 = 0xFF9EC6;
/// `func_01653C`'s count of tasks (6 down) and their angle (0xE0 up by
/// 10).
const COUNT: u32 = 0xFF9ED3;
const ANGLE: u32 = 0xFF9ED5;
/// The scroll's y (`u_scene.rs`: to the V-scroll table's first word)
/// and a word added to the x (`func_0163BE`, `func_01643E`: by 2).
const OFFSET_Y: u32 = 0xFF9EB9;
const SCROLL_ADD: u32 = 0xFFADC4;

/// A task's fields: its x and y (words: the y's low byte), a count, a
/// byte the made tasks of `func_016256` move by (their x velocity), a
/// word (a speed: `func_01653C`'s tasks' y velocity, `func_016698`'s
/// speed) and the velocity kept by `func_016454`'s tasks (4 bytes from
/// 0x18) and its animation (a long).
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_Y_LOW: u32 = 0x11;
const TASK_COUNT: u32 = 0x16;
const TASK_17: u32 = 0x17;
const TASK_18: u32 = 0x18;
const TASK_19: u32 = 0x19;
const TASK_1A: u32 = 0x1A;
const TASK_1B: u32 = 0x1B;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's frame and flags (0x80: hidden?, as the other
/// routines set it below y 0xA0).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// The routines of the tasks made here and their animations.
const ROUTINE_16256: u32 = 0x016256;
const ROUTINE_162E0: u32 = 0x0162E0;
const ROUTINE_16368: u32 = 0x016368;
const ROUTINE_163BE: u32 = 0x0163BE;
const ROUTINE_1643E: u32 = 0x01643E;
const ROUTINE_16454: u32 = 0x016454;
const ROUTINE_164F0: u32 = 0x0164F0;
const ROUTINE_1651A: u32 = 0x01651A;
const ROUTINE_165B0: u32 = 0x0165B0;
const ROUTINE_165BE: u32 = 0x0165BE;
const ROUTINE_165EA: u32 = 0x0165EA;
const ROUTINE_16698: u32 = 0x016698;
const ROUTINE_16704: u32 = 0x016704;
const ROUTINE_1613C: u32 = 0x01613C;
const ANIM_D8FAE: u32 = 0x0D8FAE;
const ANIM_D8F38: u32 = 0x0D8F38;
const ANIM_D8FBE: u32 = 0x0D8FBE;
/// The y speeds of `func_01653C`'s tasks by its count (words in the ROM
/// after the `rts` at 0x0165A2, from count x 2).
const RISE_SPEEDS: u32 = 0x0165A2;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_001222` on the family's registers: a task of `routine`
    /// with a sprite record.  Gives back its C (set: none made).
    fn u_make(&mut self, r: &mut Regs, depth: u8, priority: u8, routine: u32) -> bool {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, priority, routine);
        r.c
    }

    /// `func_0003E8` on the family's registers: a task of `routine`.
    fn u_task(&mut self, r: &mut Regs, priority: u8, routine: u32) {
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, priority, routine);
    }

    /// The task `d6` (its index's low byte) into a6 (`func_0005F8`).
    fn u_task_at(&mut self, r: &mut Regs, index: u8) {
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, index));
    }

    /// `func_000560`: the running task ends.
    fn u_end(&mut self, r: &mut Regs) {
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }

    /// `func_00121A`: the running task ends with its sprite record.
    fn u_end_sprite(&mut self, r: &mut Regs) {
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
    }

    /// `func_0013D6`: the task's animation a frame on; its C (set: the
    /// animation ended).
    fn u_animate(&mut self, r: &mut Regs) -> bool {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        r.c
    }

    /// The velocity from the task's 4 bytes at 0x18.
    fn u_velocity_from_task(&mut self, a6: u32) {
        for i in 0..4 {
            let b = self.read8(a6 + TASK_18 + i);
            self.write8(VELOCITY_X + i, b);
        }
    }

    /// `func_016240`: `STOP` cleared and a task of `func_016256` made
    /// (`func_0003E8`, priority 0x42).
    pub fn r_016240(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.write8(STOP, 0);
        self.u_task(regs, 0x42, ROUTINE_16256);
    }

    /// `func_016256`, a task's routine: it ends once `STOP` is set; else
    /// each eighth frame it makes a task of `func_0162E0` (depth 0x90)
    /// at x a random byte, y 0x100 (the x's byte then doubled): its x
    /// velocity byte 1 (the
    /// `neg` for a carry out of the doubled random byte never runs: the
    /// `move` after the `add` clears C), its word 0x1A 0xF7xx (xx
    /// random), its sprite's frame 0x1B or 0x1C (random) and flags 0,
    /// placed (`func_015F46`).
    pub fn r_016256(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let stop = self.read8(STOP);
        r.d0 = low_byte(r.d0, stop);
        r.c = false;
        if stop != 0 {
            self.u_end(r);
            return;
        }
        let frame = self.read8(BYTE_80D8) & 7;
        r.d0 = low_byte(r.d0, frame);
        if frame != 0 {
            return;
        }
        self.write16(POSITION_Y, 0x100);
        self.write8(POSITION_X, 0);
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        self.write8(POSITION_X_LOW, r.d0 as u8);
        if self.u_make(r, 0x90, 0xC2, ROUTINE_162E0) {
            return;
        }
        self.r_00152a(r.a6);
        r.d0 = 1;
        let doubled = self.read8(POSITION_X_LOW);
        let (doubled_value, carry) = doubled.overflowing_add(doubled);
        r.x = carry;
        r.d1 = low_byte(r.d1, doubled_value);
        self.write8(POSITION_X_LOW, doubled_value);
        self.write8(r.a6 + TASK_17, 1);
        self.write8(r.a6 + TASK_1A, 0xF7);
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        self.write8(r.a6 + TASK_1B, r.d0 as u8);
        self.write8(r.a5 + SPRITE_FLAGS, 0);
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        r.d0 = low_byte(r.d0, (r.d0 as u8 & 1) + 0x1B);
        r.x = false;
        self.write8(r.a5 + SPRITE_FRAME, r.d0 as u8);
        r.c = self.r_015f46(r.a5, r.a6);
    }

    /// `func_0162E0`, a task's routine: moved by its x velocity byte
    /// (`func_001A88`) and by its word 0x1A as the y velocity
    /// (`func_016312`), its depth 0x70 while that is not negative
    /// (`func_0007D2`); it ends (`func_00121A`) when its y is 0x100 or
    /// more (the high byte not 0), else its sprite is placed
    /// (`func_015F46`).
    pub fn r_0162e0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(VELOCITY_X_LOW, 0);
        let vx = self.read8(r.a6 + TASK_17);
        self.write8(VELOCITY_X, vx);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        let negative;
        (r.d0, r.x, negative) = self.r_016312(r.a6);
        if !negative {
            r.d0 = 0x70;
            r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
        }
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        if self.read8(r.a6 + TASK_Y) != 0 {
            self.u_end_sprite(r);
            return;
        }
        r.c = self.r_015f46(r.a5, r.a6);
    }

    /// `func_016312`: the task's word 0x1A up by 0x30, to `VELOCITY_Y`;
    /// its high byte at most 9 (when not negative), to the task and
    /// `VELOCITY_Y`'s high byte.  Gives back d0 (the byte), X and N (by
    /// the byte).
    pub fn r_016312(&mut self, a6: u32) -> (u32, bool, bool) {
        let (word, x) = self.read16(a6 + TASK_1A).overflowing_add(0x30);
        self.write16(a6 + TASK_1A, word);
        self.write16(VELOCITY_Y, word);
        let high = (word >> 8) as u8;
        let (high, d0) = if (high as i8) >= 9 { (9, 9) } else { (high, u32::from(word & 0xFF00) | u32::from(high)) };
        self.write8(a6 + TASK_1A, high);
        self.write8(VELOCITY_Y, high);
        (d0, x, (high as i8) < 0)
    }

    /// `func_016340`: a task of `func_016368` (depth 0x80) with
    /// animation D_0D8FAE, its sprite's flags 0.
    pub fn r_016340(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.u_make(r, 0x80, 0xC2, ROUTINE_16368) {
            return;
        }
        self.write8(r.a5 + SPRITE_FLAGS, 0);
        self.write32(r.a6 + TASK_ANIM, ANIM_D8FAE);
        r.c = false;
    }

    /// `func_016368`, a task's routine: its animation a frame on
    /// (`func_0013D6`; at its end the task ends, `func_00121A`), put
    /// where the scripted task is and its sprite placed
    /// (`func_015F46`).
    pub fn r_016368(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.u_animate(r) {
            self.u_end_sprite(r);
            return;
        }
        let scripted = self.read8(SCRIPTED);
        self.u_task_at(r, scripted);
        self.r_00153a(r.a6);
        let current = self.read8(CURRENT_TASK);
        self.u_task_at(r, current);
        self.r_00152a(r.a6);
        r.c = self.r_015f46(r.a5, r.a6);
    }

    /// `func_016392`: `STOP` cleared; tasks of `func_0163BE` and of
    /// `func_016454` (priority 0x42), the second's count 0x40.
    pub fn r_016392(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(STOP, 0);
        self.u_task(r, 0x42, ROUTINE_163BE);
        self.u_task(r, 0x42, ROUTINE_16454);
        self.write8(r.a6 + TASK_COUNT, 0x40);
        r.c = false;
    }

    /// `func_0163BE`, a task's routine: once `STOP` is set its routine
    /// is `func_01643E` (from now: it runs it).  Else the scroll's y
    /// goes down to 0x40, `SCROLL_ADD` up by 2, and each eighth frame a
    /// task of `func_0165B0` (depth 0x70) is made at a random x (its
    /// high byte 0), a random y from 0x50 to 0x14F, with animation
    /// D_0D8F38, its sprite's flags 0x40, and placed (`func_015F46`).
    pub fn r_0163be(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let stop = self.read8(STOP);
        r.d0 = low_byte(r.d0, stop);
        if stop != 0 {
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_1643E);
            self.r_01643e(r);
            return;
        }
        let y = self.read8(OFFSET_Y);
        if y != 0x40 {
            self.write8(OFFSET_Y, y.wrapping_sub(1));
            r.x = y == 0;
        }
        let (add, carry) = self.read16(SCROLL_ADD).overflowing_add(2);
        self.write16(SCROLL_ADD, add);
        r.x = carry;
        let frame = self.read8(BYTE_80D8) & 7;
        r.d0 = low_byte(r.d0, frame);
        r.c = false;
        if frame != 0 {
            return;
        }
        self.write8(POSITION_X, 0);
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        self.write8(POSITION_X_LOW, r.d0 as u8);
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        let y = (r.d0 as u16 & 0xFF) + 0x50;
        r.d0 = low_word(r.d0, y);
        r.x = false;
        self.write16(POSITION_Y, y);
        if self.u_make(r, 0x70, 0xC2, ROUTINE_165B0) {
            return;
        }
        self.r_00152a(r.a6);
        self.write8(r.a5 + SPRITE_FLAGS, 0x40);
        self.write32(r.a6 + TASK_ANIM, ANIM_D8F38);
        r.c = self.r_015f46(r.a5, r.a6);
    }

    /// `.L016430`: returns.
    pub fn r_016430(&mut self) {}

    /// `func_01643E`, a task's routine: `SCROLL_ADD` up by 2 and the
    /// scroll's y up by 1; at 0x80 the task ends.  Gives back C (the
    /// compare's with 0x80 when it goes on).
    pub fn r_01643e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let add = self.read16(SCROLL_ADD).wrapping_add(2);
        self.write16(SCROLL_ADD, add);
        let (y, carry) = self.read8(OFFSET_Y).overflowing_add(1);
        self.write8(OFFSET_Y, y);
        r.x = carry;
        r.d0 = low_byte(r.d0, y);
        if y != 0x80 {
            r.c = y < 0x80;
            return;
        }
        self.u_end(r);
    }

    /// `func_016454`, a task's routine: its count down each frame; when
    /// it is 0 its routine is `func_0164F0` (from the next frame).  On
    /// odd counts: the place is the scripted task's (`func_00153A`)
    /// moved by two random amounts (0-0x1F) and a task of
    /// `func_0165BE` (depth 0x40) is made there with animation
    /// D_0D8F38, its sprite's flags 0x40, and the velocity of speed 0x80
    /// at a random angle from 0x80 to 0xFF (`func_00410C`) kept in it.
    pub fn r_016454(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(r.a6 + TASK_COUNT);
        r.d0 = low_byte(r.d0, count);
        if count == 0 {
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_164F0);
            return;
        }
        self.write8(r.a6 + TASK_COUNT, count - 1);
        r.x = false;
        r.d0 = low_byte(r.d0, count & 1);
        r.c = false;
        if count & 1 != 0 {
            return;
        }
        let scripted = self.read8(SCRIPTED);
        self.u_task_at(r, scripted);
        self.r_00153a(r.a6);
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        r.d0 = low_byte(r.d0, r.d0 as u8 & 0x1F);
        let (x, carry) = self.read8(POSITION_X_LOW).overflowing_add(r.d0 as u8);
        self.write8(POSITION_X_LOW, x);
        r.x = carry;
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        r.d0 = low_byte(r.d0, r.d0 as u8 & 0x1F);
        let (y, carry) = self.read8(POSITION_Y_LOW).overflowing_add(r.d0 as u8);
        self.write8(POSITION_Y_LOW, y);
        r.x = carry;
        if self.u_make(r, 0x40, 0xC1, ROUTINE_165BE) {
            return;
        }
        self.r_00152a(r.a6);
        self.write8(r.a5 + SPRITE_FLAGS, 0x40);
        self.write32(r.a6 + TASK_ANIM, ANIM_D8F38);
        self.write16(SPEED, 0x80);
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        r.d0 = low_byte(r.d0, (r.d0 as u8 & 0x7F) | 0x80);
        (r.d0, r.d1, r.d2, r.x) = self.r_00410c(r.d0, r.d1, r.d2, r.x);
        for i in 0..4 {
            let b = self.read8(VELOCITY_X + i);
            self.write8(r.a6 + TASK_18 + i, b);
        }
        r.c = false;
    }

    /// `func_0164F0`, a task's routine: once the scripted task's y
    /// (its low byte) is below 0x30, `func_01653C` and its routine is
    /// `func_01651A` (from the next frame).
    pub fn r_0164f0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let scripted = self.read8(SCRIPTED);
        self.u_task_at(r, scripted);
        let y = self.read8(r.a6 + TASK_Y_LOW);
        r.d0 = low_byte(r.d0, y);
        if y >= 0x30 {
            r.c = false;
            return;
        }
        (r.d0, r.d1, r.d2, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_01653c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_1651A);
    }

    /// `func_01651A`, a task's routine: once the scripted task's y (its
    /// low byte) is 0x30 or more, the task ends and `func_01653C` runs
    /// for the scripted task.  Gives back C (set while it waits).
    pub fn r_01651a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let scripted = self.read8(SCRIPTED);
        self.u_task_at(r, scripted);
        let y = self.read8(r.a6 + TASK_Y_LOW);
        r.d0 = low_byte(r.d0, y);
        if y < 0x30 {
            r.c = true;
            return;
        }
        self.u_end(r);
        let scripted = self.read8(SCRIPTED);
        self.u_task_at(r, scripted);
        (r.d0, r.d1, r.d2, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_01653c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `func_01653C`: from where the task at a6 is (`func_00153A`), six
    /// tasks of `func_0165EA` (depth 0x60; fewer if one can't be made),
    /// their sprites' flags 0x40, each with the x velocity of speed
    /// 0x300 at angles 0xE0, 0xEA, ... (`func_00410C`) and a y speed
    /// from `RISE_SPEEDS`.  Gives back d0, d1, d2, d5, d6, a5, a6, X
    /// and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01653c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        self.r_00153a(a6);
        self.write16(SPEED, 0x300);
        self.write8(COUNT, 6);
        self.write8(ANGLE, 0xE0);
        loop {
            if self.u_make(&mut r, 0x60, 0x42, ROUTINE_165EA) {
                break;
            }
            self.r_00152a(r.a6);
            self.write8(r.a5 + SPRITE_FLAGS, 0x40);
            let angle = self.read8(ANGLE);
            (r.d0, r.d1, r.d2, r.x) = self.r_00410c(low_byte(r.d0, angle), r.d1, r.d2, r.x);
            let vx = self.read8(VELOCITY_X);
            self.write8(r.a6 + TASK_18, vx);
            let vx_low = self.read8(VELOCITY_X_LOW);
            self.write8(r.a6 + TASK_19, vx_low);
            let count = self.read8(COUNT);
            let at = (count.wrapping_add(count)) as i8 as u16;
            r.d0 = low_word(r.d0, at);
            let rise = self.read16(RISE_SPEEDS.wrapping_add(at as i16 as u32));
            self.write16(r.a6 + TASK_1A, rise);
            r.d0 = 0x0A;
            let angle = self.read8(ANGLE);
            self.write8(ANGLE, angle.wrapping_add(0x0A));
            let count = self.read8(COUNT).wrapping_sub(1);
            self.write8(COUNT, count);
            r.x = false;
            r.c = false;
            if count == 0 {
                break;
            }
        }
        (r.d0, r.d1, r.d2, r.d5, r.d6, r.a5, r.a6, r.x, r.c)
    }

    /// `func_0165B0`, a task's routine: its animation a frame on
    /// (`func_0013D6`); at its end the task ends (`func_00121A`).
    pub fn r_0165b0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.u_animate(r) {
            (r.d6, r.x, r.c) = self.r_0165ba(r.d5, r.d6, r.x);
        }
    }

    /// `.L0165BA`: the task ends with its sprite record
    /// (`func_00121A`).  Gives back d6, X and C.
    pub fn r_0165ba(&mut self, d5: u32, d6: u32, x: bool) -> (u32, bool, bool) {
        self.r_00121a(d5, d6, x)
    }

    /// `func_0165BE`, a task's routine: its animation a frame on (at its
    /// end the task ends: `func_0165BA`), moved by its kept velocity
    /// (`func_001A88`, `func_001AA2`) and its sprite placed
    /// (`func_015F46`).
    pub fn r_0165be(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.u_animate(r) {
            (r.d6, r.x, r.c) = self.r_0165ba(r.d5, r.d6, r.x);
            return;
        }
        self.u_velocity_from_task(r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        r.c = self.r_015f46(r.a5, r.a6);
    }

    /// `func_0165EA`, a task's routine: it ends (`func_0165BA`) once its
    /// y (the low byte) is 0x40 or more; else its sprite's frame is 6,
    /// it moves by its kept x velocity and by its word 0x1A as the y
    /// velocity, that sped up first (`func_001B14`), and its sprite is
    /// placed (`func_015F46`).
    pub fn r_0165ea(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let y = self.read8(r.a6 + TASK_Y_LOW);
        r.d0 = low_byte(r.d0, y);
        if y >= 0x40 {
            (r.d6, r.x, r.c) = self.r_0165ba(r.d5, r.d6, r.x);
            return;
        }
        self.write8(r.a5 + SPRITE_FRAME, 6);
        let vx = self.read8(r.a6 + TASK_18);
        self.write8(VELOCITY_X, vx);
        let vx_low = self.read8(r.a6 + TASK_19);
        self.write8(VELOCITY_X_LOW, vx_low);
        r.d0 = self.r_001b14(r.a6);
        let vy = self.read8(r.a6 + TASK_1A);
        self.write8(VELOCITY_Y, vy);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        r.c = self.r_015f46(r.a5, r.a6);
    }

    /// `func_01661E`: two tasks of `func_016698`, (depth 0x40, x
    /// velocity 0xFF80, y 0xC0, frame 0x16, speed 0x600) and (depth
    /// 0x90, x velocity 0xFFC0, y 0xC0, frame 0x17, speed 0x400), each
    /// set going by `func_01667A` (whether made or not).
    pub fn r_01661e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.u_make(r, 0x40, 0x42, ROUTINE_16698);
        self.write16(r.a6 + TASK_X, 0xFF80);
        self.write16(r.a6 + TASK_Y, 0xC0);
        self.write8(r.a5 + SPRITE_FRAME, 0x16);
        self.r_01667a(r.a5, r.a6);
        self.write8(r.a6 + TASK_1A, 6);
        self.u_make(r, 0x90, 0x42, ROUTINE_16698);
        self.write16(r.a6 + TASK_X, 0xFFC0);
        self.write16(r.a6 + TASK_Y, 0xC0);
        self.write8(r.a5 + SPRITE_FRAME, 0x17);
        self.write8(r.a6 + TASK_1A, 4);
        self.r_01667a(r.a5, r.a6);
        r.c = false;
    }

    /// `func_01667A`: the task's count 0xFF, its angle (byte 0x17)
    /// 0x40, its sprite's flags 0, placed (`func_015F46`), and the low
    /// byte of its speed (0x1B) 0.
    pub fn r_01667a(&mut self, a5: u32, a6: u32) {
        self.write8(a6 + TASK_COUNT, 0xFF);
        self.write8(a6 + TASK_17, 0x40);
        self.write8(a5 + SPRITE_FLAGS, 0);
        self.r_015f46(a5, a6);
        self.write8(a6 + TASK_1B, 0);
    }

    /// `func_016698`, a task's routine: it ends (`func_00121A`) when its
    /// count is 0; else the count goes down, its angle down by 1 on even
    /// counts, and it moves at its speed (word 0x1A) and angle
    /// (`func_00410C`, `func_001A88`, `func_001AA2`), its sprite placed
    /// (`func_015F46`).
    pub fn r_016698(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(r.a6 + TASK_COUNT);
        r.d0 = low_byte(r.d0, count);
        if count == 0 {
            self.u_end_sprite(r);
            return;
        }
        self.write8(r.a6 + TASK_COUNT, count - 1);
        r.x = false;
        if count & 1 == 0 {
            let angle = self.read8(r.a6 + TASK_17);
            self.write8(r.a6 + TASK_17, angle.wrapping_sub(1));
            r.x = angle == 0;
        }
        let speed = self.read16(r.a6 + TASK_1A);
        self.write16(SPEED, speed);
        let angle = self.read8(r.a6 + TASK_17);
        (r.d0, r.d1, r.d2, r.x) = self.r_00410c(low_byte(r.d0, angle), r.d1, r.d2, r.x);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        r.c = self.r_015f46(r.a5, r.a6);
    }

    /// `func_0166D0`: a task of `func_016704` (depth 0xFE, priority
    /// 0x30; not checked for being made) at x 0xD0, y 0xFFD0, its
    /// sprite's frame 0x1A, flags 0, placed (`func_015F46`).
    pub fn r_0166d0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.u_make(r, 0xFE, 0x30, ROUTINE_16704);
        self.write8(r.a5 + SPRITE_FLAGS, 0);
        self.write8(r.a5 + SPRITE_FRAME, 0x1A);
        self.write16(r.a6 + TASK_X, 0xD0);
        self.write16(r.a6 + TASK_Y, 0xFFD0);
        r.c = self.r_015f46(r.a5, r.a6);
    }

    /// `func_016702`: returns.  No script calls it.
    pub fn r_016702(&mut self) {}

    /// `func_016704`, a task's routine: the scroll's y up by 1; at 0xE0
    /// the task ends, else it moves down by 0x100 (`func_001AA2`) and its
    /// sprite is placed (`func_015F46`).
    pub fn r_016704(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (y, carry) = self.read8(OFFSET_Y).overflowing_add(1);
        self.write8(OFFSET_Y, y);
        r.x = carry;
        if y == 0xE0 {
            self.u_end(r);
            return;
        }
        self.write16(VELOCITY_Y, 0x100);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        r.c = self.r_015f46(r.a5, r.a6);
    }

    /// `func_016D8A`: `STOP` up by 1.
    pub fn r_016d8a(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let stop = self.read8(STOP);
        self.write8(STOP, stop.wrapping_add(1));
    }

    /// `func_016D90`: a task of `func_01613C` (depth 0x7F, priority
    /// 0x41; not checked for being made) with animation D_0D8FBE, its
    /// sprite's flags 0x40, its byte 0x16 1.
    pub fn r_016d90(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.u_make(r, 0x7F, 0x41, ROUTINE_1613C);
        self.write32(r.a6 + TASK_ANIM, ANIM_D8FBE);
        self.write8(r.a5 + SPRITE_FLAGS, 0x40);
        self.write8(r.a6 + TASK_COUNT, 1);
        r.c = false;
    }
}
