//! Routines of 0x1884A-0x18D80 around the choice of `func_018896`
//! (`CHOICE`, 0-5, or 6): the tasks that start it (`func_01884A`,
//! `func_018860`), the counts of entries (`func_018B3C`), the sprites
//! placed by `ACROSS`/`DOWN` (`func_018A0A`, `func_018BF2`,
//! `func_018B8C`, `func_018C1A`, `func_018C24`), the ends past a choice
//! (`func_018AFA`, `func_018B0C`, `func_018B24`, `func_018B2C`) and the
//! screen's loading (`func_018CB0`, `func_018D1C`, `func_018D22`).

use crate::{Game, Regs};

/// The byte compared with `LIMIT` (`func_018B3C`); its table's row
/// (`func_018B8C`, `func_018CB0`).
const ROW: u32 = 0xFFFF_953A;
/// Compared with `ROW` (`func_018B3C`).
const LIMIT: u32 = 0xFFFF_953B;
/// The entry chosen (6: the last one, `CHOICE_LAST`).
const CHOICE: u32 = 0xFFFF_953C;
/// Bits of entries: shifted out one by one by `func_018B3C`.
const ENTRY_BITS: u32 = 0xFFFF_953D;
/// Two counts `func_018B3C` sets: 4 (3 with `ROW` 0) in `COUNT_ADD3`,
/// and that less one for each clear bit of `ENTRY_BITS` (to 1) in
/// `COUNT_ADD2`.
const COUNT_ADD2: u32 = 0xFFFF_ADD2;
const COUNT_ADD3: u32 = 0xFFFF_ADD3;
/// The sprites' place: across (+ 0x58) and down (- 0x30 or 0x40).
const ACROSS: u32 = 0xFFFF_ADD4;
const DOWN: u32 = 0xFFFF_ADD5;
/// Counted up by `func_018B8C`: with bit 4 set its sprite is hidden.
const BLINK: u32 = 0xFFFF_ADD6;
/// The entry's offset in `ENTRIES` (`func_018B8C` sets it).
const OFFSET_80C5: u32 = 0xFFFF_80C5;
/// Flags of the game (`b_control.rs`): bit 3 set by `func_018B0C`.
const MODE: u32 = 0xFFFF_9540;
const MODE_BIT3: u8 = 1 << 3;
/// Bit 0 cleared by `func_018B2C`, set by `func_018CB0`.
const FLAGS_9548: u32 = 0xFFFF_9548;
/// Two words `func_018D22` sets (0x88, 0xA8).
const WORD_954A: u32 = 0xFFFF_954A;
const WORD_954C: u32 = 0xFFFF_954C;
/// Counted up by `func_018D22`.
const COUNT_A93C: u32 = 0xFFFF_A93C;
/// Cleared by `func_018D22`.
const BYTE_A945: u32 = 0xFFFF_A945;
/// Set to 1 by `func_018D22`.
const BYTE_954F: u32 = 0xFFFF_954F;
/// The list `func_0094D8` loads (a long).
const LOAD_LIST: u32 = 0xFFFF_92FA;
/// A list made in RAM by `func_018CB0` (9 bytes: `LIST_ADDA` from
/// `BYTES_19706` by `ROW`, 2, a word 0, `LIST_ADDE` from `BYTES_1970C`
/// by `ROW`, 0, a word 0x7000, 0xFF).
const LIST_ADDA: u32 = 0xFFFF_ADDA;
const LIST_ADDB: u32 = 0xFFFF_ADDB;
const LIST_ADDC: u32 = 0xFFFF_ADDC;
const LIST_ADDE: u32 = 0xFFFF_ADDE;
const LIST_ADDF: u32 = 0xFFFF_ADDF;
const LIST_ADE0: u32 = 0xFFFF_ADE0;
const LIST_ADE2: u32 = 0xFFFF_ADE2;
/// A long `func_018CB0` keeps across its second load.
const LONG_5810: u32 = 0x00FF_5810;
/// The index of the task running (`task.rs`).
const CURRENT: u32 = 0xFFFF_80EE;
/// The running task's sprite record (index and address).
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// The last entry: no sprite (`func_018B8C`), frame 0x3D
/// (`func_018C24`).
const CHOICE_LAST: u8 = 6;

/// A task's fields: a second sprite record's index (`func_018C24`),
/// the sprite's offsets across and down (`func_018BF2`).
const TASK_SLOT_2: u32 = 0x0C;
const TASK_OFF_X: u32 = 0x0D;
const TASK_OFF_Y: u32 = 0x0E;

/// A sprite record's fields (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;

/// In the ROM: the entries by `ROW` x 4 + `CHOICE` (3 bytes: the frame
/// and the offsets, `D_019712`), bytes by `ROW` (`D_019706`,
/// `D_01970C`) and the lists `func_0094D8` loads.
const ENTRIES: u32 = 0x019712;
const BYTES_19706: u32 = 0x019706;
const BYTES_1970C: u32 = 0x01970C;
const LIST_196EE: u32 = 0x0196EE;
const LIST_196FE: u32 = 0x0196FE;
/// Sprite sets (`func_000776`).
const SET_D9386: u32 = 0x0D9386;
const SET_DC3D8: u32 = 0x0DC3D8;

/// Routines (code addresses).
const TASK_18D80: u32 = 0x018D80;
const TASK_191BA: u32 = 0x0191BA;
const AFTER_18B24: u32 = 0x018B24;

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
    /// `func_01884A`: the tasks ended (`func_0003DA`) and one running
    /// `TASK_18D80` at priority 0x32.  Gives back d0, d1, d6, a0, a6,
    /// X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01884a(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let (d0, d1, d6, a0, _) = self.r_0003da(d2, d3, d4, d5, d7, a1, a2, a3, a4, a5);
        let (d6, a6, x, c) = self.r_0003e8(d4, d5, d6, 0x32, TASK_18D80);
        (d0, d1, d6, a0, a6, x, c)
    }

    /// `func_018860`: the tasks ended (`func_0003DA`) and one running
    /// `TASK_191BA` at priority 0x32.  Gives back d0, d1, d6, a0, a6,
    /// X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_018860(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        let (d0, d1, d6, a0, _) = self.r_0003da(d2, d3, d4, d5, d7, a1, a2, a3, a4, a5);
        let (d6, a6, x, c) = self.r_0003e8(d4, d5, d6, 0x32, TASK_191BA);
        (d0, d1, d6, a0, a6, x, c)
    }

    /// `func_018A0A`: the sprite at a5 at `ACROSS` + 0x58 (its low byte
    /// only) and `DOWN` - 0x30 (the word).  Gives back d0, X (the
    /// add's carry) and C (clear).
    pub fn r_018a0a(&mut self, d0: u32, a5: u32) -> (u32, bool, bool) {
        let down = u16::from(self.read8(DOWN)).wrapping_sub(0x30);
        let d0 = low_word(d0, down);
        self.write16(a5 + SPRITE_Y, down);
        let (across, carry) = self.read8(ACROSS).overflowing_add(0x58);
        let d0 = low_byte(d0, across);
        self.write8(a5 + SPRITE_X_LOW, across);
        (d0, carry, false)
    }

    /// `func_018AFA` (past `func_018A30`'s wait): `func_01793A`'s wait
    /// (which may return for the task), then `AFTER_18B24` the task's
    /// routine and on to `func_018B0C`.  Gives back the registers as
    /// they are left.
    pub fn r_018afa(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        let waiting;
        (r.d6, r.a6, r.x, r.c, waiting) = self.r_01793a(r.d6);
        if waiting {
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, AFTER_18B24);
        (r.d0, r.d1, r.a1, r.x, r.c) = self.r_018b0c(r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_018B0C`: `MODE_BIT3` set, the Z80 command 0xFE
    /// (`func_00349E`), then `func_00A51A` (4; d1 0x20).  Gives back d0,
    /// d1, a1, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_018b0c(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, bool, bool) {
        let mode = self.read8(MODE);
        self.write8(MODE, mode | MODE_BIT3);
        let d1 = 0x20;
        self.r_00349e(0xFE);
        let (d0, a1, x, c) = self.r_00a51a(4, d1, d2, d3, d4, d5, d6, d7, a2, a3, a4, a5);
        (d0, d1, a1, x, c)
    }

    /// `func_018B24`: `func_001BB4` (the task may stop there), then
    /// `loc_01797A`.  Gives back the registers as they are left.
    pub fn r_018b24(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) = self.r_01797a(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_018B2C`: `func_001BB4` (the task may stop there), then
    /// `FLAGS_9548`'s bit 0 cleared and on to `loc_0098F8`.  Gives back
    /// the registers as they are left.
    pub fn r_018b2c(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
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

    /// `func_018B3C`: `COUNT_ADD3` 4 (3 when `ROW` is 0), `COUNT_ADD2`
    /// the same; and when `LIMIT` is `ROW`, the bits of `ENTRY_BITS`
    /// (its low 4, or 3 with `ROW` 0) from the top: `COUNT_ADD2` one
    /// less for each clear one after the first, till a set one or
    /// `COUNT_ADD2` is 1.  Gives back d1 (`LIMIT` in its low byte).
    pub fn r_018b3c(&mut self, d1: u32) -> u32 {
        let mut bits = self.read8(ENTRY_BITS) << 4;
        self.write8(COUNT_ADD3, 4);
        if self.read8(ROW) == 0 {
            self.write8(COUNT_ADD3, 3);
            bits <<= 1;
        }
        let count = self.read8(COUNT_ADD3);
        self.write8(COUNT_ADD2, count);
        let limit = self.read8(LIMIT);
        let d1 = low_byte(d1, limit);
        if limit != self.read8(ROW) {
            return d1;
        }
        // add.b d0,d0: the bit shifted out is the carry
        let mut shift = || {
            let out = bits & 0x80 != 0;
            bits <<= 1;
            out
        };
        if shift() {
            return d1;
        }
        loop {
            if shift() {
                return d1;
            }
            let count = self.read8(COUNT_ADD2).wrapping_sub(1);
            self.write8(COUNT_ADD2, count);
            if count == 1 {
                return d1;
            }
        }
    }

    /// `func_018B82`: an `rts`.
    pub fn r_018b82(&mut self) {}

    /// `func_018B84`: `func_00A55E`.
    pub fn r_018b84(&mut self) {
        self.r_00a55e();
    }

    /// `func_018B8C`, a task: with `CHOICE` `CHOICE_LAST`, its sprite's
    /// frame 0; else the frame and the offsets of `ENTRIES` by `ROW` x
    /// 4 + `CHOICE` (the offset kept in `OFFSET_80C5`), the frame 0
    /// while bit 4 of `BLINK` (counted up) is set, and the sprite
    /// placed (`func_018BF2`).  Gives back d0, d1, d5, d6, a0, a5, a6,
    /// X and C.
    pub fn r_018b8c(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        let choice = self.read8(CHOICE);
        if choice == CHOICE_LAST {
            self.write8(r.a5 + SPRITE_FRAME, 0);
            r.c = false;
            return;
        }
        let index = self.read8(ROW).wrapping_mul(4).wrapping_add(choice);
        // mulu: the offset fills d0
        r.d0 = u32::from(index) * 3;
        self.write8(OFFSET_80C5, r.d0 as u8);
        r.a0 = ENTRIES;
        let frame = self.read8(word_at(ENTRIES, r.d0));
        self.write8(r.a5 + SPRITE_FRAME, frame);
        let blink = self.read8(BLINK).wrapping_add(1);
        self.write8(BLINK, blink);
        r.d1 = low_byte(r.d1, blink & 0x10);
        if blink & 0x10 != 0 {
            self.write8(r.a5 + SPRITE_FRAME, 0);
        }
        r.d6 = low_byte(r.d6, self.read8(CURRENT));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        let off_x = self.read8(word_at(ENTRIES + 1, r.d0));
        self.write8(r.a6 + TASK_OFF_X, off_x);
        let off_y = self.read8(word_at(ENTRIES + 2, r.d0));
        self.write8(r.a6 + TASK_OFF_Y, off_y);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.d1, r.x, r.c) = self.r_018bf2(r.d0, r.d1, r.a5, r.a6);
    }

    /// `func_018BF2`: the sprite at a5 at `ACROSS` + 0x58 + the task's
    /// `TASK_OFF_X` (the low byte only) and `DOWN` - 0x40 + its
    /// `TASK_OFF_Y` (the word).  Gives back d0, d1, X (the last add's
    /// carry) and C (clear).
    pub fn r_018bf2(&mut self, d0: u32, d1: u32, a5: u32, a6: u32) -> (u32, u32, bool, bool) {
        let across = self.read8(ACROSS).wrapping_add(0x58).wrapping_add(self.read8(a6 + TASK_OFF_X));
        self.write8(a5 + SPRITE_X_LOW, across);
        let down = u16::from(self.read8(DOWN)).wrapping_sub(0x40);
        let off = u16::from(self.read8(a6 + TASK_OFF_Y));
        let d1 = low_word(d1, off);
        let (down, carry) = down.overflowing_add(off);
        let d0 = low_word(d0, down);
        self.write16(a5 + SPRITE_Y, down);
        (d0, d1, carry, false)
    }

    /// `func_018C1A`, a task: its sprite placed (`func_018BF2`), its
    /// animation a frame on (`func_0013D6`).  Gives back d0, d1, d2,
    /// d5, a0, a5, X and C.
    pub fn r_018c1a(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        (r.d0, r.d1, r.x, r.c) = self.r_018bf2(r.d0, r.d1, r.a5, r.a6);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `func_018C24`, a task: its sprite and its second one
    /// (`TASK_SLOT_2`) at `ACROSS` x 3 + 0xC0 across and `DOWN` - 0x30
    /// (a byte) down; with `CHOICE` `CHOICE_LAST` the second's frame 0
    /// and its own 0x3D, else its own 0x3C and the second's 0x3E +
    /// `CHOICE`.  Gives back d0, d5, a5, X and C.
    pub fn r_018c24(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        let task = r.a6;
        // mulu: the product fills d0
        r.d0 = u32::from(self.read8(ACROSS)) * 3;
        r.d0 = low_word(r.d0, (r.d0 as u16).wrapping_add(0xC0));
        self.write16(r.a5 + SPRITE_X, r.d0 as u16);
        let slot2 = self.read8(task + TASK_SLOT_2);
        r.d5 = low_byte(r.d5, slot2);
        (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
        self.write16(r.a5 + SPRITE_X, r.d0 as u16);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        let down = self.read8(DOWN).wrapping_sub(0x30);
        r.d0 = (r.d0 & 0xFFFF_0000) | u32::from(down);
        self.write16(r.a5 + SPRITE_Y, r.d0 as u16);
        r.d5 = low_byte(r.d5, slot2);
        (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
        self.write16(r.a5 + SPRITE_Y, r.d0 as u16);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        let choice = self.read8(CHOICE);
        if choice == CHOICE_LAST {
            r.d5 = low_byte(r.d5, slot2);
            (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
            self.write8(r.a5 + SPRITE_FRAME, 0);
            r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
            r.a5 = self.read32(SLOT_ADDR);
            self.write8(r.a5 + SPRITE_FRAME, 0x3D);
        } else {
            self.write8(r.a5 + SPRITE_FRAME, 0x3C);
            r.d5 = low_byte(r.d5, slot2);
            (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
            let (frame, carry) = 0x3Eu8.overflowing_add(choice);
            r.d0 = low_byte(r.d0, frame);
            r.x = carry;
            self.write8(r.a5 + SPRITE_FRAME, frame);
        }
        r.c = false;
    }

    /// `func_018CB0`: `FLAGS_9548`'s bit 0 set, the list at `LIST_ADDA`
    /// made by `ROW`; `LIST_196EE` loaded, then that one
    /// (`func_0094D8`), `LONG_5810` kept across the second.  Gives back
    /// d0-d7 and a0-a5 as they are left.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_018cb0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let flags = self.read8(FLAGS_9548);
        self.write8(FLAGS_9548, flags | 1);
        let row = self.read8(ROW);
        let d1 = low_word(d1, row as i8 as u16);
        let first = self.read8(word_at(BYTES_19706, d1));
        self.write8(LIST_ADDA, first);
        self.write8(LIST_ADDB, 2);
        self.write16(LIST_ADDC, 0);
        let a0 = BYTES_1970C;
        let second = self.read8(word_at(BYTES_1970C, d1));
        self.write8(LIST_ADDE, second);
        self.write8(LIST_ADDF, 0);
        self.write16(LIST_ADE0, 0x7000);
        self.write8(LIST_ADE2, 0xFF);
        self.write32(LOAD_LIST, LIST_196EE);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, _) = self.r_0094d8(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        }
        let kept = self.read32(LONG_5810);
        self.write32(LOAD_LIST, LIST_ADDA);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, _, _, _) = self.r_0094d8(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        }
        self.write32(LONG_5810, kept);
        (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5)
    }

    /// `func_018D1C`: `func_018CB0`, then `func_018D22`.  Gives back
    /// d0-d7 and a0-a5 as they are left.  Differs in the check as its
    /// translation does (`func_018D22`).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_018d1c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5) = self.r_018cb0(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, x);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        }
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, _) = self.r_018d22(d0, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5)
    }

    /// `func_018D22`: the scroll tables cleared (`func_008624`), the
    /// sprites (`func_000704`), `func_001C56`; `WORD_954A` 0x88,
    /// `WORD_954C` 0xA8, `MODE` 0xC0; `func_00BE96`; `COUNT_A93C`
    /// counted up, `LIST_196FE` loaded (`func_0094D8`); `BYTE_A945`
    /// cleared, `BYTE_954F` 1, `func_001CA0`; sprite sets 4 and 5
    /// (`func_000776`).  Gives back d0-d7 and a0-a6 as they are left.
    /// Differs in the check as its translation does: 0xFF805B, which
    /// the vertical blank's handler changes during the call.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_018d22(&mut self, d0: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        self.r_008624();
        let (_, d1, _, _) = self.r_000704(d0);
        let (d6, _, _) = self.r_001c56(d4, d5, d6);
        self.write16(WORD_954A, 0x88);
        self.write16(WORD_954C, 0xA8);
        self.write8(MODE, 0xC0);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6) = self.r_00be96(d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6);
        }
        let (count, carry) = self.read8(COUNT_A93C).overflowing_add(1);
        self.write8(COUNT_A93C, count);
        self.write32(LOAD_LIST, LIST_196FE);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, _, _) = self.r_0094d8(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, carry);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6);
        }
        self.write8(BYTE_A945, 0);
        self.write8(BYTE_954F, 1);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, _, _) = self.r_001ca0(d0, d1, d2, d3, d5, d6, d7, a0, a1, a2, a3, a5, a6);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6);
        }
        self.r_000776(4, SET_D9386, a1);
        let d0 = self.r_000776(5, SET_DC3D8, a1);
        (d0, d1, d2, d3, d4, d5, d6, d7, SET_DC3D8, a1, a2, a3, a4, a5, a6)
    }
}
