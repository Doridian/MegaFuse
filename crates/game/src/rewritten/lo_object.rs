//! Objects 0-3 at their start (`func_001D6E`): their fields, their
//! sprite records, their colours (`func_001EA2` and the like), and the
//! routine they get (`func_001F7E`) with its steps (`func_001FD4` on).
//! Also `func_001D0C`, a look-up in the objects' table.

use crate::{Game, Regs};

/// The objects' addresses by index (in the ROM, `D_001D1E`), and their
/// size: object n + 5 is `COMPANION` bytes on.
const OBJECT_TABLE: u32 = 0x001D1E;
const COMPANION: u32 = 5 * 0x40;

/// An object's fields: its sprite record's address (a long) and index;
/// two bytes set from `START_10`, `START_11`; a byte set to its index
/// (`OBJ_0F`); its direction for the animation (`anim.rs`) and the kind
/// of its animation; the animation's frames to go and position.
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_0F: u32 = 0x0F;
const OBJ_10: u32 = 0x10;
const OBJ_11: u32 = 0x11;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x15;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
/// `func_001E30`'s argument (d1's low byte): it picks `OBJ_INIT_AT`
/// from `INIT_TABLE` and the sprite's frame byte (`FRAMES`).
const OBJ_2C: u32 = 0x2C;
/// Flags: `func_001D6E` sets bit 7 and clears bit 5.
const OBJ_FLAGS: u32 = 0x2D;
/// Bytes `func_001D6E` sets from `START_2E`, `START_30` and clears.
const OBJ_2E: u32 = 0x2E;
const OBJ_2F: u32 = 0x2F;
const OBJ_30: u32 = 0x30;
const OBJ_31: u32 = 0x31;
const OBJ_32: u32 = 0x32;
const OBJ_35: u32 = 0x35;
const OBJ_36: u32 = 0x36;
const OBJ_37: u32 = 0x37;
const OBJ_38: u32 = 0x38;
const OBJ_39: u32 = 0x39;

/// A sprite record's fields (`sprite.rs`): its palette byte, its frame
/// byte, a word (`func_001E30`: 0x260 + 0x10 x the object's index) and
/// its frames' table (a long).
const SPR_PALETTE: u32 = 0x03;
const SPR_4: u32 = 0x04;
const SPR_FRAME: u32 = 0x05;
const SPR_6: u32 = 0x06;
const SPR_TABLE: u32 = 0x0C;
/// What `func_001E30` puts in `SPR_TABLE` (`D_0093B4`).
const SPR_TABLE_VALUE: u32 = 0x0093B4;
/// The frame byte by `OBJ_2C` / 2 (in the ROM, `D_001E98`).
const FRAMES: u32 = 0x001E98;

/// Bit 0: three bytes by object (from `BY_OBJECT_92`, `BY_OBJECT_8D`,
/// `BY_OBJECT_97`) and `OBJ_35`, `OBJ_39` are cleared at its start; bit
/// 2 with bit 0 or 4: `func_001F7E` takes the other way (0x327A).
const MODE: u32 = 0xFFFF_9540;
const BY_OBJECT_92: u32 = 0xFFFF_9E92;
const BY_OBJECT_8D: u32 = 0xFFFF_9E8D;
const BY_OBJECT_97: u32 = 0xFFFF_9E97;
/// A byte by object set to its index at its start.
const BY_OBJECT_68: u32 = 0xFFFF_A968;
/// Bytes cleared at the start.
const CLEARED_A93C: u32 = 0xFFFF_A93C;
const CLEARED_A941: u32 = 0xFFFF_A941;
/// What the start copies to the object.
const START_2E: u32 = 0xFFFF_A937;
const START_30: u32 = 0xFFFF_A938;
const START_10: u32 = 0xFFFF_A939;
const START_11: u32 = 0xFFFF_A93A;
/// A long set to the object's routine (`ROUTINE`) at the start, and the
/// routine.
const OBJ_INIT_2: u32 = 0xFFFF_80D0;
const ROUTINE: u32 = 0x001F7E;
/// A long `func_001E30` sets from `INIT_TABLE` (longs in the ROM,
/// `D_00B534`; indexed by `OBJ_2C` x 2).
const OBJ_INIT_AT: u32 = 0xFFFF_80C8;
const INIT_TABLE: u32 = 0x00B534;

/// The object `func_001F7E` runs (its index, a byte).
const CHOSEN: u32 = 0xFFFF_A93F;
/// When not 0 (and the object's animation at frame 1 of a pair without
/// bit 1), `func_001F7E` sends the Z80 a command: `SOUND_A936` if
/// `BYTE_A936` isn't 0, else `SOUND`.
const COUNT_A93C: u32 = 0xFFFF_A93C;
const BYTE_A936: u32 = 0xFFFF_A936;
const SOUND: u8 = 0x2F;
const SOUND_A936: u8 = 0x35;
/// The animation kind the other way gives the object.
const OTHER_KIND: u8 = 0x0E;
/// `func_001FD6` sets `FAST` from d0 and clears `PUSH` (a long) and
/// `NO_SLIDE` (`n_push.rs`).
const FAST: u32 = 0xFFFF_A940;
const PUSH: u32 = 0xFFFF_9EF2;
const NO_SLIDE: u32 = 0xFFFF_A93E;
/// The sprite record of the routine being run (`task.rs`).
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// The palette in RAM (0x20 bytes a line) and the colours `func_001EA2`
/// and the like copy into it (in the ROM): by object (`OBJECT_COLOURS`,
/// 0x10 bytes each), and three sets for any object.
const PALETTE: u32 = 0x00FF_5800;
const OBJECT_COLOURS: u32 = 0x001F0E;
const COLOURS_1F4E: u32 = 0x001F4E;
const COLOURS_1F5E: u32 = 0x001F5E;
const COLOURS_1F6E: u32 = 0x001F6E;
/// How many colours are copied (less 1, `func_00946E`'s count).
const COUNT: u32 = 7;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// `ext.w` on `reg`.
fn ext_w(reg: u32) -> u32 {
    low_word(reg, reg as u8 as i8 as u16)
}

impl Game {
    /// `func_001D0C`: a5 = object `d5`'s address (its low byte);
    /// d5 is kept.
    pub fn r_001d0c(&mut self, d5: u32) -> (u32, u32) {
        let a5 = self.read32(OBJECT_TABLE + 4 * (d5 & 0xFF));
        (d5, a5)
    }

    /// `func_001D6E`: object `d4` (`func_001CFA`) started: its fields
    /// set and cleared, `OBJ_INIT_2` its routine (`func_001F7E`), set up
    /// (`func_001E30`), a second sprite record (depth 0x80,
    /// `func_000796`) kept in the fields of object d4 + 5, frame 0x22;
    /// then `func_00BCD4`.  d0-d5 and a0-a5 are kept.  Gives back d6, a6
    /// and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_001d6e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, bool) {
        let (mut d4, a4, _) = self.r_001cfa(d4);
        if self.read8(MODE) & 1 != 0 {
            d4 = ext_w(d4);
            let index = d4 as u16 as i16 as u32;
            for table in [BY_OBJECT_92, BY_OBJECT_8D, BY_OBJECT_97] {
                self.write8(table.wrapping_add(index), 0);
            }
            self.write8(a4 + OBJ_39, 0);
            self.write8(a4 + OBJ_35, 0);
        }
        for field in [OBJ_2F, OBJ_31] {
            self.write8(a4 + field, 0);
        }
        self.write8(CLEARED_A93C, 0);
        self.write8(CLEARED_A941, 0);
        for field in [OBJ_36, OBJ_37, OBJ_38] {
            self.write8(a4 + field, 0);
        }
        let byte = self.read8(START_30);
        self.write8(a4 + OBJ_30, byte);
        let byte = self.read8(START_2E);
        self.write8(a4 + OBJ_2E, byte);
        let flags = (self.read8(a4 + OBJ_FLAGS) | 0x80) & !0x20;
        let d0 = low_byte(d0, flags);
        self.write8(a4 + OBJ_FLAGS, flags);
        self.write32(OBJ_INIT_2, ROUTINE);
        let (d0, d1, d2, d5, a0, a5, x) = self.r_001e30(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        let byte = self.read8(START_10);
        self.write8(a4 + OBJ_10, byte);
        let byte = self.read8(START_11);
        self.write8(a4 + OBJ_11, byte);
        let d0 = low_byte(d0, 0x80);
        let (d5, a5, x, _) = self.r_000796(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        let d0 = low_word(d0, COMPANION as u16);
        self.write8(a4 + COMPANION + OBJ_SPRITE, d5 as u8);
        self.write32(a4 + COMPANION + OBJ_SPRITE_ADDR, a5);
        self.write8(a5 + SPR_FRAME, 0x22);
        self.write8(a5 + SPR_4, 0);
        let d4 = ext_w(d4);
        self.write8(BY_OBJECT_68.wrapping_add(d4 as u16 as i16 as u32), d4 as u8);
        self.write8(a4 + OBJ_32, 0xFF);
        self.r_00bcd4(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, x)
    }

    /// `func_001E30`: the object at a4 set up with `d1` (its low byte,
    /// to `OBJ_2C`; `OBJ_INIT_AT` from `INIT_TABLE` by it): a sprite
    /// record (`func_0012A0`), direction 2 and its animation
    /// (`func_00130E`), its frame (`func_001E86`) and colours
    /// (`func_001EA2`), the record's palette byte and `OBJ_0F` the
    /// object's index (d4), `SPR_6` and `SPR_TABLE`; a frame of its
    /// animation (`func_001336`) and its sprite placed (`func_001B32`).
    /// Gives back d0, d1, d2, d5, a0, a5 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_001e30(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, bool) {
        let which = (d1 as u16) & 0xFF;
        self.write8(a4 + OBJ_2C, which as u8);
        let (by_two, x) = which.overflowing_add(which);
        let d1 = low_word(d1, by_two);
        let init = self.read32(INIT_TABLE.wrapping_add(by_two as i16 as u32));
        self.write32(OBJ_INIT_AT, init);
        let (d0, d5, a5, _) = self.r_0012a0(d0, d1, d2, d3, d4, d5, d6, d7, INIT_TABLE, a1, a2, a3, a4, a5, x);
        self.write8(a4 + OBJ_DIRECTION, 2);
        self.r_00130e(d0, a4);
        self.r_001e86(a4, a5);
        let (d0, d1, _, _) = self.r_001ea2(d0, d4);
        let index = d4 as u8;
        self.write8(a5 + SPR_PALETTE, index);
        self.write8(a4 + OBJ_0F, index);
        let (word, x) = (u16::from(index) << 4).overflowing_add(0x260);
        let d0 = low_word(d0, word);
        self.write16(a5 + SPR_6, word);
        self.write32(a5 + SPR_TABLE, SPR_TABLE_VALUE);
        let (_, d1, d2, d5, a0, a5, _, _, _) = self.r_001336(d0, d1, d2, d5, a4, x);
        let (d0, d1, x, _) = self.r_001b32(d1, a4, a5);
        (d0, d1, d2, d5, a0, a5, x)
    }

    /// `func_001E86`: the sprite record's frame byte (at a5) from
    /// `FRAMES` by the object's `OBJ_2C` / 2 (sign-extended first).
    /// Gives back X (the bit shifted out).
    pub fn r_001e86(&mut self, a4: u32, a5: u32) -> bool {
        let which = self.read8(a4 + OBJ_2C) as i8 as u16;
        let frame = self.read8(FRAMES + u32::from(which >> 1));
        self.write8(a5 + SPR_FRAME, frame);
        which & 1 != 0
    }

    /// `func_001EA2`: object `d4`'s colours (`OBJECT_COLOURS`, its
    /// index's line, from the second) to palette line d4 + 1 (from its
    /// second colour, `func_00946E`).  Gives back d0, d1, a0 and X.
    pub fn r_001ea2(&mut self, d0: u32, d4: u32) -> (u32, u32, u32, bool) {
        let by_line = (d4 as u8 as i8 as u16).wrapping_shl(4);
        let a0 = OBJECT_COLOURS.wrapping_add(by_line as i16 as u32).wrapping_add(2);
        let d0 = low_word(d0, by_line.wrapping_add(by_line).wrapping_add(2));
        let x = self.r_00946e(d0, COUNT, a0);
        (d0, COUNT, a0, x)
    }

    /// `func_001EC2`: `func_001EC8` with `COLOURS_1F5E`.
    pub fn r_001ec2(&mut self, d0: u32, d4: u32) -> (u32, u32, u32, bool) {
        self.r_001ec8(d0, d4, COLOURS_1F5E)
    }

    /// `func_001EC8` (also `func_001EC2`'s end): the colours at `a0`
    /// (from the second) to palette line `d4` + 1 (from its second
    /// colour, `func_00946E`), the palette marked changed
    /// (`func_0094B2`).  Gives back d0, d1, a0 and X.
    pub fn r_001ec8(&mut self, d0: u32, d4: u32, a0: u32) -> (u32, u32, u32, bool) {
        let a0 = a0.wrapping_add(2);
        let d0 = low_word(d0, (d4 as u8 as i8 as u16).wrapping_shl(5).wrapping_add(2));
        let x = self.r_00946e(d0, COUNT, a0);
        self.r_0094b2();
        (d0, COUNT, a0, x)
    }

    /// `func_001EE0`: colours 2 and 6 of `COLOURS_1F4E` to palette line
    /// `d4` (in RAM), marked changed (`func_0094B2`).  Gives back d0, d1,
    /// a0, a1 and X (the last bit `asl` shifted out).
    pub fn r_001ee0(&mut self, d0: u32, d4: u32) -> (u32, u32, u32, u32, bool) {
        let index = d4 as u8 as i8 as u16;
        let by_line = index.wrapping_shl(5);
        let x = index & (1 << 11) != 0;
        let d0 = low_word(d0, by_line);
        let a1 = PALETTE.wrapping_add(by_line as i16 as u32);
        for offset in [4, 0xC] {
            let colour = self.read16(COLOURS_1F4E + offset);
            self.write16(a1 + offset, colour);
        }
        self.r_0094b2();
        (d0, COUNT, COLOURS_1F4E, a1, x)
    }

    /// `func_001F06`: `func_001EC8` with `COLOURS_1F6E`.  Gives back d1.
    pub fn r_001f06(&mut self, d0: u32, d4: u32) -> u32 {
        self.r_001ec8(d0, d4, COLOURS_1F6E).1
    }

    /// `func_001F7E` (the routine `func_001D6E` gives objects 0-3): with
    /// `MODE` bit 2 and bit 0 or 4 (the code at 0x327A), the object,
    /// `CHOSEN`, gets animation kind `OTHER_KIND` (direction 0, from the
    /// start, `func_00130E`) unless it has it, and its effects
    /// (`func_002F70`), a frame of its animation (`func_001336`) and its
    /// sprite placed (`func_001B32`).  Else it is `CHOSEN`, a sound when
    /// `COUNT_A93C` isn't 0 and its animation is at frame 1 of a pair
    /// without bit 1 (`SOUND` or `SOUND_A936`), and `func_001FD4`.
    /// Gives back every register, X and C.
    pub fn r_001f7e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let mode = self.read8(MODE);
        if mode & 4 != 0 {
            r.d0 = low_byte(r.d0, mode & 0x11);
        }
        if mode & 4 != 0 && mode & 0x11 != 0 {
            self.write8(CHOSEN, r.d4 as u8);
            if self.read8(r.a4 + OBJ_KIND) != OTHER_KIND {
                self.write8(r.a5 + SPR_PALETTE, r.d4 as u8);
                self.write8(r.a4 + OBJ_DIRECTION, 0);
                self.write8(r.a4 + OBJ_KIND, OTHER_KIND);
                (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
                self.write8(r.a4 + OBJ_TIMER, 0);
                self.write8(r.a4 + OBJ_STEP, 0);
            }
            (r.d0, r.d1, r.a0, r.a1, r.x) = self.r_002f70(r.d0, r.d1, r.d4, r.a0, r.a1);
            (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
            (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
            return;
        }
        self.write8(CHOSEN, r.d4 as u8);
        if self.read8(COUNT_A93C) != 0
            && self.read8(r.a4 + OBJ_TIMER) == 1
            && self.read8(r.a4 + OBJ_STEP) & 2 == 0
        {
            let sound = if self.read8(BYTE_A936) != 0 { SOUND_A936 } else { SOUND };
            r.d0 = low_byte(r.d0, sound);
            self.r_0034b6(r.d0);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_001fd4(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `func_001FD4`: `func_001FD6` with d0 0.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_001fd4(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.r_001fd6(0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6)
    }

    /// `func_001FD6`: `FAST` from `d0` (its low byte), `PUSH` and
    /// `NO_SLIDE` cleared, the routine for the object's cell
    /// (`func_0053A6`); then `func_001FE6`.  Gives back every register,
    /// X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_001fd6(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.write8(FAST, d0 as u8);
        self.write32(PUSH, 0);
        self.write8(NO_SLIDE, 0);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6) =
            self.r_0053a6(d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, false, false);
        }
        let (d0, d1, d2, d4, d5, d6, d7, a0, a1, a4, a5, a6, x, c) =
            self.r_001fe6(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6);
        (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c)
    }

    /// `func_001FE6`: the object at a4 a frame on: its cell
    /// (`func_0015E6`), its effects and requests (`func_00211C`), what is
    /// placed (`func_005776`), its walk (`func_0023BA`); unless its
    /// direction byte is negative, its animation (`func_00130E`,
    /// `func_001336`); its sprite placed and its depth (`func_001B32`,
    /// `func_001B5C`), its cell again (`func_0015E6`, `func_00167A`),
    /// the cell's records (`func_006948`), its effects' colours
    /// (`func_002F70`) and `func_003032`.  Gives back d0, d1, d2, d4,
    /// d5, d6, d7, a0, a1, a4, a5, a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_001fe6(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (d0, _) = self.r_0015e6(a4);
        let (d0, d1, d4, d5, d7, a0, _, a5, _) = self.r_00211c(d0, d1, d2, d4, d5, d7, a0, a5);
        let (d0, d4, d5, d6, a0, a4, a6) = self.r_005776(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5, a6);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let (d0, d1, d2, d4, d6, a0, a4, a6) = self.r_0023ba(d0, d1, d2, d4, d5, d6, a0, a4, a6);
        let (mut d1, mut d2, mut d5, mut a0) = (d1, d2, d5, a0);
        if self.read8(a4 + OBJ_DIRECTION) & 0x80 == 0 {
            let x;
            (_, x, _) = self.r_00130e(d0, a4);
            (_, d1, d2, d5, a0, _, _, _, _) = self.r_001336(d0, d1, d2, d5, a4, x);
        }
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5);
        let (d0, _) = self.r_0015e6(a4);
        self.r_00167a();
        let (d0, d1, d2, d4, d5, d6, a0, a4, a5, a6) = self.r_006948(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6);
        let (d0, d1, a0, a1, x) = self.r_002f70(d0, d1, d4, a0, a1);
        let (d0, d1, d4, d5, a0, a4, a5, x, c) = self.r_003032(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        (d0, d1, d2, d4, d5, d6, d7, a0, a1, a4, a5, a6, x, c)
    }
}
