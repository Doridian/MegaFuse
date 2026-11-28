//! Object routines of 0x12338-0x125EA that go step by step on a
//! shared count, `CUE`: a routine waits for it not to be 0 (and most
//! clear it), another counts it up when its own `OBJ_COUNT` runs out
//! (`func_012338`) or at once.  Each sets the routine of the next frame
//! in `OBJ_ROUTINE` and the one after its animation or count in
//! `OBJ_SAVED`, and most fall into the new one at once.
//!
//! Most end the same way: the sprite placed for the object
//! (`func_001B32`) and its depth less the object's offset
//! (`func_001B50`).

use crate::{Game, Regs};

/// An object's fields: its routine (a long), the routine it takes
/// when its animation or count ends (a long), its direction, a byte
/// `func_012408` counts down, its animation (a long) and the
/// animation's counters (`anim.rs`), its count (`func_012338`) and its
/// rate (a word, `a_motion.rs`).
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_SAVED: u32 = 0x06;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_1B: u32 = 0x1B;
const OBJ_ANIM: u32 = 0x20;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_COUNT: u32 = 0x26;
const OBJ_RATE: u32 = 0x28;

/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
/// The flag `func_012418` and `func_012556` set.
const SPRITE_FLAG_10: u8 = 0x10;

/// The count the routines here go by.
const CUE: u32 = 0xFFFF_9668;
/// Counted up by `func_012494`; `func_01259A` waits for 3.
const COUNT_AD58: u32 = 0xFFFF_AD58;
/// The velocity's y (a word, `a_motion.rs`).
const VELOCITY_Y: u32 = 0xFFFF_9EC6;
/// Object 0's record (`a_objects.rs`).
const OBJECT_0: u32 = 0xFFFF_A432;

/// Animations in the ROM.
const ANIM_EC9E4: u32 = 0x0EC9E4;
const ANIM_ECA0C: u32 = 0x0ECA0C;
const ANIM_ECA3A: u32 = 0x0ECA3A;
const ANIM_EF8D6: u32 = 0x0EF8D6;
const ANIM_125C2: u32 = 0x0125C2;
const ANIM_125CA: u32 = 0x0125CA;
const ANIM_125DC: u32 = 0x0125DC;
const ANIM_125E4: u32 = 0x0125E4;

/// The routines.
const ANIMATING: u32 = 0x012366;
const WAIT_012388: u32 = 0x012388;
const MOVING: u32 = 0x0123C4;
const AFTER_MOVING: u32 = 0x0123CC;
const WAIT_0123D8: u32 = 0x0123D8;
const COUNTING_012408: u32 = 0x012408;
const AFTER_012408: u32 = 0x012418;
const WAIT_012432: u32 = 0x012432;
const COUNTING_012452: u32 = 0x012452;
const AFTER_012452: u32 = 0x01245E;
const START_01246C: u32 = 0x01246C;
const COUNTING_012482: u32 = 0x012482;
const AFTER_012482: u32 = 0x012494;
const ANIMATING_0124A0: u32 = 0x0124A0;
const COUNTING: u32 = 0x0124BE;
const AFTER_COUNTING: u32 = 0x0124CC;
const AFTER_0124CC: u32 = 0x012502;
const AFTER_012502: u32 = 0x012518;
const WAIT_012526: u32 = 0x012526;
const AFTER_012526: u32 = 0x012546;
const WAIT_012556: u32 = 0x012556;
const AFTER_012556: u32 = 0x012586;
const WAIT_01259A: u32 = 0x01259A;
const ANIMATING_0125BA: u32 = 0x0125BA;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_001336`: the object's animation a frame on (C: it ended).
    /// Gives back its Z.
    fn q_animate(&mut self, r: &mut Regs) -> bool {
        let z;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, z, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        z
    }

    /// `func_001B32` and `func_001B50`: the sprite placed, its depth.
    fn q_place(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_0123bc(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_012338` and `q_place` (`loc_0124BE`).
    fn q_count_place(&mut self, r: &mut Regs) {
        self.r_012338(r.a4);
        self.q_place(r);
    }

    /// The routine from the next frame on, and the one after.
    fn q_next(&mut self, obj: u32, routine: u32, after: u32) {
        self.write32(obj + OBJ_ROUTINE, routine);
        self.write32(obj + OBJ_SAVED, after);
    }

    /// Whether `CUE` is set.
    fn q_cued(&mut self) -> bool {
        self.read8(CUE) != 0
    }

    /// `CUE` counted up (X its carry).
    fn q_cue(&mut self, r: &mut Regs) {
        let n = self.read8(CUE);
        self.write8(CUE, n.wrapping_add(1));
        r.x = n == 0xFF;
    }

    /// The sprite's frame `frame`, flag `SPRITE_FLAG_10` set (d0's low
    /// byte the flags).
    fn q_sprite_flagged(&mut self, r: &mut Regs, frame: u8) {
        self.write8(r.a5 + SPRITE_FRAME, frame);
        let flags = self.read8(r.a5 + SPRITE_FLAGS) | SPRITE_FLAG_10;
        r.d0 = low_byte(r.d0, flags);
        self.write8(r.a5 + SPRITE_FLAGS, flags);
    }

    /// `func_012338`: `OBJ_COUNT` counted down; when it gets to 0,
    /// `CUE` counted up and the routine from `OBJ_SAVED`.  (Its C, set
    /// then, isn't read.)
    /// No script calls it: reviewed by reading.
    pub fn r_012338(&mut self, a4: u32) {
        let count = self.read8(a4 + OBJ_COUNT).wrapping_sub(1);
        self.write8(a4 + OBJ_COUNT, count);
        if count == 0 {
            let n = self.read8(CUE);
            self.write8(CUE, n.wrapping_add(1));
            let saved = self.read32(a4 + OBJ_SAVED);
            self.write32(a4 + OBJ_ROUTINE, saved);
        }
    }

    /// `loc_012356`: the animation from its start, `func_012366` from
    /// now on.  Gives back d0, d1, d2, d5, a0, a5, X and C.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments)]
    pub fn r_012356(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        self.write8(a4 + OBJ_TIMER, 0);
        self.write8(a4 + OBJ_STEP, 0);
        self.write32(a4 + OBJ_ROUTINE, ANIMATING);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, n: false, z: false, v: false, c: false, unwound: false };
        self.r_012366(&mut r);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c)
    }

    /// `func_012366`: the animation a frame on; when it ends, the
    /// routine from `OBJ_SAVED`.  Then placed (`q_place`).
    /// No script calls it: reviewed by reading.
    pub fn r_012366(&mut self, regs: &mut Regs) {
        self.q_animate(regs);
        if regs.c {
            let saved = self.read32(regs.a4 + OBJ_SAVED);
            self.write32(regs.a4 + OBJ_ROUTINE, saved);
        }
        self.q_place(regs);
        regs.unwound = false;
    }

    /// `func_01237C`: `CUE` cleared, `func_012388` from now on (and now:
    /// N and Z clear, from the routine's address).
    /// No script calls it: reviewed by reading.
    pub fn r_01237c(&mut self, regs: &mut Regs) {
        self.write8(CUE, 0);
        self.write32(regs.a4 + OBJ_ROUTINE, WAIT_012388);
        regs.n = false;
        regs.z = false;
        regs.c = false;
        self.r_012388(regs);
    }

    /// `func_012388`: once `CUE` is set, the object goes up (`OBJ_RATE`
    /// 0xFE80, direction 0xFF, `ANIM_ECA0C` from its start):
    /// `func_0123C4`, then `func_0123CC`.  Placed (`q_place`).
    /// No script calls it: reviewed by reading.
    pub fn r_012388(&mut self, regs: &mut Regs) {
        let obj = regs.a4;
        if self.q_cued() {
            self.write16(obj + OBJ_RATE, 0xFE80);
            self.write8(obj + OBJ_DIRECTION, 0xFF);
            self.q_next(obj, MOVING, AFTER_MOVING);
            self.write32(obj + OBJ_ANIM, ANIM_ECA0C);
            self.write8(obj + OBJ_TIMER, 0);
            self.write8(obj + OBJ_STEP, 0);
        }
        self.q_place(regs);
        regs.unwound = false;
    }

    /// `func_0123BC` (the end of several routines here): `func_001B32`
    /// and `func_001B50`.  Gives back d0, d1, X and C.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0123bc(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        let (d0, x, c) = self.r_001b50(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        (d0, d1, x, c)
    }

    /// `func_0124C4`: the same as `func_0123BC`.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0124c4(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        self.r_0123bc(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5)
    }

    /// `func_0123C4`: the animation a frame on, then `func_004E5C`.
    /// No script calls it: reviewed by reading.
    pub fn r_0123c4(&mut self, regs: &mut Regs) {
        regs.z = self.q_animate(regs);
        self.r_004e5c(regs);
        regs.unwound = false;
    }

    /// `func_0123CC`: `CUE` cleared, `func_0123D8` from now on (and
    /// now).
    /// No script calls it: reviewed by reading.
    pub fn r_0123cc(&mut self, regs: &mut Regs) {
        self.write8(CUE, 0);
        self.write32(regs.a4 + OBJ_ROUTINE, WAIT_0123D8);
        regs.c = false;
        self.r_0123d8(regs);
    }

    /// `func_0123D8`: once `CUE` is set (cleared again), `ANIM_EC9E4`
    /// from its start, a count of 0x20 (`func_012408`, then
    /// `func_012418`), run now.  Else placed (`q_place`).
    /// No script calls it: reviewed by reading.
    pub fn r_0123d8(&mut self, regs: &mut Regs) {
        let obj = regs.a4;
        regs.c = false;
        if !self.q_cued() {
            self.q_place(regs);
            regs.unwound = false;
            return;
        }
        self.write8(CUE, 0);
        self.write32(obj + OBJ_ANIM, ANIM_EC9E4);
        self.write8(obj + OBJ_TIMER, 0);
        self.write8(obj + OBJ_STEP, 0);
        self.write8(obj + OBJ_COUNT, 0x20);
        self.q_next(obj, COUNTING_012408, AFTER_012408);
        self.r_012408(regs);
    }

    /// `func_012408`: the animation a frame on, counted
    /// (`func_012338`), `OBJ_1B` one less, placed (`q_place`).
    /// No script calls it: reviewed by reading.
    pub fn r_012408(&mut self, regs: &mut Regs) {
        self.q_animate(regs);
        self.r_012338(regs.a4);
        let n = self.read8(regs.a4 + OBJ_1B);
        self.write8(regs.a4 + OBJ_1B, n.wrapping_sub(1));
        self.q_place(regs);
        regs.unwound = false;
    }

    /// `func_012418`: the sprite's frame 1 and flag 0x10, then
    /// `func_012432` from now on (and now).
    /// No script calls it: reviewed by reading.
    pub fn r_012418(&mut self, regs: &mut Regs) {
        self.q_sprite_flagged(regs, 1);
        self.write32(regs.a4 + OBJ_ROUTINE, WAIT_012432);
        regs.c = false;
        self.r_012432(regs);
    }

    /// `func_012432`: once `CUE` is set (cleared again), a count of
    /// 0x60 (`func_012452`, then `func_01245E`), run now.  Else placed
    /// (`q_place`).
    /// No script calls it: reviewed by reading.
    pub fn r_012432(&mut self, regs: &mut Regs) {
        let obj = regs.a4;
        regs.c = false;
        if !self.q_cued() {
            self.q_place(regs);
            regs.unwound = false;
            return;
        }
        self.write8(CUE, 0);
        self.write8(obj + OBJ_COUNT, 0x60);
        self.q_next(obj, COUNTING_012452, AFTER_012452);
        self.r_012452(regs);
    }

    /// `func_012452`: the animation two frames on, counted and placed
    /// (`loc_0124BE`).
    /// No script calls it: reviewed by reading.
    pub fn r_012452(&mut self, regs: &mut Regs) {
        self.q_animate(regs);
        self.q_animate(regs);
        self.q_count_place(regs);
        regs.unwound = false;
    }

    /// `func_01245E`: `ANIM_ECA3A`, run once (`loc_012356`).
    /// No script calls it: reviewed by reading.
    pub fn r_01245e(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write32(r.a4 + OBJ_ANIM, ANIM_ECA3A);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_012356(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        r.unwound = false;
    }

    /// `func_01246C`: a count of 0x20 (`func_012482`, then
    /// `func_012494`), run now.
    /// No script calls it: reviewed by reading.
    pub fn r_01246c(&mut self, regs: &mut Regs) {
        self.q_next(regs.a4, COUNTING_012482, AFTER_012482);
        self.write8(regs.a4 + OBJ_COUNT, 0x20);
        regs.c = false;
        self.r_012482(regs);
    }

    /// `func_012482`: the object moved by `VELOCITY_Y` 0x100
    /// (`func_0019DA`), the animation a frame on, counted and placed
    /// (`loc_0124BE`).
    /// No script calls it: reviewed by reading.
    pub fn r_012482(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write16(VELOCITY_Y, 0x100);
        (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        self.q_animate(r);
        self.q_count_place(r);
        r.unwound = false;
    }

    /// `func_012494`: `COUNT_AD58` counted up, `func_0124A0` from now
    /// on (and now).
    /// No script calls it: reviewed by reading.
    pub fn r_012494(&mut self, regs: &mut Regs) {
        let n = self.read8(COUNT_AD58);
        self.write8(COUNT_AD58, n.wrapping_add(1));
        regs.x = n == 0xFF;
        self.write32(regs.a4 + OBJ_ROUTINE, ANIMATING_0124A0);
        regs.c = false;
        self.r_0124a0(regs);
    }

    /// `func_0124A0`: the animation a frame on, the sprite placed
    /// (`func_001B32`).
    /// No script calls it: reviewed by reading.
    pub fn r_0124a0(&mut self, regs: &mut Regs) {
        let r = regs;
        self.q_animate(r);
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        r.unwound = false;
    }

    /// `func_0124A8`: a count of 0x40 (`loc_0124BE`, then
    /// `func_0124CC`), run now (N and Z clear, from the routine's
    /// address).
    /// No script calls it: reviewed by reading.
    pub fn r_0124a8(&mut self, regs: &mut Regs) {
        self.write8(regs.a4 + OBJ_COUNT, 0x40);
        self.q_next(regs.a4, COUNTING, AFTER_COUNTING);
        regs.n = false;
        regs.z = false;
        regs.c = false;
        self.r_0124be(regs);
    }

    /// `loc_0124BE`: counted (`func_012338`) and placed (`q_place`).
    /// No script calls it: reviewed by reading.
    pub fn r_0124be(&mut self, regs: &mut Regs) {
        self.q_count_place(regs);
        regs.unwound = false;
    }

    /// `func_0124CC`: the object goes up (`func_0123C4`, then
    /// `func_012502`): `OBJ_RATE` 0xFD00, a count of 1 (its byte after
    /// 0), direction 0, `ANIM_EF8D6` from its start, then
    /// `func_004FD0`.
    /// No script calls it: reviewed by reading.
    pub fn r_0124cc(&mut self, regs: &mut Regs) {
        let r = regs;
        let obj = r.a4;
        self.q_next(obj, MOVING, AFTER_0124CC);
        self.write16(obj + OBJ_RATE, 0xFD00);
        self.write16(obj + OBJ_COUNT, 0x100);
        self.write8(obj + OBJ_DIRECTION, 0);
        self.write32(obj + OBJ_ANIM, ANIM_EF8D6);
        self.write8(obj + OBJ_TIMER, 0);
        self.write8(obj + OBJ_STEP, 0);
        (r.d0, r.x, r.c) = self.r_004fd0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        r.unwound = false;
    }

    /// Sets the animation and the routine after it, then runs it once
    /// (`loc_012356`).
    fn q_animation(&mut self, r: &mut Regs, anim: u32, after: u32) {
        self.write32(r.a4 + OBJ_ANIM, anim);
        self.write32(r.a4 + OBJ_SAVED, after);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_012356(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
    }

    /// `func_012502`: `ANIM_125C2`, then `func_012518`.
    /// No script calls it: reviewed by reading.
    pub fn r_012502(&mut self, regs: &mut Regs) {
        self.q_animation(regs, ANIM_125C2, AFTER_012502);
        regs.unwound = false;
    }

    /// `CUE` counted up, `routine` from the next frame on, placed
    /// (`func_0124C4`).
    fn q_cue_then(&mut self, r: &mut Regs, routine: u32) {
        self.q_cue(r);
        self.write32(r.a4 + OBJ_ROUTINE, routine);
        (r.d0, r.d1, r.x, r.c) = self.r_0124c4(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_012518`: `CUE` counted up, `func_012526` next; placed.
    /// No script calls it: reviewed by reading.
    pub fn r_012518(&mut self, regs: &mut Regs) {
        self.q_cue_then(regs, WAIT_012526);
        regs.unwound = false;
    }

    /// `func_012526`: once `CUE` is set (cleared again), `ANIM_125CA`,
    /// then `func_012546`.  Else placed.
    /// No script calls it: reviewed by reading.
    pub fn r_012526(&mut self, regs: &mut Regs) {
        let r = regs;
        r.c = false;
        if self.q_cued() {
            self.write8(CUE, 0);
            self.q_animation(r, ANIM_125CA, AFTER_012526);
        } else {
            (r.d0, r.d1, r.x, r.c) = self.r_0124c4(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
        r.unwound = false;
    }

    /// `func_012546`: `CUE` counted up, `func_012556` next; placed.
    /// No script calls it: reviewed by reading.
    pub fn r_012546(&mut self, regs: &mut Regs) {
        self.q_cue_then(regs, WAIT_012556);
        regs.unwound = false;
    }

    /// `func_012556`: once `CUE` is set, the sprite's frame 0x26 and
    /// flag 0x10, `ANIM_125DC`, then `func_012586`.  Else placed.
    /// No script calls it: reviewed by reading.
    pub fn r_012556(&mut self, regs: &mut Regs) {
        let r = regs;
        r.c = false;
        if self.q_cued() {
            self.q_sprite_flagged(r, 0x26);
            self.q_animation(r, ANIM_125DC, AFTER_012556);
        } else {
            (r.d0, r.d1, r.x, r.c) = self.r_0124c4(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
        r.unwound = false;
    }

    /// `func_012586`: object 0's routine `func_01246C`, this one's
    /// `func_01259A` from now on (and now).  a0 object 0.
    /// No script calls it: reviewed by reading.
    pub fn r_012586(&mut self, regs: &mut Regs) {
        regs.a0 = OBJECT_0;
        self.write32(OBJECT_0 + OBJ_ROUTINE, START_01246C);
        self.write32(regs.a4 + OBJ_ROUTINE, WAIT_01259A);
        regs.c = false;
        self.r_01259a(regs);
    }

    /// `func_01259A`: once `COUNT_AD58` is 3, `func_0125BA` from the
    /// next frame on with `ANIM_125E4`.  The sprite placed
    /// (`func_001B32`).
    /// No script calls it: reviewed by reading.
    pub fn r_01259a(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.read8(COUNT_AD58) == 3 {
            self.write32(r.a4 + OBJ_ROUTINE, ANIMATING_0125BA);
            self.write32(r.a4 + OBJ_ANIM, ANIM_125E4);
        }
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        r.unwound = false;
    }

    /// `func_0125BA`: the animation a frame on, the sprite placed
    /// (`func_001B32`), as `func_0124A0`.
    /// No script calls it: reviewed by reading.
    pub fn r_0125ba(&mut self, regs: &mut Regs) {
        self.r_0124a0(regs);
    }
}
