//! Four places of the stage record `D_00A1F0` (cells at `CORNER_X`,
//! `CORNER_Y`): its cell routine for kinds 3 to 6 (`func_00D9FA`) puts
//! `func_00DAE2` and `func_00DBBA` in the four first entries of the
//! lists at `LIST_A97A` and `LIST_A996`, and makes a task
//! (`func_00DA58`) that draws its place (`func_000FBE`) in five steps
//! while its byte in `MARKS` is set.  `func_00DAE2` sends the task
//! being run there (`func_00DB18`, `func_00DB3E`); `func_00DBBA` the
//! object being run (`func_00DC5E`), and the object worked on with it.
//! No pad script runs these: reviewed by reading.

use crate::{Game, Regs};

/// The task being run (its index, a byte; `func_0005F8`).
const CURRENT_TASK: u32 = 0xFF80EE;
/// The frame count: the steps go on frames with bits 0 and 1 set.
const FRAME: u32 = 0xFF80D8;
/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// The cell (column, row).
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
/// Lists of routines (longs) whose four first entries `func_00D9FA`
/// sets.
const LIST_A97A: u32 = 0xFFFF_A97A;
const LIST_A996: u32 = 0xFFFF_A996;
const LIST_LEN: u32 = 4;
/// The two routines it puts in them (D_00DA50).
const LIST_ROUTINES: u32 = 0x00DA50;
/// A byte by place: set while something is there (`func_00DD4A`,
/// `func_00DD6C`), cleared by `func_00D9FA` and `func_00DA58`.
const MARKS: u32 = 0xFFFF_ACE0;
/// The places' columns, rows (D_00DAD4, D_00DAD8) and the drawing of
/// each step (D_00DADC, for `func_000FBE`).
const CORNER_X: u32 = 0x00DAD4;
const CORNER_Y: u32 = 0x00DAD8;
const STEP_DRAWN: u32 = 0x00DADC;
const LAST_STEP: u8 = 4;
/// A facing (`func_00DB3E`), and a place (`func_00DBBA` to
/// `func_00DD4A`; `func_00DC5E`).
const FACING: u32 = 0xFFAABD;
const PLACE: u32 = 0xFFAC8B;
/// The object worked on and the object being run (index, address).
const OBJECT: u32 = 0xFFA93F;
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
/// Object indexes by an object's index (from 5 on).
const BY_INDEX: u32 = 0xFFFF_A968 - 5;
/// Records of 8 bytes by object; `func_00DBBA` clears their bytes 2, 3.
const RECORDS_AD0A: u32 = 0xFFFF_AD0A;

/// The tasks' routines and priority, and the made tasks' animations
/// (`func_00D5E0`).
const STEPS_ROUTINE: u32 = 0x00DA58;
const PRIORITY: u8 = 0x42;
const GOING_ROUTINE: u32 = 0x00DB18;
const WAITING_ROUTINE: u32 = 0x00DB3E;
const FLYING_ROUTINE: u32 = 0x005C10;
const SENT_ROUTINE: u32 = 0x00DC5E;
const MARK_ANIM: u32 = 0x0DA55C;
const MARK_AGAIN_ANIM: u32 = 0x0DA566;

/// A task's fields: its step, its place, its object's facing; flags
/// (bit 5; bits 3-5 kept, the facing in bits 0-1), a byte count and a
/// byte (a word: the rate, after), its animation (a word cleared).
const TASK_STEP: u32 = 0x13;
const TASK_FACING: u32 = 0x14;
const TASK_PLACE: u32 = 0x15;
const TASK_FLAGS: u32 = 0x18;
const TASK_COUNT: u32 = 0x1A;
const TASK_1B: u32 = 0x1B;
const TASK_RATE: u32 = 0x1A;
const TASK_ANIM: u32 = 0x20;
/// An object's fields: its routine and a kept one, flags, a count,
/// direction flags, position's high bytes, its place plus 1 (0: none).
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_KEPT: u32 = 0x06;
const OBJ_FLAGS: u32 = 0x10;
const OBJ_COUNT: u32 = 0x12;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_X_HIGH: u32 = 0x19;
const OBJ_Y_HIGH: u32 = 0x1B;
const OBJ_PLACE: u32 = 0x3A;
/// A sprite record's frame; an object's sprite (address, index).
const SPRITE_FRAME: u32 = 0x04;
const OBJ_SPRITE_ADDR: u32 = 0x0A;

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
    /// `func_00D9FA`, a cell routine (kinds 3-6, the place the kind
    /// less 3): the lists' four first entries set, a task running
    /// `func_00DA58` for the place (step 0), the place's mark cleared;
    /// command (place + 1) * 16 + 6, C clear.
    pub fn r_00d9fa(&mut self, regs: &mut Regs) {
        let place = (regs.d0 as u8).wrapping_sub(3);
        let kept = low_byte(regs.d0, place);
        let first = self.read32(LIST_ROUTINES);
        let second = self.read32(LIST_ROUTINES + 4);
        for i in 0..LIST_LEN {
            self.write32(LIST_A97A + 4 * i, first);
            self.write32(LIST_A996 + 4 * i, second);
        }
        regs.a1 = LIST_A97A + 4 * LIST_LEN;
        regs.a2 = LIST_A996 + 4 * LIST_LEN;
        (regs.d6, regs.a6, _, _) = self.r_0003e8(regs.d4, regs.d5, regs.d6, PRIORITY, STEPS_ROUTINE);
        let d0 = low_word(kept, index(place) as u16);
        self.write8(regs.a6 + TASK_PLACE, place);
        self.write8(regs.a6 + TASK_STEP, 0);
        regs.a0 = MARKS;
        self.write8(MARKS.wrapping_add(index(place)), 0);
        let next = place.wrapping_add(1);
        regs.x = next & 0x10 != 0;
        regs.d0 = low_byte(d0, (next << 4) | 6);
        regs.c = false;
        regs.unwound = false;
    }

    /// `func_00DA58`, `func_00D9FA`'s task: on frames with bits 0 and
    /// 1 of `FRAME` set, its step one more (up to `LAST_STEP`; there
    /// the mark is cleared) while its place's mark is set, one less (to
    /// 0) while not, and the place drawn at that step.
    pub fn r_00da58(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let a6 = regs.a6;
        let place = self.read8(a6 + TASK_PLACE);
        regs.d1 = low_word(regs.d1, index(place) as u16);
        regs.a0 = MARKS;
        let mark = MARKS.wrapping_add(index(place));
        let step = self.read8(a6 + TASK_STEP);
        if self.read8(mark) != 0 {
            let frame = self.read8(FRAME) & 3;
            regs.d0 = low_byte(regs.d0, frame);
            if frame != 3 {
                regs.c = true;
                return;
            }
            if step == LAST_STEP {
                self.write8(mark, 0);
            } else {
                let (next, carry) = step.overflowing_add(1);
                self.write8(a6 + TASK_STEP, next);
                regs.x = carry;
            }
        } else {
            if step == 0 {
                regs.c = false;
                return;
            }
            let frame = self.read8(FRAME) & 3;
            regs.d0 = low_byte(regs.d0, frame);
            if frame != 3 {
                regs.c = true;
                return;
            }
            self.write8(a6 + TASK_STEP, step - 1);
            regs.x = false;
        }
        let place = self.read8(a6 + TASK_PLACE);
        let column = self.read8(CORNER_X.wrapping_add(index(place)));
        self.write8(CELL_X, column);
        let row = self.read8(CORNER_Y.wrapping_add(index(place)));
        self.write8(CELL_Y, row);
        self.r_001278();
        let step = self.read8(a6 + TASK_STEP);
        regs.d1 = low_word(regs.d1, index(step) as u16);
        let drawn = self.read8(STEP_DRAWN.wrapping_add(index(step)));
        (regs.d0, regs.x, regs.c) = self.r_000fbe(low_byte(regs.d0, drawn));
    }

    /// `func_00DAE2`, in `LIST_A97A` (d0 the place plus 1): the task
    /// being run goes on as `func_00DB18`, bit 5 of its flags set, its
    /// facing the place; the place marked (`func_00DD4A`) and the Z80
    /// sent 0x37.
    pub fn r_00dae2(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let kept = regs.d0;
        regs.d6 = low_byte(regs.d6, self.read8(CURRENT_TASK));
        (regs.d6, regs.a6, _) = self.r_0005f8(regs.d6);
        (regs.x, _) = self.r_00053e(regs.d6, GOING_ROUTINE);
        let a6 = regs.a6;
        let flags = self.read8(a6 + TASK_FLAGS) | 0x20;
        self.write8(a6 + TASK_FLAGS, flags);
        self.write8(a6 + TASK_FACING, kept as u8);
        let r = &mut *regs;
        r.d0 = self.r_00dd4a(kept, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        r.d0 = low_byte(r.d0, 0x37);
        r.c = false;
        self.r_0034b6(r.d0);
    }

    /// `func_00DB18`: the task's cell cleared (`func_00168C`), its
    /// sprite's frame 0, on as `func_00DB3E` after 0x30 frames.
    pub fn r_00db18(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        regs.d0 = self.r_0015f8(regs.a6);
        self.r_00167a();
        (regs.d0, regs.a0) = self.r_00168c(regs.d0);
        self.write8(regs.a5 + SPRITE_FRAME, 0);
        self.write8(regs.a6 + TASK_COUNT, 0x30);
        (regs.x, regs.c) = self.r_00053e(regs.d6, WAITING_ROUTINE);
    }

    /// `func_00DB3E`: when its count runs out, the task put at its
    /// place (bits 0-1 of `TASK_FACING`; `func_001550`), facing the
    /// place less 3, rate -0x400, on as `func_005C10`; the Z80 sent
    /// 0x37 and `func_004FF4`.
    pub fn r_00db3e(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let a6 = regs.a6;
        let (count, borrow) = self.read8(a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(a6 + TASK_COUNT, count);
        regs.x = borrow;
        regs.c = borrow;
        if count != 0 {
            return;
        }
        let d0 = low_byte(regs.d0, self.read8(a6 + TASK_FACING));
        let place = (d0 as u16) & 3;
        let d0 = low_word(d0, place);
        regs.d1 = low_word(regs.d1, place);
        let facing = (place as u8).wrapping_sub(3) & 3;
        self.write8(FACING, facing);
        regs.a0 = CORNER_X;
        let column = self.read8(CORNER_X + u32::from(place));
        self.write8(CELL_X, column);
        self.write8(a6 + TASK_COUNT, column);
        regs.a0 = CORNER_Y;
        let row = self.read8(CORNER_Y + u32::from(place));
        self.write8(CELL_Y, row);
        self.write8(a6 + TASK_1B, row);
        let (d0, _) = self.r_001550(low_byte(d0, row), a6);
        let flags = (self.read8(a6 + TASK_FLAGS) & 0x38) | 0x20 | self.read8(FACING);
        self.write8(a6 + TASK_FLAGS, flags);
        self.write16(a6 + TASK_ANIM, 0);
        self.write16(a6 + TASK_RATE, 0xFC00);
        self.r_00053e(regs.d6, FLYING_ROUTINE);
        let d0 = low_byte(d0, 0x37);
        self.r_0034b6(d0);
        let r = &mut *regs;
        (r.d0, r.x, r.c) = self.r_004ff4(d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00DBBA`, in `LIST_A996` (d0 the place plus 1): unless the
    /// object worked on has a place, when the object being run is
    /// within the map's inner cells (2 to 13 by the low nibbles of its
    /// position's high bytes), the object worked on given the place
    /// (marked, `func_00DD4A`) and the object being run sent there:
    /// `func_00DC5E` for 0x30 frames, its routine kept, bit 7 of its
    /// flags set; the worked-on object's record bytes 2, 3 cleared and
    /// the Z80 sent 0x37.
    pub fn r_00dbba(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.write8(PLACE, regs.d0 as u8);
        let object = self.read8(OBJECT);
        (regs.d4, regs.a4, regs.x) = self.r_001cfa(low_byte(regs.d4, object));
        regs.c = false;
        if self.read8(regs.a4 + OBJ_PLACE) != 0 {
            return;
        }
        regs.d4 = low_byte(regs.d4, self.read8(RUNNING));
        regs.a4 = self.read32(RUNNING_ADDR);
        for at in [OBJ_X_HIGH, OBJ_Y_HIGH] {
            let (inner, borrow) = (self.read8(regs.a4 + at) & 0xF).overflowing_sub(2);
            regs.d0 = low_byte(regs.d0, inner);
            regs.x = borrow;
            regs.c = borrow;
            if (inner as i8) < 0 {
                return;
            }
            if inner >= 0xC {
                regs.c = false;
                return;
            }
        }
        let object = self.read8(OBJECT);
        (regs.d4, regs.a4, _) = self.r_001cfa(low_byte(regs.d4, object));
        let place = self.read8(PLACE);
        self.write8(regs.a4 + OBJ_PLACE, place);
        let r = &mut *regs;
        r.d0 = self.r_00dd4a(low_byte(r.d0, place), r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        let a4 = self.read32(RUNNING_ADDR);
        self.write8(a4 + OBJ_COUNT, 0x30);
        let routine = self.read32(a4 + OBJ_ROUTINE);
        self.write32(a4 + OBJ_KEPT, routine);
        self.write32(a4 + OBJ_ROUTINE, SENT_ROUTINE);
        let flags = self.read8(a4 + OBJ_FLAGS) | 0x80;
        self.write8(a4 + OBJ_FLAGS, flags);
        let object = self.read8(OBJECT);
        (r.d4, r.a4, _) = self.r_001cfa(low_byte(r.d4, object));
        r.d4 = low_word(r.d4, index(r.d4 as u8) as u16);
        let word = low_byte(r.d0, r.d4 as u8) as u16;
        r.x = word & (1 << 13) != 0;
        let at = (word << 3) as i16 as i32 as u32;
        r.d0 = low_word(r.d0, word << 3);
        r.a0 = RECORDS_AD0A;
        self.write8(RECORDS_AD0A.wrapping_add(2).wrapping_add(at), 0);
        self.write8(RECORDS_AD0A.wrapping_add(3).wrapping_add(at), 0);
        r.d0 = low_byte(r.d0, 0x37);
        r.c = false;
        self.r_0034b6(r.d0);
    }

    /// The sprite of the object `by` (an object's index, from 5)
    /// stands for (`BY_INDEX`, made the object worked on): its frame 0.
    /// Gives back d4 and d5.
    fn m_hide_owner(&mut self, d4: u32, d5: u32) -> (u32, u32) {
        let d4 = low_word(d4, index(d4 as u8) as u16);
        let owner = self.read8(BY_INDEX.wrapping_add(index(d4 as u8)));
        self.write8(OBJECT, owner);
        let (d5, a5) = self.r_001d0c(low_byte(d5, owner));
        let sprite = self.read32(a5 + OBJ_SPRITE_ADDR);
        self.write8(sprite + SPRITE_FRAME, 0);
        (d4, d5)
    }

    /// `func_00DC5E`, `func_00DBBA`'s object routine: the object (and,
    /// from object 5, the one it stands for) not shown for its count;
    /// then the object being run put at the place of the object worked
    /// on (`func_001852`), its routine and flags back; from object 5
    /// the one it stands for put there too (`func_0020E2`,
    /// `func_001B32`, the Z80 sent 0x37); the place marked again
    /// (`func_00DD6C`).
    pub fn r_00dc5e(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        self.write8(OBJECT, regs.d4 as u8);
        self.write8(regs.a5 + SPRITE_FRAME, 0);
        if regs.d4 as u8 >= 5 {
            let d5;
            (regs.d4, d5) = self.m_hide_owner(regs.d4, regs.d5);
            regs.d5 = low_byte(d5, self.read8(SLOT_INDEX));
            regs.a5 = self.read32(SLOT_ADDR);
        }
        let a4 = regs.a4;
        let (count, borrow) = self.read8(a4 + OBJ_COUNT).overflowing_sub(1);
        self.write8(a4 + OBJ_COUNT, count);
        regs.x = borrow;
        regs.c = borrow;
        if count != 0 {
            return;
        }
        let r = &mut *regs;
        let object = self.read8(OBJECT);
        (r.d4, r.a4, _) = self.r_001cfa(low_byte(r.d4, object));
        let place = self.read8(r.a4 + OBJ_PLACE) & 3;
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        let a4 = r.a4;
        self.write8(PLACE, place);
        r.d1 = low_word(r.d1, u16::from(place));
        r.a0 = CORNER_Y;
        let column = self.read8(CORNER_X + u32::from(place));
        self.write8(CELL_X, column);
        self.write8(a4 + OBJ_X_HIGH, column << 4);
        let row = self.read8(CORNER_Y + u32::from(place));
        self.write8(CELL_Y, row);
        r.d0 = low_byte(r.d0, row << 4);
        self.write8(a4 + OBJ_Y_HIGH, row << 4);
        self.r_001852(a4);
        let kept = self.read32(a4 + OBJ_KEPT);
        self.write32(a4 + OBJ_ROUTINE, kept);
        let direction = self.read8(a4 + OBJ_DIRECTION) & 0x7F;
        self.write8(a4 + OBJ_DIRECTION, direction);
        let flags = self.read8(a4 + OBJ_FLAGS) & 0x7F;
        self.write8(a4 + OBJ_FLAGS, flags);
        if r.d4 as u8 >= 5 {
            r.d4 = low_word(r.d4, index(r.d4 as u8) as u16);
            let owner = self.read8(BY_INDEX.wrapping_add(index(r.d4 as u8)));
            self.write8(OBJECT, owner);
            let (d5, a5) = self.r_001d0c(low_byte(r.d5, owner));
            let x = self.read8(a4 + OBJ_X_HIGH);
            self.write8(a5 + OBJ_X_HIGH, x);
            let y = self.read8(a4 + OBJ_Y_HIGH);
            self.write8(a5 + OBJ_Y_HIGH, y);
            let (_, _, _, a5, _, _) = self.r_0020e2(r.d0, r.d4, d5);
            let object = self.read8(OBJECT);
            (r.d4, r.a4, _) = self.r_001cfa(low_byte(r.d4, object));
            let (d0, _, _, _) = self.r_001b32(r.d1, r.a4, a5);
            r.d0 = low_byte(d0, 0x37);
            self.r_0034b6(r.d0);
        }
        (r.d0, r.d1, r.a0, r.x, r.c) = self.r_00dd6c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00DD4A`: the mark of place d0 less 1 set, and a task made
    /// there (`func_00D5E0`, `MARK_ANIM`).  d5, a0 and a5 are kept.
    /// Gives back d0.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00dd4a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> u32 {
        let place = index(d0 as u8);
        let d0 = low_word(d0, place as u16);
        self.write8(MARKS.wrapping_sub(1).wrapping_add(place), 1);
        let (d0, _, _, _) = self.r_00d5e0(d0, d1, d2, d3, d4, d5, d6, d7, MARK_ANIM, a1, a2, a3, a4, a5);
        d0
    }

    /// `func_00DD6C`: the mark of place d1 set, and a task made there
    /// (`func_00D5E0`, `MARK_AGAIN_ANIM`).  Gives back d0, d1, a0, X
    /// and C.
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    pub fn r_00dd6c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, bool, bool) {
        let place = index(d1 as u8);
        let d1 = low_word(d1, place as u16);
        self.write8(MARKS.wrapping_add(place), 1);
        let (d0, a0, x, c) = self.r_00d5e0(d0, d1, d2, d3, d4, d5, d6, d7, MARK_AGAIN_ANIM, a1, a2, a3, a4, a5);
        (d0, d1, a0, x, c)
    }
}
