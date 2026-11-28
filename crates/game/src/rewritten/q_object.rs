//! The object's next state (0x12CBC-0x13068): `func_012CBC` walks it
//! cell by cell (`func_007174`) and now and then turns it toward
//! object 0 (`func_012D4A`, `func_012D76`); when it is stopped
//! (`func_012DB6`) a task (`func_013750`) takes over.  Or it plays an
//! animation (`loc_012E06`) that at its step 6 makes four tasks
//! circling a point (`func_012F4C`, `func_012FCC`), rises
//! (`func_012E5A`), waits (`func_012E88`), plays another (`func_012EB0`,
//! counting `COUNT_AD56` up at its step 6), waits again
//! (`func_012EEC`), comes down (`func_012F18`) and walks again.

use crate::{Game, Regs};

/// An object's fields (`g_objects.rs`, `a_motion.rs`): its routine and
/// the long it keeps (whose first byte's high nibble is a mark), a byte
/// cleared on landing (0x12), its direction and its animation's kind,
/// a count (0x16), its animation (a long) and the animation's counters
/// (`anim.rs`), a byte set to 0xC0 (0x26), its rate (a word), its
/// offset (the high byte of a word) and the byte after it.
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_SAVED: u32 = 0x06;
const OBJ_12: u32 = 0x12;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x15;
const OBJ_COUNT: u32 = 0x16;
const OBJ_ANIM: u32 = 0x20;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_26: u32 = 0x26;
const OBJ_RATE: u32 = 0x28;
const OBJ_OFFSET: u32 = 0x2A;
const OBJ_PHASE: u32 = 0x2B;

/// A task's fields (`task.rs`, `a_motion.rs`): its position (words),
/// a byte (0x15), its angle (0x17), a byte set to 0x30 (0x18), its
/// rate (a word) and offset (a byte).
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_15: u32 = 0x15;
const TASK_ANGLE: u32 = 0x17;
const TASK_18: u32 = 0x18;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;

/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// The direction `func_012D76` picks (`func_003F1A`), also given to
/// `func_00130E` by `func_012CBC`.
const TURN_AABD: u32 = 0xFFFF_AABD;
/// The cell's address in the cell map (`func_00167A`); a cell of 5 or
/// more is one `func_012D76` doesn't turn into.
const CELL_AT: u32 = 0xFFFF_9ED6;
const CELL_OPEN: u8 = 5;
/// Bits set and cleared here: `MODE` bit 6, `SPARED` bit 0, bit 7 of
/// `BITS_A5C2` and `BITS_A582`.
const MODE: u32 = 0xFFFF_9540;
const SPARED: u32 = 0xFFFF_9548;
const BITS_A5C2: u32 = 0xFFFF_A5C2;
const BITS_A582: u32 = 0xFFFF_A582;
/// Cleared by `func_012DB6`.
const BYTE_80DF: u32 = 0xFFFF_80DF;
/// A count: 1 when stopped, cleared, counted up at the animation's
/// step 6; the circling tasks go out while it isn't negative.
const COUNT_AD56: u32 = 0xFFFF_AD56;
const COUNT_AD57: u32 = 0xFFFF_AD57;
/// The circling's turn (`func_012FCC`: added to each task's angle) and
/// speed (`q_stage.rs`), and its radius.
const POSITION_965C: u32 = 0xFFFF_965C;
const SPEED_965E: u32 = 0xFFFF_965E;
const RADIUS: u32 = 0xFFFF_9664;
/// The tasks still to make, and the next one's angle.
const TO_MAKE: u32 = 0xFFFF_AC8D;
const NEXT_ANGLE: u32 = 0xFFFF_AC8C;
/// The point the tasks circle (words) and the byte their offset is
/// made from.
const CENTRE_X: u32 = 0xFFFF_A5CA;
const CENTRE_Y: u32 = 0xFFFF_A5CC;
const HEIGHT_A5DC: u32 = 0xFFFF_A5DC;

/// The object running (index, address) and its sprite record's
/// (`a_objects.rs`): reloaded after making tasks.
const RUNNING: u32 = 0xFFFF_A42C;
const RUNNING_ADDR: u32 = 0xFFFF_A42E;
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// Animations in the ROM.
const ANIM_E86A4: u32 = 0x0E86A4;
const ANIM_E86B8: u32 = 0x0E86B8;
const ANIM_E86D2: u32 = 0x0E86D2;
const ANIM_E86D8: u32 = 0x0E86D8;

/// The routines.
const WALKING: u32 = 0x012CBC;
const PLAYING: u32 = 0x012E06;
const RISING: u32 = 0x012E5A;
const UP: u32 = 0x012E88;
const PLAYING_AGAIN: u32 = 0x012EB0;
const WAITING: u32 = 0x012EEC;
const COMING_DOWN: u32 = 0x012F18;
const TURNED: u32 = 0x0130DA;
const STOPPED: u32 = 0x0139DC;
const STOPPED_TASK: u32 = 0x013750;
const CIRCLING_TASK: u32 = 0x013068;
const CIRCLING: u32 = 0x012FCC;
const CIRCLING_OFF: u32 = 0x01305E;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_001336`: the object's animation a frame on (C: it ended).
    fn q_obj_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
    }

    /// `func_012E72` on the registers.
    fn q_obj_place(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_012e72(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// The object's `OBJ_COUNT` counted down (X its borrow); whether it
    /// got to 0.
    fn q_obj_count_down(&mut self, r: &mut Regs) -> bool {
        let n = self.read8(r.a4 + OBJ_COUNT);
        self.write8(r.a4 + OBJ_COUNT, n.wrapping_sub(1));
        r.x = n == 0;
        r.c = r.x;
        n == 1
    }

    /// Whether the animation is at `step` (its timer 0); d0's low byte
    /// the last byte read.
    fn q_at_step(&mut self, r: &mut Regs, step: u8) -> bool {
        let timer = self.read8(r.a4 + OBJ_TIMER);
        r.d0 = low_byte(r.d0, timer);
        if timer != 0 {
            return false;
        }
        let at = self.read8(r.a4 + OBJ_STEP);
        r.d0 = low_byte(r.d0, at);
        at == step
    }

    /// The animation `anim` from its start.
    fn q_obj_anim(&mut self, obj: u32, anim: u32) {
        self.write32(obj + OBJ_ANIM, anim);
        self.write8(obj + OBJ_TIMER, 0);
        self.write8(obj + OBJ_STEP, 0);
    }

    /// `func_012CBC`: unless turned toward object 0 (`func_012D4A`),
    /// walks: while the mark is set, `func_007174` (and stopped,
    /// `func_012DB6`, when it says so); else the mark set and, on a
    /// random half, the turn tried again (`func_012D76`).  Not turned:
    /// `COUNT_AD56` 0 and `ANIM_E86A4`, `loc_012E06` from now on (and
    /// now).  Turned: `COUNT_AD56` 0, the animation's counters 0, its
    /// animation for the direction `TURN_AABD` and kind 4
    /// (`func_00130E`, the object's own kept), `func_01313C`, then
    /// `loc_0130DA` from now on (and now).
    pub fn r_012cbc(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        (r.d0, r.d1, r.x, r.c) = self.r_012d4a(r.d0, r.d1, r.a4, r.x);
        let mut turned = r.c;
        if !turned {
            let mark = self.read8(r.a4 + OBJ_SAVED) & 0xF0;
            r.d0 = low_byte(r.d0, mark);
            if mark != 0 {
                r.z = false;
                r.c = false;
                self.r_007174(r);
                if self.restarted.is_some() {
                    return;
                }
                if r.c {
                    (r.d0, r.d6, r.a0, r.a6, r.x, r.c) = self.r_012db6(r.d0, r.d4, r.d5, r.d6, r.a4);
                }
                return;
            }
            self.write8(r.a4 + OBJ_SAVED, 0x80);
            (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
            r.n = r.d0 & 0x80 != 0;
            if !r.n {
                (r.d0, r.d1, r.a0, r.x, r.c) = self.r_012d76(r.d1, r.a4);
                turned = r.c;
            }
            if !turned {
                self.write8(COUNT_AD56, 0);
                self.q_obj_anim(r.a4, ANIM_E86A4);
                self.write32(r.a4 + OBJ_ROUTINE, PLAYING);
                r.c = false;
                self.r_012e06(r);
                return;
            }
        }
        self.write8(COUNT_AD56, 0);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        let direction = self.read8(r.a4 + OBJ_DIRECTION);
        let turn = self.read8(TURN_AABD);
        self.write8(r.a4 + OBJ_DIRECTION, turn);
        self.write8(r.a4 + OBJ_KIND, 4);
        (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
        self.write8(r.a4 + OBJ_KIND, 0);
        self.write8(r.a4 + OBJ_DIRECTION, direction);
        self.write32(r.a4 + OBJ_ROUTINE, TURNED);
        (r.d0, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_01313c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.r_0130da(r);
    }

    /// `func_012D4A`: whether to turn: in the middle of its cell
    /// (`func_00182E`, `func_001840`), object 0's cell
    /// (`func_003E44`) in its column or row (`func_003E5C`) and a
    /// random quarter: then `func_012D76`.  Gives back d0, d1, X and C
    /// (turned).
    pub fn r_012d4a(&mut self, d0: u32, d1: u32, a4: u32, x: bool) -> (u32, u32, bool, bool) {
        let (d0, middle, _) = self.r_00182e(d0, a4);
        if !middle {
            return (d0, d1, x, false);
        }
        let (d0, middle, _) = self.r_001840(d0, a4);
        if !middle {
            return (d0, d1, x, false);
        }
        self.r_003e44();
        let (d0, x) = self.r_0015e6(a4);
        let (d0, in_line) = self.r_003e5c(d0);
        if !in_line {
            return (d0, d1, x, false);
        }
        let (d0, d1, x) = self.r_001be2(d1);
        let d0 = low_byte(d0, (d0 as u8) & 3);
        if d0 as u8 != 0 {
            return (d0, d1, x, false);
        }
        let (d0, d1, _, x, c) = self.r_012d76(d1, a4);
        (d0, d1, x, c)
    }

    /// `func_012D76`: the direction toward object 0's cell
    /// (`func_003E44`, `func_0015E6`, `func_003F1A`) into `TURN_AABD`,
    /// the cell that way (`func_00173E`); C when it is `CELL_OPEN` or
    /// more.  Gives back d0 (its low byte the cell), d1, a0 (the
    /// cell's address), X and C.
    pub fn r_012d76(&mut self, d1: u32, a4: u32) -> (u32, u32, u32, bool, bool) {
        self.r_003e44();
        let (d0, x) = self.r_0015e6(a4);
        // As the translation passes it: the registers this routine
        // doesn't take are 0.
        let mut r = Regs { d0, d1, d2: 0, d3: 0, d4: 0, d5: 0, d6: 0, d7: 0, a0: 0, a1: 0, a2: 0, a3: 0, a4, a5: 0, a6: 0, x, n: false, z: false, v: false, c: false, unwound: false };
        self.r_003f1a(&mut r);
        self.write8(TURN_AABD, r.d0 as u8);
        let x = self.r_00173e(r.d0, a4);
        let a0 = self.read32(CELL_AT);
        let cell = self.read8(a0);
        (low_byte(r.d0, cell), r.d1, a0, x, cell >= CELL_OPEN)
    }

    /// `func_012DA0` (the end of `loc_012E06`): placed (`func_001B32`,
    /// `func_001B50`), its cell (`func_0015E6`), `func_006E28`; stopped
    /// (`func_012DB6`) when that says so.  Gives back d0, d1, d2, d6,
    /// a0, a6, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_012da0(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        self.r_001b50(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        let (d0, _) = self.r_0015e6(a4);
        let (d0, d1, d2, a0, x, c) = self.r_006e28(d0, d1, d2, a0, a4, a5);
        if c {
            let (d0, d6, a0, a6, x, c) = self.r_012db6(d0, d4, d5, d6, a4);
            return (d0, d1, d2, d6, a0, a6, x, c);
        }
        (d0, d1, d2, d6, a0, a6, x, c)
    }

    /// `func_012DB6`: stopped: its animation of kind 8 (`func_00130E`),
    /// `func_0139DC` from now on, `OBJ_26` 0xC0, put in the middle of
    /// its cell (`func_001852`), the bits of `SPARED` and `MODE` set,
    /// `BYTE_80DF` 0, the task `func_013750` (`func_0003E8`, priority
    /// 0x42), `COUNT_AD56` 1, `COUNT_AD57` 0, the Z80's command 0xF9.
    /// Gives back d0, d6, a0, a6, X and C (clear).
    pub fn r_012db6(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        self.write8(a4 + OBJ_KIND, 8);
        let (a0, _, _) = self.r_00130e(d0, a4);
        self.write32(a4 + OBJ_ROUTINE, STOPPED);
        self.write8(a4 + OBJ_26, 0xC0);
        self.r_001852(a4);
        let s = self.read8(SPARED);
        self.write8(SPARED, s | 1);
        let m = self.read8(MODE);
        self.write8(MODE, m | 0x40);
        self.write8(BYTE_80DF, 0);
        let (d6, a6, x, _) = self.r_0003e8(d4, d5, d6, 0x42, STOPPED_TASK);
        self.write8(COUNT_AD56, 1);
        self.write8(COUNT_AD57, 0);
        let d0 = low_byte(d0, 0xF9);
        self.r_00349e(d0);
        (d0, d6, a0, a6, x, false)
    }

    /// `func_012DA0` on the registers.
    fn q_012da0(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d6, r.a0, r.a6, r.x, r.c) =
            self.r_012da0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `loc_012E06`: the animation a frame on; at its step 6 the
    /// circling tasks (`func_012F4C`); then `func_012DA0`.  When it
    /// ends, bit 7 of `BITS_A5C2` set, the rate, offset, phase and
    /// `OBJ_12` 0, `func_004FD0`, `ANIM_E86D8` and `func_012E5A` from
    /// now on (and now).
    pub fn r_012e06(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let z;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, z, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if r.c {
            let b = self.read8(BITS_A5C2);
            self.write8(BITS_A5C2, b | 0x80);
            self.write16(r.a4 + OBJ_RATE, 0);
            self.write8(r.a4 + OBJ_OFFSET, 0);
            self.write8(r.a4 + OBJ_PHASE, 0);
            self.write8(r.a4 + OBJ_12, 0);
            (r.d0, r.x, r.c) = self.r_004fd0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            self.write32(r.a4 + OBJ_ANIM, ANIM_E86D8);
            self.write32(r.a4 + OBJ_ROUTINE, RISING);
            let b = self.read8(BITS_A5C2);
            self.write8(BITS_A5C2, b | 0x80);
            r.z = z;
            r.c = false;
            self.r_012e5a(r);
            return;
        }
        if self.q_at_step(r, 6) {
            (r.d4, r.d5, r.d6, r.a4, r.a5, r.a6) =
                self.r_012f4c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
        self.q_012da0(r);
    }

    /// `func_012E5A`: the animation a frame on, `func_004424` with 0xF8,
    /// its offset by its rate (`func_001ABC`); once that is under 0xF2
    /// (up), a count of 0x80 and `func_012E88` from now on (and now).
    /// Else placed.
    pub fn r_012e5a(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.q_obj_animate(r);
        r.d0 = low_byte(r.d0, 0xF8);
        (r.d0, r.x, r.n) = self.r_004424(r.d0, r.a4);
        (r.d0, r.x, r.n) = self.r_001abc(r.d0, r.a4);
        if (r.d0 as u8) < 0xF2 {
            self.write8(r.a4 + OBJ_COUNT, 0x80);
            self.write32(r.a4 + OBJ_ROUTINE, UP);
            r.c = false;
            self.r_012e88(r);
        } else {
            self.q_obj_place(r);
        }
    }

    /// `func_012E72`: placed (`func_001B32`, `func_001B50`).  Gives back
    /// d0, d1, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_012e72(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        let (d0, x, c) = self.r_001b50(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        (d0, d1, x, c)
    }

    /// `func_012E88`: the animation a frame on, bobbing (`func_012A8C`),
    /// counted down; then `ANIM_E86B8` and `func_012EB0` from now on
    /// (and now).  Else placed.
    pub fn r_012e88(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.q_obj_animate(r);
        (r.d0, r.d5, r.a5, r.x) = self.r_012a8c(r.d0, r.d5, r.a4);
        if self.q_obj_count_down(r) {
            self.q_obj_anim(r.a4, ANIM_E86B8);
            self.write32(r.a4 + OBJ_ROUTINE, PLAYING_AGAIN);
            r.c = false;
            self.r_012eb0(r);
        } else {
            self.q_obj_place(r);
        }
    }

    /// `func_012EB0`: the animation a frame on; at its step 6 sound
    /// 0x6E and `COUNT_AD56` counted up (not placed then); else placed.
    /// When it ends, `ANIM_E86D2`, a count of 0x80 and `func_012EEC`
    /// from now on (and now).
    pub fn r_012eb0(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.q_obj_animate(r);
        if r.c {
            self.write32(r.a4 + OBJ_ROUTINE, WAITING);
            self.write32(r.a4 + OBJ_ANIM, ANIM_E86D2);
            self.write8(r.a4 + OBJ_COUNT, 0x80);
            r.c = false;
            self.r_012eec(r);
            return;
        }
        if !self.q_at_step(r, 6) {
            self.q_obj_place(r);
            return;
        }
        r.d0 = low_byte(r.d0, 0x6E);
        self.r_0034b6(r.d0);
        let n = self.read8(COUNT_AD56);
        self.write8(COUNT_AD56, n.wrapping_add(1));
        r.x = n == 0xFF;
        r.c = r.x;
    }

    /// `func_012EEC`: counted down: bobbing (`func_012A8C`), the
    /// animation a frame on, placed.  Then the animation's counters and
    /// the rate 0, `func_012F18` from now on (and now).
    pub fn r_012eec(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.q_obj_count_down(r) {
            self.write8(r.a4 + OBJ_STEP, 0);
            self.write8(r.a4 + OBJ_TIMER, 0);
            self.write16(r.a4 + OBJ_RATE, 0);
            self.write32(r.a4 + OBJ_ROUTINE, COMING_DOWN);
            r.c = false;
            self.r_012f18(r);
        } else {
            (r.d0, r.d5, r.a5, r.x) = self.r_012a8c(r.d0, r.d5, r.a4);
            self.q_obj_animate(r);
            self.q_obj_place(r);
        }
    }

    /// `func_012F18`: `func_004424` with 8, its offset by its rate
    /// (`func_001ABC`); once that isn't negative (landed), bit 7 of
    /// `BITS_A5C2` cleared, the offset 0 and back to walking
    /// (`func_012F34`).  Else placed.
    pub fn r_012f18(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        r.d0 = 8;
        r.c = false;
        (r.d0, r.x, r.n) = self.r_004424(r.d0, r.a4);
        (r.d0, r.x, r.n) = self.r_001abc(r.d0, r.a4);
        if !r.n {
            let b = self.read8(BITS_A5C2);
            self.write8(BITS_A5C2, b & 0x7F);
            self.write8(r.a4 + OBJ_OFFSET, 0);
            r.c = self.r_012f34(r.a4);
        } else {
            self.q_obj_place(r);
        }
    }

    /// `func_012F34`: bit 7 of `BITS_A582` cleared, the animation's
    /// counters 0, `func_012CBC` from now on.  Gives back C (clear).
    pub fn r_012f34(&mut self, a4: u32) -> bool {
        let b = self.read8(BITS_A582);
        self.write8(BITS_A582, b & 0x7F);
        self.write8(a4 + OBJ_TIMER, 0);
        self.write8(a4 + OBJ_STEP, 0);
        self.write32(a4 + OBJ_ROUTINE, WALKING);
        false
    }

    /// `func_012F4C`: the task `func_013068` (`func_0003E8`, priority
    /// 0x42); if it could be had, `COUNT_AD56`, `RADIUS` and the turn
    /// 0, the speed 0x200, and four tasks (`func_001222`: depth 0x80,
    /// priority 0x41, `func_012FCC`; as many as can be had) with flags
    /// 0x84, `TASK_18` 0x30 and angles 0, 0x40, 0x80, 0xC0, each with
    /// `func_004FF4`.  The registers back to the object.  Gives back
    /// d4, d5, d6, a4, a5, a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_012f4c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32) {
        let (mut d0, mut d5, mut a5) = (d0, d5, a5);
        let (mut d6, mut a6, _, none) = self.r_0003e8(d4, d5, d6, 0x42, CIRCLING_TASK);
        if !none {
            self.write8(COUNT_AD56, 0);
            self.write8(RADIUS, 0);
            self.write16(SPEED_965E, 0x200);
            self.write16(POSITION_965C, 0);
            self.write8(TO_MAKE, 4);
            self.write8(NEXT_ANGLE, 0);
            loop {
                let c;
                (d5, d6, a5, a6, _, c) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0x80, 0x41, CIRCLING);
                if c {
                    break;
                }
                self.write8(a5 + SPRITE_FLAGS, 0x84);
                self.write8(a6 + TASK_18, 0x30);
                let angle = self.read8(NEXT_ANGLE);
                self.write8(a6 + TASK_ANGLE, angle);
                d0 = low_byte(d0, angle.wrapping_add(0x40));
                self.write8(NEXT_ANGLE, d0 as u8);
                (d0, _, _) = self.r_004ff4(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
                let n = self.read8(TO_MAKE).wrapping_sub(1);
                self.write8(TO_MAKE, n);
                if n == 0 {
                    break;
                }
            }
        }
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        (d4, d5, d6, a4, a5, a6)
    }

    /// `func_012FCC` (a circling task): while `COUNT_AD56` isn't
    /// negative, frame 0xEA at the point `CENTRE_X`, `CENTRE_Y` plus
    /// `func_017780` and `func_01777C` of the angle (its own plus the
    /// turn's high byte, kept in `TURN_AABD`) by `RADIUS` (half of it
    /// for y), offset `HEIGHT_A5DC` less 0x30, placed (`func_001B94`,
    /// `func_001B74`).  Else it goes off: `TASK_15` the angle's quarter
    /// (negated, also into `TASK_18`), `func_0130AE`, rate 0xFD00,
    /// `func_01305E` from now on and sound 0x86.
    pub fn r_012fcc(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let count = self.read8(COUNT_AD56);
        r.d0 = low_byte(r.d0, count);
        if count & 0x80 != 0 {
            let quarter = self.read8(r.a6 + TASK_ANGLE).wrapping_neg() >> 6;
            r.d0 = low_byte(r.d0, quarter);
            self.write8(r.a6 + TASK_15, quarter);
            let b = self.read8(r.a6 + TASK_18);
            self.write8(r.a6 + TASK_18, b | quarter);
            (r.d0, r.a0, r.x, r.c) = self.r_0130ae(r.d6, r.a6);
            self.write16(r.a6 + TASK_RATE, 0xFD00);
            (r.x, r.c) = self.r_00053e(r.d6, CIRCLING_OFF);
            r.d0 = low_byte(r.d0, 0x86);
            r.c = false;
            self.r_0034b6(r.d0);
            return;
        }
        self.write8(r.a5 + SPRITE_FRAME, 0xEA);
        let angle = self.read8(POSITION_965C).wrapping_add(self.read8(r.a6 + TASK_ANGLE));
        r.d0 = low_byte(r.d0, angle);
        self.write8(TURN_AABD, angle);
        r.d1 = low_byte(r.d1, self.read8(RADIUS));
        r.d0 = self.r_017780(r.d0, r.d1, r.d3);
        let x = (r.d0 as u16).wrapping_add(self.read16(CENTRE_X));
        r.d0 = low_word(r.d0, x);
        self.write16(r.a6 + TASK_X, x);
        r.d1 = low_byte(r.d1, self.read8(RADIUS) >> 1);
        r.d0 = low_byte(r.d0, self.read8(TURN_AABD));
        r.d0 = self.r_01777c(r.d0, r.d1, r.d3);
        let y = (r.d0 as u16).wrapping_add(self.read16(CENTRE_Y));
        r.d0 = low_word(r.d0, y);
        self.write16(r.a6 + TASK_Y, y);
        let offset = self.read8(HEIGHT_A5DC).wrapping_sub(0x30);
        r.d0 = low_byte(r.d0, offset);
        self.write8(r.a6 + TASK_OFFSET, offset);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b74(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
    }
}
