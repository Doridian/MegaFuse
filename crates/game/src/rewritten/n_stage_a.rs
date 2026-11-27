//! Routines of the table at `D_00A2AC` (and `func_00E076`, the long
//! after `D_00A250`'s three), which make a stage's tasks: each sets the
//! frame `func_00CA06` gives the new sprite (`NEW_FRAME`), makes its
//! task with a sprite (`func_001222`) and gives back the byte d0 its
//! caller takes, C clear.  Here: the task of three sprites that falls
//! when its cell is 8 (`func_00E0B6`), and the pair of `func_00E228`
//! (a task that flashes its frame, and one whose count runs down to a
//! fall, a fade and a task of its own, `func_00E30C` on).

use crate::{Game, Regs};

/// Settings bits: bit 0 is tested by `func_00E076`, `func_00E29E` and
/// `func_00E30C`.
const SETTINGS: u32 = 0xFF9540;
/// The frame `func_00CA06` gives the sprite of the task just made.
const NEW_FRAME: u32 = 0xFFAC91;
/// A count, one more for each task `func_00E228` makes: the task's
/// `TASK_COUNT` (its frames to wait) starts at it.
const MADE: u32 = 0xFFACE0;
/// A byte of `func_00E30C`'s: below 2, its count runs down 9 a frame.
const FAST_BELOW: u32 = 0xFFA429;
/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// The running task's index.
const CURRENT_TASK: u32 = 0xFF80EE;
/// A cell (column, row; one word), and a position's y (a word) the new
/// tasks are made at.
const CELL: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
const POSITION_Y: u32 = 0xFF9F06;
/// A velocity's y (a word) for `func_001AA2`.
const VELOCITY_Y: u32 = 0xFF9EC6;
/// Where `func_00E4CA` keeps byte 2 of the task's sprite record, the
/// depth (`func_0007D2`) of the sprite of the task it makes.
const KEPT_DEPTH: u32 = 0xFFACEC;

/// A task's fields: its sprite's index, flags (the low byte of its y;
/// `func_00E466` counts with it), x's fraction (its low nibble a random
/// rate for `func_00E4CA`'s task), a step (`func_00E170`), a count of
/// frames (a byte, or a word with `TASK_COUNT_LOW`), a byte flag or
/// count, the two sprites' indexes (`func_00E0B6`) or a task's index
/// and a flag (`func_00E228`), its rate (a word), its offset (a word,
/// whose high byte is the sprite's y offset), the cell's address and
/// the cell (`func_00E11C`).
const TASK_SPRITE: u32 = 0x03;
const TASK_LIFE: u32 = 0x10;
const TASK_Y_LOW: u32 = 0x11;
const TASK_RANDOM: u32 = 0x0F;
const TASK_STEP: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_17: u32 = 0x17;
const TASK_18: u32 = 0x18;
const TASK_19: u32 = 0x19;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;
const TASK_CELL_AT: u32 = 0x24;
const TASK_CELL: u32 = 0x28;
/// A sprite record's byte 2, frame and flags.
const SPRITE_2: u32 = 0x02;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
/// The flags `func_00E21C` gives a sprite (and `func_00CA06`).
const FLAGS_SHOWN: u8 = 0x04;

/// `func_00E0B6`'s task: its routine, priority, depth; the frames of
/// its sprites (the falling one by step: `D_00E1D2`, bytes in the
/// ROM), its cell's byte that makes it fall, the rate it starts at.
const THREE: u32 = 0x00E11C;
const THREE_AFTER: u32 = 0x00E142;
const THREE_FALLING: u32 = 0x00E170;
const THREE_FRAME: u8 = 0xAB;
const THREE_FRAME_2: u8 = 0xAA;
const THREE_FRAME_3: u8 = 0xAC;
const FALLING_FRAMES: u32 = 0x00E1D2;
const FALL_CELL: u8 = 8;
const FALL_RATE: u16 = 0xFA00;
/// The other sprites' y offsets (`func_00E1D6`).
const OFFSET_2: u8 = 0xE0;
const OFFSET_3: u8 = 0xF8;

/// `func_00E228`'s tasks: routines, frames.
const FLASHER: u32 = 0x00E29E;
const FLASHER_ON: u32 = 0x00E2B4;
const FLASHER_FLASH: u32 = 0x00E2D4;
const FLASHER_FRAME: u8 = 0xAF;
const FLASH_FRAME: u8 = 0xB0;
const WAITER: u32 = 0x00E30C;
const WAITER_COUNTING: u32 = 0x00E322;
const WAITER_FRAME: u8 = 0xB1;
/// What follows when the count runs out: the rise (`func_00E39C`), the
/// stop (`func_00E420`), the sprite's animation then (`D_0DA5C6`), the
/// count again (`func_00E466`, `func_00E49E`), the tasks made on the
/// way (`func_00E558`, `func_00E548` with `D_0DA5A8`).
const RISING: u32 = 0x00E39C;
const STOPPING: u32 = 0x00E420;
const STOPPED_ANIM: u32 = 0x0DA5C6;
const ANIMATING: u32 = 0x00E466;
const BLINKING: u32 = 0x00E49E;
const BLINK_FRAME: u8 = 0xB1;
const STOP_FRAME: u8 = 0xB2;
const LEFT_BEHIND: u32 = 0x00E558;
const LEFT_FRAME: u8 = 0xB3;
const PUFF: u32 = 0x00E548;
const PUFF_ANIM: u32 = 0x0DA5A8;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_00E076`: tests bit 0 of `SETTINGS` and does nothing either
    /// way.
    pub fn r_00e076(&mut self, regs: &mut Regs) {
        let _ = self.read8(SETTINGS);
        regs.unwound = false;
    }

    /// `func_00E090`: frame 0xA9, then `loc_00C7E0`'s task.
    pub fn r_00e090(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.write8(NEW_FRAME, 0xA9);
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00c7e0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        r.unwound = false;
    }

    /// `func_00E09A`: frame 0xA8, then `func_00C79C`'s task.
    pub fn r_00e09a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.write8(NEW_FRAME, 0xA8);
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00c79c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        r.unwound = false;
    }

    /// `func_00E0A4`: nothing made; d0's low byte 0, C clear.
    pub fn r_00e0a4(&mut self, regs: &mut Regs) {
        regs.d0 = low_byte(regs.d0, 0);
        regs.c = false;
        regs.unwound = false;
    }

    /// `func_00E0AE`: the task's sprite placed (`func_001B7E`) and its
    /// animation a frame on (`func_0013D6`).
    /// No script calls it: reviewed by reading.
    pub fn r_00e0ae(&mut self, d1: u32, d2: u32, d5: u32, a5: u32, a6: u32) {
        let (d0, _, _) = self.r_001b7e(a5, a6);
        self.r_0013d6(d0, d1, d2, d5, a6);
    }

    /// `func_00E0B6`: a task (`THREE`) made 0x18 pixels lower than
    /// `POSITION_Y`, with two more sprite records (`func_000796`), their
    /// indexes in `TASK_18` and `TASK_19`, shown with frames
    /// `THREE_FRAME_2` and `THREE_FRAME_3`.  Gives back d0 = 6, d5, d6,
    /// a5, a6, X, C clear.
    pub fn r_00e0b6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let y = self.read16(POSITION_Y);
        self.write16(POSITION_Y, y.wrapping_add(0x18));
        self.write8(NEW_FRAME, THREE_FRAME);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 0x10, 0x42, THREE);
        (r.d0, r.c) = self.r_00ca06(r.d0, r.a5, r.a6);
        self.write8(r.a6 + TASK_OFFSET, 0);
        let y = self.read16(POSITION_Y);
        self.write16(POSITION_Y, y.wrapping_sub(0x18));
        r.x = y < 0x18;
        for (field, frame) in [(TASK_18, THREE_FRAME_2), (TASK_19, THREE_FRAME_3)] {
            r.d0 = low_byte(r.d0, 0x80);
            (r.d5, r.a5, r.x, r.c) = self.r_000796(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
            self.write8(r.a6 + field, r.d5 as u8);
            r.d0 = low_byte(r.d0, frame);
            self.r_00e21c(r.d0, r.a5);
        }
        r.d0 = low_byte(r.d0, 6);
        r.c = false;
        r.unwound = false;
    }

    /// `func_00E11C`, `THREE`'s routine: from the next frame
    /// `THREE_AFTER`; the cell above the task's is kept (`TASK_CELL`,
    /// `TASK_CELL_AT`) and looked at (`func_00E14C`).
    pub fn r_00e11c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.r_00053e(r.d6, THREE_AFTER);
        r.d0 = self.r_0015f8(r.a6);
        let row = self.read8(CELL_Y);
        self.write8(CELL_Y, row.wrapping_sub(1));
        let cell = self.read16(CELL);
        self.write16(r.a6 + TASK_CELL, cell);
        (r.a0, _) = self.r_00167a();
        self.write32(r.a6 + TASK_CELL_AT, r.a0);
        (r.d0, r.d1, r.x, r.c) = self.r_00e14c(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        r.unwound = false;
    }

    /// `func_00E142`, `THREE_AFTER`: the kept cell looked at again
    /// (`func_00E14C`).
    pub fn r_00e142(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let cell = self.read16(r.a6 + TASK_CELL);
        self.write16(CELL, cell);
        r.a0 = self.read32(r.a6 + TASK_CELL_AT);
        (r.d0, r.d1, r.x, r.c) = self.r_00e14c(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        r.unwound = false;
    }

    /// `loc_00E14C`: when the cell at a0 is `FALL_CELL` the task falls
    /// from the next frame (`THREE_FALLING`, its rate `FALL_RATE`, its
    /// step count 3) and its three sprites are drawn (`func_00E1D6`);
    /// otherwise its sprite is placed first (`loc_00E1C6`).  Gives back
    /// d0, d1, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00e14c(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, bool, bool) {
        if self.read8(a0) != FALL_CELL {
            return self.r_00e1c6(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5, a6);
        }
        self.write16(a6 + TASK_RATE, FALL_RATE);
        self.write8(a6 + TASK_COUNT, 3);
        self.r_00053e(d6, THREE_FALLING);
        self.r_00e1d6(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a6)
    }

    /// `func_00E170`, `THREE_FALLING`: the rate 0x30 more (`func_00C742`)
    /// and the task's offset with it (`func_001ACA`).  Once the offset
    /// is down (not negative) it stops: offset 0, frame `THREE_FRAME`,
    /// `THREE` again from the next frame.  While it falls its frame
    /// steps every 3 frames through `FALLING_FRAMES`.  Then it is drawn
    /// (`loc_00E1C6`).
    pub fn r_00e170(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.d0 = 0x30;
        self.r_00c742(r.d0, r.a6);
        let up;
        (r.d0, r.x, up) = self.r_001aca(r.d0, r.a6);
        if !up {
            self.write16(r.a6 + TASK_OFFSET, 0);
            (r.x, r.c) = self.r_00053e(r.d6, THREE);
            self.write8(r.a5 + SPRITE_FRAME, THREE_FRAME);
        } else {
            let (count, borrow) = self.read8(r.a6 + TASK_COUNT).overflowing_sub(1);
            self.write8(r.a6 + TASK_COUNT, count);
            r.x = borrow;
            if count == 0 {
                self.write8(r.a6 + TASK_COUNT, 3);
                let (step, carry) = self.read8(r.a6 + TASK_STEP).overflowing_add(1);
                self.write8(r.a6 + TASK_STEP, step);
                r.x = carry;
                let index = step & 3;
                let frame = self.read8(FALLING_FRAMES + u32::from(index));
                r.d0 = (r.d0 & 0xFFFF_0000) | u32::from(frame);
                r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
                r.a5 = self.read32(SLOT_ADDR);
                self.write8(r.a5 + SPRITE_FRAME, frame);
            }
        }
        (r.d0, r.d1, r.x, r.c) = self.r_00e1c6(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        r.unwound = false;
    }

    /// `loc_00E1C6`: the task's sprite placed with its offset
    /// (`func_001B94`), its depth (`func_001B74`), then the other two
    /// (`func_00E1D6`).  Gives back d0, d1, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00e1c6(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, bool, bool) {
        let (d0, _, _) = self.r_001b94(a5, a6);
        self.r_001b74(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5, a6);
        self.r_00e1d6(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a6)
    }

    /// `loc_00E1D6`: the task's other two sprites (`TASK_18`,
    /// `TASK_19`) drawn at y offsets `OFFSET_2` and `OFFSET_3`
    /// (`func_00E208`), its own offset kept.  d5 and a5 are kept; gives
    /// back d0, d1, X and C (clear).
    #[allow(clippy::too_many_arguments)]
    pub fn r_00e1d6(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a6: u32) -> (u32, u32, bool, bool) {
        let kept = self.read8(a6 + TASK_OFFSET);
        let mut d0 = 0;
        let mut d1 = d1;
        let mut d5 = d5;
        let mut x = false;
        for (field, offset) in [(TASK_18, OFFSET_2), (TASK_19, OFFSET_3)] {
            let a5;
            let index = self.read8(a6 + field);
            (d5, a5, _) = self.r_000b5a(low_byte(d5, index));
            d1 = low_byte(d1, offset);
            (d0, x) = self.r_00e208(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5, a6);
        }
        self.write8(a6 + TASK_OFFSET, kept);
        (d0, d1, x, false)
    }

    /// `func_00E208`: the sprite record d5 (at a5) is the one being run;
    /// the task's offset's high byte d1, the sprite placed with it
    /// (`func_001B94`) and its depth from its y (`func_001B5C`).  Gives
    /// back d0 and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00e208(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, bool) {
        self.write8(SLOT_INDEX, d5 as u8);
        self.write32(SLOT_ADDR, a5);
        self.write8(a6 + TASK_OFFSET, d1 as u8);
        let (d0, _, _) = self.r_001b94(a5, a6);
        let (d0, x, _) = self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5);
        (d0, x)
    }

    /// `func_00E21C`: the sprite at a5 shown with frame d0.
    pub fn r_00e21c(&mut self, d0: u32, a5: u32) {
        self.write8(a5 + SPRITE_FRAME, d0 as u8);
        self.write8(a5 + SPRITE_FLAGS, FLAGS_SHOWN);
    }

    /// `func_00E228`: two tasks: `FLASHER` (in the middle of its cell,
    /// bit 2 of its y's low byte set, `TASK_18` clear) and `WAITER` (in
    /// the middle of its cell, its count `MADE` once one more, its
    /// `TASK_18` the first task's index).  Gives back d0 (the first
    /// task's index, the low byte 0), d5, d6, a5, a6, X, C clear.
    pub fn r_00e228(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.write8(NEW_FRAME, FLASHER_FRAME);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 0x80, 0x42, FLASHER);
        (r.d0, r.c) = self.r_00ca06(r.d0, r.a5, r.a6);
        self.r_00187c(r.a6);
        let low = self.read8(r.a6 + TASK_Y_LOW);
        self.write8(r.a6 + TASK_Y_LOW, low | 4);
        self.write8(r.a6 + TASK_18, 0);
        let first = r.d6;
        self.write8(NEW_FRAME, WAITER_FRAME);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 0x80, 0x43, WAITER);
        (r.d0, r.c) = self.r_00ca06(r.d0, r.a5, r.a6);
        self.r_00187c(r.a6);
        self.write8(r.a6 + TASK_17, 0);
        let (made, carry) = self.read8(MADE).overflowing_add(1);
        self.write8(MADE, made);
        r.x = carry;
        self.write8(r.a6 + TASK_COUNT, made);
        self.write8(r.a6 + TASK_18, first as u8);
        r.d0 = low_byte(first, 0);
        r.c = false;
        r.unwound = false;
    }

    /// `func_00E2B4`, `FLASHER_ON`: while `TASK_18` is set, frame
    /// `FLASH_FRAME` for 0x80 frames (`FLASHER_FLASH` from the next
    /// frame); the sprite drawn (`func_00E2F2`).
    pub fn r_00e2b4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.c = false;
        if self.read8(r.a6 + TASK_18) == 0 {
            (r.d0, r.x, r.c) = self.r_00e2f2(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
            r.unwound = false;
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x80);
        self.write8(r.a5 + SPRITE_FRAME, FLASH_FRAME);
        (r.x, r.c) = self.r_00053e(r.d6, FLASHER_FLASH);
        self.r_00e2d4(r);
    }

    /// `func_00E2D4`, `FLASHER_FLASH`: when the count runs out,
    /// `TASK_18` cleared, frame `FLASHER_FRAME`, `FLASHER_ON` from the
    /// next frame; the sprite drawn (`func_00E2F2`).
    pub fn r_00e2d4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let (count, borrow) = self.read8(r.a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = borrow;
        if count == 0 {
            self.write8(r.a6 + TASK_18, 0);
            self.write8(r.a5 + SPRITE_FRAME, FLASHER_FRAME);
            self.r_00053e(r.d6, FLASHER_ON);
        }
        (r.d0, r.x, r.c) = self.r_00e2f2(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        r.unwound = false;
    }

    /// `func_00E2F2`: the task's sprite placed (`func_001B7E`) and its
    /// depth from its y (`func_001B5C`).  Gives back d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00e2f2(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, bool, bool) {
        let (d0, _, _) = self.r_001b7e(a5, a6);
        self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5)
    }

    /// `loc_00E30A`, where `func_00E29E` goes on after its yield: it
    /// returns.
    /// No script calls it: reviewed by reading.
    pub fn r_00e30a(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `func_00E30C`, `WAITER`: with bit 0 of `SETTINGS` clear it waits
    /// as `WAITER_COUNTING` from now on.  With it set, its count (a
    /// word) runs down one a frame (nine while `FAST_BELOW` is below 2)
    /// and once it is out (not above 0) the task rises
    /// (`loc_00E358`); the sprite is drawn (`loc_00E378`).
    pub fn r_00e30c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(SETTINGS) & 1 == 0 {
            (r.x, r.c) = self.r_00053e(r.d6, WAITER_COUNTING);
            self.r_00e322(r);
            return;
        }
        let count = self.read16(r.a6 + TASK_COUNT);
        let mut count2 = count.wrapping_sub(1);
        r.x = count < 1;
        if self.read8(FAST_BELOW) < 2 {
            r.x = count2 < 8;
            count2 = count2.wrapping_sub(8);
        }
        self.write16(r.a6 + TASK_COUNT, count2);
        (r.d0, r.x, r.c) = if (count2 as i16) <= 0 {
            self.r_00e358(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6)
        } else {
            self.r_00e378(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6)
        };
    }

    /// `func_00E322`, `WAITER_COUNTING`: its count (a word) one less;
    /// when its high byte is 0: with `TASK_17` 0 it rises
    /// (`loc_00E358`), with `TASK_17` below 0x20 the task `TASK_18`'s
    /// byte 0x18 is set to 1.  The sprite is drawn (`loc_00E378`).
    pub fn r_00e322(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read16(r.a6 + TASK_COUNT);
        self.write16(r.a6 + TASK_COUNT, count.wrapping_sub(1));
        r.x = count < 1;
        if self.read8(r.a6 + TASK_COUNT) == 0 {
            let flag = self.read8(r.a6 + TASK_17);
            r.d0 = low_byte(r.d0, flag);
            if flag == 0 {
                (r.d0, r.x, r.c) = self.r_00e358(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
                return;
            }
            if flag < 0x20 {
                let index = self.read8(r.a6 + TASK_18);
                let (_, other) = self.r_00060a(low_byte(r.d5, index));
                self.write8(other + TASK_18, 1);
                r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
                r.a5 = self.read32(SLOT_ADDR);
            }
        }
        (r.d0, r.x, r.c) = self.r_00e378(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
    }

    /// `loc_00E358`: the task rises from the next frame (`RISING`, its
    /// rate 0, its count 0x80), sound 0x58 (`func_0013C0`); the sprite
    /// drawn (`loc_00E378`).  Gives back d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00e358(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, bool, bool) {
        self.write16(a6 + TASK_RATE, 0);
        self.write8(a6 + TASK_COUNT, 0x80);
        self.r_00053e(d6, RISING);
        self.r_0013c0(0x58, a5);
        self.r_00e378(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5, a6)
    }

    /// `loc_00E378`: the task's sprite placed (`func_001B7E`) and its
    /// depth from its y (`func_001B5C`).  Gives back d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00e378(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, bool, bool) {
        let (d0, _, _) = self.r_001b7e(a5, a6);
        self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5)
    }

    /// `func_00E39C`, `RISING`: for 0x80 frames the task moves up ever
    /// faster (its rate 0x20 less each frame, `func_001AA2`), puffs
    /// made on the way (`func_00E4CA`).  Then: `func_00CED8`, the task
    /// in the middle of its cell (`func_001550`), sound 0x4C, its life
    /// one less, its count 0x40, `STOPPING` from the next frame; and a
    /// task `LEFT_BEHIND` made (frame 0) in the middle of the cell,
    /// count 0x40.
    pub fn r_00e39c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(r.a6 + TASK_COUNT).wrapping_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        if count != 0 {
            (r.d0, r.d5, r.d6, r.a5, r.a6) = self.r_00e4ca(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            let rate = self.read16(r.a6 + TASK_RATE).wrapping_sub(0x20);
            r.d0 = (r.d0 & 0xFFFF_0000) | u32::from(rate);
            self.write16(r.a6 + TASK_RATE, rate);
            self.write16(VELOCITY_Y, rate);
            (r.d0, r.d1, _, _, _) = self.r_001aa2(r.d0, r.d1, r.a6);
            (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
            return;
        }
        (r.d0, r.d1, r.a0) = self.r_00ced8(r.d1);
        (r.d0, r.x) = self.r_001550(r.d0, r.a6);
        (r.d0, _, _) = self.r_001b7e(r.a5, r.a6);
        self.r_0013c0(low_byte(r.d0, 0x4C), r.a5);
        let life = self.read8(r.a6 + TASK_LIFE);
        self.write8(r.a6 + TASK_LIFE, life.wrapping_sub(1));
        self.write8(r.a6 + TASK_COUNT, 0x40);
        let (d0, _, _) = self.r_001b7e(r.a5, r.a6);
        (r.d0, _, _) = self.r_001b5c(d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        self.r_00053e(r.d6, STOPPING);
        self.write8(NEW_FRAME, 0);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 0xFE, 0xC3, LEFT_BEHIND);
        (r.d0, r.c) = self.r_00ca06(r.d0, r.a5, r.a6);
        (r.d0, r.x) = self.r_001550(r.d0, r.a6);
        self.write8(r.a6 + TASK_COUNT, 0x40);
        r.c = false;
    }

    /// `func_00E420`, `STOPPING`: frame `STOP_FRAME`; for 0x40 frames
    /// it moves at 4 pixels a frame down (`func_001AA2`), puffs made
    /// (`func_00E4CA`); then `func_00CF42`, animation `STOPPED_ANIM`,
    /// `ANIMATING` from the next frame, sound 0x4D.
    pub fn r_00e420(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, STOP_FRAME);
        let count = self.read8(r.a6 + TASK_COUNT).wrapping_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        if count != 0 {
            (r.d0, r.d5, r.d6, r.a5, r.a6) = self.r_00e4ca(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            self.write16(VELOCITY_Y, 0x400);
            (r.d0, r.d1, _, _, _) = self.r_001aa2(r.d0, r.d1, r.a6);
            (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
            return;
        }
        (r.d0, r.d2, r.a0) = self.r_00cf42(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        self.write32(r.a6 + TASK_OFFSET, STOPPED_ANIM);
        (r.x, _) = self.r_00053e(r.d6, ANIMATING);
        r.d0 = low_byte(r.d0, 0x4D);
        r.c = self.r_0013c0(r.d0, r.a5);
    }

    /// `func_00E466`, `ANIMATING`: the animation a frame on
    /// (`func_0013D6`); once it is over, the count 8, the task
    /// `TASK_18`'s position (`func_00153A`) taken by this one
    /// (`func_00152A`), its y's low byte 0xF more, `BLINKING` from now
    /// on.  Else the sprite is drawn (`loc_00E4C2`).
    pub fn r_00e466(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let over;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, over) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        r.c = over;
        if !over {
            (r.d0, r.x, r.c) = self.r_00e4c2(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 8);
        let other = self.read8(r.a6 + TASK_18);
        (r.d6, r.a6, _) = self.r_0005f8(low_byte(r.d6, other));
        self.r_00153a(r.a6);
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, _) = self.r_0005f8(low_byte(r.d6, current));
        self.r_00152a(r.a6);
        let low = self.read8(r.a6 + TASK_Y_LOW);
        self.write8(r.a6 + TASK_Y_LOW, low | 0x0F);
        (r.x, r.c) = self.r_00053e(r.d6, BLINKING);
        self.r_00e49e(r);
    }

    /// `func_00E49E`, `BLINKING`: frame `BLINK_FRAME`, its y's low byte
    /// one less a frame for 8 frames; then the count 0x600 (a word) and
    /// `WAITER` again from the next frame.  The sprite is drawn
    /// (`loc_00E4C2`).
    pub fn r_00e49e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, BLINK_FRAME);
        let low = self.read8(r.a6 + TASK_Y_LOW);
        self.write8(r.a6 + TASK_Y_LOW, low.wrapping_sub(1));
        let (count, borrow) = self.read8(r.a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = borrow;
        if count == 0 {
            self.write16(r.a6 + TASK_COUNT, 0x600);
            self.r_00053e(r.d6, WAITER);
        }
        (r.d0, r.x, r.c) = self.r_00e4c2(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
    }

    /// `loc_00E4C2`: the task's sprite placed (`func_001B7E`) and its
    /// depth from its y (`func_001B5C`).  Gives back d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00e4c2(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, bool, bool) {
        let (d0, _, _) = self.r_001b7e(a5, a6);
        self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5)
    }

    /// `func_00E4CA`: every 8th frame of the count (its low 3 bits all
    /// set) a puff is made where the task is: a task `PUFF` (frame 0,
    /// sprite flags 0x44) with a random rate (4-11, `func_001BC0`) in
    /// its x's fraction's low nibble, animation `PUFF_ANIM`, its depth
    /// from byte 2 of the task's sprite record (`func_0007D2`).  The running task and
    /// its sprite again after (`func_0005F8`, `func_000B5A`).  Gives
    /// back d0, d5, d6, a5, a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00e4ca(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32) {
        let count = self.read8(a6 + TASK_COUNT) & 7;
        let mut d0 = low_byte(d0, count);
        let (mut d5, mut d6) = (d5, d6);
        if count == 7 {
            let byte2 = self.read8(a5 + SPRITE_2);
            self.write8(KEPT_DEPTH, byte2);
            self.r_00153a(a6);
            self.write8(NEW_FRAME, 0);
            let (sprite, task, failed);
            (d5, d6, sprite, task, _, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0x80, 0xC3, PUFF);
            if !failed {
                self.r_00ca06(d0, sprite, task);
                self.write8(sprite + SPRITE_FLAGS, 0x44);
                let random = self.read8(task + TASK_RANDOM);
                self.write8(task + TASK_RANDOM, random & 0xF0);
                d0 = self.r_001bc0(7, d1);
                let (rate, x) = (d0 as u8).overflowing_add(4);
                d0 = low_byte(d0, rate);
                let random = self.read8(task + TASK_RANDOM);
                self.write8(task + TASK_RANDOM, random | rate);
                self.write32(task + TASK_OFFSET, PUFF_ANIM);
                self.write8(SLOT_INDEX, d5 as u8);
                self.write32(SLOT_ADDR, sprite);
                d0 = low_byte(d0, self.read8(KEPT_DEPTH));
                self.r_0007d2(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, x);
            }
        }
        let current = self.read8(CURRENT_TASK);
        let (d6, a6, _) = self.r_0005f8(low_byte(d6, current));
        let sprite = self.read8(a6 + TASK_SPRITE);
        let (d5, a5, _) = self.r_000b5a(low_byte(d5, sprite));
        (d0, d5, d6, a5, a6)
    }

    /// `func_00E548`, `PUFF`'s routine: its animation a frame on
    /// (`func_0013D6`); the task ends when it is over
    /// (`func_00121A`), else its sprite is placed (`func_001B7E`).
    pub fn r_00e548(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let over;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, over) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if over {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        } else {
            (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        }
    }

    /// `func_00E558`, `LEFT_BEHIND`'s routine: frame `LEFT_FRAME`; the
    /// task ends when its count runs out (`func_00121A`), else its
    /// sprite is placed (`func_001B7E`).
    pub fn r_00e558(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, LEFT_FRAME);
        let (count, borrow) = self.read8(r.a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = borrow;
        if count == 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        } else {
            (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        }
    }

    /// `func_00E56E`, the long after `D_00A2C8`'s three: it returns.
    pub fn r_00e56e(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }
}
