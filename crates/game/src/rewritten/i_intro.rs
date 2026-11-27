//! A stage's opening (`func_009C7A`): two tasks with sprites, moved
//! by the stage's kind (`KIND`, the tables of routines by it), and the
//! task that counts down to the stage's start (`func_009ACA`).
//!
//! The tasks count frames in their `TASK_COUNT` and go from one
//! routine to the next (`func_00053E`) when it runs out.  The moves
//! (`func_009D7C` .. `func_009FAA`) take the running task's record
//! again first (`func_0005F8` with `CURRENT`).

use crate::{Game, Regs};

/// A byte: the stage's kind (0-5), the index into the tables of
/// routines, of sounds (`SOUNDS`) and of sprite frames.
const KIND: u32 = 0xFFFF_953A;
/// A byte the second sprite's frame is chosen by.
const KIND_2: u32 = 0xFFFF_953C;
/// The kind without the sprites (only the first record is set).
const BARE: u8 = 5;
/// The Z80 command by kind (bytes in the ROM, `D_009C74`).
const SOUNDS: u32 = 0x009C74;
/// The tables of routines by kind (longs in the ROM).
const FIRST_MOVES: u32 = 0x009D64;
const FIRST_STEPS: u32 = 0x009DF2;
const SECOND_MOVES: u32 = 0x009F18;
const SECOND_STEPS: u32 = 0x009F58;

/// The running task's index (`task.rs`).
const CURRENT: u32 = 0xFFFF_80EE;
/// The velocity (words: x, y) `func_001A88` and `func_001AA2` move a
/// task by (`a_motion.rs`).
const VELOCITY_X: u32 = 0xFFFF_9EC4;
const VELOCITY_Y: u32 = 0xFFFF_9EC6;

/// The countdown's task (`func_009ACA`): its priority, routine, and
/// the frames it waits.
const COUNTDOWN_PRIORITY: u8 = 0x0B;
const COUNTDOWN: u32 = 0x009AEC;
/// Set (bytes) when the countdown starts; `STOPPED` stops the objects
/// meanwhile (`a_objects.rs`).
const CLEARED_ON_COUNTDOWN: u32 = 0xFFFF_A42A;
const STOPPED: u32 = 0xFFFF_A3FC;
/// Set to 0xFFFF (a word) when the countdown ends.
const ENDED: u32 = 0xFFFF_A3F8;
/// `func_00A63A`'s argument when the countdown ends.
const ON_END: u32 = 3;

/// The opening's tasks: priority and depth of their sprite records,
/// their routines.
const OPENING_PRIORITY: u8 = 0x42;
const OPENING_DEPTH: u8 = 1;
const FIRST: u32 = 0x009D02;
const FIRST_TURN: u32 = 0x009D34;
const FIRST_WAIT: u32 = 0x009DBE;
const FIRST_LEAVE: u32 = 0x009DCC;
const SECOND: u32 = 0x009EC0;
const SECOND_TURN: u32 = 0x009EEE;
const SECOND_LEAVE: u32 = 0x009F30;

/// A task's fields (as `a_motion.rs`'s): its position's pixels, and
/// the rate `func_00A040` adds to.
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
/// The two extra sprite records' indexes (bytes) of the first task.
const TASK_SPRITE_2: u32 = 0x12;
const TASK_SPRITE_3: u32 = 0x13;
/// Frames left (a byte).
const TASK_COUNT: u32 = 0x16;
const TASK_RATE: u32 = 0x1A;

/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const FLAGS_SHOWN: u8 = 0x83;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_009C7A`: the kind's sound (`SOUNDS`, `func_0034B6`) and
    /// the opening's two tasks (`func_001222`): the first (`FIRST`) at
    /// (0xFF80, 0x60) with two more sprite records (`func_000796`, no
    /// frame), the second (`SECOND`) at (0x180, 0x80), frame 0x16; both
    /// with 0x40 frames to count.  Gives back d0, d5, d6, a5, a6 (the
    /// second task's) and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_009c7a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool) {
        let kind = self.read8(KIND);
        let sound = self.read8(SOUNDS + u32::from(kind));
        let d0 = low_byte(low_word(d0, 0), sound);
        self.r_0034b6(d0);
        let (d5, d6, a5, a6, x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, OPENING_DEPTH, OPENING_PRIORITY, FIRST);
        self.write16(a6 + TASK_X, 0xFF80);
        self.write16(a6 + TASK_Y, 0x60);
        self.write8(a6 + TASK_COUNT, 0x40);
        let (d5, a5, x, _) = self.r_000796(1, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        self.write8(a6 + TASK_SPRITE_2, d5 as u8);
        self.write8(a5 + SPRITE_FRAME, 0);
        let (d5, a5, ..) = self.r_000796(1, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        self.write8(a6 + TASK_SPRITE_3, d5 as u8);
        self.write8(a5 + SPRITE_FRAME, 0);
        let (d5, d6, a5, a6, x, _) = self.r_001222(1, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, OPENING_DEPTH, OPENING_PRIORITY, SECOND);
        self.write8(a5 + SPRITE_FRAME, 0x16);
        self.write8(a5 + SPRITE_FLAGS, FLAGS_SHOWN);
        self.write16(a6 + TASK_X, 0x180);
        self.write16(a6 + TASK_Y, 0x80);
        self.write8(a6 + TASK_COUNT, 0x40);
        (1, d5, d6, a5, a6, x)
    }

    /// The task's `TASK_COUNT` down by 1, X and C its borrow; true when
    /// it reaches 0.
    fn i_count_down(&mut self, r: &mut Regs) -> bool {
        let count = self.read8(r.a6 + TASK_COUNT);
        self.write8(r.a6 + TASK_COUNT, count.wrapping_sub(1));
        r.x = count == 0;
        r.c = r.x;
        count == 1
    }

    /// The kind x 4 into d0's low word (X, C clear): the offset into a
    /// table by kind.
    fn i_by_kind(&mut self, r: &mut Regs) -> u32 {
        let offset = u16::from(self.read8(KIND)) * 4;
        r.d0 = low_word(r.d0, offset);
        r.x = false;
        r.c = false;
        u32::from(offset)
    }

    /// `func_009D02`, the first task: moved right 4 pixels a frame
    /// (`func_001A88`), its sprites placed (`func_009FC6`); when its
    /// count runs out, 0x3C frames more of `FIRST_TURN` and the
    /// countdown made (`func_009ACA`).
    pub fn r_009d02(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.write16(VELOCITY_X, 0x400);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d5, r.a5, r.x, r.c) = self.r_009fc6(r.d5, r.a5, r.a6);
        if self.i_count_down(r) {
            self.write8(r.a6 + TASK_COUNT, 0x3C);
            (r.x, r.c) = self.r_00053e(r.d6, FIRST_TURN);
            (r.d6, r.a6, r.x, r.c) = self.r_009aca(r.d4, r.d5, r.d6);
        }
        r.unwound = false;
    }

    /// `func_009D32`: an `rts`.
    pub fn r_009d32(&mut self) {}

    /// `func_009D34`, the first task: when its count runs out, the
    /// kind's move (`FIRST_MOVES`), 0x80 frames of `FIRST_WAIT`, and a
    /// step (`func_009DD8`).
    pub fn r_009d34(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        if !self.i_count_down(r) {
            r.unwound = false;
            return;
        }
        let offset = self.i_by_kind(r);
        let routine = self.read32(FIRST_MOVES + offset);
        r.a0 = routine;
        r.unwound = false;
        let mut called = Regs { ..*r };
        self.call_family_1(routine, &mut called);
        (r.d6, r.a6, r.x, r.c, r.unwound) = (called.d6, called.a6, called.x, called.c, called.unwound);
        if r.unwound {
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, FIRST_WAIT);
        self.write8(r.a6 + TASK_COUNT, 0x80);
        (r.d0, r.d1, r.d5, r.d6, r.a0, r.a5, r.a6, r.x, r.c) = self.r_009dd8(r.d0, r.d1, r.d5, r.d6, r.a5);
    }

    /// The running task's record again (`func_0005F8` with `CURRENT`)
    /// into d6, a6.
    fn i_running(&mut self, r: &mut Regs) {
        let index = low_byte(r.d6, self.read8(CURRENT));
        (r.d6, r.a6, r.x) = self.r_0005f8(index);
    }

    /// `func_009D7C`, a move: the task's rate 0x200.
    pub fn r_009d7c(&mut self, regs: &mut Regs) {
        self.i_set_rate(regs, 0x200);
    }

    /// `func_009D8C`, a move: the task's rate 0xFC00.
    pub fn r_009d8c(&mut self, regs: &mut Regs) {
        self.i_set_rate(regs, 0xFC00);
    }

    /// `func_009D9C`, a move: the task's rate 0.
    pub fn r_009d9c(&mut self, regs: &mut Regs) {
        self.i_set_rate(regs, 0);
    }

    /// `func_009DAC`, a move: the task's rate 0xFE00.
    pub fn r_009dac(&mut self, regs: &mut Regs) {
        self.i_set_rate(regs, 0xFE00);
    }

    /// `func_009DBC`, a move: nothing.
    pub fn r_009dbc(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// The running task's `TASK_RATE` set to `rate`; C clear.
    fn i_set_rate(&mut self, r: &mut Regs, rate: u16) {
        self.i_running(r);
        self.write16(r.a6 + TASK_RATE, rate);
        r.c = false;
        r.unwound = false;
    }

    /// `func_009DBE`, the first task: `FIRST_LEAVE` from the next frame.
    pub fn r_009dbe(&mut self, regs: &mut Regs) {
        (regs.x, regs.c) = self.r_00053e(regs.d6, FIRST_LEAVE);
        regs.unwound = false;
    }

    /// `func_009DCC`, the first task: a step (`func_009DD8`) until its
    /// count runs out, then it ends (`func_00121A`).
    pub fn r_009dcc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        if self.i_count_down(r) {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        } else {
            (r.d0, r.d1, r.d5, r.d6, r.a0, r.a5, r.a6, r.x, r.c) = self.r_009dd8(r.d0, r.d1, r.d5, r.d6, r.a5);
        }
        r.unwound = false;
    }

    /// `func_009DD8`: the first task's step by kind (`FIRST_STEPS`),
    /// then its sprites placed (`func_009FC6`).  Gives back d0, d1, d5,
    /// d6, a0, a5, a6, X and C.
    #[allow(clippy::type_complexity)]
    pub fn r_009dd8(&mut self, d0: u32, d1: u32, d5: u32, d6: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d5, d6, a5, ..Regs::default() };
        let offset = self.i_by_kind(&mut r);
        r.a0 = self.read32(FIRST_STEPS + offset);
        let routine = r.a0;
        let mut called = Regs { ..r };
        self.call_family_17(routine, &mut called);
        let (d1, d6, a6) = (called.d1, called.d6, called.a6);
        if called.unwound {
            return (r.d0, d1, d5, d6, r.a0, a5, a6, r.x, r.c);
        }
        let (d0, d5, a5, x, c) = self.r_009fc6(d5, a5, a6);
        (d0, d1, d5, d6, r.a0, a5, a6, x, c)
    }

    /// `func_009E0A`, a step: right 2 pixels a frame, and down by the
    /// rate less 0x20 (`func_009E1C`).
    pub fn r_009e0a(&mut self, regs: &mut Regs) {
        self.i_running(regs);
        self.i_move_x(regs, 0x200);
        regs.d1 = self.r_009e1c(regs.d0, regs.d1, regs.a6);
        regs.unwound = false;
    }

    /// `func_009E1C`: the task's rate less 0x20 (`func_00A040`) its y
    /// velocity, and it moved by it (`func_001AA2`).  Gives back d1.
    pub fn r_009e1c(&mut self, d0: u32, d1: u32, a6: u32) -> u32 {
        self.i_rate_y(low_word(d0, 0xFFE0), d1, a6)
    }

    /// `func_009E2E`, a step: right 2 pixels a frame, and down by the
    /// rate plus 0x30 (`func_009E40`).
    pub fn r_009e2e(&mut self, regs: &mut Regs) {
        self.i_running(regs);
        self.i_move_x(regs, 0x200);
        regs.d1 = self.r_009e40(regs.d0, regs.d1, regs.a6);
        regs.unwound = false;
    }

    /// `func_009E40`: as `func_009E1C`, the rate plus 0x30.
    pub fn r_009e40(&mut self, d0: u32, d1: u32, a6: u32) -> u32 {
        self.i_rate_y(low_word(d0, 0x30), d1, a6)
    }

    /// `func_009E52`, a step: up 1.5 pixels a frame, and sideways at
    /// random (`func_009E60`).
    pub fn r_009e52(&mut self, regs: &mut Regs) {
        self.i_running(regs);
        self.write16(VELOCITY_Y, 0xFE80);
        regs.d1 = self.r_009e60(regs.d1, regs.a6);
        regs.unwound = false;
    }

    /// `func_009E60`: the task's rate plus a number from -0x30 to 0x30
    /// (`func_001BC0`, `func_00A040`) its x velocity; it moved by both
    /// (`func_001A88`, `func_001AA2`).  Gives back d1.
    pub fn r_009e60(&mut self, d1: u32, a6: u32) -> u32 {
        let d0 = self.r_001bc0(0x60, d1);
        let d0 = low_word(d0, (d0 as u16).wrapping_sub(0x30));
        let d0 = self.r_00a040(d0, a6);
        self.write16(VELOCITY_X, d0 as u16);
        let (d0, d1, ..) = self.r_001a88(d0, d1, a6);
        let (_, d1, ..) = self.r_001aa2(d0, d1, a6);
        d1
    }

    /// `func_009E7C`, a step: down by the rate plus 0x30
    /// (`func_009E86`).
    pub fn r_009e7c(&mut self, regs: &mut Regs) {
        self.i_running(regs);
        regs.d1 = self.r_009e86(0x30, regs.d1, regs.a6);
        regs.unwound = false;
    }

    /// `func_009E86`: as `func_009E1C`, the rate plus `d0`.
    pub fn r_009e86(&mut self, d0: u32, d1: u32, a6: u32) -> u32 {
        self.i_rate_y(d0, d1, a6)
    }

    /// The task's rate plus `d0` (`func_00A040`) its y velocity, and it
    /// moved by it (`func_001AA2`).  Gives back d1.
    fn i_rate_y(&mut self, d0: u32, d1: u32, a6: u32) -> u32 {
        let d0 = self.r_00a040(d0, a6);
        self.write16(VELOCITY_Y, d0 as u16);
        let (_, d1, ..) = self.r_001aa2(d0, d1, a6);
        d1
    }

    /// The task moved by x velocity `velocity` (`func_001A88`).
    fn i_move_x(&mut self, r: &mut Regs, velocity: u16) {
        self.write16(VELOCITY_X, velocity);
        (r.d0, r.d1, ..) = self.r_001a88(r.d0, r.d1, r.a6);
    }

    /// `func_009E94`, a step: down 1.5 pixels a frame, and sideways at
    /// random (`func_009E60`).
    pub fn r_009e94(&mut self, regs: &mut Regs) {
        self.i_running(regs);
        self.write16(VELOCITY_Y, 0x180);
        regs.d1 = self.r_009e60(regs.d1, regs.a6);
        regs.unwound = false;
    }

    /// `func_009EA4`, a step: right 4 and up 4 pixels a frame.
    pub fn r_009ea4(&mut self, regs: &mut Regs) {
        self.i_running(regs);
        self.i_move_both(regs, 0x400, 0xFC00);
    }

    /// The task moved by velocity (`x`, `y`).
    fn i_move_both(&mut self, r: &mut Regs, x: u16, y: u16) {
        self.write16(VELOCITY_X, x);
        self.write16(VELOCITY_Y, y);
        let (d0, d1, ..) = self.r_001a88(r.d0, r.d1, r.a6);
        (_, r.d1, ..) = self.r_001aa2(d0, d1, r.a6);
        r.unwound = false;
    }

    /// `func_009EC0`, the second task: moved left 4 pixels a frame, its
    /// sprite placed (`func_00A030`); when its count runs out, 0x3C
    /// frames more of `SECOND_TURN`.
    pub fn r_009ec0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.i_move_x(r, 0xFC00);
        (r.d0, r.x, r.c) = self.r_00a030(0, r.a5, r.a6);
        if self.i_count_down(r) {
            (r.x, r.c) = self.r_00053e(r.d6, SECOND_TURN);
            self.write8(r.a6 + TASK_COUNT, 0x3C);
            r.c = false;
        }
        r.unwound = false;
    }

    /// `func_009EEC`: an `rts`.
    pub fn r_009eec(&mut self) {}

    /// `func_009EEE`, the second task: when its count runs out, 0x80
    /// frames of `SECOND_LEAVE` and the kind's move (`SECOND_MOVES`).
    pub fn r_009eee(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        if !self.i_count_down(r) {
            r.unwound = false;
            return;
        }
        self.write8(r.a6 + TASK_COUNT, 0x80);
        (r.x, r.c) = self.r_00053e(r.d6, SECOND_LEAVE);
        let offset = self.i_by_kind(r);
        let routine = self.read32(SECOND_MOVES + offset);
        r.a0 = routine;
        r.unwound = false;
        let mut called = Regs { ..*r };
        self.call_family_1(routine, &mut called);
        (r.d6, r.a6, r.x, r.c, r.unwound) = (called.d6, called.a6, called.x, called.c, called.unwound);
    }

    /// `func_009F30`, the second task: the kind's step (`SECOND_STEPS`)
    /// and its sprite placed (`func_00A030`) until its count runs out,
    /// then it ends (`func_00121A`).
    pub fn r_009f30(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        if self.i_count_down(r) {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            r.unwound = false;
            return;
        }
        let offset = self.i_by_kind(r);
        let routine = self.read32(SECOND_STEPS + offset);
        r.a0 = routine;
        r.unwound = false;
        let mut called = Regs { ..*r };
        self.call_family_17(routine, &mut called);
        (r.d1, r.d6, r.a6, r.unwound) = (called.d1, called.d6, called.a6, called.unwound);
        if r.unwound {
            return;
        }
        (r.d0, r.x, r.c) = self.r_00a030(0, r.a5, r.a6);
    }

    /// `func_009F70`, a step: left 2 pixels a frame, and down by the
    /// rate less 0x20 (`func_009E1C`).
    pub fn r_009f70(&mut self, regs: &mut Regs) {
        self.i_running(regs);
        self.i_move_x(regs, 0xFE00);
        regs.d1 = self.r_009e1c(regs.d0, regs.d1, regs.a6);
        regs.unwound = false;
    }

    /// `func_009F86`, a step: left 2 pixels a frame, and down by the
    /// rate plus 0x30 (`func_009E40`).
    pub fn r_009f86(&mut self, regs: &mut Regs) {
        self.i_running(regs);
        self.i_move_x(regs, 0xFE00);
        regs.d1 = self.r_009e40(regs.d0, regs.d1, regs.a6);
        regs.unwound = false;
    }

    /// `func_009F9C`, a step: down by the rate plus 0x28
    /// (`func_009E86`).
    pub fn r_009f9c(&mut self, regs: &mut Regs) {
        self.i_running(regs);
        regs.d1 = self.r_009e86(0x28, regs.d1, regs.a6);
        regs.unwound = false;
    }

    /// `func_009FAA`, a step: left 4 and down 4 pixels a frame.
    pub fn r_009faa(&mut self, regs: &mut Regs) {
        self.i_running(regs);
        self.i_move_both(regs, 0xFC00, 0x400);
    }

    /// `func_009FC6`: the first task's sprites placed (`func_00A030`):
    /// its own, frame 0x15; but for the `BARE` kind it is frame 0xF,
    /// and the two more (`TASK_SPRITE_2`, `TASK_SPRITE_3`, 0x10 and 0x30
    /// to the right) frames 0x10 + `KIND` and 0x10 + `KIND_2`.  Gives
    /// back d0, d5, a5 (the last record's), X and C.
    pub fn r_009fc6(&mut self, d5: u32, a5: u32, a6: u32) -> (u32, u32, u32, bool, bool) {
        let (d0, x, _) = self.r_00a030(0, a5, a6);
        self.write8(a5 + SPRITE_FLAGS, FLAGS_SHOWN);
        self.write8(a5 + SPRITE_FRAME, 0x15);
        let kind = self.read8(KIND);
        if kind == BARE {
            return (d0, d5, a5, x, false);
        }
        self.write8(a5 + SPRITE_FRAME, 0x0F);
        let sprite = self.read8(a6 + TASK_SPRITE_2);
        let (d5, a5, _) = self.r_000b5a(low_byte(d5, sprite));
        self.write8(a5 + SPRITE_FLAGS, FLAGS_SHOWN);
        let frame = 0x10u8.wrapping_add(self.read8(KIND));
        self.write8(a5 + SPRITE_FRAME, frame);
        self.r_00a030(0x10, a5, a6);
        let sprite = self.read8(a6 + TASK_SPRITE_3);
        let (d5, a5, _) = self.r_000b5a(low_byte(d5, sprite));
        self.write8(a5 + SPRITE_FLAGS, FLAGS_SHOWN);
        let frame = 0x10u8.wrapping_add(self.read8(KIND_2));
        self.write8(a5 + SPRITE_FRAME, frame);
        let (d0, x, c) = self.r_00a030(0x30, a5, a6);
        (d0, d5, a5, x, c)
    }

    /// `func_009ACA`: the countdown's task made (`func_0003E8`), 1
    /// frame to count; `STOPPED` set, `CLEARED_ON_COUNTDOWN` cleared.
    /// Gives back d6, a6 (the task's), X and C (clear).
    pub fn r_009aca(&mut self, d4: u32, d5: u32, d6: u32) -> (u32, u32, bool, bool) {
        self.write8(CLEARED_ON_COUNTDOWN, 0);
        let (d6, a6, x, _) = self.r_0003e8(d4, d5, d6, COUNTDOWN_PRIORITY, COUNTDOWN);
        self.write8(a6 + TASK_COUNT, 1);
        self.write8(STOPPED, 1);
        (d6, a6, x, false)
    }

    /// `func_009AEC`, the countdown: when its count runs out it ends
    /// (`func_000560`), `ENDED` set and `func_00A63A` with `ON_END`.
    pub fn r_009aec(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        if self.i_count_down(r) {
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
            self.write16(ENDED, 0xFFFF);
            r.d0 = ON_END;
            (r.d0, r.a1, r.x, r.c) = self.r_00a63a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        }
        r.unwound = false;
    }
}
