//! An object walking on the map (0x23BA-0x2CBF and `func_0020B0`):
//! what `REQUEST` asks for turned into velocities, each way tried
//! against the cells ahead (`func_002B1A`'s probes), the object slid
//! round a corner when one probe is blocked and the next free.
//!
//! The four ways are the bits of `REQUEST`'s low nibble: bit 0 and
//! bit 1 move along y (`VELOCITY_Y`, by `SPEED_Y`), bit 3 and bit 2
//! along x (`VELOCITY_X`, by `SPEED_X`); each sets the object's
//! `OBJ_FACING` to 0, 2, 1 and 3.

use crate::{Game, Regs};

/// The request (a byte): bits 0-3 the ways asked for, the high nibble
/// kept apart.
const REQUEST: u32 = 0xFFFF_A932;
/// Two speeds (words): `SPEED_POS` for a way that adds, `SPEED_NEG`
/// for one that takes away (`func_00211C` sets them).
const SPEED_NEG: u32 = 0xFFFF_9EEA;
const SPEED_POS: u32 = 0xFFFF_9EEC;
/// The velocities (words) worked out from `REQUEST`, x and y, and what
/// is added to them each frame.
const SPEED_X: u32 = 0xFFFF_9EEE;
const SPEED_Y: u32 = 0xFFFF_9EF0;
const PUSH_X: u32 = 0xFFFF_9EF2;
const PUSH_Y: u32 = 0xFFFF_9EF4;
/// The velocity `func_0019C0` and `func_0019DA` move the object by,
/// x and y (words).
const VELOCITY_X: u32 = 0xFFFF_9EC4;
const VELOCITY_Y: u32 = 0xFFFF_9EC6;
/// A point (words, x then y), its low bytes set apart.
const FROM_X: u32 = 0xFFFF_9EFA;
const FROM_X_LOW: u32 = 0xFFFF_9EFB;
const FROM_Y: u32 = 0xFFFF_9EFE;
const FROM_Y_LOW: u32 = 0xFFFF_9EFF;
/// The object's position's low bytes, saved before it moves.
const SAVED_X_LOW: u32 = 0xFFFF_9F02;
const SAVED_Y_LOW: u32 = 0xFFFF_9F03;
/// A map cell (column, row) the probes move a step.
const CELL_X: u32 = 0xFFFF_9F08;
const CELL_Y: u32 = 0xFFFF_9F09;
/// What `func_0017B8` found in the last cell it looked at (1: a kind
/// that makes `func_002BFC` call `func_005AC4`).
const FOUND: u32 = 0xFFFF_9EF6;
/// The object being run (its index, for d4, and its address, for a4):
/// the probes set d4 and a4 back to them.
const CURRENT: u32 = 0xFFFF_A42C;
const CURRENT_ADDR: u32 = 0xFFFF_A42E;
/// An object's index (`func_001CFA`): its byte 0x10 says which cells
/// it may go through.
const WALKER: u32 = 0xFFFF_A93F;
/// The object's facing before it moved (`func_0023BA`).
const OLD_FACING: u32 = 0xFFFF_A93B;
/// Counts the probes that let the object through on the way.
const PASSED: u32 = 0xFFFF_A93D;
/// While this isn't 0, `func_00254C` doesn't try a slide.
const NO_SLIDE: u32 = 0xFFFF_A93E;
/// The running object's sprite record's address.
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// The probes' offsets (bytes, by probe: x in `PROBE_X`, y in
/// `PROBE_Y`), in the ROM.
const PROBE_X: u32 = 0x002AFE;
const PROBE_Y: u32 = 0x002B0C;
/// The slides by way (`func_002602`, ...) and the cell steps by way
/// (`func_002C54`, ...): longs in the ROM.
const SLIDES: u32 = 0x002586;
const STEPS: u32 = 0x002C44;
/// By way: the `REQUEST` bits it may not slide against (bytes, ROM).
const SLIDE_MASK: u32 = 0x009C6C;

/// An object's fields.
/// Which cells it may go through: bit 5 kind 1, bit 6 kinds 0, 2, 3.
const OBJ_PASS: u32 = 0x10;
/// The way it slides (`func_00254C`).
const OBJ_SLIDE: u32 = 0x13;
/// Its facing (bits 0-1), bit 7 set while it stands.
const OBJ_FACING: u32 = 0x14;
/// Its position (words), and their low bytes.
const OBJ_X: u32 = 0x18;
const OBJ_X_LOW: u32 = 0x19;
const OBJ_Y: u32 = 0x1A;
const OBJ_Y_LOW: u32 = 0x1B;
/// Its animation's frames to go and step (`anim.rs`).
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
/// A count `func_002494` takes `SPEED_POS`'s high byte from (not
/// below 0).
const OBJ_31: u32 = 0x31;
/// A sprite record's flags: shown again.
const SPRITE_FLAGS: u32 = 0x05;
const BUDGETED: u8 = 1 << 4;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// A routine of the object at a4 (`func_001A4C`, `func_001A02`, ...).
type ObjFn = fn(&mut Game, u32);
/// `func_001A4C`, `func_001A6A`: (d1, a4) -> d1.
type CenterFn = fn(&mut Game, u32, u32) -> u32;
/// `func_00182E`, `func_001840`: (d0, a4) -> (d0, Z, C).
type TestFn = fn(&mut Game, u32, u32) -> (u32, bool, bool);
/// `func_0019C0`, `func_0019DA`: (d0, d1, a4) -> (d0, d1, X).
type MoveFn = fn(&mut Game, u32, u32, u32) -> (u32, u32, bool);
/// The ways' tries and `func_002B84`'s kind: the registers they pass on.
type TryFn = fn(&mut Game, u32, u32, u32, u32, u32, u32, u32, u32, bool) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool);
type AlignFn = fn(&mut Game, u32, u32, u32, u32, u32, u32, u32, u32) -> (u32, u32, u32, u32, u32, u32, u32);
type SlideFn = fn(&mut Game, u32, u32, u32, u32) -> (u32, u32, u32, u32, bool, bool);

/// A slide (`func_00262A`, ...): the object, going one way, is first
/// put on its cell across (`center`, from its other coordinate `along`
/// into `from`); if probe `ahead` is blocked it goes on only when bit
/// 3 of `cell` (its position across, low byte) isn't `stuck`, after
/// `step` (and `back` when that made it so).  Then the two probes
/// beside, `sides`: when one is blocked and the other free, the object
/// moves across (`side_move` by `side_velocity` set from `SPEED_NEG`
/// or `SPEED_POS`), `side_push` after it as `side_test` says.
struct Slide {
    ahead: u8,
    along: u32,
    from: u32,
    center: CenterFn,
    cell: u32,
    stuck: bool,
    step: ObjFn,
    back: ObjFn,
    sides: (u8, u8),
    side_velocity: u32,
    side_move: MoveFn,
    side_test: TestFn,
    side_push: ObjFn,
}

/// Moving with x adding (bit 3), taking away (bit 2), y taking away
/// (bit 0), adding (bit 1).
const SLIDE_X_POS: Slide = Slide {
    ahead: 1, along: OBJ_Y, from: FROM_Y, center: Game::r_001a4c, cell: OBJ_X_LOW, stuck: true,
    step: Game::r_0019f4, back: Game::r_001858, sides: (7, 8), side_velocity: VELOCITY_Y,
    side_move: Game::r_0019da, side_test: Game::r_001840, side_push: Game::r_00186a,
};
const SLIDE_X_NEG: Slide = Slide {
    ahead: 3, along: OBJ_Y, from: FROM_Y, center: Game::r_001a4c, cell: OBJ_X_LOW, stuck: false,
    step: Game::r_0019f4, back: Game::r_001858, sides: (0xB, 0xC), side_velocity: VELOCITY_Y,
    side_move: Game::r_0019da, side_test: Game::r_001840, side_push: Game::r_00186a,
};
const SLIDE_Y_NEG: Slide = Slide {
    ahead: 0, along: OBJ_X, from: FROM_X, center: Game::r_001a6a, cell: OBJ_Y_LOW, stuck: false,
    step: Game::r_001a02, back: Game::r_00186a, sides: (5, 6), side_velocity: VELOCITY_X,
    side_move: Game::r_0019c0, side_test: Game::r_00182e, side_push: Game::r_001858,
};
const SLIDE_Y_POS: Slide = Slide {
    ahead: 2, along: OBJ_X, from: FROM_X, center: Game::r_001a6a, cell: OBJ_Y_LOW, stuck: true,
    step: Game::r_001a02, back: Game::r_00186a, sides: (9, 0xA), side_velocity: VELOCITY_X,
    side_move: Game::r_0019c0, side_test: Game::r_00182e, side_push: Game::r_001858,
};

/// What a way's slide is set to when the probe ahead lets it through
/// (`OBJ_SLIDE`): a number, or a byte of the ROM (the original reads
/// addresses 0 and 3 where it seems to mean the numbers).
#[derive(Clone, Copy)]
enum Turn {
    To(u8),
    RomByte(u32),
}

/// A way's try (`func_002596`, ...): asked for by `bit` (the object
/// then faces `facing`), when `speed` (a word) is above 0 (below, if
/// `negative`); `align` puts it on its cell, `slide` moves it, and
/// when that is blocked the probes `sides` say whether `func_00254C`
/// slides it.
struct Way {
    bit: u8,
    facing: u8,
    speed: u32,
    negative: bool,
    align: AlignFn,
    slide: SlideFn,
    sides: (u8, u8),
    turn: Turn,
}

const WAY_X_POS: Way = Way {
    bit: 3, facing: 1, speed: SPEED_X, negative: false, align: Game::r_002ba2,
    slide: Game::r_002624, sides: (7, 8), turn: Turn::To(1),
};
const WAY_X_NEG: Way = Way {
    bit: 2, facing: 3, speed: SPEED_X, negative: true, align: Game::r_002bde,
    slide: Game::r_00277c, sides: (0xB, 0xC), turn: Turn::RomByte(3),
};
const WAY_Y_NEG: Way = Way {
    bit: 0, facing: 0, speed: SPEED_Y, negative: true, align: Game::r_002b84,
    slide: Game::r_0028d4, sides: (6, 5), turn: Turn::RomByte(0),
};
const WAY_Y_POS: Way = Way {
    bit: 1, facing: 2, speed: SPEED_Y, negative: false, align: align_y_pos,
    slide: slide_y_pos, sides: (0xA, 9), turn: Turn::To(2),
};

/// `func_002BC0` and `func_002A2C` as the other ways' align and slide
/// (they leave a4 as it was, take it from `CURRENT_ADDR`).
#[allow(clippy::too_many_arguments)]
fn align_y_pos(g: &mut Game, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
    let (d0, d1, d4, d6, a0, a6) = g.r_002bc0(d0, d1, d2, d4, d6, a0, a4, a6);
    (d0, d1, d4, d6, a0, a4, a6)
}

fn slide_y_pos(g: &mut Game, d0: u32, d1: u32, d4: u32, _a4: u32) -> (u32, u32, u32, u32, bool, bool) {
    g.r_002a2c(d0, d1, d4)
}

impl Game {
    fn request(&mut self, bit: u8) -> bool {
        self.read8(REQUEST) & (1 << bit) != 0
    }

    /// d4 and a4 set back to `CURRENT`'s.
    fn current(&mut self, d4: u32) -> (u32, u32) {
        (low_byte(d4, self.read8(CURRENT)), self.read32(CURRENT_ADDR))
    }

    /// `func_0020B0`: the object's position's low bytes a step toward
    /// its cell, x then y, as `func_00182E` and `func_001840` say (Z:
    /// there; C: back).  Gives back d0, X and C as the original
    /// leaves them.
    pub fn r_0020b0(&mut self, d0: u32, obj: u32, x: bool) -> (u32, bool, bool) {
        let mut x = x;
        let (d0, z, c) = self.r_00182e(d0, obj);
        if !z {
            x = self.step_byte(obj + OBJ_X_LOW, c);
        }
        let (d0, z, mut c) = self.r_001840(d0, obj);
        if !z {
            x = self.step_byte(obj + OBJ_Y_LOW, c);
            c = x;
        }
        (d0, x, c)
    }

    /// The byte at `at` one less (`down`) or one more; gives back the
    /// carry.
    fn step_byte(&mut self, at: u32, down: bool) -> bool {
        let b = self.read8(at);
        if down {
            self.write8(at, b.wrapping_sub(1));
            b == 0
        } else {
            self.write8(at, b.wrapping_add(1));
            b == 0xFF
        }
    }

    /// `func_0023BA`: the object at a4 moved as `REQUEST` asks: the
    /// velocities from `SPEED_NEG`, `SPEED_POS` (0 for an axis not
    /// asked for) plus `PUSH_X`, `PUSH_Y`; asked for nothing, it stands
    /// (`OBJ_FACING`'s bit 7, its animation from the start); then the
    /// ways tried (`func_002494`), and its sprite shown again if its
    /// facing changed.  Gives back d0, d1, d2, d4, d6, a0, a4, a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0023ba(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d5: u32, d6: u32, a0: u32, obj: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        self.r_00157a();
        let x_low = self.read8(obj + OBJ_X_LOW);
        self.write8(SAVED_X_LOW, x_low);
        let y_low = self.read8(obj + OBJ_Y_LOW);
        self.write8(SAVED_Y_LOW, y_low);
        let request = self.read8(REQUEST);
        let speed_x = if request & 0xC == 0 {
            0
        } else if request & 8 != 0 {
            self.read16(SPEED_POS)
        } else {
            self.read16(SPEED_NEG)
        };
        self.write16(SPEED_X, speed_x);
        let request = self.read8(REQUEST);
        let speed_y = match request & 3 {
            0 => 0,
            1 | 3 => self.read16(SPEED_NEG),
            _ => self.read16(SPEED_POS),
        };
        self.write16(SPEED_Y, speed_y);
        let push_x = self.read16(PUSH_X);
        let sx = self.read16(SPEED_X);
        self.write16(SPEED_X, sx.wrapping_add(push_x));
        let push_y = self.read16(PUSH_Y);
        let sy = self.read16(SPEED_Y);
        let (sy, mut x) = sy.overflowing_add(push_y);
        self.write16(SPEED_Y, sy);
        let facing = self.read8(obj + OBJ_FACING);
        self.write8(OLD_FACING, facing & 3);
        let request = self.read8(REQUEST);
        let mut d0 = low_word(d0, push_y);
        let (mut d1, mut d2, mut a0) = (d1, d2, a0);
        if request & 0xF == 0 {
            let facing = self.read8(obj + OBJ_FACING);
            d0 = low_byte(d0, facing);
            if facing & 0x80 == 0 {
                d0 = low_byte(d0, facing | 0x80);
                self.write8(obj + OBJ_FACING, facing | 0x80);
                self.write8(obj + OBJ_TIMER, 0);
                self.write8(obj + OBJ_STEP, 0);
                (_, x, _) = self.r_00130e(d0, obj);
                (d0, d1, d2, _, a0, _, x, _, _) = self.r_001336(d0, d1, d2, d5, obj, x);
            }
        } else {
            d0 = low_byte(d0, request & 0xF);
        }
        let (d0, d1, d4, d6, a0, obj, a6) = self.r_002494(d0, d1, d2, d4, d6, a0, obj, a6, x);
        let facing = self.read8(obj + OBJ_FACING);
        let d0 = if facing & 3 == self.read8(OLD_FACING) {
            low_byte(d0, facing & 3)
        } else {
            let sprite = self.read32(SLOT_ADDR);
            let flags = self.read8(sprite + SPRITE_FLAGS);
            self.write8(sprite + SPRITE_FLAGS, flags | BUDGETED);
            low_byte(d0, self.read8(obj + OBJ_FACING))
        };
        (d0, d1, d2, d4, d6, a0, obj, a6)
    }

    /// `func_002494`: the ways tried, those `REQUEST` asks for along x
    /// first if any (x adding, x taking away, y taking away, y adding),
    /// else y first (y taking away, y adding, x adding, x taking
    /// away), until one moves the object (C clear) and was asked for
    /// (or is the last); then `WALKER`'s `OBJ_31` goes down by
    /// `SPEED_POS`'s high byte (not below 0).  Gives back d0, d1, d4,
    /// d6, a0, a4, a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_002494(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32) {
        let request = self.read8(REQUEST);
        let mut r = (low_byte(d0, request & 0xC), d1, d4, d6, a0, a4, a6, x, false);
        let order: [(TryFn, u8); 4] = if request & 0xC != 0 {
            [(Game::r_002596, 3), (Game::r_0026ee, 2), (Game::r_002846, 0), (Game::r_00299e, 0xFF)]
        } else {
            [(Game::r_002846, 0), (Game::r_00299e, 1), (Game::r_002596, 3), (Game::r_0026ee, 0xFF)]
        };
        let mut moved = false;
        for (way, bit) in order {
            let (d0, d1, d4, d6, a0, a4, a6, x, _) = r;
            r = way(self, d0, d1, d2, d4, d6, a0, a4, a6, x);
            if !r.8 && (bit == 0xFF || self.request(bit)) {
                moved = true;
                break;
            }
            if bit == 0xFF {
                break;
            }
        }
        let (d0, d1, d4, d6, a0, a4, a6, _, _) = r;
        if !moved {
            return (d0, d1, d4, d6, a0, a4, a6);
        }
        let d4 = low_byte(d4, self.read8(WALKER));
        let (d4, walker, _) = self.r_001cfa(d4);
        let left = self.read8(walker + OBJ_31).wrapping_sub(self.read8(SPEED_POS));
        let left = if left & 0x80 != 0 { 0 } else { left };
        self.write8(walker + OBJ_31, left);
        let (d4, a4) = self.current(d4);
        (low_byte(d0, left), d1, d4, d6, a0, a4, a6)
    }

    /// `func_00254C`: the object slides the way `OBJ_SLIDE` says (a
    /// routine of `SLIDES`), unless `NO_SLIDE` or `REQUEST` has one of
    /// `SLIDE_MASK`'s bits for that way: then d4 and a4 are
    /// `CURRENT`'s and C is set.  Gives back d0, d1, d4, a0, a4, X, C.
    pub fn r_00254c(&mut self, d0: u32, d1: u32, d4: u32, a0: u32, a4: u32, x: bool) -> (u32, u32, u32, u32, u32, bool, bool) {
        if self.read8(NO_SLIDE) == 0 {
            let way = self.read8(a4 + OBJ_SLIDE);
            let index = way as i8 as u16;
            let d0 = low_word(d0, index);
            let mask = self.read8(SLIDE_MASK.wrapping_add(index as i16 as u32));
            let mask = mask & self.read8(REQUEST);
            let d1 = low_byte(d1, mask);
            if mask == 0 {
                let (by4, carry) = way.wrapping_add(way).overflowing_add(way.wrapping_add(way));
                let d0 = low_byte(d0, by4);
                let slide = self.read32(SLIDES.wrapping_add(d0 as u16 as i16 as u32));
                let mut regs = Regs { d0, d1, d4, a0: slide, a4, x: carry, ..Regs::default() };
                self.call_family_23(slide, &mut regs);
                return (regs.d0, regs.d1, regs.d4, slide, regs.a4, regs.x, regs.c);
            }
            let (d4, a4) = self.current(d4);
            return (d0, d1, d4, a0, a4, x, true);
        }
        let (d4, a4) = self.current(d4);
        (d0, d1, d4, a0, a4, x, true)
    }

    /// A way's try (`func_002596`, ...; see `Way`).  C set: the object
    /// didn't move that way.
    #[allow(clippy::too_many_arguments)]
    fn try_way(&mut self, way: &Way, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        if self.request(way.bit) {
            self.write8(a4 + OBJ_FACING, way.facing);
        }
        let high = self.read8(way.speed);
        let mut d0 = d0;
        let going = if way.negative {
            high & 0x80 != 0
        } else if high & 0x80 != 0 {
            false
        } else if high != 0 {
            true
        } else {
            let low = self.read8(way.speed + 1);
            d0 = low_byte(d0, low);
            low != 0
        };
        if !going {
            return (d0, d1, d4, d6, a0, a4, a6, x, true);
        }
        let (d0, d1, d4, d6, a0, a4, a6) = (way.align)(self, d0, d1, d2, d4, d6, a0, a4, a6);
        let (d0, d1, d4, a4, x, c) = (way.slide)(self, d0, d1, d4, a4);
        if !c {
            if self.request(way.bit) {
                let turn = match way.turn {
                    Turn::To(n) => n,
                    Turn::RomByte(at) => self.read8(at),
                };
                self.write8(a4 + OBJ_SLIDE, turn);
            }
            return (d0, d1, d4, d6, a0, a4, a6, x, false);
        }
        let (d0, d1, d4, a4, x, c) = self.r_002b1a(low_byte(d0, way.sides.0), d1, d4);
        let (d0, d1, d4, a4, x, c) = if c {
            self.r_002b1a(low_byte(d0, way.sides.1), d1, d4)
        } else {
            (d0, d1, d4, a4, x, false)
        };
        if c {
            return (d0, d1, d4, d6, a0, a4, a6, x, true);
        }
        let (d0, d1, d4, a0, a4, x, c) = self.r_00254c(d0, d1, d4, a0, a4, x);
        (d0, d1, d4, d6, a0, a4, a6, x, c)
    }

    /// `func_002596`: the way x adding tried (`Way`).
    #[allow(clippy::too_many_arguments)]
    pub fn r_002596(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.try_way(&WAY_X_POS, d0, d1, d2, d4, d6, a0, a4, a6, x)
    }

    /// `func_0026EE`: the way x taking away tried (`Way`).
    #[allow(clippy::too_many_arguments)]
    pub fn r_0026ee(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.try_way(&WAY_X_NEG, d0, d1, d2, d4, d6, a0, a4, a6, x)
    }

    /// `func_002846`: the way y taking away tried (`Way`).
    #[allow(clippy::too_many_arguments)]
    pub fn r_002846(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.try_way(&WAY_Y_NEG, d0, d1, d2, d4, d6, a0, a4, a6, x)
    }

    /// `func_00299E`: the way y adding tried (`Way`).
    #[allow(clippy::too_many_arguments)]
    pub fn r_00299e(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.try_way(&WAY_Y_POS, d0, d1, d2, d4, d6, a0, a4, a6, x)
    }

    /// A slide's way in (`func_002602`, ...): d4 and a4 `CURRENT`'s;
    /// not when `test` says the object is on its cell (Z) or gives 7;
    /// else `velocity` from `speed` and the slide.
    fn slide_in(&mut self, regs: &mut Regs, test: TestFn, velocity: u32, speed: u32, slide: &Slide) {
        let (d4, a4) = self.current(regs.d4);
        regs.d4 = d4;
        regs.a4 = a4;
        let (d0, z, _) = test(self, regs.d0, a4);
        regs.d0 = d0;
        if z || d0 as u8 == 7 {
            regs.c = true;
            return;
        }
        let v = self.read16(speed);
        self.write16(velocity, v);
        (regs.d0, regs.d1, regs.d4, regs.a4, regs.x, regs.c) = self.slide(slide, regs.d0, regs.d1, regs.d4, a4);
    }

    /// `func_002602`: slides x adding, by `SPEED_POS` (`slide_in`).
    pub fn r_002602(&mut self, regs: &mut Regs) {
        self.slide_in(regs, Game::r_00182e, VELOCITY_X, SPEED_POS, &SLIDE_X_POS);
    }

    /// `func_00275A`: slides x taking away, by `SPEED_NEG`.  No script
    /// calls it (checked by reading: `func_002602` with the other
    /// speed and slide).
    pub fn r_00275a(&mut self, regs: &mut Regs) {
        self.slide_in(regs, Game::r_00182e, VELOCITY_X, SPEED_NEG, &SLIDE_X_NEG);
    }

    /// `func_0028B2`: slides y taking away, by `SPEED_NEG`.
    pub fn r_0028b2(&mut self, regs: &mut Regs) {
        self.slide_in(regs, Game::r_001840, VELOCITY_Y, SPEED_NEG, &SLIDE_Y_NEG);
    }

    /// `func_002A0A`: slides y adding, by `SPEED_POS`.
    pub fn r_002a0a(&mut self, regs: &mut Regs) {
        self.slide_in(regs, Game::r_001840, VELOCITY_Y, SPEED_POS, &SLIDE_Y_POS);
    }

    /// `func_002624`: `VELOCITY_X` from `SPEED_X`, the slide x adding
    /// (`func_00262A`).
    pub fn r_002624(&mut self, d0: u32, d1: u32, d4: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        let v = self.read16(SPEED_X);
        self.write16(VELOCITY_X, v);
        self.r_00262a(d0, d1, d4, a4)
    }

    /// `func_00277C`: `VELOCITY_X` from `SPEED_X`, the slide x taking
    /// away (`func_002782`).
    pub fn r_00277c(&mut self, d0: u32, d1: u32, d4: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        let v = self.read16(SPEED_X);
        self.write16(VELOCITY_X, v);
        self.r_002782(d0, d1, d4, a4)
    }

    /// `func_0028D4`: `VELOCITY_Y` from `SPEED_Y`, the slide y taking
    /// away (`func_0028DA`).
    pub fn r_0028d4(&mut self, d0: u32, d1: u32, d4: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        let v = self.read16(SPEED_Y);
        self.write16(VELOCITY_Y, v);
        self.r_0028da(d0, d1, d4, a4)
    }

    /// `func_002A2C`: d4 and a4 `CURRENT`'s, `VELOCITY_Y` from
    /// `SPEED_Y`, the slide y adding (`func_002A3A`).
    pub fn r_002a2c(&mut self, d0: u32, d1: u32, d4: u32) -> (u32, u32, u32, u32, bool, bool) {
        let (d4, a4) = self.current(d4);
        let v = self.read16(SPEED_Y);
        self.write16(VELOCITY_Y, v);
        self.r_002a3a(d0, d1, d4, a4)
    }

    /// `func_00262A`: the slide x adding (`Slide`).
    pub fn r_00262a(&mut self, d0: u32, d1: u32, d4: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        self.slide(&SLIDE_X_POS, d0, d1, d4, a4)
    }

    /// `func_002782`: the slide x taking away (`Slide`).
    pub fn r_002782(&mut self, d0: u32, d1: u32, d4: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        self.slide(&SLIDE_X_NEG, d0, d1, d4, a4)
    }

    /// `func_0028DA`: the slide y taking away (`Slide`).
    pub fn r_0028da(&mut self, d0: u32, d1: u32, d4: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        self.slide(&SLIDE_Y_NEG, d0, d1, d4, a4)
    }

    /// `func_002A3A`: the slide y adding (`Slide`).
    pub fn r_002a3a(&mut self, d0: u32, d1: u32, d4: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        self.slide(&SLIDE_Y_POS, d0, d1, d4, a4)
    }

    /// `loc_002662`, `loc_0027BA`, `loc_002912`, `loc_002A72`: a slide
    /// that can't go: C set.
    pub fn r_002662(&mut self) -> bool {
        true
    }

    /// `loc_0027BA`: as `r_002662`.
    pub fn r_0027ba(&mut self) -> bool {
        true
    }

    /// `loc_002912`: as `r_002662`.
    pub fn r_002912(&mut self) -> bool {
        true
    }

    /// `loc_002A72`: as `r_002662`.
    pub fn r_002a72(&mut self) -> bool {
        true
    }

    /// A slide (see `Slide`).  C set: the object didn't move (no probe
    /// let it through).  Gives back d0, d1, d4, a4, X, C.
    fn slide(&mut self, s: &Slide, d0: u32, d1: u32, d4: u32, a4: u32) -> (u32, u32, u32, u32, bool, bool) {
        self.write8(PASSED, 0);
        let along = self.read16(a4 + s.along);
        self.write16(s.from, along);
        let d1 = (s.center)(self, d1, a4);
        let (d0, d1, d4, a4, x, blocked) = self.r_002b1a(low_byte(d0, s.ahead), d1, d4);
        let stuck = |g: &mut Game| (g.read8(a4 + s.cell) & 8 != 0) == s.stuck;
        if blocked {
            if stuck(self) {
                return (d0, d1, d4, a4, x, true);
            }
            (s.step)(self, a4);
            if stuck(self) {
                (s.back)(self, a4);
                return (d0, d1, d4, a4, x, true);
            }
        } else {
            (s.step)(self, a4);
        }
        let passed = self.read8(PASSED);
        self.write8(PASSED, passed.wrapping_add(1));
        let (d0, d1, d4, _, _, first) = self.r_002b1a(low_byte(d0, s.sides.0), d1, d4);
        let (d0, d1, d4, a4, x, second) = self.r_002b1a(low_byte(d0, s.sides.1), d1, d4);
        if first == second {
            let passed = self.read8(PASSED);
            return (low_byte(d0, passed), d1, d4, a4, x, passed == 0);
        }
        // one side blocked, the other free: across, away from the
        // blocked one
        let speed = self.read16(if first { SPEED_POS } else { SPEED_NEG });
        self.write16(s.side_velocity, speed);
        let (d0, d1, _) = (s.side_move)(self, d0, d1, a4);
        let passed = self.read8(PASSED);
        self.write8(PASSED, passed.wrapping_add(1));
        let x = passed == 0xFF;
        let (d0, _, c) = (s.side_test)(self, d0, a4);
        if c == first {
            (s.side_push)(self, a4);
        }
        (d0, d1, d4, a4, x, false)
    }

    /// `func_002B1A`: probe `d0` (its low byte): the cell at the
    /// offsets `PROBE_X`, `PROBE_Y` from the object (`func_0017B8`,
    /// C set: blocked); a kind 1 lets `WALKER` through if its
    /// `OBJ_PASS` has bit 5, kinds 0, 2 and 3 if it has bit 6, others
    /// always.  d4 and a4 are `CURRENT`'s after.  Gives back d0, d1,
    /// d4, a4, X and C (set: blocked).
    pub fn r_002b1a(&mut self, d0: u32, d1: u32, d4: u32) -> (u32, u32, u32, u32, bool, bool) {
        let index = d0 as u8 as i8 as i16;
        let d0 = low_word(d0, index as u16);
        let dy = self.read8(PROBE_Y.wrapping_add(index as u32));
        let d1 = low_byte(d1, dy);
        let dx = self.read8(PROBE_X.wrapping_add(index as u32));
        let d0 = low_byte(d0, dx);
        let (d4, _) = self.current(d4);
        let (mut d0, mut x, c) = self.r_0017b8(d0, d1);
        let mut blocked = c;
        let mut d4 = d4;
        if !c {
            let walker;
            let index = self.read8(WALKER);
            (d4, walker, x) = self.r_001cfa(low_byte(d4, index));
            let kind = d0 as u8;
            if kind < 4 {
                let pass = self.read8(walker + OBJ_PASS);
                d0 = low_byte(d0, pass);
                blocked = pass & (if kind == 1 { 1 << 5 } else { 1 << 6 }) == 0;
            }
        }
        let (d4, a4) = self.current(d4);
        (d0, d1, d4, a4, x, blocked)
    }

    /// An align (`func_002B84`, ...): when `REQUEST` asks for `bit`,
    /// the cell a step `way` (`func_002C36`) and, if that can be,
    /// `func_002BFC`.
    #[allow(clippy::too_many_arguments)]
    fn align(&mut self, bit: u8, way: u32, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        if !self.request(bit) {
            return (d0, d1, d4, d6, a0, a4, a6);
        }
        let (d0, a0, _, c) = self.r_002c36(way, a4);
        if c {
            return (d0, d1, d4, d6, a0, a4, a6);
        }
        self.r_002bfc(d0, d1, d2, d4, d6, a0, a4, a6)
    }

    /// `func_002B84`: align for bit 0 (y taking away; `align`).
    #[allow(clippy::too_many_arguments)]
    pub fn r_002b84(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        self.align(0, 0, d0, d1, d2, d4, d6, a0, a4, a6)
    }

    /// `func_002BA2`: align for bit 3 (x adding).
    #[allow(clippy::too_many_arguments)]
    pub fn r_002ba2(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        self.align(3, 1, d0, d1, d2, d4, d6, a0, a4, a6)
    }

    /// `func_002BC0`: align for bit 1 (y adding).  Gives back all but
    /// a4.
    #[allow(clippy::too_many_arguments)]
    pub fn r_002bc0(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32) {
        let (d0, d1, d4, d6, a0, _, a6) = self.align(1, 2, d0, d1, d2, d4, d6, a0, a4, a6);
        (d0, d1, d4, d6, a0, a6)
    }

    /// `func_002BDE`: align for bit 2 (x taking away).
    #[allow(clippy::too_many_arguments)]
    pub fn r_002bde(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        self.align(2, 3, d0, d1, d2, d4, d6, a0, a4, a6)
    }

    /// `func_002BFC`: the object's position's low bytes into `FROM_X`,
    /// `FROM_Y`, probe by its facing (`func_002B1A`); a cell of kind 1
    /// found, `func_005AC4` by its facing and, if that clears C, sound
    /// command 0x34 (`func_0034B6`); then `func_0015E6`.  Gives back
    /// d0, d1, d4, d6, a0, a4, a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_002bfc(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d6: u32, a0: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        let x_low = self.read8(a4 + OBJ_X_LOW);
        self.write8(FROM_X_LOW, x_low);
        let y_low = self.read8(a4 + OBJ_Y_LOW);
        self.write8(FROM_Y_LOW, y_low);
        let facing = self.read8(a4 + OBJ_FACING);
        let (d0, mut d1, mut d4, mut a4, _, _) = self.r_002b1a(low_byte(d0, facing), d1, d4);
        let found = self.read8(FOUND);
        let mut d0 = low_byte(d0, found);
        let (mut d6, mut a0, mut a6) = (d6, a0, a6);
        if found == 1 {
            d0 = low_byte(d0, self.read8(a4 + OBJ_FACING));
            let c;
            (d1, d4, d6, a0, a4, a6, c) = self.r_005ac4(d0, d1, d2, d4, d6, a0, a6);
            if !c {
                d0 = low_byte(d0, 0x34);
                self.r_0034b6(d0);
            }
        }
        let (d0, _) = self.r_0015e6(a4);
        (d0, d1, d4, d6, a0, a4, a6)
    }

    /// `func_002C36`: the cell a step way `d0` (its low byte), by the
    /// routine of `STEPS` (`func_002C54`, ...).  Gives back d0, a0 (the
    /// routine), X and C (set: it can't).
    pub fn r_002c36(&mut self, d0: u32, a4: u32) -> (u32, u32, bool, bool) {
        let w = d0 as u8 as i8 as u16;
        let w2 = w.wrapping_add(w);
        let (by4, carry) = w2.overflowing_add(w2);
        let d0 = low_word(d0, by4);
        let step = self.read32(STEPS.wrapping_add(by4 as i16 as u32));
        let mut regs = Regs { d0, a0: step, a4, x: carry, c: carry, ..Regs::default() };
        self.call_family_14(step, &mut regs);
        (regs.d0, step, regs.x, regs.c)
    }

    /// `func_002C54`: `CELL_Y` a row up, unless `func_001840` gives 9
    /// or more or the row goes below 0 (C set).  Gives back d0, X, C.
    pub fn r_002c54(&mut self, regs: &mut Regs) {
        let (d0, _, _) = self.r_001840(regs.d0, regs.a4);
        regs.d0 = d0;
        if (d0 as u8) < 9 {
            let row = self.read8(CELL_Y);
            self.write8(CELL_Y, row.wrapping_sub(1));
            regs.x = row == 0;
            regs.c = row.wrapping_sub(1) & 0x80 != 0;
        } else {
            regs.c = true;
        }
    }

    /// `func_002C74`: `CELL_X` a column on if `func_00182E` sets C
    /// (else C set).  Gives back d0, X, C.
    pub fn r_002c74(&mut self, regs: &mut Regs) {
        let (d0, _, c) = self.r_00182e(regs.d0, regs.a4);
        regs.d0 = d0;
        if c {
            regs.x = self.step_byte(CELL_X, false);
        }
        regs.c = !c;
    }

    /// `func_002C8C`: `CELL_Y` a row down if `func_001840` sets C
    /// (else C set).  Gives back d0, X, C.
    pub fn r_002c8c(&mut self, regs: &mut Regs) {
        let (d0, _, c) = self.r_001840(regs.d0, regs.a4);
        regs.d0 = d0;
        if c {
            regs.x = self.step_byte(CELL_Y, false);
        }
        regs.c = !c;
    }

    /// `func_002CA4`: `CELL_X` a column back, unless `func_00182E`
    /// gives 9 or more (C set).  Gives back d0, X, C.
    pub fn r_002ca4(&mut self, regs: &mut Regs) {
        let (d0, _, _) = self.r_00182e(regs.d0, regs.a4);
        regs.d0 = d0;
        if (d0 as u8) < 9 {
            regs.x = self.step_byte(CELL_X, true);
            regs.c = false;
        } else {
            regs.c = true;
        }
    }
}
