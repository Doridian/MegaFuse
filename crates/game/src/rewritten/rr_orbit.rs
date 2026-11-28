//! Tasks that circle around a point and are let go, 0x1305E-0x1338A.
//! `func_012F4C` (before this range) makes the task `func_013068`,
//! which turns the circle (`ORBIT_ANGLE`, `func_012AB4`/`func_012AEE`)
//! and widens it (`ORBIT_RADIUS`), and four tasks of `func_012FCC` on
//! it.  When `ORBIT_STATE` goes negative they are let go: each becomes
//! one of the put tasks (`func_0130AE`, `PLACED`) and is thrown
//! (`func_01305E`, `func_01319E`); on landing it rolls
//! (`func_01323E`) from cell to cell, leaving tasks of `func_01337A`
//! behind (`func_0132E8`), until a cell stops it.  The object's
//! routines `loc_0130DA`, `func_013116`, `func_013124` and
//! `func_013134` are the thrower's side: the count `ORBIT_STATE`
//! goes up when its animation comes to its 8th step.
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read.

use crate::{Game, Regs};

/// An object's fields: its routine (a long), byte 0x12, its
/// animation's timer and step, its rate (a word, `a_motion.rs`).
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_12: u32 = 0x12;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_RATE: u32 = 0x28;
/// A sprite record's index (byte 3), frame and flags (`sprite.rs`).
const SPRITE_3: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
/// A task's fields: its sprite record's index, a count, its direction
/// (bits 0-1, `func_001620`), byte 0x18 (0x30, 0x10; `func_012FCC`
/// ors the quarter of its angle into it), 0x19, its rate (a word) and
/// offset (the high byte, `a_motion.rs`).
const TASK_SPRITE: u32 = 0x03;
const TASK_COUNT: u32 = 0x16;
const TASK_DIRECTION: u32 = 0x17;
const TASK_18: u32 = 0x18;
const TASK_19: u32 = 0x19;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;

/// The circle's state: 0 while it widens, then counted up by the
/// thrower (`loc_0130DA`), 0xFF when they are let go (bit 7: the
/// tasks of `func_012FCC` see it), then compared with each piece's
/// number (`rr_ring.rs`).
const ORBIT_STATE: u32 = 0xFFAD56;
/// The circle's angle (a word, its high byte the angle; turned by
/// `func_012AEE`) and its radius (up to `ORBIT_RADIUS_MAX`).
const ORBIT_ANGLE: u32 = 0xFF965C;
const ORBIT_RADIUS: u32 = 0xFF9664;
const ORBIT_RADIUS_MAX: u8 = 0x40;
/// The turn's speed asked of `func_012AB4` (a signed byte).
const TURN_ASKED: u32 = 0xFFAC8B;
/// Bit 7 set when the thrower lets them go.
const FLAGS_A5C2: u32 = 0xFFA5C2;
/// The running object (index, address) and the sprite record of the
/// routine being run (index, address; `a_objects.rs`).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// The running task's index and the task made last (`task.rs`).
const CURRENT_TASK: u32 = 0xFF80EE;
const NEW_TASK: u32 = 0xFF80EF;
/// The frame count (`func_0132E8` acts on every 4th frame).
const FRAME: u32 = 0xFF80D8;
/// The speed `func_001620` makes a velocity of (a word).
const SPEED: u32 = 0xFF9F00;
/// The cell's address in the cell map (`func_00167A`).
const CELL_AT: u32 = 0xFF9ED6;
/// The direction handed to `func_01313C`'s task.
const DIRECTION: u32 = 0xFFAABD;
/// Set to 8 when a rolling task stops (`func_0060CC` reads it).
const BYTE_AABE: u32 = 0xFFAABE;
/// A word set to 0x400 when a rolling task hits an object.
const WORD_ACEE: u32 = 0xFFACEE;
/// A task's offset kept across `func_001222` (`func_0132E8`).
const KEPT_OFFSET: u32 = 0xFFAC8D;
/// The entry's kind in `PLACED` (`func_0059F6`).
const PLACED_KIND: u32 = 2;

/// The routines of the tasks.
const ROUTINE_013116: u32 = 0x013116;
const ROUTINE_013124: u32 = 0x013124;
const ROUTINE_01319E: u32 = 0x01319E;
const ROUTINE_01323E: u32 = 0x01323E;
const ROUTINE_01337A: u32 = 0x01337A;
const ROUTINE_PUT: u32 = 0x005D46;

/// The cell bytes a rolling task stops on (5 and 6 have an object
/// looked for first).
const CELL_STOPS: u8 = 7;
const CELL_HITS: u8 = 5;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_01305E`, a thrown task's routine: `func_0132E8`, then it
    /// goes on as a thrown task (`loc_005C32`).
    pub fn r_01305e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x, r.c) = self.r_0132e8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        self.r_005c32(r);
    }

    /// `func_013068`, a task's routine: while `ORBIT_STATE` is 0 the
    /// circle turns at the speed `TURN_ASKED` (0x7F, `func_012AB4`) and
    /// widens a pixel a frame to `ORBIT_RADIUS_MAX`; after, it turns
    /// (`func_012AEE`) until the angle comes back under 4, when
    /// `ORBIT_STATE` becomes 0xFF (they are let go), the angle 0, and
    /// the task ends.
    pub fn r_013068(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let state = self.read8(ORBIT_STATE);
        r.d0 = low_byte(r.d0, state);
        if state != 0 {
            (r.d0, r.x) = self.r_012aee(r.d0);
            let angle = self.read8(ORBIT_ANGLE);
            r.d0 = low_byte(r.d0, angle);
            r.c = angle < 4;
            if !r.c {
                return;
            }
            self.write8(ORBIT_ANGLE, 0);
            self.write8(ORBIT_STATE, 0xFF);
            r.c = false;
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
            return;
        }
        self.write8(TURN_ASKED, 0x7F);
        (r.d0, r.x) = self.r_012ab4(r.d0);
        let radius = self.read8(ORBIT_RADIUS);
        r.d0 = low_byte(r.d0, radius);
        r.c = false;
        if radius != ORBIT_RADIUS_MAX {
            let (wider, carry) = radius.overflowing_add(1);
            self.write8(ORBIT_RADIUS, wider);
            (r.x, r.c) = (carry, carry);
        }
    }

    /// `func_0130AE`: the running task (a6, index d6) put in `PLACED`
    /// as an entry of kind 2 (`func_0059F6`), its byte 0x19 2, count
    /// 0x40 and direction 8.  Gives back d0 (the entry's offset), a0
    /// (`PLACED`), X and C (clear).
    pub fn r_0130ae(&mut self, d6: u32, a6: u32) -> (u32, u32, bool, bool) {
        let (d0, a0, x) = self.r_0059f6(PLACED_KIND, d6 & 0xFFFF_0000);
        let entry = a0.wrapping_add(d0 as u16 as i16 as u32);
        self.write8(entry, d6 as u8);
        self.write32(entry.wrapping_add(2), a6);
        self.write8(a6 + TASK_19, 2);
        self.write8(a6 + TASK_COUNT, 0x40);
        self.write8(a6 + TASK_DIRECTION, 8);
        (d0, a0, x, false)
    }

    /// `loc_0130DA`, an object's routine: its animation run
    /// (`func_001336`); when it comes to the 8th step `ORBIT_STATE`
    /// goes up, its routine becomes `func_013116` (byte 0x12 cleared,
    /// rate 0xFC00), bit 7 of `FLAGS_A5C2` is set and `func_004FD0`
    /// runs; otherwise `.L012DA0`.
    pub fn r_0130da(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        let timer = self.read8(r.a4 + OBJ_TIMER);
        r.d0 = low_byte(r.d0, timer);
        let at_8 = timer == 0 && {
            let step = self.read8(r.a4 + OBJ_STEP);
            r.d0 = low_byte(r.d0, step);
            step == 8
        };
        if !at_8 {
            (r.d0, r.d1, r.d2, r.d6, r.a0, r.a6, r.x, r.c) = self.r_012da0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            return;
        }
        let state = self.read8(ORBIT_STATE).wrapping_add(1);
        self.write8(ORBIT_STATE, state);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_013116);
        self.write8(r.a4 + OBJ_12, 0);
        let flags = self.read8(FLAGS_A5C2);
        self.write8(FLAGS_A5C2, flags | 0x80);
        self.write16(r.a4 + OBJ_RATE, 0xFC00);
        (r.d0, r.x, r.c) = self.r_004fd0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_013116`, an object's routine: its animation run
    /// (`func_001336`) over and over until it ends, then it goes on as
    /// `func_013124`.
    pub fn r_013116(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        loop {
            (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
            if r.c {
                break;
            }
        }
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_013124);
        r.c = false;
        self.r_013124(r);
    }

    /// `func_013124`, an object's routine: it falls (`func_001AF6`,
    /// `func_001ABC`); while up its sprite is placed (`func_013134`),
    /// once down it goes back to `func_012CBC` (`.L012F34`).
    pub fn r_013124(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = self.r_001af6(r.a4);
        (r.d0, r.x, r.n) = self.r_001abc(r.d0, r.a4);
        if r.n {
            self.r_013134(r);
        } else {
            r.c = self.r_012f34(r.a4);
        }
    }

    /// `func_013134`: the object's sprite placed (`func_001B32`) and its
    /// depth (`func_001B50`); N is clear after (`func_0007D2`).
    pub fn r_013134(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        (r.d0, r.x, r.c) = self.r_001b50(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        r.n = false;
    }

    /// `func_01313C`: a task of `func_01319E` (depth 0x80, priority
    /// 0x41) at the object's position, flags 0x84, byte 0x18 0x30,
    /// rising (rate 0xFD80, offset 0) in direction `DIRECTION`, and
    /// `func_004FF4`.  The object and the routine's sprite record are
    /// taken back (d4, a4, d5, a5).  Gives back d0, d4, d5, d6, a4, a5,
    /// a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01313c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.r_001542(a4);
        let (d5, d6, a5, a6, x, c) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0x80, 0x41, ROUTINE_01319E);
        if !c {
            self.r_00152a(a6);
            self.write8(a5 + SPRITE_FLAGS, 0x84);
            self.write8(a6 + TASK_18, 0x30);
            self.write16(a6 + TASK_RATE, 0xFD80);
            self.write8(a6 + TASK_OFFSET, 0);
            let direction = self.read8(DIRECTION);
            self.write8(a6 + TASK_DIRECTION, direction);
        }
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        if c {
            return (d0, d4, d5, d6, a4, a5, a6, x, false);
        }
        let (d0, x, c) = self.r_004ff4(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        (d0, d4, d5, d6, a4, a5, a6, x, c)
    }

    /// `func_01319E`, a thrown task's routine: frame 0xEA, it falls
    /// (`func_00C742`, 0x20); while up it moves in its direction (speed
    /// 0x60) and its sprite is placed; once down it becomes a put task
    /// (`func_0131B4`) unless `ORBIT_STATE` is set, when it rolls
    /// (`func_01323E`: flags 0x81, byte 0x18 0x10, frame 0xA1 + its
    /// direction, rate 0x200, the Z80 getting 0x85).
    pub fn r_01319e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, 0xEA);
        r.d0 = 0x20;
        self.r_00c742(r.d0, r.a6);
        let up;
        (r.d0, r.x, up) = self.r_001aca(r.d0, r.a6);
        if !up {
            (r.d0, r.a0, r.x, r.c) = self.r_0131b4(r.d6, r.a5, r.a6);
            return;
        }
        self.write16(SPEED, 0x60);
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_DIRECTION));
        self.r_001620(r.d0);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b74(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        let state = self.read8(ORBIT_STATE);
        r.d0 = low_byte(r.d0, state);
        r.c = false;
        if state == 0 {
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_01323E);
        self.write8(r.a5 + SPRITE_FLAGS, 0x81);
        self.write8(r.a6 + TASK_18, 0x10);
        let (frame, carry) = self.read8(r.a6 + TASK_DIRECTION).overflowing_add(0xA1);
        r.d0 = low_byte(r.d0, frame);
        r.x = carry;
        self.write8(r.a5 + SPRITE_FRAME, frame);
        self.write8(r.a6 + TASK_OFFSET, 0);
        self.write16(r.a6 + TASK_RATE, 0x200);
        r.d0 = low_byte(r.d0, 0x85);
        r.c = false;
        self.r_0034b6(r.d0);
        self.r_01323e(r);
    }

    /// `.L0131B4`: the task becomes a put task (`func_005D46`; flags
    /// 0x84, frame 0xEA, offset 0, byte 0x18 0x10) in `PLACED`
    /// (`func_0130AE`).  Gives back d0, a0, X and C.
    pub fn r_0131b4(&mut self, d6: u32, a5: u32, a6: u32) -> (u32, u32, bool, bool) {
        self.write8(a5 + SPRITE_FLAGS, 0x84);
        self.write8(a5 + SPRITE_FRAME, 0xEA);
        self.write8(a6 + TASK_OFFSET, 0);
        self.write8(a6 + TASK_18, 0x10);
        self.r_00053e(d6, ROUTINE_PUT);
        self.write8(a6 + TASK_18, 0x10);
        self.r_0130ae(d6, a6)
    }

    /// `func_01323E`, a rolling task's routine: the cell ahead in its
    /// direction (`func_0016F6`).  On a cell whose byte is under 5 or 7
    /// and more it stops: `BYTE_AABE` 8, the task ends (`func_00121A`)
    /// and `func_0060CC` runs on its cell.  On 5 or 6 an object there
    /// (`func_0018C4`) is hit: it is made the running object, gets
    /// `func_00E6F8` with the direction + 1, `WORD_ACEE` 0x400, and the
    /// task becomes a put task (`func_0131B4`).  Otherwise it rolls on
    /// (speed its rate, `func_001B14`) and leaves a trail
    /// (`func_0132E8`).
    pub fn r_01323e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = self.r_0015f8(r.a6);
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_DIRECTION));
        (r.d0, r.d1, r.a0, r.x) = self.r_0016f6(r.d0, r.d1);
        r.a0 = self.read32(CELL_AT);
        let cell = self.read8(r.a0);
        r.d0 = low_byte(r.d0, cell);
        if !(CELL_HITS..CELL_STOPS).contains(&cell) {
            r.d0 = self.r_0015f8(r.a6);
            (r.a0, r.x) = self.r_00167a();
            self.write8(BYTE_AABE, 8);
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            (r.d0, r.d1, r.d2, r.d5, r.d6, r.a0, r.a5, r.a6, r.x, r.c) = self.r_0060cc(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
            return;
        }
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_DIRECTION));
        let found;
        (r.d0, r.d1, r.a0, r.x, found) = self.r_0018c4(r.d0, r.d1, r.a0, r.a6);
        if found {
            let (object, address, _) = self.r_001cfa(low_byte(r.d4, r.d0 as u8));
            self.write8(RUNNING, object as u8);
            self.write32(RUNNING_ADDR, address);
            let (direction, carry) = self.read8(r.a6 + TASK_DIRECTION).overflowing_add(1);
            r.d0 = low_byte(r.d0, direction);
            (r.x, r.c) = (carry, carry);
            self.r_00e6f8(r);
            r.unwound = false;
            self.write16(WORD_ACEE, 0x400);
            let current = self.read8(CURRENT_TASK);
            (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
            r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
            r.a5 = self.read32(SLOT_ADDR);
            (r.d0, r.a0, r.x, r.c) = self.r_0131b4(r.d6, r.a5, r.a6);
            return;
        }
        r.d0 = self.r_001b14(r.a6);
        let rate = self.read16(r.a6 + TASK_RATE);
        self.write16(SPEED, rate);
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_DIRECTION));
        self.r_001620(r.d0);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        (r.d0, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x, r.c) = self.r_0132e8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_0132E8`: every 4th frame a task of `func_01337A` (depth
    /// 0x80, priority 0xC2, count 8) is left where the running task is,
    /// with its offset and the sprite of the routine's record (its
    /// index, frame, and flags with bit 6 set), its sprite placed.
    /// The running task and its sprite record are taken back (d6, a6,
    /// d5, a5) and made the routine's.  Gives back d0, d4, d5, d6, a4
    /// (the routine's sprite record, if a task was made), a5, a6, X
    /// and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0132e8(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let turn = self.read8(FRAME) & 3;
        let mut d0 = low_byte(d0, turn);
        if turn != 0 {
            return (d0, d4, d5, d6, a4, a5, a6, x, false);
        }
        let (mut d4, mut a4) = (d4, a4);
        let offset = self.read8(a6 + TASK_OFFSET);
        self.write8(KEPT_OFFSET, offset);
        self.r_00153a(a6);
        let (d5, d6, a5, a6, _, made) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0x80, 0xC2, ROUTINE_01337A);
        if !made {
            self.r_00152a(a6);
            self.write8(a6 + TASK_COUNT, 8);
            let offset = self.read8(KEPT_OFFSET);
            self.write8(a6 + TASK_OFFSET, offset);
            d4 = low_byte(d4, self.read8(SLOT_INDEX));
            a4 = self.read32(SLOT_ADDR);
            let index = self.read8(a4 + SPRITE_3);
            self.write8(a5 + SPRITE_3, index);
            let frame = self.read8(a4 + SPRITE_FRAME);
            self.write8(a5 + SPRITE_FRAME, frame);
            let flags = self.read8(a4 + SPRITE_FLAGS) | 0x40;
            self.write8(a5 + SPRITE_FLAGS, flags);
            let new = self.read8(NEW_TASK);
            let (d6, a6, _) = self.r_0005f8(low_byte(d6, new));
            (d0, _, _) = self.r_001b94(a5, a6);
            self.write8(SLOT_INDEX, d5 as u8);
            self.write32(SLOT_ADDR, a5);
            (d0, _, _) = self.r_001b74(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5, a6);
        }
        let current = self.read8(CURRENT_TASK);
        let (d6, a6, _) = self.r_0005f8(low_byte(d6, current));
        let index = self.read8(a6 + TASK_SPRITE);
        let (d5, a5, x) = self.r_000b5a(low_byte(d5, index));
        self.write8(SLOT_INDEX, d5 as u8);
        self.write32(SLOT_ADDR, a5);
        (d0, d4, d5, d6, a4, a5, a6, x, false)
    }

    /// `func_01337A`, a trail task's routine: its sprite's depth
    /// (`func_001B74`) for 8 frames, then it ends (`func_00121A`).
    pub fn r_01337a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(r.a6 + TASK_COUNT).wrapping_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        if count != 0 {
            (r.d0, r.x, r.c) = self.r_001b74(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        } else {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        }
    }
}
