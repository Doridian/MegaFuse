//! A boss's arrival, from 0x15060 (`func_015060`, one of `D_01924E`'s
//! routines): its graphics loaded, a task that brings it down
//! (`func_0150DA`) and then makes the object (`func_015334`, in
//! `t_boss.rs`), and stays as its shadow (`func_015144`).

use super::t_common::*;
use crate::{Game, Regs};

/// The arrival's task (code addresses).
const ARRIVAL: u32 = 0x0150DA;
const SHADOW: u32 = 0x015144;
/// The boss object's first routine (`t_boss.rs`).
const BOSS_ROUTINE: u32 = 0x015334;

/// The sprite set the arrival loads (`func_000776`) and its frames'
/// table (in the ROM).
const SPRITE_SET: u32 = 1;
const SET_FRAMES: u32 = 0x0E8782;
/// The list of things to load (`i_load.rs`) and the one here, in the
/// ROM (`D_0150CA`).
const LOAD_LIST: u32 = 0xFFFF_92FA;
const ARRIVAL_LIST: u32 = 0x0150CA;
/// A long the loading changes, kept around it (absolute: 0xFF5810).
const KEPT_5810: u32 = 0x00FF_5810;
/// The boss object (`func_001CFA`'s index) and its animations (in the
/// ROM, `D_014FF0`).
const BOSS_INDEX: u8 = 6;
const BOSS_ANIMS: u32 = 0x014FF0;
/// The shadow's frames by the boss's height (`D_015170`, in the ROM).
const SHADOW_FRAMES: u32 = 0x015170;
/// The boss's (object 6's) offset (`OBJ_OFFSET`'s high byte: 0 on
/// the ground) and its rate's high byte (negative going up).
const HEIGHT: u32 = 0xFFFF_A5DC;
const HEIGHT_SIGN: u32 = 0xFFFF_A5DA;
/// The y the arrival comes down to (its y's low byte).
const LANDED_Y: u8 = 0x38;

impl Game {
    /// `func_015060`: `func_015068`.  Gives back every register, X and
    /// C.
    pub fn r_015060(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_015068(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        r.unwound = false;
    }

    /// `func_015068`: the boss arrives: sprite set `SPRITE_SET`, the
    /// list `ARRIVAL_LIST` loaded (`KEPT_5810` kept), the flags
    /// (`func_0094B2`), the task `ARRIVAL` (priority 0x42, sprite depth
    /// 0xFE) at (0x88, 0xF8), sound 0xE, and the step runs.  Gives back
    /// every register, X and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_015068(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let a0 = SET_FRAMES;
        let d0 = self.r_000776(SPRITE_SET, a0, a1);
        let kept = self.read32(KEPT_5810);
        self.write32(LOAD_LIST, ARRIVAL_LIST);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_none() {
            self.write32(KEPT_5810, kept);
            self.r_0094b2();
            self.t_make_task(&mut r, 0xFE, 0x42, ARRIVAL);
            self.write16(r.a6 + TASK_X, 0x88);
            self.write16(r.a6 + TASK_Y, 0xF8);
            self.write8(r.a5 + SPRITE_FLAGS, 0x81);
            self.t_sound(&mut r, 0x0E);
            self.write8(STEP_RUNS, 1);
            r.c = false;
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_0150DA`, `ARRIVAL`: frame 0x45, the task comes up 2 a
    /// frame; at `LANDED_Y` the boss object (`BOSS_INDEX`) is made
    /// there (`BOSS_ANIMS`, routine `BOSS_ROUTINE`; offset 0x88, low
    /// bytes 0x21 and 6, 2 in `OBJ_26`), `HELPERS` 2, and the task
    /// stays as `SHADOW` (at once).  Gives back the registers as the
    /// original leaves them.
    pub fn r_0150da(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, 0x45);
        let y = self.read8(r.a6 + TASK_Y_LOW).wrapping_sub(2);
        self.write8(r.a6 + TASK_Y_LOW, y);
        r.d0 = low_byte(r.d0, y);
        if y != LANDED_Y {
            self.t_task_place(r);
            return;
        }
        self.r_00153a(r.a6);
        r.d4 = low_byte(r.d4, BOSS_INDEX);
        (r.d4, r.a4, r.x) = self.r_001cfa(r.d4);
        self.write32(OBJ_INIT_ANIMS, BOSS_ANIMS);
        self.write32(OBJ_INIT_ROUTINE, BOSS_ROUTINE);
        (r.d0, r.d5, r.a5, r.x) = self.r_0012a0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
        self.write8(r.a4 + OBJ_OFFSET, 0x88);
        self.write8(r.a5 + SPRITE_FLAGS, 0x81);
        self.write8(r.a4 + OBJ_0F, 0x21);
        self.write8(r.a4 + OBJ_11, 6);
        self.write8(r.a4 + OBJ_26, 2);
        self.write8(HELPERS, 2);
        self.t_next_task(r, SHADOW);
        self.r_015144(r);
    }

    /// `func_015144`, `SHADOW`: while the boss is off the ground
    /// (`HEIGHT` not 0, or `HEIGHT_SIGN` negative), the running sprite
    /// record's frame from `SHADOW_FRAMES` by -`HEIGHT` / 16, placed
    /// for the task; else the task ends.  Gives back d0, d5, a5, X and
    /// C, or d6, X and C.
    pub fn r_015144(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let height = self.read8(HEIGHT);
        r.d0 = low_word(r.d0, u16::from(height));
        if height == 0 && (self.read8(HEIGHT_SIGN) as i8) >= 0 {
            r.c = false;
            self.t_task_end(r);
            return;
        }
        let neg = height.wrapping_neg();
        r.x = neg & 0x08 != 0;
        let index = neg >> 4;
        r.d0 = low_byte(r.d0, index);
        self.t_reload_sprite(r);
        let frame = self.read8(SHADOW_FRAMES + u32::from(index));
        self.write8(r.a5 + SPRITE_FRAME, frame);
        self.t_task_place(r);
    }
}
