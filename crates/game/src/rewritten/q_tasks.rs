//! Tasks of 0x125EA-0x127C2: `func_0125EA` makes one with an animation
//! (`func_01262C`) that, once `START_9663` is set and its animation
//! ends, makes four pairs of tasks with sprites (`func_0126E2`): one of
//! each pair flies off by an angle from `ANGLES` (`func_012722`, then
//! `func_01277E`), the other follows it (`func_01278C`).

use crate::{Game, Regs};

/// A task's fields (`task.rs`, `a_motion.rs`, `anim.rs`): its
/// position's low bytes, its animation's counters, a count (0x16), an
/// angle (0x17), its rate (a word), and at 0x20 either its animation
/// (a long) or its offset (a word: a byte and the byte added to its
/// x).
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y_LOW: u32 = 0x11;
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_ANGLE: u32 = 0x17;
const TASK_RATE: u32 = 0x1A;
const TASK_ANIM: u32 = 0x20;
const TASK_OFFSET: u32 = 0x20;
const TASK_OFFSET_X: u32 = 0x21;

/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// `func_01262C` waits for it not to be 0.
const START_9663: u32 = 0xFFFF_9663;
/// The sprites' frame for the pairs' tasks, from 0x2D, one more a pair.
const FRAME_AC90: u32 = 0xFFFF_AC90;
/// The pair being made (0-3).
const PAIR_AC91: u32 = 0xFFFF_AC91;
/// Where `func_01278C` keeps its leader's rate's high byte.
const RATE_AC8B: u32 = 0xFFFF_AC8B;
/// Counted up by `func_01277E` as each leader ends.
const ENDED_AD57: u32 = 0xFFFF_AD57;
/// The position (words, `a_objects.rs`) tasks are put at
/// (`func_00152A`).
const POSITION_X: u32 = 0xFFFF_9F04;
const POSITION_Y: u32 = 0xFFFF_9F06;
/// The speed (a word) `func_00410C` makes velocities of.
const SPEED: u32 = 0xFFFF_9F00;
/// A sprite record's index (`func_0126E2`) and a task's
/// (`func_01278C`).
const SPRITE_8A59: u32 = 0xFFFF_8A59;
const TASK_80EE: u32 = 0xFFFF_80EE;

/// By pair: the offset's byte, and the angle (ROM).
const OFFSETS: u32 = 0x01271A;
const ANGLES: u32 = 0x01271E;
/// The animation `func_0125EA` gives its task (ROM).
const ANIM_E8544: u32 = 0x0E8544;

/// The routines.
const WAITING: u32 = 0x01262C;
const LEADER: u32 = 0x012722;
const LEADER_END: u32 = 0x01277E;
const FOLLOWER: u32 = 0x01278C;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_001B7E`: the sprite placed for the task.
    fn q_task_place(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_001222` with the registers.
    fn q_make_task(&mut self, r: &mut Regs, depth: u8, priority: u8, routine: u32) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, priority, routine);
    }

    /// `func_0125EA`: a task (`func_001222`: depth 0xFC, priority 0x42,
    /// `func_01262C`) at `POSITION` (`func_00152A`) with the low bytes
    /// 0x88 and 0x18, its sprite's flags 1 and frame 0x1B, animation
    /// `ANIM_E8544`.  d0's low byte 0, C clear.
    pub fn r_0125ea(&mut self, regs: &mut Regs) {
        let r = regs;
        self.q_make_task(r, 0xFC, 0x42, WAITING);
        self.r_00152a(r.a6);
        self.write8(r.a6 + TASK_X_LOW, 0x88);
        self.write8(r.a6 + TASK_Y_LOW, 0x18);
        self.write8(r.a5 + SPRITE_FLAGS, 1);
        self.write8(r.a5 + SPRITE_FRAME, 0x1B);
        self.write32(r.a6 + TASK_ANIM, ANIM_E8544);
        r.d0 = low_byte(r.d0, 0);
        r.c = false;
        r.unwound = false;
    }

    /// `func_01262C`: placed until `START_9663` is set; then its
    /// animation a frame on (`func_0013D6`), while its `TASK_TIMER` is 0,
    /// `func_0043A8` with 0x20 and, at its step 6, sound 0x5F.  Once it
    /// ends, the task ends (`func_00121A`) and makes the pairs at
    /// (0x88, 0x28): for each, a leader (depth 0x80, priority 0x42,
    /// `func_012722`, count 2) and a follower (depth 0xFE, priority
    /// 0x43, `func_01278C`) that keeps the leader's index in its count,
    /// its sprite's flags 0x41 and frame 0x31; both set up by
    /// `func_0126E2`.  (Whether the tasks could be had isn't looked
    /// at.)  Then `func_013F14`.
    pub fn r_01262c(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        r.c = false;
        if self.read8(START_9663) != 0 {
            (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
            if r.c {
                (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
                self.write8(FRAME_AC90, 0x2D);
                self.write16(POSITION_X, 0x88);
                self.write16(POSITION_Y, 0x28);
                r.d0 = low_byte(r.d0, 0);
                loop {
                    self.write8(PAIR_AC91, r.d0 as u8);
                    self.q_make_task(r, 0x80, 0x42, LEADER);
                    (r.d0, r.d1, r.d5, r.a5) = self.r_0126e2(r.d1, r.d5, r.a5, r.a6);
                    self.write8(r.a6 + TASK_COUNT, 2);
                    let leader = r.d6;
                    self.q_make_task(r, 0xFE, 0x43, FOLLOWER);
                    (r.d0, r.d1, r.d5, r.a5) = self.r_0126e2(r.d1, r.d5, r.a5, r.a6);
                    r.d0 = leader;
                    self.write8(r.a6 + TASK_COUNT, leader as u8);
                    self.write8(r.a5 + SPRITE_FLAGS, 0x41);
                    self.write8(r.a5 + SPRITE_FRAME, 0x31);
                    let frame = self.read8(FRAME_AC90);
                    self.write8(FRAME_AC90, frame.wrapping_add(1));
                    let pair = self.read8(PAIR_AC91).wrapping_add(1);
                    r.d0 = low_byte(r.d0, pair);
                    if pair == 4 {
                        break;
                    }
                }
                (r.d0, r.d1, r.d4, r.d5, r.a4, r.a5, r.x, r.c) =
                    self.r_013f14(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
                return;
            }
            if self.read8(r.a6 + TASK_TIMER) == 0 {
                r.d0 = 0x20;
                (r.d0, r.x) = self.r_0043a8(r.d0, r.d4, r.d5, r.d6);
                if self.read8(r.a6 + TASK_STEP) == 6 {
                    r.d0 = low_byte(r.d0, 0x5F);
                    self.r_0034b6(r.d0);
                }
            }
        }
        self.q_task_place(r);
    }

    /// `func_0126E2`: a pair's task at `POSITION` (`func_00152A`), its
    /// sprite's frame `FRAME_AC90` and flags 1, rate 0xFC00, offset and
    /// angle by `PAIR_AC91` (`OFFSETS`, `ANGLES`), then the sprite
    /// record `SPRITE_8A59` (`func_000B5A`) placed for it
    /// (`func_001B94`).  Gives back d0, d1, d5 and a5.
    pub fn r_0126e2(&mut self, d1: u32, d5: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32) {
        self.r_00152a(a6);
        let frame = self.read8(FRAME_AC90);
        self.write8(a5 + SPRITE_FRAME, frame);
        self.write16(a6 + TASK_RATE, 0xFC00);
        self.write8(a5 + SPRITE_FLAGS, 1);
        let pair = self.read8(PAIR_AC91) as i8 as i16;
        let d1 = (d1 & 0xFFFF_0000) | u32::from(pair as u16);
        let index = pair as i32 as u32;
        let offset = self.read8(OFFSETS.wrapping_add(index));
        self.write8(a6 + TASK_OFFSET, offset);
        let angle = self.read8(ANGLES.wrapping_add(index));
        self.write8(a6 + TASK_ANGLE, angle);
        self.write8(a6 + TASK_OFFSET_X, 0);
        let d5 = low_byte(d5, self.read8(SPRITE_8A59));
        let (d5, a5, _) = self.r_000b5a(d5);
        let (d0, _, _) = self.r_001b94(a5, a6);
        (d0, d1, d5, a5)
    }

    /// `func_012722` (a leader): `SPEED` 0x200 by its angle
    /// (`func_00410C`), moved (`func_001A88`, `func_001AA2`),
    /// `func_00C742` with 0x40, its offset by its rate (`func_001ACA`);
    /// when that isn't negative (it landed), the offset's byte 0 and
    /// the count down: up again (rate 0xFD00) until it runs out, then
    /// a count of 0x40, its sprite's flags 0x41 and `func_01277E` from
    /// now on.  Placed (`func_012776`).
    pub fn r_012722(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write16(SPEED, 0x200);
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_ANGLE));
        (r.d0, r.d1, r.d2, r.x) = self.r_00410c(r.d0, r.d1, r.d2, r.x);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        r.d0 = 0x40;
        self.r_00c742(r.d0, r.a6);
        let negative;
        (r.d0, r.x, negative) = self.r_001aca(r.d0, r.a6);
        if !negative {
            self.write8(r.a6 + TASK_OFFSET, 0);
            let count = self.read8(r.a6 + TASK_COUNT);
            self.write8(r.a6 + TASK_COUNT, count.wrapping_sub(1));
            r.x = count == 0;
            if count != 1 {
                self.write16(r.a6 + TASK_RATE, 0xFD00);
            } else {
                self.write8(r.a6 + TASK_COUNT, 0x40);
                self.write8(r.a5 + SPRITE_FLAGS, 0x41);
                (r.x, r.c) = self.r_00053e(r.d6, LEADER_END);
            }
        }
        (r.d0, r.x, r.c) = self.r_012776(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        r.unwound = false;
    }

    /// `func_012776` (the end of the leaders' routines): the sprite
    /// placed (`func_001B94`), its depth (`func_001B74`).  Gives back
    /// d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_012776(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, bool, bool) {
        let (d0, _, _) = self.r_001b94(a5, a6);
        self.r_001b74(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5, a6)
    }

    /// `func_01277E` (a leader, landed): the count down, placed
    /// (`func_012776`); once it runs out, `ENDED_AD57` counted up and
    /// the task ends (`func_00121A`).
    pub fn r_01277e(&mut self, regs: &mut Regs) {
        let r = regs;
        let count = self.read8(r.a6 + TASK_COUNT);
        self.write8(r.a6 + TASK_COUNT, count.wrapping_sub(1));
        r.x = count == 0;
        if count != 1 {
            (r.d0, r.x, r.c) = self.r_012776(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        } else {
            let n = self.read8(ENDED_AD57);
            self.write8(ENDED_AD57, n.wrapping_add(1));
            r.x = n == 0xFF;
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        }
        r.unwound = false;
    }

    /// `func_01278C` (a follower): `POSITION` from its leader (the task
    /// whose index is its count, `func_0005F8`, `func_00153A`), the
    /// leader's rate's high byte to `RATE_AC8B`.  When the leader's
    /// offset is 0 and that byte isn't negative, it ends (`func_00121A`);
    /// else the task `TASK_80EE` is put at
    /// `POSITION` (`func_00152A`) and the sprite placed for it
    /// (`func_001B7E`).
    pub fn r_01278c(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        r.d6 = low_byte(r.d6, self.read8(r.a6 + TASK_COUNT));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        self.r_00153a(r.a6);
        let rate = self.read8(r.a6 + TASK_RATE);
        self.write8(RATE_AC8B, rate);
        let offset = self.read8(r.a6 + TASK_OFFSET);
        r.d0 = low_byte(r.d0, offset);
        if offset == 0 {
            r.d0 = low_byte(r.d0, rate);
            if rate & 0x80 == 0 {
                (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
                return;
            }
        }
        r.d6 = low_byte(r.d6, self.read8(TASK_80EE));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        self.r_00152a(r.a6);
        self.q_task_place(r);
    }
}
