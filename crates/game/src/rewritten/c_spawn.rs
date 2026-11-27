//! The objects a stage starts with: read from a list of 16 entries
//! (3 bytes: a kind, then the cell, column and row) picked by the byte
//! at `LIST_INDEX`; each kind an entry of `KINDS`.

use crate::Game;

/// What a stage's list is picked by (x 0x30 from `LISTS`).
const LIST_INDEX: u32 = 0xFF953F;
/// Bits 0 and 7: while either is set, `func_00355E` makes nothing.
const MODE: u32 = 0xFF9540;
/// The list (a pointer: `LISTS` + 0x30 x `LIST_INDEX`) and the offset
/// in it of the entry being read (to 0x30).
const LIST: u32 = 0xFF80C4;
const LIST_AT: u32 = 0xFFA95C;
/// The objects made (counts the index from object 6 they go to,
/// `func_001CFA`).
const MADE: u32 = 0xFFA958;
/// Counts the objects of kind 0, and of kinds with bit 3 of their
/// flags (`func_0036D6`); `COUNTED` gets it once the list is read.
const COUNT: u32 = 0xFFA959;
const COUNTED: u32 = 0xFFA95A;
/// Cleared here.
const CLEARED_1: u32 = 0xFFA956;
const CLEARED_2: u32 = 0xFFA957;
/// 0 or 1: the X `func_00BFE4` leaves.  While it's 1, the entries of
/// kind 0 and of kinds with bit 3 of their flags are left out.
const LEAVE_OUT: u32 = 0xFFA95E;

/// What an object is set up with (`anim.rs`): its routine (`OBJ_2`),
/// its animations; its position (pixels: words) and cell.
const OBJ_INIT_2: u32 = 0xFF80D0;
const OBJ_INIT_ANIMS: u32 = 0xFF80C8;
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
const CELL: u32 = 0xFF9F08;
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;

/// An object's fields (`anim.rs`, `c_chase.rs`).
/// Its kind + 1.
const OBJ_KIND_1: u32 = 0x00;
const OBJ_TIMER: u32 = 0x06;
const OBJ_HOLD: u32 = 0x07;
const OBJ_0F: u32 = 0x0F;
const OBJ_FLAGS: u32 = 0x10;
/// From bits 4-5 of the kind's flags.
const OBJ_11: u32 = 0x11;
const OBJ_ANIM_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x26;
const OBJ_TIMER_ADD: u32 = 0x27;
/// Its cell (column, row) when made.
const OBJ_HOME: u32 = 0x28;
/// A sprite record's flags (`sprite.rs`).
const SPRITE_FLAGS: u32 = 0x05;

/// The ROM's tables.
/// The lists (D_00F4E8): 0x30 bytes each.
const LISTS: u32 = 0x00F4E8;
/// The kinds (D_003704): 10 bytes each: a routine, the animations
/// (longs), flags (bits 0-2: `OBJ_TIMER_ADD`; bit 3: see `LEAVE_OUT`
/// and `func_0036D6`; bits 4-5: `OBJ_11`; bits 6-7: bits 5-6 of
/// `OBJ_FLAGS`) and a byte for `OBJ_KIND`.
const KINDS: u32 = 0x003704;
const KIND_SIZE: u32 = 10;
const KIND_FLAGS: u32 = 8;
const KIND_BYTE: u32 = 9;
/// The routine of the object `func_0036D6` makes (func_00706C).
const ROUTINE_706C: u32 = 0x00706C;
/// `func_00389E`'s tables: a byte by kind (D_0038EE), and what
/// `func_00AC64` gets (D_003918).
const BY_KIND: u32 = 0x0038EE;
const AMOUNTS: u32 = 0x003918;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_00355E`: the stage's objects, one by entry of its list (an
    /// entry with bit 7 of its kind set is none; `func_003608`), unless
    /// bit 0 or 7 of `MODE` is set.  Gives back d5 and a5.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00355e(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32) {
        self.write8(CLEARED_2, 0);
        self.write8(COUNT, 0);
        self.write8(COUNTED, 0);
        if self.read8(MODE) & 0x81 != 0 {
            return (d5, a5);
        }
        self.write16(LEAVE_OUT, 0);
        let (x, _) = self.r_00bfe4();
        self.write16(LEAVE_OUT, u16::from(x));
        let list = u32::from(self.read8(LIST_INDEX)) * 0x30 + LISTS;
        self.write32(LIST, list);
        self.write8(LIST_AT, 0);
        self.write8(MADE, 0);
        self.write8(CLEARED_1, 0);
        let (mut d0, mut d1, mut d4, mut d5, mut a4, mut a5) = (list, d1, d4, d5, a4, a5);
        loop {
            let at = self.read8(LIST_AT);
            let list = self.read32(LIST);
            let kind = self.read8(list + u32::from(at));
            d0 = low_byte(d0, kind);
            if kind & 0x80 != 0 {
                self.write8(LIST_AT, at.wrapping_add(3));
            } else {
                let column = self.read8(list + u32::from(at) + 1);
                self.write8(CELL_X, column);
                self.write16(POSITION_X, (column as i8 as i16 as u16) << 4);
                let row = self.read8(list + u32::from(at) + 2);
                self.write8(CELL_Y, row);
                self.write16(POSITION_Y, (row as i8 as i16 as u16) << 4);
                d1 = low_word(d1, u16::from(at + 3));
                self.write8(LIST_AT, at + 3);
                (d0, d1, d4, d5, a4, a5) = self.r_003608(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
            }
            if self.read8(LIST_AT) == 0x30 {
                break;
            }
        }
        let count = self.read8(COUNT);
        self.write8(COUNTED, count);
        (d5, a5)
    }

    /// `func_003608`: an object of kind `d0` (its low byte) at `CELL`
    /// and `POSITION`: the next from object 6 (`MADE`, `func_001CFA`),
    /// set up from its entry of `KINDS`; a kind with bit 3 of its flags
    /// makes another object first (`func_0036D6`).  While `LEAVE_OUT`
    /// is set, kind 0 and those kinds make nothing.  Gives back d0 (its
    /// low byte + 1, as the object's byte 0), d1 (its low byte bits
    /// 4-5 of the flags), d4, d5, a4 and a5 (the object and its sprite
    /// record).
    #[allow(clippy::too_many_arguments)]
    pub fn r_003608(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32) {
        let leave_out = self.read16(LEAVE_OUT) != 0;
        if d0 as u8 == 0 {
            if leave_out {
                return (d0, d1, d4, d5, a4, a5);
            }
            let count = self.read8(COUNT);
            self.write8(COUNT, count.wrapping_add(1));
        }
        let kind = (d0 & 0xFF) * KIND_SIZE + KINDS;
        let (mut d4, mut d5, mut a5) = (d4, d5, a5);
        if self.read8(kind + KIND_FLAGS) & 8 != 0 {
            if leave_out {
                return (d0, d1, d4, d5, a4, a5);
            }
            (d4, d5, a5) = self.r_0036d6(low_byte(kind, 8), d1, d2, d3, d4, d5, d6, d7, kind, a1, a2, a3, a5);
        }
        let routine = self.read32(kind);
        self.write32(OBJ_INIT_2, routine);
        let anims = self.read32(kind + 4);
        self.write32(OBJ_INIT_ANIMS, anims);
        let flags = self.read8(kind + KIND_FLAGS);
        let byte = self.read8(kind + KIND_BYTE);
        let made = self.read8(MADE);
        self.write8(MADE, made.wrapping_add(1));
        let (d4, obj, x) = self.r_001cfa(low_byte(d4, made.wrapping_add(6)));
        let (_, d5, a5, _) = self.r_0012a0(low_byte(kind, byte), low_byte(d1, flags), d2, d3, d4, d5, d6, d7, kind + KIND_SIZE, a1, a2, a3, obj, a5, x);
        self.write8(a5 + SPRITE_FLAGS, 1);
        self.write8(obj + OBJ_0F, 0);
        self.r_001852(obj);
        self.write8(obj + OBJ_ANIM_DIRECTION, 2);
        let cell = self.read16(CELL);
        self.write16(obj + OBJ_HOME, cell);
        self.write8(obj + OBJ_KIND, byte);
        self.write8(obj + OBJ_TIMER_ADD, flags & 7);
        self.write8(obj + OBJ_TIMER, (flags & 7) | 0x40);
        let d1 = low_byte(d1, (flags >> 4) & 3);
        self.write8(obj + OBJ_11, (flags >> 4) & 3);
        self.write8(obj + OBJ_FLAGS, (flags >> 1) & 0x60);
        self.write8(obj + OBJ_HOLD, 0);
        let d0 = low_byte(d0, (d0 as u8).wrapping_add(1));
        self.write8(obj + OBJ_KIND_1, d0 as u8);
        (d0, d1, d4, d5, obj, a5)
    }

    /// `func_0036D6`: an object with the routine `ROUTINE_706C` (and the
    /// animations `OBJ_INIT_ANIMS` has), the next from object 6, its
    /// sprite record's flags 4; counted in `COUNT`.  Gives back d4, d5
    /// and a5.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0036d6(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32) -> (u32, u32, u32) {
        let count = self.read8(COUNT);
        self.write8(COUNT, count.wrapping_add(1));
        self.write32(OBJ_INIT_2, ROUTINE_706C);
        let made = self.read8(MADE);
        self.write8(MADE, made.wrapping_add(1));
        let (d4, obj, x) = self.r_001cfa(low_byte(d4, made.wrapping_add(6)));
        let (_, d5, a5, _) = self.r_0012a0(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, obj, a5, x);
        self.write8(a5 + SPRITE_FLAGS, 4);
        self.write8(obj + OBJ_0F, 0);
        self.r_001852(obj);
        (d4, d5, a5)
    }

    /// `func_00389E`: for the object at a4 (by its byte 0 - 1,
    /// `BY_KIND`: bits 0-2 a step, bit 3 another row), the step +
    /// `CLEARED_1` - 1 (7 at most) picks an amount from `AMOUNTS` for
    /// `func_00AC64` (which adds it to the word at 0xFF9544).  Gives back d0: its low byte what was picked + 1
    /// (14 at most).
    pub fn r_00389e(&mut self, d0: u32, obj: u32) -> u32 {
        let index = self.read8(obj).wrapping_sub(1) as i8 as i32 as u32;
        let entry = self.read8(BY_KIND.wrapping_add(index));
        let row = entry & 8;
        let step = (entry & 7).wrapping_add(self.read8(CLEARED_1)).wrapping_sub(1);
        let d0 = if step >= 7 { 7 } else { low_byte(d0, step) };
        let picked = (d0 as u8) | row;
        let d0 = low_word(d0, picked as i8 as i16 as u16);
        let amount = self.read8(AMOUNTS.wrapping_add(picked as i8 as i32 as u32));
        let (d0, _) = self.r_00ac64(low_byte(d0, amount));
        low_byte(d0, picked.min(0xD) + 1)
    }
}
