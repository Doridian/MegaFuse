//! A task that sets the VDP up (`func_0082FA`), waits two frames,
//! turns the picture on, and then steps a palette through its table
//! of colours (`func_008286`) each frame until a pad's start bit is
//! seen or `LEFT` runs out; then the picture off, the palette faded
//! (`func_009490`, `func_0094B2`) and the task goes on to
//! `func_009248` (0x821C-0x8285).

use crate::{Game, Regs};

/// The first pad's kind (a word: less than 2 is a pad whose buttons
/// are read), and the first two pads' third button byte (bit 7:
/// start).
const PAD_KIND: u32 = 0xFFFF_800A;
const PAD_1_START: u32 = 0xFFFF_800D;
const PAD_2_START: u32 = 0xFFFF_8017;
/// Frames left to wait (a word, set to 0x12C by `func_0082FA`).
const LEFT: u32 = 0xFFFF_805C;

/// Where the task goes on after each yield.
const AFTER_SET_UP: u32 = 0x822A;
const AFTER_WAIT: u32 = 0x822E;
const AFTER_SHOWN: u32 = 0x8238;
const AFTER_OFF: u32 = 0x8268;
const AFTER_FADE: u32 = 0x8278;
/// The task's next routine at the end.
const NEXT: u32 = 0x9248;

impl Game {
    /// The yield (`func_000522`) to `resume`; if it doesn't return
    /// from the routine (it always does), `then` runs.
    fn h_yield(&mut self, r: &mut Regs, resume: u32, then: fn(&mut Game, &mut Regs)) {
        let unwound;
        (r.x, r.c, unwound) = self.r_000522(r.d6, resume);
        if !unwound {
            then(self, r);
        }
        r.unwound = false;
    }

    /// `func_00821C`: `func_0103D4`, the VDP set up (`func_0082FA`,
    /// which waits a frame), and a yield.
    pub fn r_00821c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.x = self.r_0103d4();
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a4, r.a5, r.a6) =
            self.r_0082fa(r.d4);
        self.h_yield(r, AFTER_SET_UP, Game::r_00822a);
    }

    /// `func_00822A`: a frame more.
    pub fn r_00822a(&mut self, regs: &mut Regs) {
        self.h_yield(regs, AFTER_WAIT, Game::r_00822e);
    }

    /// `func_00822E`: the picture on (`func_008552`), and a yield.
    pub fn r_00822e(&mut self, regs: &mut Regs) {
        self.r_008552();
        self.h_yield(regs, AFTER_SHOWN, Game::r_008238);
    }

    /// `func_008238`, each frame: the palette's step (`func_008286`);
    /// on when a pad is read (its kind under 2) and neither pad's
    /// start bit is set, or `LEFT` counts down to 0, the picture off
    /// (`func_00855A`) and a yield; else it returns (X and C the
    /// count's borrow).
    pub fn r_008238(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.d0, r.a0, r.a1) = self.r_008286(r.d0, r.a0, r.a1);
        let kind = self.read16(PAD_KIND);
        r.d0 = (r.d0 & 0xFFFF_0000) | u32::from(kind);
        let pressed = kind < 2
            && (self.read8(PAD_1_START) & 0x80 != 0 || self.read8(PAD_2_START) & 0x80 != 0);
        if !pressed {
            let left = self.read16(LEFT);
            let after = left.wrapping_sub(1);
            self.write16(LEFT, after);
            if after != 0 {
                r.c = left == 0;
                r.x = r.c;
                r.unwound = false;
                return;
            }
        }
        self.r_00855a();
        self.h_yield(r, AFTER_OFF, Game::r_008268);
    }

    /// `func_008268`: the palette faded (`func_009490`, `func_0094B2`)
    /// and a yield.
    pub fn r_008268(&mut self, regs: &mut Regs) {
        self.r_009490();
        regs.c = self.r_0094b2();
        self.h_yield(regs, AFTER_FADE, Game::r_008278);
    }

    /// `func_008278`: the task's next routine `func_009248`.
    pub fn r_008278(&mut self, regs: &mut Regs) {
        (regs.x, regs.c) = self.r_00053e(regs.d6, NEXT);
        regs.unwound = false;
    }
}
