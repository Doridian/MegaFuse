//! The scene of `D_01924E`'s first entry (`func_01C28C`, run by the task
//! of `func_019xxx` after its yield): a list loaded, a task with a
//! sprite (`func_01C366`) and objects 6-9 going through their routines
//! by animation and count:
//!
//! - object 6 comes down (`func_01C380`), makes object 7 and the task
//!   `func_01C9A6`, waits for object 7 (`func_01C420`), then walks
//!   (`func_01C514`, `func_01C592`) until it meets something
//!   (`func_006E7C`) or object 7 is done with it, and goes
//!   (`func_01C620`, `func_01C63E`);
//! - object 7 comes in (`func_01C45A`, `func_01C494`, `func_01C4C4`),
//!   walks by `func_0040C0` (`func_01C64E`, `func_01C6B4`), then
//!   (`func_01C716`) sways (`func_01C75A`, `func_01C780`), leaves to the
//!   left (`func_01C800`) through the jump `func_004E5C` twice
//!   (`func_01C8AC`, `func_01C8EA`) and after a count makes object 9
//!   (`func_01C91A`), which makes the task `func_019262` at its
//!   animation's end (`func_01C982`);
//! - the task `func_01C9A6` makes object 8 every 0x3C0 frames, which
//!   goes after object 6 (`func_01CA4E`, `func_01CAD2`) until it is
//!   near or object 6's `OBJ_11` is 0.

use crate::{Game, Regs};

/// `func_01C294`'s frames set (`func_000776`, set 6) and list (for
/// `func_0094D8` through `LOAD_LIST`; `D_01C356`, in the ROM).
const FRAMES: u32 = 0x0DE38E;
const FRAMES_SET: u32 = 6;
const LOAD_LIST: u32 = 0xFF92FA;
const LIST: u32 = 0x01C356;
/// A long kept around the load (`func_0094D8` changes it).
const KEPT_5810: u32 = 0xFF5810;

/// `func_01B684`'s arguments: the task's position (words, x then y; the
/// y's low byte apart), its sprite's flags (the low byte of a word) and
/// its animation (a long).  `func_0012A0` takes the second and fourth
/// longs: an object's animations and routine.
const ARG_X: u32 = 0xFF80C4;
const ARG_Y: u32 = 0xFF80C8;
const ARG_Y_LOW: u32 = 0xFF80C9;
const ARG_FLAGS: u32 = 0xFF80CC;
const ARG_ANIM: u32 = 0xFF80D0;
const OBJ_INIT_ANIMS: u32 = 0xFF80C8;
const OBJ_INIT_ROUTINE: u32 = 0xFF80D0;
/// The count `func_01C9A6` keeps (a word, also its task's `TASK_COUNT`).
const COUNT: u32 = 0xFF80C4;
/// A byte compared with across objects (`func_01CA4E`).
const OTHER: u32 = 0xFF80C5;
/// The position `func_0012A0` puts an object at (words).
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
/// The velocity (words; `a_motion.rs`) and speed (`func_001620`,
/// `func_00112C`).
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;
const SPEED: u32 = 0xFF9F00;
/// `func_00112C`'s points (`math.rs`).
const FROM_X: u32 = 0xFF9EFA;
const FROM_Y: u32 = 0xFF9EFE;
const TO_X: u32 = 0xFF9EE6;
const TO_Y: u32 = 0xFF9EE8;
/// The speed `func_00398C` moves at (a word; `c_chase.rs`).
const MOVE_SPEED: u32 = 0xFFA960;

/// The scene's bytes: a state (0 at the start, 3 once object 7 is
/// made, 2 once object 6 goes; `func_01C366` ends when it isn't 0;
/// `func_0040C0` takes it), a count of object 6's steps (a sound at
/// `STEP_SOUND_AT`, cleared when its animation ends), object 7's count
/// before object 9 (`func_01C91A`).
const STATE: u32 = 0xFFAEDC;
const STEPS: u32 = 0xFFAEDA;
const WAIT: u32 = 0xFFAEDB;
/// Set to 1 at the start, cleared by `func_01C716`.
const BYTE_80DF: u32 = 0xFF80DF;
/// Cleared by `func_01C716`.
const BYTE_954F: u32 = 0xFF954F;
/// Flags: set to 0x80 by `func_01C514`, bit 6 set by `func_01C716`.
const FLAGS: u32 = 0xFF9540;
/// Its bit 0 cleared by `func_01C514`, set by `func_01C716`.
const SPARED: u32 = 0xFF9548;

/// An object's fields: its kind (0 free), routine, the routine after
/// `func_004E5C` (a long), its sprite record (address, index), a byte
/// `func_0012B6` takes, a count `func_01C9A6` goes by (8, 4, 0), a
/// byte, its direction (`func_001620`), the pick by direction and the
/// kind of its animations (`func_00130E`), its position (words; the
/// low bytes the pixels), its animations, the animation's counters,
/// two counts, a rate and an offset (`a_motion.rs`).
const OBJ_KIND: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_SAVED: u32 = 0x06;
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_0F: u32 = 0x0F;
const OBJ_11: u32 = 0x11;
const OBJ_12: u32 = 0x12;
const OBJ_DIRECTION: u32 = 0x13;
const OBJ_FACING: u32 = 0x14;
const OBJ_ANIM_KIND: u32 = 0x15;
const OBJ_X: u32 = 0x18;
const OBJ_X_LOW: u32 = 0x19;
const OBJ_Y: u32 = 0x1A;
const OBJ_Y_LOW: u32 = 0x1B;
const OBJ_ANIMS: u32 = 0x1C;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_26: u32 = 0x26;
const OBJ_27: u32 = 0x27;
const OBJ_RATE: u32 = 0x28;
const OBJ_OFFSET: u32 = 0x2A;
/// A task's count (a word) and a byte after it.
const TASK_COUNT: u32 = 0x18;
const TASK_19: u32 = 0x19;
/// A sprite record's frame, flags and the low bytes of its position.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y_LOW: u32 = 0x0B;

/// The objects (`func_001CFA`).
const OBJ6: u32 = 6;
const OBJ7: u32 = 7;
const OBJ8: u32 = 8;
const OBJ9: u32 = 9;

/// The task with a sprite `func_01C294` makes: routine, priority,
/// depth, position, flags, animation.
const SPRITE_TASK: u32 = 0x01C366;
const SPRITE_TASK_PRIORITY: u8 = 0x32;
const SPRITE_TASK_DEPTH: u8 = 0xE6;
const SPRITE_TASK_ANIM: u32 = 0x0DFCA2;
/// Object 6's animations and first routine, position.
const OBJ6_ANIMS: u32 = 0x01CB10;
const OBJ6_DOWN: u32 = 0x01C380;
/// Object 6: waits for object 7, its walk's animations, the walk, the
/// last two.
const OBJ6_WAIT: u32 = 0x01C420;
const OBJ6_START: u32 = 0x01C514;
const OBJ6_WALK_ANIMS: u32 = 0x01CB54;
const OBJ6_WALK: u32 = 0x01C592;
const OBJ6_GOING: u32 = 0x01C620;
const OBJ6_GONE: u32 = 0x01C63E;
/// Object 7: its animations, routines.
const OBJ7_ANIMS: u32 = 0x01CB18;
const OBJ7_IN: u32 = 0x01C45A;
const OBJ7_ON: u32 = 0x01C494;
const OBJ7_LAND: u32 = 0x01C4C4;
const OBJ7_WALK_ANIMS: u32 = 0x01CB24;
const OBJ7_PICK: u32 = 0x01C64E;
const OBJ7_WALK: u32 = 0x01C6B4;
const OBJ7_DONE: u32 = 0x01C75A;
const OBJ7_SWAY: u32 = 0x01C780;
/// How long each of `func_01C780`'s sways lasts (`D_01C7FA`, bytes in
/// the ROM by `OBJ_27`).
const SWAYS: u32 = 0x01C7FA;
const SWAY_COUNT: u8 = 5;
const OBJ7_LEAVE: u32 = 0x01C800;
const OBJ7_JUMP2: u32 = 0x01C8AC;
const OBJ7_LANDED: u32 = 0x01C8EA;
const OBJ7_LAST: u32 = 0x01C91A;
/// The jump (`func_004E5C`), its routine after in `OBJ_SAVED`.
const JUMP: u32 = 0x004E5C;
/// The task with a sprite `func_01C800` makes.
const LEAVE_TASK: u32 = 0x01C908;
const LEAVE_TASK_PRIORITY: u8 = 0x64;
const LEAVE_TASK_DEPTH: u8 = 0x14;
const LEAVE_TASK_ANIM: u32 = 0x0DFC3C;
/// Object 9's animations and routine; the task it makes.
const OBJ9_ANIMS: u32 = 0x01CB4C;
const OBJ9_ROUTINE: u32 = 0x01C982;
const AFTER_TASK: u32 = 0x019262;
const AFTER_PRIORITY: u8 = 0x50;
/// The task making object 8 and its period; object 8's animations and
/// routine.
const MAKER: u32 = 0x01C9A6;
const MAKER_PRIORITY: u8 = 0x50;
const MAKE_EVERY: u16 = 0x3C0;
const OBJ8_ANIMS: u32 = 0x01CB6C;
const OBJ8_ROUTINE: u32 = 0x01CA4E;
/// How near object 8 gets (pixels each way).
const NEAR: u8 = 8;

/// Sounds (`func_0034B6`).
const START_SOUND: u8 = 0x0F;
const STEP_SOUND: u8 = 0x62;
const STEP_SOUND_AT: u8 = 0x0A;
const GOING_SOUND: u8 = 0x82;
const LEAVE_SOUND: u8 = 0x5E;
const MAKE_SOUND: u8 = 0x61;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// The object's animation a frame on (`func_001336`): true (and C)
    /// at its end.  Z is left alone.
    fn zc_animate(&mut self, r: &mut Regs) -> bool {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        r.c
    }

    /// The sprite placed for the object (`func_001B32`): d0, d1, X, C.
    fn zc_place(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
    }

    /// Object `index` to a4 (`func_001CFA`).
    fn zc_object(&mut self, r: &mut Regs, index: u32) {
        (r.d4, r.a4, r.x) = self.r_001cfa(index);
    }

    /// A byte of the object counted up (`addq`, X its carry) and
    /// compared with `limit` (C below): true when it is there.
    fn zc_count(&mut self, r: &mut Regs, field: u32, limit: u8) -> bool {
        let (n, carry) = self.read8(r.a4 + field).overflowing_add(1);
        self.write8(r.a4 + field, n);
        r.x = carry;
        r.c = n < limit;
        n == limit
    }

    /// An object made at a4 (`func_0012A0`): `OBJ_0F`, its sprite's flags.
    fn zc_make(&mut self, r: &mut Regs, byte_0f: u8) {
        (r.d0, r.d5, r.a5, r.x) = self.r_0012a0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
        self.write8(r.a4 + OBJ_0F, byte_0f);
        self.write8(r.a5 + SPRITE_FLAGS, 0x86);
    }

    /// The object's animation by its facing, from its start
    /// (`func_01C89E`).
    fn zc_new_anim(&mut self, r: &mut Regs) {
        (r.a0, r.x, _) = self.r_01c89e(r.d0, r.a4);
    }

    /// `func_01C28C`, `D_01C356`'s first entry: `func_01C294`.
    pub fn r_01c28c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_01c294(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
    }

    /// `func_01C294`: the scene set up: frames set 6, the list `LIST`
    /// loaded (`func_0094D8`, `KEPT_5810` kept around it; it may run for
    /// frames), `func_0094B2`, `STATE` 0, the task `SPRITE_TASK` with
    /// its sprite (`func_001222`, `func_01B684`), object 6 made at
    /// (0x88, -0x20) going down (`OBJ6_DOWN`), `OBJ_11` 8; the sound
    /// `START_SOUND`, `STEPS` 0 and `BYTE_80DF` 1.  Gives back every
    /// register, X and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01c294(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let d0 = self.r_000776(FRAMES_SET, FRAMES, a1);
        let kept = self.read32(KEPT_5810);
        self.write32(LOAD_LIST, LIST);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c) =
            self.r_0094d8(d0, d1, d2, d3, d4, d5, d6, d7, FRAMES, a1, a2, a3, a4, a5, a6, x);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c);
        }
        self.write32(KEPT_5810, kept);
        self.r_0094b2();
        self.write8(STATE, 0);
        let (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, SPRITE_TASK_DEPTH, SPRITE_TASK_PRIORITY, SPRITE_TASK);
        self.write16(ARG_X, 0x88);
        self.write16(ARG_Y, 0x37);
        self.write16(ARG_FLAGS, 0x186);
        self.write32(ARG_ANIM, SPRITE_TASK_ANIM);
        let (d0, _, _) = self.r_01b684(a5, a6);
        self.write16(POSITION_X, 0x88);
        self.write16(POSITION_Y, 0xFFE0);
        self.write32(OBJ_INIT_ANIMS, OBJ6_ANIMS);
        self.write32(OBJ_INIT_ROUTINE, OBJ6_DOWN);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        self.zc_object(&mut r, OBJ6);
        self.zc_make(&mut r, 0x31);
        self.write8(r.a4 + OBJ_FACING, 0);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.zc_animate(&mut r);
        self.write8(r.a4 + OBJ_26, 0);
        self.write8(r.a4 + OBJ_11, 8);
        r.d0 = low_byte(r.d0, START_SOUND);
        self.r_0034b6(r.d0);
        self.write8(STEPS, 0);
        self.write8(BYTE_80DF, 1);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, false)
    }

    /// `func_01C366`, `SPRITE_TASK`: animated and placed
    /// (`func_0013D6`, `func_001B7E`); once `STATE` isn't 0 its sprite
    /// is freed and it ends.  C clear while it waits.
    pub fn r_01c366(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        (r.d0, r.x, _) = self.r_001b7e(r.a5, r.a6);
        if self.read8(STATE) == 0 {
            r.c = false;
            return;
        }
        self.r_000890();
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }

    /// `func_01C380`, `OBJ6_DOWN`: object 6 moves down (y velocity
    /// 0x100) for 0x57 frames (`OBJ_26`); then its frame 0x3C, its
    /// routine `OBJ6_WAIT`, and object 7 is made at (0x98, 0x22) with
    /// `OBJ7_ANIMS` and `OBJ7_IN` (`OBJ_RATE` -0x100, `OBJ_11` 4), the
    /// task `MAKER`, `STATE` 3.  a4 is object 7 then.
    pub fn r_01c380(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_animate(r);
        self.write16(VELOCITY_Y, 0x100);
        (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        self.zc_place(r);
        if !self.zc_count(r, OBJ_26, 0x57) {
            return;
        }
        self.write8(r.a5 + SPRITE_FRAME, 0x3C);
        self.write32(r.a4 + OBJ_ROUTINE, OBJ6_WAIT);
        self.write16(POSITION_X, 0x98);
        self.write16(POSITION_Y, 0x22);
        self.write32(OBJ_INIT_ANIMS, OBJ7_ANIMS);
        self.write32(OBJ_INIT_ROUTINE, OBJ7_IN);
        self.zc_object(r, OBJ7);
        self.zc_make(r, 0x11);
        self.write16(r.a4 + OBJ_RATE, 0xFF00);
        self.write8(r.a4 + OBJ_26, 0);
        self.write8(r.a4 + OBJ_27, 0);
        self.write8(r.a4 + OBJ_11, 4);
        self.write8(r.a4 + OBJ_FACING, 0);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.zc_animate(r);
        (r.d6, r.a6, r.x, _) = self.r_0003e8(r.d4, r.d5, r.d6, MAKER_PRIORITY, MAKER);
        self.write8(r.a6 + TASK_COUNT, 0);
        self.write8(r.a6 + TASK_19, 0);
        self.write8(STATE, 3);
        r.c = false;
    }

    /// `func_01C41E`: a bare `rts` (`func_01C380`'s way out).
    pub fn r_01c41e(&mut self) {}

    /// `func_01C420`, `OBJ6_WAIT`: once object 7's `OBJ_27` isn't 0 it
    /// is cleared (with its `OBJ_26` and `OBJ_OFFSET`) and object 6 goes
    /// on as `OBJ6_START`, its facing one on, a new animation; animated
    /// and placed.  C clear while it waits (a4 object 7).
    pub fn r_01c420(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_object(r, OBJ7);
        if self.read8(r.a4 + OBJ_27) == 0 {
            r.c = false;
            return;
        }
        self.write8(r.a4 + OBJ_26, 0);
        self.write8(r.a4 + OBJ_27, 0);
        self.write16(r.a4 + OBJ_OFFSET, 0);
        self.zc_object(r, OBJ6);
        self.write32(r.a4 + OBJ_ROUTINE, OBJ6_START);
        self.zc_turn(r);
        self.zc_animate(r);
        self.zc_place(r);
    }

    /// The object's facing one on and its new animation.
    fn zc_turn(&mut self, r: &mut Regs) {
        let facing = self.read8(r.a4 + OBJ_FACING).wrapping_add(1);
        self.write8(r.a4 + OBJ_FACING, facing);
        self.zc_new_anim(r);
    }

    /// `func_01C45A`, `OBJ7_IN`: animated, falling (`func_001AF6`,
    /// `func_001ABC`) and moving right (0x80), placed; after 0x1D frames
    /// (`OBJ_26`) its facing one on, a new animation and `OBJ7_ON`.
    pub fn r_01c45a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_animate(r);
        r.d0 = self.r_001af6(r.a4);
        (r.d0, r.x, _) = self.r_001abc(r.d0, r.a4);
        self.write16(VELOCITY_X, 0x80);
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
        self.zc_place(r);
        if !self.zc_count(r, OBJ_26, 0x1D) {
            return;
        }
        self.zc_turn(r);
        self.write32(r.a4 + OBJ_ROUTINE, OBJ7_ON);
        r.c = false;
    }

    /// `func_01C494`, `OBJ7_ON`: animated, moving right (0x100), placed;
    /// once its sprite's x is 0xA0, its facing one on, a new animation
    /// and `OBJ7_LAND`.  C: the x below 0xA0, else clear.
    pub fn r_01c494(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_animate(r);
        self.write16(VELOCITY_X, 0x100);
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
        self.zc_place(r);
        let x = self.read8(r.a5 + SPRITE_X_LOW);
        r.c = x < 0xA0;
        if x != 0xA0 {
            return;
        }
        self.zc_turn(r);
        self.write32(r.a4 + OBJ_ROUTINE, OBJ7_LAND);
        r.c = false;
    }

    /// `func_01C4C4`, `OBJ7_LAND`: at its animation's end its offset
    /// goes into its y (`OBJ_OFFSET`'s high byte added, then 0), its
    /// animations `OBJ7_WALK_ANIMS` from facing 0, routine `OBJ7_PICK`,
    /// `OBJ_26` 0xB, `OBJ_SAVED`'s first byte 0 and `OBJ_27` 2 (object 6
    /// waits for it).  Placed.
    pub fn r_01c4c4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.zc_animate(r) {
            let offset = self.read8(r.a4 + OBJ_OFFSET);
            r.d0 = low_byte(r.d0, offset);
            let y = self.read8(r.a4 + OBJ_Y_LOW).wrapping_add(offset);
            self.write8(r.a4 + OBJ_Y_LOW, y);
            self.write8(r.a4 + OBJ_OFFSET, 0);
            self.write32(r.a4 + OBJ_ANIMS, OBJ7_WALK_ANIMS);
            self.write8(r.a4 + OBJ_FACING, 0);
            (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
            self.write8(r.a4 + OBJ_TIMER, 0);
            self.write8(r.a4 + OBJ_STEP, 0);
            self.write32(r.a4 + OBJ_ROUTINE, OBJ7_PICK);
            self.write8(r.a4 + OBJ_26, 0xB);
            self.write8(r.a4 + OBJ_SAVED, 0);
            self.write8(r.a4 + OBJ_27, 2);
        }
        self.zc_place(r);
    }

    /// `func_01C514`, `OBJ6_START`: at its animation's end on as
    /// `OBJ6_WALK` (`OBJ_26` 3, `OBJ_SAVED`'s first byte 0, `OBJ_27` 2)
    /// with `OBJ6_WALK_ANIMS` from facing 0, `FLAGS` 0x80 and `SPARED`'s
    /// bit 0 cleared.  Placed.
    pub fn r_01c514(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.zc_animate(r) {
            self.write32(r.a4 + OBJ_ROUTINE, OBJ6_WALK);
            self.write8(r.a4 + OBJ_26, 3);
            self.write8(r.a4 + OBJ_SAVED, 0);
            self.write8(r.a4 + OBJ_27, 2);
            self.write32(r.a4 + OBJ_ANIMS, OBJ6_WALK_ANIMS);
            self.write8(r.a4 + OBJ_FACING, 0);
            self.zc_new_anim(r);
            self.write8(FLAGS, 0x80);
            let spared = self.read8(SPARED);
            self.write8(SPARED, spared & 0xFE);
        }
        self.zc_place(r);
    }

    /// `func_01C558`: the object's facing from its direction (bit 7 of
    /// the facing kept; none when the direction's bit 7 is set), its
    /// animation (`func_00130E`) a frame on, placed and its depth
    /// (`func_001B50`).  The interrupt mask is kept around the last two
    /// (an SR saved and restored).  Gives back d0, d1, d2, d5, a0, a5
    /// and C: not Z of `func_001336`.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01c558(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32) -> (u32, u32, u32, u32, u32, u32, bool) {
        let direction = self.read8(a4 + OBJ_DIRECTION);
        let d1 = low_byte(d1, direction);
        let mut d0 = d0;
        if direction & 0x80 == 0 {
            let facing = (self.read8(a4 + OBJ_FACING) & 0x80) | direction;
            d0 = low_byte(d0, facing);
            self.write8(a4 + OBJ_FACING, facing);
        }
        let (_, x, _) = self.r_00130e(d0, a4);
        let (_, d1, d2, d5, a0, a5, _, z, _) = self.r_001336(d0, d1, d2, d5, a4, x);
        let ipl = self.ipl();
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        let (d0, _, _) = self.r_001b50(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        self.set_ipl(ipl);
        (d0, d1, d2, d5, a0, a5, !z)
    }

    /// `func_01C592`, `OBJ6_WALK`: `STEPS` counted (the sound
    /// `STEP_SOUND` at `STEP_SOUND_AT`; back to 0 when `func_01C558`
    /// sets C), the walk toward object 0 (`func_003E44`, `func_00398C`
    /// at speed 0xCC), `func_0015E6`.  While object 7's `OBJ_11` isn't 0
    /// and object 6 meets nothing (`func_006E7C`) that's all; else object
    /// 6 goes: `OBJ6_GOING`, facing 0 of kind 4, a new animation,
    /// `STATE` 2, `OBJ_12` 0, the sound `GOING_SOUND` (d5 and a5 its
    /// sprite record).  C clear.
    pub fn r_01c592(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let steps = self.read8(STEPS).wrapping_add(1);
        self.write8(STEPS, steps);
        if steps == STEP_SOUND_AT {
            r.d0 = low_byte(r.d0, STEP_SOUND);
            self.r_0034b6(r.d0);
        }
        r.d0 = self.r_003e44();
        self.write16(MOVE_SPEED, 0xCC);
        (r.d0, r.d1, r.d4, r.a4) = self.r_00398c(r.d0, r.d1, r.d4, r.a4, r.a5);
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, ended) = self.r_01c558(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4);
        if ended {
            self.write8(STEPS, 0);
        }
        (r.d0, r.x) = self.r_0015e6(r.a4);
        self.zc_object(r, OBJ7);
        if self.read8(r.a4 + OBJ_11) != 0 {
            self.zc_object(r, OBJ6);
            (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e7c(r.d1, r.d2, r.a0, r.a4, r.a5);
            if !r.c {
                return;
            }
        }
        self.zc_object(r, OBJ6);
        self.write32(r.a4 + OBJ_ROUTINE, OBJ6_GOING);
        self.write8(r.a4 + OBJ_FACING, 0);
        self.write8(r.a4 + OBJ_ANIM_KIND, 4);
        self.zc_new_anim(r);
        self.write8(STATE, 2);
        self.write8(r.a4 + OBJ_12, 0);
        r.d5 = low_byte(r.d5, self.read8(r.a4 + OBJ_SPRITE));
        r.a5 = self.read32(r.a4 + OBJ_SPRITE_ADDR);
        r.d0 = low_byte(r.d0, GOING_SOUND);
        self.r_0034b6(r.d0);
        r.c = false;
    }

    /// `func_01C620`, `OBJ6_GOING`: at its animation's end its facing
    /// one on (`func_00130E`) and `OBJ6_GONE`.  Placed (`func_01C638`).
    pub fn r_01c620(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.zc_animate(r) {
            let facing = self.read8(r.a4 + OBJ_FACING).wrapping_add(1);
            self.write8(r.a4 + OBJ_FACING, facing);
            (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
            self.write32(r.a4 + OBJ_ROUTINE, OBJ6_GONE);
        }
        (r.d0, r.d1, r.x, r.c) = self.r_01c638(r.d1, r.a4, r.a5);
    }

    /// `func_01C638`: `func_001B32` (the object's sprite placed).  Gives
    /// back d0, d1, X and C.
    pub fn r_01c638(&mut self, d1: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        self.r_001b32(d1, a4, a5)
    }

    /// `func_01C63E`, `OBJ6_GONE`: at its animation's end the sprite
    /// freed and the object (kind 0); else placed.
    pub fn r_01c63e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.zc_animate(r) {
            (r.d0, r.d1, r.x, r.c) = self.r_01c638(r.d1, r.a4, r.a5);
            return;
        }
        self.r_000890();
        self.write8(r.a4 + OBJ_KIND, 0);
        r.c = false;
    }

    /// `func_01C64E`, `OBJ7_PICK`: a direction from `func_0040C0` (with
    /// `STATE`) into `OBJ_DIRECTION` and `OBJ_FACING`; if it gave one
    /// (C, not N) the object walks: `OBJ_ANIM_KIND` 0, `OBJ7_WALK` (this
    /// routine kept in `OBJ_SAVED`), the counts and the animation's 0.
    /// Else kind 0xB facing 0.  Then `func_01C558`, and if it meets
    /// something (`func_006E7C`) `func_01C716`.
    pub fn r_01c64e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let state = self.read8(STATE);
        let (negative, picked);
        (r.d0, r.d1, negative, picked) = self.r_0040c0(low_byte(r.d0, state), r.d1, r.a0, r.a4, r.a5);
        self.write8(r.a4 + OBJ_DIRECTION, r.d0 as u8);
        self.write8(r.a4 + OBJ_FACING, r.d0 as u8);
        if !negative && picked {
            self.write8(r.a4 + OBJ_ANIM_KIND, 0);
            let routine = self.read32(r.a4 + OBJ_ROUTINE);
            self.write32(r.a4 + OBJ_SAVED, routine);
            self.write32(r.a4 + OBJ_ROUTINE, OBJ7_WALK);
            self.write8(r.a4 + OBJ_26, 0);
            self.write8(r.a4 + OBJ_27, 0);
            self.write8(r.a4 + OBJ_TIMER, 0);
            self.write8(r.a4 + OBJ_STEP, 0);
        } else {
            self.write8(r.a4 + OBJ_ANIM_KIND, 0xB);
            self.write8(r.a4 + OBJ_DIRECTION, 0);
            self.write8(r.a4 + OBJ_FACING, 0);
        }
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, _) = self.r_01c558(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e7c(r.d1, r.d2, r.a0, r.a4, r.a5);
        if r.c {
            (r.d4, r.a0, r.a4, r.x, r.c) = self.r_01c716(r.d0);
        }
    }

    /// `func_01C6B4`, `OBJ7_WALK`: `func_01C558`; whether it met
    /// something (`func_0015E6`, `func_006E7C`) kept in `OBJ_27` (1);
    /// moved in its direction at 0x133 (`func_001620`, `func_0019C0`,
    /// `func_0019DA`), placed; after 0xD frames (`OBJ_26`) put in its
    /// cell's middle (`func_001852`) and `func_01C716` if it met
    /// something, else back to `OBJ_SAVED`'s routine.
    pub fn r_01c6b4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.c) = self.r_01c558(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4);
        if self.read8(r.a4 + OBJ_27) == 0 {
            (r.d0, r.x) = self.r_0015e6(r.a4);
            (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e7c(r.d1, r.d2, r.a0, r.a4, r.a5);
            if r.c {
                self.write8(r.a4 + OBJ_27, 1);
            }
        }
        self.write16(SPEED, 0x133);
        r.d0 = low_byte(r.d0, self.read8(r.a4 + OBJ_DIRECTION));
        self.r_001620(r.d0);
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
        (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        self.zc_place(r);
        if !self.zc_count(r, OBJ_26, 0xD) {
            return;
        }
        self.r_001852(r.a4);
        if self.read8(r.a4 + OBJ_27) == 0 {
            let routine = self.read32(r.a4 + OBJ_SAVED);
            self.write32(r.a4 + OBJ_ROUTINE, routine);
            r.c = false;
            return;
        }
        (r.d4, r.a0, r.a4, r.x, r.c) = self.r_01c716(r.d0);
    }

    /// `func_01C716`: object 7 done: `OBJ7_DONE`, facing 0 of kind 4, a
    /// new animation; `BYTE_954F` 0, object 6's `OBJ_11` 0, `SPARED`'s
    /// bit 0 set, `BYTE_80DF` 0 and `FLAGS` bit 6 set.  Gives back d4
    /// and a4 (object 7), a0, X and C (clear).
    pub fn r_01c716(&mut self, d0: u32) -> (u32, u32, u32, bool, bool) {
        let (_, obj7, _) = self.r_001cfa(OBJ7);
        self.write32(obj7 + OBJ_ROUTINE, OBJ7_DONE);
        self.write8(obj7 + OBJ_FACING, 0);
        self.write8(obj7 + OBJ_ANIM_KIND, 4);
        let (a0, _, _) = self.r_01c89e(d0, obj7);
        self.write8(BYTE_954F, 0);
        let (_, obj6, _) = self.r_001cfa(OBJ6);
        self.write8(obj6 + OBJ_11, 0);
        let (d4, a4, x) = self.r_001cfa(OBJ7);
        let spared = self.read8(SPARED);
        self.write8(SPARED, spared | 1);
        self.write8(BYTE_80DF, 0);
        let flags = self.read8(FLAGS);
        self.write8(FLAGS, flags | 0x40);
        (d4, a0, a4, x, false)
    }

    /// `func_01C75A`, `OBJ7_DONE`: at its animation's end its facing
    /// one on, a new animation, the counts 0 and `OBJ7_SWAY`.  Placed.
    pub fn r_01c75a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.zc_animate(r) {
            self.zc_turn(r);
            self.write8(r.a4 + OBJ_26, 0);
            self.write8(r.a4 + OBJ_27, 0);
            self.write32(r.a4 + OBJ_ROUTINE, OBJ7_SWAY);
        }
        (r.d0, r.d1, r.x, r.c) = self.r_01c638(r.d1, r.a4, r.a5);
    }

    /// `func_01C780`, `OBJ7_SWAY`: animated, moving right (0x300, facing
    /// 2) on even sways (`OBJ_27`), left (-0x300, facing 1) on odd ones,
    /// placed; each sway lasts `SWAYS`[`OBJ_27`] frames (`OBJ_26`), and
    /// after `SWAY_COUNT` it goes on as `OBJ7_LEAVE`, facing 1, a new
    /// animation.  Gives back d0 (the count) and d1 (the sway, a word).
    pub fn r_01c780(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_animate(r);
        if self.read8(r.a4 + OBJ_27) & 1 != 0 {
            self.write16(VELOCITY_X, 0xFD00);
            self.write8(r.a4 + OBJ_FACING, 1);
        } else {
            self.write16(VELOCITY_X, 0x300);
            self.write8(r.a4 + OBJ_FACING, 2);
        }
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.zc_place(r);
        let (count, carry) = self.read8(r.a4 + OBJ_26).overflowing_add(1);
        self.write8(r.a4 + OBJ_26, count);
        r.x = carry;
        r.d0 = low_byte(r.d0, count);
        let sway = self.read8(r.a4 + OBJ_27) as i8 as i16;
        r.d1 = low_word(r.d1, sway as u16);
        let frames = self.read8(SWAYS.wrapping_add(sway as i32 as u32));
        r.c = count < frames;
        if count != frames {
            return;
        }
        self.zc_object(r, OBJ7);
        self.write8(r.a4 + OBJ_26, 0);
        if !self.zc_count(r, OBJ_27, SWAY_COUNT) {
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, OBJ7_LEAVE);
        self.write8(r.a4 + OBJ_FACING, 1);
        (r.a0, r.x, r.c) = self.r_01c89e(r.d0, r.a4);
    }

    /// `func_01C7F8`: a bare `rts` (`func_01C780`'s way out).
    pub fn r_01c7f8(&mut self) {}

    /// `func_01C800`, `OBJ7_LEAVE`: animated, moving left (-0x300),
    /// placed; once its x is below 0x20 it jumps (`JUMP`, then
    /// `OBJ7_JUMP2`; `func_004FD0`, rate -0x300, `OBJ_26` a word 0xC0,
    /// facing 1 of kind 7), and the task `LEAVE_TASK` is made with its
    /// sprite at (0x18, the object's sprite's y less 0x28); the sound
    /// `LEAVE_SOUND`.  C: the x below 0x20 (clear after it jumps).
    pub fn r_01c800(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_animate(r);
        self.write16(VELOCITY_X, 0xFD00);
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
        self.zc_place(r);
        r.c = self.read8(r.a4 + OBJ_X_LOW) < 0x20;
        if !r.c {
            return;
        }
        self.zc_jump(r, OBJ7_JUMP2, 0xFD00, 0xC0);
        self.zc_animate(r);
        let y = self.read8(r.a5 + SPRITE_Y_LOW).wrapping_sub(0x28);
        r.d0 = low_byte(r.d0, y);
        self.write8(ARG_Y_LOW, y);
        (r.d5, r.d6, r.a5, r.a6, r.x, _) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, LEAVE_TASK_DEPTH, LEAVE_TASK_PRIORITY, LEAVE_TASK);
        self.write16(ARG_X, 0x18);
        self.write8(ARG_Y, 0);
        self.write16(ARG_FLAGS, 0x1186);
        self.write32(ARG_ANIM, LEAVE_TASK_ANIM);
        (r.d0, r.x, _) = self.r_01b684(r.a5, r.a6);
        r.d0 = low_byte(r.d0, LEAVE_SOUND);
        self.r_0034b6(r.d0);
        r.c = false;
    }

    /// The object jumps (`JUMP`, `func_004FD0`), on as `next` after:
    /// `rate` (a word), `OBJ_26` (a word) `count`, facing 1 of kind 7,
    /// a new animation.
    fn zc_jump(&mut self, r: &mut Regs, next: u32, rate: u16, count: u16) {
        self.write32(r.a4 + OBJ_SAVED, next);
        self.write32(r.a4 + OBJ_ROUTINE, JUMP);
        (r.d0, r.x, _) = self.r_004fd0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.write16(r.a4 + OBJ_RATE, rate);
        self.write16(r.a4 + OBJ_26, count);
        self.write8(r.a4 + OBJ_FACING, 1);
        self.write8(r.a4 + OBJ_ANIM_KIND, 7);
        self.zc_new_anim(r);
    }

    /// `func_01C89E`: the object's animation by its facing
    /// (`func_00130E`) from its start.  Gives back a0, X and C (clear).
    pub fn r_01c89e(&mut self, d0: u32, a4: u32) -> (u32, bool, bool) {
        let (a0, x, _) = self.r_00130e(d0, a4);
        self.write8(a4 + OBJ_TIMER, 0);
        self.write8(a4 + OBJ_STEP, 0);
        (a0, x, false)
    }

    /// `func_01C8AC`, `OBJ7_JUMP2` (after the first jump): jumps again
    /// (rate -0x150, `OBJ_26` 0x80) to `OBJ7_LANDED`, animated.
    pub fn r_01c8ac(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_jump(r, OBJ7_LANDED, 0xFEB0, 0x80);
        self.zc_animate(r);
    }

    /// `func_01C8EA`, `OBJ7_LANDED`: animated, then facing 0 of kind 9,
    /// a new animation, `OBJ7_LAST` with `WAIT` 0.  C clear.
    pub fn r_01c8ea(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_animate(r);
        self.write8(r.a4 + OBJ_ANIM_KIND, 9);
        self.write8(r.a4 + OBJ_FACING, 0);
        self.zc_new_anim(r);
        self.write32(r.a4 + OBJ_ROUTINE, OBJ7_LAST);
        self.write8(WAIT, 0);
        r.c = false;
    }

    /// `func_01C908`, `LEAVE_TASK`: animated (`func_0013D6`); at the
    /// animation's end the sprite freed and the task ended.
    pub fn r_01c908(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            self.r_000890();
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
        }
    }

    /// `func_01C91A`, `OBJ7_LAST`: at its animation's end the sprite
    /// freed and the object (kind 0).  Else `WAIT` counted: at 0x8D
    /// object 9 is made at object 7's position with `OBJ9_ANIMS` and
    /// `OBJ9_ROUTINE`, facing 0; the sound `GOING_SOUND`.  C: `WAIT`
    /// below 0x8D (clear when it is made).
    pub fn r_01c91a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.zc_animate(r) {
            self.r_000890();
            self.write8(r.a4 + OBJ_KIND, 0);
            r.c = false;
            return;
        }
        let (wait, carry) = self.read8(WAIT).overflowing_add(1);
        self.write8(WAIT, wait);
        r.x = carry;
        r.c = wait < 0x8D;
        if wait != 0x8D {
            return;
        }
        let x = self.read16(r.a4 + OBJ_X);
        self.write16(POSITION_X, x);
        let y = self.read16(r.a4 + OBJ_Y);
        self.write16(POSITION_Y, y);
        self.write32(OBJ_INIT_ANIMS, OBJ9_ANIMS);
        self.write32(OBJ_INIT_ROUTINE, OBJ9_ROUTINE);
        self.zc_object(r, OBJ9);
        self.zc_make(r, 0x31);
        self.write8(r.a4 + OBJ_FACING, 0);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.zc_animate(r);
        r.d0 = low_byte(r.d0, GOING_SOUND);
        self.r_0034b6(r.d0);
        r.c = false;
    }

    /// `func_01C982`, `OBJ9_ROUTINE`: at its animation's end the sprite
    /// freed, the object (kind 0), and the task `AFTER_TASK`.
    pub fn r_01c982(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.zc_animate(r) {
            return;
        }
        self.r_000890();
        self.write8(r.a4 + OBJ_KIND, 0);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, AFTER_PRIORITY, AFTER_TASK);
    }

    /// `func_01C9A6`, `MAKER`: ends once object 6's `OBJ_11` is 0.
    /// Counts (`TASK_COUNT`, a word, copied to `COUNT`); every
    /// `MAKE_EVERY` frames, if object 8 is free, it is made at object
    /// 7's position (y 0x40 on) with `OBJ8_ANIMS` and `OBJ8_ROUTINE`,
    /// frame 0, facing 0 of kind 0, offset 0xC0; animated and placed;
    /// the sound `MAKE_SOUND`.  C: the count below `MAKE_EVERY`, else
    /// clear.
    pub fn r_01c9a6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_object(r, OBJ6);
        if self.read8(r.a4 + OBJ_11) == 0 {
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
            return;
        }
        let (count, carry) = self.read16(r.a6 + TASK_COUNT).overflowing_add(1);
        self.write16(r.a6 + TASK_COUNT, count);
        r.x = carry;
        r.d0 = low_word(r.d0, count);
        self.write16(COUNT, count);
        r.c = count < MAKE_EVERY;
        if count != MAKE_EVERY {
            return;
        }
        self.write16(r.a6 + TASK_COUNT, 0);
        self.zc_object(r, OBJ8);
        r.c = false;
        if self.read8(r.a4 + OBJ_KIND) != 0 {
            return;
        }
        self.zc_object(r, OBJ7);
        let x = self.read16(r.a4 + OBJ_X);
        self.write16(POSITION_X, x);
        let y = self.read16(r.a4 + OBJ_Y).wrapping_add(0x40);
        r.d0 = low_word(r.d0, y);
        self.write16(POSITION_Y, y);
        self.write32(OBJ_INIT_ANIMS, OBJ8_ANIMS);
        self.write32(OBJ_INIT_ROUTINE, OBJ8_ROUTINE);
        self.zc_object(r, OBJ8);
        self.zc_make(r, 0x31);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.write8(r.a4 + OBJ_FACING, 0);
        self.write8(r.a4 + OBJ_ANIM_KIND, 0);
        self.write8(r.a4 + OBJ_OFFSET, 0xC0);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.zc_animate(r);
        self.zc_place(r);
        r.d0 = low_byte(r.d0, MAKE_SOUND);
        self.r_0034b6(r.d0);
        r.c = false;
    }

    /// `func_01CA4E`, `OBJ8_ROUTINE`: animated; while object 6's
    /// `OBJ_11` isn't 0 it moves toward object 6 (`func_01CAD2`) and is
    /// placed, until both its x and y are within `NEAR` of object 6's
    /// (their low bytes); then, or once `OBJ_11` is 0, its sprite freed
    /// and object 8 too.  Gives back d0 (the last difference), d4 and
    /// a4 (object 6 or 8), d5 and a5 (object 8's sprite record), X and C
    /// (clear).
    pub fn r_01ca4e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_animate(r);
        self.zc_object(r, OBJ6);
        if self.read8(r.a4 + OBJ_11) != 0 {
            r.a4 = self.r_01cad2(r.d0, r.d1);
            r.d5 = low_byte(r.d5, self.read8(r.a4 + OBJ_SPRITE));
            r.a5 = self.read32(r.a4 + OBJ_SPRITE_ADDR);
            self.zc_place(r);
            if !self.zc_near(r, OBJ_X_LOW) || !self.zc_near(r, OBJ_Y_LOW) {
                return;
            }
        }
        self.r_000890();
        self.zc_object(r, OBJ8);
        self.write8(r.a4 + OBJ_KIND, 0);
        r.c = false;
    }

    /// Object 6's byte `field` less object 8's (kept in `OTHER`), made
    /// positive, into d0's low byte (X the borrow or the negation's);
    /// true (and C) when it is below `NEAR`.  a4 is object 6.
    fn zc_near(&mut self, r: &mut Regs, field: u32) -> bool {
        self.zc_object(r, OBJ8);
        let other = self.read8(r.a4 + field);
        self.write8(OTHER, other);
        self.zc_object(r, OBJ6);
        let own = self.read8(r.a4 + field);
        let mut diff = own.wrapping_sub(other);
        r.x = own < other;
        if diff & 0x80 != 0 {
            r.x = diff != 0;
            diff = diff.wrapping_neg();
        }
        r.d0 = low_byte(r.d0, diff);
        r.c = diff < NEAR;
        r.c
    }

    /// `func_01CAD2`: object 8 moved toward object 6 at 0x280
    /// (`func_00112C`, `func_0019C0`, `func_0019DA`).  Gives back a4
    /// (object 8).
    pub fn r_01cad2(&mut self, d0: u32, d1: u32) -> u32 {
        let (_, obj8, _) = self.r_001cfa(OBJ8);
        let x = self.read16(obj8 + OBJ_X);
        self.write16(FROM_X, x);
        let y = self.read16(obj8 + OBJ_Y);
        self.write16(FROM_Y, y);
        let (_, obj6, _) = self.r_001cfa(OBJ6);
        let x = self.read16(obj6 + OBJ_X);
        self.write16(TO_X, x);
        let y = self.read16(obj6 + OBJ_Y);
        self.write16(TO_Y, y);
        self.write16(SPEED, 0x280);
        self.r_00112c();
        let (_, a4, _) = self.r_001cfa(OBJ8);
        let (d0, d1, _) = self.r_0019c0(d0, d1, a4);
        self.r_0019da(d0, d1, a4);
        a4
    }
}
