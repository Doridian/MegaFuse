//! Routines of tasks (a6) and objects (a4) with a sprite record (a5):
//! a task's frames stepped through (`func_006074`), an object's count
//! of hits taken on certain cells (`func_006E28`) and the routines it
//! is given after the last (`func_006EE0` on), and two task routines
//! that count down.

use crate::{Game, Regs};

/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// Bit 0: the sprite is placed by its y (`func_00601E`).
const BY_Y: u32 = 0xFFAABC;
/// Flags: bit 5 set by `func_006FF0`.
const FLAGS: u32 = 0xFF9540;
/// The objects given `func_006EE0` (counted up and down).
const FALLING: u32 = 0xFFA956;
/// Set to 0x10 when an object's `func_006EE0` is over.
const LANDED: u32 = 0xFFAC7D;
/// A velocity's y (a word; `a_motion.rs`).
const VELOCITY_Y: u32 = 0xFF9EC6;

/// A sprite record's frame, flags and the low byte of its y.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_Y_LOW: u32 = 0x0B;
const SPRITE_HIDDEN: u8 = 1 << 6;

/// A task's fields: its frame's timer and number, its flags, a count
/// down.
const TASK_TIMER: u32 = 0x12;
const TASK_FRAME: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_FLAGS: u32 = 0x18;
/// The frames each of a task's frames lasts: by bit 3, then bit 4, of
/// its flags, else the default.
const SLOW: u8 = 0x16;
const FAST: u8 = 0x04;
const DEFAULT: u8 = 0x0E;
/// A task's frames' numbers (D_0060C8) and sprite frames.
const FRAMES: u32 = 0x0060C8;

/// An object's fields: its kind; routine and a kept one (longs); a
/// byte of two nibbles (`func_006FA0`); flags; its hits left and a
/// timer; direction and kind; a count down; y (a word); its
/// animation's step.
const OBJ_KIND_BYTE: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_NIBBLES: u32 = 0x06;
const OBJ_KEPT: u32 = 0x06;
const OBJ_FLAGS: u32 = 0x10;
const OBJ_HITS: u32 = 0x11;
const OBJ_TIMER: u32 = 0x12;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x15;
const OBJ_COUNT: u32 = 0x16;
const OBJ_Y: u32 = 0x1A;
const OBJ_ANIM_TIMER: u32 = 0x24;
const OBJ_ANIM_STEP: u32 = 0x25;
/// The frames an object is hidden every other two of after a hit.
const HIT_FRAMES: u8 = 0x40;

/// The routines the objects are given.
const FALLING_ROUTINE: u32 = 0x006EE0;
const RISING_ROUTINE: u32 = 0x006F42;
const FADING_ROUTINE: u32 = 0x006F66;
const DONE_ROUTINE: u32 = 0x006F9E;
/// The routine `func_006FC6` gives its task.
const LAST_ROUTINE: u32 = 0x006FF0;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_00601E`: the task's sprite (the slot's): bit 6 of the
    /// task's flags set while the sprite's frame isn't 0; moved (`func_001B7E`),
    /// its frame the next of `FRAMES` (`func_006074`), and placed at
    /// depth 0xF8, or by its y when `BY_Y` says so (`func_0007D2`).
    /// Gives back d0, d5, a0, a5, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00601e(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a6: u32) -> (u32, u32, u32, u32, bool, bool) {
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        if self.read8(a5 + SPRITE_FRAME) != 0 {
            let value = self.read8(a6 + TASK_FLAGS) | 0x40;
            self.write8(a6 + TASK_FLAGS, value);
        }
        let (d0, _, _) = self.r_001b7e(a5, a6);
        let (d0, mut x) = self.r_006074(d0, a6);
        let frame = d0 as u8 as i8 as i16;
        let d0 = low_word(d0, frame as u16);
        let value = self.read8(FRAMES.wrapping_add(i32::from(frame) as u32));
        self.write8(a5 + SPRITE_FRAME, value);
        let mut depth = 0xF8;
        if self.read8(BY_Y) & 1 != 0 {
            let (less, borrow) = self.read8(a5 + SPRITE_Y_LOW).overflowing_sub(0x0E);
            depth = (!less & 0xFE) | 2;
            x = borrow;
        }
        let d0 = low_byte(d0, depth);
        let c = self.r_0007d2(d0, d1, d2, d3, d4, d5, d6, d7, FRAMES, a1, a2, a3, x);
        (d0, d5, FRAMES, a5, x, c)
    }

    /// `func_006072`: nothing.
    pub fn r_006072(&mut self) {}

    /// `func_006074`: the task's frame number (0-3, d0): the next once
    /// its timer is out (the timer then `SLOW`, `FAST` or `DEFAULT` by
    /// its flags).  Gives back d0 and X.
    pub fn r_006074(&mut self, d0: u32, a6: u32) -> (u32, bool) {
        let timer = self.read8(a6 + TASK_TIMER);
        if timer != 0 {
            self.write8(a6 + TASK_TIMER, timer - 1);
            return (low_byte(d0, self.read8(a6 + TASK_FRAME)), false);
        }
        let flags = self.read8(a6 + TASK_FLAGS);
        let timer = if flags & (1 << 3) != 0 {
            SLOW
        } else if flags & (1 << 4) != 0 {
            FAST
        } else {
            DEFAULT
        };
        self.write8(a6 + TASK_TIMER, timer);
        let (next, x) = self.read8(a6 + TASK_FRAME).overflowing_add(1);
        let frame = next & 3;
        self.write8(a6 + TASK_FRAME, frame);
        (low_byte(d0, frame), x)
    }

    /// `func_006E10`: `func_006E82`.
    pub fn r_006e10(&mut self, d0: u32, a0: u32, a4: u32, a5: u32) {
        self.r_006e82(d0, a0, a4, a5);
    }

    /// `func_006E28`: unless the object has hits left after this one
    /// (`func_006E7C`), it is set up to fall (`FALLING_ROUTINE`, its
    /// routine kept; `FALLING` one more) and C is set.  Gives back d0,
    /// d1, d2, a0, X and C.
    pub fn r_006e28(&mut self, d0: u32, d1: u32, d2: u32, a0: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        let (mut d0, mut d1, mut d2) = (low_byte(d0, self.read8(a4 + OBJ_HITS)), d1, d2);
        if d0 as u8 != 0 {
            let (e0, e1, e2, e3, x, last) = self.r_006e7c(d1, d2, a0, a4, a5);
            (d0, d1, d2) = (e0, e1, e2);
            if !last {
                let hits = self.read8(a4 + OBJ_HITS);
                d0 = low_byte(d0, hits);
                if hits != 0 {
                    return (d0, d1, d2, e3, x, false);
                }
            }
        }
        self.write8(a4 + OBJ_DIRECTION, 0);
        self.write8(a4 + OBJ_KIND, 4);
        let (a0, _, _) = self.r_00130e(d0, a4);
        self.write8(a4 + OBJ_ANIM_TIMER, 0);
        self.write8(a4 + OBJ_ANIM_STEP, 0);
        self.write8(a4 + OBJ_TIMER, 1);
        let value = self.read32(a4 + OBJ_ROUTINE);
        self.write32(a4 + OBJ_KEPT, value);
        self.write32(a4 + OBJ_ROUTINE, FALLING_ROUTINE);
        let (falling, x) = self.read8(FALLING).overflowing_add(1);
        self.write8(FALLING, falling);
        (d0, d1, d2, a0, x, true)
    }

    /// `func_006E7C`: the Z80 told 7 for the object (`func_0034DE`),
    /// then `func_006E82`.  Gives back d0, d1, d2, a0, X and C.
    pub fn r_006e7c(&mut self, d1: u32, d2: u32, a0: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        let (d1, d2, _) = self.r_0034de(7, d1, d2, a4);
        let (d0, a0, x, c) = self.r_006e82(7, a0, a4, a5);
        (d0, d1, d2, a0, x, c)
    }

    /// `func_006E82`: the object's sprite shown, or hidden every other
    /// two frames while its timer runs; with the timer out, a cell
    /// holding 3 or 7 or more (`func_00167A`) is a hit: the timer
    /// `HIT_FRAMES` again, a hit less, C set at the last.  Gives back
    /// d0, a0, X and C.
    pub fn r_006e82(&mut self, d0: u32, a0: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let flags = self.read8(a5 + SPRITE_FLAGS);
        self.write8(a5 + SPRITE_FLAGS, flags & !SPRITE_HIDDEN);
        let timer = self.read8(a4 + OBJ_TIMER);
        if timer != 0 {
            let timer = timer - 1;
            self.write8(a4 + OBJ_TIMER, timer);
            if timer & 2 != 0 {
                let value = self.read8(a5 + SPRITE_FLAGS) | SPRITE_HIDDEN;
                self.write8(a5 + SPRITE_FLAGS, value);
            }
            return (d0, a0, false, false);
        }
        let (a0, x) = self.r_00167a();
        let held = self.read8(a0);
        let d0 = low_byte(d0, held);
        if held != 3 && held < 7 {
            return (d0, a0, x, false);
        }
        self.write8(a4 + OBJ_TIMER, HIT_FRAMES);
        let hits = self.read8(a4 + OBJ_HITS);
        self.write8(a4 + OBJ_HITS, hits.wrapping_sub(1));
        (d0, a0, hits == 0, hits == 1)
    }

    /// `func_006EE0`, an object's routine: its sprite shown and moved
    /// (`func_001B32`, `func_001B5C`; kind 0x21 at step 10 gets sprite
    /// flags 0x41), until its animation is over (`func_001336`): then
    /// `LANDED` 0x10 and `func_006F1A`.
    pub fn r_006ee0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let flags = self.read8(r.a5 + SPRITE_FLAGS);
        self.write8(r.a5 + SPRITE_FLAGS, flags & !SPRITE_HIDDEN);
        (r.d0, r.d1, _, _) = self.r_001b32(r.d1, r.a4, r.a5);
        (r.d0, r.x, _) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        let kind = self.read8(r.a4 + OBJ_KIND_BYTE);
        r.d0 = low_byte(r.d0, kind);
        if kind == 0x21 {
            let step = self.read8(r.a4 + OBJ_ANIM_STEP);
            r.d0 = low_byte(r.d0, step);
            if step == 0x0A {
                self.write8(r.a5 + SPRITE_FLAGS, 0x41);
            }
        }
        let over;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, over) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        r.c = over;
        r.unwound = false;
        if over {
            self.write8(LANDED, 0x10);
            (r.d0, r.d1, r.x, r.c) = self.r_006f1a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        }
    }

    /// `func_006F1A`: the object (flags bit 7) given the sprite frame of
    /// `func_00389E`, sprite flags 3, a count of 0x20, y 16 less, and
    /// `RISING_ROUTINE`, run at once.  Gives back d0, d1, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_006f1a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, bool, bool) {
        let value = self.read8(a4 + OBJ_FLAGS) | 0x80;
        self.write8(a4 + OBJ_FLAGS, value);
        let d0 = self.r_00389e(d0, a4);
        self.write8(a5 + SPRITE_FRAME, d0 as u8);
        self.write8(a5 + SPRITE_FLAGS, 3);
        self.write8(a4 + OBJ_COUNT, 0x20);
        let (y, x) = self.read16(a4 + OBJ_Y).overflowing_add(0xFFF0);
        self.write16(a4 + OBJ_Y, y);
        self.write32(a4 + OBJ_ROUTINE, RISING_ROUTINE);
        let mut regs = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c: false, ..Regs::default() };
        self.r_006f42(&mut regs);
        (regs.d0, regs.d1, regs.x, regs.c)
    }

    /// One of the object's count's frames (`OBJ_COUNT` one less): X
    /// and C, and whether it is out.
    fn count_down(&mut self, regs: &mut Regs) -> bool {
        let count = self.read8(regs.a4 + OBJ_COUNT);
        self.write8(regs.a4 + OBJ_COUNT, count.wrapping_sub(1));
        regs.x = count == 0;
        regs.c = regs.x;
        count == 1
    }

    /// `func_006F42`, an object's routine: it moves up (`VELOCITY_Y`
    /// -0.25, `func_0019DA`; `func_001B32`) for its count, then is given
    /// `FADING_ROUTINE` with a count of 0x40, run at once.
    pub fn r_006f42(&mut self, regs: &mut Regs) {
        self.write16(VELOCITY_Y, 0xFFC0);
        let r = &mut *regs;
        (r.d0, r.d1, _) = self.r_0019da(r.d0, r.d1, r.a4);
        (r.d0, r.d1, _, _) = self.r_001b32(r.d1, r.a4, r.a5);
        r.unwound = false;
        if !self.count_down(regs) {
            return;
        }
        self.write8(regs.a4 + OBJ_COUNT, 0x40);
        self.write32(regs.a4 + OBJ_ROUTINE, FADING_ROUTINE);
        regs.c = false;
        self.r_006f66(regs);
    }

    /// `func_006F66`, an object's routine: shown (`func_001B32`) for its
    /// count, then `FALLING` one less and the object ended (its sprite,
    /// `func_000890`, and its kind 0), or with bit 5 of its flags given
    /// sprite frame 0, flags 1 and `DONE_ROUTINE`.
    pub fn r_006f66(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.d0, r.d1, _, _) = self.r_001b32(r.d1, r.a4, r.a5);
        r.unwound = false;
        if !self.count_down(regs) {
            return;
        }
        let (falling, borrow) = self.read8(FALLING).overflowing_sub(1);
        self.write8(FALLING, falling);
        regs.x = borrow;
        regs.c = false;
        if self.read8(regs.a4 + OBJ_FLAGS) & (1 << 5) == 0 {
            self.r_000890();
            self.write8(regs.a4 + OBJ_KIND_BYTE, 0);
        } else {
            self.write8(regs.a5 + SPRITE_FRAME, 0);
            self.write8(regs.a5 + SPRITE_FLAGS, 1);
            self.write32(regs.a4 + OBJ_ROUTINE, DONE_ROUTINE);
        }
    }

    /// `func_006F9E`, an object's routine: nothing.
    pub fn r_006f9e(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `func_006FA0`: the high nibble of the object's `OBJ_NIBBLES` one
    /// less, unless it is 0.
    pub fn r_006fa0(&mut self, a4: u32) {
        let byte = self.read8(a4 + OBJ_NIBBLES);
        let high = byte & 0xF0;
        if high != 0 {
            self.write8(a4 + OBJ_NIBBLES, (byte & 0x0F) | (high - 0x10));
        }
    }

    /// `func_006FC6`, a task's routine: unless the objects are stopped
    /// (`func_001BB4`), its count goes down; at 0 the Z80 is sent 0x46
    /// and the task counts 0x20 more frames in `LAST_ROUTINE`.
    pub fn r_006fc6(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let (c, stop) = self.r_001bb4();
        if stop {
            regs.c = c;
            return;
        }
        let count = self.read8(regs.a6 + TASK_COUNT);
        self.write8(regs.a6 + TASK_COUNT, count.wrapping_sub(1));
        regs.x = count == 0;
        regs.c = regs.x;
        if count != 1 {
            return;
        }
        regs.d0 = low_byte(regs.d0, 0x46);
        self.r_0034b6(regs.d0);
        self.write8(regs.a6 + TASK_COUNT, 0x20);
        (regs.x, regs.c) = self.r_00053e(regs.d6, LAST_ROUTINE);
    }

    /// `func_006FEE`: nothing.
    pub fn r_006fee(&mut self) {}

    /// `func_006FF0`, a task's routine: its count goes down; at 0 bit 5
    /// of `FLAGS` is set and the task ends (`func_000560`).
    pub fn r_006ff0(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let count = self.read8(regs.a6 + TASK_COUNT);
        self.write8(regs.a6 + TASK_COUNT, count.wrapping_sub(1));
        regs.x = count == 0;
        regs.c = regs.x;
        if count != 1 {
            return;
        }
        let value = self.read8(FLAGS) | (1 << 5);
        self.write8(FLAGS, value);
        (regs.d6, regs.x, regs.c) = self.r_000560(regs.d5, regs.d6, regs.x);
    }
}
