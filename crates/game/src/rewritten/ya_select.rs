//! The screens of choices' tasks, the parts that yield (0x0179EA,
//! 0x017C2A-0x018008): the screen of choices (`func_0179EA`), the
//! last screen's (`func_017C2A`), its way on when a choice is taken
//! (`loc_017D20`), and the routines of `D_0179CE` by the number of
//! choices: `func_017D6C` (the first), `func_017E58` and `func_017EE4`
//! (the last ones), each a run of steps a frame or more apart, every
//! step a routine of its own (the yield, `func_000522`, gives the next
//! one).  The steps that go on once the objects run (`v_select.rs`)
//! and the task routines they start (`v_sparks.rs`, `w_ring.rs`) are
//! elsewhere.

use crate::{Game, Regs};

/// The pad's byte read: bit 7 takes the choice.
const PAD: u32 = 0xFFFF_AD02;
const TAKE: u8 = 0x80;
/// The choice and the number of choices less one; `func_017952`'s
/// bits (cleared by `func_017D6C`).
const CHOICE: u32 = 0xFFFF_953A;
const LAST: u32 = 0xFFFF_953B;
const BITS: u32 = 0xFFFF_953D;
/// Cleared by `func_017C2A`.
const BYTE_953C: u32 = 0xFFFF_953C;
/// Set to 0xFF by `func_017C2A` (and by `loc_017A5C` when the choice is
/// taken).
const BYTE_954E: u32 = 0xFFFF_954E;
/// The argument of `func_0184A8` and `func_0185EC` (a byte).
const ARGUMENT: u32 = 0xFFFF_80C4;
/// The list of things to load (`i_load.rs`, `func_0094D8`) and the
/// lists these load.
const LIST: u32 = 0xFFFF_92FA;
const LIST_187DA: u32 = 0x0187DA;
const LIST_187E8: u32 = 0x0187E8;
const LIST_187F2: u32 = 0x0187F2;
/// Words `func_017C2A` sets before `func_008664`.
const WORD_D002: u32 = 0xFFFF_D002;
const WORD_D402: u32 = 0xFFFF_D402;
/// Where `func_0179EA` keeps its sprite record's index (`func_000B5A`'s
/// d5) and address.
const KEPT_INDEX: u32 = 0xFFFF_8A57;
const KEPT_RECORD: u32 = 0xFFFF_8A5A;
/// The running task's index.
const CURRENT_TASK: u32 = 0xFFFF_80EE;

/// A task's fields: its sprite record (a byte, `func_000B5A`), two
/// bytes (the sparks' count, a turn; a count down), the frames
/// `func_01793A` waits (a word), its animation (a long).
const TASK_SPRITE: u32 = 0x03;
const TASK_0C: u32 = 0x0C;
const TASK_0D: u32 = 0x0D;
const WAIT: u32 = 0x1A;
const ANIM: u32 = 0x20;
/// A sprite record's frame, a byte set to 0x81 and the low byte of
/// its x.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_05: u32 = 0x05;
const SPRITE_X_LOW: u32 = 0x09;
const ANIM_DE202: u32 = 0x0DE202;

/// The routines started and their priorities: the sparks
/// (`func_0175A4`), the turning sprite (`func_01868A`, made by
/// `func_001222`), the screen of choices (`func_0179EA`).
const ROUTINE_0175A4: u32 = 0x0175A4;
const ROUTINE_01868A: u32 = 0x01868A;
const ROUTINE_0179EA: u32 = 0x0179EA;
const PRIORITY_32: u8 = 0x32;
const SPRITE_KIND_B4: u8 = 0xB4;
const SCREEN_PRIORITY: u8 = 0x96;
/// The sparks' count.
const SPARKS: u8 = 0x10;

/// Where the tasks go on after each yield.
const AFTER_0179EA: u32 = 0x017A5C;
const AFTER_017C2A: u32 = 0x017CD6;
const AFTER_017CD6: u32 = 0x017CE6;
const AFTER_TAKEN: u32 = 0x017D16;
const AFTER_LAST: u32 = 0x017D4C;
const AFTER_OTHER: u32 = 0x017D62;
const AFTER_017D6C: u32 = 0x017D88;
const AFTER_017D88: u32 = 0x017DBE;
const AFTER_017E58: u32 = 0x017E70;
const AFTER_017E70: u32 = 0x017E8C;
const AFTER_017E8C: u32 = 0x017EC8;
const AFTER_017EC8: u32 = 0x017EDA;
const AFTER_017EE4: u32 = 0x017F1E;
const AFTER_017F1E: u32 = 0x017F3A;
const AFTER_017F3A: u32 = 0x017F70;
const AFTER_017F70: u32 = 0x017FE0;
const AFTER_017FE0: u32 = 0x017FF6;
const AFTER_017FF6: u32 = 0x018008;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// The yield (`func_000522`) to `resume`; the routine returns.
    pub(super) fn ya_yield(&mut self, r: &mut Regs, resume: u32) {
        (r.x, r.c, _) = self.r_000522(r.d6, resume);
    }

    /// `func_001BB4` on the family's registers: true if the objects
    /// are stopped (the caller returns).
    pub(super) fn ya_stopped(&mut self, r: &mut Regs) -> bool {
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        stopped
    }

    /// A family routine called on the registers: what it leaves in
    /// them but the N, Z and V flags.
    pub(super) fn ya_call(&mut self, r: &mut Regs, routine: fn(&mut Game, &mut Regs)) {
        let mut called = *r;
        called.unwound = false;
        routine(self, &mut called);
        let (n, z, v) = (r.n, r.z, r.v);
        *r = called;
        (r.n, r.z, r.v) = (n, z, v);
        r.unwound = false;
    }

    /// The Z80's command `command` (`func_0034B6`), in d0's low byte.
    pub(super) fn ya_sound(&mut self, r: &mut Regs, command: u8) {
        r.d0 = low_byte(r.d0, command);
        self.r_0034b6(r.d0);
    }

    /// The Z80's command 0xFE with d1 `d1` (`func_00349E`).
    pub(super) fn ya_command_fe(&mut self, r: &mut Regs, d1: u32) {
        r.d1 = d1;
        r.d0 = low_byte(r.d0, 0xFE);
        self.r_00349e(r.d0);
    }

    /// `func_00A51A` with d0 `d0` (a fade).
    pub(super) fn ya_fade(&mut self, r: &mut Regs, d0: u32) {
        r.d0 = d0;
        (r.d0, r.a1, r.x, r.c) = self.r_00a51a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_018772` on the registers; true if the game restarted in it.
    fn ya_018772(&mut self, r: &mut Regs) -> bool {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_018772(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        self.restarted.is_some()
    }

    /// The list `list` loaded (`func_0094D8`); true if the game
    /// restarted in it.
    fn ya_load(&mut self, r: &mut Regs, list: u32) -> bool {
        self.write32(LIST, list);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        self.restarted.is_some()
    }

    /// `func_017E12` on the registers; true if the game restarted in it.
    fn ya_017e12(&mut self, r: &mut Regs) -> bool {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5) =
            self.r_017e12(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        self.restarted.is_some()
    }

    /// A task of the sparks (`func_0175A4`, its count `SPARKS`).
    fn ya_sparks(&mut self, r: &mut Regs) {
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, PRIORITY_32, ROUTINE_0175A4);
        self.write8(r.a6 + TASK_0C, SPARKS);
    }

    /// The turning sprite's task (`func_001222` with `func_01868A`),
    /// its animation `ANIM_DE202`, its sprite's byte 5 0x81.
    fn ya_turning(&mut self, r: &mut Regs) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, PRIORITY_32, SPRITE_KIND_B4, ROUTINE_01868A);
        self.write32(r.a6 + ANIM, ANIM_DE202);
        self.write8(r.a5 + SPRITE_05, 0x81);
    }

    /// The running task's `WAIT` set to `frames` (the task from
    /// `func_0005F8`).
    fn ya_wait(&mut self, r: &mut Regs, frames: u16) {
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        self.write16(r.a6 + WAIT, frames);
    }

    /// `func_01793A` on the registers: true while the running task
    /// waits (the caller returns).
    fn ya_waiting(&mut self, r: &mut Regs) -> bool {
        let waiting;
        (r.d6, r.a6, r.x, r.c, waiting) = self.r_01793a(r.d6);
        waiting
    }

    /// `func_0184A8` with `ARGUMENT` `argument` and d0 `which`: a6 the
    /// task it gives.
    fn ya_task_of(&mut self, r: &mut Regs, argument: u8, which: u8) {
        self.write8(ARGUMENT, argument);
        r.d0 = low_byte(r.d0, which);
        (r.d0, r.d6, r.a6) = self.r_0184a8(r.d0, r.d6);
    }

    /// `func_0186E8` on the registers.
    fn ya_0186e8(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_0186e8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_0186B2` on the registers.
    fn ya_0186b2(&mut self, r: &mut Regs) {
        (r.d0, r.d5, r.d6, r.a0, r.a5, r.a6) =
            self.r_0186b2(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_01865E` on the registers.
    fn ya_01865e(&mut self, r: &mut Regs) {
        (r.d5, r.d6, r.a5, r.a6) =
            self.r_01865e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_0179EA`, the screen of choices' task: `func_018772`, the
    /// list `LIST_187DA` loaded, `func_0185EC` with `ARGUMENT` `LAST`,
    /// the Z80's command 2, `func_0186B2`, its sprite record's index and
    /// address kept (`KEPT_INDEX`, `KEPT_RECORD`), `func_0186E8`, the
    /// choice the last (`CHOICE` is `LAST`) and its picture loaded
    /// (`func_017B5A`), the sparks; it yields to `loc_017A5C`.
    pub fn r_0179ea(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.ya_018772(r) || self.ya_load(r, LIST_187DA) {
            return;
        }
        let last = self.read8(LAST);
        self.write8(ARGUMENT, last);
        (r.d2, r.d5, r.d6, r.a1, r.a5) = self.r_0185ec(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        self.ya_sound(r, 2);
        self.ya_0186b2(r);
        // d5 and a5 are saved around it (movem)
        let index = low_byte(r.d5, self.read8(r.a6 + TASK_SPRITE));
        let (index, record, x) = self.r_000b5a(index);
        r.x = x;
        self.write8(KEPT_INDEX, index as u8);
        self.write32(KEPT_RECORD, record);
        self.ya_0186e8(r);
        let last = self.read8(LAST);
        self.write8(CHOICE, last);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_017b5a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return;
        }
        self.ya_sparks(r);
        self.ya_yield(r, AFTER_0179EA);
    }

    /// `func_017C2A`, the last screen's task: `BYTE_954E` 0xFF, `CHOICE`
    /// 5, `BYTE_953C` cleared, `func_018772`, the lists `LIST_187F2` and
    /// `LIST_187E8` loaded, `WORD_D002` 0xFF80 and `WORD_D402` 0x80
    /// (`func_008664`), `func_00A506` with 4, `func_0185CC`,
    /// `func_01865E` (its task's byte 0x0D 0xAA), `func_0186B2` (its
    /// task's bytes 0x0C 0xDC and 0x0D 0x64), the turning sprite and the
    /// sparks; it yields to `func_017CD6`.
    pub fn r_017c2a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(BYTE_954E, 0xFF);
        self.write8(CHOICE, 5);
        self.write8(BYTE_953C, 0);
        if self.ya_018772(r) || self.ya_load(r, LIST_187F2) || self.ya_load(r, LIST_187E8) {
            return;
        }
        self.write16(WORD_D002, 0xFF80);
        self.write16(WORD_D402, 0x80);
        self.ya_call(r, Game::r_008664);
        r.d0 = 4;
        (r.d0, r.a1, r.x, r.c) = self.r_00a506(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        self.r_0185cc();
        self.ya_01865e(r);
        self.write8(r.a6 + TASK_0D, 0xAA);
        self.ya_0186b2(r);
        self.write8(r.a6 + TASK_0C, 0xDC);
        self.write8(r.a6 + TASK_0D, 0x64);
        self.ya_turning(r);
        self.ya_sparks(r);
        self.ya_yield(r, AFTER_017C2A);
    }

    /// `func_017CD6`, the last screen's task, the frame after: unless
    /// the objects are stopped, the Z80's command 3 and it yields to
    /// `func_017CE6`.
    pub fn r_017cd6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.ya_stopped(r) {
            return;
        }
        self.ya_sound(r, 3);
        self.ya_yield(r, AFTER_017CD6);
    }

    /// `func_017CE6`, the last screen's task each frame: until the pad
    /// takes it, the turning sprite's step (`func_018564`); then the
    /// Z80's commands 0x21 and 0xFE (d1 0x20), a fade (`func_00A51A`
    /// with 4) and it yields to `loc_017D16`.
    pub fn r_017ce6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let take = self.read8(PAD) & TAKE;
        r.d0 = low_byte(r.d0, take);
        r.c = false;
        if take == 0 {
            self.ya_call(r, Game::r_018564);
            return;
        }
        self.ya_sound(r, 0x21);
        self.ya_command_fe(r, 0x20);
        self.ya_fade(r, 4);
        self.ya_yield(r, AFTER_TAKEN);
    }

    /// `loc_017D20`, where a choice taken goes on: the Z80's command
    /// 0xFE (d1 0x20), a fade (`func_00A51A` with 4), and the task
    /// yields: to `loc_017D4C` when `CHOICE` is `LAST` or more and
    /// `func_017952` gives Z (its four bits set), to `loc_017D62`
    /// otherwise.  Gives back d0, d1, d6, a0, a1, a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_017d20(&mut self, d0: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        let d1 = 0x20;
        let d0 = low_byte(d0, 0xFE);
        self.r_00349e(d0);
        let d0 = low_byte(d0, self.read8(CHOICE));
        let mut resume = AFTER_OTHER;
        // d0 is 4 after: what func_017952 leaves in it doesn't matter
        if d0 as u8 >= self.read8(LAST) && self.r_017952(d0).1 {
            resume = AFTER_LAST;
        }
        let (d0, a1, _, _) = self.r_00a51a(4, d1, d2, d3, d4, d5, d6, d7, a2, a3, a4, a5);
        let (x, c, _) = self.r_000522(d6, resume);
        (d0, d1, d6, a0, a1, a6, x, c)
    }

    /// `func_017D6C` (`D_0179CE`'s first): `CHOICE` and `BITS`
    /// cleared, a task of the screen of choices (`func_0179EA`), and it
    /// yields to `func_017D88`.
    pub fn r_017d6c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(CHOICE, 0);
        self.write8(BITS, 0);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, SCREEN_PRIORITY, ROUTINE_0179EA);
        self.ya_yield(r, AFTER_017D6C);
    }

    /// `func_017D88`, the frame after: the task `func_0184A8` gives
    /// (`ARGUMENT` 0x96, d0 0) has its sprite's frame 0 and the low
    /// byte of its x 0xFF; unless the objects are stopped, its
    /// animation `func_017B50`'s, the Z80's command 0x79, and it yields
    /// to `loc_017DBE` (the sprite slides in).
    pub fn r_017d88(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.ya_task_of(r, 0x96, 0);
        r.d5 = low_byte(r.d5, self.read8(r.a6 + TASK_SPRITE));
        (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.write8(r.a5 + SPRITE_X_LOW, 0xFF);
        if self.ya_stopped(r) {
            return;
        }
        r.c = self.r_017b50(r.a6);
        self.ya_sound(r, 0x79);
        self.ya_yield(r, AFTER_017D88);
    }

    /// `func_017E58` (`D_0179CE`'s second to fifth): the screen for the
    /// last choice (`func_017E12`), the running task's `WAIT` 0x80, and
    /// it yields to `func_017E70`.
    pub fn r_017e58(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.ya_017e12(r) {
            return;
        }
        self.ya_wait(r, 0x80);
        self.ya_yield(r, AFTER_017E58);
    }

    /// `func_017E70`, each frame: `func_0186E8`; unless the objects are
    /// stopped or the task waits (`func_01793A`), the Z80's command
    /// 0x75 and it yields to `func_017E8C`.
    pub fn r_017e70(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.ya_0186e8(r);
        if self.ya_stopped(r) || self.ya_waiting(r) {
            return;
        }
        self.ya_sound(r, 0x75);
        self.ya_yield(r, AFTER_017E70);
    }

    /// `func_017E8C`, each frame: the task `func_0184A8` gives
    /// (`ARGUMENT` 0x64, d0 the choice) counts its byte 0x0D down with
    /// `func_0186E8` each frame; at 0, the Z80's command 0x76, the
    /// running task's `WAIT` 0x80, and it yields to `func_017EC8`.
    pub fn r_017e8c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let choice = self.read8(CHOICE);
        self.ya_task_of(r, 0x64, choice);
        let count = self.read8(r.a6 + TASK_0D);
        r.d0 = low_byte(r.d0, count);
        if count != 0 {
            self.write8(r.a6 + TASK_0D, count - 1);
            self.ya_0186e8(r);
            return;
        }
        self.ya_sound(r, 0x76);
        self.ya_wait(r, 0x80);
        self.ya_yield(r, AFTER_017E8C);
    }

    /// `func_017EC8`, each frame: once the task has waited
    /// (`func_01793A`), a fade (`func_00A51A` with 4) and it yields to
    /// `loc_017EDA`.
    pub fn r_017ec8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.ya_waiting(r) {
            return;
        }
        self.ya_fade(r, 4);
        self.ya_yield(r, AFTER_017EC8);
    }

    /// `func_017EE4` (`D_0179CE`'s sixth): the screen for the last
    /// choice (`func_017E12`), the turning sprite, the running task's
    /// `WAIT` 0x80, and it yields to `func_017F1E`.
    pub fn r_017ee4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.ya_017e12(r) {
            return;
        }
        self.ya_turning(r);
        self.ya_wait(r, 0x80);
        self.ya_yield(r, AFTER_017EE4);
    }

    /// `func_017F1E`, each frame: `func_0186E8`; unless the objects are
    /// stopped or the task waits (`func_01793A`), the Z80's command
    /// 0x75 and it yields to `func_017F3A`.
    pub fn r_017f1e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.ya_0186e8(r);
        if self.ya_stopped(r) || self.ya_waiting(r) {
            return;
        }
        self.ya_sound(r, 0x75);
        self.ya_yield(r, AFTER_017F1E);
    }

    /// `func_017F3A`, each frame: the task `func_0184A8` gives
    /// (`ARGUMENT` 0x64, d0 the choice) counts its byte 0x0D down with
    /// `func_0186E8` each frame; at 0, the Z80's command 0x77,
    /// `func_00A52E` with 3, and it yields to `func_017F70`.
    pub fn r_017f3a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let choice = self.read8(CHOICE);
        self.ya_task_of(r, 0x64, choice);
        let count = self.read8(r.a6 + TASK_0D);
        r.d0 = low_byte(r.d0, count);
        if count != 0 {
            self.write8(r.a6 + TASK_0D, count - 1);
            self.ya_0186e8(r);
            return;
        }
        self.ya_sound(r, 0x77);
        r.d0 = 3;
        (r.d0, r.d6, r.a0, r.a1, r.a6, r.x, r.c) =
            self.r_00a52e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a2, r.a3, r.a4, r.a5, r.a6);
        self.ya_yield(r, AFTER_017F3A);
    }

    /// `func_017F70`, the frame after: unless the objects are stopped,
    /// the sprite at a5 has frame 0, `func_00033A`, `func_000302`, the
    /// list `LIST_187F2` loaded, the five tasks `func_0184A8` gives
    /// (`ARGUMENT` 0x64, d0 0 to 4) end with their sprites
    /// (`func_000870`, `func_000564`), `func_01865E`, `func_00A542` with
    /// 3, `func_0185CC`, the running task's bytes 0x0C 0x4D and 0x0D
    /// 0xFF, and it yields to `func_017FE0`.
    pub fn r_017f70(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.ya_stopped(r) {
            return;
        }
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.r_00033a();
        (r.a0, r.x) = self.r_000302();
        if self.ya_load(r, LIST_187F2) {
            return;
        }
        r.d2 = low_byte(r.d2, 0);
        loop {
            let which = r.d2 as u8;
            self.ya_task_of(r, 0x64, which);
            r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_SPRITE));
            r.x = self.r_000870(r.d0, r.d5);
            (r.x, r.c) = self.r_000564(r.d5, r.d6, r.x);
            let next = (r.d2 as u8).wrapping_add(1);
            r.d2 = low_byte(r.d2, next);
            r.c = next < 5;
            if !r.c {
                break;
            }
        }
        self.ya_01865e(r);
        r.d0 = 3;
        (r.d0, r.d6, r.a0, r.a1, r.a6, r.x) =
            self.r_00a542(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a2, r.a3, r.a4, r.a5, r.a6);
        self.r_0185cc();
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        self.write8(r.a6 + TASK_0C, 0x4D);
        self.write8(r.a6 + TASK_0D, 0xFF);
        self.ya_yield(r, AFTER_017F70);
    }

    /// `func_017FE0`, the frame after: unless the objects are stopped,
    /// the running task's `WAIT` 0x80 and it yields to `func_017FF6`.
    pub fn r_017fe0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.ya_stopped(r) {
            return;
        }
        self.ya_wait(r, 0x80);
        self.ya_yield(r, AFTER_017FE0);
    }

    /// `func_017FF6`, each frame: once the task has waited
    /// (`func_01793A`), the Z80's command 0x7A and it yields to
    /// `loc_018008`.
    pub fn r_017ff6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.ya_waiting(r) {
            return;
        }
        self.ya_sound(r, 0x7A);
        self.ya_yield(r, AFTER_017FF6);
    }
}
