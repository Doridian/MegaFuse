//! An object's states: the routines its `OBJ_ROUTINE` runs a frame
//! (`func_002038`, `func_002DE2`, ...), the ones that put it in a state
//! (`func_002D3E`, `func_002E4E`, `func_002EB8`), what they share
//! (`func_0020E2`, `func_002E24`, `func_002F70`), and the view's
//! offsets (`func_002CC0`).

use crate::{Game, Regs};

/// The routines an object's state is set to (code addresses).
const STAY: u32 = 0x0020AE;
const NEXT_LOW: u32 = 0x001F7E;
const NEXT_HIGH: u32 = 0x0045E2;
const TIMED_LOW: u32 = 0x002DE2;
const TIMED_HIGH: u32 = 0x0046FC;
const THROWN_LOW: u32 = 0x002EB0;
const THROWN_HIGH: u32 = 0x004D6C;
const HELD_LOW: u32 = 0x002F34;
const HELD_HIGH: u32 = 0x00477A;

/// The object `func_00211C` and the walk work on (`b_walk.rs`).
const WALKER: u32 = 0xFFFF_A93F;
/// How much an object's `OBJ_TIMER_29` goes down a frame (and, with
/// the frame count, how often `func_002F70` calls `func_001EA2`).
const TICK: u32 = 0xFFFF_A942;
/// The frame count (`sprite.rs`).
const FRAME: u32 = 0xFFFF_80D8;
/// 8 bytes an object (by index): byte 3 its second request.
const PADS: u32 = 0xFFFF_AD0A;
/// Flags of the game (`b_control.rs`): bit 0 the effects' mode, bit
/// 7 no view offsets.
const MODE: u32 = 0xFFFF_9540;
/// Bytes by object index (`b_control.rs`).
const EFFECT: u32 = 0xFFFF_9E92;
const EFFECT_COUNT: u32 = 0xFFFF_9E8D;
const EFFECT_LEFT: u32 = 0xFFFF_9E97;
/// For an index of 5 on, the index it goes with (bytes, from index 5).
const OWNERS: u32 = 0xFFFF_A968;
/// The frames by `OBJ_LOOK` and facing (bytes in the ROM, `D_00B510`).
const LOOKS: u32 = 0x00B510;

/// The map's size in cells (bytes): 0x10 wide, 0x1A high the view has
/// no offset that way.
const WIDTH: u32 = 0xFFFF_9EB2;
const HEIGHT: u32 = 0xFFFF_9EB3;
/// A point (words) the view's offsets follow, less 0x88 and 0x68.
const POINT_X: u32 = 0xFFFF_A44A;
const POINT_Y: u32 = 0xFFFF_A44C;
/// The view's offsets (words), their low bytes.
const OFFSET_X: u32 = 0xFFFF_9EB4;
const OFFSET_X_LOW: u32 = 0xFFFF_9EB5;
const OFFSET_Y: u32 = 0xFFFF_9EB8;
const OFFSET_Y_LOW: u32 = 0xFFFF_9EB9;

/// An object's fields.
/// The routine run each frame, and the one put there when
/// `OBJ_TIMER_29` runs out (longs).
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_NEXT: u32 = 0x06;
/// Its sprite record's address and index (`anim.rs`).
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_FLAGS_10: u32 = 0x10;
/// Counted down while not 0 (`func_002F70`, out of the effects' mode).
const OBJ_COUNT_12: u32 = 0x12;
const OBJ_FACING: u32 = 0x14;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
/// A word set to 0xFD00 by `func_002E4E`; its low byte a timer.
const OBJ_28: u32 = 0x28;
const OBJ_TIMER_29: u32 = 0x29;
/// What it looks like: x 2 with its facing, into `LOOKS`.
const OBJ_LOOK: u32 = 0x2C;
/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const BUDGETED: u8 = 1 << 4;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

fn ext(reg: u32) -> u32 {
    low_word(reg, reg as u8 as i8 as u16)
}

/// `table` + the low word of `index` (signed).
fn at(table: u32, index: u32) -> u32 {
    table.wrapping_add(index as u16 as i16 as u32)
}

impl Game {
    /// `func_002038`: a state: `WALKER` 0, the object's animation on
    /// (`func_001336`), a step to its cell (`func_0020B0`); while
    /// `func_001ABC` gives N clear it stays (`OBJ_ROUTINE` `STAY`); else
    /// `func_002F70`, `func_001AD8`, `func_001B32`, `func_001B50`, and
    /// for index 5 `func_0047C6` and `func_004828`.
    pub fn r_002038(&mut self, regs: &mut Regs) {
        self.write8(WALKER, 0);
        let r = &mut *regs;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, _) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        self.arrive(regs, |g, r| {
            (r.d0, r.x, r.c) = g.r_001b50(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            r.c = (r.d4 as u8) < 5;
            if r.d4 as u8 == 5 {
                (r.d0, r.d1, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c) = g.r_0047c6(r.d0, r.d1, r.d4, r.d5, r.a4);
                (r.d0, r.d1, r.d4, r.d5, r.a4, r.a5, r.x, r.c) = g.r_004828(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3);
            }
        });
    }

    /// `func_002038`'s and `func_00207C`'s end: a step to the cell
    /// (`func_0020B0`); if `func_001ABC` gives N, `func_002F70`,
    /// `func_001AD8`, `func_001B32` and `then`; else the object stays
    /// (C clear).  Gives back whether it stays.
    fn arrive(&mut self, r: &mut Regs, then: fn(&mut Game, &mut Regs)) -> bool {
        (r.d0, r.x, r.c) = self.r_0020b0(r.d0, r.a4, r.x);
        let n;
        (r.d0, r.x, n) = self.r_001abc(r.d0, r.a4);
        if !n {
            self.write32(r.a4 + OBJ_ROUTINE, STAY);
            r.c = false;
            return true;
        }
        (r.d0, r.d1, r.a0, r.a1, r.x) = self.r_002f70(r.d0, r.d1, r.d4, r.a0, r.a1);
        self.r_001ad8(r.a4);
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        then(self, r);
        false
    }

    /// `func_00207C`: a state of object d4: `WALKER` it, a step to its
    /// cell; while `func_001ABC` gives N clear it stays (`STAY`, then
    /// `func_004568` and its frame by its look, `func_0020E2`); else as
    /// `func_002038` up to `func_001B32`.
    pub fn r_00207c(&mut self, regs: &mut Regs) {
        self.write8(WALKER, regs.d4 as u8);
        if self.arrive(regs, |_, _| {}) {
            let r = &mut *regs;
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.a3, r.a4) = self.r_004568(r.d1, r.d2, r.d3, r.d5, r.d6, r.d7, r.a1, r.a2, r.a4);
            (r.d0, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0020e2(r.d0, r.d4, r.d5);
        }
    }

    /// `func_00209E`: an `rts`.
    pub fn r_00209e(&mut self) {}

    /// `func_0020AE`: a state that does nothing (`STAY`).
    pub fn r_0020ae(&mut self, _regs: &mut Regs) {}

    /// `func_0020E2`: `WALKER`'s sprite frame by its look and facing
    /// (`LOOKS`), shown again.  Gives back d0, d5 and a5 (its sprite
    /// record's index and address), a0, X and C.
    pub fn r_0020e2(&mut self, d0: u32, d4: u32, d5: u32) -> (u32, u32, u32, u32, bool, bool) {
        let index = self.read8(WALKER);
        let (_, obj, _) = self.r_001cfa(low_byte(d4, index));
        let look = self.read8(obj + OBJ_LOOK);
        let (twice, x) = look.overflowing_add(look);
        let which = ext(low_byte(d0, twice | self.read8(obj + OBJ_FACING)));
        let frame = self.read8(at(LOOKS, which));
        let d0 = low_byte(which, frame);
        let d5 = low_byte(d5, self.read8(obj + OBJ_SPRITE));
        let sprite = self.read32(obj + OBJ_SPRITE_ADDR);
        self.write8(sprite + SPRITE_FRAME, frame);
        let flags = self.read8(sprite + SPRITE_FLAGS);
        self.write8(sprite + SPRITE_FLAGS, flags | BUDGETED);
        (d0, d5, LOOKS, sprite, x, false)
    }

    /// `func_002CC0`: the view's offsets from `POINT_X`, `POINT_Y`
    /// (less 0x88 and 0x68), not below 0; x's only on a map not 0x10
    /// wide (0x100 on: its low byte cleared), y's only on one 0x1A high
    /// (not above 0xD0).  Not in the effects' mode or with `MODE` bit
    /// 7.  Gives back d0, X and C.
    pub fn r_002cc0(&mut self, d0: u32, x: bool, c: bool) -> (u32, bool, bool) {
        let mode = self.read8(MODE);
        if mode & 1 != 0 || mode & 0x80 != 0 {
            return (d0, x, c);
        }
        let (mut d0, mut x) = (d0, x);
        if self.read8(WIDTH) != 0x10 {
            let point = self.read16(POINT_X);
            let offset = point.wrapping_sub(0x88);
            x = point < 0x88;
            d0 = low_word(d0, offset);
            self.write16(OFFSET_X, offset);
            if offset & 0x8000 != 0 {
                self.write16(OFFSET_X, 0);
            } else if offset >= 0x100 {
                self.write8(OFFSET_X_LOW, 0);
            }
        }
        let height = self.read8(HEIGHT);
        if height != 0x1A {
            return (d0, x, height < 0x1A);
        }
        let point = self.read16(POINT_Y);
        let offset = point.wrapping_sub(0x68);
        let x = point < 0x68;
        let d0 = low_word(d0, offset);
        self.write16(OFFSET_Y, offset);
        if offset & 0x8000 != 0 {
            self.write16(OFFSET_Y, 0);
            return (d0, x, false);
        }
        if offset < 0x100 && self.read8(OFFSET_Y_LOW) < 0xD0 {
            return (d0, x, true);
        }
        self.write16(OFFSET_Y, 0xD0);
        (d0, x, false)
    }

    /// `func_002D3E`: object `d0` (its low byte) put in the timed state
    /// (`func_002DE2`, below index 5) or, for index 5 or in the effects'
    /// mode, `func_0046FC` (its owner's frame shown facing 2 by
    /// `func_0020E2`); then facing its way, animation 0x34, timer 0x80.
    /// Other objects are left.  Gives back d0 and a0.
    pub fn r_002d3e(&mut self, d0: u32, d4: u32, d5: u32, a0: u32) -> (u32, u32) {
        let (d4, obj, _) = self.r_001cfa(low_byte(d4, d0 as u8));
        let index = d4 as u8;
        if index < 5 {
            self.write32(obj + OBJ_NEXT, NEXT_LOW);
            self.write32(obj + OBJ_ROUTINE, TIMED_LOW);
        } else if self.read8(MODE) & 1 != 0 || index == 5 {
            self.write32(obj + OBJ_NEXT, NEXT_HIGH);
            self.write32(obj + OBJ_ROUTINE, TIMED_HIGH);
            let owner = self.read8(at(OWNERS - 5, ext(d4)));
            self.write8(WALKER, owner);
            let d4 = low_byte(ext(d4), owner);
            let (d4, other, _) = self.r_001cfa(d4);
            let facing = self.read8(other + OBJ_FACING);
            self.write8(other + OBJ_FACING, 2);
            self.r_0020e2(low_byte(d0, facing), d4, d5);
            self.write8(other + OBJ_FACING, facing);
        } else {
            return (d0, a0);
        }
        let facing = self.read8(obj + OBJ_FACING) & 3;
        self.write8(obj + OBJ_FACING, facing);
        let a0 = self.r_00132a(0x34, obj);
        self.write8(obj + OBJ_TIMER, 0);
        self.write8(obj + OBJ_STEP, 0);
        self.write8(obj + OBJ_TIMER_29, 0x80);
        (0x34, a0)
    }

    /// `func_002DE2`: the timed state: `WALKER` d4, the animation on,
    /// `TICK` 2 (8 if the object's second pad byte has a way),
    /// `func_002F70`, the timer (`func_002E24`), `func_0015E6`,
    /// `func_00167A`, then `func_003032`.
    pub fn r_002de2(&mut self, regs: &mut Regs) {
        self.write8(WALKER, regs.d4 as u8);
        let r = &mut *regs;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        self.timed(regs, 2, 8);
    }

    /// `func_002F34`: the held state: `WALKER` d4, `TICK` 1 (4 if the
    /// object's second pad byte has a way), then as `func_002DE2`.  No
    /// script runs it (checked by reading).
    pub fn r_002f34(&mut self, regs: &mut Regs) {
        self.write8(WALKER, regs.d4 as u8);
        self.timed(regs, 1, 4);
    }

    /// `func_002DE2`'s and `func_002F34`'s end (see them).
    fn timed(&mut self, r: &mut Regs, tick: u8, fast: u8) {
        self.write8(TICK, tick);
        r.d4 = ext(r.d4);
        r.d0 = low_word(r.d0, (r.d4 as u16) << 3);
        r.a0 = PADS;
        let ways = self.read8(at(PADS + 3, r.d0)) & 0xF;
        r.d0 = low_byte(r.d0, ways);
        if ways != 0 {
            self.write8(TICK, fast);
        }
        (r.d0, r.d1, r.a0, r.a1, r.x) = self.r_002f70(r.d0, r.d1, r.d4, r.a0, r.a1);
        r.c = self.r_002e24(r.a4);
        (r.d0, r.x) = self.r_0015e6(r.a4);
        (r.a0, r.x) = self.r_00167a();
        (r.d0, r.d1, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c) = self.r_003032(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
    }

    /// `func_002E24`: the object's `OBJ_TIMER_29` down by `TICK`; when
    /// it runs out (C set), its animation from the start and
    /// `OBJ_NEXT` its routine.
    pub fn r_002e24(&mut self, obj: u32) -> bool {
        let timer = self.read8(obj + OBJ_TIMER_29);
        let tick = self.read8(TICK);
        if timer < tick {
            self.write8(obj + OBJ_TIMER, 0);
            self.write8(obj + OBJ_STEP, 0);
            let next = self.read32(obj + OBJ_NEXT);
            self.write32(obj + OBJ_ROUTINE, next);
            return true;
        }
        self.write8(obj + OBJ_TIMER_29, timer - tick);
        false
    }

    /// `func_002E4E`: object `d0` (its low byte) thrown: `func_002EB0`
    /// (below index 5) or `func_004D6C`, `OBJ_28` 0xFD00, facing 0x82,
    /// `OBJ_FLAGS_10` bit 7, animation 0x10, then `func_004FD0`.  Gives
    /// back d0 and a0.
    #[allow(clippy::too_many_arguments)]
    pub fn r_002e4e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a5: u32) -> (u32, u32) {
        let (d4, obj, _) = self.r_001cfa(low_byte(d4, d0 as u8));
        let (next, routine) = if (d4 as u8) < 5 { (NEXT_LOW, THROWN_LOW) } else { (NEXT_HIGH, THROWN_HIGH) };
        self.write32(obj + OBJ_NEXT, next);
        self.write32(obj + OBJ_ROUTINE, routine);
        self.write16(obj + OBJ_28, 0xFD00);
        self.write8(obj + OBJ_FACING, 0x82);
        let flags = self.read8(obj + OBJ_FLAGS_10);
        self.write8(obj + OBJ_FLAGS_10, flags | 0x80);
        let a0 = self.r_00132a(0x10, obj);
        self.write8(obj + OBJ_TIMER, 0);
        self.write8(obj + OBJ_STEP, 0);
        let (d0, _, _) = self.r_004fd0(0x10, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, obj, a5);
        (d0, a0)
    }

    /// `func_002EB0`: the thrown state: the animation on, then
    /// `func_004E5C`.
    pub fn r_002eb0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let z;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, z, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        let mut next = Regs { z, n: false, v: false, unwound: false, ..*r };
        self.r_004e5c(&mut next);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.x, r.c) = (next.d0, next.d1, next.d2, next.d3, next.d4, next.d5, next.x, next.c);
    }

    /// `func_002EB8`: object `d0` (its low byte) held: timer 0xF0, its
    /// facing's bits 0-1 kept, `func_002F34` (object 0) or `func_00477A`
    /// (its sprite frame 0x3B, and object 0 facing 0xFF); then the
    /// sprite frame 0x33 for it (object 0 in the second case).  No
    /// script calls it (checked by reading).
    pub fn r_002eb8(&mut self, d0: u32, d4: u32) {
        let (d4, mut obj, _) = self.r_001cfa(low_byte(d4, d0 as u8));
        self.write8(obj + OBJ_TIMER_29, 0xF0);
        let facing = self.read8(obj + OBJ_FACING);
        self.write8(obj + OBJ_FACING, facing & 3);
        if d4 as u8 != 0 {
            self.write32(obj + OBJ_NEXT, NEXT_HIGH);
            self.write32(obj + OBJ_ROUTINE, HELD_HIGH);
            self.show_frame(obj, 0x3B);
            (_, obj, _) = self.r_001cfa(low_byte(d4, 0));
            self.write8(obj + OBJ_FACING, 0xFF);
        } else {
            self.write32(obj + OBJ_NEXT, NEXT_LOW);
            self.write32(obj + OBJ_ROUTINE, HELD_LOW);
        }
        self.show_frame(obj, 0x33);
    }

    /// The object's sprite frame `frame`, shown again.
    fn show_frame(&mut self, obj: u32, frame: u8) {
        let sprite = self.read32(obj + OBJ_SPRITE_ADDR);
        self.write8(sprite + SPRITE_FRAME, frame);
        let flags = self.read8(sprite + SPRITE_FLAGS);
        self.write8(sprite + SPRITE_FLAGS, flags | BUDGETED);
    }

    /// `func_002F70`: `WALKER`'s flashing.  Out of the effects' mode:
    /// while its `OBJ_COUNT_12` isn't 0 it counts down, `func_001EC2`
    /// when bit 1 of it is set, else `func_001EA2` and `func_0094B2`.
    /// In it: with no `EFFECT_LEFT` or `EFFECT_COUNT` its `EFFECT`
    /// ends (`func_001EA2`, `func_0094B2`); else the counts go down
    /// (`EFFECT_LEFT` only when `EFFECT_COUNT` was 0) and, as the
    /// frame count has `TICK` (4; 8 when no `EFFECT_LEFT` and
    /// `EFFECT_COUNT` below 0x80), `func_001EA2` and `func_0094B2`,
    /// else `func_001EE0`.  Gives back d0, d1, a0, a1 and X.
    pub fn r_002f70(&mut self, d0: u32, d1: u32, d4: u32, a0: u32, a1: u32) -> (u32, u32, u32, u32, bool) {
        let index = self.read8(WALKER);
        let (d4, obj, x) = self.r_001cfa(low_byte(d4, index));
        let (mut d0, mut d1, mut a0, a1, mut x) = (d0, d1, a0, a1, x);
        if self.read8(MODE) & 1 == 0 {
            let count = self.read8(obj + OBJ_COUNT_12);
            if count == 0 {
                return (d0, d1, a0, a1, x);
            }
            self.write8(obj + OBJ_COUNT_12, count - 1);
            let bit = self.read8(obj + OBJ_COUNT_12) & 2;
            d0 = low_byte(d0, bit);
            if bit != 0 {
                (d0, d1, a0, x) = self.r_001ec2(d0, d4);
                return (d0, d1, a0, a1, x);
            }
        } else if self.read8(at(EFFECT_LEFT, d4)) == 0 && self.read8(at(EFFECT_COUNT, d4)) == 0 {
            self.write8(at(EFFECT, d4), 0);
        } else {
            let d4 = ext(d4);
            if self.read8(at(EFFECT_COUNT, d4)) == 0 {
                let left = self.read8(at(EFFECT_LEFT, d4));
                self.write8(at(EFFECT_LEFT, d4), left.wrapping_sub(1));
            }
            let count = self.read8(at(EFFECT_COUNT, d4));
            self.write8(at(EFFECT_COUNT, d4), count.wrapping_sub(1));
            self.write8(TICK, 4);
            if self.read8(at(EFFECT_LEFT, d4)) == 0 && self.read8(at(EFFECT_COUNT, d4)) < 0x80 {
                self.write8(TICK, 8);
            }
            let on = self.read8(FRAME) & self.read8(TICK);
            d0 = low_byte(d0, on);
            if on == 0 {
                return self.r_001ee0(d0, d4);
            }
            (d0, d1, a0, x) = self.r_001ea2(d0, d4);
            self.r_0094b2();
            return (d0, d1, a0, a1, x);
        }
        (d0, d1, a0, x) = self.r_001ea2(d0, d4);
        self.r_0094b2();
        (d0, d1, a0, a1, x)
    }
}
