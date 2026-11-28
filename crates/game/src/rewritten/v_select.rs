//! A screen of choices (0x0174BA, 0x01793A-0x018000): `func_01797A`
//! and `func_0179B0` start its task, `func_017B5A` loads the picture
//! of the choice, the pad steps the choice (`func_0179EA`'s
//! continuation, `loc_017A5C`) and a sprite turns to it
//! (`func_017B92`, `func_017B46`, `func_017B50`); `func_017E12` sets
//! up the screen for the last choice, `func_017DBE` slides a sprite in,
//! and the continuations `loc_017D16`, `loc_017D4C`, `loc_017D62`,
//! `loc_017EDA` go on to the next part once the objects run.

use crate::{Game, Regs};

/// The pad's byte read: bit 0 or 2 steps the choice back, 1 or 3 on,
/// bit 7 takes it.
const PAD: u32 = 0xFFFF_AD02;
const BACK: u8 = 0x05;
const ON: u8 = 0x0A;
const TAKE: u8 = 0x80;
/// The choice and the number of choices less one (the choice goes
/// from 0 to it and around), and the choice compared with the taken
/// one (`TAKEN`).
const CHOICE: u32 = 0xFFFF_953A;
const LAST: u32 = 0xFFFF_953B;
const TAKEN: u32 = 0xFFFF_ADD0;
/// Bits whose low nibble `func_017952` tests (kept in `SCRATCH`, bit 3
/// with them when `CHOICE` is 0).
const BITS: u32 = 0xFFFF_953D;
const SCRATCH: u32 = 0xFFFF_80C5;
/// Cleared when the choice is stepped from one that isn't 0.
const BYTE_ADCE: u32 = 0xFFFF_ADCE;
/// Set to 0xFF when the choice is taken.
const BYTE_954E: u32 = 0xFFFF_954E;
/// Cleared before the tasks are started again (`RUNNING` of
/// `i_main.rs`).
const RUNNING: u32 = 0xFFFF_80DF;
/// The argument of `func_0184A8` and `func_0185EC` (a byte).
const ARGUMENT: u32 = 0xFFFF_80C4;
/// The list of things to load (`i_load.rs`), and the entry
/// `func_017B5A` makes for it: the item (by choice, D_017B8C), a byte
/// 7, words 0x8400 and 0xC0, and the end 0xFF.
const LIST: u32 = 0xFFFF_92FA;
const ENTRY: u32 = 0xFFFF_ADDA;
const ENTRY_7: u32 = 0xFFFF_ADDB;
const ENTRY_8400: u32 = 0xFFFF_ADDC;
const ENTRY_C0: u32 = 0xFFFF_ADDE;
const ENTRY_END: u32 = 0xFFFF_ADE0;
const ITEMS: u32 = 0x017B8C;
/// The list `func_017E12` loads.
const LIST_187E0: u32 = 0x0187E0;

/// A task's fields: its sprite record, a turn (a byte: 0x40 and 0xC0
/// change the animation), a byte set to 0x80 when the choice is taken,
/// the frames it waits (a word, `func_01793A`), its animation (a
/// long).
const TASK_SPRITE: u32 = 0x03;
const TURN: u32 = 0x0C;
const TASK_0D: u32 = 0x0D;
const WAIT: u32 = 0x1A;
const ANIM: u32 = 0x20;
/// The turn for each choice (D_018824).
const TURNS: u32 = 0x018824;
const ANIM_DE1FC: u32 = 0x0DE1FC;
const ANIM_DE1F6: u32 = 0x0DE1F6;
/// A sprite record's frame and the low byte of its x.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_X_LOW: u32 = 0x09;

/// The running task's index.
const CURRENT_TASK: u32 = 0xFFFF_80EE;

/// The routines started, with their priorities: the screen's
/// (`func_0179EA`; `func_017C2A` when `LAST` is `LAST_5`), by `LAST`
/// (D_0179CE), the sparks (`v_sparks.rs`), after `loc_017D62`, and the
/// task's own once the choice is taken.
const ROUTINE_0179EA: u32 = 0x0179EA;
const ROUTINE_017C2A: u32 = 0x017C2A;
const LAST_5: u8 = 5;
const BY_LAST: u32 = 0x0179CE;
const BY_LAST_PRIORITY: u8 = 0x9B;
const SCREEN_PRIORITY: u8 = 0x96;
const ROUTINE_0175A4: u32 = 0x0175A4;
const ROUTINE_018876: u32 = 0x018876;
const PRIORITY_32: u8 = 0x32;
const ROUTINE_017B92: u32 = 0x017B92;
/// The Z80's commands (`func_0034B6`): the choice stepped, taken, and
/// taken again.
const STEPPED: u8 = 0x20;
const TAKEN_SOUND: u8 = 0x21;
const TAKEN_OTHER: u8 = 0x79;
/// The colours `func_0174BA` loads (words from colour 0x10 / 2).
const COLOUR_AT: u32 = 0x10;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_001BB4` on the family's registers: true if the objects
    /// are stopped (the caller returns).
    fn v_stopped(&mut self, r: &mut Regs) -> bool {
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        stopped
    }

    /// `func_017B5A` on the family's registers.
    fn v_load_choice(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_017b5a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
    }

    /// `loc_017D20` on the family's registers.
    fn v_go_on(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d6, r.a0, r.a1, r.a6, r.x, r.c) = self.r_017d20(r.d0, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// The way from `CHOICE` to `TAKEN` (d0's low byte): their
    /// difference (made positive, plus 2, when below 0) less 1, and 2.
    /// Gives back X (the less 1's borrow) and whether it is set.
    fn v_way(&mut self, r: &mut Regs) -> bool {
        let mut way = self.read8(CHOICE).wrapping_sub(self.read8(TAKEN));
        if way & 0x80 != 0 {
            way = (!way).wrapping_add(3);
        }
        r.x = way == 0;
        way = way.wrapping_sub(1) & 2;
        r.d0 = low_byte(r.d0, way);
        way != 0
    }

    /// `func_0174BA`, a routine of a list: the colours from the address
    /// `func_015F80` gives (a count, words) to colour `COLOUR_AT` / 2
    /// on, and `func_0094B2`.
    pub fn r_0174ba(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.a0) = self.r_015f80(r.d0, r.d1);
        r.a0 = r.d0;
        let count = self.read16(r.a0);
        r.a0 = r.a0.wrapping_add(2);
        r.d0 = low_word(r.d0, count);
        r.d1 = COLOUR_AT;
        r.x = self.r_00946e(r.d0, r.d1, r.a0);
        r.c = self.r_0094b2();
    }

    /// `func_01793A`: the running task's `WAIT` (`func_0005F8`); while
    /// it isn't 0 it goes down and the caller returns too.  Gives back
    /// d6, a6, X, C and that.
    pub fn r_01793a(&mut self, d6: u32) -> (u32, u32, bool, bool, bool) {
        let d6 = low_byte(d6, self.read8(CURRENT_TASK));
        let (d6, a6, x) = self.r_0005f8(d6);
        let wait = self.read16(a6 + WAIT);
        if wait == 0 {
            return (d6, a6, x, false, false);
        }
        self.write16(a6 + WAIT, wait - 1);
        (d6, a6, false, false, true)
    }

    /// `func_017952`: `BITS` (with bit 3 when `CHOICE` is 0) in
    /// `SCRATCH`; gives back d0 (its low nibble) and Z: all four set.
    pub fn r_017952(&mut self, d0: u32) -> (u32, bool) {
        let bits = self.read8(BITS);
        self.write8(SCRATCH, bits);
        let mut d0 = d0;
        if self.read8(CHOICE) == 0 {
            d0 = low_byte(d0, 8);
            let scratch = self.read8(SCRATCH);
            self.write8(SCRATCH, scratch | 8);
        }
        let nibble = self.read8(SCRATCH) & 0xF;
        (low_byte(d0, nibble), nibble == 0xF)
    }

    /// `func_01797A` (`loc_01797A`): `RUNNING` cleared, the tasks
    /// cleared (`func_0003DA`) and the screen's task started
    /// (`func_0179EA`, or `func_017C2A` when `LAST` is `LAST_5`).
    /// Gives back d0, d1, d6, a0, a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01797a(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        self.write8(RUNNING, 0);
        let (d0, d1, d6, a0, _) = self.r_0003da(d2, d3, d4, d5, d7, a1, a2, a3, a4, a5);
        let routine = if self.read8(LAST) == LAST_5 { ROUTINE_017C2A } else { ROUTINE_0179EA };
        let (d6, a6, x, c) = self.r_0003e8(d4, d5, d6, SCREEN_PRIORITY, routine);
        (d0, d1, d6, a0, a6, x, c)
    }

    /// `func_0179B0`: `RUNNING` cleared, the tasks cleared
    /// (`func_0003DA`) and a task of the routine for `LAST` (`BY_LAST`)
    /// started.  Gives back d0, d1, d6, a0, a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0179b0(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool, bool) {
        self.write8(RUNNING, 0);
        let (d0, d1, d6, _, _) = self.r_0003da(d2, d3, d4, d5, d7, a1, a2, a3, a4, a5);
        let offset = self.read8(LAST).wrapping_mul(4);
        let d0 = low_word(d0, offset as i8 as i16 as u16);
        let a0 = self.read32(BY_LAST.wrapping_add(offset as i8 as u32));
        let d0 = low_byte(d0, BY_LAST_PRIORITY);
        let (d6, a6, x, c) = self.r_000404(d0, d4, d5, d6, a0);
        (d0, d1, d6, a0, a6, x, c)
    }

    /// `loc_017A5C`, where `func_0179EA` goes on after its yield, each
    /// frame unless the objects are stopped: the pad steps `CHOICE`
    /// back or on (around from 0 to `LAST`), with `STEPPED` to the
    /// Z80 and the choice's picture loaded (`func_017B5A`).  When the
    /// pad takes it (`TAKEN_SOUND`): if it is `TAKEN`, on to
    /// `loc_017D20`; otherwise `TAKEN_OTHER`, `BYTE_954E` 0xFF, the
    /// task's byte 0x0D 0x80, its routine `func_017B92` from the next
    /// frame and its animation by the way to `TAKEN` and its `TURN`.
    pub fn r_017a5c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.v_stopped(r) {
            return;
        }
        let pad = self.read8(PAD);
        let mut stepped = false;
        if pad & BACK != 0 {
            r.d0 = low_byte(r.d0, pad & BACK);
            if self.read8(CHOICE) != 0 {
                self.write8(BYTE_ADCE, 0);
            }
            let choice = self.read8(CHOICE);
            self.write8(CHOICE, choice.wrapping_sub(1));
            r.x = choice == 0;
            if choice.wrapping_sub(1) & 0x80 != 0 {
                let last = self.read8(LAST);
                self.write8(CHOICE, last);
            }
            stepped = true;
        } else if pad & ON != 0 {
            if self.read8(CHOICE) != 0 {
                self.write8(BYTE_ADCE, 0);
            }
            let choice = self.read8(CHOICE).wrapping_add(1);
            self.write8(CHOICE, choice);
            let less = choice.wrapping_sub(1);
            r.d0 = low_byte(r.d0, less);
            r.x = choice == 0;
            if less >= self.read8(LAST) {
                self.write8(CHOICE, 0);
            }
            stepped = true;
        }
        if stepped {
            r.d0 = low_byte(r.d0, STEPPED);
            self.r_0034b6(r.d0);
            self.v_load_choice(r);
            if self.restarted.is_some() {
                return;
            }
        }
        let take = self.read8(PAD) & TAKE;
        r.d0 = low_byte(r.d0, take);
        r.c = false;
        if take == 0 {
            return;
        }
        r.d0 = low_byte(r.d0, TAKEN_SOUND);
        self.r_0034b6(r.d0);
        r.d0 = low_byte(r.d0, self.read8(TAKEN));
        if r.d0 as u8 == self.read8(CHOICE) {
            self.v_go_on(r);
            return;
        }
        r.d0 = low_byte(r.d0, TAKEN_OTHER);
        self.r_0034b6(r.d0);
        (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_017B92);
        self.write8(BYTE_954E, 0xFF);
        self.write8(r.a6 + TASK_0D, 0x80);
        let way = self.v_way(r);
        let low = self.read8(r.a6 + TURN).wrapping_sub(0x40) & 0x80 != 0;
        r.c = if way == low { self.r_017b50(r.a6) } else { self.r_017b46(r.a6) };
    }

    /// `func_017B46`: the task's animation `ANIM_DE1FC`.  Gives back C
    /// (clear).
    pub fn r_017b46(&mut self, a6: u32) -> bool {
        self.write32(a6 + ANIM, ANIM_DE1FC);
        false
    }

    /// `func_017B50`: the task's animation `ANIM_DE1F6`.  Gives back C
    /// (clear).
    pub fn r_017b50(&mut self, a6: u32) -> bool {
        self.write32(a6 + ANIM, ANIM_DE1F6);
        false
    }

    /// `func_017B5A`: `CHOICE`'s item (`ITEMS`) loaded (`func_0094D8`,
    /// its entry made at `ENTRY`).  Gives back every register (d1's low
    /// word the choice) and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_017b5a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let choice = self.read8(CHOICE) as i8;
        let d1 = low_word(d1, choice as i16 as u16);
        let item = self.read8(ITEMS.wrapping_add(choice as u32));
        self.write8(ENTRY, item);
        self.write8(ENTRY_7, 7);
        self.write16(ENTRY_8400, 0x8400);
        self.write16(ENTRY_C0, 0xC0);
        self.write8(ENTRY_END, 0xFF);
        self.write32(LIST, ENTRY);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, _) =
            self.r_0094d8(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x);
        (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x)
    }

    /// `func_017B92`, a task's routine: its `TURN` goes on or back by
    /// the way to `TAKEN` (at 0x40 and 0xC0 its animation changes)
    /// until it is the choice's (`TURNS`): then on to `loc_017D20`;
    /// meanwhile its sprite is put by the turn (`func_018564`) and its
    /// animation run (`func_0013D6`).
    pub fn r_017b92(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let on = self.v_way(r);
        let was = self.read8(r.a6 + TURN);
        let turn = if on { was.wrapping_add(1) } else { was.wrapping_sub(1) };
        self.write8(r.a6 + TURN, turn);
        r.x = if on { turn == 0 } else { was == 0 };
        r.d0 = low_byte(r.d0, turn);
        if turn == 0x40 {
            r.c = self.r_017b46(r.a6);
        } else if turn == 0xC0 {
            r.c = self.r_017b50(r.a6);
        }
        let choice = self.read8(CHOICE) as i8;
        r.d1 = low_word(r.d1, choice as i16 as u16);
        let turn = self.read8(r.a6 + TURN);
        r.d0 = low_byte(r.d0, turn);
        r.a0 = TURNS;
        let target = self.read8(TURNS.wrapping_add(choice as u32));
        if turn == target {
            self.v_go_on(r);
            return;
        }
        r.c = turn < target;
        self.r_018564(r);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `loc_017D16`, a task's continuation: unless the objects are
    /// stopped, `loc_0098F8`.
    pub fn r_017d16(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.v_stopped(r) {
            return;
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0098f8(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `loc_017D4C`, a task's continuation: unless the objects are
    /// stopped, `func_01884A`.
    pub fn r_017d4c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.v_stopped(r) {
            return;
        }
        (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) = self.r_01884a(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `loc_017D62`, a task's continuation: unless the objects are
    /// stopped, the tasks cleared (`func_0003DA`) and a task of
    /// `func_018876` started.
    pub fn r_017d62(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.v_stopped(r) {
            return;
        }
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, PRIORITY_32, ROUTINE_018876);
    }

    /// `loc_017DBE`, a task's continuation: the task `func_0184A8`
    /// gives (`ARGUMENT` 0x96, d0 0) has its sprite's x come down a
    /// pixel a frame to its place by its `TURN` and byte 0x0D
    /// (`func_017770`), its animation run (`func_0013D6`); there its
    /// sprite's frame is 0x0D and the running task ends.
    pub fn r_017dbe(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(ARGUMENT, 0x96);
        r.d0 = low_byte(r.d0, 0);
        (r.d0, r.d6, r.a6) = self.r_0184a8(r.d0, r.d6);
        r.d0 = low_byte(r.d0, self.read8(r.a6 + TURN));
        r.d1 = low_byte(r.d1, self.read8(r.a6 + TASK_0D));
        (r.d0, r.x) = self.r_017770(r.d0, r.d1, r.d3);
        r.d5 = low_byte(r.d5, self.read8(r.a6 + TASK_SPRITE));
        (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
        let x = self.read8(r.a5 + SPRITE_X_LOW);
        if r.d0 as u8 == x {
            self.write8(r.a5 + SPRITE_FRAME, 0x0D);
            r.c = false;
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
            return;
        }
        self.write8(r.a5 + SPRITE_X_LOW, x.wrapping_sub(1));
        self.write8(ARGUMENT, 0x96);
        r.d0 = low_byte(r.d0, 0);
        (r.d0, r.d6, r.a6) = self.r_0184a8(r.d0, r.d6);
        r.d5 = low_byte(r.d5, self.read8(r.a6 + TASK_SPRITE));
        (r.d5, r.a5, r.x) = self.r_000b5a(r.d5);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `func_017E12`: `CHOICE` is `LAST` less 1; `func_018772`, the list
    /// `LIST_187E0` loaded, `func_0185EC` with `ARGUMENT` the choice, a
    /// task of `func_0175A4` (the sparks, its count 0x10) and
    /// `func_0186B2`.  Gives back d0-d7 and a0-a5.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_017e12(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let last = self.read8(LAST);
        let d0 = low_byte(d0, last.wrapping_sub(1));
        self.write8(CHOICE, d0 as u8);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x) =
            self.r_018772(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, last == 0);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        }
        self.write32(LIST, LIST_187E0);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, _, _, _) =
            self.r_0094d8(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        }
        let choice = self.read8(CHOICE);
        self.write8(ARGUMENT, choice);
        let (d2, d5, d6, a1, a5) = self.r_0185ec(d1, d2, d3, d4, d5, d6, d7, a2, a3, a4, a5);
        let (d6, task, _, _) = self.r_0003e8(d4, d5, d6, PRIORITY_32, ROUTINE_0175A4);
        self.write8(task + TURN, 0x10);
        let (d0, d5, d6, a0, a5, _) = self.r_0186b2(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5)
    }

    /// `loc_017EDA`, a task's continuation: unless the objects are
    /// stopped, the screen again (`func_01797A`).
    pub fn r_017eda(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.v_stopped(r) {
            return;
        }
        (r.d0, r.d1, r.d6, r.a0, r.a6, r.x, r.c) = self.r_01797a(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }
}
