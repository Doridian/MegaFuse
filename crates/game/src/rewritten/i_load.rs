//! A list of things to load (`func_0094D8`): pairs of an entry of a
//! table (packed data and a length) and a command that does something
//! with the data, decoded a byte at a time (`decode.rs`): patterns to
//! VRAM, colours, a map to plane A.

use crate::{Game, Regs};

/// The list's address (a long).  Its bytes: 0xFF ends it, 0xFE then
/// an index picks the table (`TABLES`), else an index into the table
/// then the command's index (`COMMANDS`), each command reading what
/// else it takes from the list.
const LIST: u32 = 0xFFFF_92FA;
const END: u8 = 0xFF;
const PICK_TABLE: u8 = 0xFE;
/// The tables (longs in the ROM, `D_009554`).
const TABLES: u32 = 0x009554;
/// The commands (longs in the ROM, `D_00958A`).
const COMMANDS: u32 = 0x00958A;
/// The table in use (a long).  Its entries: 6 bytes, the data's
/// address (a long; bit 31 set: run-packed, else LZ) and its length (a
/// word).
const TABLE: u32 = 0xFFFF_9302;
const ENTRY_SIZE: u32 = 6;
/// The entry picked: the data's address (`decode.rs`'s `SOURCE`), its
/// top byte with the run-packed bit, and its length (a word).
const SOURCE: u32 = 0xFFFF_930A;
const RUNS: u8 = 0x80;
const LENGTH: u32 = 0xFFFF_9308;
/// The length's low byte: bit 7 set, `func_009692` reads two bytes
/// more of the list.
const LENGTH_LOW: u32 = 0xFFFF_9309;
/// The routine that gives the next byte of the data (a long), and the
/// two it is set to: the LZ decoder's and the run decoder's.
const NEXT_BYTE: u32 = 0xFFFF_92FE;
const LZ_BYTE: u32 = 0x000CF4;
const RUN_BYTE: u32 = 0x000D8E;
/// A byte: its bit 0 x 0x1000 is added to `func_0095BA`'s pattern, and
/// to `func_009648`'s first colour.
const BANK: u32 = 0xFFFF_9306;

/// Where the patterns are decoded to, and the copy to VRAM made of
/// them (`i_queue.rs`'s entry: source, VRAM address, length).
const PATTERNS: u32 = 0x00FF_6000;
const COPY_SOURCE: u32 = 0xFFFF_A94E;
const COPY_TARGET: u32 = 0xFFFF_A952;
const COPY_LENGTH: u32 = 0xFFFF_A954;
/// The first pattern's number in a command is this more than its
/// index.
const PATTERN_BIAS: u16 = 0x1000;

/// The colour `func_0093FC` starts at (`i_palette.rs`).
const FIRST: u32 = 0xFFFF_80D0;
const FIRST_LOW: u32 = 0xFFFF_80D1;
/// The map's place (a word, plane A from 0xE000) for `func_0100AE`.
const PLACE: u32 = 0xFFFF_80C8;
const PLANE_A: u16 = 0xE000;
/// A word added to `func_009658`'s place: its bits 6-15 to the row,
/// its low byte to the column (`FOLLOW_LOW`).
const FOLLOW: u32 = 0xFFFF_9ECE;
const FOLLOW_LOW: u32 = 0xFFFF_9ECF;
/// Where `func_009622` and `func_00962A` keep the entry's address.
const KEPT_1: u32 = 0xFFFF_9E6C;
const KEPT_2: u32 = 0xFFFF_9E70;

/// The backdrop's colour while the list is loaded, then the default.
const BACKDROP_BUSY: u16 = 0x000E;
const BACKDROP_DEFAULT: u16 = 0xFFFF;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_0094D8`: the list at `LIST` loaded: for each entry its
    /// data's decoder started (LZ, or runs for one with `RUNS` set,
    /// that bit cleared) and its command run, with a4 at the list.  d0
    /// to d4 and a0 to a4 are kept.  The backdrop is `BACKDROP_BUSY`
    /// meanwhile.  Gives back every register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0094d8(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.r_000296(BACKDROP_BUSY);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        r.a4 = self.read32(LIST);
        loop {
            let op = self.read8(r.a4);
            r.a4 += 1;
            r.d0 = low_byte(r.d0, op);
            if op == END {
                let c = self.r_000296(BACKDROP_DEFAULT);
                return (d0, d1, d2, d3, d4, r.d5, r.d6, r.d7, a0, a1, a2, a3, a4, r.a5, r.a6, r.x, c);
            }
            if op == PICK_TABLE {
                let index = self.i_list_index(&mut r);
                let table = self.read32(TABLES + index);
                self.write32(TABLE, table);
                continue;
            }
            self.r_0096ac(r.d0);
            self.write32(NEXT_BYTE, LZ_BYTE);
            self.r_000cba();
            let top = self.read8(SOURCE);
            self.write8(SOURCE, top & !RUNS);
            if top & RUNS != 0 {
                self.r_000d7a();
                self.write32(NEXT_BYTE, RUN_BYTE);
            }
            let index = self.i_list_index(&mut r);
            let command = self.read32(COMMANDS + index);
            r.a0 = command;
            (r.n, r.z, r.v, r.c) = (false, index == 0, false, false);
            r.unwound = false;
            self.call_family_12(command, &mut r);
            if r.unwound || self.restarted.is_some() {
                return (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, false);
            }
        }
    }

    /// The list's next byte x 4 (a table's offset), into d0's low word;
    /// X clear (the add's carry).
    fn i_list_index(&mut self, r: &mut Regs) -> u32 {
        let index = u16::from(self.read8(r.a4)) * 4;
        r.a4 += 1;
        r.d0 = low_word(r.d0, index);
        r.x = false;
        u32::from(index)
    }

    /// `func_00956C`: a word of the data (two bytes, `NEXT_BYTE`'s
    /// routine twice, the first the high byte).  d2 is kept.  Gives
    /// back the registers the decoder leaves, d0's low word the word.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00956c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, x, c, ..Regs::default() };
        self.i_load_word(&mut r);
        (r.d0, r.d1, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x)
    }

    /// `func_00956C` on `r` (d2 changed: the caller keeps it).
    fn i_load_word(&mut self, r: &mut Regs) {
        r.a0 = self.read32(NEXT_BYTE);
        let routine = r.a0;
        r.unwound = false;
        self.call_family_16(routine, r);
        if r.unwound || self.restarted.is_some() {
            return;
        }
        // asl.w #8 of d2 with the byte in its low byte
        let high = (r.d2 as u16 & 0xFF00) | u16::from(r.d0 as u8);
        r.c = high & 0x100 != 0;
        r.x = r.c;
        r.d2 = low_word(r.d2, high << 8);
        r.a0 = self.read32(NEXT_BYTE);
        let routine = r.a0;
        self.call_family_16(routine, r);
        if r.unwound || self.restarted.is_some() {
            return;
        }
        r.d2 = low_byte(r.d2, r.d0 as u8);
        r.d0 = low_word(r.d0, r.d2 as u16);
    }

    /// `func_0095BA`, a command: patterns (the list's next word, plus
    /// `BANK`'s bit 0 x 0x1000, is the first one's number) loaded
    /// (`func_0095D0`).
    pub fn r_0095ba(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let first = self.read16(r.a4);
        r.a4 += 2;
        let bank = (u16::from(self.read8(BANK)) << 12) & 0x1000;
        r.d1 = low_word(r.d1, bank);
        let (first, x) = first.overflowing_add(bank);
        r.d0 = low_word(r.d0, first);
        r.x = x;
        self.i_patterns(r);
    }

    /// `func_0095CE` (and `loc_009632`, `func_0096AA`), a command:
    /// patterns (the list's next word the first one's number) loaded
    /// (`func_0095D0`).
    pub fn r_0095ce(&mut self, regs: &mut Regs) {
        let first = self.read16(regs.a4);
        regs.a4 += 2;
        regs.d0 = low_word(regs.d0, first);
        self.i_patterns(regs);
    }

    /// `func_0095CE`'s and `func_0095BA`'s end: `func_0095D0` on `r`.
    fn i_patterns(&mut self, r: &mut Regs) {
        let (d0, d1, d3, d5, d6, d7, a1, a2, a3, a5, x) =
            self.r_0095d0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d0, r.d1, r.d3, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5, r.x) = (d0, d1, d3, d5, d6, d7, a1, a2, a3, a5, x);
        r.unwound = false;
    }

    /// `func_0095D0`: `LENGTH` words of the data (`func_00956C`) to
    /// `PATTERNS`, then copied to VRAM from pattern `d0` less
    /// `PATTERN_BIAS` (`COPY_SOURCE`, `COPY_TARGET`, `COPY_LENGTH` for
    /// `func_00B9D0`).  d2, d4, a4 and a6 are kept.  Gives back the
    /// registers the decoder leaves, and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0095d0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let target = (d0 as u16).wrapping_sub(PATTERN_BIAS).wrapping_mul(2);
        self.write16(COPY_TARGET, target);
        self.write32(COPY_SOURCE, PATTERNS);
        let length = self.read16(LENGTH);
        self.write16(COPY_LENGTH, length);
        // the count (dbra: 0 is 0x10000) in d4, which the decoder is
        // given and gives back
        let mut r = Regs { d0: low_word(d0, target), d1, d2, d3, d4: low_word(d4, length.wrapping_sub(1)), d5, d6, d7, a1, a2, a3, a4, a5, a6: PATTERNS, ..Regs::default() };
        // subq.w #1 on the count: its borrow
        r.x = length == 0;
        r.c = r.x;
        loop {
            self.i_load_word(&mut r);
            if self.restarted.is_some() {
                break;
            }
            self.write16(r.a6, r.d0 as u16);
            r.a6 += 2;
            r.c = false;
            let left = (r.d4 as u16).wrapping_sub(1);
            r.d4 = low_word(r.d4, left);
            if left == 0xFFFF {
                self.r_00b9d0();
                break;
            }
        }
        (r.d0, r.d1, r.d3, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5, r.x)
    }

    /// `func_00960A`, a command: colours (`func_0093FC`, `LENGTH` of
    /// them) from the list's next byte on (`FIRST_LOW`); the byte after
    /// is read into d0.
    pub fn r_00960a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let first = self.read8(r.a4);
        self.write8(FIRST_LOW, first);
        r.d0 = low_byte(r.d0, self.read8(r.a4 + 1));
        r.a4 += 2;
        self.write8(FIRST, 0);
        r.d1 = low_word(r.d1, self.read16(LENGTH));
        r.x = self.r_0093fc(r.d1, r.d2);
        r.unwound = false;
    }

    /// `func_009620` (and `func_009634`), a command: nothing (no
    /// script's list has it).
    pub fn r_009620(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    pub fn r_009634(&mut self) {}

    /// `func_009622`, a command: the entry's address kept in `KEPT_1`.
    pub fn r_009622(&mut self, regs: &mut Regs) {
        let source = self.read32(SOURCE);
        self.write32(KEPT_1, source);
        regs.unwound = false;
    }

    /// `func_00962A`, a command: the entry's address kept in `KEPT_2`.
    pub fn r_00962a(&mut self, regs: &mut Regs) {
        let source = self.read32(SOURCE);
        self.write32(KEPT_2, source);
        regs.unwound = false;
    }

    /// `loc_009632`, a command: `func_0095CE`.
    pub fn r_009632(&mut self, regs: &mut Regs) {
        self.r_0095ce(regs);
    }

    /// `func_0096AA`, a command: `func_0095CE` (no script's list has
    /// it).
    pub fn r_0096aa(&mut self, regs: &mut Regs) {
        self.r_0095ce(regs);
    }

    /// `func_009636`, a command: a map to plane A (`func_009674`) at
    /// the list's next word (after `func_009692`).
    pub fn r_009636(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.d0, r.a4) = self.r_009692(r.d0, r.a4);
        let place = self.read16(r.a4);
        r.a4 += 2;
        r.d0 = low_word(r.d0, place);
        (r.d0, r.d1, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5, r.a6, r.x) =
            self.r_009674(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        r.unwound = false;
    }

    /// `func_009640`, a command: a map to plane A at the list's next
    /// word moved by `FOLLOW` (`func_009658`, after `func_009692`).  No
    /// script's list has it.
    pub fn r_009640(&mut self, regs: &mut Regs) {
        (regs.d0, regs.a4) = self.r_009692(regs.d0, regs.a4);
        self.i_followed(regs);
    }

    /// `func_009648`, a command: as `func_009640`, its first colour
    /// moved on by `BANK`'s bit 0.
    pub fn r_009648(&mut self, regs: &mut Regs) {
        (regs.d0, regs.a4) = self.r_009692(regs.d0, regs.a4);
        let bank = self.read8(BANK) & 1;
        regs.d0 = low_byte(regs.d0, bank);
        let first = self.read8(FIRST_LOW);
        self.write8(FIRST_LOW, first.wrapping_add(bank));
        self.i_followed(regs);
    }

    /// `func_009658` on `r`.
    fn i_followed(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_009658(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        r.unwound = false;
    }

    /// `func_009658`: the list's next word (a place: row x 0x40 +
    /// column) moved by `FOLLOW` (the column within the row's 0x40),
    /// then `func_009674`.  Gives back the registers it leaves, a4
    /// past the word.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_009658(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let place = self.read16(a4);
        let column = (place as u8).wrapping_add(self.read8(FOLLOW_LOW));
        let column = u16::from(column) & 0x3F;
        let d1 = low_word(d1, column);
        let row = self.read16(FOLLOW) & 0xFFC0;
        let d2 = low_word(d2, row);
        let a4 = a4 + 2;
        let place = (place & 0xFFC0).wrapping_add(row) | column;
        let (d0, d1, d3, d4, d5, d6, d7, a1, a2, a3, a5, a6, x) =
            self.r_009674(low_word(d0, place), d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6);
        (d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, x)
    }

    /// `func_009674`: a map (`LENGTH` cells of the data) to plane A at
    /// place `d0` (`PLACE`, `func_0100AE`).  a4 is kept.  Gives back
    /// what `func_0100AE` does.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_009674(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let place = (d0 as u16).wrapping_mul(2).wrapping_add(PLANE_A);
        self.write16(PLACE, place);
        let d1 = low_word(d1, self.read16(LENGTH));
        self.r_0100ae(low_word(d0, place), d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6)
    }

    /// `func_009692`: when `LENGTH_LOW` has bit 7 set, the list's next
    /// byte less 1, with bit 7 set, to `FIRST_LOW`, and the byte after
    /// into d0.  Gives back d0 and a4.
    pub fn r_009692(&mut self, d0: u32, a4: u32) -> (u32, u32) {
        if self.read8(LENGTH_LOW) & 0x80 == 0 {
            return (d0, a4);
        }
        let first = self.read8(a4).wrapping_sub(1) | 0x80;
        self.write8(FIRST_LOW, first);
        (low_byte(d0, self.read8(a4 + 1)), a4 + 2)
    }

    /// `func_0096AC`: entry `d0` (its low byte) of the table in use
    /// (`func_0096B0`).
    pub fn r_0096ac(&mut self, d0: u32) {
        let table = self.read32(TABLE);
        self.r_0096b0(d0, table);
    }

    /// `func_0096B0`: entry `d0` (its low byte) of the table at `a0`
    /// picked: `SOURCE` and `LENGTH`.
    pub fn r_0096b0(&mut self, d0: u32, a0: u32) {
        let entry = a0.wrapping_add((d0 & 0xFF) * ENTRY_SIZE);
        let source = self.read32(entry);
        self.write32(SOURCE, source);
        let length = self.read16(entry + 4);
        self.write16(LENGTH, length);
    }
}
