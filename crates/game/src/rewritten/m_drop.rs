//! More of `D_00A220`'s cell routines (the stage record `D_00A1F0`):
//! `func_00DE62` (kind 7) makes a task that waits (`func_00DEA2`) until
//! the tasks of kinds 8 and 9 (`func_00DD84`) have both set their bit
//! in its `TASK_BITS`, each when the cell below it holds 4 or 6
//! (`func_00DE06`).  Then it falls (`func_00DEBE`), and objects it
//! meets are hit (`func_00DF26`): objects 0-5 through `func_002EB8`,
//! the others stunned for a while (`func_00DFFE`, framed by
//! `func_00DFC2`).  `func_00DEBE` and the routines it calls are run by
//! no pad script: reviewed by reading.

use crate::{Game, Regs};

/// The task being run (its index, a byte; `func_0005F8`).
const CURRENT_TASK: u32 = 0xFF80EE;
/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// A byte the cell routines set: read by `func_00CA06` for the task's
/// sprite.
const SPRITE_KIND: u32 = 0xFFAC91;
/// The index of `func_00DE62`'s task, for `func_00DD84`'s.
const WAITER: u32 = 0xFFACEA;
/// The cell (column, row; a word together) and the row.
const CELL: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
/// A velocity's y (a word; `a_motion.rs`).
const VELOCITY_Y: u32 = 0xFF9EC6;

/// The tasks' routines, depth, priority and animation.
const PRIORITY: u8 = 0x42;
const DEPTH: u8 = 0x80;
const MARKER_ROUTINE: u32 = 0x00DDC6;
const MARKER_NEXT: u32 = 0x00DDE0;
const MARKER_LOOKING: u32 = 0x00DE06;
const MARKER_DONE: u32 = 0x00D61C;
const MARKER_DONE_ANIM: u32 = 0x0DA5C6;
const WAITER_ROUTINE: u32 = 0x00DEA2;
const FALLING_ROUTINE: u32 = 0x00DEBE;
const STUNNED_ROUTINE: u32 = 0x00DFFE;
/// `TASK_BITS` with both bits.
const BOTH: u8 = 3;

/// A task's fields: its position (words), a count (the frames it
/// falls; `func_00DD84`'s: the waiter's index), bits (`func_00DE62`'s;
/// `func_00DD84`'s: its bit), a rate (a word), a byte, the cell's
/// address and the cell (a word) `func_00DE06` looks at.
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_COUNT: u32 = 0x16;
const TASK_BITS: u32 = 0x18;
const TASK_RATE: u32 = 0x1A;
const TASK_20: u32 = 0x20;
const TASK_ANIM: u32 = 0x20;
const TASK_CELL_ADDR: u32 = 0x24;
const TASK_CELL: u32 = 0x28;

/// An object's fields: its kind (0: none), routine and a kept one, its
/// sprite (address, index), flags (bit 1: stunned; bit 7), a byte
/// (`OBJ_11`, not 0 to be hit), a byte cleared, a count, position
/// (words).
const OBJ_KIND: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_KEPT: u32 = 0x06;
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_FLAGS: u32 = 0x10;
const OBJ_11: u32 = 0x11;
const OBJ_12: u32 = 0x12;
const OBJ_COUNT: u32 = 0x16;
const OBJ_X: u32 = 0x18;
const OBJ_Y: u32 = 0x1A;
const STUNNED: u8 = 1 << 1;
/// The objects hit, from 0 to `HIT_END` (objects below `HIT_OWN`
/// through `func_002EB8`), and how near (y, x).
const HIT_END: u8 = 0x14;
const HIT_OWN: u8 = 6;
const HIT_NEAR_Y: u16 = 0x10;
const HIT_NEAR_X: u16 = 0x16;
/// Sprite frames by kind (`func_00DFC2`: the table after it at 0xDFF6,
/// read from 0xDFE0, written `func_00DFC2+0x1E` in the disassembly:
/// kinds from 0x16), and kind 1's.
const FRAME_BY_KIND: u32 = 0x00DFE0;
const KIND_1_FRAME: u8 = 0x1E;

/// A sprite record's frame and flags.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// A word's distance (`sub.w`, `neg.w` when negative).
fn distance(a: u16, b: u16) -> u16 {
    let d = a.wrapping_sub(b);
    if (d as i16) < 0 { d.wrapping_neg() } else { d }
}

impl Game {
    /// A cell routine's task (`func_001222`, depth `DEPTH`) made and
    /// placed (`func_00CA06`, `func_00187C`) with `SPRITE_KIND` `kind`.
    fn m_make_dropper(&mut self, regs: &mut Regs, kind: u8, routine: u32) {
        self.write8(SPRITE_KIND, kind);
        let r = &mut *regs;
        (r.d5, r.d6, r.a5, r.a6, r.x, _) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, DEPTH, PRIORITY, routine);
        (r.d0, _) = self.r_00ca06(r.d0, r.a5, r.a6);
        self.r_00187c(r.a6);
        r.unwound = false;
    }

    /// `func_00DD84`, a cell routine (kinds 8, 9: the bit the kind less
    /// 7): `SPRITE_KIND` 0x97, a task running `func_00DDC6` with its
    /// bit and the waiter's index; command 0, C clear.
    pub fn r_00dd84(&mut self, regs: &mut Regs) {
        let bit = (regs.d0 as u8).wrapping_sub(7);
        let kept = low_byte(regs.d0, bit);
        regs.d0 = kept;
        self.m_make_dropper(regs, 0x97, MARKER_ROUTINE);
        self.write8(regs.a6 + TASK_BITS, bit);
        let waiter = self.read8(WAITER);
        self.write8(regs.a6 + TASK_COUNT, waiter);
        regs.d0 = low_byte(kept, 0);
        regs.c = false;
    }

    /// `func_00DDC6`, `func_00DD84`'s task: placed once, on as
    /// `func_00DDE0` (unless the objects are stopped, `func_001BB4`).
    pub fn r_00ddc6(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        (r.d0, _, _) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.x, _) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, MARKER_NEXT);
    }

    /// `func_00DDE0` (unless stopped): the cell below the task's kept
    /// (`TASK_CELL`, `TASK_CELL_ADDR`), and on as `func_00DE06`.
    pub fn r_00dde0(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let stopped;
        (regs.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        self.r_00053e(regs.d6, MARKER_LOOKING);
        let a6 = regs.a6;
        regs.d0 = self.r_0015f8(a6);
        let row = self.read8(CELL_Y).wrapping_add(1);
        self.write8(CELL_Y, row);
        let cell = self.read16(CELL);
        self.write16(a6 + TASK_CELL, cell);
        (regs.a0, regs.x) = self.r_0016ac();
        self.write32(a6 + TASK_CELL_ADDR, regs.a0);
        regs.c = false;
        self.r_00de06(regs);
    }

    /// `func_00DE06` (unless stopped): when the kept cell holds 4 or 6
    /// (bit 7 aside), the waiter (`func_00060A`) gets the task's bit,
    /// and the task runs `MARKER_DONE_ANIM` to its end (`func_00D61C`).
    /// Placed.
    pub fn r_00de06(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let stopped;
        (regs.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        let a6 = regs.a6;
        let cell = self.read16(a6 + TASK_CELL);
        self.write16(CELL, cell);
        regs.a0 = self.read32(a6 + TASK_CELL_ADDR);
        let value = self.read8(regs.a0);
        regs.d0 = low_byte(regs.d0, value);
        if value != 0 {
            let value = value & 0x7F;
            regs.d0 = low_byte(regs.d0, value);
            if value == 4 || value == 6 {
                let waiter = self.read8(a6 + TASK_COUNT);
                let (d5, a5) = self.r_00060a(low_byte(regs.d5, waiter));
                let bit = self.read8(a6 + TASK_BITS);
                regs.d0 = low_byte(regs.d0, bit);
                let bits = self.read8(a5 + TASK_BITS) | bit;
                self.write8(a5 + TASK_BITS, bits);
                regs.d5 = low_byte(d5, self.read8(SLOT_INDEX));
                regs.a5 = self.read32(SLOT_ADDR);
                self.write32(a6 + TASK_ANIM, MARKER_DONE_ANIM);
                self.r_00053e(regs.d6, MARKER_DONE);
            }
        }
        let r = &mut *regs;
        (r.d0, _, _) = self.r_001b7e(r.a5, a6);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_00DE62`, a cell routine (kind 7): `SPRITE_KIND` 0x95, a
    /// task running `func_00DEA2` (bits 0, rate -0x200), its index in
    /// `WAITER`; command 6, C clear.
    pub fn r_00de62(&mut self, regs: &mut Regs) {
        self.m_make_dropper(regs, 0x95, WAITER_ROUTINE);
        let a6 = regs.a6;
        self.write8(a6 + TASK_BITS, 0);
        self.write16(a6 + TASK_RATE, 0xFE00);
        self.write8(a6 + TASK_20, 0);
        self.write8(WAITER, regs.d6 as u8);
        regs.d0 = low_byte(regs.d0, 6);
        regs.c = false;
    }

    /// `func_00DEA2`, `func_00DE62`'s task: drawn (`func_00DF1E`) until
    /// both bits are set; then it falls for 0x80 frames (`func_00DEBE`).
    pub fn r_00dea2(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        if self.read8(regs.a6 + TASK_BITS) != BOTH {
            let r = &mut *regs;
            (r.d0, r.x, r.c) = self.r_00df1e(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
            return;
        }
        self.write8(regs.a6 + TASK_COUNT, 0x80);
        (regs.x, regs.c) = self.r_00053e(regs.d6, FALLING_ROUTINE);
        self.r_00debe(regs);
    }

    /// `func_00DEBE`: for its count, framed 0x95 or 0x96 (by bit 2 of
    /// the count), bouncing (`func_00C742` with 0x20; down again, not
    /// N from `func_001ACA`: rate -0x200, the sprite's height 0x56), moved
    /// down (`VELOCITY_Y` 0x200, `func_001AA2`), hitting the objects
    /// near (`func_00DF26`), and drawn; then it ends.
    pub fn r_00debe(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let a6 = regs.a6;
        let (count, borrow) = self.read8(a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(a6 + TASK_COUNT, count);
        regs.x = borrow;
        regs.c = borrow;
        if count == 0 {
            (regs.d6, regs.x, regs.c) = self.r_00121a(regs.d5, regs.d6, regs.x);
            return;
        }
        let frame = ((count >> 2) & 1).wrapping_add(0x95);
        self.write8(regs.a5 + SPRITE_FRAME, frame);
        regs.d0 = 0x20;
        self.r_00c742(regs.d0, a6);
        let up;
        (regs.d0, _, up) = self.r_001aca(regs.d0, a6);
        if !up {
            self.write16(a6 + TASK_RATE, 0xFE00);
            self.write8(a6 + TASK_20, 0);
            regs.d0 = low_byte(regs.d0, 0x56);
            self.r_0013c0(regs.d0, regs.a5);
        }
        self.write16(VELOCITY_Y, 0x200);
        let r = &mut *regs;
        (r.d0, r.d1, _, _, _) = self.r_001aa2(r.d0, r.d1, a6);
        (r.d2, r.d4, r.a4) = self.r_00df26(r.d0, r.d2, r.d4, r.d5, a6);
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, _) = self.r_0005f8(r.d6);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.x, r.c) = self.r_00df1e(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
    }

    /// `func_00DEA2`'s end (0xDF1E): drawn (`func_001B94`,
    /// `func_001B74`).  Gives back d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00df1e(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, bool, bool) {
        let (d0, _, _) = self.r_001b94(a5, a6);
        self.r_001b74(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5, a6)
    }

    /// `func_00DF26`: each object in use with `OBJ_11` set and not bit
    /// 7 of its flags, within `HIT_NEAR_Y` and `HIT_NEAR_X` of the task,
    /// is hit and the Z80 sent 0x39: below `HIT_OWN` through
    /// `func_002EB8`; others not yet stunned get `STUNNED`, the routine
    /// `func_00DFFE` (theirs kept), count 0xFF, framed (`func_00DFC2`).
    /// Gives back d2, d4 and a4 (the last object).
    pub fn r_00df26(&mut self, d0: u32, d2: u32, d4: u32, d5: u32, a6: u32) -> (u32, u32, u32) {
        let mut d0 = d0;
        let mut d2 = d2;
        let mut d5 = d5;
        let mut d4 = low_word(d4, 0);
        loop {
            let a4;
            (d4, a4, _) = self.r_001cfa(d4);
            if self.read8(a4 + OBJ_KIND) != 0
                && self.read8(a4 + OBJ_11) != 0
                && (self.read8(a4 + OBJ_FLAGS) as i8) >= 0
            {
                let dy = distance(self.read16(a4 + OBJ_Y), self.read16(a6 + TASK_Y));
                d2 = low_word(d2, dy);
                if dy < HIT_NEAR_Y {
                    let dx = distance(self.read16(a4 + OBJ_X), self.read16(a6 + TASK_X));
                    d2 = low_word(d2, dx);
                    if dx < HIT_NEAR_X {
                        d0 = low_byte(d0, d4 as u8);
                        if (d4 as u8) < HIT_OWN {
                            self.r_002eb8(d0, d4);
                            d0 = low_byte(d0, 0x39);
                            self.r_0034b6(d0);
                        } else if self.read8(a4 + OBJ_FLAGS) & STUNNED == 0 {
                            let flags = self.read8(a4 + OBJ_FLAGS) | STUNNED;
                            self.write8(a4 + OBJ_FLAGS, flags);
                            d0 = low_byte(d0, 0x39);
                            self.r_0034b6(d0);
                            let routine = self.read32(a4 + OBJ_ROUTINE);
                            self.write32(a4 + OBJ_KEPT, routine);
                            self.write32(a4 + OBJ_ROUTINE, STUNNED_ROUTINE);
                            (d0, d5, _) = self.r_00dfc2(d0, d5, a4);
                            self.write8(a4 + OBJ_12, 0);
                            self.write8(a4 + OBJ_COUNT, 0xFF);
                        }
                    }
                }
            }
            d4 = low_word(d4, (d4 as u16).wrapping_add(1));
            if d4 as u8 == HIT_END {
                return (d2, d4, a4);
            }
        }
    }

    /// `func_00DFC2`: the object's sprite framed by its kind
    /// (`FRAME_BY_KIND`; kind 1: `KIND_1_FRAME`, flags 1).  Gives back
    /// d0, d5 and a5 (the sprite).
    pub fn r_00dfc2(&mut self, d0: u32, d5: u32, a4: u32) -> (u32, u32, u32) {
        let d5 = low_byte(d5, self.read8(a4 + OBJ_SPRITE));
        let a5 = self.read32(a4 + OBJ_SPRITE_ADDR);
        let kind = self.read8(a4 + OBJ_KIND);
        let d0 = low_byte(d0, kind);
        if kind == 1 {
            self.write8(a5 + SPRITE_FLAGS, 1);
            self.write8(a5 + SPRITE_FRAME, KIND_1_FRAME);
            return (d0, d5, a5);
        }
        let at = kind as i8 as i32 as u32;
        let d0 = low_word(d0, at as u16);
        let frame = self.read8(FRAME_BY_KIND.wrapping_add(at));
        self.write8(a5 + SPRITE_FRAME, frame);
        (d0, d5, a5)
    }

    /// `func_00DFFE`, a stunned object's routine: framed (`func_00DFC2`;
    /// blinking, frame 0 on bit 1 of its count, below 0x3C), until its
    /// count runs out or it is on a cell holding 3 or 7 and more (kind
    /// 1: when `func_001486` says so, C); then its routine back, not
    /// stunned, the kept routine's high word cleared.  Placed
    /// (`func_001B32`) at depth 0xFE (`func_0007D2`).
    pub fn r_00dffe(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let a4 = regs.a4;
        let (d0, x) = self.r_0015e6(a4);
        let (d0, d5, a5) = self.r_00dfc2(d0, regs.d5, a4);
        regs.d5 = d5;
        regs.a5 = a5;
        let count = self.read8(a4 + OBJ_COUNT);
        let mut d0 = low_byte(d0, count);
        if count < 0x3C {
            d0 = low_byte(d0, count & 2);
            if count & 2 != 0 {
                self.write8(a5 + SPRITE_FRAME, 0);
            }
        }
        let ends = if self.read8(a4 + OBJ_KIND) != 1 {
            (regs.a0, _) = self.r_00167a();
            let value = self.read8(regs.a0);
            value == 3 || value >= 7
        } else {
            self.r_001486(d0, regs.d1, regs.a0, x).1
        };
        let ends = ends || {
            let count = self.read8(a4 + OBJ_COUNT).wrapping_sub(1);
            self.write8(a4 + OBJ_COUNT, count);
            count == 0
        };
        if ends {
            let kept = self.read32(a4 + OBJ_KEPT);
            self.write32(a4 + OBJ_ROUTINE, kept);
            let flags = self.read8(a4 + OBJ_FLAGS) & !STUNNED;
            self.write8(a4 + OBJ_FLAGS, flags);
            self.write8(a4 + OBJ_KEPT, 0);
            self.write8(a4 + OBJ_KEPT + 1, 0);
        }
        let x;
        (d0, regs.d1, x, _) = self.r_001b32(regs.d1, a4, a5);
        regs.d0 = low_byte(d0, 0xFE);
        regs.x = x;
        let r = &mut *regs;
        r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, x);
    }
}
