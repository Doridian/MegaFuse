//! Angles (a byte, 0x100 to the turn): a velocity from an angle and a
//! speed by a quarter of a sine wave (`func_00414A`), the angle from one
//! point to another by an arctangent's table (`func_0041D0`), and small
//! helpers about the same two points.

use crate::Game;

/// The angle `func_00414A` works from; its bit 7 makes the result
/// negative.
const ANGLE: u32 = 0xFF80C5;
/// The speed (a word) `func_00414A` scales by the sine.
const SPEED: u32 = 0xFF9F00;
/// The two words `func_00410C` makes: by `ANGLE`, and by `ANGLE` + 0xC0.
const VELOCITY_A: u32 = 0xFF9EC4;
const VELOCITY_B: u32 = 0xFF9EC6;
/// A quarter of a sine wave: 0x40 bytes, 0 to 0xFF (D_004190).
const SINE: u32 = 0x004190;

/// Two points (words): `func_0041D0` gives the angle of A - B,
/// `func_004348` looks at B - A.
const A_X: u32 = 0xFF9EE6;
const A_Y: u32 = 0xFF9EE8;
const B_X: u32 = 0xFF9EFA;
const B_Y: u32 = 0xFF9EFE;
/// What `func_0041D0` keeps of A - B: the high bytes of x and y (their
/// signs), and the low bytes' sizes.
const DX_HIGH: u32 = 0xFF9ED3;
const DY_HIGH: u32 = 0xFF9ED2;
const DX_SIZE: u32 = 0xFF9ECF;
const DY_SIZE: u32 = 0xFF9ECE;
/// An arctangent: the angle (0 to 0x1F) whose tangent is i / 0x40, for
/// i from 0 to 0x3F (D_0042AC).
const ARCTAN: u32 = 0x0042AC;

/// An object's word `func_004424` adds to.
const OBJ_28: u32 = 0x28;

/// Two bytes `func_00442C` and the like compare (their low bits).
const INDEX_A42C: u32 = 0xFFA42C;
const BYTE_80D8: u32 = 0xFF80D8;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_00410C`: both of `func_004132`'s words, `VELOCITY_A` by
    /// the angle `d0` and `VELOCITY_B` by it + 0xC0 (left in `ANGLE`).
    /// d1 and d2 are kept.  Gives back d0 (the second sine), X.
    pub fn r_00410c(&mut self, d0: u32, d1: u32, d2: u32, x: bool) -> (u32, u32, u32, bool) {
        let d0 = self.r_004132(d0, d1, x);
        let (angle, x) = self.read8(ANGLE).overflowing_add(0xC0);
        self.write8(ANGLE, angle);
        let (d0, velocity, x) = self.r_00414a(low_byte(d0, angle), d1, x);
        self.write16(VELOCITY_B, velocity as u16);
        (d0, d1, d2, x)
    }

    /// `func_004132`: `VELOCITY_A` by the angle `d0` (left in
    /// `ANGLE`), `func_00414A`.  d1 and d2 are kept.  Gives back d0.
    pub fn r_004132(&mut self, d0: u32, d1: u32, x: bool) -> u32 {
        self.write8(ANGLE, d0 as u8);
        let (d0, velocity, _) = self.r_00414a(d0, d1, x);
        self.write16(VELOCITY_A, velocity as u16);
        d0
    }

    /// `func_00414A`: `SPEED` x the sine of the angle `d0` (its low
    /// byte), to d1: at 0x40 and 0xC0 `SPEED` itself, else by the table
    /// `SINE` (`func_0011CA`, / 0x100); negated (its low word) if
    /// `ANGLE`'s bit 7 is set.  Gives back d0 (the sine), d1 and X.
    pub fn r_00414a(&mut self, d0: u32, d1: u32, x: bool) -> (u32, u32, bool) {
        let angle = d0 as u8;
        let (d0, mut d1, mut x) = if angle == 0x40 || angle == 0xC0 {
            (d0, low_word(d1, self.read16(SPEED)), x)
        } else {
            // the second quarter mirrors the first
            let quarter = if angle & 0x40 != 0 { !angle } else { angle } & 0x3F;
            let sine = self.read8(SINE + u32::from(quarter));
            let d0 = low_word(d0, u16::from(sine));
            let speed = u32::from(self.read16(SPEED));
            let (d1, x) = self.r_0011ca(d0, speed);
            (d0, d1, x)
        };
        if self.read8(ANGLE) & 0x80 != 0 {
            let w = d1 as u16;
            d1 = low_word(d1, w.wrapping_neg());
            x = w != 0;
        }
        (d0, d1, x)
    }

    /// `func_0041D0`: the angle (to d0's low byte) of point A - point
    /// B: 0x40 or 0xC0 straight along x, 0 or 0x80 along y, 0x20 at 45
    /// degrees, else by the table `ARCTAN` of the smaller of the low
    /// bytes over the larger (`func_0042EC`), then turned by the
    /// quarter the signs say.  Keeps the signs in `DX_HIGH`, `DY_HIGH`
    /// and the sizes in `DX_SIZE`, `DY_SIZE`.  d0's high word is kept
    /// but where the result is a `moveq` (0x40, 0xC0, 0x20).
    pub fn r_0041d0(&mut self, d0: u32, _d1: u32) -> u32 {
        let high = d0 & 0xFFFF_0000;
        let (dx_high, dx_size) = self.d_size(A_X, B_X);
        self.write8(DX_HIGH, dx_high);
        self.write8(DX_SIZE, dx_size);
        let (dy_high, dy_size) = self.d_size(A_Y, B_Y);
        self.write8(DY_HIGH, dy_high);
        self.write8(DY_SIZE, dy_size);
        let x_negative = self.read8(DX_HIGH) & 0x80 != 0;
        let y_negative = self.read8(DY_HIGH) & 0x80 != 0;
        let dy = self.read8(DY_SIZE);
        if dy == 0 {
            return if x_negative { 0xC0 } else { 0x40 };
        }
        let dx = self.read8(DX_SIZE);
        if dx == 0 {
            return high | if y_negative { 0 } else { 0x80 };
        }
        let mut high = high;
        let mut angle = if dx == dy {
            high = 0;
            0x20
        } else if dx > dy {
            let ratio = self.r_0042ec(u32::from(dx), u32::from(dy)) as u8;
            self.read8(ARCTAN + u32::from(ratio >> 2)).wrapping_neg() & 0x3F
        } else {
            let ratio = self.r_0042ec(u32::from(dy), u32::from(dx)) as u8;
            self.read8(ARCTAN + u32::from(ratio >> 2))
        };
        if !y_negative {
            angle = (angle.wrapping_neg() & 0x3F) | 0x40;
        }
        if x_negative {
            angle = angle.wrapping_neg();
        }
        high | u32::from(angle)
    }

    /// The word at `a` - the word at `b`: its high byte, and its low
    /// byte negated if the high one is negative.
    fn d_size(&mut self, a: u32, b: u32) -> (u8, u8) {
        let d = self.read16(a).wrapping_sub(self.read16(b));
        let high = (d >> 8) as u8;
        let low = d as u8;
        (high, if high & 0x80 != 0 { low.wrapping_neg() } else { low })
    }

    /// `func_0042EC`: (d1's low byte x 0x100) / d0's low byte (never 0
    /// where it is called), to d0's low word.
    pub fn r_0042ec(&mut self, d0: u32, d1: u32) -> u32 {
        let dividend = u32::from((d1 as u16) << 8);
        let divisor = d0 & 0xFF;
        low_word(d0, (dividend / divisor) as u16)
    }

    /// `func_0042FE`: d1's low byte a step of `d0` toward d2's: if they
    /// are at least `d0` / 2 (or 1) apart, d1 + `d0` (d1 - d2 negative)
    /// or d1 - `d0`, else d1 itself (C set), to d0's low byte.  d1 and
    /// d2 are kept.  Gives back d0 and X.
    pub fn r_0042fe(&mut self, d0: u32, d1: u32, d2: u32) -> (u32, bool) {
        let step = d0 as u8;
        let near = (step >> 1) | 1;
        let (diff, borrow) = (d1 as u8).overflowing_sub(d2 as u8);
        let negative = diff & 0x80 != 0;
        // the neg of a negative difference sets X
        let (size, x) = if negative { (diff.wrapping_neg(), true) } else { (diff, borrow) };
        if size == 0 || size < near {
            return (low_byte(d0, d1 as u8), x);
        }
        let step = if negative { step } else { step.wrapping_neg() };
        let (moved, x) = step.overflowing_add(d1 as u8);
        (low_byte(d0, moved), x)
    }

    /// `func_004348`: whether point B is within `d0` (its low byte) of
    /// point A both ways.  If so, d0's low byte 0 and C set; if not, the
    /// direction it is off by (x: 3 if B - A >= 0, else 1; then y: 0 or
    /// 2) to d0's low byte, C clear.  d0's low word keeps the last
    /// difference's high byte.  Gives back d0, X and C.
    pub fn r_004348(&mut self, d0: u32) -> (u32, bool, bool) {
        let near = d0 as u8;
        let high = d0 & 0xFFFF_0000;
        let mut last = 0;
        let mut x = false;
        for (a, b, positive, negative) in [(A_X, B_X, 3u8, 1u8), (A_Y, B_Y, 0, 2)] {
            let (diff, borrow) = self.read16(b).overflowing_sub(self.read16(a));
            let (size, direction) = if diff & 0x8000 != 0 {
                x = true;
                (diff.wrapping_neg(), negative)
            } else {
                x = borrow;
                (diff, positive)
            };
            last = size;
            if size >= 0x100 || size as u8 >= near {
                return (high | u32::from(size & 0xFF00) | u32::from(direction), x, false);
            }
        }
        (high | u32::from(last & 0xFF00), x, true)
    }

    /// `func_004424`: the byte `d0` (sign-extended to d0's low word)
    /// added to the object's word `OBJ_28`.  Gives back d0, X and N.
    pub fn r_004424(&mut self, d0: u32, obj: u32) -> (u32, bool, bool) {
        let d0 = low_word(d0, d0 as u8 as i8 as u16);
        let old = self.read16(obj + OBJ_28);
        let (new, carry) = old.overflowing_add(d0 as u16);
        self.write16(obj + OBJ_28, new);
        (d0, carry, new & 0x8000 != 0)
    }

    /// `func_00442C`: `INDEX_A42C` against `BYTE_80D8`, bit 0 of each
    /// (cmp); d0 and d1 are kept.  Gives back Z and C.
    pub fn r_00442c(&mut self) -> (bool, bool) {
        let a = self.read8(INDEX_A42C) & 1;
        let b = self.read8(BYTE_80D8) & 1;
        (a == b, a < b)
    }

    /// `func_004448`: as `func_00442C` with bits 0-1; no caller reads
    /// its flags, and it changes nothing else.
    /// No script calls it: reviewed by reading.
    pub fn r_004448(&mut self) {}

    /// `func_004464`: as `func_00442C` with bits 0-2; no caller reads
    /// its flags, and it changes nothing else.
    /// No script calls it: reviewed by reading.
    pub fn r_004464(&mut self) {}
}
