//! A scene, from 0x15C52 (`func_015C52`, called at the end of
//! `func_01816C`'s run): its set-up, two VRAM copies on the copy tasks'
//! turns (`func_015DDE`), a fade (`func_015E6E`), and a task that flies
//! along a curve (`func_015ED0`) driven by a script of commands in the
//! ROM (`func_015FAE`, its reader `func_015F54` and the like).

use super::t_common::*;
use crate::{Game, Regs};

/// The scene's tasks (code addresses).
const COPIES: u32 = 0x015DDE;
const FADE: u32 = 0x015E6E;
const FADED: u32 = 0x015E8E;
const AFTER_FADE: u32 = 0x015EAC;
const COPY_TASK: u32 = 0x00B8C0;
const TASK_161C0: u32 = 0x0161C0;
const FLYER: u32 = 0x015EB4;
const TASK_1613C: u32 = 0x01613C;

/// The list of things to load (`i_load.rs`) and the scene's, in the
/// ROM (`D_015D94`).
const LOAD_LIST: u32 = 0xFFFF_92FA;
const SCENE_LIST: u32 = 0x015D94;
/// The sprite set's frames' table (in the ROM) for set 0
/// (`func_000776`).
const SET_FRAMES: u32 = 0x0D867C;
/// The script `FLYER` starts with (in the ROM, `D_016FFA`) and the
/// copy task's table of sources (`D_015DD0`).
const SCRIPT_START: u32 = 0x016FFA;
const COPY_TABLE: u32 = 0x015DD0;
/// Animations in the ROM.
const ANIM_1613C: u32 = 0x0D8F9A;
const ANIM_FADED: u32 = 0x016A8C;
/// The commands' table (longs, in the ROM, after `func_015FAE`).
const COMMANDS: u32 = 0x016004;

/// The script command that loops: a count and an address.
const LOOP: u8 = 0x11;

/// The VRAM copy (`k_vdp.rs`): asked for, under way, to, from, length.
const COPY_ASKED: u32 = 0xFFFF_A944;
const COPY_ON: u32 = 0xFFFF_A945;
const COPY_TO: u32 = 0xFFFF_A946;
const COPY_FROM: u32 = 0xFFFF_A948;
const COPY_LENGTH: u32 = 0xFFFF_A94A;
/// The count whose low 2 bits say whose turn it is (`k_vdp.rs`).
const TURNS: u32 = 0xFFFF_A94C;
/// The copies `COPIES` still makes: bit 0 to 0xC000 on turn 1, bit 1
/// to 0xD000 on turn 2.
const COPIES_LEFT: u32 = 0xFFFF_AD79;
/// A copy task's fields (`k_vdp.rs`): its table, where to and how
/// long, its wait and delay, its step and steps, its turn.
const COPY_TABLE_FIELD: u32 = 0x0C;
const COPY_TO_FIELD: u32 = 0x10;
const COPY_LENGTH_FIELD: u32 = 0x12;
const COPY_WAIT: u32 = 0x14;
const COPY_DELAY: u32 = 0x15;
const COPY_STEP: u32 = 0x16;
const COPY_STEPS: u32 = 0x17;
const COPY_TURN: u32 = 0x18;

/// The objects stopped (`a_objects.rs`) and the fade's lines
/// (`j_colours.rs`).
const STOPPED: u32 = 0xFFFF_A3FC;
const FADE_LINES: u32 = 0xFFFF_A3F9;
/// The fade's kind (`func_00A63A`'s d0).
const FADE_KIND: u32 = 4;
/// A byte `FADED` sets to 0x20.
const BYTE_AD7B: u32 = 0xFFFF_AD7B;
/// The view's y offset's low byte (`h_scroll.rs`).
const OFFSET_Y_LOW: u32 = 0xFFFF_9EB9;
/// A word cleared here.
const WORD_ADC4: u32 = 0xFFFF_ADC4;
/// Words set to 0x10 and 0x150 at the end.
const WORD_80C8: u32 = 0xFFFF_80C8;
const WORD_80CC: u32 = 0xFFFF_80CC;
/// The flags of the game (`MODE` in `t_common.rs`), set to 1 here.
const MODE_START: u8 = 1;

/// `FLYER`'s index (a byte).
const FLYER_INDEX: u32 = 0xFFFF_AD81;
/// `FLYER`'s curve: its speed (a word, into `SPEED`; not below 0) and
/// the byte added to it each frame; its angle (a word; the high byte
/// to `func_00410C`), the word added to it and the byte added to that;
/// a phase (its high nibble picks the frame from `FIRST_FRAME`), the
/// byte added to it and the byte added to that.
const CURVE_SPEED: u32 = 0xFFFF_AD6E;
const CURVE_SPEED_STEP: u32 = 0xFFFF_AD74;
const CURVE_ANGLE: u32 = 0xFFFF_AD70;
const CURVE_TURN: u32 = 0xFFFF_AD72;
const CURVE_TURN_STEP: u32 = 0xFFFF_AD75;
const CURVE_PHASE: u32 = 0xFFFF_AD76;
const CURVE_PHASE_RATE: u32 = 0xFFFF_AD77;
const CURVE_PHASE_STEP: u32 = 0xFFFF_AD78;
const FIRST_FRAME: u8 = 0x33;
/// The y (its low byte) from which `FLYER`'s sprite is behind
/// (flags 0x80).
const BEHIND_Y: u8 = 0xA0;
/// The script: where it is (a long), the frames to wait before its
/// next commands, and the times a `LOOP` has gone back.
const SCRIPT: u32 = 0xFFFF_AD5E;
const SCRIPT_WAIT: u32 = 0xFFFF_AD7F;
const SCRIPT_LOOPS: u32 = 0xFFFF_AD7E;

impl Game {
    /// `func_015C52`: the scene set up: the step stopped, the picture
    /// off and VRAM cleared from 0xE000, the tasks and sprites cleared
    /// (`func_00033A`, `func_0003DA`, `func_000704`), `SCENE_LIST`
    /// loaded, sprite set 0, the window (`func_00B4E4`), the objects
    /// (`func_000E3E`); then the tasks `COPIES`, `FADE`, a copy task
    /// (`COPY_TABLE`), `func_016726`, `TASK_161C0`, `FLYER` (at
    /// 0x180, -0x40, its script `SCRIPT_START` from the next frame) and
    /// `TASK_1613C`, sound 0x1D, and the flags (`func_0094B2`).  Gives
    /// back every register, X and C.
    /// Differs in the check as its translation does (the loading runs
    /// for frames, the vertical blank's handler meanwhile).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_015c52(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.write8(STEP_RUNS, 0);
        self.write8(COPY_ON, 0);
        self.r_0002e0();
        self.r_00033a();
        self.r_0002e6();
        let mut r = Regs { d2, d3, d4, d5, d7, a1, a2, a3, a4, a5, ..Regs::default() };
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(d2, d3, d4, d5, d7, a1, a2, a3, a4, a5);
        (r.d0, r.d1, r.a0, r.x) = self.r_000704(r.d0);
        self.write32(LOAD_LIST, SCENE_LIST);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
        }
        r.a0 = SET_FRAMES;
        r.d0 = low_word(r.d0, 0);
        r.d0 = self.r_000776(r.d0, r.a0, r.a1);
        self.r_00b4e4();
        (r.d0, r.d1, r.d6, r.a6, r.x) = self.r_000e3e(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.write8(MODE, MODE_START);
        self.write8(OFFSET_Y_LOW, 0x80);
        self.write16(WORD_ADC4, 0);
        self.write8(COPIES_LEFT, 3);
        self.t_make_bare_task(&mut r, 0x40, COPIES);
        self.t_make_bare_task(&mut r, 0x40, FADE);
        self.t_make_bare_task(&mut r, 0x20, COPY_TASK);
        let copy = r.a6;
        self.write16(copy + COPY_TO_FIELD, 0x1900);
        self.write16(copy + COPY_LENGTH_FIELD, 0xFF);
        self.write32(copy + COPY_TABLE_FIELD, COPY_TABLE);
        self.write8(copy + COPY_DELAY, 2);
        self.write8(copy + COPY_STEPS, 7);
        self.write8(copy + COPY_STEP, 0);
        self.write8(copy + COPY_WAIT, 0);
        self.write8(copy + COPY_TURN, 0);
        (r.d0, r.d1, r.d5, r.d6, r.a0, r.a5) = self.r_016726(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.t_make_bare_task(&mut r, 0x40, TASK_161C0);
        self.t_make_task(&mut r, 0x80, 0x40, FLYER);
        self.write8(FLYER_INDEX, r.d6 as u8);
        self.write8(r.a5 + SPRITE_FLAGS, 0);
        self.write16(r.a6 + TASK_X, 0x180);
        self.write16(r.a6 + TASK_Y, 0xFFC0);
        self.write32(SCRIPT, SCRIPT_START);
        self.write8(SCRIPT_WAIT, 1);
        self.t_make_task(&mut r, 0x80, 0x41, TASK_1613C);
        self.write32(r.a6 + TASK_ANIM, ANIM_1613C);
        self.write8(r.a5 + SPRITE_FLAGS, 0x40);
        self.write8(r.a6 + TASK_COUNT, 0);
        self.t_sound(&mut r, 0x1D);
        self.write16(WORD_80C8, 0x10);
        self.write16(WORD_80CC, 0x150);
        r.c = self.r_0094b2();
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_015DDE`, `COPIES`: on turn 1 (`TURNS`' low 2 bits) with
    /// `COPIES_LEFT`' bit 0, a copy of 0x6C0 words from 0x6000 to
    /// 0xC000 asked for (interrupts masked meanwhile), `COPY_ON` set,
    /// the bit cleared; else on turn 2 with bit 1, the same to 0xD000.
    /// Gives back d0 and C (X as it was: the status register put back).
    pub fn r_015dde(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        for (bit, turn, to) in [(0u8, 1u8, 0xC000u16), (1, 2, 0xD000)] {
            if self.read8(COPIES_LEFT) & (1 << bit) == 0 {
                continue;
            }
            let t = self.read8(TURNS) & 3;
            r.d0 = low_byte(r.d0, t);
            r.c = t < turn;
            if t != turn {
                continue;
            }
            let ipl = self.ipl();
            self.set_ipl(ipl | 7);
            self.write16(COPY_TO, to);
            self.write16(COPY_FROM, 0x6000);
            self.write16(COPY_LENGTH, 0x6C0);
            self.write8(COPY_ASKED, 1);
            self.set_ipl(ipl);
            self.write8(COPY_ON, 1);
            r.c = false;
            let left = self.read8(COPIES_LEFT);
            self.write8(COPIES_LEFT, left & !(1 << bit));
            return;
        }
    }

    /// `func_015E6E`, `FADE`: `FADED` from the next frame, the objects
    /// stopped, the fade on every line, and the fade `FADE_KIND`
    /// (`func_00A63A`).
    pub fn r_015e6e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.t_next_task(r, FADED);
        self.write8(STOPPED, 1);
        self.write8(FADE_LINES, 0xFF);
        r.d0 = FADE_KIND;
        (r.d0, r.a1, r.x, r.c) = self.r_00a63a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_015E8E`, `FADED`: nothing while the objects are stopped
    /// (`func_001BB4`); then `BYTE_AD7B` 0x20, `ANIM_FADED`, and
    /// `AFTER_FADE` (at once).
    pub fn r_015e8e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        self.t_next_task(r, AFTER_FADE);
        self.write8(BYTE_AD7B, 0x20);
        self.write32(r.a6 + TASK_ANIM, ANIM_FADED);
        r.c = false;
        self.r_015eac(r);
    }

    /// `func_015EAC`, `AFTER_FADE`: `func_016880`.
    pub fn r_015eac(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d6, r.a6, r.x, r.c) = self.r_016880(r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a6, r.x);
    }

    /// `func_015EB4`, `FLYER`: its curve (`func_015ED0`), the script
    /// (`func_015FAE`); then d6, a6, d5 and a5 back to the task.
    pub fn r_015eb4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2) = self.r_015ed0(r.d0, r.d1, r.d2, r.a5, r.a6);
        (r.d0, r.d1, r.d2, r.d3, r.d5, r.d6, r.a0) =
            self.r_015fae(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        self.t_reload_task(r);
        r.c = false;
    }

    /// `func_015ED0`: a step of the curve: the speed (not below 0, into
    /// `SPEED`), the angle and the phase stepped; the frame
    /// `FIRST_FRAME` plus the phase / 16; the task moved at the angle
    /// (`func_00410C`); its sprite behind from `BEHIND_Y` down, placed
    /// (`func_015F46`).  Gives back d0, d1 and d2.
    pub fn r_015ed0(&mut self, d0: u32, d1: u32, d2: u32, a5: u32, a6: u32) -> (u32, u32, u32) {
        let step = self.read8(CURVE_SPEED_STEP) as i8 as u16;
        let speed = step.wrapping_add(self.read16(CURVE_SPEED));
        self.write16(CURVE_SPEED, speed);
        self.write16(SPEED, speed);
        if (speed as i16) < 0 {
            self.write16(SPEED, 0);
            self.write16(CURVE_SPEED, 0);
        }
        let step = self.read8(CURVE_TURN_STEP) as i8 as u16;
        let turn = step.wrapping_add(self.read16(CURVE_TURN));
        self.write16(CURVE_TURN, turn);
        let angle = self.read16(CURVE_ANGLE).wrapping_add(turn);
        self.write16(CURVE_ANGLE, angle);
        let rate = self.read8(CURVE_PHASE_STEP).wrapping_add(self.read8(CURVE_PHASE_RATE));
        self.write8(CURVE_PHASE_RATE, rate);
        let phase = rate.wrapping_add(self.read8(CURVE_PHASE));
        self.write8(CURVE_PHASE, phase);
        let (frame, x) = (phase >> 4).overflowing_add(FIRST_FRAME);
        self.write8(a5 + SPRITE_FRAME, frame);
        let d0 = low_byte(d0, self.read8(CURVE_ANGLE));
        let (d0, d1, d2, _) = self.r_00410c(d0, d1, d2, x);
        let (d0, d1, ..) = self.r_001a88(d0, d1, a6);
        let (d0, d1, ..) = self.r_001aa2(d0, d1, a6);
        self.write8(a5 + SPRITE_FLAGS, 0);
        let y = self.read8(a6 + TASK_Y_LOW);
        if y >= BEHIND_Y {
            self.write8(a5 + SPRITE_FLAGS, 0x80);
        }
        self.r_015f46(a5, a6);
        (low_byte(d0, y), d1, d2)
    }

    /// `func_015F46`: the sprite at the task's position.  Gives back C
    /// (clear).
    pub fn r_015f46(&mut self, a5: u32, a6: u32) -> bool {
        let x = self.read16(a6 + TASK_X);
        self.write16(a5 + SPRITE_X, x);
        let y = self.read16(a6 + TASK_Y);
        self.write16(a5 + SPRITE_Y, y);
        false
    }

    /// `func_015F54`: the script's next byte (into d0's low byte),
    /// `SCRIPT` past it.  Gives back d0, a0 (`SCRIPT`), N and Z (the
    /// byte's).
    pub fn r_015f54(&mut self, d0: u32) -> (u32, u32, bool, bool) {
        let at = self.read32(SCRIPT);
        let byte = self.read8(at);
        let a0 = at.wrapping_add(1);
        self.write32(SCRIPT, a0);
        (low_byte(d0, byte), a0, (byte as i8) < 0, byte == 0)
    }

    /// `func_015F62`: the script's next word (two bytes, high first)
    /// into d0's low word; d1 is kept.  Gives back d0 and a0.
    pub fn r_015f62(&mut self, d0: u32, _d1: u32) -> (u32, u32) {
        let (d0, _, _, _) = self.r_015f54(d0);
        let high = d0 as u8;
        let (d0, a0, _, _) = self.r_015f54(d0);
        let word = u16::from(high) << 8 | u16::from(d0 as u8);
        (low_word(d0, word), a0)
    }

    /// `func_015F80`: the script's next long (two words, high first)
    /// into d0; d1 is kept.  Gives back d0 and a0.
    pub fn r_015f80(&mut self, d0: u32, d1: u32) -> (u32, u32) {
        let (d0, _) = self.r_015f62(d0, d1);
        let high = d0 & 0xFFFF;
        let (d0, a0) = self.r_015f62(d0, d1);
        (high << 16 | (d0 & 0xFFFF), a0)
    }

    /// `func_015FAE`: the script: after `SCRIPT_WAIT`'s frames, its
    /// commands to the next 0 byte (and a byte after it skipped) or a
    /// negative one (left there): `LOOP` (a count, an address: goes
    /// back to the address the count's times, then on), or a command
    /// of `COMMANDS` by its number (each its arguments read from the
    /// script).  Gives back d0, d1, d2, d3, d5, d6 and a0.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_015fae(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        let wait = self.read8(SCRIPT_WAIT);
        if wait != 0 {
            self.write8(SCRIPT_WAIT, wait - 1);
            return (d0, d1, d2, d3, d5, d6, a0);
        }
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        loop {
            let (negative, zero);
            (r.d0, r.a0, negative, zero) = self.r_015f54(r.d0);
            if zero {
                let at = self.read32(SCRIPT).wrapping_add(1);
                self.write32(SCRIPT, at);
                break;
            }
            if negative {
                break;
            }
            let command = r.d0 as u8;
            if command == LOOP {
                (r.d0, r.a0, _, _) = self.r_015f54(r.d0);
                let loops = self.read8(SCRIPT_LOOPS);
                if r.d0 as u8 == loops {
                    self.write8(SCRIPT_LOOPS, 0);
                    (r.d0, r.a0) = self.r_015f80(r.d0, r.d1);
                } else {
                    self.write8(SCRIPT_LOOPS, loops.wrapping_add(1));
                    (r.d0, r.a0) = self.r_015f80(r.d0, r.d1);
                    self.write32(SCRIPT, r.d0);
                }
                break;
            }
            let offset = u16::from(command) * 4;
            r.d0 = low_word(r.d0, offset);
            r.a0 = self.read32(COMMANDS + u32::from(offset));
            (r.x, r.n, r.z, r.v, r.c) = (false, false, false, false, false);
            r.unwound = false;
            self.call_family_0(r.a0, &mut r);
            if r.unwound {
                break;
            }
        }
        (r.d0, r.d1, r.d2, r.d3, r.d5, r.d6, r.a0)
    }
}
