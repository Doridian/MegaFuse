//! A stage set up (`loc_0098F8`, `func_0099E6`): the tasks and objects
//! anew, the sprite sets and the lists of things to load by kind
//! (`i_load.rs`), the map, the objects placed (`func_009B0A`), and the
//! opening (`i_intro.rs`) or, in the `MODE` with bit 0, `func_00A506`.

use crate::{Game, Regs};

/// Flags: bit 0 a mode (the objects placed by `func_009B0A`'s other
/// way, other sets and lists, no opening), bit 6 set while
/// `func_001C96` runs once from `func_009B0A` (`b_control.rs`).
const MODE: u32 = 0xFFFF_9540;
const MODE_OTHER: u8 = 1 << 0;
const MODE_ONCE: u8 = 0x40;
/// A byte: how many objects `func_009B0A` places in that mode.
const PLACED: u32 = 0xFFFF_9541;
/// The stage's kind and a second byte (`i_intro.rs`), a count the
/// number `func_009966` works out is moved on by, and that number.
const KIND: u32 = 0xFFFF_953A;
const KIND_2: u32 = 0xFFFF_953C;
const STEP: u32 = 0xFFFF_953E;
const NUMBER: u32 = 0xFFFF_953F;
/// `func_009966`'s tables (bytes in the ROM): by kind, the start in
/// `NUMBERS`; the numbers.
const STARTS: u32 = 0x009984;
const NUMBERS: u32 = 0x00998C;

/// The sprite sets by kind (longs in the ROM, `D_0098CA`) for set 1.
const SETS_BY_KIND: u32 = 0x0098CA;
/// The sprite sets' tables (`func_000776`: frames, costs), and two
/// tables of frames given with the costs left in a1 from the call
/// before.
const SET_0: (u32, u32) = (0x0E_B6B4, 0x0E_CBC0);
const SET_1: (u32, u32) = (0x0E_C250, 0x0E_E6C8);
const SET_2: (u32, u32) = (0x0E_DC6C, 0x0E_FA92);
const SET_3: (u32, u32) = (0x0E_F17E, 0x0F_0D76);
const FRAMES_D9386: u32 = 0x0D_9386;
const FRAMES_DA5EA: u32 = 0x0D_A5EA;

/// The list of things to load (`i_load.rs`) and the lists: the
/// mode's, the stage's, and a short one made in RAM from `PATCH`
/// (13 bytes, three of them set by kind).
const LIST: u32 = 0xFFFF_92FA;
const LIST_OTHER: u32 = 0x009C50;
const LIST_STAGE: u32 = 0x009C24;
const PATCH: u32 = 0x009C16;
const PATCH_LEN: u32 = 13;
const MADE_LIST: u32 = 0xFFFF_9550;
/// In the made list: the first entry's index (0xAA + kind), and two
/// more (0xB0 + kind x 2, the one after).
const MADE_FIRST: u32 = 0xFFFF_9550;
const MADE_SECOND: u32 = 0xFFFF_9554;
const MADE_THIRD: u32 = 0xFFFF_9558;

/// Set when the stage is set up (bytes): the step's switch (`i_main`
/// `func_0093BA`), the objects stopped (`a_objects.rs`).
const RUNNING: u32 = 0xFFFF_80DF;
const STOPPED: u32 = 0xFFFF_A3FC;
/// Cleared when the stage is set up (bytes).
const CLEARED_1: u32 = 0xFFFF_AC7D;
const CLEARED_2: u32 = 0xFFFF_A94C;
const CLEARED_3: u32 = 0xFFFF_A42A;
/// `func_00A506`'s argument in the mode.
const IN_MODE: u32 = 4;

/// `func_0099A2`: object 0's bytes copied, and one picked by its 0x32.
const OBJ_COPIED: [(u32, u32); 4] = [(0x2E, 0xFFFF_A937), (0x30, 0xFFFF_A938), (0x10, 0xFFFF_A939), (0x11, 0xFFFF_A93A)];
const COPY_FLAGS: u32 = 0xFFFF_A939;
const OBJ_PICK: u32 = 0x32;
/// Bytes picked by object 0's byte 0x32 (the code has it as
/// 0xFFFFA96E less 5).
const PICKS: u32 = 0xFFFF_A969;
const PICKED: u32 = 0xFFFF_954E;

/// `func_009B0A`: the step's routine (`i_main.rs`), one for each way.
const STEP_ROUTINE: u32 = 0xFFFF_80E0;
const STEP_ONE: u32 = 0x0120F8;
const STEP_OTHER: u32 = 0x01220E;
/// A place (words: x, y) `func_001D6E` puts an object at, and their
/// low bytes; the start of it in one way (`START_X`, `START_Y`).
const PLACE_X: u32 = 0xFFFF_9F04;
const PLACE_X_LOW: u32 = 0xFFFF_9F05;
const PLACE_Y: u32 = 0xFFFF_9F06;
const PLACE_Y_LOW: u32 = 0xFFFF_9F07;
const START_X: u32 = 0xFFFF_954A;
const START_Y: u32 = 0xFFFF_954C;
/// The places in the other way, by object (bytes in the ROM).
const PLACES_X: u32 = 0x009C0A;
const PLACES_Y: u32 = 0x009C10;
/// Cleared by `func_009B0A` (bytes).
const CLEARED_B: u32 = 0xFFFF_A45F;
const WALKER: u32 = 0xFFFF_A93F;
const CLEARED_C: u32 = 0xFFFF_954F;
/// Object 0 (`a_objects.rs`), its fields: the routine and the next
/// one (longs), flags, facing, look.
const OBJECT_0: u32 = 0xFFFF_A432;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_NEXT: u32 = 0x06;
const OBJ_FLAGS: u32 = 0x10;
const OBJ_FACING: u32 = 0x14;
const OBJ_LOOK: u32 = 0x2C;
/// The routine that does nothing (`b_state.rs`'s `STAY`), and two
/// longs `func_004480` takes.
const STAY: u32 = 0x0020AE;
const ARG_1: u32 = 0xFFFF_80C8;
const ARG_2: u32 = 0xFFFF_80D0;
const ARG_1_VALUE: u32 = 0x00B558;
const ARG_2_VALUE: u32 = 0x0045E2;
/// Cleared by `func_009B0A` (0x340 longs from 0xFFFF0000).
const CLEARED_AREA: u32 = 0xFFFF_0000;
const CLEARED_LONGS: u32 = 0x340;
/// The objects `func_009B0A` leaves alone in the other way: from the
/// last placed to 5, bit 7 cleared of the byte at `UNPLACED` + index.
/// (0x2D is in the ROM: the writes do nothing.)
const UNPLACED: u32 = 0x00_002D;
const OBJECTS: u8 = 5;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_009840`: the mode's sprite sets (`func_000776`): 1, 3 and
    /// 5, then `func_00989A`'s.  Gives back d0, a0 and a1.
    pub fn r_009840(&mut self) -> (u32, u32, u32) {
        self.r_000776(1, SET_1.0, SET_1.1);
        self.r_000776(3, SET_3.0, SET_3.1);
        self.r_000776(5, FRAMES_DA5EA, SET_3.1);
        self.r_00989a()
    }

    /// `func_009874`: the stage's sprite sets: 1 by kind
    /// (`SETS_BY_KIND`), 3, then `func_00989A`'s.  Gives back d0, a0,
    /// a1 and X (clear).
    pub fn r_009874(&mut self, a1: u32) -> (u32, u32, u32, bool) {
        let kind = u32::from(self.read8(KIND));
        let frames = self.read32(SETS_BY_KIND + 4 * kind);
        self.r_000776(1, frames, a1);
        self.r_000776(3, FRAMES_DA5EA, a1);
        let (d0, a0, a1) = self.r_00989a();
        (d0, a0, a1, false)
    }

    /// `func_00989A`: sprite sets 0, 2 and 4.  Gives back d0, a0 and a1.
    pub fn r_00989a(&mut self) -> (u32, u32, u32) {
        self.r_000776(0, SET_0.0, SET_0.1);
        self.r_000776(2, SET_2.0, SET_2.1);
        let d0 = self.r_000776(4, FRAMES_D9386, SET_2.1);
        (d0, FRAMES_D9386, SET_2.1)
    }

    /// `loc_0098F8`: the stage set up from `STEP` 0 (`loc_0098FC`).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0098f8(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.write8(STEP, 0);
        self.r_0098fc(d2, d3, d4, d5, d7, a1, a2, a3, a4, a5)
    }

    /// `loc_0098FC`: the tasks (`func_0003DA`), sprites (`func_000704`)
    /// and objects (`func_001C56`) anew, `MODE` 0, the number worked out
    /// (`func_009966`), then `func_0099E6`.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0098fc(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (d0, _, d6, _, _) = self.r_0003da(d2, d3, d4, d5, d7, a1, a2, a3, a4, a5);
        let (d0, d1, a0, _) = self.r_000704(d0);
        let (d6, a6, _) = self.r_001c56(d4, d5, d6);
        self.write8(MODE, 0);
        let (d0, d1, x) = self.r_009966(d0, d1);
        self.r_0099e6(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x)
    }

    /// `func_009966`: `NUMBER` = `NUMBERS`[`STARTS`[kind] + `KIND_2`]
    /// + `STEP`.  Gives back d0 (the number), d1 and X.
    pub fn r_009966(&mut self, d0: u32, d1: u32) -> (u32, u32, bool) {
        let kind = self.read8(KIND);
        let start = self.read8(STARTS + u32::from(kind));
        let at = self.read8(KIND_2).wrapping_add(start);
        let d1 = low_word(d1, u16::from(at));
        let (number, x) = self.read8(NUMBERS + u32::from(at)).overflowing_add(self.read8(STEP));
        self.write8(NUMBER, number);
        (low_word(d0, u16::from(number)), d1, x)
    }

    /// `func_0099A2`: object 0's bytes copied (`OBJ_COPIED`, bit 7 of
    /// the flags cleared), and `PICKED` its byte 0x32 or, when that
    /// isn't negative, `PICKS`[it].  d0, d4, a0 and a4 are kept.
    /// Gives back X and C (clear).
    pub fn r_0099a2(&mut self, d4: u32) -> (bool, bool) {
        let (_, object, x) = self.r_001cfa(d4 & 0xFFFF_0000);
        for (field, to) in OBJ_COPIED {
            let b = self.read8(object + field);
            self.write8(to, b);
        }
        let flags = self.read8(COPY_FLAGS);
        self.write8(COPY_FLAGS, flags & 0x7F);
        let mut picked = self.read8(object + OBJ_PICK);
        if picked & 0x80 == 0 {
            picked = self.read8(PICKS + u32::from(picked));
        }
        self.write8(PICKED, picked);
        (x, false)
    }

    /// `func_0099E6`: the screen off and cleared (`func_0002E0`,
    /// `func_00033A`, `func_0002E6`), `func_00B49A`, `func_0103D4`, the
    /// map (`func_000E3E`, `func_001030`), the sprite sets and lists by
    /// the mode (`func_009840`, `LIST_OTHER`; else `func_009874`,
    /// `LIST_STAGE` and the made list), `func_000E9E`, `func_00644A`,
    /// `func_00ABE0`, `func_00ABA6`, `RUNNING`, the objects placed
    /// (`func_009B0A`), `func_00AF26`; then in the mode `func_00A506`,
    /// else the opening (`func_009C7A`) with the objects `STOPPED`.
    /// Gives back every register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0099e6(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        self.r_0002e0();
        self.r_00033a();
        self.r_0002e6();
        self.r_00b49a(&mut r);
        r.x = self.r_0103d4();
        (r.d0, r.d1, r.d6, r.a6, r.x) = self.r_000e3e(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_001030(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        if self.restarted.is_some() {
            return i_all(&r);
        }
        if self.read8(MODE) & MODE_OTHER != 0 {
            (r.d0, r.a0, r.a1) = self.r_009840();
            self.write32(LIST, LIST_OTHER);
            if self.i_load(&mut r) {
                return i_all(&r);
            }
        } else {
            (r.d0, r.a0, r.a1, r.x) = self.r_009874(r.a1);
            self.write32(LIST, LIST_STAGE);
            if self.i_load(&mut r) {
                return i_all(&r);
            }
            for i in 0..PATCH_LEN {
                let b = self.read8(PATCH + i);
                self.write8(MADE_LIST + i, b);
            }
            let kind = self.read8(KIND);
            // d0's low word 0xFFFF after the copy's dbra
            r.d0 = 0xFF00 | u32::from(kind.wrapping_add(0xAA));
            self.write8(MADE_FIRST, r.d0 as u8);
            let second = kind.wrapping_add(kind).wrapping_add(0xB0);
            self.write8(MADE_SECOND, second);
            let (third, carry) = second.overflowing_add(1);
            r.d1 = low_byte(r.d1, third);
            r.x = carry;
            r.c = carry;
            self.write8(MADE_THIRD, third);
            self.write32(LIST, MADE_LIST);
            r.a0 = PATCH + PATCH_LEN;
            r.a1 = MADE_LIST + PATCH_LEN;
            if self.i_load(&mut r) {
                return i_all(&r);
            }
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_000e9e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return i_all(&r);
        }
        (r.d0, r.d1, r.d2, r.d3, r.a0, r.a1, r.a2, r.x) = self.r_00644a(r.d1, r.d2, r.d3);
        (r.d0, r.d1, r.x) = self.r_00abe0(r.d0, r.d1, r.x);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_00aba6(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return i_all(&r);
        }
        self.write8(RUNNING, 1);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6) =
            self.r_009b0a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        if self.restarted.is_some() {
            return i_all(&r);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x) =
            self.r_00af26(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6);
        self.write8(CLEARED_1, 0);
        self.write8(CLEARED_2, 0);
        if self.read8(MODE) & MODE_OTHER == 0 {
            (r.d0, r.d5, r.d6, r.a5, r.a6, r.x) =
                self.r_009c7a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            self.write8(STOPPED, 1);
            self.write8(CLEARED_3, 0);
        } else {
            (r.d0, r.a1, r.x, r.c) =
                self.r_00a506(IN_MODE, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
            return i_all(&r);
        }
        r.c = false;
        i_all(&r)
    }

    /// `func_0094D8` on `r`; true when the game restarted meanwhile.
    fn i_load(&mut self, r: &mut Regs) -> bool {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        self.restarted.is_some()
    }

    /// `func_009B0A`: the objects placed (`func_001D6E`).  One way:
    /// object 0 at (`START_X`, `START_Y`); unless `PICKED` is negative,
    /// it is set to `STAY` (its routine kept in `OBJ_NEXT`) and
    /// `func_004480` makes another; then the view (`func_002CC0`),
    /// `func_000E6C`, `CLEARED_AREA` cleared, and the objects run once
    /// (`func_001C96`) with `MODE_ONCE`.  The other way (`MODE_OTHER`):
    /// objects 0 to `PLACED` - 1 at `PLACES_X`, `PLACES_Y`, each with
    /// its look.  Gives back every register.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_009b0a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        if self.read8(MODE) & MODE_OTHER != 0 {
            self.i_place_all(&mut r);
            return (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        }
        self.write8(CLEARED_B, 0);
        self.write32(STEP_ROUTINE, STEP_ONE);
        let x = self.read16(START_X);
        self.write16(PLACE_X, x);
        let y = self.read16(START_Y);
        self.write16(PLACE_Y, y);
        r.d4 = 0;
        r.a4 = OBJECT_0;
        r.d1 = 0;
        (r.d6, r.a6, r.x) = self.r_001d6e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5, r.a6);
        self.write8(WALKER, 0);
        r.c = false;
        let picked = self.read8(PICKED);
        if picked & 0x80 == 0 {
            let facing = self.read8(r.a4 + OBJ_FACING);
            self.write8(r.a4 + OBJ_FACING, facing.wrapping_add(1));
            let flags = self.read8(r.a4 + OBJ_FLAGS);
            self.write8(r.a4 + OBJ_FLAGS, flags | 0x80);
            let routine = self.read32(r.a4 + OBJ_ROUTINE);
            self.write32(r.a4 + OBJ_NEXT, routine);
            self.write32(r.a4 + OBJ_ROUTINE, STAY);
            self.write32(ARG_1, ARG_1_VALUE);
            self.write32(ARG_2, ARG_2_VALUE);
            r.d1 = low_byte(r.d1, 2);
            r.d2 = low_byte(r.d2, 0);
            r.d0 = low_byte(r.d0, picked);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.a0, r.a3, r.a4, r.a5, r.x, r.c) =
                self.r_004480(r.d0, r.d1, r.d2, r.d3, r.d5, r.d6, r.d7, r.a1, r.a2);
        }
        (r.d0, r.x, r.c) = self.r_002cc0(r.d0, r.x, r.c);
        r.unwound = false;
        self.r_000e6c(&mut r);
        r.a0 = CLEARED_AREA;
        r.d0 = CLEARED_LONGS;
        (r.d0, r.d1, r.a0, r.x) = self.r_0011d2(r.d0, r.d1, r.a0);
        self.write8(CLEARED_C, 0);
        let mode = self.read8(MODE);
        r.d0 = low_byte(r.d0, MODE_ONCE);
        self.write8(MODE, mode | MODE_ONCE);
        r.c = false;
        r.unwound = false;
        self.r_001c96(&mut r);
        if self.restarted.is_none() {
            self.write8(MODE, mode);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6)
    }

    /// `func_009B0A`'s other way.
    fn i_place_all(&mut self, r: &mut Regs) {
        self.write32(STEP_ROUTINE, STEP_OTHER);
        self.write8(PLACE_X, 0);
        self.write8(PLACE_Y, 0);
        let mut index: u8 = 0;
        r.d0 = 0;
        loop {
            let (d4, object, _) = self.r_001cfa(low_byte(r.d4, index));
            let x = self.read8(PLACES_X + u32::from(index));
            self.write8(PLACE_X_LOW, x);
            let y = self.read8(PLACES_Y + u32::from(index));
            self.write8(PLACE_Y_LOW, y);
            r.d1 = low_byte(r.d1, self.read8(object + OBJ_LOOK));
            (r.d6, r.a6, r.x) = self.r_001d6e(r.d0, r.d1, r.d2, r.d3, low_byte(d4, index), r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5, r.a6);
            index = index.wrapping_add(1);
            r.d0 = u32::from(index);
            if index == self.read8(PLACED) {
                break;
            }
        }
        while index != OBJECTS {
            r.a0 = UNPLACED;
            let at = UNPLACED + u32::from(index);
            let b = self.read8(at);
            self.write8(at, b & 0x7F);
            index = index.wrapping_add(1);
            r.d0 = u32::from(index);
        }
    }
}

/// Every register of `r`, X and C, as the routines give them back.
#[allow(clippy::type_complexity)]
fn i_all(r: &Regs) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
    (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
}
