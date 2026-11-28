//! A slide of the screen in a direction (`func_011AA4`): a position
//! moved on by a step, and a task that scrolls plane by plane toward it
//! two pixels a frame (eight while a button is held), then sets the
//! scroll to the position (`func_011AD8`).

use crate::{Game, Regs};

/// The direction (0 up, 1 right, 2 down, 3 left; a byte).  Set to 2
/// when the slide is over.
const DIRECTION: u32 = 0xFFAABD;
const HORIZONTAL: u8 = 1 << 0;
/// The steps a slide moves the position by (words): across, down.
const STEP_X: u32 = 0xFF9EEE;
const STEP_Y: u32 = 0xFF9EF0;
/// The position (words, x then y).
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
/// The first words of the horizontal and vertical scroll tables
/// (`h_scroll.rs`): the slide moves them, `func_008664` makes the
/// tables' first words of them.
const H_VALUE: u32 = 0xFFD002;
const V_VALUE: u32 = 0xFFD402;
/// Counted up when a slide is over.
const SLIDES: u32 = 0xFFAD3C;
/// While not 0, a slide goes fast.
const FAST_ALWAYS: u32 = 0xFFAD43;
/// The fast steps' offset into the tables (4, else 0).
const FAST: u32 = 0xFFAC8B;
const FAST_OFFSET: u8 = 4;
/// The frames a fast slide takes off its count, one more than a frame.
const FAST_FRAMES: u8 = 3;
/// The steps by direction (and + 4 fast; signed bytes): across (which
/// the slide subtracts) and down.
const STEPS_X: u32 = 0x011B52;
const STEPS_Y: u32 = 0x011B5A;

/// The task's frames to go (a byte) and its direction.
const TASK_COUNT: u32 = 0x16;
const TASK_DIRECTION: u32 = 0x15;
/// The task's routine while it slides.
const SLIDING: u32 = 0x011AD8;

/// The buttons that make a slide fast: bits 4-7 of the pads' bytes
/// `func_0102B6` gathers in d0's high byte.
const FAST_BUTTONS: u8 = 0xF0;

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_011AA4`: a slide in direction d0 (its low byte): kept in
    /// `DIRECTION` and the task (a6), the position moved by its step
    /// (across when the direction is odd, else down), and the task's
    /// routine `func_011AD8` from now on.
    pub fn r_011aa4(&mut self, d0: u32, d6: u32, a6: u32) {
        let direction = d0 as u8;
        self.write8(DIRECTION, direction);
        self.write8(a6 + TASK_DIRECTION, direction);
        if self.read8(DIRECTION) & HORIZONTAL != 0 {
            let x = self.read16(POSITION_X).wrapping_add(self.read16(STEP_X));
            self.write16(POSITION_X, x);
        } else {
            let y = self.read16(POSITION_Y).wrapping_add(self.read16(STEP_Y));
            self.write16(POSITION_Y, y);
        }
        self.r_00053e(d6, SLIDING);
    }

    /// `func_011AD8`, a task's routine: while its count lasts (taken
    /// one a frame, three more when fast: `FAST_ALWAYS`, or a button
    /// held), the scroll values moved by the direction's step and the
    /// tables' first words made (`func_008664`).  When it runs out,
    /// the scroll is the position (x negated), the direction 2,
    /// `SLIDES` counted and the task's kept routine its routine again
    /// (`loc_000558`).  Gives back d0 and d1 (by the step; d0 as
    /// `func_008664` leaves it), X and C.
    pub fn r_011ad8(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let a6 = regs.a6;
        let count = self.read8(a6 + TASK_COUNT).wrapping_sub(1);
        self.write8(a6 + TASK_COUNT, count);
        if (count as i8) < 0 {
            let x = self.read16(POSITION_X);
            regs.d0 = low_word(regs.d0, x.wrapping_neg());
            self.write16(H_VALUE, x.wrapping_neg());
            let y = self.read16(POSITION_Y);
            self.write16(V_VALUE, y);
            regs.x = x != 0;
            regs.c = regs.x;
            self.r_008664(regs);
            self.write8(DIRECTION, 2);
            let (slides, carry) = self.read8(SLIDES).overflowing_add(1);
            self.write8(SLIDES, slides);
            regs.x = carry;
            regs.c = self.r_000558(a6);
            return;
        }
        self.write8(FAST, 0);
        let fast = if self.read8(FAST_ALWAYS) != 0 {
            true
        } else {
            regs.d0 = self.r_0102b6(regs.d0);
            let buttons = (regs.d0 >> 8) as u8 & FAST_BUTTONS;
            regs.d0 = low_word(regs.d0, u16::from(buttons));
            buttons != 0
        };
        if fast {
            self.write8(FAST, FAST_OFFSET);
            let count = self.read8(a6 + TASK_COUNT);
            self.write8(a6 + TASK_COUNT, count.wrapping_sub(FAST_FRAMES));
        }
        let index = (self.read8(a6 + TASK_DIRECTION) | self.read8(FAST)) as i8;
        regs.d0 = low_word(regs.d0, index as u16);
        let offset = i32::from(index) as u32;
        let step_x = self.read8(STEPS_X.wrapping_add(offset)) as i8 as u16;
        regs.d1 = low_word(regs.d1, step_x.wrapping_neg());
        let h = self.read16(H_VALUE).wrapping_add(step_x.wrapping_neg());
        self.write16(H_VALUE, h);
        let step_y = self.read8(STEPS_Y.wrapping_add(offset)) as i8 as u16;
        regs.d1 = low_word(regs.d1, step_y);
        let (v, carry) = self.read16(V_VALUE).overflowing_add(step_y);
        self.write16(V_VALUE, v);
        regs.x = carry;
        regs.c = carry;
        self.r_008664(regs);
    }
}
