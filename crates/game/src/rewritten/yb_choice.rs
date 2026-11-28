//! The task of the choice of `CHOICE` (0x18876-0x18AFA; its helpers
//! and ends are in `w_choice.rs`): `func_018876` loads the screen
//! (`func_018D1C`) and makes the task `func_01966A`; `func_018896` makes
//! the sprites and tasks of the entries (`func_018B8C`, `func_018C24`,
//! one `func_018C1A` for each entry whose bit of `ENTRY_BITS` is
//! clear) and takes the first set bit's entry as `CHOICE`, or, with
//! `ROW` below `LIMIT`, loads `LIST_196F6`; then (`func_0189DC`) a
//! sprite slides in by `DOWN` and `ACROSS`, and `func_018A30` moves
//! `CHOICE` with the pad's bits 2 and 0/3, takes bit 1 for the last
//! entry and ends with any of bits 5-7.  Run in the exit-* scripts.

use crate::rewritten::y_screen::{low_byte, low_word};
use crate::{Game, Regs};

/// The byte compared with `LIMIT` (`w_choice.rs`).
const ROW: u32 = 0xFFFF_953A;
const LIMIT: u32 = 0xFFFF_953B;
/// The entry chosen (`CHOICE_LAST`: the last one).
const CHOICE: u32 = 0xFFFF_953C;
/// Bits of entries: a set bit makes no `func_018C1A` task.
const ENTRY_BITS: u32 = 0xFFFF_953D;
/// The counts `func_018B3C` sets: `CHOICE` stays below `COUNT_ADD2`,
/// the entries are `COUNT_ADD3`.
const COUNT_ADD2: u32 = 0xFFFF_ADD2;
const COUNT_ADD3: u32 = 0xFFFF_ADD3;
/// The sprites' place (`func_018A0A`): across and down.
const ACROSS: u32 = 0xFFFF_ADD4;
const DOWN: u32 = 0xFFFF_ADD5;
/// Counted up by `func_018B8C`; cleared when `CHOICE` moves.
const BLINK: u32 = 0xFFFF_ADD6;
/// The entry's offset in `OFFSETS` (by 2) and the entry's bit, while
/// `func_018896` makes the entries' tasks.
const OFFSET_80C4: u32 = 0xFFFF_80C4;
const BIT_80C5: u32 = 0xFFFF_80C5;
/// The pad's byte (`i_main.rs`).
const PAD: u32 = 0xFFFF_AD02;
/// Cleared by `func_018876`.
const BYTE_80DF: u32 = 0xFFFF_80DF;
/// Set to 2 when the last entry is taken.
const BYTE_954F: u32 = 0xFFFF_954F;
/// Counted up when the last entry is taken (`w_choice.rs`).
const COUNT_A93C: u32 = 0xFFFF_A93C;

/// The pad's bits: 2 moves `CHOICE` back, 0 or 3 on, 1 picks
/// `CHOICE_LAST`, any of 5-7 takes the choice.
const PAD_BACK: u8 = 1 << 2;
const PAD_ON: u8 = 0x09;
const PAD_LAST: u8 = 1 << 1;
const PAD_TAKE: u8 = 0xE0;
/// The last entry.
const CHOICE_LAST: u8 = 6;
/// `DOWN` and `ACROSS` where the sprite stops sliding.
const DOWN_END: u8 = 0x70;

/// A task's fields: its sprite record's index, a second one's
/// (`TASK_SLOT_2`, `w_choice.rs`), the sprite's offsets across and
/// down (from `OFFSETS`) and its animation (a long).
const TASK_SLOT: u32 = 0x03;
const TASK_SLOT_2: u32 = 0x0C;
const TASK_OFF_X: u32 = 0x0D;
const TASK_OFF_Y: u32 = 0x0E;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
/// The flags of the sprites made here.
const SHOWN: u8 = 0x85;

/// In the ROM: the offsets of the entries' tasks (2 bytes an entry,
/// `D_01974E`, by `ROW` x 8), the list loaded with `ROW` below `LIMIT`,
/// an animation.
const OFFSETS: u32 = 0x01974E;
const LIST_196F6: u32 = 0x0196F6;
const ANIM_DE276: u32 = 0x0DE276;

/// Routines: the tasks made, and where this one goes on.
const TASK_1966A: u32 = 0x01966A;
const TASK_18B8C: u32 = 0x018B8C;
const TASK_18C24: u32 = 0x018C24;
const TASK_18C1A: u32 = 0x018C1A;
const SLIDING: u32 = 0x0189DC;
const CHOOSING: u32 = 0x018A30;
const AFTER_LAST: u32 = 0x018AFA;
const AFTER_TAKEN: u32 = 0x018B2C;

impl Game {
    /// `loc_018876`, a task: `func_008624`, `BYTE_80DF` cleared, the
    /// screen loaded (`func_018D1C`), a task `func_01966A` (priority
    /// 0x28), and on in `func_018896`.  Gives back the registers as the
    /// original leaves them.
    pub fn r_018876(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.r_008624();
        self.write8(BYTE_80DF, 0);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5) = self.r_018d1c(
            r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x,
        );
        if self.restarted.is_some() {
            return;
        }
        self.y_start_task(r, 0x28, TASK_1966A);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_018896(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_018896` (the running task's): the Z80's command 4,
    /// `ACROSS` 0x28 and `DOWN` 0, a sprite (depth 0x3C, frame 0, flags
    /// `SHOWN`) as the task's, the counts (`func_018B3C`), the tasks
    /// `func_018B8C` and `func_018C24` (depth 0x32, priority 0x3C), a
    /// second sprite (depth 0x28) as its `TASK_SLOT_2`.  With `ROW`
    /// below `LIMIT`: `CHOICE` 0, `LIST_196F6` loaded and a yield to
    /// `func_0189DC`.  Else, for each of the `COUNT_ADD3` entries whose
    /// bit of `ENTRY_BITS` is clear, a task `func_018C1A` (depth 0x28,
    /// priority 0x3C, `ANIM_DE276`) with its offsets from `OFFSETS`;
    /// `CHOICE` the first entry whose bit is clear (0 if none), and the
    /// task's routine `func_0189DC`.  Gives back every register, X and
    /// C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_018896(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut regs = Regs { d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, ..Regs::default() };
        let r = &mut regs;
        self.yb_entries(r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_018896`'s work on the registers.
    fn yb_entries(&mut self, r: &mut Regs) {
        self.y_sound(r, 4);
        self.write8(ACROSS, 0x28);
        self.write8(DOWN, 0);
        self.yb_own(r);
        self.yb_new_sprite(r, 0x3C);
        self.write8(r.a6 + TASK_SLOT, r.d5 as u8);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.write8(r.a5 + SPRITE_FLAGS, SHOWN);
        r.d1 = self.r_018b3c(r.d1);
        self.y_make_task(r, 0x32, 0x3C, TASK_18B8C);
        self.write8(r.a5 + SPRITE_FLAGS, SHOWN);
        self.y_make_task(r, 0x32, 0x3C, TASK_18C24);
        self.write8(r.a5 + SPRITE_FLAGS, SHOWN);
        self.yb_new_sprite(r, 0x28);
        self.write8(r.a6 + TASK_SLOT_2, r.d5 as u8);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.write8(r.a5 + SPRITE_FLAGS, SHOWN);
        let row = self.read8(ROW);
        r.d0 = low_byte(r.d0, row);
        if row < self.read8(LIMIT) {
            self.write8(CHOICE, 0);
            if self.y_load(r, LIST_196F6) {
                return;
            }
            self.yb_yield(r, SLIDING);
            return;
        }
        r.d1 = low_word(r.d1, 0);
        self.write8(BIT_80C5, 1);
        let offset = row << 3;
        r.d0 = low_byte(r.d0, offset);
        self.write8(OFFSET_80C4, offset);
        loop {
            let saved = r.d1;
            let clear = self.read8(ENTRY_BITS) & self.read8(BIT_80C5);
            r.d0 = low_byte(r.d0, clear);
            if clear == 0 {
                self.y_make_task(r, 0x28, 0x3C, TASK_18C1A);
                self.write32(r.a6 + TASK_ANIM, ANIM_DE276);
                self.write8(r.a5 + SPRITE_FLAGS, SHOWN);
                let at = self.read8(OFFSET_80C4) as i8 as i16 as u16;
                r.d1 = low_word(r.d1, at);
                r.a0 = OFFSETS;
                let entry = OFFSETS.wrapping_add(at as i16 as u32);
                let across = self.read8(entry);
                self.write8(r.a6 + TASK_OFF_X, across);
                let down = self.read8(entry + 1);
                self.write8(r.a6 + TASK_OFF_Y, down);
            }
            let bit = self.read8(BIT_80C5).wrapping_add(self.read8(BIT_80C5));
            r.d0 = low_byte(r.d0, bit);
            self.write8(BIT_80C5, bit);
            let offset = self.read8(OFFSET_80C4).wrapping_add(2);
            self.write8(OFFSET_80C4, offset);
            r.d1 = low_byte(saved, (saved as u8).wrapping_add(1));
            if r.d1 as u8 == self.read8(COUNT_ADD3) {
                break;
            }
        }
        r.d1 = low_word(r.d1, 0);
        r.d2 = low_byte(r.d2, 1);
        loop {
            let set = self.read8(ENTRY_BITS) & r.d2 as u8;
            r.d0 = low_byte(r.d0, set);
            if set == 0 {
                break;
            }
            r.d2 = low_byte(r.d2, (r.d2 as u8).wrapping_add(r.d2 as u8));
            r.d1 = low_byte(r.d1, (r.d1 as u8).wrapping_add(1));
            if r.d1 as u8 == self.read8(COUNT_ADD3) {
                r.d1 = low_word(r.d1, 0);
                break;
            }
        }
        self.write8(CHOICE, r.d1 as u8);
        self.y_next(r, SLIDING);
    }

    /// `func_0189DC`, each frame: the task's sprite frame 0x39, placed
    /// (`func_018A0A`); `func_001BB4` (the task may stop there); then
    /// `DOWN` up by 2 to `DOWN_END` and `ACROSS` down by 2 to 0, the
    /// sprite placed again; when both are there, `func_018B82` and a
    /// yield to `func_018A30`.
    pub fn r_0189dc(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, 0x39);
        (r.d0, r.x, r.c) = self.r_018a0a(r.d0, r.a5);
        if self.yb_stopped(r) {
            return;
        }
        let down = self.read8(DOWN);
        if down != DOWN_END {
            self.write8(DOWN, down.wrapping_add(2));
        } else {
            let across = self.read8(ACROSS);
            if across == 0 {
                self.r_018b82();
                self.yb_yield(r, CHOOSING);
                return;
            }
            self.write8(ACROSS, across.wrapping_sub(2));
        }
        (r.d0, r.x, r.c) = self.r_018a0a(r.d0, r.a5);
    }

    /// `func_018A30`, each frame, by the pad's byte: bit 2 moves
    /// `CHOICE` back (from 0 to `COUNT_ADD2` - 1), bit 0 or 3 on (back
    /// to 0 at `COUNT_ADD2`), each with `BLINK` cleared, the Z80's
    /// command 0x20 and `func_018B82`; bit 1 the command 0x20,
    /// `func_018B84` and `CHOICE_LAST`.  Any of bits 5-7: the command
    /// 0x21 and, for `CHOICE_LAST`, `BYTE_954F` 2, `COUNT_A93C` up and a
    /// wait of 0x30 frames (on in `func_018AFA`); for another entry the
    /// task's routine `func_018B2C` and on to `func_018B0C`.
    pub fn r_018a30(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let pad = self.read8(PAD);
        r.d0 = low_byte(r.d0, pad);
        let mut moved = false;
        if pad & PAD_BACK != 0 {
            self.write8(BLINK, 0);
            let (choice, borrow) = self.read8(CHOICE).overflowing_sub(1);
            self.write8(CHOICE, choice);
            r.x = borrow;
            r.d0 = low_byte(r.d0, choice);
            let count = self.read8(COUNT_ADD2);
            if choice >= count {
                let (last, borrow) = count.overflowing_sub(1);
                r.d0 = low_byte(r.d0, last);
                r.x = borrow;
                self.write8(CHOICE, last);
            }
            moved = true;
        } else {
            r.d0 = low_byte(r.d0, pad & PAD_ON);
            if pad & PAD_ON != 0 {
                self.write8(BLINK, 0);
                let (choice, carry) = self.read8(CHOICE).overflowing_add(1);
                self.write8(CHOICE, choice);
                r.x = carry;
                r.d0 = low_byte(r.d0, choice);
                if choice >= self.read8(COUNT_ADD2) {
                    self.write8(CHOICE, 0);
                }
                moved = true;
            }
        }
        if moved {
            self.y_sound(r, 0x20);
            self.r_018b82();
        }
        let last = self.read8(PAD) & PAD_LAST;
        r.d0 = low_byte(r.d0, last);
        if last != 0 {
            self.y_sound(r, 0x20);
            self.r_018b84();
            self.write8(CHOICE, CHOICE_LAST);
        }
        let take = self.read8(PAD) & PAD_TAKE;
        r.d0 = low_byte(r.d0, take);
        r.c = false;
        if take == 0 {
            return;
        }
        self.y_sound(r, 0x21);
        let choice = self.read8(CHOICE);
        r.d0 = low_byte(r.d0, choice);
        if choice == CHOICE_LAST {
            self.write8(BYTE_954F, 2);
            let count = self.read8(COUNT_A93C);
            self.write8(COUNT_A93C, count.wrapping_add(1));
            self.yb_sleep(r, 0x30, AFTER_LAST);
        } else {
            self.y_next(r, AFTER_TAKEN);
            (r.d0, r.d1, r.a1, r.x, r.c) =
                self.r_018b0c(r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        }
    }
}
