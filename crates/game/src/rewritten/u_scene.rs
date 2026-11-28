//! Two of the ending's tasks: one that follows the scripted task
//! (`func_015EB4`: the task `SCRIPTED` holds) with its frame by the
//! script's (`func_01613C`), and one that each frame scrolls the planes
//! and moves by the script's second speed and angle (`func_0161C0`).

use crate::{Game, Regs};

/// The scripted task's index (`func_015EB4`'s, made at the ending's
/// start).
const SCRIPTED: u32 = 0xFFAD81;
/// The scripted task's angle (a word; its high byte) and frame (the
/// high nibble), `u_script.rs`.
const ANGLE_A: u32 = 0xFFAD70;
const FRAME_A: u32 = 0xFFAD76;
/// The script's second speed (a word, to `SPEED`; 0 and the scripted
/// task's speed 0 when it goes below 0) and the byte added to it each
/// frame; its angle (a word, the high byte to `func_00410C`), the word
/// added to it each frame and the byte added to that.
const SPEED_A: u32 = 0xFFAD6E;
const SPEED_B: u32 = 0xFFAD66;
const SPEED_B_STEP: u32 = 0xFFAD6C;
const ANGLE_B: u32 = 0xFFAD68;
const TURN_B: u32 = 0xFFAD6A;
const TURN_B_STEP: u32 = 0xFFAD6D;
/// `func_0161C0`'s velocity (words, the x then the y: `func_00410C`'s).
const VELOCITY_X_KEPT: u32 = 0xFFAD62;
const VELOCITY_Y_KEPT: u32 = 0xFFAD64;
/// Words added to the first words of the scroll tables: `OFFSET_X`
/// (with `SCROLL_ADD`, negated, to the H-scroll table's first word) and
/// `OFFSET_Y` (to the V-scroll table's).  `SCROLL_ADD` grows by 2 each
/// frame in some of the ending's tasks (`u_makers.rs`).
const OFFSET_X: u32 = 0xFFFF_9EB4;
const OFFSET_X_FRACTION: u32 = 0xFFFF_9EB6;
const OFFSET_X_LOW: u32 = 0xFFFF_9EB5;
const OFFSET_Y: u32 = 0xFFFF_9EB8;
const SCROLL_ADD: u32 = 0xFFFF_ADC4;
const H_SCROLL_FIRST: u32 = 0xFFFF_D002;
const V_SCROLL_FIRST: u32 = 0xFFFF_D402;

/// The task running (its index), the sprite record of the routine being
/// run (index, address).
const CURRENT_TASK: u32 = 0xFF80EE;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// A word kept through `func_0013D6` (`func_01613C`).
const KEPT: u32 = 0xFF80C4;
/// The speed `func_00410C` makes the velocity for, the velocity (words,
/// x then y).
const SPEED: u32 = 0xFF9F00;
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;
const VELOCITY_Y_LOW: u32 = 0xFF9EC7;

/// A task's animation (a long), and a byte `func_01613C` reads.
const TASK_ANIM: u32 = 0x20;
const TASK_16: u32 = 0x16;
/// The animation `func_01613C`'s task gets when its byte 0x16 is 0 and
/// the scripted task's angle is 0x80 or more.
const ANIM_TURNED: u32 = 0x0D8FE6;
/// The y velocity by the high nibble of the scripted task's frame
/// (D_0161B0, 16 bytes).
const RISE_BY_FRAME: u32 = 0x0161B0;
/// The factor (a word) `func_0161C0` multiplies its x velocity by.
const SCROLL_FACTOR: u16 = 0x48;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_01613C`, a task's routine: its animation a frame on
    /// (`func_0013D6`; the animation's high word kept through it), as
    /// `ANIM_TURNED` when its byte 0x16 is 0 and the scripted task's
    /// angle is 0x80 or more; it ends with its animation
    /// (`func_00121A`).  Else it is put where the scripted task is
    /// (`func_00153A`, `func_00152A`), rises by its frame
    /// (`RISE_BY_FRAME`, `func_001AA2`) and its sprite is placed
    /// (`func_015F46`, with a6 the sprite record of the routine run
    /// and d6 its index).
    pub fn r_01613c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let kept = self.read16(r.a6 + TASK_ANIM);
        self.write16(KEPT, kept);
        let byte = self.read8(r.a6 + TASK_16);
        r.d0 = low_byte(r.d0, byte);
        if byte == 0 {
            let angle = self.read8(ANGLE_A);
            r.d0 = low_byte(r.d0, angle);
            if angle >= 0x80 {
                self.write32(r.a6 + TASK_ANIM, ANIM_TURNED);
            }
        }
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        let kept = self.read16(KEPT);
        self.write16(r.a6 + TASK_ANIM, kept);
        let scripted = self.read8(SCRIPTED);
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, scripted));
        self.r_00153a(r.a6);
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
        self.r_00152a(r.a6);
        let nibble = self.read8(FRAME_A) >> 4;
        r.d0 = low_word(r.d0, u16::from(nibble));
        let rise = self.read8(RISE_BY_FRAME + u32::from(nibble));
        self.write8(VELOCITY_Y, rise);
        self.write8(VELOCITY_Y_LOW, 0);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        r.d6 = low_byte(r.d6, self.read8(SLOT_INDEX));
        r.a6 = self.read32(SLOT_ADDR);
        r.c = self.r_015f46(r.a5, r.a6);
    }

    /// `func_0161C0`, a task's routine: the first words of the scroll
    /// tables (`OFFSET_X` + `SCROLL_ADD` negated, `OFFSET_Y`) and
    /// `func_008664`; the second speed and angle a frame on (the speed
    /// to `SPEED`, 0 below 0, when the scripted task's speed is 0 too)
    /// and the velocity by them (`func_00410C`), kept in
    /// `VELOCITY_X_KEPT`, `VELOCITY_Y_KEPT`; `OFFSET_X`'s low byte (and
    /// the fraction byte after the word) plus the x velocity, negated,
    /// times 0x48 / 0x100.  Gives back X (the last carry) and C (clear).
    pub fn r_0161c0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let sum = self.read16(OFFSET_X).wrapping_add(self.read16(SCROLL_ADD));
        let scroll = sum.wrapping_neg();
        r.d0 = low_word(r.d0, scroll);
        r.x = sum != 0;
        self.write16(H_SCROLL_FIRST, scroll);
        let offset_y = self.read16(OFFSET_Y);
        r.d0 = low_word(r.d0, offset_y);
        self.write16(V_SCROLL_FIRST, offset_y);
        r.c = false;
        self.r_008664(r);
        let step = self.read8(TURN_B_STEP) as i8 as u16;
        let turn = step.wrapping_add(self.read16(TURN_B));
        self.write16(TURN_B, turn);
        let angle = self.read16(ANGLE_B);
        self.write16(ANGLE_B, angle.wrapping_add(turn));
        let step = self.read8(SPEED_B_STEP) as i8 as u16;
        let old = self.read16(SPEED_B);
        let (speed, carry) = step.overflowing_add(old);
        r.x = carry;
        r.d0 = low_word(r.d0, speed);
        self.write16(SPEED_B, speed);
        self.write16(SPEED, speed);
        if (speed as i16) < 0 {
            self.write16(SPEED, 0);
            self.write16(SPEED_A, 0);
        }
        let angle = self.read8(ANGLE_B);
        (r.d0, r.d1, r.d2, r.x) = self.r_00410c(low_byte(r.d0, angle), r.d1, r.d2, r.x);
        let vx = self.read16(VELOCITY_X);
        self.write16(VELOCITY_X_KEPT, vx);
        r.d0 = low_word(r.d0, vx.wrapping_neg());
        r.d2 = low_word(r.d2, SCROLL_FACTOR);
        (r.d1, _) = self.r_0011ca(r.d0, r.d2);
        let whole = (r.d1 as u16 >> 8) as u8;
        r.d0 = low_word(r.d0, r.d1 as u16 >> 8);
        let fraction = self.read8(OFFSET_X_FRACTION);
        let (fraction, carry) = fraction.overflowing_add(r.d1 as u8);
        self.write8(OFFSET_X_FRACTION, fraction);
        let low = self.read8(OFFSET_X_LOW);
        let sum = u16::from(low) + u16::from(whole) + u16::from(carry);
        r.d1 = low_byte(r.d1, sum as u8);
        r.x = sum > 0xFF;
        self.write8(OFFSET_X_LOW, sum as u8);
        let vy = self.read16(VELOCITY_Y);
        self.write16(VELOCITY_Y_KEPT, vy);
        r.c = false;
    }
}
