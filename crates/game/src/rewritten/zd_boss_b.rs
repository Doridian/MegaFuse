//! 0x1D806-0x1DC7C: a second boss set up (`func_01D806`,
//! `func_01D80E`): its state bytes (from `STATE`) cleared, its graphics loaded
//! (`func_0094D8`), four falling specks (`func_01D96E`, `func_01D9BE`),
//! object 6 (the boss, `func_01E12A` on, `zd_boss_d.rs`) and object 7
//! (`func_01DC7C`, `zd_boss_c.rs`), and three tasks that follow the
//! boss: one that waits for its count (`COUNT`) to get to 0xD0 and then
//! stays at it (`func_01DA30`, `func_01DA48`), one that shows an
//! animation at it until then (`func_01DA7C`), and one that shows the
//! animations `func_01E374` and `func_01E38C` ask for (`func_01DB0C`,
//! `func_01DB7E`) and, when the boss is beaten, makes five tasks
//! (`func_01DBB0`, `func_01DC12`, `func_01DC60`) whose animations set
//! bit 1 of `STATE` when they end (`func_01DC46` ends the task that made
//! them after 8 frames).  The tasks bob with the boss's animation step
//! (`func_01DAAE`, `func_01DAE0`, `OFFSETS`).
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read.

use crate::{Game, Regs};

/// An object's fields: its sprite record's index (byte 0xF set to
/// 0x21), its hits, direction, the animation's step and the offset.
const OBJ_0F: u32 = 0x0F;
const OBJ_HITS: u32 = 0x11;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_STEP: u32 = 0x25;
const OBJ_OFFSET: u32 = 0x2A;
/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
/// A task's fields: its position (words: x, y; their low bytes), the
/// animation's timer and step, a speck's fraction of x (a byte; the
/// word cleared) and velocity (a word), a count (byte 0x18), and the
/// animation (a long) whose high byte is also the offset.
const TASK_X: u32 = 0x0E;
const TASK_X_HIGH: u32 = 0x0E;
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y: u32 = 0x10;
const TASK_Y_HIGH: u32 = 0x10;
const TASK_Y_LOW: u32 = 0x11;
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_FRACTION: u32 = 0x16;
const TASK_VELOCITY: u32 = 0x18;
const TASK_COUNT: u32 = 0x18;
const TASK_ANIM: u32 = 0x20;
const TASK_OFFSET: u32 = 0x20;

/// The running task's index (`task.rs`).
const CURRENT_TASK: u32 = 0xFF80EE;
/// What `func_0012A0` sets an object up with: its animations (a table
/// of longs) and its routine.
const OBJ_INIT_ANIMS: u32 = 0xFF80C8;
const OBJ_INIT_ROUTINE: u32 = 0xFF80D0;
/// A position (words: x then y; `func_001542`, `func_00152A`); the
/// high byte of y.
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
const POSITION_Y_HIGH: u32 = 0xFF9F06;
/// The list `func_0094D8` loads (a long).
const LIST: u32 = 0xFF92FA;
/// A long `func_0094D8` changes, kept around it.
const KEPT_5810: u32 = 0xFF5810;
/// The running object (index, address) and the sprite record of the
/// routine being run (index, address).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;

/// The boss's state: even values pick its routine (`func_01E16E`);
/// 0x84 ends the specks and the following task, bit 7 set when it is
/// beaten (the tasks of `func_01DBB0` then set bit 1).  The first of
/// the 0x2C bytes `func_01D80E` clears.
const STATE: u32 = 0xFFAEEE;
const CLEARED: u32 = 0x2C;
/// Set by the boss's tasks each frame they run (`func_01E570` ends
/// the boss once none has).
const TASKS_RAN: u32 = 0xFFAEEF;
/// The boss's object index.
const BOSS_INDEX: u32 = 0xFFAEF0;
/// The boss's count of frames as it comes in (`func_01E12A`).
const COUNT: u32 = 0xFFAF00;
/// Set while the boss is hurt (`func_01E428`): the task of
/// `func_01DB0C` is put out of sight.
const HURT: u32 = 0xFFAF04;
/// Cleared at the set-up (`func_01E1CA`'s last pick, its flag).
const LAST_PICK: u32 = 0xFFAF06;
const PICK_2: u32 = 0xFFAF12;
/// The animation `func_01DB0C` shows (1 or 3: `func_01DB7E` takes
/// longs at (it - 1) x 2 from `SHOWN_ANIMS`), 0 when none; and its
/// count of frames (the Z80 gets `SHOWN_SOUND` at 0xE0 and 0).
const SHOWN: u32 = 0xFFAF07;
const SHOWN_COUNT: u32 = 0xFFAF08;
/// A counter: the specks left to make; the place of the next of
/// `func_01DBB0`'s tasks (by 2).
const LEFT: u32 = 0xFFAF15;

/// The boss's and object 7's indexes, the boss's hits.
const BOSS: u32 = 6;
const OBJECT_7: u32 = 7;
const BOSS_HITS: u8 = 6;
/// The sprite set (`func_000776`) and list (`func_0094D8`) of the
/// boss's graphics.
const SET: u8 = 1;
const SET_FRAMES: u32 = 0x0EAD2C;
const BOSS_LIST: u32 = 0x01D95E;
/// The objects' animations and routines.
const BOSS_ANIMS: u32 = 0x01D7F6;
const ROUTINE_01E12A: u32 = 0x01E12A;
const ROUTINE_01DC7C: u32 = 0x01DC7C;
/// The boss's start (words).
const BOSS_X: u16 = 0x88;
const BOSS_Y: u16 = 0xFFA8;
/// The tasks: routines, depths and priorities, the first's position
/// (words) and frame, the others' animations.
const ROUTINE_01D9BE: u32 = 0x01D9BE;
const ROUTINE_01DA30: u32 = 0x01DA30;
const ROUTINE_01DA48: u32 = 0x01DA48;
const ROUTINE_01DA7C: u32 = 0x01DA7C;
const ROUTINE_01DB0C: u32 = 0x01DB0C;
const ROUTINE_01DBB0: u32 = 0x01DBB0;
const ROUTINE_01DC46: u32 = 0x01DC46;
const ROUTINE_01DC60: u32 = 0x01DC60;
const SPECK_DEPTH: u8 = 2;
const SPECK_PRIORITY: u8 = 0x42;
const SPECKS: u8 = 4;
const SPECK_FRAME: u8 = 0x1F;
const WAITER_DEPTH: u8 = 0xFE;
const TASK_DEPTH: u8 = 1;
const TASK_PRIORITY: u8 = 0x48;
const WAITER_X: u16 = 0x88;
const WAITER_Y: u16 = 0x38;
const WAITER_FRAME: u8 = 9;
const FLAGS: u8 = 0x81;
const ANIM_EB4C0: u32 = 0x0EB4C0;
const ANIM_EB4C6: u32 = 0x0EB4C6;
const ANIM_EB536: u32 = 0x0EB536;
const OBJECT_7_FRAME: u8 = 0xC;
/// The Z80's command at the set-up.
const START_SOUND: u8 = 0xF;
/// `func_01DA7C`: its count's end and the frame shown just before.
const COMING_END: u8 = 0xD0;
const COMING_LAST: u8 = 0xCC;
const COMING_FRAME: u8 = 0xB;
/// `func_01DB0C`: its depth, its frame with no animation, the y's high
/// byte while the boss is hurt.
const SHOWN_DEPTH: u8 = 5;
const SHOWN_IDLE: u8 = 0x15;
const HURT_Y_HIGH: u8 = 2;
const SHOWN_ANIMS: u32 = 0x01DBA8;
const SHOWN_SOUND: u8 = 0x6E;
/// The offsets by the boss's animation step / 2 (bytes).
const OFFSETS: u32 = 0x01DADA;
/// `func_01DC12`'s steps of `POSITION` (words, by `LEFT`: x, y) and
/// the last place.
const STEPS_X: u32 = 0x01DC32;
const STEPS_Y: u32 = 0x01DC3C;
const PLACES_END: u8 = 0xA;
/// `func_01DC46` ends after this many frames.
const MAKER_FRAMES: u8 = 8;
/// What ends the specks and the following task.
const STATE_ENDING: u8 = 0x84;
/// `func_01DC60`'s bit of `STATE`.
const STATE_BIT_1: u8 = 2;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// The running task ends (`func_000890`, `func_00121A`) on the
    /// family's registers.
    fn zd_end_task(&mut self, r: &mut Regs) {
        self.r_000890();
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
    }

    /// `func_001222` on the family's registers: a task of `routine`.
    fn zd_task(&mut self, r: &mut Regs, depth: u8, priority: u8, routine: u32) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, priority, routine);
    }

    /// `func_0013D6` on the family's registers.
    fn zd_task_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `func_001B7E` on the family's registers.
    fn zd_show(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_001B94` on the family's registers.
    fn zd_show_offset(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
    }

    /// The object of `BOSS_INDEX` to d4 and a4, its position to
    /// `POSITION` (`func_001542`).
    fn zd_boss_position(&mut self, r: &mut Regs) {
        let index = self.read8(BOSS_INDEX);
        (r.d4, r.a4, r.x) = self.r_001cfa(low_byte(r.d4, index));
        self.r_001542(r.a4);
    }

    /// `func_01D806`, the boss's set-up as a task's routine
    /// (`func_01D80E`).
    pub fn r_01d806(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_01d80e(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `func_01D80E`: the boss set up: `CLEARED` bytes from `STATE`
    /// cleared, sprite set `SET` (`func_000776`), the list `BOSS_LIST`
    /// loaded (`func_0094D8`, `KEPT_5810` kept around it), the palette
    /// flag (`func_0094B2`), the specks (`func_01D96E`); object 6 (its
    /// routine `func_01E12A`) at (0x88, -0x58), placed, `BOSS_HITS`
    /// hits, its index to `BOSS_INDEX`; the tasks `func_01DA30` (at
    /// 0x88, 0x38, frame 9), `func_01DA7C` and `func_01DB0C`; object 7
    /// (`func_01DC7C`, frame 0xC); `STATE`, `COUNT`, `HURT`, `PICK_2`
    /// and `LAST_PICK` cleared, `TASKS_RAN` 1, and the Z80 gets
    /// `START_SOUND`.  Gives back every register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01d80e(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        (r.d0, r.d1, r.a0, r.x) = self.r_0011d2(CLEARED, r.d1, STATE);
        r.a0 = SET_FRAMES;
        r.d0 = self.r_000776(low_byte(r.d0, SET), r.a0, r.a1);
        let kept = self.read32(KEPT_5810);
        self.write32(LIST, BOSS_LIST);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_none() {
            self.write32(KEPT_5810, kept);
            self.r_0094b2();
            (r.d0, r.d1, r.d5, r.d6, r.a5) = self.r_01d96e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            self.write16(POSITION_X, BOSS_X);
            self.write16(POSITION_Y, BOSS_Y);
            self.write32(OBJ_INIT_ANIMS, BOSS_ANIMS);
            self.write32(OBJ_INIT_ROUTINE, ROUTINE_01E12A);
            (r.d4, r.a4, r.x) = self.r_001cfa(BOSS);
            (r.d0, r.d5, r.a5, r.x) = self.r_0012a0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
            self.write8(r.a5 + SPRITE_FLAGS, FLAGS);
            self.write8(r.a4 + OBJ_0F, 0x21);
            self.write8(r.a4 + OBJ_DIRECTION, 0);
            (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
            (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
            (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
            self.write8(r.a4 + OBJ_HITS, BOSS_HITS);
            self.write8(BOSS_INDEX, r.d4 as u8);
            self.zd_task(&mut r, WAITER_DEPTH, TASK_PRIORITY, ROUTINE_01DA30);
            self.write16(r.a6 + TASK_X, WAITER_X);
            self.write16(r.a6 + TASK_Y, WAITER_Y);
            self.write8(r.a5 + SPRITE_FLAGS, FLAGS);
            self.write8(r.a5 + SPRITE_FRAME, WAITER_FRAME);
            self.zd_show(&mut r);
            for (routine, anim) in [(ROUTINE_01DA7C, ANIM_EB4C0), (ROUTINE_01DB0C, ANIM_EB4C6)] {
                self.zd_task(&mut r, TASK_DEPTH, TASK_PRIORITY, routine);
                self.write8(r.a5 + SPRITE_FLAGS, FLAGS);
                self.write32(r.a6 + TASK_ANIM, anim);
                self.zd_task_animate(&mut r);
            }
            self.write32(OBJ_INIT_ROUTINE, ROUTINE_01DC7C);
            (r.d4, r.a4, r.x) = self.r_001cfa(OBJECT_7);
            (r.d0, r.d5, r.a5, r.x) = self.r_0012a0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
            self.write8(r.a5 + SPRITE_FLAGS, FLAGS);
            self.write8(r.a4 + OBJ_0F, 0x21);
            self.write8(r.a5 + SPRITE_FRAME, OBJECT_7_FRAME);
            for at in [STATE, COUNT, HURT, PICK_2, LAST_PICK] {
                self.write8(at, 0);
            }
            self.write8(TASKS_RAN, 1);
            r.d0 = low_byte(r.d0, START_SOUND);
            r.c = false;
            self.r_0034b6(r.d0);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_01D96E`: `SPECKS` tasks of `func_01D9BE` (`func_001222`),
    /// each at a random x (low byte, high 0) and y (low byte, high 1),
    /// fraction and velocity 0, flags 0x81, frame 0x1F.  Gives back d0,
    /// d1, d5, d6 and a5.
    #[allow(clippy::too_many_arguments)]
    pub fn r_01d96e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, ..Regs::default() };
        self.write8(LEFT, SPECKS);
        loop {
            self.zd_task(&mut r, SPECK_DEPTH, SPECK_PRIORITY, ROUTINE_01D9BE);
            (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
            self.write8(r.a6 + TASK_X_LOW, r.d0 as u8);
            self.write8(r.a6 + TASK_X_HIGH, 0);
            (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
            self.write8(r.a6 + TASK_Y_LOW, r.d0 as u8);
            self.write8(r.a6 + TASK_Y_HIGH, 1);
            self.write16(r.a6 + TASK_VELOCITY, 0);
            self.write16(r.a6 + TASK_FRACTION, 0);
            self.write8(r.a5 + SPRITE_FLAGS, FLAGS);
            self.write8(r.a5 + SPRITE_FRAME, SPECK_FRAME);
            let left = self.read8(LEFT).wrapping_sub(1);
            self.write8(LEFT, left);
            if left == 0 {
                return (r.d0, r.d1, r.d5, r.d6, r.a5);
            }
        }
    }

    /// `func_01D9BE`, a speck's routine (`TASKS_RAN` set): its velocity
    /// changes by a random -0x10 to 0x10 (`func_001BC0`), x (with its
    /// fraction, 8 bits) moves by the velocity / 0x100, and it falls a
    /// pixel.  Once y's high byte is past 0 (and positive) it goes back
    /// up (y's high byte 0xFF) at a random x and y (velocity 0), unless
    /// `STATE` is `STATE_ENDING`: then the task ends.  Placed
    /// (`func_001B7E`).
    pub fn r_01d9be(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(TASKS_RAN, 1);
        r.d0 = self.r_001bc0(low_byte(r.d0, 0x20), r.d1);
        let change = (r.d0 as u8).wrapping_sub(0x10) as i8 as i16 as u16;
        let velocity = self.read16(r.a6 + TASK_VELOCITY).wrapping_add(change);
        self.write16(r.a6 + TASK_VELOCITY, velocity);
        let wide = i32::from(velocity as i16) as u32;
        let fraction = self.read8(r.a6 + TASK_FRACTION);
        let sum = fraction.wrapping_add(wide as u8);
        self.write8(r.a6 + TASK_FRACTION, sum);
        let carry = sum < fraction;
        r.d0 = wide.rotate_right(8);
        let x = self.read16(r.a6 + TASK_X).wrapping_add(r.d0 as u16).wrapping_add(u16::from(carry));
        r.d1 = low_word(r.d1, x);
        self.write16(r.a6 + TASK_X, x);
        let y = self.read16(r.a6 + TASK_Y);
        r.x = y == 0xFFFF;
        self.write16(r.a6 + TASK_Y, y.wrapping_add(1));
        r.c = false;
        let high = self.read8(r.a6 + TASK_Y_HIGH) as i8;
        if high > 0 {
            self.write8(r.a6 + TASK_Y_HIGH, 0xFF);
            if self.read8(STATE) == STATE_ENDING {
                self.zd_end_task(r);
                return;
            }
            (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
            self.write8(r.a6 + TASK_X_LOW, r.d0 as u8);
            self.write8(r.a6 + TASK_X_HIGH, 0);
            (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
            self.write8(r.a6 + TASK_Y_LOW, r.d0 as u8);
            self.write16(r.a6 + TASK_VELOCITY, 0);
        }
        self.zd_show(r);
    }

    /// `func_01DA30`, a task's routine: once `COUNT` is `COMING_END` it
    /// goes on as `func_01DA48`.
    pub fn r_01da30(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(COUNT);
        if count == COMING_END {
            (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_01DA48);
        } else {
            r.c = count < COMING_END;
        }
    }

    /// `func_01DA48`, a task's routine (`TASKS_RAN` set): it ends once
    /// `STATE` is `STATE_ENDING`; else it is put at the boss and placed
    /// (`func_001B7E`).
    pub fn r_01da48(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(TASKS_RAN, 1);
        if self.read8(STATE) == STATE_ENDING {
            self.zd_end_task(r);
            return;
        }
        self.zd_boss_position(r);
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
        self.r_00152a(r.a6);
        self.zd_show(r);
    }

    /// `func_01DA7C`, a task's routine: at the boss (`func_01DAAE`),
    /// placed with its offset; frame `COMING_FRAME` at `COUNT`
    /// `COMING_LAST`; it ends at `COMING_END`.
    pub fn r_01da7c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d4, r.d5, r.d6, r.a4, r.a5, r.a6) = self.r_01daae(r.d4, r.d5, r.d6);
        self.r_00152a(r.a6);
        self.zd_show_offset(r);
        let count = self.read8(COUNT);
        r.d0 = low_byte(r.d0, count);
        if count == COMING_END {
            self.zd_end_task(r);
            return;
        }
        if count == COMING_LAST {
            self.write8(r.a5 + SPRITE_FRAME, COMING_FRAME);
            r.c = false;
        } else {
            r.c = count < COMING_LAST;
        }
    }

    /// `func_01DAAE`: the boss's position to `POSITION`, the running
    /// task's offset from `OFFSETS` by the boss's animation step / 2;
    /// d5 and a5 the slot's sprite record.  Gives back d4, d5, d6, a4,
    /// a5 and a6.
    pub fn r_01daae(&mut self, d4: u32, d5: u32, d6: u32) -> (u32, u32, u32, u32, u32, u32) {
        let index = self.read8(BOSS_INDEX);
        let (d4, a4, _) = self.r_001cfa(low_byte(d4, index));
        self.r_001542(a4);
        let step = self.read8(a4 + OBJ_STEP);
        let current = self.read8(CURRENT_TASK);
        let (d6, a6, _) = self.r_0005f8(low_byte(d6, current));
        let offset = self.read8(OFFSETS + u32::from(step >> 1));
        self.write8(a6 + TASK_OFFSET, offset);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        (d4, d5, d6, a4, a5, a6)
    }

    /// `func_01DAE0`: the boss's position to `POSITION`, the running
    /// object's (d4, a4) offset from `OFFSETS` by the boss's animation
    /// step / 2; d5 and a5 the slot's sprite record.  Gives back d4,
    /// d5, a4 and a5.
    pub fn r_01dae0(&mut self, d4: u32, d5: u32) -> (u32, u32, u32, u32) {
        let index = self.read8(BOSS_INDEX);
        let (d4, a4, _) = self.r_001cfa(low_byte(d4, index));
        self.r_001542(a4);
        let step = self.read8(a4 + OBJ_STEP);
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        let offset = self.read8(OFFSETS + u32::from(step >> 1));
        self.write8(a4 + OBJ_OFFSET, offset);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        (d4, d5, a4, a5)
    }

    /// `func_01DB0C`, a task's routine (depth 5, `func_0007D2`): while
    /// the boss is `HURT` it is put out of sight (y's high byte 2).
    /// Else it shows `SHOWN`'s animation (`func_01DB7E`) until that
    /// ends, then frame `SHOWN_IDLE` (`SHOWN` cleared); once the boss
    /// is beaten (`STATE` negative) it goes on as `func_01DBB0`.  It
    /// stays at the boss (`func_01DAAE`), placed with its offset.
    pub fn r_01db0c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = low_byte(r.d0, SHOWN_DEPTH);
        r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
        if self.read8(HURT) != 0 {
            self.r_00153a(r.a6);
            self.write8(POSITION_Y_HIGH, HURT_Y_HIGH);
            self.r_00152a(r.a6);
            self.zd_show_offset(r);
            return;
        }
        let shown = self.read8(SHOWN);
        r.d0 = low_byte(r.d0, shown);
        let mut idle = true;
        if shown != 0 {
            self.r_01db7e(r.d0, r.a6);
            self.zd_task_animate(r);
            if r.c {
                self.write8(SHOWN, 0);
            } else {
                idle = false;
            }
        }
        if idle {
            self.write8(r.a5 + SPRITE_FRAME, SHOWN_IDLE);
            self.write8(r.a6 + TASK_TIMER, 0);
            self.write8(r.a6 + TASK_STEP, 0);
            self.write8(SHOWN, 0);
            r.c = false;
            if (self.read8(STATE) as i8) < 0 {
                (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_01DBB0);
            }
        }
        (r.d4, r.d5, r.d6, r.a4, r.a5, r.a6) = self.r_01daae(r.d4, r.d5, r.d6);
        self.r_00152a(r.a6);
        self.zd_show_offset(r);
    }

    /// `func_01DB7E`: the task's animation the long at (`shown` - 1) x 2
    /// in `SHOWN_ANIMS` (the table's longs are 4 apart: `shown` 1 and 3
    /// pick its two); `SHOWN_COUNT` down, the Z80 getting `SHOWN_SOUND`
    /// at 0 and 0xE0.
    pub fn r_01db7e(&mut self, d0: u32, a6: u32) {
        let at = (d0 as u8).wrapping_sub(1).wrapping_mul(2) as i8 as i16 as i32 as u32;
        let anim = self.read32(SHOWN_ANIMS.wrapping_add(at));
        self.write32(a6 + TASK_ANIM, anim);
        let count = self.read8(SHOWN_COUNT).wrapping_sub(1);
        self.write8(SHOWN_COUNT, count);
        if count == 0 || count == 0xE0 {
            self.r_0034b6(low_byte(d0, SHOWN_SOUND));
        }
    }

    /// `func_01DBB0`, a task's routine: on as `func_01DC46` (its count
    /// 0), and five tasks of `func_01DC60` (`func_001222`) made at the
    /// boss's position plus the steps of `func_01DC12`, flags 0x81,
    /// animation 0xEB536 started and placed.
    pub fn r_01dbb0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_01DC46);
        self.write8(r.a6 + TASK_COUNT, 0);
        self.write8(LEFT, 0);
        loop {
            self.zd_boss_position(r);
            r.d0 = self.r_01dc12(r.d0);
            self.zd_task(r, TASK_DEPTH, TASK_PRIORITY, ROUTINE_01DC60);
            self.r_00152a(r.a6);
            self.write8(r.a5 + SPRITE_FLAGS, FLAGS);
            self.write32(r.a6 + TASK_ANIM, ANIM_EB536);
            self.zd_task_animate(r);
            self.zd_show(r);
            let was = self.read8(LEFT);
            let place = was.wrapping_add(2);
            self.write8(LEFT, place);
            r.x = place < was;
            if place == PLACES_END {
                r.c = false;
                return;
            }
        }
    }

    /// `func_01DC12`: `POSITION` moved by the words at `LEFT` (a byte
    /// offset) in `STEPS_X` and `STEPS_Y`.  d1 is kept.  Gives back d0.
    pub fn r_01dc12(&mut self, d0: u32) -> u32 {
        let place = self.read8(LEFT) as i8 as i16 as u16;
        let d0 = low_word(d0, place);
        let at = i32::from(place as i16) as u32;
        let x = self.read16(POSITION_X).wrapping_add(self.read16(STEPS_X.wrapping_add(at)));
        self.write16(POSITION_X, x);
        let y = self.read16(POSITION_Y).wrapping_add(self.read16(STEPS_Y.wrapping_add(at)));
        self.write16(POSITION_Y, y);
        d0
    }

    /// `func_01DC46`, a task's routine: placed with its offset for
    /// `MAKER_FRAMES` frames (its count), then it ends.
    pub fn r_01dc46(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let was = self.read8(r.a6 + TASK_COUNT);
        let count = was.wrapping_add(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = count < was;
        if count >= MAKER_FRAMES {
            self.zd_end_task(r);
        } else {
            self.zd_show_offset(r);
        }
    }

    /// `func_01DC60`, a task's routine: placed with its offset, its
    /// animation a frame on; when it ends, bit 1 of `STATE` is set and
    /// the task ends.
    pub fn r_01dc60(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zd_show_offset(r);
        self.zd_task_animate(r);
        if r.c {
            let state = self.read8(STATE);
            self.write8(STATE, state | STATE_BIT_1);
            self.zd_end_task(r);
        }
    }
}
