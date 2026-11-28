//! The steps `func_0093F0` runs through `STATE` (0x120F8-0x12338), and
//! `func_01208E`, a task's routine.  Each step gives back d0: 1
//! while it goes on (or has moved `STATE` to the next), 0 when the
//! steps are back at their start (`func_0120F8`, `func_01220E`) without
//! anything to do.
//!
//! Two chains of steps, each started by bit 7 of `PAD_AD02`:
//!
//! - `func_0120F8` .. `func_0121F2`: sound 0x26 and the Z80's command
//!   0xFC; three words set under the interrupt mask, `BUSY` set, waited
//!   on (cleared elsewhere); once the VDP's line is 0x18 or more,
//!   `func_0094D8` on `LIST_12204`; then, on the bit again, sound 0x27,
//!   command 0xFD, the three words the other way round, waited on, and
//!   back to the start.
//! - `func_01220E` .. `func_0122FA`: a task with a sprite moved up by
//!   `VELOCITY_Y` 0x600, waits (its frame now and then picked by the
//!   random numbers), then on the bit moved down (0xF800) and ended.

use crate::{Game, Regs};

/// The routine `func_0093F0` calls (a long): the step.
const STATE: u32 = 0xFFFF_80E0;
/// Bit 7: the next step.
const PAD_AD02: u32 = 0xFFFF_AD02;
const BIT_NEXT: u8 = 7;
/// Bits `func_01208E` sets bit 3 of.
const MODE: u32 = 0xFFFF_9540;
const MODE_BIT3: u8 = 1 << 3;
/// Set to 1 after the three words `WORD_A946` .. `WORD_A94A` (with
/// the interrupts masked); the steps wait for it to be 0 again.
const BUSY: u32 = 0xFFFF_A944;
/// Set to 1 with `BUSY`.
const BUSY_A945: u32 = 0xFFFF_A945;
const WORD_A946: u32 = 0xFFFF_A946;
const WORD_A948: u32 = 0xFFFF_A948;
const WORD_A94A: u32 = 0xFFFF_A94A;
/// The VDP's H/V counter: its high byte the line.
const HV_COUNTER: u32 = 0xC00008;
/// The line the steps wait for.
const LINE: u8 = 0x18;
/// The list `func_0094D8` reads (a long), and the one the steps give
/// it, in the ROM.
const LIST: u32 = 0xFFFF_92FA;
const LIST_12204: u32 = 0x012204;
/// The task's index `func_01220E` keeps (d6's low byte).
const TASK_INDEX: u32 = 0xFFFF_80E5;
/// The velocity's y, a word (`a_motion.rs`).
const VELOCITY_Y: u32 = 0xFFFF_9EC6;
/// The running object's sprite record (`a_objects.rs`), the task's
/// sprite record's when the task has ended.
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// A task's fields (`task.rs`): its position (words), its counter
/// (0x16) and a byte flipped between 0 and 1 (0x17).
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_COUNT: u32 = 0x16;
const TASK_FLIP: u32 = 0x17;
/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// The steps (`STATE`).
const STEP_0120F8: u32 = 0x0120F8;
const STEP_012122: u32 = 0x012122;
const STEP_01215C: u32 = 0x01215C;
const STEP_01218C: u32 = 0x01218C;
const STEP_0121B2: u32 = 0x0121B2;
const STEP_0121F2: u32 = 0x0121F2;
const STEP_01220E: u32 = 0x01220E;
const STEP_012276: u32 = 0x012276;
const STEP_0122A4: u32 = 0x0122A4;
const STEP_0122FA: u32 = 0x0122FA;
/// The task's routine `func_01208E` sets.
const TASK_0120B4: u32 = 0x0120B4;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// Whether bit 7 of `PAD_AD02` is set.
    fn q_next_pressed(&mut self) -> bool {
        self.read8(PAD_AD02) & (1 << BIT_NEXT) != 0
    }

    /// Sound `sound` (`func_0034B6`) and, if `command`, the Z80's
    /// command (`func_00349E`); d0's low byte the last.
    fn q_sound(&mut self, r: &mut Regs, sound: u8, command: Option<u8>) {
        r.d0 = low_byte(r.d0, sound);
        self.r_0034b6(r.d0);
        if let Some(command) = command {
            r.d0 = low_byte(r.d0, command);
            self.r_00349e(r.d0);
        }
    }

    /// The three words (`WORD_A948`, `WORD_A94A`, `WORD_A946`) set with
    /// the interrupts masked, `BUSY` with them; then `BUSY_A945`.
    fn q_set_words(&mut self, a948: u16, a94a: u16, a946: u16) {
        let ipl = self.ipl();
        self.set_ipl(ipl | 7);
        self.write16(WORD_A948, a948);
        self.write16(WORD_A94A, a94a);
        self.write16(WORD_A946, a946);
        self.write8(BUSY, 1);
        self.set_ipl(ipl);
        self.write8(BUSY_A945, 1);
    }

    /// One read of the VDP's line (d0's low word: the counter shifted
    /// down 8; X the last bit out); whether it is `LINE` or more.
    fn q_line_reached(&mut self, r: &mut Regs) -> bool {
        let hv = self.read16(HV_COUNTER);
        r.d0 = (r.d0 & 0xFFFF_0000) | u32::from(hv >> 8);
        r.x = hv & 0x80 != 0;
        (hv >> 8) as u8 >= LINE
    }

    /// `func_01208E`: bit 3 of `MODE` set, `func_0120B4` the running
    /// task's routine (`func_00053E`), `func_00A51A` with 4, d1 0x10 and the Z80's
    /// command 0xFE.  Gives back the registers as the original leaves
    /// them (C clear).
    pub fn r_01208e(&mut self, regs: &mut Regs) {
        let r = regs;
        let mode = self.read8(MODE);
        self.write8(MODE, mode | MODE_BIT3);
        (r.x, r.c) = self.r_00053e(r.d6, TASK_0120B4);
        r.d0 = 4;
        (r.d0, r.a1, r.x, r.c) = self.r_00a51a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        r.d1 = 0x10;
        r.d0 = low_byte(r.d0, 0xFE);
        r.c = false;
        self.r_00349e(r.d0);
        r.unwound = false;
    }

    /// `func_0120F8`: on bit 7 of `PAD_AD02`, sound 0x26, the Z80's
    /// command 0xFC and `func_012122` next (d0 1); else d0 0.
    pub fn r_0120f8(&mut self, regs: &mut Regs) {
        if self.q_next_pressed() {
            self.write32(STATE, STEP_012122);
            self.q_sound(regs, 0x26, Some(0xFC));
            regs.d0 = 1;
        } else {
            regs.d0 = 0;
        }
        regs.unwound = false;
    }

    /// `func_012122`: once `BUSY` is 0, the words 0xE000, 0x100, 0xFD00
    /// (`q_set_words`) and `func_01215C` next.  d0 1.
    pub fn r_012122(&mut self, regs: &mut Regs) {
        if self.read8(BUSY) == 0 {
            self.q_set_words(0xE000, 0x100, 0xFD00);
            self.write32(STATE, STEP_01215C);
        }
        regs.d0 = 1;
        regs.unwound = false;
    }

    /// `func_01215C`: once `BUSY` is 0, waits for the VDP's line
    /// `LINE` (looking at `BUSY` again between reads), then
    /// `func_0094D8` on `LIST_12204` and `func_01218C` next.  d0 1.
    pub fn r_01215c(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        loop {
            if self.read8(BUSY) != 0 {
                break;
            }
            if !self.q_line_reached(r) {
                self.pass(78);
                continue;
            }
            self.write32(LIST, LIST_12204);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
                self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
            if self.restarted.is_some() {
                return;
            }
            self.write32(STATE, STEP_01218C);
            break;
        }
        r.d0 = 1;
    }

    /// `func_01218C`: on bit 7 of `PAD_AD02`, `func_0121B2` next, sound
    /// 0x27 and the Z80's command 0xFD.  d0 1.
    pub fn r_01218c(&mut self, regs: &mut Regs) {
        if self.q_next_pressed() {
            self.write32(STATE, STEP_0121B2);
            self.q_sound(regs, 0x27, Some(0xFD));
        }
        regs.d0 = 1;
        regs.unwound = false;
    }

    /// `func_0121B2`: waits for the VDP's line `LINE`, then the words
    /// 0xFD00, 0x100, 0xE000 (`q_set_words`) and `func_0121F2` next.
    /// d0 1.
    pub fn r_0121b2(&mut self, regs: &mut Regs) {
        while !self.q_line_reached(regs) {
            self.pass(56);
        }
        self.q_set_words(0xFD00, 0x100, 0xE000);
        self.write32(STATE, STEP_0121F2);
        regs.d0 = self.r_0121ee();
        regs.unwound = false;
    }

    /// `func_0121EE` (the end of `func_0121B2` and `func_0121F2`): d0 1.
    pub fn r_0121ee(&mut self) -> u32 {
        1
    }

    /// `func_0121F2`: once `BUSY` is 0, back to `func_0120F8` (d0 0);
    /// d0 1 until then.
    pub fn r_0121f2(&mut self, regs: &mut Regs) {
        if self.read8(BUSY) != 0 {
            regs.d0 = self.r_0121ee();
        } else {
            self.write32(STATE, STEP_0120F8);
            regs.d0 = 0;
        }
        regs.unwound = false;
    }

    /// `func_01220E`: on bit 7 of `PAD_AD02`, a task (`func_001222`:
    /// depth 3, priority 0x80, `func_012276`; nothing more if there is
    /// none) at (0x88, 0xFFC0), counting 0x18, its sprite's frame 0x19
    /// and flags 0x84, placed (`func_001B7E`); its index in
    /// `TASK_INDEX`; `func_012276` next, sound 0x26 and the Z80's
    /// command 0xFC, d0 1.  Else d0 0.
    pub fn r_01220e(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        if self.q_next_pressed() {
            (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
                self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 3, 0x80, STEP_012276);
            if !r.c {
                self.write8(TASK_INDEX, r.d6 as u8);
                self.write16(r.a6 + TASK_X, 0x88);
                self.write16(r.a6 + TASK_Y, 0xFFC0);
                self.write8(r.a6 + TASK_COUNT, 0x18);
                self.write8(r.a5 + SPRITE_FRAME, 0x19);
                self.write8(r.a5 + SPRITE_FLAGS, 0x84);
                (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
                self.write32(STATE, STEP_012276);
                self.q_sound(r, 0x26, Some(0xFC));
                r.d0 = 1;
                return;
            }
        }
        r.d0 = 0;
    }

    /// The task moved by `VELOCITY_Y` `velocity` (`func_001AA2`) and
    /// placed (`func_001B7E`), its `TASK_COUNT` counted down; whether
    /// it got to 0.
    fn q_task_moved(&mut self, r: &mut Regs, velocity: u16) -> bool {
        self.write16(VELOCITY_Y, velocity);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        let count = self.read8(r.a6 + TASK_COUNT);
        self.write8(r.a6 + TASK_COUNT, count.wrapping_sub(1));
        r.x = count == 0;
        r.c = r.x;
        count == 1
    }

    /// `func_012276`: the task moved down the screen by 0x600 a frame;
    /// once counted out, `func_0122A4` next (counting 0x20, `TASK_FLIP`
    /// 0).  d0 1.
    pub fn r_012276(&mut self, regs: &mut Regs) {
        if self.q_task_moved(regs, 0x600) {
            self.write32(STATE, STEP_0122A4);
            self.write8(regs.a6 + TASK_COUNT, 0x20);
            self.write8(regs.a6 + TASK_FLIP, 0);
        }
        regs.d0 = 1;
        regs.unwound = false;
    }

    /// `func_0122A4`: on bit 7 of `PAD_AD02`, counting 0x18,
    /// `func_0122FA` next and sound 0x27.  Else the sprite's frame
    /// 0x19, and each time `TASK_COUNT` runs out `TASK_FLIP` flips and
    /// the count is 6, or, when the flip is back to 0, a random number
    /// (`func_001BE2`) with bits 0-2 set and bit 7 clear.  d0 1.
    pub fn r_0122a4(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.q_next_pressed() {
            self.write8(r.a6 + TASK_COUNT, 0x18);
            self.write32(STATE, STEP_0122FA);
            self.q_sound(r, 0x27, None);
        } else {
            self.write8(r.a5 + SPRITE_FRAME, 0x19);
            let count = self.read8(r.a6 + TASK_COUNT).wrapping_sub(1);
            self.write8(r.a6 + TASK_COUNT, count);
            if count == 0 {
                self.write8(r.a6 + TASK_COUNT, 6);
                r.d0 = 1;
                let flip = self.read8(r.a6 + TASK_FLIP) ^ 1;
                self.write8(r.a6 + TASK_FLIP, flip);
                if flip == 0 {
                    (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
                    let count = ((r.d0 as u8) | 7) & 0x7F;
                    r.d0 = low_byte(r.d0, count);
                    self.write8(r.a6 + TASK_COUNT, count);
                }
            }
        }
        r.d0 = 1;
        r.unwound = false;
    }

    /// `func_0122FA`: the task moved up by 0xF800 a frame; once counted
    /// out, the task `d6` ends (`func_000564`), its sprite record
    /// (`func_000B5A` with d5) goes to `SLOT_INDEX`, `SLOT_ADDR` and is
    /// freed (`func_000890`), the Z80's command 0xFD and back to
    /// `func_01220E`.  d0 1.
    pub fn r_0122fa(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.q_task_moved(r, 0xF800) {
            (r.x, r.c) = self.r_000564(r.d5, r.d6, r.x);
            (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
            self.write8(SLOT_INDEX, r.d5 as u8);
            self.write32(SLOT_ADDR, r.a5);
            r.c = self.r_000890();
            r.d0 = low_byte(r.d0, 0xFD);
            self.r_00349e(r.d0);
            self.write32(STATE, STEP_01220E);
        }
        r.d0 = 1;
        r.unwound = false;
    }
}
