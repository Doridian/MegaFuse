//! Routines the script gives the scripted task (`func_016104` sets
//! them): it still runs as `func_015EB4` each frame (the script and its
//! moves), and plays animation D_016D78 through, shows frame 0x48 then
//! its own frame (0x49-0x4C, by the routine it started at) for 0x30
//! frames, plays D_016D82 and goes back to `func_015EB4` with the
//! script's frame 0.

use crate::{Game, Regs};

/// The scripted task's frame (`u_script.rs`).
const FRAME_A: u32 = 0xFFAD76;

/// A task's animation timer and step (`anim.rs`), its count
/// and the frame it shows (`func_016D2A`), its animation (a long).
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_FRAME: u32 = 0x17;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's frame.
const SPRITE_FRAME: u32 = 0x04;

/// The two animations (D_016D78, D_016D82) and the routines.
const ANIM_OPEN: u32 = 0x016D78;
const ANIM_CLOSE: u32 = 0x016D82;
const ROUTINE_16D00: u32 = 0x016D00;
const ROUTINE_16D2A: u32 = 0x016D2A;
const ROUTINE_16D56: u32 = 0x016D56;
const ROUTINE_15EB4: u32 = 0x015EB4;

impl Game {
    /// `func_016CC0`: `func_016CE4` with frame 0x4B.
    pub fn r_016cc0(&mut self, regs: &mut Regs) {
        self.u_start(regs, 0x4B);
    }

    /// `func_016CCA`: `func_016CE4` with frame 0x49.
    pub fn r_016cca(&mut self, regs: &mut Regs) {
        self.u_start(regs, 0x49);
    }

    /// `func_016CD4`: `func_016CE4` with frame 0x4A.
    pub fn r_016cd4(&mut self, regs: &mut Regs) {
        self.u_start(regs, 0x4A);
    }

    /// `func_016CDE`: `func_016CE4` with frame 0x4C.
    pub fn r_016cde(&mut self, regs: &mut Regs) {
        self.u_start(regs, 0x4C);
    }

    /// The task's frame `frame`, then `func_016CE4`.
    fn u_start(&mut self, r: &mut Regs, frame: u8) {
        self.write8(r.a6 + TASK_FRAME, frame);
        r.unwound = false;
        self.u_016ce4(r);
    }

    /// `.L016CE4` on the family's registers.
    fn u_016ce4(&mut self, r: &mut Regs) {
        self.write32(r.a6 + TASK_ANIM, ANIM_OPEN);
        self.write8(r.a6 + TASK_STEP, 0);
        self.write8(r.a6 + TASK_TIMER, 0);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_16D00);
        self.r_016d00(r);
    }

    /// `.L016CE4`: the task's animation D_016D78 from its start, its
    /// routine `func_016D00` (and run now).  Gives back d0, d1, d2, d3,
    /// d5, d6, a0, a5, a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_016ce4(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        self.u_016ce4(&mut r);
        (r.d0, r.d1, r.d2, r.d3, r.d5, r.d6, r.a0, r.a5, r.a6, r.x, r.c)
    }

    /// `func_016D00`, a task's routine: `func_015EB4` and its animation
    /// a frame on (`func_0013D6`); at its end the sprite's frame is
    /// 0x48, the count 0x30 and the routine `func_016D2A` (from the
    /// next frame).
    pub fn r_016d00(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.r_015eb4(r);
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if !r.c {
            return;
        }
        self.write8(r.a5 + SPRITE_FRAME, 0x48);
        self.write8(r.a6 + TASK_COUNT, 0x30);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_16D2A);
    }

    /// `func_016D2A`, a task's routine: `func_015EB4`, the sprite's
    /// frame the task's; after the count's frames, animation D_016D82
    /// and the routine `func_016D56` (from the next frame).  Gives back
    /// X and C (the count's borrow) while it counts.
    pub fn r_016d2a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.r_015eb4(r);
        r.unwound = false;
        let frame = self.read8(r.a6 + TASK_FRAME);
        self.write8(r.a5 + SPRITE_FRAME, frame);
        let count = self.read8(r.a6 + TASK_COUNT);
        let left = count.wrapping_sub(1);
        self.write8(r.a6 + TASK_COUNT, left);
        r.x = count == 0;
        r.c = r.x;
        if left != 0 {
            return;
        }
        self.write32(r.a6 + TASK_ANIM, ANIM_CLOSE);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_16D56);
    }

    /// `func_016D56`, a task's routine: `func_015EB4` and its animation
    /// a frame on (`func_0013D6`); at its end the script's frame is 0
    /// and the routine `func_015EB4` again (from the next frame).
    pub fn r_016d56(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.r_015eb4(r);
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if !r.c {
            return;
        }
        self.write8(FRAME_A, 0);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_15EB4);
    }
}
