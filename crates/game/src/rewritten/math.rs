//! Arithmetic: a velocity toward a point.

use crate::Game;

/// `func_00112C`'s points (words): from (`FROM_X`, `FROM_Y`) to
/// (`TO_X`, `TO_Y`), at `SPEED` (a fraction, 0x100 one).
const TO_X: u32 = 0xFF9EE6;
const TO_Y: u32 = 0xFF9EE8;
const FROM_X: u32 = 0xFF9EFA;
const FROM_Y: u32 = 0xFF9EFE;
const SPEED: u32 = 0xFF9F00;
/// What it works out: the signs of the differences (their high bytes),
/// x's share of the distance (of 0x100), the velocity (words).
const SIGN_Y: u32 = 0xFF9ED2;
const SIGN_X: u32 = 0xFF9ED3;
const SHARE_X: u32 = 0xFF9ED4;
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;

impl Game {
    /// `divu.w`'s quotient (the low word it leaves: the dividend's when
    /// it overflows); 0 for a divisor of 0, which the game checks for.
    fn share(&self, part: u32, total: u16) -> u16 {
        if total == 0 {
            0
        } else {
            self.divu(part, total).0 as u16
        }
    }

    /// `func_00112C`: the velocity from `FROM` to `TO` at `SPEED`, each
    /// axis's share of the distance (|dx| + |dy|, by 16-bit words) times
    /// the speed.  y's share is worked out from the low byte of |dy|
    /// only.
    pub fn r_00112c(&mut self) {
        let dx = self.read16(TO_X).wrapping_sub(self.read16(FROM_X));
        let sign_x = (dx >> 8) as u8;
        self.write8(SIGN_X, sign_x);
        let abs_x = if sign_x & 0x80 != 0 { dx.wrapping_neg() } else { dx };
        let dy = self.read16(TO_Y).wrapping_sub(self.read16(FROM_Y));
        let sign_y = (dy >> 8) as u8;
        self.write8(SIGN_Y, sign_y);
        let abs_y = if sign_y & 0x80 != 0 { dy.wrapping_neg() } else { dy };
        let total = abs_y.wrapping_add(abs_x);
        let share_x = self.share(u32::from(abs_x) << 8, total);
        self.write16(SHARE_X, share_x);
        let share_y = self.share(u32::from(abs_y as u8) << 8, total);
        let speed = self.read16(SPEED);
        let (vy, _) = self.r_0011ca(u32::from(share_y), u32::from(speed));
        let vy = vy as u16;
        let vy = if self.read8(SIGN_Y) & 0x80 != 0 { vy.wrapping_neg() } else { vy };
        self.write16(VELOCITY_Y, vy);
        let speed = self.read16(SPEED);
        let share_x = self.read16(SHARE_X);
        let (vx, _) = self.r_0011ca(u32::from(share_x), u32::from(speed));
        let vx = vx as u16;
        let vx = if self.read8(SIGN_X) & 0x80 != 0 { vx.wrapping_neg() } else { vx };
        self.write16(VELOCITY_X, vx);
    }

    /// `func_0011CA`: d1 = `d0` x `d2` (their low words) / 0x100.  Gives
    /// back X, the last bit shifted out.
    pub fn r_0011ca(&mut self, d0: u32, d2: u32) -> (u32, bool) {
        let product = u32::from(d0 as u16) * u32::from(d2 as u16);
        (product >> 8, product & 0x80 != 0)
    }
}
