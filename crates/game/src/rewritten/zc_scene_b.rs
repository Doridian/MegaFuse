//! The scene of `D_01924E`'s fourth entry (`func_01CB74`): a list
//! loaded and object 6 made, which comes in on a closing circle
//! (`func_01CC22`), waits (`func_01CCB2`) and makes object 7 that keeps
//! to it (`func_01CD5A`); then object 6 goes, every 0x78 frames, to a
//! cell found above object 0 (`func_01CDAA`, `func_01CE6A`,
//! `func_01CDDA`), rises and sinks by its offset (`func_01CECA`,
//! `func_01CEEE`), and picks what it does next at random
//! (`func_01CF14`): `func_01CF86` (which makes the task `func_01D028`
//! and waits, `func_01CFCA`, `func_01CFF6`), `func_01D31C` or
//! `func_01D3BC`.

use crate::{Game, Regs};

/// `func_01CB7C`'s frames sets (`func_000776`) and list (for
/// `func_0094D8` through `LOAD_LIST`; `D_01CCA2`, in the ROM).
const FRAMES_7: u32 = 0x0DFCA6;
const FRAMES_6: u32 = 0x0E1170;
const LOAD_LIST: u32 = 0xFF92FA;
const LIST: u32 = 0x01CCA2;
/// A long kept around the load (`func_0094D8` changes it).
const KEPT_5810: u32 = 0xFF5810;

/// What `func_0012A0` makes an object with: its position (words), its
/// animations and routine (longs).
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
const OBJ_INIT_ANIMS: u32 = 0xFF80C8;
const OBJ_INIT_ROUTINE: u32 = 0xFF80D0;
/// The low byte of the position's y (`func_001542` sets the position
/// from an object's, `func_0015D4` takes its cell).
const POSITION_Y_LOW: u32 = 0xFF9F07;
/// The cell `func_0015D4` finds (column, row).
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
/// Where `func_01CE6A` leaves the middle of the cell it found (bytes:
/// row, column); `func_01CF14` keeps its table there (a long).
const FOUND_ROW: u32 = 0xFF80C4;
const FOUND_COLUMN: u32 = 0xFF80C5;
const PICKS_AT: u32 = 0xFF80C4;
/// `func_00112C`'s points and speed (`math.rs`).
const FROM_X: u32 = 0xFF9EFA;
const FROM_Y: u32 = 0xFF9EFE;
const TO_X: u32 = 0xFF9EE6;
const TO_Y: u32 = 0xFF9EE8;
const SPEED: u32 = 0xFF9F00;
/// The object being run (index, address; `a_objects.rs`) and a copy of
/// the index the routines here make.
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const CHOSEN: u32 = 0xFFA93F;

/// The scene's bytes, `SCENE_LEN` from `SCENE` cleared at the start:
/// a byte `func_01CF86` picks the facing by, a count to `GO_EVERY`
/// (`func_01CDAA`), a count `func_01CCB2` and `func_01CFCA` go by, and
/// a state object 7 goes by (1: it stays; 0xFF: it goes,
/// `loc_01D30A`; set at `loc_01D632`).
const SCENE: u32 = 0xFFFFAEDE;
const SCENE_LEN: u32 = 0x12;
const PICK: u32 = 0xFFAEDE;
const GO_COUNT: u32 = 0xFFAEDF;
const COUNT: u32 = 0xFFAEE1;
const STATE: u32 = 0xFFAEEC;
/// Set to 1 at the start.
const BYTE_80DF: u32 = 0xFF80DF;
/// Flags: set to 0x80 by `func_01CCB2`.
const FLAGS: u32 = 0xFF9540;
/// Its bit 0 cleared by `func_01CCB2`.
const SPARED: u32 = 0xFF9548;

/// An object's fields: its routine, sprite record (address, index), a
/// byte `func_0012B6` takes, a byte `func_01CF14` picks its table by,
/// a byte (`func_01CD5A`), the pick by direction and the kind of its
/// animations (`func_00130E`), its position (words; the low bytes the
/// pixels), the animation's counters, two bytes (the circle's radius
/// and angle in `func_01CC22`, the cell's middle in `func_01CDDA`), an
/// offset (`a_motion.rs`; its high byte here).
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_0F: u32 = 0x0F;
const OBJ_11: u32 = 0x11;
const OBJ_18: u32 = 0x18;
const OBJ_FACING: u32 = 0x14;
const OBJ_ANIM_KIND: u32 = 0x15;
const OBJ_X: u32 = 0x18;
const OBJ_X_LOW: u32 = 0x19;
const OBJ_Y: u32 = 0x1A;
const OBJ_Y_LOW: u32 = 0x1B;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
const OBJ_26: u32 = 0x26;
const OBJ_27: u32 = 0x27;
const OBJ_OFFSET: u32 = 0x2A;
/// A task's byte `func_01CFCA` clears.
const TASK_18: u32 = 0x18;
/// A sprite record's frame and flags.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// The objects (`func_001CFA`).
const OBJ0: u32 = 0;
const OBJ6: u32 = 6;
const OBJ7: u32 = 7;

/// Object 6's animations and routines.
const OBJ6_ANIMS: u32 = 0x01D78A;
const CIRCLING: u32 = 0x01CC22;
const ARRIVED: u32 = 0x01CCB2;
const WAITING: u32 = 0x01CDAA;
/// How often it goes (`GO_COUNT`).
const GO_EVERY: u8 = 0x78;
const GOING: u32 = 0x01CDDA;
const RISING: u32 = 0x01CECA;
const PICKING: u32 = 0x01CF14;
/// The picks (`D_01CF76`, `D_01CF7E`: bytes 0-2 in the ROM, by a random
/// number's bits 0-2; the second for `OBJ_11` 1).
const PICKS: u32 = 0x01CF76;
const PICKS_1: u32 = 0x01CF7E;
const PICK_0: u32 = 0x01CF86;
const PICK_1: u32 = 0x01D31C;
const PICK_2: u32 = 0x01D3BC;
const COUNTING: u32 = 0x01CFCA;
const WATCHING: u32 = 0x01CFF6;
const SINKING: u32 = 0x01CEEE;
/// The task `func_01CFCA` makes.
const TASK: u32 = 0x01D028;
const TASK_PRIORITY: u8 = 0x32;
/// The offset it rises from and sinks back to (its high byte).
const LOW: u8 = 0xE7;
/// Object 7's animations and routine.
const OBJ7_ANIMS: u32 = 0x01D7DE;
const OBJ7_ROUTINE: u32 = 0x01CD5A;
/// How far `func_01CE6A` looks above object 0 (`D_01CEC6`, bytes in
/// the ROM taken from the y one after another).
const LOOK_UP: u32 = 0x01CEC6;
/// How near the cell's middle `func_01CDDA` stops (pixels each way).
const NEAR: u8 = 8;

/// Sounds (`func_0034B6`) and when.
const START_SOUND: u8 = 0x0F;
const ARRIVED_SOUND: u8 = 0x6A;
const ARRIVED_SOUND_AT: u8 = 0x3C;
/// `func_0007D2`'s depths (the first a `moveq`).
const CIRCLING_DEPTH: u32 = 0x02;
const OBJ7_DEPTH: u8 = 0xFA;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// The object's animation a frame on (`func_001336`): true (and C)
    /// at its end.
    fn zc_b_animate(&mut self, r: &mut Regs) -> bool {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        r.c
    }

    /// The sprite placed for the object (`func_001B32`).
    fn zc_b_place(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
    }

    /// The object's sprite record to d5 and a5.
    fn zc_b_sprite(&mut self, r: &mut Regs) {
        r.d5 = low_byte(r.d5, self.read8(r.a4 + OBJ_SPRITE));
        r.a5 = self.read32(r.a4 + OBJ_SPRITE_ADDR);
    }

    /// The running record at the depth in d0's low byte (`func_0007D2`).
    fn zc_b_depth(&mut self, r: &mut Regs) {
        self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
    }

    /// `func_01CB74`, `D_01924E`'s fourth entry: `func_01CB7C`.
    pub fn r_01cb74(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_01cb7c(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
    }

    /// `func_01CB7C`: the scene set up: frames sets 7 and 6, the list
    /// `LIST` loaded (`func_0094D8`, `KEPT_5810` kept around it; it may
    /// run for frames), `func_0094B2`, object 6 made at (0x88, 0) with
    /// `OBJ6_ANIMS` and `CIRCLING` (radius 0xFF, angle 0, `OBJ_11` 8),
    /// the scene's bytes cleared, the sound `START_SOUND` and
    /// `BYTE_80DF` 1.  Gives back every register, X and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01cb7c(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        self.r_000776(7, FRAMES_7, a1);
        let d0 = self.r_000776(6, FRAMES_6, a1);
        let kept = self.read32(KEPT_5810);
        self.write32(LOAD_LIST, LIST);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c) =
            self.r_0094d8(d0, d1, d2, d3, d4, d5, d6, d7, FRAMES_6, a1, a2, a3, a4, a5, a6, x);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c);
        }
        self.write32(KEPT_5810, kept);
        self.r_0094b2();
        self.write16(POSITION_X, 0x88);
        self.write16(POSITION_Y, 0);
        self.write32(OBJ_INIT_ANIMS, OBJ6_ANIMS);
        self.write32(OBJ_INIT_ROUTINE, CIRCLING);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        (r.d4, r.a4, r.x) = self.r_001cfa(OBJ6);
        (r.d0, r.d5, r.a5, r.x) = self.r_0012a0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
        self.write8(r.a4 + OBJ_0F, 0x21);
        self.write8(r.a5 + SPRITE_FLAGS, 0x86);
        self.write8(r.a4 + OBJ_FACING, 0);
        self.write8(r.a4 + OBJ_ANIM_KIND, 0);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.zc_b_animate(&mut r);
        self.write8(r.a4 + OBJ_26, 0xFF);
        self.write8(r.a4 + OBJ_27, 0);
        self.write8(r.a4 + OBJ_11, 8);
        for i in 0..SCENE_LEN {
            self.write8(SCENE + i, 0);
        }
        // dbra leaves d0's low word 0xFFFF, a0 past the bytes.
        r.a0 = SCENE + SCENE_LEN;
        r.d0 = 0xFF00 | u32::from(START_SOUND);
        self.r_0034b6(r.d0);
        self.write8(BYTE_80DF, 1);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, false)
    }

    /// `func_01CC22`, `CIRCLING`: animated, at depth 2; its y and x on
    /// a circle of radius `OBJ_26` at angle `OBJ_27` (`func_01776C`,
    /// `func_017770`; y less 0x28, x plus 8), the radius one less and
    /// the angle one more; once the radius is 0 its sprite's flags
    /// 0x87, its facing one on, a new animation, `ARRIVED` with `COUNT`
    /// 0.  Placed.
    pub fn r_01cc22(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_b_animate(r);
        r.d0 = CIRCLING_DEPTH;
        self.zc_b_depth(r);
        r.d1 = low_byte(r.d1, self.read8(r.a4 + OBJ_26));
        r.d0 = low_byte(r.d0, self.read8(r.a4 + OBJ_27));
        (r.d0, r.x) = self.r_01776c(r.d0, r.d1, r.d3);
        r.d5 = low_byte(r.d5, self.read8(r.a4 + OBJ_SPRITE));
        let y = (r.d0 as u8).wrapping_sub(0x28);
        r.d0 = low_byte(r.d0, y);
        self.write8(r.a4 + OBJ_Y_LOW, y);
        r.d1 = low_byte(r.d1, self.read8(r.a4 + OBJ_26));
        r.d0 = low_byte(r.d0, self.read8(r.a4 + OBJ_27));
        (r.d0, r.x) = self.r_017770(r.d0, r.d1, r.d3);
        self.zc_b_sprite(r);
        let x = (r.d0 as u8).wrapping_add(8);
        r.d0 = low_byte(r.d0, x);
        self.write8(r.a4 + OBJ_X_LOW, x);
        let radius = self.read8(r.a4 + OBJ_26).wrapping_sub(1);
        self.write8(r.a4 + OBJ_26, radius);
        let angle = self.read8(r.a4 + OBJ_27).wrapping_add(1);
        self.write8(r.a4 + OBJ_27, angle);
        if radius == 0 {
            self.write32(r.a4 + OBJ_ROUTINE, ARRIVED);
            self.write8(r.a5 + SPRITE_FLAGS, 0x87);
            let facing = self.read8(r.a4 + OBJ_FACING).wrapping_add(1);
            self.write8(r.a4 + OBJ_FACING, facing);
            (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
            self.write8(r.a4 + OBJ_TIMER, 0);
            self.write8(r.a4 + OBJ_STEP, 0);
            self.zc_b_animate(r);
            self.write8(COUNT, 0);
        }
        self.zc_b_place(r);
    }

    /// `func_01CCB2`, `ARRIVED`: `COUNT` counted (the sound
    /// `ARRIVED_SOUND` at `ARRIVED_SOUND_AT`); at its animation's end
    /// object 6 goes on as `WAITING` (kind 2 facing 0, a new animation,
    /// offset `LOW`, the bytes 0), object 7 is made at its position with
    /// `OBJ7_ANIMS` and `OBJ7_ROUTINE` (frame 0, flags 0x87, facing 0 of
    /// kind 0), `FLAGS` 0x80 and `SPARED`'s bit 0 cleared.  C clear.
    pub fn r_01ccb2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let count = self.read8(COUNT).wrapping_add(1);
        self.write8(COUNT, count);
        if count == ARRIVED_SOUND_AT {
            r.d0 = low_byte(r.d0, ARRIVED_SOUND);
            self.r_0034b6(r.d0);
        }
        if !self.zc_b_animate(r) {
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, WAITING);
        self.write8(r.a4 + OBJ_ANIM_KIND, 2);
        self.write8(r.a4 + OBJ_FACING, 0);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.write8(r.a4 + OBJ_OFFSET, LOW);
        self.write8(r.a4 + OBJ_26, 0);
        self.write8(r.a4 + OBJ_27, 0);
        (r.d4, r.a4, r.x) = self.r_001cfa(OBJ6);
        let x = self.read16(r.a4 + OBJ_X);
        self.write16(POSITION_X, x);
        let y = self.read16(r.a4 + OBJ_Y);
        self.write16(POSITION_Y, y);
        self.write32(OBJ_INIT_ANIMS, OBJ7_ANIMS);
        self.write32(OBJ_INIT_ROUTINE, OBJ7_ROUTINE);
        (r.d4, r.a4, r.x) = self.r_001cfa(OBJ7);
        (r.d0, r.d5, r.a5, r.x) = self.r_0012a0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.write8(r.a4 + OBJ_0F, 0x21);
        self.write8(r.a5 + SPRITE_FLAGS, 0x87);
        self.write8(r.a4 + OBJ_FACING, 0);
        self.write8(r.a4 + OBJ_ANIM_KIND, 0);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.write8(FLAGS, 0x80);
        let spared = self.read8(SPARED);
        self.write8(SPARED, spared & 0xFE);
        r.c = false;
    }

    /// `func_01CD5A`, `OBJ7_ROUTINE`: by `STATE`: 1, `OBJ_18` 1 and
    /// placed; 0xFF, gone (`loc_01D30A`); else animated, at depth 0xFA,
    /// at object 6's position (its low bytes; `func_001D0C`), `OBJ_18`
    /// 0 and placed.
    pub fn r_01cd5a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let state = self.read8(STATE);
        r.d0 = low_byte(r.d0, state);
        if state == 1 {
            self.write8(r.a4 + OBJ_18, 1);
            self.zc_b_place(r);
            return;
        }
        if state == 0xFF {
            r.c = self.r_01d30a(r.a4);
            return;
        }
        self.zc_b_animate(r);
        r.d0 = low_byte(r.d0, OBJ7_DEPTH);
        self.zc_b_depth(r);
        (r.d5, r.a5) = self.r_001d0c(OBJ6);
        let x = self.read8(r.a5 + OBJ_X_LOW);
        self.write8(r.a4 + OBJ_X_LOW, x);
        let y = self.read8(r.a5 + OBJ_Y_LOW);
        self.write8(r.a4 + OBJ_Y_LOW, y);
        self.write8(r.a4 + OBJ_18, 0);
        self.zc_b_sprite(r);
        self.zc_b_place(r);
    }

    /// `func_01CDAA`, `WAITING`: `CHOSEN` the running object, animated;
    /// every `GO_EVERY` frames (`GO_COUNT`) a cell to go to
    /// (`func_01CE6A`) and `GOING`.  Placed.
    pub fn r_01cdaa(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_b_choose();
        self.zc_b_animate(r);
        let count = self.read8(GO_COUNT).wrapping_add(1);
        self.write8(GO_COUNT, count);
        if count == GO_EVERY {
            self.write8(GO_COUNT, 0);
            (r.a0, r.a1) = self.r_01ce6a(r.d4, r.a4);
            self.write32(r.a4 + OBJ_ROUTINE, GOING);
        }
        self.zc_b_place(r);
    }

    /// `CHOSEN` the running object.
    fn zc_b_choose(&mut self) {
        let running = self.read8(RUNNING);
        self.write8(CHOSEN, running);
    }

    /// `own` less `target` made positive, into d0's low byte (X the
    /// borrow or the negation's); true (and C) when it is below `NEAR`.
    fn zc_b_near(r: &mut Regs, own: u8, target: u8) -> bool {
        let mut diff = own.wrapping_sub(target);
        r.x = own < target;
        if diff & 0x80 != 0 {
            r.x = diff != 0;
            diff = diff.wrapping_neg();
        }
        r.d0 = low_byte(r.d0, diff);
        r.c = diff < NEAR;
        r.c
    }

    /// `func_01CDDA`, `GOING`: `CHOSEN` the running object, animated;
    /// once its x and y are within `NEAR` of `OBJ_26`, `OBJ_27` it is put
    /// in its cell's middle (`func_001852`) and goes on as `RISING` (C
    /// clear).  Else it moves toward them at 0x180 (`func_00112C`,
    /// `func_0019C0`, `func_0019DA`; d4 and a4 the running object),
    /// placed and its depth (`func_001B50`).
    pub fn r_01cdda(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_b_choose();
        self.zc_b_animate(r);
        let x = self.read8(r.a4 + OBJ_X_LOW);
        let target_x = self.read8(r.a4 + OBJ_26);
        if Self::zc_b_near(r, x, target_x) {
            let y = self.read8(r.a4 + OBJ_Y_LOW);
            let target_y = self.read8(r.a4 + OBJ_27);
            if Self::zc_b_near(r, y, target_y) {
                self.r_001852(r.a4);
                self.write32(r.a4 + OBJ_ROUTINE, RISING);
                r.c = false;
                return;
            }
        }
        let x = self.read16(r.a4 + OBJ_X);
        self.write16(FROM_X, x);
        let y = self.read16(r.a4 + OBJ_Y);
        self.write16(FROM_Y, y);
        let target_x = self.read8(r.a4 + OBJ_26);
        self.write16(TO_X, u16::from(target_x));
        let target_y = self.read8(r.a4 + OBJ_27);
        self.write16(TO_Y, u16::from(target_y));
        self.write16(SPEED, 0x180);
        self.r_00112c();
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
        (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        self.zc_b_place(r);
        self.zc_b_sprite(r);
        (r.d0, r.x, r.c) = self.r_001b50(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_01CE6A`: a cell above object 0: from its position, y less
    /// each of `LOOK_UP`'s bytes in turn until the cell there
    /// (`func_0015D4`, `func_00167A`) isn't 0; its middle (column x 16
    /// + 8, row x 16 + 8) into `FOUND_COLUMN`, `FOUND_ROW` and the
    /// object's `OBJ_26`, `OBJ_27`.  d4-d6 and a4-a6 are kept (movem).
    /// Gives back a0 (the cell's address) and a1 (past the byte used).
    pub fn r_01ce6a(&mut self, d4: u32, a4: u32) -> (u32, u32) {
        let (_, obj0, _) = self.r_001cfa(low_word(d4, OBJ0 as u16));
        let mut look = LOOK_UP;
        let cell = loop {
            self.r_001542(obj0);
            let up = self.read8(look);
            look += 1;
            let y = self.read8(POSITION_Y_LOW).wrapping_sub(up);
            self.write8(POSITION_Y_LOW, y);
            self.r_0015d4();
            let (cell, _) = self.r_00167a();
            if self.read8(cell) != 0 {
                break cell;
            }
        };
        let column = (self.read8(CELL_X) << 4) | 8;
        self.write8(FOUND_COLUMN, column);
        let row = (self.read8(CELL_Y) << 4) | 8;
        self.write8(FOUND_ROW, row);
        self.write8(a4 + OBJ_26, column);
        self.write8(a4 + OBJ_27, row);
        (cell, look)
    }

    /// `func_01CECA`, `RISING`: `CHOSEN` the running object, animated,
    /// its offset one up, placed; once the offset is 0, `PICKING`.  C
    /// clear.
    pub fn r_01ceca(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_b_choose();
        self.zc_b_animate(r);
        let offset = self.read8(r.a4 + OBJ_OFFSET).wrapping_add(1);
        self.write8(r.a4 + OBJ_OFFSET, offset);
        self.zc_b_place(r);
        if offset == 0 {
            self.write32(r.a4 + OBJ_ROUTINE, PICKING);
        }
        r.c = false;
    }

    /// `func_01CEEE`, `SINKING`: `CHOSEN` the running object, animated,
    /// its offset one down, placed; once it is `LOW`, `WAITING` again.
    /// C: the offset below `LOW` (clear at it).
    pub fn r_01ceee(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.zc_b_choose();
        self.zc_b_animate(r);
        let offset = self.read8(r.a4 + OBJ_OFFSET).wrapping_sub(1);
        self.write8(r.a4 + OBJ_OFFSET, offset);
        self.zc_b_place(r);
        r.c = offset < LOW;
        if offset == LOW {
            self.write32(r.a4 + OBJ_ROUTINE, WAITING);
            r.c = false;
        }
    }

    /// `func_01CF14`, `PICKING`: a random number's bits 0-2
    /// (`func_001BE2`) pick from `PICKS` (`PICKS_1` when `OBJ_11` is 1;
    /// the table kept in `PICKS_AT`, and in a0): 0 `PICK_0`, 1 `PICK_1`
    /// (both with `COUNT` 0), else `PICK_2`.  Gives back d0 (the pick,
    /// over the index's word), d1, a0, X and C (clear).
    pub fn r_01cf14(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let table = if self.read8(r.a4 + OBJ_11) == 1 { PICKS_1 } else { PICKS };
        self.write32(PICKS_AT, table);
        (r.d0, r.d1, r.x) = self.r_001be2(r.d1);
        let index = r.d0 as u8 & 7;
        r.a0 = self.read32(PICKS_AT);
        let pick = self.read8(r.a0 + u32::from(index));
        r.d0 = low_byte(low_word(r.d0, u16::from(index)), pick);
        let next = match pick {
            0 => PICK_0,
            1 => PICK_1,
            _ => PICK_2,
        };
        if pick < 2 {
            self.write8(COUNT, 0);
        }
        self.write32(r.a4 + OBJ_ROUTINE, next);
        r.c = false;
    }

    /// `func_01CF86`, `PICK_0`: facing 2, 1 or 3 by `PICK` (0, 1,
    /// other), a new animation, animated, and `COUNTING`.  C clear.
    pub fn r_01cf86(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let pick = self.read8(PICK);
        r.d0 = low_byte(r.d0, pick);
        let facing = match pick {
            0 => 2,
            1 => 1,
            _ => 3,
        };
        self.write8(r.a4 + OBJ_FACING, facing);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.zc_b_animate(r);
        self.write32(r.a4 + OBJ_ROUTINE, COUNTING);
        r.c = false;
    }

    /// `func_01CFCA`, `COUNTING`: `COUNT` counted; at 0xF, `WATCHING`
    /// and the task `TASK` (its `TASK_18` 0).  C: the count below 0xF
    /// (clear at it).
    pub fn r_01cfca(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (count, carry) = self.read8(COUNT).overflowing_add(1);
        self.write8(COUNT, count);
        r.x = carry;
        r.c = count < 0xF;
        if count != 0xF {
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, WATCHING);
        (r.d6, r.a6, r.x, _) = self.r_0003e8(r.d4, r.d5, r.d6, TASK_PRIORITY, TASK);
        self.write8(r.a6 + TASK_18, 0);
        r.c = false;
    }

    /// `func_01CFF6`, `WATCHING`: if it meets something
    /// (`func_0015E6`, `func_006E7C`), `loc_01D632`; else animated, and
    /// at its animation's end facing 0, a new animation, animated, and
    /// `SINKING` (C clear).
    pub fn r_01cff6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.x) = self.r_0015e6(r.a4);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e7c(r.d1, r.d2, r.a0, r.a4, r.a5);
        if r.c {
            (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_01d632(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
            return;
        }
        if !self.zc_b_animate(r) {
            return;
        }
        self.write8(r.a4 + OBJ_FACING, 0);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        (r.a0, r.x, _) = self.r_00130e(r.d0, r.a4);
        self.zc_b_animate(r);
        self.write32(r.a4 + OBJ_ROUTINE, SINKING);
        r.c = false;
    }
}
