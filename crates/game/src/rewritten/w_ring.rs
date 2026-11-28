//! Tasks of 0x180CA-0x18800 whose sprites are placed by an angle and a
//! radius (`func_017770`/`func_01776C` give a position about 0x80,
//! `func_017780`/`func_01777C` an offset): five made from a table
//! (`func_0185EC`, running `func_0184DA`), one that spirals out
//! (`func_018314`), ones placed about another task's sprite
//! (`func_01841A`, `func_0186E8`), the search for that task
//! (`func_0184A8`), sprites made at random places while `ACTIVE`
//! (`func_0183B8`, `func_0183A0`), the planes' scroll by an angle
//! (`func_01859A`) and the screen's set-up (`func_018772`); with
//! `func_0180CA` and `func_018306`, the ends of the tasks of
//! `func_0180DA` and `func_018008` past their waits.

use crate::{Game, Regs};

/// Set to 1 by `func_0180DA`, cleared by `func_018314` when its
/// animation ends; the tasks of `func_0183A0` and `func_01841A` end
/// when it is 0.
const ACTIVE: u32 = 0xFFFF_ADCF;
/// Counted up by `func_0184DA` for the task whose index is `CHOSEN`.
const COUNT_ADCE: u32 = 0xFFFF_ADCE;
/// The index compared with the tasks' (`func_0184DA`, `func_018712`)
/// and kept in `CHOSEN_COPY` (`func_0186B2`).
const CHOSEN: u32 = 0xFFFF_953A;
/// `CHOSEN` as `func_0186B2` found it; `func_0186E8` searches for the
/// task with this index.
const CHOSEN_COPY: u32 = 0xFFFF_ADD0;
/// `func_0184A8`'s search: the priority (`func_0185EC`: tasks from
/// this index on get radius `RADIUS_FAR`) and the index looked for.
const SEARCH_PRIORITY: u32 = 0xFFFF_80C4;
const SEARCH_INDEX: u32 = 0xFFFF_80C5;
/// The animation `func_018712` gives its tasks (a long).
const ANIM_80C8: u32 = 0xFFFF_80C8;
/// Bit 0 cleared by `func_0180CA`.
const FLAGS_9548: u32 = 0xFFFF_9548;
/// The index of the task running (`task.rs`).
const CURRENT: u32 = 0xFFFF_80EE;
/// The running task's sprite record (index and address).
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;
/// The planes' offsets (`h_scroll.rs`): plane A's across (a word; its
/// low byte `H_OFFSET_LOW`) and down (`V_OFFSET`, a word; its low byte
/// `V_OFFSET_LOW`).
const H_OFFSET: u32 = 0xFFFF_D002;
const H_OFFSET_LOW: u32 = 0xFFFF_D003;
const V_OFFSET: u32 = 0xFFFF_D402;
const V_OFFSET_LOW: u32 = 0xFFFF_D403;
/// Set by `func_018772`: a routine's address (`D_00ABA4`) and a long.
const LONG_A40A: u32 = 0xFFFF_A40A;
const LONG_8A5E: u32 = 0xFFFF_8A5E;
/// Cleared by `func_018772`.
const BYTE_A945: u32 = 0xFFFF_A945;
/// The list `func_0094D8` loads (a long).
const LOAD_LIST: u32 = 0xFFFF_92FA;

/// A task's fields.
/// Its sprite record's index (`func_001222`, `func_0186B2`).
const TASK_SLOT: u32 = 0x03;
/// The angle and the radius its sprite is placed by.
const TASK_ANGLE: u32 = 0x0C;
const TASK_RADIUS: u32 = 0x0D;
/// Its index among those made together (`func_0185EC`, `func_018712`).
const TASK_INDEX: u32 = 0x0E;
/// `func_018314`'s position (words: x, y).
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
/// Counted up by `func_01841A`; its bit 4 picks the frame.
const TASK_TICKS: u32 = 0x12;
/// `func_018314`'s angle, radius and the radius's fraction.
const SPIRAL_ANGLE: u32 = 0x16;
const SPIRAL_RADIUS: u32 = 0x17;
const SPIRAL_FRACTION: u32 = 0x18;
/// The task's animation (a long, `anim.rs`).
const TASK_ANIM: u32 = 0x20;

/// A sprite record's fields (`sprite.rs`).
const SPRITE_03: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;
/// The flags the sprites here are given.
const SHOWN: u8 = 0x81;

/// The tasks' routines (code addresses).
const RANDOM_TASK: u32 = 0x0183A0;
const TABLE_TASK: u32 = 0x0184DA;
const ANGLE_TASK: u32 = 0x0184FE;
const PLACED: u32 = 0x018564;
/// `func_0185EC`'s table (`D_0185CE`, in the ROM): 6 bytes a task, its
/// sprite's depth, its angle and its animation (a long).
const TABLE: u32 = 0x0185CE;
const TABLE_ANGLE: u32 = 0x0185CF;
const TABLE_ANIM: u32 = 0x0185D0;
/// Angles by index (`D_018824`, `D_01882A`: the chosen one's 5 on).
const ANGLES_18824: u32 = 0x018824;
const ANGLES_1882A: u32 = 0x01882A;
/// Animations (in the ROM).
const ANIM_DE228: u32 = 0x0DE228;
/// `func_018772`'s: the routine kept, the sprite set, the lists.
const ROUTINE_ABA4: u32 = 0x00ABA4;
const SET_DC3D8: u32 = 0x0DC3D8;
const LIST_18800: u32 = 0x018800;

/// The radius of `func_0185EC`'s tasks from `SEARCH_PRIORITY` on.
const RADIUS_FAR: u8 = 0x30;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// A table's entry by a register's low word (signed).
fn word_at(table: u32, index: u32) -> u32 {
    table.wrapping_add(index as u16 as i16 as u32)
}

impl Game {
    /// `func_001BB4` and, unless it stops the task, `FLAGS_9548`'s bit
    /// 0 cleared and `loc_0098F8`.
    fn w_stop_and_98f8(&mut self, r: &mut Regs) {
        r.unwound = false;
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        let flags = self.read8(FLAGS_9548);
        self.write8(FLAGS_9548, flags & !1);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0098f8(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_0180CA` (from `func_01808A`, past its wait): `func_001BB4`
    /// (the task may stop there), then `FLAGS_9548`'s bit 0 cleared
    /// and on to `loc_0098F8`.  Gives back its registers.  Runs in
    /// exit-5-boss; one call (`loc_0098F8`'s, for frames) differs in
    /// the check as its translation does.
    pub fn r_0180ca(&mut self, regs: &mut Regs) {
        self.w_stop_and_98f8(regs);
    }

    /// `func_018306` (the end of `func_0180DA`'s task): `func_01793A`'s
    /// wait (which may return for the task), then `func_015C52`.
    /// Gives back the registers as they are left.  Runs in ending; one
    /// call (`func_015C52`'s, for frames) differs in the check as its
    /// translation does.
    pub fn r_018306(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let waiting;
        (regs.d6, regs.a6, regs.x, regs.c, waiting) = self.r_01793a(regs.d6);
        if waiting {
            return;
        }
        let r = &mut *regs;
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_015c52(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_018314`, a task: its position moves by its angle and
    /// radius (`func_017780` across, `func_01777C` down), the radius
    /// grows by 0x38/0x100 and the angle turns back 3 a frame; its
    /// sprite placed (`func_001B7E`) and its animation a frame on
    /// (`func_0013D6`); when that ends, `ACTIVE` is cleared and the
    /// task ends.  Gives back d0, d1, d2, d5, d6, a0, a5, X and C.
    pub fn r_018314(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let task = regs.a6;
        let r = &mut *regs;
        r.d0 = low_byte(r.d0, self.read8(task + SPIRAL_ANGLE));
        r.d1 = low_byte(r.d1, self.read8(task + SPIRAL_RADIUS));
        r.d0 = self.r_017780(r.d0, r.d1, r.d3);
        let x = self.read16(task + TASK_X).wrapping_add(r.d0 as u16);
        self.write16(task + TASK_X, x);
        r.d0 = low_byte(r.d0, self.read8(task + SPIRAL_ANGLE));
        r.d1 = low_byte(r.d1, self.read8(task + SPIRAL_RADIUS));
        r.d0 = self.r_01777c(r.d0, r.d1, r.d3);
        let y = self.read16(task + TASK_Y).wrapping_add(r.d0 as u16);
        self.write16(task + TASK_Y, y);
        let (fraction, carry) = self.read8(task + SPIRAL_FRACTION).overflowing_add(0x38);
        self.write8(task + SPIRAL_FRACTION, fraction);
        let radius = self.read8(task + SPIRAL_RADIUS).wrapping_add(u8::from(carry));
        r.d0 = low_byte(r.d0, radius);
        r.d1 = low_byte(r.d1, 0);
        self.write8(task + SPIRAL_RADIUS, radius);
        let angle = self.read8(task + SPIRAL_ANGLE);
        self.write8(task + SPIRAL_ANGLE, angle.wrapping_sub(3));
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, task);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, task);
        if r.c {
            self.write8(ACTIVE, 0);
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
        }
    }

    /// `func_0183A0`, a task: ends when `ACTIVE` is 0; else its
    /// animation a frame on (`func_0013D6`), and when that ends the
    /// task ends with its sprite (`func_00121A`) and another is made
    /// (`func_0183B8`).  Gives back the registers as they are left.
    pub fn r_0183a0(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        if self.read8(ACTIVE) == 0 {
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
            return;
        }
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if !r.c {
            self.r_018418();
            return;
        }
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        (r.d0, r.d1, r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_0183b8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_0183B8`: a task running `RANDOM_TASK` (priority 0xB4,
    /// sprite depth 0x32) with animation `ANIM_DE228`, its sprite at a
    /// random place: across 0x100 + 0-0x3F + `H_OFFSET_LOW`, down 0xD0
    /// + 0-0x1F - `V_OFFSET_LOW` (`func_001BE2`; the high bytes by the
    /// borrows).  Gives back d0, d1, d5, d6, a5, a6, X (the last
    /// borrow) and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0183b8(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        let (d5, d6, sprite, task, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0x32, 0xB4, RANDOM_TASK);
        self.write32(task + TASK_ANIM, ANIM_DE228);
        self.write8(sprite + SPRITE_FLAGS, SHOWN);
        // across: the random number less the negated offset
        let (d0, d1, _) = self.r_001be2(d1);
        let random = d0 as u8 & 0x3F;
        let minus = self.read8(H_OFFSET_LOW).wrapping_neg();
        let (low, borrow) = random.overflowing_sub(minus);
        self.write8(sprite + SPRITE_X_LOW, low);
        self.write8(sprite + SPRITE_X, 1u8.wrapping_sub(u8::from(borrow)));
        // down (d1's low byte cleared for the borrow's subx)
        let (d0, d1, _) = self.r_001be2(low_byte(d1, 0));
        let random = (d0 as u8 & 0x1F).wrapping_add(0xD0);
        let (low, borrow) = random.overflowing_sub(self.read8(V_OFFSET_LOW));
        self.write8(sprite + SPRITE_Y_LOW, low);
        let high = 0u8.wrapping_sub(u8::from(borrow));
        self.write8(sprite + SPRITE_Y, high);
        self.r_018418();
        (low_byte(d0, high), low_byte(d1, 0), d5, d6, sprite, task, borrow, false)
    }

    /// `func_018418`: an `rts`.
    pub fn r_018418(&mut self) {}

    /// `func_01841A`: the sprite of the task running placed about the
    /// sprite of the task of priority 0x64 and index 0 (`func_0184A8`)
    /// by its angle and radius; its byte 3 `TASK_INDEX` + 0x7A, its
    /// frame 0x35 or 0x36 by bit 4 of `TASK_TICKS` (counted up); the
    /// task ends when `ACTIVE` is 0.
    /// Nothing in the code calls it: reviewed by reading.
    pub fn r_01841a(&mut self, d0: u32, d3: u32, d5: u32, d6: u32) {
        self.write8(SEARCH_PRIORITY, 0x64);
        let (_, d6, found) = self.r_0184a8(low_byte(d0, 0), d6);
        let d6 = low_byte(d6, self.read8(found + TASK_SLOT));
        let (d6, other, _) = self.r_000b48(d6);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let sprite = self.read32(SLOT_ADDR);
        let x = self.read8(other + SPRITE_X_LOW);
        self.write8(sprite + SPRITE_X_LOW, x);
        let y = self.read8(other + SPRITE_Y_LOW);
        self.write8(sprite + SPRITE_Y_LOW, y);
        self.write8(sprite + SPRITE_X, 0);
        self.write8(sprite + SPRITE_Y, 0);
        let d6 = low_byte(d6, self.read8(CURRENT));
        let (d6, task, _) = self.r_0005f8(d6);
        let angle = self.read8(task + TASK_ANGLE);
        let radius = u32::from(self.read8(task + TASK_RADIUS));
        let d0 = self.r_017780(u32::from(angle), radius, d3);
        let x = self.read16(sprite + SPRITE_X).wrapping_add(d0 as u16);
        self.write16(sprite + SPRITE_X, x);
        let d0 = self.r_01777c(low_byte(d0, angle), radius, d3);
        let y = self.read16(sprite + SPRITE_Y).wrapping_add(d0 as u16);
        self.write16(sprite + SPRITE_Y, y);
        let index = self.read8(task + TASK_INDEX);
        self.write8(sprite + SPRITE_03, index.wrapping_add(0x7A));
        let ticks = self.read8(task + TASK_TICKS).wrapping_add(1);
        self.write8(task + TASK_TICKS, ticks);
        self.write8(sprite + SPRITE_FRAME, ((ticks >> 4) & 1) + 0x35);
        if self.read8(ACTIVE) == 0 {
            // the frame's add carried nothing: X clear
            self.r_000560(d5, d6, false);
        }
    }

    /// `func_0184A8`: the task list from its head, by the next index
    /// and address, to the task whose priority is `SEARCH_PRIORITY` and
    /// whose `TASK_INDEX` is d0's low byte (kept in `SEARCH_INDEX`), or
    /// to the tail (d0's low byte 0xFF).  Gives back d0, d6 (the task's
    /// index in its low byte, the low word's high byte clear) and a6.
    pub fn r_0184a8(&mut self, d0: u32, d6: u32) -> (u32, u32, u32) {
        const NEXT: u32 = 0x00;
        const PRIORITY: u32 = 0x02;
        const NEXT_ADDR: u32 = 0x08;
        let wanted = d0 as u8;
        self.write8(SEARCH_INDEX, wanted);
        let mut d0 = d0;
        let mut d6 = d6 & 0xFFFF_0000;
        loop {
            let (_, task, _) = self.r_0005f8(d6);
            let next = self.read8(task + NEXT);
            d0 = low_byte(d0, next);
            if next == 0xFF {
                return (d0, d6, task);
            }
            let task = self.read32(task + NEXT_ADDR);
            d6 = low_byte(d6, next);
            let priority = self.read8(task + PRIORITY);
            d0 = low_byte(d0, priority);
            if priority != self.read8(SEARCH_PRIORITY) {
                continue;
            }
            let index = self.read8(task + TASK_INDEX);
            d0 = low_byte(d0, index);
            if index == self.read8(SEARCH_INDEX) {
                return (d0, d6, task);
            }
        }
    }

    /// `func_0184DA`, a task (`func_0185EC`'s): its sprite placed by its
    /// angle and radius (`func_018564`); `COUNT_ADCE` counted up when
    /// its index is `CHOSEN` (its bit 4 then tested for nothing: both
    /// ways go on); its animation a frame on (`func_0013D6`).  Gives
    /// back d0, d1, d2, d5, a0, a5, X and C.
    pub fn r_0184da(&mut self, regs: &mut Regs) {
        self.r_018564(regs);
        let r = &mut *regs;
        let index = self.read8(r.a6 + TASK_INDEX);
        r.d0 = low_byte(r.d0, index);
        if index == self.read8(CHOSEN) {
            let n = self.read8(COUNT_ADCE);
            self.write8(COUNT_ADCE, n.wrapping_add(1));
        }
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `func_018564`, `PLACED`: the running task's sprite at the
    /// position of its angle and radius about 0x80 (`func_017770`
    /// across, `func_01776C` down; the high bytes clear).  Gives back
    /// d0, d1, d5, a5, X (`func_01776C`'s) and C (clear).
    pub fn r_018564(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let task = regs.a6;
        let r = &mut *regs;
        r.d0 = low_byte(r.d0, self.read8(task + TASK_ANGLE));
        r.d1 = low_byte(r.d1, self.read8(task + TASK_RADIUS));
        (r.d0, r.x) = self.r_017770(r.d0, r.d1, r.d3);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        self.write8(r.a5 + SPRITE_X_LOW, r.d0 as u8);
        self.write8(r.a5 + SPRITE_X, 0);
        self.write8(r.a5 + SPRITE_Y, 0);
        r.d0 = low_byte(r.d0, self.read8(task + TASK_ANGLE));
        r.d1 = low_byte(r.d1, self.read8(task + TASK_RADIUS));
        (r.d0, r.x) = self.r_01776c(r.d0, r.d1, r.d3);
        self.write8(r.a5 + SPRITE_Y_LOW, r.d0 as u8);
        r.c = false;
    }

    /// `func_01859A`: the planes scrolled by the task's angle and
    /// radius: `H_OFFSET` the negated position across (`func_017770`,
    /// 0-0xFF), `V_OFFSET_LOW` the one down (`func_01776C`); then the
    /// scroll tables' first words (`func_008664`).  Gives back d0 and
    /// d1.
    #[allow(clippy::too_many_arguments)]
    pub fn r_01859a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32) {
        let angle = self.read8(a6 + TASK_ANGLE);
        let radius = self.read8(a6 + TASK_RADIUS);
        let d1 = low_byte(d1, radius);
        let (d0, _) = self.r_017770(low_byte(d0, angle), d1, d3);
        let across = (d0 as u16 & 0xFF).wrapping_neg();
        let d0 = low_word(d0, across);
        self.write16(H_OFFSET, across);
        let (d0, x) = self.r_01776c(low_byte(d0, angle), d1, d3);
        self.write8(V_OFFSET_LOW, d0 as u8);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        self.r_008664(&mut r);
        (r.d0, d1)
    }

    /// `func_0185CC`: an `rts`.
    pub fn r_0185cc(&mut self) {}

    /// `func_0185EC`: `func_0002DA` and `func_00A506` (4), then five
    /// tasks running `TABLE_TASK` (priority 0x64), each with its depth,
    /// angle and animation from `TABLE`, its index in `TASK_INDEX`, its
    /// radius 0, or `RADIUS_FAR` from `SEARCH_PRIORITY` on.  Gives back
    /// d2, d5, d6, a1 and a5.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0185ec(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32) {
        self.r_0002da();
        let (_, a1, _, _) = self.r_00a506(4, d1, d2, d3, d4, d5, d6, d7, a2, a3, a4, a5);
        let (mut d2, mut d5, mut d6, mut a5) = (d2 & 0xFFFF_0000, d5, d6, a5);
        loop {
            let index = d2 as u8;
            // mulu: the entry's offset fills d0
            let d0 = (d2 & 0xFFFF) * 6;
            let depth = self.read8(word_at(TABLE, d0));
            let task;
            (d5, d6, a5, task, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, depth, 0x64, TABLE_TASK);
            self.write8(a5 + SPRITE_FLAGS, SHOWN);
            self.write8(task + TASK_INDEX, index);
            let angle = self.read8(word_at(TABLE_ANGLE, d0));
            self.write8(task + TASK_ANGLE, angle);
            let anim = self.read32(word_at(TABLE_ANIM, d0));
            self.write32(task + TASK_ANIM, anim);
            self.write8(task + TASK_RADIUS, 0);
            if index >= self.read8(SEARCH_PRIORITY) {
                self.write8(task + TASK_RADIUS, RADIUS_FAR);
            }
            let index = index.wrapping_add(1);
            d2 = low_byte(d2, index);
            if index == 5 {
                return (d2, d5, d6, a1, a5);
            }
        }
    }

    /// `func_01865E`: a task running `PLACED` (priority 0x64, sprite
    /// depth 0x64) at angle 0x4D, radius 0; its sprite's frame 0x10.
    /// Gives back d5, d6, a5 and a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_01865e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32) {
        let (d5, d6, sprite, task, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0x64, 0x64, PLACED);
        self.write8(task + TASK_ANGLE, 0x4D);
        self.write8(task + TASK_RADIUS, 0);
        self.write8(sprite + SPRITE_FRAME, 0x10);
        self.write8(sprite + SPRITE_FLAGS, SHOWN);
        (d5, d6, sprite, task)
    }

    /// `func_01868A`, a task: its sprite at 0x150 less plane A's
    /// scroll across (0-0xFF), 0xB8 less `V_OFFSET_LOW` down; its
    /// animation a frame on (`func_0013D6`).  Gives back d0, d1, d2,
    /// d5, a0, a5, X and C.
    pub fn r_01868a(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        let scroll = self.read16(H_OFFSET).wrapping_neg() & 0xFF;
        r.d1 = low_word(r.d1, scroll);
        r.d0 = low_word(r.d0, 0x150u16.wrapping_sub(scroll));
        self.write16(r.a5 + SPRITE_X, r.d0 as u16);
        let y = 0xB8u8.wrapping_sub(self.read8(V_OFFSET_LOW));
        r.d0 = low_byte(r.d0, y);
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        self.write8(r.a5 + SPRITE_Y, 0);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `func_0186B2`: a sprite record at depth 0x32 (`func_000796`) for
    /// the task running (its index in `TASK_SLOT`), frame 0xD; `CHOSEN`
    /// kept in `CHOSEN_COPY` and the task's angle from `ANGLES_18824`
    /// by it.  Gives back d0 (`CHOSEN` sign-extended to the word), d5,
    /// d6, a0, a5 and a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0186b2(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32) {
        let d6 = low_byte(d6, self.read8(CURRENT));
        let (d6, task, x) = self.r_0005f8(d6);
        let (d5, sprite, _, _) = self.r_000796(0x32, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        self.write8(task + TASK_SLOT, d5 as u8);
        self.write8(sprite + SPRITE_FRAME, 0xD);
        self.write8(sprite + SPRITE_FLAGS, SHOWN);
        let chosen = self.read8(CHOSEN);
        self.write8(CHOSEN_COPY, chosen);
        let d0 = u32::from(chosen as i8 as u16);
        let angle = self.read8(word_at(ANGLES_18824, d0));
        self.write8(task + TASK_ANGLE, angle);
        (d0, d5, d6, ANGLES_18824, sprite, task)
    }

    /// `func_0186E8`: the running task's radius the radius of the task
    /// of priority 0x64 and index `CHOSEN_COPY` (`func_0184A8`) + 0x6E,
    /// its sprite placed (`func_018564`).  Gives back d0, d1, d5, d6,
    /// a5, a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0186e8(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        self.write8(SEARCH_PRIORITY, 0x64);
        let wanted = self.read8(CHOSEN_COPY);
        let (d0, d6, found) = self.r_0184a8(low_byte(d0, wanted), d6);
        let radius = self.read8(found + TASK_RADIUS).wrapping_add(0x6E);
        let d0 = low_byte(d0, radius);
        let d6 = low_byte(d6, self.read8(CURRENT));
        let (d6, a6, x) = self.r_0005f8(d6);
        self.write8(a6 + TASK_RADIUS, radius);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        self.r_018564(&mut r);
        (r.d0, r.d1, r.d5, r.d6, r.a5, r.a6, r.x, r.c)
    }

    /// `func_018712`: for d0's low byte below 1 (index 0 only: the loop
    /// back to the start tests the next index against 1 again), a task
    /// running `ANGLE_TASK` (priority 0xC8, sprite depth 0x32) with the
    /// animation in `ANIM_80C8`, the index, and its angle from
    /// `ANGLES_1882A` by it (5 on for the `CHOSEN` one).
    /// Nothing in the code calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments)]
    pub fn r_018712(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) {
        let (mut d0, mut d5, mut d6, mut a5) = (d0, d5, d6, a5);
        while (d0 as u8) < 1 {
            let task;
            (d5, d6, a5, task, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0x32, 0xC8, ANGLE_TASK);
            let anim = self.read32(ANIM_80C8);
            self.write32(task + TASK_ANIM, anim);
            self.write8(a5 + SPRITE_FLAGS, SHOWN);
            let index = d0 as u8;
            self.write8(task + TASK_INDEX, index);
            let entry = if index == self.read8(CHOSEN) { index.wrapping_add(5) } else { index };
            let angle = self.read8(word_at(ANGLES_1882A, u32::from(entry as i8 as u16)));
            self.write8(task + TASK_ANGLE, angle);
            d0 = low_byte(d0, index.wrapping_add(1));
            if index.wrapping_add(1) >= 5 {
                return;
            }
        }
    }

    /// `func_018772`: the screen set up: the scroll tables cleared
    /// (`func_008624`, offsets 0, `func_008664`), the window
    /// (`func_00B4C6`), `LONG_A40A` = `ROUTINE_ABA4`, `LONG_8A5E` =
    /// 0xFE00, sprite set 1 from `SET_DC3D8` (`func_000776`) with its
    /// tiles (`func_000752`: 0, 0, 7), `func_0002E0`, `func_00033A`,
    /// `func_000302`; `BYTE_A945` cleared, the list `LIST_18800` loaded
    /// (`func_0094D8`), and the sprites (`func_000704`).  Gives back
    /// the registers and X as they are left.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_018772(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        self.r_008624();
        self.write16(H_OFFSET, 0);
        self.write16(V_OFFSET, 0);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        self.r_008664(&mut r);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        self.r_00b4c6(&mut r);
        self.write32(LONG_A40A, ROUTINE_ABA4);
        self.write32(LONG_8A5E, 0xFE00);
        let d0 = self.r_000776(1, SET_DC3D8, a1);
        let d0 = low_word(d0, 0);
        let d1 = low_word(d1, 0);
        let d2 = 7;
        self.r_000752(d0, d1, d2);
        self.r_0002e0();
        self.r_00033a();
        let (a0, x) = self.r_000302();
        self.write8(BYTE_A945, 0);
        self.write32(LOAD_LIST, LIST_18800);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, _) = self.r_0094d8(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x);
        }
        let (d0, d1, a0, x) = self.r_000704(d0);
        (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x)
    }
}
