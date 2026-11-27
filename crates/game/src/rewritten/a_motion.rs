//! Positions and velocities of objects (a4) and tasks (a6), and their
//! sprites' places on the screen.
//!
//! A position on an axis is a word of pixels with a byte of fraction
//! before it (see the fields below); a velocity is a word whose high
//! byte is the pixels (signed) and low byte the fraction.  A cell is 16
//! pixels: a position is in the middle of one when the low nibble of
//! its pixels is 8.

use crate::Game;

/// An object's fields: x (fraction, pixels), y (fraction, pixels); the
/// pixels' low bytes.
const OBJ_X_FRACTION: u32 = 0x16;
const OBJ_Y_FRACTION: u32 = 0x17;
const OBJ_X: u32 = 0x18;
const OBJ_X_LOW: u32 = 0x19;
const OBJ_Y: u32 = 0x1A;
const OBJ_Y_LOW: u32 = 0x1B;
/// A word that `func_001AD8`, `func_001AF6` add to (its high byte
/// capped at 4), and the word it is added to (`func_001ABC`), whose
/// high byte is an offset to y on the screen.
const OBJ_RATE: u32 = 0x28;
const OBJ_OFFSET: u32 = 0x2A;

/// A task's fields, as an object's.
const TASK_X_FRACTION: u32 = 0x0C;
const TASK_Y_FRACTION: u32 = 0x0D;
const TASK_X: u32 = 0x0E;
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y: u32 = 0x10;
const TASK_Y_LOW: u32 = 0x11;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;
/// A byte added to the sprite's x (`func_001BA4`).
const TASK_OFFSET_X: u32 = 0x21;

/// A sprite record's position (words, x then y; one long) and the low
/// byte of its y.
const SPRITE_X: u32 = 0x08;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;

/// A velocity (words, x then y; the fraction the byte after each) and
/// a speed (a word), set by `func_001620`.
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;
const SPEED: u32 = 0xFF9F00;
/// A position worked out apart from the object's or task's: x
/// (fraction, pixels), y (fraction, pixels).
const POS_X_FRACTION: u32 = 0xFF9EF9;
const POS_X: u32 = 0xFF9EFA;
const POS_Y_FRACTION: u32 = 0xFF9EFD;
const POS_Y: u32 = 0xFF9EFE;
/// Subtracted from a position for the sprite's (a long: x, y), and a
/// word subtracted from y as well.
const VIEW: u32 = 0xFF80E6;
const VIEW_Y_MORE: u32 = 0xFF80EA;
/// Added to both of a sprite's coordinates (the screen's border).
const BORDER: u16 = 0x10;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// (fraction, pixels) plus a velocity (`whole`, signed, and `fraction`),
/// the fraction's carry going to the pixels.  Gives back the new
/// fraction, pixels and the pixels' carry.
fn add_velocity(fraction: u8, pixels: u16, whole: u8, by_fraction: u8) -> (u8, u16, bool) {
    let (fraction, carry) = fraction.overflowing_add(by_fraction);
    let whole = whole as i8 as i16 as u16;
    let sum = u32::from(pixels) + u32::from(whole) + u32::from(carry);
    (fraction, sum as u16, sum > 0xFFFF)
}

impl Game {
    /// `func_001620`: the velocity at `SPEED` in direction `d0` (its
    /// bits 0-1: 0 up, 1 right, 2 down, 3 left); `SPEED` is left negated
    /// for up and left.
    pub fn r_001620(&mut self, d0: u32) {
        let direction = d0 as u8;
        let across = direction & 1 != 0;
        let toward_start = (direction & 2 != 0) == across;
        if toward_start {
            let speed = self.read16(SPEED);
            self.write16(SPEED, speed.wrapping_neg());
        }
        let speed = self.read16(SPEED);
        let (moving, still) = if across { (VELOCITY_X, VELOCITY_Y) } else { (VELOCITY_Y, VELOCITY_X) };
        self.write16(moving, speed);
        self.write16(still, 0);
    }

    /// d0 = the low nibble of the byte at `at` (in its low byte); Z set
    /// when it is 8 (the middle of a cell), C when it is 8 or more.
    fn in_cell(&mut self, d0: u32, at: u32) -> (u32, bool, bool) {
        let nibble = self.read8(at) & 0x0F;
        (low_byte(d0, nibble), nibble == 8, nibble >= 8)
    }

    /// `func_00182E`: the object's x within its cell.  Gives back d0, Z
    /// (in the middle) and C (in the middle or past it).
    pub fn r_00182e(&mut self, d0: u32, a4: u32) -> (u32, bool, bool) {
        self.in_cell(d0, a4 + OBJ_X_LOW)
    }

    /// `func_001840`: the object's y within its cell, as `func_00182E`.
    pub fn r_001840(&mut self, d0: u32, a4: u32) -> (u32, bool, bool) {
        self.in_cell(d0, a4 + OBJ_Y_LOW)
    }

    /// The pixels' byte at `at` put in the middle of its cell.
    fn centre(&mut self, at: u32) {
        let low = self.read8(at);
        self.write8(at, (low & 0xF0) | 8);
    }

    /// `func_001852`: the object's y, then x, put in the middle of the
    /// cell (`func_00186A`, `func_001858`).
    pub fn r_001852(&mut self, a4: u32) {
        self.r_00186a(a4);
        self.r_001858(a4);
    }

    /// `func_001858`: the object's x put in the middle of its cell, its
    /// fraction cleared.
    pub fn r_001858(&mut self, a4: u32) {
        self.write8(a4 + OBJ_X_FRACTION, 0);
        self.centre(a4 + OBJ_X_LOW);
    }

    /// `func_00186A`: the object's y put in the middle of its cell, its
    /// fraction cleared.
    pub fn r_00186a(&mut self, a4: u32) {
        self.write8(a4 + OBJ_Y_FRACTION, 0);
        self.centre(a4 + OBJ_Y_LOW);
    }

    /// `func_00187C`: the task's x and y put in the middle of the cell
    /// (the fractions kept).
    pub fn r_00187c(&mut self, a6: u32) {
        self.centre(a6 + TASK_X_LOW);
        self.centre(a6 + TASK_Y_LOW);
    }

    /// `func_001896`: whether the task is in the middle of its cell:
    /// x (`func_0018A8`), then, if it is, y (`func_0018B6`).  Gives back
    /// d0 and Z (set when both are).
    pub fn r_001896(&mut self, d0: u32, a6: u32) -> (u32, bool) {
        let (d0, z) = self.r_0018a8(d0, a6);
        if z {
            self.r_0018b6(d0, a6)
        } else {
            (d0, z)
        }
    }

    /// `func_0018A8`: the task's x within its cell.  Gives back d0 and Z
    /// (in the middle).
    pub fn r_0018a8(&mut self, d0: u32, a6: u32) -> (u32, bool) {
        let (d0, z, _) = self.in_cell(d0, a6 + TASK_X_LOW);
        (d0, z)
    }

    /// `func_0018B6`: the task's y within its cell, as `func_0018A8`.
    pub fn r_0018b6(&mut self, d0: u32, a6: u32) -> (u32, bool) {
        let (d0, z, _) = self.in_cell(d0, a6 + TASK_Y_LOW);
        (d0, z)
    }

    /// The position (fraction at `fraction`, pixels at `pixels`) moved by
    /// the velocity at `velocity`.  Gives back d0 (the new pixels in its
    /// low word, the fraction of the velocity in its low byte before),
    /// d1 (the velocity's pixels, a word) and X.
    fn move_by(&mut self, d0: u32, d1: u32, velocity: u32, fraction: u32, pixels: u32) -> (u32, u32, bool) {
        let whole = self.read8(velocity);
        let d1 = low_word(d1, whole as i8 as i16 as u16);
        let by_fraction = self.read8(velocity + 1);
        let (new_fraction, new_pixels, x) = add_velocity(self.read8(fraction), self.read16(pixels), whole, by_fraction);
        self.write8(fraction, new_fraction);
        self.write16(pixels, new_pixels);
        (low_word(d0, new_pixels), d1, x)
    }

    /// `func_0019C0`: the object moved by the velocity's x.  Gives back
    /// d0, d1 and X.
    pub fn r_0019c0(&mut self, d0: u32, d1: u32, a4: u32) -> (u32, u32, bool) {
        self.move_by(d0, d1, VELOCITY_X, a4 + OBJ_X_FRACTION, a4 + OBJ_X)
    }

    /// `func_0019DA`: the object moved by the velocity's y.  Gives back
    /// d0, d1 and X.
    pub fn r_0019da(&mut self, d0: u32, d1: u32, a4: u32) -> (u32, u32, bool) {
        self.move_by(d0, d1, VELOCITY_Y, a4 + OBJ_Y_FRACTION, a4 + OBJ_Y)
    }

    /// `func_001A88`: the task moved by the velocity's x.  Gives back
    /// d0, d1, X, N (of the new x) and C (clear).
    pub fn r_001a88(&mut self, d0: u32, d1: u32, a6: u32) -> (u32, u32, bool, bool, bool) {
        let (d0, d1, x) = self.move_by(d0, d1, VELOCITY_X, a6 + TASK_X_FRACTION, a6 + TASK_X);
        (d0, d1, x, d0 & 0x8000 != 0, false)
    }

    /// `func_001AA2`: the task moved by the velocity's y, as
    /// `func_001A88`.
    pub fn r_001aa2(&mut self, d0: u32, d1: u32, a6: u32) -> (u32, u32, bool, bool, bool) {
        let (d0, d1, x) = self.move_by(d0, d1, VELOCITY_Y, a6 + TASK_Y_FRACTION, a6 + TASK_Y);
        (d0, d1, x, d0 & 0x8000 != 0, false)
    }

    /// The position (fraction at `fraction`, pixels at `pixels`) moved by
    /// the velocity at `velocity`, put at `to_fraction`, `to_pixels`.
    /// Gives back d1 (the velocity's pixels, a word).
    fn moved(&mut self, d1: u32, velocity: u32, fraction: u32, pixels: u32, to_fraction: u32, to_pixels: u32) -> u32 {
        let whole = self.read8(velocity);
        let d1 = low_word(d1, whole as i8 as i16 as u16);
        let by_fraction = self.read8(velocity + 1);
        let (new_fraction, new_pixels, _) = add_velocity(self.read8(fraction), self.read16(pixels), whole, by_fraction);
        self.write8(to_fraction, new_fraction);
        self.write16(to_pixels, new_pixels);
        d1
    }

    /// `func_001A4C`: `POS_X` (and its fraction), the object's x moved
    /// by the velocity's.  Gives back d1.
    pub fn r_001a4c(&mut self, d1: u32, a4: u32) -> u32 {
        self.moved(d1, VELOCITY_X, a4 + OBJ_X_FRACTION, a4 + OBJ_X, POS_X_FRACTION, POS_X)
    }

    /// `func_001A6A`: `POS_Y` (and its fraction), the object's y moved
    /// by the velocity's.  Gives back d1.
    pub fn r_001a6a(&mut self, d1: u32, a4: u32) -> u32 {
        self.moved(d1, VELOCITY_Y, a4 + OBJ_Y_FRACTION, a4 + OBJ_Y, POS_Y_FRACTION, POS_Y)
    }

    /// Copies a fraction byte and a pixels word.
    fn copy_position(&mut self, fraction: u32, pixels: u32, to_fraction: u32, to_pixels: u32) {
        let f = self.read8(fraction);
        self.write8(to_fraction, f);
        let p = self.read16(pixels);
        self.write16(to_pixels, p);
    }

    /// `func_0019F4`: the object's x (and fraction) from `POS_X`.
    pub fn r_0019f4(&mut self, a4: u32) {
        self.copy_position(POS_X_FRACTION, POS_X, a4 + OBJ_X_FRACTION, a4 + OBJ_X);
    }

    /// `func_001A02`: the object's y (and fraction) from `POS_Y`.
    pub fn r_001a02(&mut self, a4: u32) {
        self.copy_position(POS_Y_FRACTION, POS_Y, a4 + OBJ_Y_FRACTION, a4 + OBJ_Y);
    }

    /// `func_001A30`: the task's x (and fraction) from `POS_X`.
    pub fn r_001a30(&mut self, a6: u32) {
        self.copy_position(POS_X_FRACTION, POS_X, a6 + TASK_X_FRACTION, a6 + TASK_X);
    }

    /// `func_001A3E`: the task's y (and fraction) from `POS_Y`.
    pub fn r_001a3e(&mut self, a6: u32) {
        self.copy_position(POS_Y_FRACTION, POS_Y, a6 + TASK_Y_FRACTION, a6 + TASK_Y);
    }

    /// Copies a word.
    fn copy_word(&mut self, from: u32, to: u32) {
        let w = self.read16(from);
        self.write16(to, w);
    }

    /// `func_001A10`: `POS_X` = the object's x (pixels).
    pub fn r_001a10(&mut self, a4: u32) {
        self.copy_word(a4 + OBJ_X, POS_X);
    }

    /// `func_001A18`: `POS_Y` = the object's y (pixels).
    pub fn r_001a18(&mut self, a4: u32) {
        self.copy_word(a4 + OBJ_Y, POS_Y);
    }

    /// `func_001A20`: `POS_X` = the task's x (pixels).
    pub fn r_001a20(&mut self, a6: u32) {
        self.copy_word(a6 + TASK_X, POS_X);
    }

    /// `func_001A28`: `POS_Y` = the task's y (pixels).
    pub fn r_001a28(&mut self, a6: u32) {
        self.copy_word(a6 + TASK_Y, POS_Y);
    }

    /// The word at `offset` plus the one at `rate`.  Gives back d0 (the
    /// rate in its low word, the offset's new high byte in its low
    /// byte), X and N (of that byte).
    fn add_rate(&mut self, d0: u32, rate: u32, offset: u32) -> (u32, bool, bool) {
        let by = self.read16(rate);
        let (sum, x) = self.read16(offset).overflowing_add(by);
        self.write16(offset, sum);
        let high = self.read8(offset);
        (low_byte(low_word(d0, by), high), x, high & 0x80 != 0)
    }

    /// `func_001ABC`: the object's `OBJ_OFFSET` plus its `OBJ_RATE`.
    /// Gives back d0, X and N.
    pub fn r_001abc(&mut self, d0: u32, a4: u32) -> (u32, bool, bool) {
        self.add_rate(d0, a4 + OBJ_RATE, a4 + OBJ_OFFSET)
    }

    /// `func_001ACA`: the task's `TASK_OFFSET` plus its `TASK_RATE`.
    /// Gives back d0, X and N.
    pub fn r_001aca(&mut self, d0: u32, a6: u32) -> (u32, bool, bool) {
        self.add_rate(d0, a6 + TASK_RATE, a6 + TASK_OFFSET)
    }

    /// The word at `rate` plus `by`, its high byte capped at 4 (when it
    /// isn't negative).  Gives back d0 (the high byte before the cap).
    fn speed_up(&mut self, rate: u32, by: u16) -> u32 {
        let sum = self.read16(rate).wrapping_add(by);
        self.write16(rate, sum);
        let high = self.read8(rate);
        if (high as i8) >= 0 && high >= 4 {
            self.write8(rate, 4);
        }
        u32::from(high)
    }

    /// `func_001AD8`: the object's `OBJ_RATE` plus 0x50 (capped).
    pub fn r_001ad8(&mut self, a4: u32) {
        self.speed_up(a4 + OBJ_RATE, 0x50);
    }

    /// `func_001AF6`: the object's `OBJ_RATE` plus 0x20 (capped).  Gives
    /// back d0.
    pub fn r_001af6(&mut self, a4: u32) -> u32 {
        self.speed_up(a4 + OBJ_RATE, 0x20)
    }

    /// `func_001B14`: the task's `TASK_RATE` plus 0x50 (capped).  Gives
    /// back d0.
    pub fn r_001b14(&mut self, a6: u32) -> u32 {
        self.speed_up(a6 + TASK_RATE, 0x50)
    }

    /// The sprite's position (at a5) from `position` (x, y): less
    /// `VIEW`, y less `VIEW_Y_MORE` too and plus `y_by`, both plus
    /// `BORDER` (x through the carry-free long, y by the word).  Gives
    /// back d0 (the sprite's position) and X (y's carry).
    fn place(&mut self, position: u32, y_by: u16, a5: u32) -> (u32, bool) {
        let d0 = position.wrapping_sub(self.read32(VIEW));
        let y = (d0 as u16).wrapping_sub(self.read16(VIEW_Y_MORE)).wrapping_add(y_by);
        let (y, x) = y.overflowing_add(BORDER);
        let d0 = low_word(d0, y);
        self.write32(a5 + SPRITE_X, d0);
        (d0, x)
    }

    /// `func_001B32`: the sprite at a5 placed for the object at a4, y
    /// plus its offset (`OBJ_OFFSET`'s high byte, signed).  Gives back
    /// d0, d1 (the offset, a word), X and C (clear).
    pub fn r_001b32(&mut self, d1: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let position = self.read32(a4 + OBJ_X);
        let offset = self.read8(a4 + OBJ_OFFSET) as i8 as i16 as u16;
        let d1 = low_word(d1, offset);
        let (d0, x) = self.place(position, offset, a5);
        (d0, d1, x, false)
    }

    /// `func_001B7E`: the sprite at a5 placed for the task at a6.  Gives
    /// back d0, X and C (clear).
    pub fn r_001b7e(&mut self, a5: u32, a6: u32) -> (u32, bool, bool) {
        let position = self.read32(a6 + TASK_X);
        let (d0, x) = self.place(position, 0, a5);
        (d0, x, false)
    }

    /// Adds `by` to the sprite's word at `at`.  Gives back its carry.
    fn add_to_word(&mut self, at: u32, by: u16) -> bool {
        let (sum, carry) = self.read16(at).overflowing_add(by);
        self.write16(at, sum);
        carry
    }

    /// `func_001B94`: `func_001B7E`, y plus the task's `TASK_OFFSET`
    /// (its high byte, signed).  Gives back d0, X and C (the carry).
    pub fn r_001b94(&mut self, a5: u32, a6: u32) -> (u32, bool, bool) {
        let (d0, ..) = self.r_001b7e(a5, a6);
        let offset = self.read8(a6 + TASK_OFFSET) as i8 as i16 as u16;
        let d0 = low_word(d0, offset);
        let carry = self.add_to_word(a5 + SPRITE_Y, offset);
        (d0, carry, carry)
    }

    /// `func_001BA4` (`loc_001BA4`): `func_001B7E`, x plus the task's
    /// `TASK_OFFSET_X` (unsigned).  Gives back d0, X and C (the carry).
    pub fn r_001ba4(&mut self, a5: u32, a6: u32) -> (u32, bool, bool) {
        let (d0, ..) = self.r_001b7e(a5, a6);
        let offset = u16::from(self.read8(a6 + TASK_OFFSET_X));
        let d0 = low_word(d0, offset);
        let carry = self.add_to_word(a5 + SPRITE_X, offset);
        (d0, carry, carry)
    }

    /// `func_001B50`: the sprite's depth from its y's low byte less the
    /// object's offset (`OBJ_OFFSET`'s high byte), `func_001B60`.
    /// Gives back d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_001b50(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, bool, bool) {
        let y = self.read8(a5 + SPRITE_Y_LOW).wrapping_sub(self.read8(a4 + OBJ_OFFSET));
        self.r_001b60(low_byte(d0, y), d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3)
    }

    /// `func_001B5C`: the sprite's depth from its y's low byte,
    /// `func_001B60`.  Gives back d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_001b5c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32) -> (u32, bool, bool) {
        let y = self.read8(a5 + SPRITE_Y_LOW);
        self.r_001b60(low_byte(d0, y), d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3)
    }

    /// `func_001B74`: the sprite's depth from its y's low byte less the
    /// task's offset (`TASK_OFFSET`'s high byte), `func_001B60`.  Gives
    /// back d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_001b74(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, bool, bool) {
        let y = self.read8(a5 + SPRITE_Y_LOW).wrapping_sub(self.read8(a6 + TASK_OFFSET));
        self.r_001b60(low_byte(d0, y), d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3)
    }

    /// `func_001B60` (`.L001B60`): the running task's sprite moved to the
    /// depth made from `d0`'s low byte, a y: `!(y - 0x10)`, made even,
    /// bit 1 set (`func_0007D2`).  Gives back d0
    /// (the depth), X (the borrow of y - 0x10) and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_001b60(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32) -> (u32, bool, bool) {
        let (y, x) = (d0 as u8).overflowing_sub(BORDER as u8);
        let d0 = low_byte(d0, (!y & 0xFE) | 2);
        let c = self.r_0007d2(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, x);
        (d0, x, c)
    }
}
