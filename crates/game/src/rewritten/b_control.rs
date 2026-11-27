//! What an object is asked to do this frame (`REQUEST`, from its pad
//! bytes) and how fast it goes (`SPEED_NEG`, `SPEED_POS`): `func_00211C`
//! and the changes it makes by an object's `EFFECT` (0x22DC-0x23B9).
//!
//! No script reaches the `MODE` with bit 0, so the effects (and
//! `func_00211C`'s part for them) are checked by reading only.

use crate::{Game, Regs};

/// The request: bits 0-3 the ways (`b_walk.rs`), bit 4 and bit 5 more;
/// `REQUEST_2` a second byte set with it.
const REQUEST: u32 = 0xFFFF_A932;
const REQUEST_2: u32 = 0xFFFF_A933;
/// Two bytes set with it: `ARG_A934` from the object's byte 0x2E,
/// `ARG_A935` 0x98 (`func_0023AC` and `func_0023B2` change it).
const ARG_A934: u32 = 0xFFFF_A934;
const ARG_A935: u32 = 0xFFFF_A935;
/// The speeds (words), for a way taking away and one adding.
const SPEED_NEG: u32 = 0xFFFF_9EEA;
const SPEED_POS: u32 = 0xFFFF_9EEC;
/// The speeds by the object's byte 0x10 (bits 0-1), words in the ROM
/// (`D_0021A6`, `D_0021AE`).
const SPEEDS_POS: u32 = 0x0021A6;
const SPEEDS_NEG: u32 = 0x0021AE;
/// The changes by `EFFECT` (1 on), longs in the ROM (`D_0022B8`).
const EFFECTS: u32 = 0x0022B8;
/// By way (bits 0-1 of the object's facing): the request's low nibble
/// when it asks for no way (`func_002354`; bytes in the ROM).
const FACING_REQUEST: u32 = 0x009C68;

/// Flags of the game: bit 0 the speeds come from the effects, bit 6
/// every object's request is `ALL_REQUEST`.
const MODE: u32 = 0xFFFF_9540;
const ALL_REQUEST: u32 = 0xFFFF_954F;
/// When this is 0x39, the speeds start at 0x400 (not 0x100).
const STAGE: u32 = 0xFFFF_953F;
/// 8 bytes an object (by index): bytes 2 and 3 its requests.
const PADS: u32 = 0xFFFF_AD0A;
/// Bytes by object index: the effect it has (0 none), and two counts.
const EFFECT: u32 = 0xFFFF_9E92;
const EFFECT_COUNT: u32 = 0xFFFF_9E8D;
const EFFECT_LEFT: u32 = 0xFFFF_9E97;
/// For an index of 5 on, the index whose effects it takes (bytes, from
/// index 5).
const OWNERS: u32 = 0xFFFF_A968;
/// For an index of 5 on, a byte: 1 lets `func_002354` keep a request
/// with bit 4.
const KINDS: u32 = 0xFFFF_A96E;
/// While this isn't 0, the speeds are 0x600 and the request isn't
/// swapped (`func_002316`).
const FAST: u32 = 0xFFFF_A940;
/// Counts the frames `func_00211C` calls `func_0087F6`.
const DRIVEN: u32 = 0xFFFF_A943;
/// The object being run (index and address), and the one asked
/// (`b_walk.rs`).
const CURRENT: u32 = 0xFFFF_A42C;
const CURRENT_ADDR: u32 = 0xFFFF_A42E;
const WALKER: u32 = 0xFFFF_A93F;
/// The running object's sprite record (index and address).
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// An object's fields.
/// Bits 0-1: which speeds (`SPEEDS_POS`, `SPEEDS_NEG`).
const OBJ_SPEED: u32 = 0x10;
const OBJ_FACING: u32 = 0x14;
/// Bit 4: driven by `func_0087F6`, its pad not read.
const OBJ_FLAGS: u32 = 0x2D;
const OBJ_2E: u32 = 0x2E;
/// A byte `func_00211C` clears and counts up after `func_00190A`.
const OBJ_00: u32 = 0x00;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// A register's low byte sign-extended into its low word.
fn ext(reg: u32) -> u32 {
    low_word(reg, reg as u8 as i8 as u16)
}

/// `table` + the low word of `index` (signed).
fn at(table: u32, index: u32) -> u32 {
    table.wrapping_add(index as u16 as i16 as u32)
}

impl Game {
    /// `func_00211C`: `WALKER`'s request and speeds for the frame.
    /// The request from its `PADS` bytes (or `ALL_REQUEST` in that
    /// mode; or none read, `func_0087F6` driving it, when its
    /// `OBJ_FLAGS` has bit 4).  The speeds by its `OBJ_SPEED`, or in
    /// the `MODE` with bit 0, 0x100 (0x400 on stage 0x39) changed by
    /// its `EFFECT`'s routine; an effect with counts left, or kept
    /// (`EFFECT_LEFT`), goes to the object `func_00190A` finds (C set)
    /// if that has none (sound 0x2B).  `FAST` makes them 0x600.  d4 and
    /// a4 are `CURRENT`'s after.  Gives back d0, d1, d4, d5, d7, a0, a4,
    /// a5 and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00211c(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d5: u32, d7: u32, a0: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let (mut d0, mut d1, mut d5, mut d7, mut a0, mut a5) = (d0, d1, d5, d7, a0, a5);
        let index = self.read8(WALKER);
        let (mut d4, a4, mut x) = self.r_001cfa(low_byte(d4, index));
        let arg = self.read8(a4 + OBJ_2E);
        self.write8(ARG_A934, arg);
        self.write8(ARG_A935, 0x98);
        if self.read8(a4 + OBJ_FLAGS) & (1 << 4) != 0 {
            let n = self.read8(DRIVEN);
            self.write8(DRIVEN, n.wrapping_add(1));
            (d4, d7, x) = self.r_0087f6(d0, d1, d2, d4, d7);
        } else if self.read8(MODE) & (1 << 6) != 0 {
            let request = self.read8(ALL_REQUEST);
            d0 = low_byte(d0, request);
            self.write8(REQUEST, request);
            self.write8(REQUEST_2, request);
        } else {
            d4 = ext(d4);
            let by8 = (d4 as u16) << 3;
            x = (d4 as u16) & 0x2000 != 0;
            d0 = low_word(d0, by8);
            a0 = PADS;
            let request = self.read8(at(PADS + 2, d0));
            self.write8(REQUEST, request);
            let request = self.read8(at(PADS + 3, d0));
            self.write8(REQUEST_2, request);
        }
        if self.read8(MODE) & 1 == 0 {
            let speeds = self.read8(a4 + OBJ_SPEED) & 3;
            d0 = low_word(d0, u16::from(speeds) * 2);
            x = false;
            let pos = self.read16(at(SPEEDS_POS, d0));
            self.write16(SPEED_POS, pos);
            let neg = self.read16(at(SPEEDS_NEG, d0));
            self.write16(SPEED_NEG, neg);
            d5 = low_byte(d5, self.read8(SLOT_INDEX));
            a5 = self.read32(SLOT_ADDR);
        } else {
            let (neg, pos) = if self.read8(STAGE) == 0x39 { (0xFC00, 0x400) } else { (0xFF00, 0x100) };
            self.write16(SPEED_NEG, neg);
            self.write16(SPEED_POS, pos);
            d4 = ext(d4);
            a0 = EFFECT;
            let effect = self.read8(at(EFFECT, d4));
            d0 = low_byte(d0, effect);
            if effect != 0 {
                (d0, d1, d4, a0, x) = self.effect(d0, d1, d4, a4, x);
            }
        }
        if self.read8(FAST) != 0 {
            self.write16(SPEED_NEG, 0xFA00);
            self.write16(SPEED_POS, 0x600);
        }
        let d4 = low_byte(d4, self.read8(CURRENT));
        let a4 = self.read32(CURRENT_ADDR);
        (d0, d1, d4, d5, d7, a0, a4, a5, x)
    }

    /// `func_00211C`'s effect `d0` (its low byte, 1 on) run (a routine
    /// of `EFFECTS`); then, with counts left, handed on.  Gives back
    /// d0, d1, d4, a0, X.
    fn effect(&mut self, d0: u32, d1: u32, d4: u32, a4: u32, x: bool) -> (u32, u32, u32, u32, bool) {
        let d0 = ext(d0);
        let d0 = low_word(d0, (d0 as u16).wrapping_mul(4));
        let routine = self.read32(at(EFFECTS - 4, d0));
        let mut regs = Regs { d0, d1, d4, a0: routine, a4, x, ..Regs::default() };
        self.call_family_25(routine, &mut regs);
        let (mut d0, d1) = (regs.d0, regs.d1);
        let index = self.read8(WALKER);
        let (d4, _, mut x) = self.r_001cfa(low_byte(regs.d4, index));
        let mut d4 = ext(d4);
        let mut a0 = if self.read8(at(EFFECT_LEFT, d4)) != 0 {
            EFFECT_LEFT
        } else if self.read8(at(EFFECT_COUNT, d4)) >= 2 {
            EFFECT_COUNT
        } else {
            return (d0, d1, d4, EFFECT_COUNT, x);
        };
        d4 = low_byte(d4, self.read8(CURRENT));
        let current = self.read32(CURRENT_ADDR);
        self.write8(current + OBJ_00, 0);
        let c;
        (d0, x, c) = self.r_00190a(current, x);
        let n = self.read8(current + OBJ_00);
        self.write8(current + OBJ_00, n.wrapping_add(1));
        if !c {
            return (d0, d1, d4, a0, x);
        }
        if (d0 as u8) >= 5 {
            d0 = ext(d0);
            d0 = low_byte(d0, self.read8(at(OWNERS - 5, d0)));
        }
        d0 = ext(d0);
        a0 = EFFECT;
        if self.read8(at(EFFECT, d0)) == 0 {
            self.r_0034b6(low_byte(d0, 0x2B));
            let index = self.read8(WALKER);
            (d4, _, x) = self.r_001cfa(low_byte(d4, index));
            for table in [EFFECT, EFFECT_COUNT, EFFECT_LEFT] {
                let b = self.read8(at(table, d4));
                self.write8(at(table, d0), b);
                a0 = table;
            }
        }
        (d0, d1, d4, a0, x)
    }

    /// `func_0022DC`: speeds 0x80.
    pub fn r_0022dc(&mut self, _regs: &mut Regs) {
        self.write16(SPEED_NEG, 0xFF80);
        self.write16(SPEED_POS, 0x80);
    }

    /// `func_0022EA`: speeds 0x400.
    pub fn r_0022ea(&mut self, _regs: &mut Regs) {
        self.write16(SPEED_NEG, 0xFC00);
        self.write16(SPEED_POS, 0x400);
    }

    /// `func_0022F8`: bit 5 off in both requests.
    pub fn r_0022f8(&mut self, _regs: &mut Regs) {
        let r = self.read8(REQUEST);
        self.write8(REQUEST, r & !(1 << 5));
        let r = self.read8(REQUEST_2);
        self.write8(REQUEST_2, r & !(1 << 5));
    }

    /// `func_002306`: bit 5 on in `REQUEST`.
    pub fn r_002306(&mut self, _regs: &mut Regs) {
        let r = self.read8(REQUEST);
        self.write8(REQUEST, r | (1 << 5));
    }

    /// `func_00230E`: `ARG_A934` 1.
    pub fn r_00230e(&mut self, _regs: &mut Regs) {
        self.write8(ARG_A934, 1);
    }

    /// `func_002316`: the ways asked for swapped (0 with 1, 2 with 3),
    /// unless `FAST` or `WALKER` is driven (`OBJ_FLAGS` bit 4).  Gives
    /// back d0, d1, d4.
    pub fn r_002316(&mut self, regs: &mut Regs) {
        if self.read8(FAST) != 0 {
            return;
        }
        let index = self.read8(WALKER);
        let (d4, walker, _) = self.r_001cfa(low_byte(regs.d4, index));
        regs.d4 = d4;
        if self.read8(walker + OBJ_FLAGS) & (1 << 4) != 0 {
            return;
        }
        let request = self.read8(REQUEST);
        let d1 = low_byte(regs.d1, request & 5);
        regs.d1 = low_word(d1, (d1 as u16).wrapping_add(d1 as u16));
        let swapped = (((request & 0xA) >> 1) | regs.d1 as u8) & 0xF;
        regs.d0 = low_byte(regs.d0, swapped);
        let r = self.read8(REQUEST) & 0xF0;
        self.write8(REQUEST, r | swapped);
    }

    /// `func_002354`: no way asked for, the object asks for the way it
    /// faces (`FACING_REQUEST`), unless it is one of index 5 on whose
    /// `KINDS` byte is 1 and the request has bit 4.  Gives back d0, d1,
    /// d4.
    pub fn r_002354(&mut self, regs: &mut Regs) {
        let ways = self.read8(REQUEST) & 0xF;
        regs.d0 = low_byte(regs.d0, ways);
        if ways != 0 {
            return;
        }
        let index = self.read8(CURRENT);
        regs.d4 = low_byte(regs.d4, index);
        let obj = self.read32(CURRENT_ADDR);
        if index >= 5 {
            regs.d4 = ext(regs.d4);
            if self.read8(at(KINDS - 5, regs.d4)) == 1 && self.read8(REQUEST) & (1 << 4) != 0 {
                return;
            }
        }
        let facing = self.read8(obj + OBJ_FACING) & 3;
        regs.d0 = low_word(regs.d0, u16::from(facing));
        let high = self.read8(REQUEST) & 0xF0;
        let request = high | self.read8(FACING_REQUEST + u32::from(facing));
        regs.d1 = low_byte(regs.d1, request);
        self.write8(REQUEST, request);
    }

    /// `func_0023AC`: `ARG_A935` 0.
    pub fn r_0023ac(&mut self, _regs: &mut Regs) {
        self.write8(ARG_A935, 0);
    }

    /// `func_0023B2`: `ARG_A935` 0x40.
    pub fn r_0023b2(&mut self, _regs: &mut Regs) {
        self.write8(ARG_A935, 0x40);
    }
}
