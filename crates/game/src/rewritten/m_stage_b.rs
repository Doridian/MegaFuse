//! The routines of the stage record `D_00A1F0`: its routine
//! `func_00D906` (nothing to do) and some of its routines by cell kind
//! (`D_00A220`: `func_00D95E`, `func_00D910`, `func_00D91A`; each
//! leaves in d0 the Z80's command), with the tasks they make
//! (`func_00D990`, `func_00D9AA`, `func_00D9CA`).  The others are in
//! `m_corners.rs` and `m_drop.rs`.

use crate::{Game, Regs};

/// Flags: bit 5 starts `func_00D9AA`'s task.
const FLAGS: u32 = 0xFF9540;
/// A byte the cell routines set: read by `func_00CA06` for the task's
/// sprite.
const SPRITE_KIND: u32 = 0xFFAC91;
/// A pointer to a byte `func_00D91A` picks the sprite by (below 0x40,
/// `SPRITE_KIND` 0x9C).
const POINTER_9EDA: u32 = 0xFF9EDA;
/// The cell's row.
const CELL_Y: u32 = 0xFF9F09;

/// The tasks' routines, depths, priority and animation.
const PRIORITY: u8 = 0x42;
const ROUTINE_CAE8: u32 = 0x00CAE8;
const DEPTH_3: u8 = 3;
const DEPTHS_ROUTINE: u32 = 0x00D990;
const OPENER_ROUTINE: u32 = 0x00D9AA;
const OPENER_DEPTH: u8 = 0xF0;
const OPENER_ANIM: u32 = 0x0DA570;
const OPENING_ROUTINE: u32 = 0x00D9CA;
const PLACED_ROUTINE: u32 = 0x00D9F6;

/// A task's animation.
const TASK_ANIM: u32 = 0x20;
/// A sprite record's flags.
const SPRITE_FLAGS: u32 = 0x05;
/// An object's sprite's y's low byte (`func_00D990`, as the object at a4).
const OBJ_SPRITE_Y_LOW: u32 = 0x0B;
/// Cells' second plane (0x200 after a cell, `func_00167A`).
const PLANE_2: u32 = 0x200;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_00D906`, `D_00A1F0`'s routine: nothing.  (The four bytes
    /// after its `rts`, 0x4C80 0x4DC0, are not run.)
    pub fn r_00d906(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// A task (`func_001222`) made and placed (`func_00CA06`) with
    /// `SPRITE_KIND` `kind`.
    fn m_make(&mut self, regs: &mut Regs, kind: u8, depth: u8, routine: u32) {
        self.write8(SPRITE_KIND, kind);
        let r = &mut *regs;
        (r.d5, r.d6, r.a5, r.a6, r.x, _) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, PRIORITY, routine);
        (r.d0, r.c) = self.r_00ca06(r.d0, r.a5, r.a6);
        r.unwound = false;
    }

    /// `func_00D910`, a cell routine: `SPRITE_KIND` 0x8F, a task
    /// running `loc_00CAE8` (by the code at 0xC9F2, before `func_00CA06`).
    /// d0 and C are `func_00CA06`'s.
    pub fn r_00d910(&mut self, regs: &mut Regs) {
        self.m_make(regs, 0x8F, DEPTH_3, ROUTINE_CAE8);
    }

    /// `func_00D91A`, a cell routine: `SPRITE_KIND` 0x9B (0x9C when the
    /// byte at `POINTER_9EDA` is below 0x40), a task running
    /// `func_00D990` with sprite flags 4; command 0.
    pub fn r_00d91a(&mut self, regs: &mut Regs) {
        regs.a0 = self.read32(POINTER_9EDA);
        let byte = self.read8(regs.a0);
        regs.d1 = low_byte(regs.d1, byte);
        let kind = if byte < 0x40 { 0x9C } else { 0x9B };
        self.m_make(regs, kind, DEPTH_3, DEPTHS_ROUTINE);
        self.write8(regs.a5 + SPRITE_FLAGS, 4);
        regs.d0 = low_byte(regs.d0, 0);
        regs.c = false;
    }

    /// `func_00D95E`, a cell routine: `SPRITE_KIND` 0x98, a task
    /// running `func_00D9AA` (`OPENER_ANIM`); command 0x74.
    pub fn r_00d95e(&mut self, regs: &mut Regs) {
        self.m_make(regs, 0x98, OPENER_DEPTH, OPENER_ROUTINE);
        self.write32(regs.a6 + TASK_ANIM, OPENER_ANIM);
        regs.d0 = low_byte(regs.d0, 0x74);
        regs.c = false;
    }

    /// `func_00D990`, `func_00D91A`'s task: placed (`func_001B7E`), its
    /// depth from the y of the object at a4 (`func_0007D2`).
    pub fn r_00d990(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        (regs.d0, _, _) = self.r_001b7e(regs.a5, regs.a6);
        let (y, borrow) = self.read8(regs.a4 + OBJ_SPRITE_Y_LOW).overflowing_sub(8);
        regs.d0 = low_byte(regs.d0, (!y & 0xFE) | 2);
        regs.x = borrow;
        let r = &mut *regs;
        r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
    }

    /// `func_00D9AA`, `func_00D95E`'s task: placed until bit 5 of
    /// `FLAGS`; then on as `func_00D9CA`, the Z80 sent 0x54.
    pub fn r_00d9aa(&mut self, regs: &mut Regs) {
        if self.read8(FLAGS) & (1 << 5) == 0 {
            self.r_00d9f6(regs);
            return;
        }
        regs.unwound = false;
        (regs.x, _) = self.r_00053e(regs.d6, OPENING_ROUTINE);
        regs.d0 = low_byte(regs.d0, 0x54);
        regs.c = false;
        self.r_0034b6(regs.d0);
    }

    /// `func_00D9CA`: its animation run (`func_0013D6`); at its end the
    /// cell two rows below the task's becomes 4 (0 in the second
    /// plane) and the task only placed from now (`loc_00D9F6`).  Placed.
    pub fn r_00d9ca(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, ended) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if ended {
            r.d0 = self.r_0015f8(r.a6);
            let row = self.read8(CELL_Y).wrapping_add(2);
            self.write8(CELL_Y, row);
            (r.a0, _) = self.r_00167a();
            self.write8(r.a0, 4);
            self.write8(r.a0 + PLANE_2, 0);
            self.r_00053e(r.d6, PLACED_ROUTINE);
        }
        self.r_00d9f6(regs);
    }

    /// `loc_00D9F6`, a task's routine: placed (`func_001B7E`).
    pub fn r_00d9f6(&mut self, regs: &mut Regs) {
        (regs.d0, regs.x, regs.c) = self.r_001b7e(regs.a5, regs.a6);
        regs.unwound = false;
    }
}
