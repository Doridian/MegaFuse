//! The routines of the stage record `D_00A1A0`: its routine
//! `func_00D630` (a task that puts a moving sprite at a random cell,
//! `func_00D770`, `func_00D7EC`), the routines by cell kind of
//! `D_00A1D0` (`func_00D820`, `func_00D816`, `func_00D74E`,
//! `func_00D75C`; each leaves in d0 the Z80's command and C clear), the
//! task `func_00D820` makes, and the tasks that run an animation to its
//! end (`func_00D61C`, `func_00D8AA`, `func_00D8F0`, made by
//! `func_00D5E0` and `func_00D8BE`).  And `func_00D652`, which nothing
//! runs, with the task it makes (`func_00D6DC`).

use crate::{Game, Regs};

/// Flags: bit 0 leaves out `func_00D630`'s task, bit 5 starts
/// `func_00D84A`'s.
const FLAGS: u32 = 0xFF9540;
/// The task being run (its index, a byte; `func_0005F8`).
const CURRENT_TASK: u32 = 0xFF80EE;
/// The frame count: bit 1 picks `func_00D6DC`'s sprite frame.
const FRAME: u32 = 0xFF80D8;
/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// A byte the cell routines set (and the made tasks clear): read by
/// `func_00CA06` for the task's sprite.
const SPRITE_KIND: u32 = 0xFFAC91;
/// A velocity (words, x then y; `a_motion.rs`).
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;
/// The cell (column, row) and the map's size.
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
const WIDTH: u32 = 0xFF9EB2;
const HEIGHT: u32 = 0xFF9EB3;

/// The tasks' routines, depths, priorities and animations.
const WAIT_ROUTINE: u32 = 0x00D770;
const WAIT_COUNT: u8 = 0x20;
const PRIORITY_42: u8 = 0x42;
const PRIORITY_C2: u8 = 0xC2;
const RISER_ROUTINE: u32 = 0x00D7EC;
const RISER_DEPTH: u8 = 0xF0;
const RISER_ANIM: u32 = 0x0DA530;
const RISER_FRAME: u8 = 0x51;
const SIGN_ROUTINE: u32 = 0x00D84A;
const SIGN_DEPTH: u8 = 0xF0;
const SIGN_COUNTING: u32 = 0x00D89A;
const SIGN_COUNT: u8 = 0x10;
const SIGN_MADE_ROUTINE: u32 = 0x00D8AA;
const SIGN_MADE_DEPTH: u8 = 0x50;
const SIGN_MADE_ANIM: u32 = 0x0DA536;
const SHOWN_ROUTINE: u32 = 0x00D8F0;
const SHOWN_DEPTH: u8 = 0x40;
const SHOWN_ANIM: u32 = 0x0DA51E;
const FLYER_ROUTINE: u32 = 0x00D6DC;
const FLYER_DEPTH: u8 = 0x10;

/// A task's fields: the count of `func_00D652`'s flyers, a side (0,
/// 1), a byte count (with the frame count after it, a word together
/// for `func_00D652`), a frame count, a rate (a word; its high byte
/// the y step), its animation.
const TASK_FLYERS: u32 = 0x12;
const TASK_SIDE: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_FRAMES: u32 = 0x17;
const TASK_RATE: u32 = 0x1A;
const TASK_ANIM: u32 = 0x20;
/// `func_00D652`'s count and frame count together (a word).
const FLYER_EVERY: u16 = 0x4B0;

/// A sprite record's frame, flags and position (words, x then y; the
/// bytes of them).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;
/// The x steps of `func_00D6DC`'s flyer by side (bytes).
const FLYER_X_STEP: u32 = 0x00D74C;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// A byte's index sign-extended (`ext.w`, then `(table,dn.w)`).
fn index(b: u8) -> u32 {
    b as i8 as i32 as u32
}

impl Game {
    /// A cell routine's end: d0 the Z80's command, C clear.
    fn m_command(regs: &mut Regs, command: u8) {
        regs.d0 = low_byte(regs.d0, command);
        regs.c = false;
        regs.unwound = false;
    }

    /// A task's count (`TASK_COUNT`) one less: whether it is 0, X and
    /// C the borrow.
    fn m_count_down(&mut self, regs: &mut Regs) -> bool {
        let (count, borrow) = self.read8(regs.a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(regs.a6 + TASK_COUNT, count);
        regs.x = borrow;
        regs.c = borrow;
        count == 0
    }

    /// `func_00D61C`'s body: the task's animation run (`func_0013D6`);
    /// at its end (C) the task ends (`func_00121A`), else the sprite is
    /// placed (`func_001B7E`, `func_001B5C`).
    fn m_animate(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let ended;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, ended) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if ended {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        } else {
            (r.d0, _, _) = self.r_001b7e(r.a5, r.a6);
            (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        }
        r.unwound = false;
    }

    /// `func_00D61C`, the routine of `func_00D5E0`'s tasks: their
    /// animation to its end.
    pub fn r_00d61c(&mut self, regs: &mut Regs) {
        self.m_animate(regs);
    }

    /// `func_00D8AA`, the routine of `func_00D84A`'s task: its
    /// animation to its end.
    pub fn r_00d8aa(&mut self, regs: &mut Regs) {
        self.m_animate(regs);
    }

    /// `func_00D8F0`, the routine of `func_00D8BE`'s task: its
    /// animation to its end.
    pub fn r_00d8f0(&mut self, regs: &mut Regs) {
        self.m_animate(regs);
    }

    /// `func_00D630`, `D_00A1A0`'s routine: unless bit 0 of `FLAGS`, a
    /// task running `func_00D770`, its count `WAIT_COUNT`.
    pub fn r_00d630(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        if self.read8(FLAGS) & 1 != 0 {
            return;
        }
        (regs.d6, regs.a6, regs.x, _) = self.r_0003e8(regs.d4, regs.d5, regs.d6, PRIORITY_42, WAIT_ROUTINE);
        self.write8(regs.a6 + TASK_COUNT, WAIT_COUNT);
        regs.c = false;
    }

    /// `func_00D652` (nothing runs it), a task's routine: each time its
    /// frame count runs out its count is one less; at 0 (both from
    /// `FLYER_EVERY` again) a flyer (`func_00D6DC`) made on the side
    /// its flyers' count says, with its count 0x7F and its rate -0x600.
    /// The flyer not made, the
    /// original returns to the address d0 was made from (pushed on the
    /// stack and left there, docs/PORT.md, "Bugs"); this returns.
    /// Reviewed by reading.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00d652(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) {
        let d0 = low_byte(d0, (d0 as u8).wrapping_add(2)).wrapping_add(2);
        let d0 = low_byte(d0, (d0 as u8).wrapping_sub(2));
        let frames = self.read8(a6 + TASK_FRAMES).wrapping_sub(1);
        self.write8(a6 + TASK_FRAMES, frames);
        if frames != 0 {
            return;
        }
        if self.read8(a6 + TASK_COUNT) == 0 {
            self.write16(a6 + TASK_COUNT, FLYER_EVERY);
            let flyers = self.read8(a6 + TASK_FLYERS).wrapping_add(1);
            self.write8(a6 + TASK_FLYERS, flyers);
            let side = flyers & 1;
            let d0 = low_byte(d0, side);
            self.write8(SPRITE_KIND, 0);
            let (_, _, a5, a6, _, failed) =
                self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, FLYER_DEPTH, PRIORITY_C2, FLYER_ROUTINE);
            if failed {
                return;
            }
            self.r_00ca06(d0, a5, a6);
            self.write16(a6 + TASK_RATE, 0xFA00);
            self.write8(a6 + TASK_COUNT, 0x80);
            self.write16(a5 + SPRITE_Y, 0xD0);
            self.write16(a5 + SPRITE_X, 0xFFA0);
            self.write8(a6 + TASK_SIDE, side);
            if side != 0 {
                self.write8(a5 + SPRITE_X, 1);
                self.write8(a5 + SPRITE_X_LOW, 0x40);
                self.write8(a5 + SPRITE_Y_LOW, 0xF0);
            }
            let count = self.read8(a6 + TASK_COUNT).wrapping_sub(1);
            self.write8(a6 + TASK_COUNT, count);
            return;
        }
        let count = self.read8(a6 + TASK_COUNT).wrapping_sub(1);
        self.write8(a6 + TASK_COUNT, count);
    }

    /// `func_00D6DC`, `func_00D652`'s flyer: for its count, moved
    /// across by its side's step (`FLYER_X_STEP`) and down by its
    /// rate's high byte (falling faster each frame, up to 6), framed
    /// 0x60 plus twice its side plus bit 1 of the frame count; then it
    /// ends (`func_00121A`).  Not run (nor is `func_00D652`): reviewed
    /// by reading.
    pub fn r_00d6dc(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        if self.m_count_down(regs) {
            (regs.d6, regs.x, regs.c) = self.r_00121a(regs.d5, regs.d6, regs.x);
            return;
        }
        let a6 = regs.a6;
        let a5 = regs.a5;
        let rate = self.read16(a6 + TASK_RATE).wrapping_add(0x40);
        self.write16(a6 + TASK_RATE, rate);
        let step = (rate >> 8) as u8;
        if (step as i8) >= 0 && step >= 6 {
            self.write8(a6 + TASK_RATE, 6);
        }
        let side = self.read8(a6 + TASK_SIDE);
        let dx = self.read8(FLYER_X_STEP.wrapping_add(index(side)));
        let x = self.read16(a5 + SPRITE_X).wrapping_add(dx as i8 as u16);
        self.write16(a5 + SPRITE_X, x);
        regs.d6 = low_byte(regs.d6, self.read8(CURRENT_TASK));
        (regs.d6, regs.a6, _) = self.r_0005f8(regs.d6);
        let a6 = regs.a6;
        let step = self.read8(a6 + TASK_RATE);
        let mut d0 = low_word(0x40, dx as i8 as u16);
        d0 = low_byte(d0, step);
        let dy = if (step as i8) < 0 {
            Some(step)
        } else if step != 0 {
            Some(step.wrapping_neg())
        } else {
            None
        };
        if let Some(dy) = dy {
            d0 = low_word(d0, dy as i8 as u16);
            let y = self.read16(a5 + SPRITE_Y).wrapping_add(d0 as u16);
            self.write16(a5 + SPRITE_Y, y);
        }
        let frame_base = self.read8(a6 + TASK_SIDE).wrapping_mul(2).wrapping_add(0x60);
        regs.d0 = low_byte(d0, frame_base);
        let frame = self.read8(FRAME);
        regs.x = frame & 1 != 0;
        let sprite_frame = ((frame >> 1) & 1) | frame_base;
        regs.d1 = low_byte(regs.d1, sprite_frame);
        self.write8(a5 + SPRITE_FRAME, sprite_frame);
        regs.c = false;
    }

    /// `func_00D74E`, a cell routine: `SPRITE_KIND` 0x64; command 0.
    pub fn r_00d74e(&mut self, regs: &mut Regs) {
        self.write8(SPRITE_KIND, 0x64);
        Self::m_command(regs, 0);
    }

    /// `func_00D75C`, a cell routine: `SPRITE_KIND` 0x68; command 0.
    pub fn r_00d75c(&mut self, regs: &mut Regs) {
        self.write8(SPRITE_KIND, 0x68);
        Self::m_command(regs, 0);
    }

    /// `func_00D75C`'s end (0xD766): command 0, C clear.
    pub fn r_00d766(&mut self, d0: u32) -> (u32, bool) {
        (low_byte(d0, 0), false)
    }

    /// `func_00D770`, `func_00D630`'s task: when its count runs out, a
    /// task running `func_00D7EC` (`RISER_ANIM`) made at a cell taken
    /// at random (column 1 to the width less 2, row 2 to the height
    /// less 3), its sprite framed `RISER_FRAME` and placed; the count
    /// is left at 0 (it waits no more).  Not made, it tries again the
    /// next frame.
    pub fn r_00d770(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        if !self.m_count_down(regs) {
            return;
        }
        self.write8(regs.a6 + TASK_COUNT, 1);
        let r = &mut *regs;
        let failed;
        (r.d5, r.d6, r.a5, r.a6, r.x, failed) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, RISER_DEPTH, PRIORITY_C2, RISER_ROUTINE);
        r.c = failed;
        if failed {
            return;
        }
        let (a5, a6) = (r.a5, r.a6);
        (r.d0, _) = self.r_00ca06(r.d0, a5, a6);
        self.write32(a6 + TASK_ANIM, RISER_ANIM);
        let width = self.read8(WIDTH).wrapping_sub(2);
        r.d0 = self.r_001bc0(low_byte(r.d0, width), r.d1);
        let column = (r.d0 as u8).wrapping_add(1);
        r.d0 = low_byte(r.d0, column);
        self.write8(CELL_X, column);
        let height = self.read8(HEIGHT).wrapping_sub(4);
        r.d0 = self.r_001bc0(low_byte(r.d0, height), r.d1);
        let row = (r.d0 as u8).wrapping_add(2);
        r.d0 = low_byte(r.d0, row);
        self.write8(CELL_Y, row);
        (r.d0, _) = self.r_001550(r.d0, a6);
        (r.d0, _, _) = self.r_001b7e(a5, a6);
        r.d0 = low_byte(r.d0, RISER_FRAME);
        self.r_0013c0(r.d0, a5);
        self.write8(SLOT_INDEX, r.d5 as u8);
        self.write32(SLOT_ADDR, a5);
        (r.d0, _, _) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, a5);
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        self.write8(r.a6 + TASK_COUNT, 0);
        r.c = false;
    }

    /// `func_00D7EC`, `func_00D770`'s task: moved up (`VELOCITY_Y`
    /// -0x180) and across at random (-2 to 2 in `VELOCITY_X`'s high
    /// byte; `func_001A88`, `func_001AA2`); off the top (N) it ends,
    /// else its animation runs and it is placed.
    pub fn r_00d7ec(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        r.d0 = self.r_001bc0(4, r.d1);
        let x = (r.d0 as u8).wrapping_sub(2);
        r.d0 = low_byte(r.d0, x);
        self.write8(VELOCITY_X, x);
        self.write16(VELOCITY_Y, 0xFE80);
        (r.d0, r.d1, _, _, _) = self.r_001a88(r.d0, r.d1, r.a6);
        let off;
        (r.d0, r.d1, r.x, off, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        if off {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, _, _) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_00D816`, a cell routine: command 0.
    pub fn r_00d816(&mut self, regs: &mut Regs) {
        Self::m_command(regs, 0);
    }

    /// `func_00D820`, a cell routine: `SPRITE_KIND` 0x7E, a task
    /// running `func_00D84A` placed (`func_00CA06`); command 0x74.
    pub fn r_00d820(&mut self, regs: &mut Regs) {
        self.write8(SPRITE_KIND, 0x7E);
        let r = &mut *regs;
        (r.d5, r.d6, r.a5, r.a6, r.x, _) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, SIGN_DEPTH, PRIORITY_42, SIGN_ROUTINE);
        (r.d0, _) = self.r_00ca06(r.d0, r.a5, r.a6);
        Self::m_command(regs, 0x74);
    }

    /// `func_00D84A`, `func_00D820`'s task: placed (`func_001B7E`)
    /// until bit 5 of `FLAGS`; then it counts `SIGN_COUNT` frames
    /// (`func_00D89A`), with a task running `func_00D8AA`
    /// (`SIGN_MADE_ANIM`; `SPRITE_KIND` 0) and the Z80 0x50.
    pub fn r_00d84a(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        if self.read8(FLAGS) & (1 << 5) == 0 {
            (regs.d0, regs.x, regs.c) = self.r_001b7e(regs.a5, regs.a6);
            return;
        }
        let r = &mut *regs;
        self.r_00053e(r.d6, SIGN_COUNTING);
        self.write8(r.a6 + TASK_COUNT, SIGN_COUNT);
        self.r_00153a(r.a6);
        self.write8(SPRITE_KIND, 0);
        (r.d5, r.d6, r.a5, r.a6, r.x, _) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, SIGN_MADE_DEPTH, PRIORITY_42, SIGN_MADE_ROUTINE);
        (r.d0, _) = self.r_00ca06(r.d0, r.a5, r.a6);
        self.write32(r.a6 + TASK_ANIM, SIGN_MADE_ANIM);
        r.d0 = low_byte(r.d0, 0x50);
        r.c = false;
        self.r_0034b6(r.d0);
    }

    /// `func_00D89A`: placed for its count, then on to `func_00C86E`'s
    /// end (0xC86E).
    pub fn r_00d89a(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        if self.m_count_down(r) {
            (r.d0, r.d6, r.a0, r.x, r.c) = self.r_00c86e(r.d5, r.d6, r.a6);
        } else {
            (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        }
    }

    /// `func_00D84A`'s end (0xD8A6): the sprite placed (`func_001B7E`).
    pub fn r_00d8a6(&mut self, a5: u32, a6: u32) -> (u32, bool, bool) {
        self.r_001b7e(a5, a6)
    }

    /// `func_00D8BE`: a task running `func_00D8F0` (`SHOWN_ANIM`,
    /// sprite flags 4) placed and made the routine's sprite, and run
    /// once.  Gives back d0, d1, d2, d5, d6, a0, a5 and a6.
    #[allow(clippy::type_complexity)]
    pub fn r_00d8be(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        let (d5, d6, a5, a6, _, failed) =
            self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, SHOWN_DEPTH, PRIORITY_C2, SHOWN_ROUTINE);
        if failed {
            return (d0, d1, d2, d5, d6, a0, a5, a6);
        }
        let (d0, x) = self.r_001550(d0, a6);
        self.write8(a5 + SPRITE_FLAGS, 4);
        self.write32(a6 + TASK_ANIM, SHOWN_ANIM);
        self.write8(SLOT_INDEX, d5 as u8);
        self.write32(SLOT_ADDR, a5);
        let mut regs = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, n: false, z: false, v: false, c: false, unwound: false };
        self.r_00d8f0(&mut regs);
        (regs.d0, regs.d1, regs.d2, regs.d5, regs.d6, regs.a0, regs.a5, regs.a6)
    }
}
