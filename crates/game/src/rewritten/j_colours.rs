//! The colours as the game keeps them (`COLOURS`, sent on by
//! `func_0094B2`) changed over frames: the fades, tasks that step the
//! colours from a saved copy (`SAVED`) through a table of levels
//! (`func_00A506` .. `func_00A744`, 0xA506-0xA75B), the levels applied
//! (`func_00A75C` .. `func_00A7C8`), a step toward other colours
//! (`func_00A8C2` .. `func_00A952`), one colour cycled through a list
//! (`func_00AA00`, `func_00AA52`) and the lines they work on
//! (`func_00AAC4` .. `func_00AAE2`).
//!
//! A colour is a word with three components of 3 bits, at bits 1-3,
//! 5-7 and 9-11 (the masks 0xE, 0xE0, 0xE00 below).
//!
//! No script calls `func_00A592`, `func_00A5AC`, `func_00A770`,
//! `func_00A8C2`, `func_00A904` or `func_00A952`: they are checked by
//! reading only.  `func_00A8C2` is the routine of a task that only
//! code the disassembly has as data makes (0xA87A-0xA8C1, between
//! `D_00A83A` and `func_00A8C2`).

use crate::{Game, Regs};

/// The colours: 4 lines of 16 words (0x20 bytes a line).
const COLOURS: u32 = 0xFF5800;
const LINE_BYTES: u32 = 0x20;
const LINE_WORDS: u8 = 0x10;
const LINE_COUNT: u8 = 4;
/// The copy of `COLOURS` a fade starts from and puts back at its end.
const SAVED: u32 = 0xFFFF_9F1A;
/// Where `func_00A784` and `func_00A904` read the colours they start
/// from (a long).
const SOURCE: u32 = 0xFFFF_9F16;
/// The level `func_00A7C8` scales each component by (0 none of it, 7
/// all of it) and what it adds to each after (both bytes).
const LEVEL: u32 = 0xFFFF_A3F6;
const LIFT: u32 = 0xFFFF_A3F7;
/// The lines a fade works on (a bit a line, line 0 bit 0):
/// `func_00A556` sets it to all four.
const LINES_DEFAULT: u32 = 0xFFFF_A3F9;
const ALL_LINES: u8 = 0xF;
/// A word shifted right a bit a line (`func_00AADC`): its low byte
/// (`LINES_LOW`) is the lines to work on, set before the loop.
const LINES: u32 = 0xFFFF_A3FA;
const LINES_LOW: u32 = 0xFFFF_A3FB;
/// While this isn't 0 the objects don't run (`a_objects.rs`): a fade
/// sets it.
const OBJECTS_STOPPED: u32 = 0xFFFF_A3FC;
/// The loops' counts (bytes): lines left, then words left in the line.
/// `LINES_LEFT` also holds a fade's speed while its task is made.
const LINES_LEFT: u32 = 0xFFFF_A3ED;
const WORDS_LEFT: u32 = 0xFFFF_A3EC;
/// `func_00A7C8`'s colour (a word, `COLOUR_LOW` its low byte) and the
/// low byte it makes.
const COLOUR_WORD: u32 = 0xFFFF_A3F0;
const COLOUR_LOW: u32 = 0xFFFF_A3F1;
const MADE_LOW: u32 = 0xFFFF_A3F5;
/// `func_00A952`'s components (bytes).
const STEP_A: u32 = 0xFFFF_A3F3;
const STEP_B: u32 = 0xFFFF_A3EF;
const STEP_C: u32 = 0xFFFF_A3F2;
/// The scaled components: by component (0-7) x 8 + `LEVEL`, the new
/// component x 2 (bytes in the ROM, `D_00A83A`).
const SCALE: u32 = 0x00A83A;
/// The highest component (x 2) `func_00A7C8` makes.
const TOP: u8 = 0xE;
/// `func_00A952`'s table: by one component x 8 + another, a component
/// x 2 (bytes in the ROM, `D_00A9C0`).
const STEPS: u32 = 0x00A9C0;

/// The colour cycle's list (a long, while its task is made), the
/// colour's index (a word, `CYCLE_INDEX_LOW` its low byte; the high
/// byte cleared).
const CYCLE_LIST: u32 = 0xFFFF_A3EA;
const CYCLE_INDEX: u32 = 0xFFFF_A3F0;
const CYCLE_INDEX_LOW: u32 = 0xFFFF_A3F1;
/// The colour index `func_00AA00` takes (a byte).
const CYCLE_COLOUR: u32 = 0xFFFF_80C9;
/// The cycle's entry read (a word): its high byte x 2 the frames it
/// stays, the rest the colour (`CYCLE_COLOUR_MASK`).
const CYCLE_ENTRY: u32 = 0xFFFF_A3EC;
const CYCLE_COLOUR_MASK: u16 = 0xEEE;
/// The entries that end the list and that start it again.
const CYCLE_END: u16 = 0;
const CYCLE_AGAIN: u16 = 0xFFFF;

/// The tasks: the head of the list (`task.rs`) and a record's fields.
const TASK_HEAD: u32 = 0xFFFF_80F2;
const TASK_NEXT: u32 = 0x00;
const TASK_PRIORITY: u32 = 0x02;
const TASK_NEXT_ADDR: u32 = 0x08;
/// The tail's `TASK_NEXT`.
const NO_TASK: u8 = 0xFF;
/// The priority of the fades' and the cycle's tasks.
const FADE_PRIORITY: u8 = 0xA;

/// A fade task's fields (bytes): frames between steps, frames left to
/// the next, the level and the lift (x 2) the step uses (or, in
/// `func_00A8C2`'s, the steps left), the lines (`LINES_DEFAULT` when
/// it is made).
const FADE_SPEED: u32 = 0x0C;
const FADE_WAIT: u32 = 0x0D;
const FADE_LEVEL: u32 = 0x0E;
const FADE_LIFT: u32 = 0x0F;
const FADE_LINES: u32 = 0x14;
/// `func_00A8C2`'s colours to step toward (a long).
const FADE_TARGET: u32 = 0x10;
/// The task's byte 0x1B (`task.rs`): 1 while it runs; bit 7 asks it
/// to end (`func_00A55E`).
const TASK_FLAG: u32 = 0x1B;
const FLAG_RUNNING: u8 = 1;
const FLAG_END: u8 = 0x80;
/// The cycle task's fields: its list (a long), the frames left (a
/// byte), the colour's address (a long), the next entry's offset.
const CYCLE_TASK_LIST: u32 = 0x0E;
const CYCLE_TASK_WAIT: u32 = 0x0D;
const CYCLE_TASK_COLOUR: u32 = 0x12;
const CYCLE_TASK_AT: u32 = 0x16;

/// The tasks' routines and where a yield goes on (code addresses).
const FADE_FROM_WHITE: u32 = 0x00A5E6;
const FADE_TO_WHITE: u32 = 0x00A61C;
const FADE_IN: u32 = 0x00A664;
const FADE_OUT: u32 = 0x00A6A4;
const FADE_END_1: u32 = 0x00A73C;
const FADE_END_2: u32 = 0x00A744;
const STEP_END: u32 = 0x00A8F8;
const CYCLE_TASK: u32 = 0x00AA52;
const CYCLE_END_TASK: u32 = 0x00AAC0;

/// The frames `func_00A63A` waits after its first step; the levels
/// and lifts the fades start or end at.
const FIRST_WAIT: u8 = 8;
const FULL: u8 = 7;
const LIFT_STEPS: u8 = 8;

impl Game {
    /// `func_00A506`: every fade ended (`func_00A55E`), all lines
    /// (`func_00A556`), then a fade in from black (`func_00A63A`) at
    /// speed `d0`.  Gives back d0 (the task's index), a1, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00a506(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        self.r_00a55e();
        self.r_00a556();
        self.r_00a63a(d0, d1, d2, d3, d4, d5, d6, d7, a2, a3, a4, a5)
    }

    /// `func_00A51A`: as `func_00A506`, a fade out to black
    /// (`loc_00A686`).
    #[allow(clippy::too_many_arguments)]
    pub fn r_00a51a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        self.r_00a55e();
        self.r_00a556();
        self.r_00a686(d0, d1, d2, d3, d4, d5, d6, d7, a2, a3, a4, a5)
    }

    /// `func_00A52E`: as `func_00A506`, a fade to white at speed `d0`:
    /// a task running `func_00A61C`, its level full.  d6, a0, a6 are
    /// kept.  Gives back d0 (the task's index), d6, a0, a1, a6, X and C
    /// (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00a52e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        self.r_00a55e();
        self.r_00a556();
        let (_, index, a1, task, x) = self.r_00a6c0(d0, d1, d2, d3, d4, d5, d6, d7, FADE_TO_WHITE, a2, a3, a4, a5);
        self.write8(task + FADE_LEVEL, FULL);
        (index, d6, a0, a1, a6, x, false)
    }

    /// `func_00A542`: as `func_00A506`, a fade from white at speed `d0`:
    /// a task running `func_00A5E6`, its level full and its lift too.
    /// d6, a0, a6 are kept.  Gives back d0 (the task's index), d6, a0,
    /// a1, a6 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00a542(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, bool) {
        self.r_00a55e();
        self.r_00a556();
        let (_, index, a1, task, x) = self.r_00a6c0(d0, d1, d2, d3, d4, d5, d6, d7, FADE_FROM_WHITE, a2, a3, a4, a5);
        self.write8(task + FADE_LEVEL, FULL);
        self.write8(task + FADE_LIFT, FULL);
        (index, d6, a0, a1, a6, x)
    }

    /// `func_00A556`: a fade works on all four lines.
    pub fn r_00a556(&mut self) {
        self.write8(LINES_DEFAULT, ALL_LINES);
    }

    /// `func_00A55E`: every task at `FADE_PRIORITY` is asked to end
    /// (`FLAG_END`), from the list's head to the first task after them.
    pub fn r_00a55e(&mut self) {
        let mut task = TASK_HEAD;
        while self.read8(task + TASK_NEXT) != NO_TASK {
            task = self.read32(task + TASK_NEXT_ADDR);
            let priority = self.read8(task + TASK_PRIORITY);
            if priority > FADE_PRIORITY {
                break;
            }
            if priority == FADE_PRIORITY {
                self.write8(task + TASK_FLAG, FLAG_END);
            }
        }
    }

    /// `func_00A592`: the lines in `LINES_DEFAULT` of `COLOURS` at their
    /// full level with `d0` added (`func_00A770`).
    pub fn r_00a592(&mut self, d0: u32, d3: u32) {
        self.write8(LIFT, d0 as u8);
        self.write8(LEVEL, FULL);
        self.write32(SOURCE, COLOURS);
        self.r_00a770(d3);
    }

    /// `func_00A5AC`: the lines in `LINES_DEFAULT` of `COLOURS` at level
    /// `d0` (`func_00A770`).
    pub fn r_00a5ac(&mut self, d0: u32, d3: u32) {
        self.write8(LEVEL, d0 as u8);
        self.write8(LIFT, 0);
        self.write32(SOURCE, COLOURS);
        self.r_00a770(d3);
    }

    /// `func_00A5E6` (a task's routine), the fade from white: at each
    /// step (`func_00A702`) the lift goes down; once it is below 0 the
    /// fade ends (`func_00A738`).  Gives back d0, X and C (and what
    /// the end does).
    pub fn r_00a5e6(&mut self, regs: &mut Regs) {
        if !self.j_fade_step(regs) {
            return;
        }
        let lift = self.read8(regs.a6 + FADE_LIFT);
        let lift = self.j_count_down(regs, regs.a6 + FADE_LIFT, lift);
        if lift & 0x80 != 0 {
            self.j_fade_end(regs);
        }
    }

    /// `func_00A61C` (a task's routine), the fade to white: at each step
    /// (`func_00A702`) the lift goes up; once it is `LIFT_STEPS` the
    /// fade ends (`func_00A738`).  Gives back d0, X and C.
    pub fn r_00a61c(&mut self, regs: &mut Regs) {
        if !self.j_fade_step(regs) {
            return;
        }
        let at = regs.a6 + FADE_LIFT;
        if self.j_count_up(regs, at) >= LIFT_STEPS {
            self.j_fade_end(regs);
        }
    }

    /// `func_00A63A`: a fade in from black at speed `d0`: a task running
    /// `func_00A664`, its first step taken at once (`func_00A702`, level
    /// 0) and the next `FIRST_WAIT` frames on.  d6 is kept.  Gives back
    /// d0 (the task's index), a1, X and C (clear).
    #[allow(clippy::too_many_arguments)]
    pub fn r_00a63a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let (d0, index, a1, task, _) = self.r_00a6c0(d0, d1, d2, d3, d4, d5, d6, d7, FADE_IN, a2, a3, a4, a5);
        self.write8(task + FADE_WAIT, 1);
        let (_, x, _, _) = self.r_00a702(d0, d3, task);
        self.write8(task + FADE_WAIT, FIRST_WAIT);
        (index, a1, x, false)
    }

    /// `func_00A664` (a task's routine), the fade in: at each step
    /// (`func_00A702`) the picture is on (`func_0002DA`) and the level
    /// goes up; once it is past `FULL` the fade ends (`func_00A738`).
    /// Gives back d0, X and C.
    pub fn r_00a664(&mut self, regs: &mut Regs) {
        if !self.j_fade_step(regs) {
            return;
        }
        self.r_0002da();
        let at = regs.a6 + FADE_LEVEL;
        if self.j_count_up(regs, at) > FULL {
            self.j_fade_end(regs);
        }
    }

    /// `loc_00A686`: a fade out to black at speed `d0`: a task running
    /// `func_00A6A4`, its level full.  Gives back d0 (the task's index),
    /// a1, X and C (clear).
    #[allow(clippy::too_many_arguments)]
    pub fn r_00a686(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let (_, index, a1, task, x) = self.r_00a6c0(d0, d1, d2, d3, d4, d5, d6, d7, FADE_OUT, a2, a3, a4, a5);
        self.write8(task + FADE_LEVEL, FULL);
        (index, a1, x, false)
    }

    /// `func_00A6A4` (a task's routine), the fade out: at each step
    /// (`func_00A702`) the level goes down; once it is below 0 the
    /// picture is off (`func_0002E0`) and the fade ends
    /// (`func_00A738`).  Gives back d0, X and C.
    pub fn r_00a6a4(&mut self, regs: &mut Regs) {
        if !self.j_fade_step(regs) {
            return;
        }
        let level = self.read8(regs.a6 + FADE_LEVEL);
        let level = self.j_count_down(regs, regs.a6 + FADE_LEVEL, level);
        if level & 0x80 != 0 {
            self.r_0002e0();
            self.j_fade_end(regs);
        }
    }

    /// `func_00A6C0`: a fade's task, running `routine`
    /// (`func_000404`, at `FADE_PRIORITY`), at speed `d0` (its low
    /// byte): `COLOURS` copied to `SAVED` (`func_00AAE2`), its first
    /// step next frame, level and lift 0, its lines `LINES_DEFAULT`
    /// (`func_00AAC4`); the objects stop.  Gives back d0 (0xA), d6 and
    /// a6 (the task's index and record), a1 (`SAVED`) and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00a6c0(&mut self, d0: u32, _d1: u32, _d2: u32, _d3: u32, d4: u32, d5: u32, d6: u32, _d7: u32, routine: u32, _a2: u32, _a3: u32, _a4: u32, _a5: u32) -> (u32, u32, u32, u32, bool) {
        self.write8(LINES_LEFT, d0 as u8);
        let d0 = u32::from(FADE_PRIORITY);
        let (d6, task, _, _) = self.r_000404(d0, d4, d5, d6, routine);
        let x = self.r_00aae2(COLOURS, SAVED);
        let speed = self.read8(LINES_LEFT);
        self.write8(task + FADE_SPEED, speed);
        self.write8(task + FADE_WAIT, 1);
        self.write8(task + FADE_LEVEL, 0);
        self.write8(task + FADE_LIFT, 0);
        self.write8(task + TASK_FLAG, FLAG_RUNNING);
        self.write8(OBJECTS_STOPPED, 1);
        let mut regs = Regs { a6: task, ..Regs::default() };
        self.r_00aac4(&mut regs);
        (d0, d6, SAVED, task, x)
    }

    /// `func_00A702`: a fade's wait counted down; when it runs out, it
    /// starts again from the speed and the step is taken: the task's
    /// lines of `SAVED` to `COLOURS` at its level and lift
    /// (`func_00A75C`), sent on (`func_0094B2`).  Gives back d0 (0 if the
    /// step was taken), X, Z (set if it was) and C.
    pub fn r_00a702(&mut self, d0: u32, d3: u32, task: u32) -> (u32, bool, bool, bool) {
        let wait = self.read8(task + FADE_WAIT);
        self.write8(task + FADE_WAIT, wait.wrapping_sub(1));
        if wait != 1 {
            // the borrow of subq
            return (d0, wait == 0, false, wait == 0);
        }
        let speed = self.read8(task + FADE_SPEED);
        self.write8(task + FADE_WAIT, speed);
        let level = self.read8(task + FADE_LEVEL);
        self.write8(LEVEL, level);
        let lift = self.read8(task + FADE_LIFT);
        self.write8(LIFT, lift.wrapping_add(lift));
        self.write32(SOURCE, SAVED);
        let x = self.r_00a75c(d3, task);
        self.r_0094b2();
        (0, x, true, false)
    }

    /// `func_00A738` (where a fade ends; the end of a task's routine):
    /// the task yields (`func_000522`) and goes on in `func_00A73C`.
    /// Gives back d6, a0, a1, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00a738(&mut self, _d0: u32, _d1: u32, _d2: u32, _d3: u32, _d4: u32, _d5: u32, d6: u32, _d7: u32, a0: u32, a1: u32, _a2: u32, _a3: u32, _a4: u32, _a5: u32, _a6: u32) -> (u32, u32, u32, bool, bool) {
        let (x, c, _) = self.r_000522(d6, FADE_END_1);
        (d6, a0, a1, x, c)
    }

    /// `func_00A73C` (a task's routine, a fade's end, the frame after):
    /// the task's flag cleared, and it yields (`func_000522`) to go on
    /// in `func_00A744`.  Gives back X and C.
    pub fn r_00a73c(&mut self, regs: &mut Regs) {
        self.write8(regs.a6 + TASK_FLAG, 0);
        let (x, c, _) = self.r_000522(regs.d6, FADE_END_2);
        regs.x = x;
        regs.c = c;
    }

    /// `func_00A744` (a task's routine, a fade's end, the frame after
    /// that): `SAVED` put back in `COLOURS` (`func_00AAE2`), the objects
    /// run again and the task ends (`func_000560`).  Gives back d6, a0
    /// (`SAVED`), a1 (`COLOURS`), X and C.
    pub fn r_00a744(&mut self, regs: &mut Regs) {
        regs.a0 = SAVED;
        regs.a1 = COLOURS;
        let x = self.r_00aae2(SAVED, COLOURS);
        self.write8(OBJECTS_STOPPED, 0);
        (regs.d6, regs.x, regs.c) = self.r_000560(regs.d5, regs.d6, x);
    }

    /// `func_00A75C`: `func_00A784` on the lines of fade task `task`.
    /// Gives back X.
    pub fn r_00a75c(&mut self, d3: u32, task: u32) -> bool {
        self.r_00aad4(task);
        self.r_00a784(d3)
    }

    /// `func_00A770`: `func_00A784` on the lines in `LINES_DEFAULT`.
    pub fn r_00a770(&mut self, d3: u32) {
        self.r_00aacc();
        self.r_00a784(d3);
    }

    /// `func_00A784`: each line in `LINES_LOW` of the colours at
    /// `SOURCE` to `COLOURS`, each colour at `LEVEL` and `LIFT`
    /// (`func_00A7C8`).  Gives back X (clear: the last line's count).
    pub fn r_00a784(&mut self, d3: u32) -> bool {
        let mut from = self.read32(SOURCE);
        let mut to = COLOURS;
        let mut d3 = d3;
        for _ in 0..LINE_COUNT {
            let (_, take) = self.r_00aadc();
            if take {
                for _ in 0..LINE_WORDS {
                    let colour = self.read16(from);
                    d3 = self.r_00a7c8((d3 & 0xFFFF_0000) | u32::from(colour));
                    self.write16(to, d3 as u16);
                    from += 2;
                    to += 2;
                }
                self.write8(WORDS_LEFT, 0);
            } else {
                from += LINE_BYTES;
                to += LINE_BYTES;
            }
        }
        self.write8(LINES_LEFT, 0);
        false
    }

    /// `func_00A7C8`: colour `d3` (its low word, kept in `COLOUR_WORD`)
    /// at `LEVEL` and `LIFT`: each component scaled (`SCALE`) and `LIFT`
    /// added, `TOP` at most (`MADE_LOW` the low byte meanwhile).  The
    /// high word of d3 is kept, or cleared if any component reached
    /// `TOP`.  Gives back d3, the colour.
    pub fn r_00a7c8(&mut self, d3: u32) -> u32 {
        let colour = d3 as u16;
        self.write16(COLOUR_WORD, colour);
        let mut high = d3 & 0xFFFF_0000;
        let first = self.j_scaled((colour as u8 & 0xE) << 2, &mut high);
        self.write8(MADE_LOW, first);
        let green = self.read8(COLOUR_LOW);
        let second = self.j_scaled((green & 0xE0) >> 2, &mut high);
        let low = self.read8(MADE_LOW) | second << 4;
        self.write8(MADE_LOW, low);
        let blue = self.read8(COLOUR_WORD);
        let third = self.j_scaled((blue & 0xE) << 2, &mut high);
        high | u32::from(third) << 8 | u32::from(self.read8(MADE_LOW))
    }

    /// A component (x 8, from `func_00A7C8`) scaled to `LEVEL` with
    /// `LIFT` added, `TOP` at most (then the high word of the register
    /// is cleared: `moveq`).
    fn j_scaled(&mut self, component: u8, high: &mut u32) -> u8 {
        let index = component | self.read8(LEVEL);
        let scaled = self.read8(SCALE + u32::from(index));
        let value = scaled.wrapping_add(self.read8(LIFT));
        if value < TOP {
            value
        } else {
            *high = 0;
            TOP
        }
    }

    /// `func_00A8C2` (a task's routine, made by code the disassembly
    /// has as data at 0xA87A): unless asked to end, a step every
    /// `FADE_SPEED` frames: the task's lines of `COLOURS` a step toward
    /// those at `FADE_TARGET` (`func_00A904`), sent on (`func_0094B2`),
    /// `FADE_LEVEL` steps.  After the last (or when asked), it yields
    /// (`func_000522`) to go on at 0xA8F8 (the flag cleared, a yield
    /// again, then the task ends), a place the translation has no
    /// routine for.  Gives back X and C.
    pub fn r_00a8c2(&mut self, regs: &mut Regs) {
        let task = regs.a6;
        if self.read8(task + TASK_FLAG) & 0x80 == 0 {
            let wait = self.read8(task + FADE_WAIT);
            self.write8(task + FADE_WAIT, wait.wrapping_sub(1));
            (regs.x, regs.c) = (wait == 0, wait == 0);
            if wait != 1 {
                return;
            }
            let speed = self.read8(task + FADE_SPEED);
            self.write8(task + FADE_WAIT, speed);
            let target = self.read32(task + FADE_TARGET);
            self.write32(SOURCE, target);
            self.r_00a904(task);
            self.r_0094b2();
            let steps = self.read8(task + FADE_LEVEL);
            self.write8(task + FADE_LEVEL, steps.wrapping_sub(1));
            (regs.x, regs.c) = (steps == 0, steps == 0);
            if steps != 1 {
                return;
            }
        }
        let (x, c, _) = self.r_000522(regs.d6, STEP_END);
        regs.x = x;
        regs.c = c;
    }

    /// `func_00A904`: each line of fade task `task` (`FADE_LINES`) of
    /// `COLOURS` a step toward the colours at `SOURCE` (`func_00A952`).
    pub fn r_00a904(&mut self, task: u32) {
        self.r_00aad4(task);
        let mut from = self.read32(SOURCE);
        let mut to = COLOURS;
        for _ in 0..LINE_COUNT {
            let (_, take) = self.r_00aadc();
            if take {
                for _ in 0..LINE_WORDS {
                    (from, to) = self.r_00a952(from, to);
                }
                self.write8(WORDS_LEFT, 0);
            } else {
                from += LINE_BYTES;
                to += LINE_BYTES;
            }
        }
        self.write8(LINES_LEFT, 0);
    }

    /// `func_00A952`: the colour at `to` a step toward the one at
    /// `target` through `STEPS`, but with the bytes as they come: the
    /// target's high byte's bits 1-3 with the colour's bits 1-3 (to bits
    /// 1-3), its bits 5-7 with the colour's bits 5-7 (to bits 5-7), the
    /// target's low byte's bits 1-3 with the colour's bits 9-11 (to bits
    /// 9-11), the components meanwhile in `STEP_A`, `STEP_B`,
    /// `STEP_C`.  Gives back both addresses, a word on.
    pub fn r_00a952(&mut self, target: u32, to: u32) -> (u32, u32) {
        let colour = self.read16(to);
        let low = colour as u8;
        self.write8(STEP_A, (low & 0xE) >> 1);
        self.write8(STEP_B, (low & 0xE0) >> 5);
        self.write8(STEP_C, ((colour >> 8) as u8 & 0xE) >> 1);
        let high_byte = self.read8(target);
        let low_byte = self.read8(target + 1);
        self.j_step(STEP_A, (high_byte & 0xE) << 2);
        self.j_step(STEP_B, (high_byte & 0xE0) >> 2);
        self.j_step(STEP_C, (low_byte & 0xE) << 2);
        let made = self.read8(STEP_B) | self.read8(STEP_A);
        self.write8(to, made);
        let made = self.read8(STEP_C);
        self.write8(to + 1, made);
        (target + 2, to + 2)
    }

    /// The component kept at `at` replaced by `STEPS`' entry for it and
    /// `row` (`func_00A952`).
    fn j_step(&mut self, at: u32, row: u8) {
        let index = row | self.read8(at);
        let value = self.read8(STEPS + u32::from(index));
        self.write8(at, value);
    }

    /// `func_00AA00`: a task (`func_0003E8`, at `FADE_PRIORITY`) that
    /// cycles colour `CYCLE_COLOUR` of `COLOURS` through the list at
    /// `list` (`func_00AA52`), from its start, next frame.  d6 and a6
    /// are kept.  Gives back d0 (the task's index), a0 (the colour's
    /// address), X and C (clear).
    pub fn r_00aa00(&mut self, d4: u32, d5: u32, d6: u32, list: u32) -> (u32, u32, bool, bool) {
        self.write32(CYCLE_LIST, list);
        let colour = self.read8(CYCLE_COLOUR);
        self.write8(CYCLE_INDEX_LOW, colour);
        self.write8(CYCLE_INDEX, 0);
        let (index, task, _, _) = self.r_0003e8(d4, d5, d6, FADE_PRIORITY, CYCLE_TASK);
        let list = self.read32(CYCLE_LIST);
        self.write32(task + CYCLE_TASK_LIST, list);
        self.write8(task + CYCLE_TASK_AT, 0);
        self.write8(task + CYCLE_TASK_WAIT, 1);
        let offset = self.read16(CYCLE_INDEX) << 1;
        let address = COLOURS.wrapping_add(offset as i16 as u32);
        self.write32(task + CYCLE_TASK_COLOUR, address);
        self.write8(task + TASK_FLAG, FLAG_RUNNING);
        // X: add.w's carry, never (the index is a byte)
        (index, address, false, false)
    }

    /// `func_00AA52` (a task's routine), the colour cycle: unless asked
    /// to end or the objects are stopped (`func_001BB4`), when its wait
    /// runs out the next entry of its list: the colour
    /// (`CYCLE_COLOUR_MASK`) set and sent on (`func_0094B2`), the wait
    /// from its high byte.  `CYCLE_AGAIN` goes back to the first entry;
    /// at `CYCLE_END` (or when asked) the task's flag is cleared and it
    /// yields (`func_000522`) to end in `func_00AAC0`.  Gives back d0,
    /// a0, X and C.
    pub fn r_00aa52(&mut self, regs: &mut Regs) {
        let task = regs.a6;
        if self.read8(task + TASK_FLAG) & 0x80 == 0 {
            let (c, stopped) = self.r_001bb4();
            regs.c = c;
            if stopped {
                return;
            }
            let wait = self.read8(task + CYCLE_TASK_WAIT);
            self.write8(task + CYCLE_TASK_WAIT, wait.wrapping_sub(1));
            (regs.x, regs.c) = (wait == 0, wait == 0);
            if wait != 1 {
                return;
            }
            let list = self.read32(task + CYCLE_TASK_LIST);
            regs.a0 = list;
            let at = self.read8(task + CYCLE_TASK_AT);
            let entry = self.read16(list + u32::from(at));
            self.write16(CYCLE_ENTRY, entry);
            self.write8(task + CYCLE_TASK_AT, at.wrapping_add(2));
            regs.d0 = (regs.d0 & 0xFFFF_0000) | u32::from(entry);
            if entry != CYCLE_END {
                if entry == CYCLE_AGAIN {
                    let first = self.read16(list);
                    self.write16(CYCLE_ENTRY, first);
                    self.write8(task + CYCLE_TASK_AT, 2);
                }
                let frames = self.read8(CYCLE_ENTRY);
                regs.x = frames & 1 != 0;
                self.write8(task + CYCLE_TASK_WAIT, frames >> 1);
                let address = self.read32(task + CYCLE_TASK_COLOUR);
                regs.a0 = address;
                let colour = self.read16(CYCLE_ENTRY) & CYCLE_COLOUR_MASK;
                regs.d0 = (regs.d0 & 0xFFFF_0000) | u32::from(colour);
                self.write16(address, colour);
                regs.c = self.r_0094b2();
                return;
            }
        }
        self.write8(task + TASK_FLAG, 0);
        let (x, c, _) = self.r_000522(regs.d6, CYCLE_END_TASK);
        regs.x = x;
        regs.c = c;
    }

    /// `func_00AAC0` (a task's routine): the task ends (`func_000560`).
    /// Gives back d6, X and C.
    pub fn r_00aac0(&mut self, regs: &mut Regs) {
        (regs.d6, regs.x, regs.c) = self.r_000560(regs.d5, regs.d6, regs.x);
    }

    /// `func_00AAC4`: fade task a6's lines are `LINES_DEFAULT`.  Gives
    /// back N, Z and C (clear) of the byte.
    pub fn r_00aac4(&mut self, regs: &mut Regs) {
        let lines = self.read8(LINES_DEFAULT);
        self.write8(regs.a6 + FADE_LINES, lines);
        regs.n = lines & 0x80 != 0;
        regs.z = lines == 0;
        regs.c = false;
    }

    /// `func_00AACC`: the lines to work on are `LINES_DEFAULT`.
    pub fn r_00aacc(&mut self) {
        let lines = self.read8(LINES_DEFAULT);
        self.write8(LINES_LOW, lines);
    }

    /// `func_00AAD4`: the lines to work on are fade task `task`'s.
    pub fn r_00aad4(&mut self, task: u32) {
        let lines = self.read8(task + FADE_LINES);
        self.write8(LINES_LOW, lines);
    }

    /// `func_00AADC`: the next line: `LINES` shifted right a bit.  Gives
    /// back X and C, the bit shifted out (set: work on the line).
    pub fn r_00aadc(&mut self) -> (bool, bool) {
        let lines = self.read16(LINES);
        self.write16(LINES, lines >> 1);
        let bit = lines & 1 != 0;
        (bit, bit)
    }

    /// `func_00AAE2`: the lines in `LINES_DEFAULT` of the colours at
    /// `from` copied to `to`.  Gives back X (the last line's bit).
    pub fn r_00aae2(&mut self, from: u32, to: u32) -> bool {
        self.r_00aacc();
        let mut x = false;
        for line in 0..u32::from(LINE_COUNT) {
            let take;
            (x, take) = self.r_00aadc();
            if take {
                for k in (0..LINE_BYTES).step_by(4) {
                    let long = self.read32(from + line * LINE_BYTES + k);
                    self.write32(to + line * LINE_BYTES + k, long);
                }
            }
        }
        x
    }

    /// A fade's step (`func_00A702`, from a fade task's routine).
    /// Gives back whether it was taken.
    fn j_fade_step(&mut self, regs: &mut Regs) -> bool {
        let (d0, x, stepped, c) = self.r_00a702(regs.d0, regs.d3, regs.a6);
        regs.d0 = d0;
        regs.x = x;
        regs.c = c;
        stepped
    }

    /// The byte `value` at `at` less 1 (subq: X and C its borrow).
    /// Gives back what it is now.
    fn j_count_down(&mut self, regs: &mut Regs, at: u32, value: u8) -> u8 {
        let value_now = value.wrapping_sub(1);
        self.write8(at, value_now);
        regs.x = value == 0;
        regs.c = value == 0;
        value_now
    }

    /// The byte at `at` plus 1 (addq: X its carry), then compared with
    /// 8 (cmpi: C if below).  Gives back what it is now.
    fn j_count_up(&mut self, regs: &mut Regs, at: u32) -> u8 {
        let value = self.read8(at).wrapping_add(1);
        self.write8(at, value);
        regs.x = value == 0;
        regs.c = value < 8;
        value
    }

    /// A fade ends (`func_00A738`, the end of the task's routine).
    fn j_fade_end(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.d6, r.a0, r.a1, r.x, r.c) = self.r_00a738(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }
}
