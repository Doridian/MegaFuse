//! The task object 6 makes over a cell of 1 near it (`func_019E46`,
//! made by 0x19D10 in `x_object.rs`; script stage-3-boss): it moves to
//! its cell's place (`func_00112C`, its sprite half way to the target
//! by `func_019FB6`); there, on a cell of 1 that `func_005A76` takes,
//! the cell is taken (`func_005A4E`, the cell emptied, `func_0052F0`)
//! and it plays its animation (`func_019F0E`); then it moves to the
//! target (`TARGET_X`, `TARGET_Y`; `func_019F2E`), and once there, or
//! once object 6's byte 0 is 1, its second sprite is freed and it ends
//! (`func_019F90`, `func_019F9A`).

use crate::{Game, Regs};

/// The index of the task running (`task.rs`), and a byte it is kept in
/// while `func_005A4E` runs as another.
const CURRENT_TASK: u32 = 0xFFFF_80EE;
const KEPT_TASK: u32 = 0xFFFF_80C5;
/// The sprite record kept (index, address).
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;
/// Byte 0 of object 6 (0xFFA5B2): at `GONE` the task goes; the task
/// sets it to `GONE` at its end.
const OBJECT_6_STATE: u32 = 0xFFFF_A5B2;
const GONE: u8 = 1;
/// Point A (words; `d_angle.rs`) and the speed (a word, 8.8;
/// `func_00112C`).
const A_X: u32 = 0xFFFF_9EE6;
const A_Y: u32 = 0xFFFF_9EE8;
const SPEED: u32 = 0xFFFF_9F00;
const SPEED_1: u16 = 0x0100;
/// Where the task goes after its cell (words, x then y; their low
/// bytes).
const TARGET_X: u32 = 0xFFFF_ADE6;
const TARGET_X_LOW: u32 = 0xFFFF_ADE7;
const TARGET_Y: u32 = 0xFFFF_ADE8;
const TARGET_Y_LOW: u32 = 0xFFFF_ADE9;

/// The task's fields: its first sprite record (index), its position's
/// low bytes (`x_object.rs`'s `TASK_X`, `TASK_Y` words), the cell's
/// place (bytes, x then y) and its second sprite record (index).
const TASK_SPRITE: u32 = 0x03;
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y_LOW: u32 = 0x11;
const TASK_PLACE_X: u32 = 0x19;
const TASK_PLACE_Y: u32 = 0x1A;
const TASK_SPRITE_2: u32 = 0x1B;

/// The map's cell that is taken (`func_00167A`).
const CELL_1: u8 = 1;
/// Z80 command when it is taken (`func_0034B6`).
const TAKEN_SOUND: u8 = 0x67;

/// Where the task goes on (resume addresses).
const AFTER_TAKEN: u32 = 0x019F0E;
const TO_TARGET: u32 = 0x019F2E;
const ENDING: u32 = 0x019F9A;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// The task a step toward (`to_x`, `to_y`) (A; `func_001A20`,
    /// `func_001A28`, `func_00112C` at `SPEED_1`, `func_001A88`,
    /// `func_001AA2`), its second sprite half way (`func_019FB6`); with
    /// `OBJECT_6_STATE` at `GONE`, the end (`func_019F90`), else placed
    /// (`func_001B7E`, the sprite record kept as a5).
    fn yc_step_to(&mut self, r: &mut Regs, to_x: u16, to_y: u16) {
        self.r_001a20(r.a6);
        self.r_001a28(r.a6);
        self.write16(A_X, to_x);
        self.write16(A_Y, to_y);
        self.write16(SPEED, SPEED_1);
        self.r_00112c();
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        (r.d0, r.x) = self.r_019fb6(r.d0, r.d5, r.a6);
        if self.read8(OBJECT_6_STATE) == GONE {
            self.yc_end(r);
            return;
        }
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// The end (0x19F90) for the registers in `r`.
    fn yc_end(&mut self, r: &mut Regs) {
        let t = self.r_019f90(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        (r.d0, r.d6, r.x, r.c) = t;
    }

    /// The way to the target (0x19F2A) for the registers in `r`.
    fn yc_to_target(&mut self, r: &mut Regs) {
        let t = self.r_019f2a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        (r.d0, r.d1, r.d5, r.d6, r.a5, r.x, r.c) = t;
    }

    /// `func_019E46` (a task's routine): away from its cell's place, a
    /// step to it (`TASK_PLACE_X`, `TASK_PLACE_Y`).  At it: unless the
    /// cell (`func_0015F8`, `func_00167A`) is 1 and `func_005A76` takes
    /// it (C), the way to the target (0x19F2A); else the task's first
    /// sprite kept (`func_000B5A`, in `SLOT_INDEX`, `SLOT_ADDR`), the
    /// cell taken as task `d6` (`func_005A4E`, `func_00168C`,
    /// `func_001278`, `func_0052F0`; the running task kept in
    /// `KEPT_TASK`), the Z80 command `TAKEN_SOUND` and a frame on
    /// (`func_019F0E`).
    pub fn r_019e46(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let place_x = self.read8(r.a6 + TASK_PLACE_X);
        r.d0 = low_byte(r.d0, place_x);
        let mut there = place_x == self.read8(r.a6 + TASK_X_LOW);
        if there {
            let place_y = self.read8(r.a6 + TASK_PLACE_Y);
            r.d0 = low_byte(r.d0, place_y);
            there = place_y == self.read8(r.a6 + TASK_Y_LOW);
        }
        if !there {
            let to_x = u16::from(self.read8(r.a6 + TASK_PLACE_X));
            let to_y = u16::from(self.read8(r.a6 + TASK_PLACE_Y));
            self.yc_step_to(r, to_x, to_y);
            return;
        }
        let current = self.read8(CURRENT_TASK);
        self.write8(KEPT_TASK, current);
        r.d0 = self.r_0015f8(r.a6);
        (r.a0, r.x) = self.r_00167a();
        let cell = self.read8(r.a0);
        r.d0 = low_byte(r.d0, cell);
        if cell != CELL_1 {
            r.c = cell < CELL_1;
            self.yc_to_target(r);
            return;
        }
        (r.d6, r.a6, r.x, _, r.c) = self.r_005a76(r.d2, r.d6, r.a6);
        if !r.c {
            self.yc_to_target(r);
            return;
        }
        let (d5, a5) = (r.d5, r.a5);
        let index = low_byte(r.d5, self.read8(r.a6 + TASK_SPRITE));
        let (index, sprite, _) = self.r_000b5a(index);
        self.write8(SLOT_INDEX, index as u8);
        self.write32(SLOT_ADDR, sprite);
        (r.d5, r.a5) = (d5, a5);
        self.write8(CURRENT_TASK, r.d6 as u8);
        (r.d0, r.d4, r.d6, r.a0, r.a4, r.x, r.c) = self.r_005a4e(r.d0, r.d4, r.d5, r.d6, r.a6);
        (r.d0, r.a0) = self.r_00168c(r.d0);
        self.r_001278();
        (r.x, r.c) = self.r_0052f0(r.d0);
        let kept = self.read8(KEPT_TASK);
        self.write8(CURRENT_TASK, kept);
        r.d0 = low_byte(r.d0, TAKEN_SOUND);
        self.r_0034b6(r.d0);
        self.yc_yield(r, AFTER_TAKEN);
    }

    /// `func_019F0E` (a task's routine): with `OBJECT_6_STATE` at
    /// `GONE`, the end (0x19F90); else its animation a frame on
    /// (`func_0013D6`, the sprite record kept); at its end the way to
    /// the target (0x19F2A).
    pub fn r_019f0e(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.read8(OBJECT_6_STATE) == GONE {
            self.yc_end(r);
            return;
        }
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            self.yc_to_target(r);
        }
    }

    /// `func_019F2A` (0x19F2A): a frame on, to go to the target
    /// (`func_019F2E`).  Gives back d0, d1, d5, d6, a5, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_019f2a(&mut self, d0: u32, d1: u32, _d2: u32, _d3: u32, _d4: u32, d5: u32, d6: u32, _d7: u32, _a0: u32, _a1: u32, _a2: u32, _a3: u32, _a4: u32, a5: u32, _a6: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let (x, c, _) = self.r_000522(d6, TO_TARGET);
        (d0, d1, d5, d6, a5, x, c)
    }

    /// `func_019F2E` (a task's routine): at the target (its low bytes),
    /// the end (0x19F90); else a step to it.
    pub fn r_019f2e(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let x_low = self.read8(r.a6 + TASK_X_LOW);
        r.d0 = low_byte(r.d0, x_low);
        if x_low == self.read8(TARGET_X_LOW) {
            let y_low = self.read8(r.a6 + TASK_Y_LOW);
            r.d0 = low_byte(r.d0, y_low);
            let target = self.read8(TARGET_Y_LOW);
            if y_low == target {
                r.c = false;
                self.yc_end(r);
                return;
            }
        }
        let to_x = self.read16(TARGET_X);
        let to_y = self.read16(TARGET_Y);
        self.yc_step_to(r, to_x, to_y);
    }

    /// `func_019F90` (0x19F90): `OBJECT_6_STATE` set to `GONE` and a
    /// frame on (`func_019F9A`).  Gives back d0, d6, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_019f90(&mut self, d0: u32, _d1: u32, _d2: u32, _d3: u32, _d4: u32, _d5: u32, d6: u32, _d7: u32, _a0: u32, _a1: u32, _a2: u32, _a3: u32, _a4: u32, _a5: u32, _a6: u32) -> (u32, u32, bool, bool) {
        self.write8(OBJECT_6_STATE, GONE);
        let (x, c, _) = self.r_000522(d6, ENDING);
        (d0, d6, x, c)
    }

    /// `func_019F9A` (a task's routine): its second sprite freed
    /// (`func_000870`; the sprite record kept stays) and the task ends
    /// (`func_00121A`).
    pub fn r_019f9a(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let index = self.read8(SLOT_INDEX);
        let addr = self.read32(SLOT_ADDR);
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_SPRITE_2));
        r.x = self.r_000870(r.d0, r.d5);
        self.write32(SLOT_ADDR, addr);
        self.write8(SLOT_INDEX, index);
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
    }
}
