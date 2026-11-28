//! Object 7 coming down, 0x13F14-0x1401A: `func_013F14` sets it up
//! (routine `func_013F58`), which moves it down by its rate (a word,
//! slowed by 8 a frame, `func_004424`) until that goes negative; then
//! two animations (`func_013FA0`, `func_013FC0`), each waiting for
//! `COUNT_AD57` to be 0, the second stepping it; after them the task
//! `func_01481A` and the set-up of 0x14xxx (`func_014842`,
//! `func_0147B4`, `func_014886`), and on to `func_01401A`.
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read.

use crate::{Game, Regs};

/// An object's fields: its routine, its x's flags byte (0xF), its
/// animation (a long), its rate (a word: the pixels, then the
/// fraction, it moves down a frame).
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_0F: u32 = 0x0F;
const OBJ_ANIM: u32 = 0x20;
const OBJ_RATE: u32 = 0x28;
const OBJ_RATE_FRACTION: u32 = 0x29;
/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// Cleared by the set-up (`rr_orbit.rs`).
const ORBIT_STATE: u32 = 0xFFAD56;
/// While it is set the animations wait; stepped after the first.
const COUNT_AD57: u32 = 0xFFAD57;
/// `POSITION`'s y (a word; `a_objects.rs`), which `func_0012A0` puts
/// the object at.
const POSITION_Y: u32 = 0xFF9F06;
/// The animations and the routine `func_0012A0` gives the object it
/// sets up.
const OBJ_INIT_ANIMS: u32 = 0xFF80C8;
const OBJ_INIT_2: u32 = 0xFF80D0;
/// The velocity's y (bytes: pixels and fraction; `a_motion.rs`).
const VELOCITY_Y: u32 = 0xFF9EC6;
const VELOCITY_Y_FRACTION: u32 = 0xFF9EC7;
/// The running object (index, address) and the sprite record of the
/// routine being run (index, address).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;

const OBJECT_7: u32 = 7;
/// Routines, animations and the task made.
const ANIMS_00FCF8: u32 = 0x00FCF8;
const ROUTINE_013F58: u32 = 0x013F58;
const ROUTINE_013FA0: u32 = 0x013FA0;
const ROUTINE_013FC0: u32 = 0x013FC0;
const ROUTINE_01401A: u32 = 0x01401A;
const ANIM_E8690: u32 = 0x0E8690;
const ANIM_E8696: u32 = 0x0E8696;
const ANIM_E868A: u32 = 0x0E868A;
const TASK_01481A: u32 = 0x01481A;
const TASK_PRIORITY: u8 = 0x31;
/// The rate's change a frame.
const SLOWING: u32 = 0xF8;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_013F14`: object 7 set up (`ORBIT_STATE` cleared): with
    /// the animations `D_00FCF8` and routine `func_013F58`, at y 0x18
    /// (`func_0012A0`), flags 0x81, byte 0xF 0x21, frame 0xA, rate
    /// 0x200, its sprite placed (`func_001B32`).  Gives back d0, d1, d4,
    /// d5, a4, a5, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_013f14(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        self.write8(ORBIT_STATE, 0);
        let (d4, a4, x) = self.r_001cfa(low_byte(d4, OBJECT_7 as u8));
        self.write16(POSITION_Y, 0x18);
        self.write32(OBJ_INIT_ANIMS, ANIMS_00FCF8);
        self.write32(OBJ_INIT_2, ROUTINE_013F58);
        let (_, d5, a5, _) = self.r_0012a0(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        self.write8(a5 + SPRITE_FLAGS, 0x81);
        self.write8(a4 + OBJ_0F, 0x21);
        self.write8(a5 + SPRITE_FRAME, 0xA);
        self.write16(a4 + OBJ_RATE, 0x200);
        let (d0, d1, x, c) = self.r_001b32(d1, a4, a5);
        (d0, d1, d4, d5, a4, a5, x, c)
    }

    /// `func_013F58`, an object's routine: its rate slowed (`SLOWING`,
    /// `func_004424`); once negative the rate is 0, the Z80 gets 0x1E,
    /// it is put in the middle of its cell and goes on as
    /// `func_013FA0` with the animation `ANIM_E8690`.  It moves down by
    /// its rate (`func_0019DA`) and its sprite is placed
    /// (`func_013F98`).
    pub fn r_013f58(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let negative;
        (r.d0, r.x, negative) = self.r_004424(low_byte(r.d0, SLOWING as u8), r.a4);
        if negative {
            self.write8(r.a4 + OBJ_RATE, 0);
            self.write8(r.a4 + OBJ_RATE_FRACTION, 0);
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_013FA0);
            r.d0 = low_byte(r.d0, 0x1E);
            self.r_0034b6(r.d0);
            self.write32(r.a4 + OBJ_ANIM, ANIM_E8690);
            self.r_001852(r.a4);
        }
        let fraction = self.read8(r.a4 + OBJ_RATE_FRACTION);
        self.write8(VELOCITY_Y_FRACTION, fraction);
        let pixels = self.read8(r.a4 + OBJ_RATE);
        self.write8(VELOCITY_Y, pixels);
        r.c = false;
        (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        (r.d0, r.d1, r.x, r.c) = self.r_013f98(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `.L013F98`: the object's sprite placed (`func_001B32`) and its
    /// depth with its offset (`func_001B50`).  Gives back d0, d1, X
    /// and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_013f98(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        let (d0, x, c) = self.r_001b50(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        (d0, d1, x, c)
    }

    /// The animation run (`func_001336`) unless `COUNT_AD57` is set;
    /// true when it ended.  Otherwise the sprite placed
    /// (`func_013F98`).
    fn rr_drop_animate(&mut self, r: &mut Regs) -> bool {
        if self.read8(COUNT_AD57) == 0 {
            (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
            if r.c {
                return true;
            }
        }
        (r.d0, r.d1, r.x, r.c) = self.r_013f98(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        false
    }

    /// `func_013FA0`, an object's routine: its animation (while
    /// `COUNT_AD57` is 0); at its end `COUNT_AD57` goes up and it goes
    /// on as `func_013FC0` with the animation `ANIM_E8696`.
    pub fn r_013fa0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.rr_drop_animate(r) {
            return;
        }
        let (count, carry) = self.read8(COUNT_AD57).overflowing_add(1);
        self.write8(COUNT_AD57, count);
        r.x = carry;
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_013FC0);
        self.write32(r.a4 + OBJ_ANIM, ANIM_E8696);
        r.c = false;
        self.r_013fc0(r);
    }

    /// `func_013FC0`, an object's routine: its animation (while
    /// `COUNT_AD57` is 0); at its end it goes on as `func_01401A` with
    /// the animation `ANIM_E868A`, the task `func_01481A` (priority
    /// 0x31) is made, `func_0002DA`, `func_014842`, `func_0147B4` and
    /// `func_014886` run, the running object and the routine's sprite
    /// record are taken back, and the Z80 gets 0x12.
    pub fn r_013fc0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.rr_drop_animate(r) {
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01401A);
        self.write32(r.a4 + OBJ_ANIM, ANIM_E868A);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, TASK_PRIORITY, TASK_01481A);
        self.r_0002da();
        r.d6 = self.r_014842(r.d4, r.d5, r.d6);
        (r.d5, r.d6, r.a5) = self.r_0147b4(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d0, r.d1, r.d4, r.d5, r.d6, r.a0, r.a6, r.x) = self.r_014886(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5);
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        r.d0 = low_byte(r.d0, 0x12);
        r.c = false;
        self.r_0034b6(r.d0);
        self.r_01401a(r);
    }
}
