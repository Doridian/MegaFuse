//! 0x1E12A-0x1E756: the boss of `zd_boss_b.rs` (object 6).  It comes
//! down (`func_01E12A`), then each frame (`func_01E16E`) runs the
//! routine its state picks (`STATES`): a pick (`func_01E1CA`), a charge
//! at object 0 (`func_01E21C`, speeding up for half the way and
//! slowing down after: `func_01E300`, `func_01E33A`), the animations of
//! its task (`func_01E374` with object 7's swing, `func_01E38C`), or
//! eight thrown objects (`func_01E3A0`); and is hit (`func_01E428`,
//! `func_01E478`).  Beaten, it waits for its tasks (`func_01E4B0`),
//! sprays pieces (`func_01E596`-`func_01E648`) and rings of four
//! (`func_01E66E`-`func_01E716`) for 0x100 frames (`func_01E4F4`), and
//! once no task of its runs any more (`func_01E556`, `func_01E570`)
//! ends with the task `func_019262`.
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read.

use crate::{Game, Regs};

/// An object's fields: its first byte (0: free), routine (a long),
/// byte 0xF, hits, byte 0x12 (0 when its hurt is over), direction, its
/// position (words: x, y), the animation's timer and step, and for the
/// thrown objects their angle, radius, phase and count (`zd_boss_c.rs`).
const OBJ_ID: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_0F: u32 = 0x0F;
const OBJ_HITS: u32 = 0x11;
const OBJ_12: u32 = 0x12;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_X: u32 = 0x18;
const OBJ_Y: u32 = 0x1A;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_ANGLE: u32 = 0x26;
const OBJ_RADIUS: u32 = 0x27;
const OBJ_PHASE: u32 = 0x28;
const OBJ_29: u32 = 0x29;
/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
/// A task's fields: the animation's timer and step, a count, an angle
/// and radius (the rings'), the animation (a long).
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_ANGLE: u32 = 0x18;
const TASK_RADIUS: u32 = 0x19;
const TASK_ANIM: u32 = 0x20;

/// Set while the boss comes down.
const SET_80DF: u32 = 0xFF80DF;
/// What `func_0012A0` sets an object up with: its routine.
const OBJ_INIT_ROUTINE: u32 = 0xFF80D0;
/// A position (words: x then y; `func_001300`, `func_001542`).
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
/// The speed (a word) `func_00112C` makes a velocity of, the velocity
/// (words: x, y).
const SPEED: u32 = 0xFF9F00;
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;
/// From where to where `func_00112C` aims (words).
const FROM_X: u32 = 0xFF9EFA;
const FROM_Y: u32 = 0xFF9EFE;
const TO_X: u32 = 0xFF9EE6;
const TO_Y: u32 = 0xFF9EE8;
/// Object 0's position (words).
const OBJECT_0_X: u32 = 0xFFA44A;
const OBJECT_0_Y: u32 = 0xFFA44C;
/// The running object (index, address) and the sprite record of the
/// routine being run (index, address).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// Set to 0x80 (bit 6 later) and bit 0 cleared (set again when the
/// boss is beaten).
const FLAGS: u32 = 0xFF9540;
const SPARED: u32 = 0xFF9548;
/// A byte copied each frame from byte 2 of the record at `RECORD_AD0A`,
/// cleared when the boss is beaten.
const BYTE_954F: u32 = 0xFF954F;
const RECORD_AD0A: u32 = 0xFFFFAD0A;

/// The boss's state: its routine is the long at `STATES` + state x 2
/// (0 pick, 2 charge, 4 and 8 its task's animations, 0xA throw, 6 and
/// 0xC wait for the end); 0x80 once beaten (its tasks then set bits 0
/// and 1), 0x84 at the end.
const STATE: u32 = 0xFFAEEE;
/// Set by the boss's tasks each frame they run.
const TASKS_RAN: u32 = 0xFFAEEF;
/// The charge's phase: 0 to start, 2 speeding up, 3 slowing down (4,
/// 5 for one along y, which never comes: see `func_01E21C`).
const PHASE: u32 = 0xFFAEF1;
/// The centre of the thrown objects' circle (words: x, y); the charge's
/// start.
const CENTRE_X: u32 = 0xFFAEF2;
const CENTRE_Y: u32 = 0xFFAEF4;
/// The charge: its target (words), speed (a word), the way left and
/// half the way (words, 8.8: their low bytes cleared).
const TARGET_X: u32 = 0xFFAEF6;
const TARGET_Y: u32 = 0xFFAEF8;
const CHARGE_SPEED: u32 = 0xFFAEFA;
const WAY: u32 = 0xFFAEFC;
const WAY_LOW: u32 = 0xFFAEFD;
const HALF: u32 = 0xFFAEFE;
const HALF_LOW: u32 = 0xFFAEFF;
/// The boss's count of frames.
const COUNT: u32 = 0xFFAF00;
/// The angle and radius of `func_01DE36` and `func_01DE4E`.
const ANGLE: u32 = 0xFFAF01;
const RADIUS: u32 = 0xFFAF03;
/// Set while the boss is hurt; its hits before `func_006E7C`.
const HURT: u32 = 0xFFAF04;
const HITS_BEFORE: u32 = 0xFFAF05;
/// The last pick, and the flag a thrown object sets when it meets
/// object 0 (the next pick is then 2's).
const LAST_PICK: u32 = 0xFFAF06;
const MET: u32 = 0xFFAF12;
/// The animation the boss's task shows (`func_01DB0C`), its count, and
/// object 7's swing (`zd_boss_c.rs`).
const SHOWN: u32 = 0xFFAF07;
const SHOWN_COUNT: u32 = 0xFFAF08;
const SWING: u32 = 0xFFAF09;
/// The area the pieces are sprayed over (words: its corner), the
/// centre of the rings (words).
const SPRAY_X: u32 = 0xFFAF0A;
const SPRAY_Y: u32 = 0xFFAF0C;
const RING_X: u32 = 0xFFAF0E;
const RING_Y: u32 = 0xFFAF10;
/// The thrown objects left.
const THROWN_LEFT: u32 = 0xFFAF13;
/// A word: what the last difference came to (`func_01E21C`,
/// `func_01E300`, `func_01E33A`).
const SUM: u32 = 0xFFAF14;
/// A counter of the tasks made.
const LEFT: u32 = 0xFFAF15;
/// The charge's distances (words: x, y; their low bytes).
const DISTANCE_X: u32 = 0xFFAF16;
const DISTANCE_X_LOW: u32 = 0xFFAF17;
const DISTANCE_Y: u32 = 0xFFAF18;

/// The routines of the states (longs).
const STATES: u32 = 0x01E1AE;
/// The picks (bytes, by a random 0-7; `func_01E1CA`), and the second
/// choice when the first is the last pick.
const PICKS: u32 = 0x01E20C;
const PICKS_AGAIN: u32 = 0x01E214;
const MET_PICK: u32 = 2;
/// The boss comes down at 1 a frame for 0x90 frames, and goes on at
/// 0xE0.
const DOWN_FRAMES: u8 = 0x90;
const DOWN_SPEED: u16 = 0x100;
const DOWN_END: u8 = 0xE0;
const ROUTINE_01E16E: u32 = 0x01E16E;
/// The charge's speed step.
const CHARGE_STEP: u16 = 8;
/// What `func_01E374` and `func_01E38C` ask for: the animation, its
/// count, the state after.
const SWING_SHOWN: u8 = 1;
const SWING_COUNT: u8 = 0x48;
const SWING_STATE: u8 = 6;
const OTHER_SHOWN: u8 = 3;
const OTHER_COUNT: u8 = 0x2A;
const OTHER_STATE: u8 = 0xA;
/// `func_01E3A0`: objects 8-0xF, their routine, frame, flags, radius,
/// the centre above the boss, the state after and the Z80's command.
const FIRST_THROWN: u8 = 8;
const PAST_THROWN: u8 = 0x10;
const THROWN: u8 = 8;
const ROUTINE_01DF1E: u32 = 0x01DF1E;
const THROWN_FRAME: u8 = 0x1F;
const FLAGS_81: u8 = 0x81;
const THROWN_RADIUS: u8 = 0x50;
const ABOVE: u16 = 0x20;
const THROW_STATE: u8 = 0xC;
const THROW_SOUND: u8 = 0x6D;
/// Beaten: the state, the routine, its direction (hurt 1, else 0).
const BEATEN: u8 = 0x80;
const ROUTINE_01E4B0: u32 = 0x01E4B0;
const BEATEN_DIRECTION: u8 = 2;
/// `func_01E4B0`: the Z80's command once both tasks are done, the next
/// routine.
const DONE_SOUND: u8 = 0x6F;
const ROUTINE_01E4F4: u32 = 0x01E4F4;
/// `func_01E4F4`: rings every 0x20 frames to 0xC0; then the routine,
/// direction, the end state.
const RINGS_UNTIL: u8 = 0xC0;
const ROUTINE_01E556: u32 = 0x01E556;
const END_DIRECTION: u8 = 3;
const END_STATE: u8 = 0x84;
const ROUTINE_01E570: u32 = 0x01E570;
/// The task made at the end, its priority.
const ROUTINE_019262: u32 = 0x019262;
const END_PRIORITY: u8 = 0x32;
/// The pieces: count, routine, depth, priority, animation, frames.
const PIECES: u8 = 3;
const ROUTINE_01E60A: u32 = 0x01E60A;
const TASK_DEPTH: u8 = 1;
const TASK_PRIORITY: u8 = 0x48;
const PIECE_ANIM: u32 = 0x0EB500;
const PIECE_TIMES: u8 = 0x14;
/// The rings: count, routine, animation, the angles (bytes, by the
/// count 1-4, the table after `func_01E6AC`'s `rts`), first radius,
/// the frame once the radius is 0x80.
const RING: u8 = 4;
const ROUTINE_01E6CC: u32 = 0x01E6CC;
const RING_ANIM: u32 = 0x0EB50C;
const RING_ANGLES: u32 = 0x01E6C7;
const RING_RADIUS: u8 = 0x30;
const RING_FRAME: u8 = 0x1E;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// `roxr.b #n` of `b` with X: the 9 bits X:b turned right `n` times.
/// Gives back the byte and X.
fn roxr8(b: u8, n: u32, x: bool) -> (u8, bool) {
    let mut v = (u16::from(x) << 8) | u16::from(b);
    for _ in 0..n {
        v = (v >> 1) | ((v & 1) << 8);
    }
    (v as u8, v & 0x100 != 0)
}

impl Game {
    /// `func_001336`, `func_001B32` and `func_001B50` on the family's
    /// registers: the boss animated, placed and `func_001B50`.
    fn zd_boss_frame(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        (r.d0, r.x, r.c) = self.r_001b50(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// The running task ends (`func_000890`, `func_00121A`).
    fn zd_task_ends(&mut self, r: &mut Regs) {
        self.r_000890();
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
    }

    /// `func_01E12A`, the boss's routine as it comes in (`SET_80DF`
    /// set): down a pixel a frame (`func_0019DA`) for `DOWN_FRAMES`
    /// frames (`COUNT`); at `DOWN_END` it goes on as `func_01E16E`,
    /// `FLAGS` 0x80, bit 0 of `SPARED` cleared.  Animated and placed.
    pub fn r_01e12a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(SET_80DF, 1);
        if self.read8(COUNT) < DOWN_FRAMES {
            self.write16(VELOCITY_Y, DOWN_SPEED);
            (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        }
        let was = self.read8(COUNT);
        let count = was.wrapping_add(1);
        self.write8(COUNT, count);
        r.x = count < was;
        if count >= DOWN_END {
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01E16E);
            self.write8(FLAGS, 0x80);
            let spared = self.read8(SPARED);
            self.write8(SPARED, spared & !1);
        }
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
    }

    /// `func_01E16E`, the boss's routine: `BYTE_954F` from the record
    /// at `RECORD_AD0A`, object 0's cell (`func_003E44`); unless it is
    /// `HURT`, its state's routine (`STATES`, family 2, with the slot's
    /// sprite record); then its hits (`func_01E428`), animated, placed,
    /// `func_001B50`.
    pub fn r_01e16e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.a0 = RECORD_AD0A;
        let byte = self.read8(RECORD_AD0A + 2);
        self.write8(BYTE_954F, byte);
        r.d0 = self.r_003e44();
        if self.read8(HURT) == 0 {
            let state = self.read8(STATE);
            r.d1 = low_word(r.d1, u16::from(state) * 2);
            r.a0 = self.read32(STATES + u32::from(state) * 2);
            r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
            r.a5 = self.read32(SLOT_ADDR);
            self.call_family_2(r.a0, r);
            if r.unwound {
                return;
            }
        }
        (r.d0, r.d1, r.d2, r.x) = self.r_01e428(r.d1, r.d2, r.a0, r.a4, r.a5);
        self.zd_boss_frame(r);
        self.r_01e1ac(r);
    }

    /// `func_01E1AC`, the routine of states 6 and 0xC: an `rts`.
    pub fn r_01e1ac(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `func_01E1CA`, state 0: a pick by a random 0-7 (`func_001BE2`;
    /// 2 when `MET` is set, which is cleared) from `PICKS`, or from
    /// `PICKS_AGAIN` when that is `LAST_PICK`; it is the state and
    /// `LAST_PICK`, `PHASE` and `COUNT` cleared.  d1, d5 and a5 are
    /// kept.
    pub fn r_01e1ca(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, _, r.x) = self.r_001be2(r.d1);
        let mut pick = u32::from(r.d0 as u8 & 7);
        r.d0 = low_word(r.d0, pick as u16);
        if self.read8(MET) != 0 {
            self.write8(MET, 0);
            pick = MET_PICK;
            r.d0 = MET_PICK;
        }
        let mut state = self.read8(PICKS + pick);
        if state == self.read8(LAST_PICK) {
            state = self.read8(PICKS_AGAIN + pick);
        }
        self.write8(STATE, state);
        self.write8(LAST_PICK, state);
        self.write8(PHASE, 0);
        self.write8(COUNT, 0);
        r.c = false;
    }

    /// `func_01E21C`, state 2: the charge.  Phase 0 sets it up: from
    /// the boss's position (`CENTRE_X`, `CENTRE_Y`) to object 0's
    /// (`TARGET_X`, `TARGET_Y`), speed 0, the way the distance along x
    /// (its low byte, x 0x100) and half that, phase 2.  (Along y when
    /// that is longer, phase 4, but the original branches on C after
    /// a `move`, which is clear, so it never does.)  Then each frame
    /// the velocity toward the target at the speed (`func_00112C`), the
    /// speed changed (`func_01E300`), the way used (`func_01E33A`), and
    /// unless that ended the charge (state 0) the boss moves
    /// (`func_0019C0`, `func_0019DA`).
    pub fn r_01e21c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(PHASE) == 0 {
            let x = self.read16(r.a4 + OBJ_X);
            self.write16(CENTRE_X, x);
            let y = self.read16(r.a4 + OBJ_Y);
            self.write16(CENTRE_Y, y);
            let x = self.read16(OBJECT_0_X);
            self.write16(TARGET_X, x);
            let y = self.read16(OBJECT_0_Y);
            self.write16(TARGET_Y, y);
            self.write16(CHARGE_SPEED, 0);
            let (to, from) = (self.read16(TARGET_X), self.read16(CENTRE_X));
            let dx = if to < from { from.wrapping_sub(to) } else { to.wrapping_sub(from) };
            self.write16(DISTANCE_X, dx);
            let (to, from) = (self.read16(TARGET_Y), self.read16(CENTRE_Y));
            let dy = if to < from { from.wrapping_sub(to) } else { to.wrapping_sub(from) };
            self.write16(DISTANCE_Y, dy);
            self.write8(WAY_LOW, 0);
            self.write8(HALF_LOW, 0);
            let difference = self.read16(DISTANCE_X).wrapping_sub(self.read16(DISTANCE_Y));
            self.write16(SUM, difference);
            let way = self.read8(DISTANCE_X_LOW);
            self.write8(WAY, way);
            self.write8(HALF, way >> 1);
            let (half_low, x) = roxr8(self.read8(HALF_LOW), 1, way & 1 != 0);
            r.d0 = low_byte(low_word(r.d0, difference), half_low);
            self.write8(HALF_LOW, half_low);
            r.x = x;
            self.write8(PHASE, 2);
            r.c = false;
            return;
        }
        let x = self.read16(r.a4 + OBJ_X);
        self.write16(FROM_X, x);
        let y = self.read16(r.a4 + OBJ_Y);
        self.write16(FROM_Y, y);
        let x = self.read16(TARGET_X);
        self.write16(TO_X, x);
        let y = self.read16(TARGET_Y);
        self.write16(TO_Y, y);
        let speed = self.read16(CHARGE_SPEED);
        self.write16(SPEED, speed);
        self.r_00112c();
        self.r_01e300();
        r.x = self.zd_use_way();
        let state = self.read8(STATE);
        r.d0 = low_byte(r.d0, state);
        r.c = false;
        if state != 0 {
            (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
            (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        }
    }

    /// `func_01E300`: the charge's speed.  Phase 2 (even): up by 8 while
    /// the way left is more than half (`SUM` the difference), else the
    /// next phase.  Phase 3 (odd): down by 8; when it would go below 0
    /// the charge is over (state 0).
    pub fn r_01e300(&mut self) {
        let phase = self.read8(PHASE);
        let speed = self.read16(CHARGE_SPEED);
        if phase & 1 != 0 {
            self.write16(CHARGE_SPEED, speed.wrapping_sub(CHARGE_STEP));
            if speed < CHARGE_STEP {
                self.write8(STATE, 0);
            }
            return;
        }
        let way = self.read16(WAY);
        let half = self.read16(HALF);
        self.write16(SUM, way.wrapping_sub(half));
        if way > half {
            self.write16(CHARGE_SPEED, speed.wrapping_add(CHARGE_STEP));
        } else {
            self.write8(PHASE, phase.wrapping_add(1));
        }
    }

    /// `func_01E33A`: the way left less the velocity's size along the
    /// charge (x; y for phases 4-7); below 0 the charge is over.
    pub fn r_01e33a(&mut self) {
        self.zd_use_way();
    }

    /// `func_01E33A`'s work.  Gives back X (the subtraction's borrow).
    fn zd_use_way(&mut self) -> bool {
        let velocity = if self.read8(PHASE) & 4 == 0 { VELOCITY_X } else { VELOCITY_Y };
        let v = self.read16(velocity);
        self.write16(SUM, v);
        if (self.read8(SUM) as i8) < 0 {
            self.write16(SUM, v.wrapping_neg());
        }
        let size = self.read16(SUM);
        let way = self.read16(WAY);
        self.write16(WAY, way.wrapping_sub(size));
        let borrow = way < size;
        if borrow {
            self.write8(STATE, 0);
        }
        borrow
    }

    /// `func_01E374`, state 4: the task's animation 1 (count 0x48) and
    /// object 7's swing (`SWING` 1); state 6.
    pub fn r_01e374(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = u32::from(SWING_SHOWN);
        self.write8(SHOWN, SWING_SHOWN);
        self.write8(SWING, SWING_SHOWN);
        self.write8(SHOWN_COUNT, SWING_COUNT);
        self.write8(STATE, SWING_STATE);
        r.c = false;
    }

    /// `func_01E38C`, state 8: the task's animation 3 (count 0x2A);
    /// state 0xA.
    pub fn r_01e38c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(SHOWN, OTHER_SHOWN);
        self.write8(SHOWN_COUNT, OTHER_COUNT);
        self.write8(STATE, OTHER_STATE);
        r.c = false;
    }

    /// `func_01E3A0`, state 0xA: once the task's animation is over,
    /// objects 8-0xF made (`func_0012A0`, routine `func_01DF1E`) round a
    /// centre 0x20 above the boss: frame 0x1F, flags 0x81, radius 0x50,
    /// angle the index turned right 4 through X (the X `func_0012A0`
    /// leaves: 0x80, 0x90, ...), phase and count 0.  State 0xC,
    /// `THROWN_LEFT` 8, d4 and a4 the running object again, and the Z80
    /// gets `THROW_SOUND`.
    pub fn r_01e3a0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.c = false;
        if self.read8(SHOWN) != 0 {
            return;
        }
        let x = self.read16(r.a4 + OBJ_X);
        self.write16(CENTRE_X, x);
        let y = self.read16(r.a4 + OBJ_Y).wrapping_sub(ABOVE);
        r.d0 = low_word(r.d0, y);
        self.write16(CENTRE_Y, y);
        r.d4 = u32::from(FIRST_THROWN);
        loop {
            let (d5, a5) = (r.d5, r.a5);
            let index = r.d4;
            (_, r.a4, r.x) = self.r_001cfa(index);
            self.write32(OBJ_INIT_ROUTINE, ROUTINE_01DF1E);
            (r.d0, r.d5, r.a5, r.x) = self.r_0012a0(r.d0, r.d1, r.d2, r.d3, index, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
            self.write8(r.a5 + SPRITE_FRAME, THROWN_FRAME);
            self.write8(r.a5 + SPRITE_FLAGS, FLAGS_81);
            self.write8(r.a4 + OBJ_0F, 0x21);
            self.write8(r.a4 + OBJ_RADIUS, THROWN_RADIUS);
            let angle;
            (angle, _) = roxr8(index as u8, 4, r.x);
            r.d0 = low_byte(index, angle);
            self.write8(r.a4 + OBJ_ANGLE, angle);
            self.write8(r.a4 + OBJ_PHASE, 0);
            self.write8(r.a4 + OBJ_29, 0);
            (r.d5, r.a5) = (d5, a5);
            let next = (index as u8).wrapping_add(1);
            r.d4 = low_byte(index, next);
            r.x = next < index as u8;
            if next >= PAST_THROWN {
                break;
            }
        }
        self.write8(STATE, THROW_STATE);
        self.write8(THROWN_LEFT, THROWN);
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        r.d0 = low_byte(r.d0, THROW_SOUND);
        r.c = false;
        self.r_0034b6(r.d0);
    }

    /// `func_01E428`: the boss hit (`func_0015E6`, `func_006E7C`).  Hit
    /// through (C), it is beaten (state 0x80, bit 0 of `SPARED` and bit
    /// 6 of `FLAGS` set, `BYTE_954F` 0, direction 2, on as
    /// `func_01E4B0`).  Else a hit lost makes it `HURT` (direction 1);
    /// while hurt, once its byte 0x12 is 0 it is well again (direction
    /// 0).  A new direction starts its animation (`func_01E478`).
    /// Gives back d0, d1, d2 and X.
    pub fn r_01e428(&mut self, d1: u32, d2: u32, a0: u32, a4: u32, a5: u32) -> (u32, u32, u32, bool) {
        let hurt = self.read8(HURT) != 0;
        if !hurt {
            let hits = self.read8(a4 + OBJ_HITS);
            self.write8(HITS_BEFORE, hits);
        }
        self.r_0015e6(a4);
        let (mut d0, d1, d2, _, mut x, c) = self.r_006e7c(d1, d2, a0, a4, a5);
        if c {
            self.write8(STATE, BEATEN);
            d0 = low_byte(d0, BEATEN);
            let spared = self.read8(SPARED);
            self.write8(SPARED, spared | 1);
            self.write32(a4 + OBJ_ROUTINE, ROUTINE_01E4B0);
            self.write8(a4 + OBJ_DIRECTION, BEATEN_DIRECTION);
            let flags = self.read8(FLAGS);
            self.write8(FLAGS, flags | 0x40);
            self.write8(BYTE_954F, 0);
        } else if hurt {
            let still = self.read8(a4 + OBJ_12);
            d0 = low_byte(d0, still);
            if still != 0 {
                return (d0, d1, d2, x);
            }
            self.write8(HURT, 0);
            self.write8(a4 + OBJ_DIRECTION, 0);
        } else {
            let hits = self.read8(a4 + OBJ_HITS);
            d0 = low_byte(d0, hits);
            if hits == self.read8(HITS_BEFORE) {
                return (d0, d1, d2, x);
            }
            let count = self.read8(HURT).wrapping_add(1);
            self.write8(HURT, count);
            self.write8(a4 + OBJ_DIRECTION, 1);
        }
        (_, x) = self.r_01e478(d0, a4);
        (d0, d1, d2, x)
    }

    /// `func_01E478`: the object's animation by its direction
    /// (`func_00130E`), its timer and step cleared.  Gives back a0 and
    /// X.
    pub fn r_01e478(&mut self, d0: u32, a4: u32) -> (u32, bool) {
        self.write8(a4 + OBJ_STEP, 0);
        self.write8(a4 + OBJ_TIMER, 0);
        let (a0, x, _) = self.r_00130e(d0, a4);
        (a0, x)
    }

    /// `.L01E484`: an `rts`.
    pub fn r_01e484(&mut self) {}

    /// `func_01E4B0`, the beaten boss's routine: animated, placed,
    /// `func_001B50`; once its tasks have set bits 0 and 1 of the
    /// state, the Z80 gets `DONE_SOUND`, it goes on as `func_01E4F4`
    /// (`COUNT` 0), the areas set (`func_01E596`), three pieces
    /// (`func_01E5C2`) and a ring (`func_01E66E`).
    pub fn r_01e4b0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zd_boss_frame(r);
        let bits = self.read8(STATE) & 3;
        r.d0 = low_byte(r.d0, bits);
        if bits != 3 {
            r.c = bits < 3;
            return;
        }
        r.d0 = low_byte(r.d0, DONE_SOUND);
        self.r_0034b6(r.d0);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01E4F4);
        self.write8(COUNT, 0);
        self.r_01e596(r.a4);
        (r.d0, r.d1, r.d5, r.d6, r.a5) = self.r_01e5c2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_01e66e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_01E4F4`, the beaten boss's routine: animated, placed,
    /// `func_001B50`; `COUNT` up, a ring (`func_01E66E`) every 0x20
    /// frames below 0xC0; at 0x100 direction 3 (started), 0x20 higher,
    /// state `END_STATE`, the Z80 gets `THROW_SOUND`, and on as
    /// `func_01E556`.
    pub fn r_01e4f4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zd_boss_frame(r);
        let count = self.read8(COUNT).wrapping_add(1);
        self.write8(COUNT, count);
        r.x = count == 0;
        r.c = count == 0;
        if count != 0 {
            r.d0 = low_byte(r.d0, count);
            r.c = false;
            if count < RINGS_UNTIL {
                let step = count & 0x1F;
                r.d0 = low_byte(r.d0, step);
                if step == 0 {
                    (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_01e66e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
                }
            }
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01E556);
        self.write8(r.a4 + OBJ_DIRECTION, END_DIRECTION);
        (r.a0, r.x) = self.r_01e478(r.d0, r.a4);
        self.r_001542(r.a4);
        let y = self.read16(POSITION_Y);
        self.write16(POSITION_Y, y.wrapping_sub(ABOVE));
        r.x = y < ABOVE;
        self.r_001300(r.a4);
        self.write8(STATE, END_STATE);
        r.d0 = low_byte(r.d0, THROW_SOUND);
        r.c = false;
        self.r_0034b6(r.d0);
    }

    /// `func_01E556`, the boss's routine: once its animation ends it
    /// goes on as `func_01E570`; placed, `func_001B50`.
    pub fn r_01e556(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if r.c {
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_01E570);
        }
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        (r.d0, r.x, r.c) = self.r_001b50(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_01E570`, the boss's last routine: once no task of its ran
    /// (`TASKS_RAN` 0) the boss ends (`func_000890`, its first byte
    /// cleared) with a task of `func_019262` (priority 0x32);
    /// `TASKS_RAN` cleared.
    pub fn r_01e570(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let ran = self.read8(TASKS_RAN);
        r.d0 = low_byte(r.d0, ran);
        if ran == 0 {
            self.r_000890();
            self.write8(r.a4 + OBJ_ID, 0);
            (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, END_PRIORITY, ROUTINE_019262);
        }
        self.write8(TASKS_RAN, 0);
        r.c = false;
    }

    /// `func_01E596`: the rings' centre the boss's position 0x20 up,
    /// the pieces' area from 0x1E left and 0x44 up.
    pub fn r_01e596(&mut self, a4: u32) {
        let x = self.read16(a4 + OBJ_X);
        let y = self.read16(a4 + OBJ_Y);
        self.write16(RING_X, x);
        self.write16(RING_Y, y.wrapping_sub(0x20));
        self.write16(SPRAY_X, x.wrapping_sub(0x1E));
        self.write16(SPRAY_Y, y.wrapping_sub(0x44));
    }

    /// `func_01E5C2`: `PIECES` tasks of `func_01E60A` (`func_001222`) at
    /// random places (`func_01E648`), animation `PIECE_ANIM`, flags
    /// 0x81, count 0x14.  Gives back d0, d1, d5, d6 and a5.
    #[allow(clippy::too_many_arguments)]
    pub fn r_01e5c2(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32) {
        let (mut d0, mut d1, mut d5, mut d6, mut a5) = (d0, d1, d5, d6, a5);
        self.write8(LEFT, PIECES);
        loop {
            let a6;
            (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, TASK_DEPTH, TASK_PRIORITY, ROUTINE_01E60A);
            (d0, d1) = self.r_01e648(d1);
            self.r_00152a(a6);
            self.write32(a6 + TASK_ANIM, PIECE_ANIM);
            self.write8(a6 + TASK_TIMER, 0);
            self.write8(a6 + TASK_STEP, 0);
            self.write8(a5 + SPRITE_FLAGS, FLAGS_81);
            self.write8(a6 + TASK_COUNT, PIECE_TIMES);
            let left = self.read8(LEFT).wrapping_sub(1);
            self.write8(LEFT, left);
            if left == 0 {
                return (d0, d1, d5, d6, a5);
            }
        }
    }

    /// `func_01E60A`, a piece's routine (`TASKS_RAN` set): placed,
    /// animated (`func_0013D6`); each time its animation ends it is put
    /// at a new random place (`func_01E648`), its count down; at 0 it
    /// ends.
    pub fn r_01e60a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(TASKS_RAN, 1);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if !r.c {
            return;
        }
        (r.d0, r.d1) = self.r_01e648(r.d1);
        self.r_00152a(r.a6);
        self.write32(r.a6 + TASK_ANIM, PIECE_ANIM);
        self.write8(r.a6 + TASK_TIMER, 0);
        self.write8(r.a6 + TASK_STEP, 0);
        let was = self.read8(r.a6 + TASK_COUNT);
        self.write8(r.a6 + TASK_COUNT, was.wrapping_sub(1));
        r.x = was == 0;
        r.c = was == 0;
        if was == 1 {
            self.zd_task_ends(r);
        }
    }

    /// `func_01E648`: `POSITION` a random place in the 0x40-square from
    /// `SPRAY_X`, `SPRAY_Y` (`func_001BE2`; x's number plus 0x83).
    /// Gives back d0 and d1.
    pub fn r_01e648(&mut self, d1: u32) -> (u32, u32) {
        let (d0, d1, _) = self.r_001be2(d1);
        let d0 = low_byte(d0, (d0 as u8).wrapping_add(0x83));
        let x = ((d0 as u16) & 0x3F).wrapping_add(self.read16(SPRAY_X));
        self.write16(POSITION_X, x);
        let (d0, d1, _) = self.r_001be2(d1);
        let y = ((d0 as u16) & 0x3F).wrapping_add(self.read16(SPRAY_Y));
        self.write16(POSITION_Y, y);
        (low_word(d0, y), d1)
    }

    /// `func_01E66E`: a ring: `RING` tasks of `func_01E6CC`
    /// (`func_001222`) at the angles of `func_01E6AC`, animation
    /// `RING_ANIM`, flags 0x81.  Gives back d5, d6, a5, a6, X and C
    /// (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01e66e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        let (mut d5, mut d6, mut a5) = (d5, d6, a5);
        self.write8(LEFT, RING);
        loop {
            let a6;
            (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, TASK_DEPTH, TASK_PRIORITY, ROUTINE_01E6CC);
            self.r_01e6ac(a6);
            self.write32(a6 + TASK_ANIM, RING_ANIM);
            self.write8(a6 + TASK_TIMER, 0);
            self.write8(a6 + TASK_STEP, 0);
            self.write8(a5 + SPRITE_FLAGS, FLAGS_81);
            let left = self.read8(LEFT).wrapping_sub(1);
            self.write8(LEFT, left);
            if left == 0 {
                return (d5, d6, a5, a6, false, false);
            }
        }
    }

    /// `func_01E6AC`: the task's angle from `RING_ANGLES` by `LEFT`
    /// (1-4: 0, 0x40, 0x80, 0xC0), its radius `RING_RADIUS`.  d1 is
    /// kept.
    pub fn r_01e6ac(&mut self, a6: u32) {
        let at = self.read8(LEFT) as i8 as i32 as u32;
        let angle = self.read8(RING_ANGLES.wrapping_add(at));
        self.write8(a6 + TASK_ANGLE, angle);
        self.write8(a6 + TASK_RADIUS, RING_RADIUS);
    }

    /// `func_01E6CC`, a ring task's routine (`TASKS_RAN` set): it turns
    /// and grows (`func_01E716`) and ends when its radius passes 0xFF;
    /// placed; from radius 0x80 frame `RING_FRAME`, before that its
    /// animation (`RING_ANIM`, started again when it ends).
    pub fn r_01e6cc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(TASKS_RAN, 1);
        let gone;
        (r.d0, r.x, gone) = self.r_01e716(r.d0, r.d3, r.a6);
        if gone {
            self.zd_task_ends(r);
            return;
        }
        self.r_00152a(r.a6);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        if (self.read8(r.a6 + TASK_RADIUS) as i8) < 0 {
            self.write8(r.a5 + SPRITE_FRAME, RING_FRAME);
            r.c = false;
            return;
        }
        self.write32(r.a6 + TASK_ANIM, RING_ANIM);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            self.write8(r.a6 + TASK_TIMER, 0);
            self.write8(r.a6 + TASK_STEP, 0);
            r.c = false;
        }
    }

    /// `func_01E716`: the task's angle 8 on and radius 4 more; C when
    /// the radius passes 0xFF.  Else `POSITION` the circle's point
    /// (`func_01DE36`, `func_01DE4E`) from `RING_X`, `RING_Y`.  Gives
    /// back d0, X and C.
    pub fn r_01e716(&mut self, d0: u32, d3: u32, a6: u32) -> (u32, bool, bool) {
        let angle = self.read8(a6 + TASK_ANGLE).wrapping_add(8);
        self.write8(a6 + TASK_ANGLE, angle);
        let was = self.read8(a6 + TASK_RADIUS);
        let radius = was.wrapping_add(4);
        self.write8(a6 + TASK_RADIUS, radius);
        if radius < was {
            return (d0, true, true);
        }
        self.write8(ANGLE, angle);
        self.write8(RADIUS, radius);
        let d0 = self.r_01de36(d0, d3);
        let x = (d0 as u16).wrapping_add(self.read16(RING_X));
        self.write16(POSITION_X, x);
        let d0 = self.r_01de4e(low_word(d0, x), d3);
        let dy = d0 as u16;
        let y = dy.wrapping_add(self.read16(RING_Y));
        self.write16(POSITION_Y, y);
        (low_word(d0, y), y < dy, false)
    }
}
