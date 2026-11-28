//! The objects (`func_001CFA`'s) set up on a screen of choices, their
//! routines placing their sprites by their index, side and the screen
//! (`func_011B62`, `func_011BC0`, `func_011D98`), and the tasks with a
//! sprite made beside them (`func_011CCA`, `func_011F64`,
//! `func_01179C`).

use crate::{Game, Regs};

/// The number of objects in play (a byte).
const OBJECTS: u32 = 0xFF9541;
/// Flags: bit 1, the objects go by side.
const FLAGS: u32 = 0xFF9540;
const BY_SIDE: u8 = 1 << 1;
/// The screen shown (a byte: 5 and 7 and below 5 place the objects
/// in a row, 5 by `X_5`/`Y_5`; below 6 they get a mark,
/// `func_011F64`).
const SCREEN: u32 = 0xFFAD3A;
const SCREEN_ROW_LOW: u8 = 5;
const SCREEN_ROW_HIGH: u8 = 7;
const SCREEN_MARKED: u8 = 6;
/// The rows' y (d1): on the screens below 5, and on 7.
const ROW_Y_LOW: u32 = 0x34;
const ROW_Y_HIGH: u32 = 0x65;
/// A screen number `func_011F86` compares with (to no effect).
const SCREEN_2: u8 = 2;
/// The direction (`p_slide.rs`): an object's kind follows it.
const DIRECTION: u32 = 0xFFAABD;
/// The routine an object is set up with (`func_001E30`, a long).
const OBJ_INIT_ROUTINE: u32 = 0xFF80D0;
/// The object a routine places (index, address) and its sprite
/// record (index, address).
const OBJ_INDEX: u32 = 0xFFA42C;
const OBJ_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// The index of the object being set up, kept across calls.
const SETTING_UP: u32 = 0xFFAC91;
/// The objects set up so far not on `ONE_SIDE`'s side (`func_011BC0`).
const OTHERS: u32 = 0xFFAC90;
/// One object's index (when not by side) or a side (bit 6, by side).
const ONE_SIDE: u32 = 0xFFAD41;
/// Two bytes by the number of objects (`func_011A72`): where the row
/// starts and its spacing's part.
const ROW_START: u32 = 0xFFAD3D;
const ROW_LESS: u32 = 0xFFAD3E;
/// While not 0 the objects walk to their place in the row a pixel a
/// frame, else they are put there.
const WALK: u32 = 0xFFAD40;
/// `func_011D98`'s work: the objects to step over (then the side's
/// count, `func_011F1C`), the step (then a place among the side's), and
/// the x it comes to.
const STEPS: u32 = 0xFFAC8B;
const STEP: u32 = 0xFFAC8A;
const ROW_X: u32 = 0xFFAC8D;
/// Set while `func_01179C`'s task runs, cleared when it is over.
const RUNNING: u32 = 0xFFAD44;
/// The pick `func_01179C` makes (0-7) and the y it gives.
const PICK: u32 = 0xFFAD42;
/// A velocity's y (a word; `a_motion.rs`).
const VELOCITY_Y: u32 = 0xFF9EC6;

/// An object's fields: its sprite record (address, index), its
/// direction and kind, its position's (`func_001336`) x and y (words),
/// its look, its flags (bit 6 its side; bit 4 picks the mark's frame;
/// bits 0-2 a number `func_01179C` counts).
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x15;
const OBJ_X_HIGH: u32 = 0x18;
const OBJ_X: u32 = 0x19;
const OBJ_Y_HIGH: u32 = 0x1A;
const OBJ_Y: u32 = 0x1B;
const OBJ_ANIM: u32 = 0x20;
const OBJ_LOOK: u32 = 0x2C;
const OBJ_FLAGS: u32 = 0x2D;
const SIDE: u8 = 1 << 6;
const MARK_B: u8 = 1 << 4;
/// The kinds of objects `func_011B62` and `func_011BC0` set up.
const KIND_ROW: u8 = 2;
const KIND_STILL: u8 = 0x0F;
const KIND_SIDE: u8 = 0x0D;

/// A sprite record's fields (`sprite.rs`): its index, frame, flags,
/// its tile, x and y (words; their low bytes apart), its frames'
/// table.
const SPRITE_INDEX: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_TILE: u32 = 0x06;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;
const SPRITE_FRAMES: u32 = 0x0C;
const CHANGED: u8 = 1 << 4;
const FLIPPED: u8 = 1 << 3;
/// An x off the screen.
const AWAY: u16 = 0xFF80;

/// A task's fields: its sprite's index; bytes 0xC (an object's index),
/// 0xE-0x11 (its position: words, x then y), its count (0x16), 0x18,
/// 0x19 (an object's index), its animation (a long).
const TASK_OBJECT: u32 = 0x0C;
const TASK_X: u32 = 0x0E;
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y: u32 = 0x10;
const TASK_Y_LOW: u32 = 0x11;
const TASK_COUNT: u32 = 0x16;
const TASK_18: u32 = 0x18;
const TASK_INDEX: u32 = 0x19;
const TASK_ANIM: u32 = 0x20;
const TASK_ANIM_LOW: u32 = 0x21;

/// The objects' routines.
const ROW_ROUTINE: u32 = 0x011D98;
const STILL_ROUTINE: u32 = 0x011C92;
/// The tasks made: routine, priority, depth.
const MARK: (u32, u8, u8) = (0x011F86, 0x50, 0x79);
const BADGE: (u32, u8, u8) = (0x011D4C, 0x50, 0xF0);
const PICKER: (u32, u8, u8) = (0x01185A, 0x60, 0x40);
const PICKED: (u32, u8, u8) = (0x0118B4, 0x42, 0x02);
/// Frames' tables and animations set.
const BADGE_FRAMES: u32 = 0x0093B4;
const PICKER_ANIM: u32 = 0x0DA4B0;
const PICKED_ANIM: u32 = 0x0DA7BC;
/// The sound command `func_01185A` sends at its end.
const PICKED_SOUND: u8 = 0x25;

/// Tables in the ROM (bytes).  By the objects' number (2-5): the row's
/// step (at func_011D98 + 0xC4, D_011E5E's bytes 2 on).
const ROW_STEPS: u32 = 0x011E5C;
/// x and y by object (+ 5 when not by side; D_011E8A, D_011E94).
const X_5: u32 = 0x011E8A;
const Y_5: u32 = 0x011E94;
/// By side: the first place by the side's count (D_011F04 - 1), and x
/// and y by place (D_011F08, D_011F12).
const SIDE_FIRST: u32 = 0x011F03;
const SIDE_X: u32 = 0x011F08;
const SIDE_Y: u32 = 0x011F12;
/// `func_011BC0`'s: the first place by the side's count, or by the
/// objects' number (D_011F04 - 1 and - 2), and x and y by place
/// (D_011CA2, D_011CAC).
const STILL_FIRST: u32 = 0x011F04;
const STILL_X: u32 = 0x011CA2;
const STILL_Y: u32 = 0x011CAC;
/// The place of the side's objects past the one not.
const STILL_SIDE_PLACES: u8 = 0x14;
/// `func_011D4C`'s animations' low bytes by index (D_011D88).
const BADGE_ANIMS: u32 = 0x011D88;
/// `func_01179C`'s y by object (D_011E99), and its x by pick (at
/// func_01179C + 0xB7: index 0 is the low byte of the `rts` before the
/// table that follows it).
const PICKER_Y: u32 = 0x011E99;
const PICK_X: u32 = 0x011853;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// A byte of a table by a byte index (signed, as `(d8,pc,dn.w)`).
fn at(table: u32, index: u8) -> u32 {
    table.wrapping_add(i32::from(index as i8) as u32)
}

impl Game {
    /// `func_001E30` on the object `r.a4` (index `r.d4`) with `routine`
    /// and its look, then the object and its sprite record kept for
    /// the routines (`OBJ_INDEX` ..., `SLOT_ADDR`).
    fn p_set_up(&mut self, r: &mut Regs, routine: u32) {
        r.d1 = low_byte(r.d1, self.read8(r.a4 + OBJ_LOOK));
        self.write32(OBJ_INIT_ROUTINE, routine);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x) =
            self.r_001e30(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    fn p_keep_object(&mut self, r: &Regs) {
        self.write8(OBJ_INDEX, r.d4 as u8);
        self.write32(OBJ_ADDR, r.a4);
        self.write8(SLOT_INDEX, r.d5 as u8);
        self.write32(SLOT_ADDR, r.a5);
    }

    /// `func_011B62`: the four objects set up with `func_011D98` as
    /// their routine (kind 2, direction 0), placed by it at once, and
    /// on the screens below 6 a mark each (`func_011F64`).  Gives back
    /// d0, d1, d2, d4, d5, d6, a0, a4, a5 and a6.
    pub fn r_011b62(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let mut r = Regs { d0, d1, d2, d3, d4: low_word(d4, 0), d5, d6, d7, a1, a2, a3, a5, a6, ..Regs::default() };
        loop {
            let kept = r.d4;
            (_, r.a4, r.x) = self.r_001cfa(r.d4);
            self.write8(SETTING_UP, r.d4 as u8);
            self.p_set_up(&mut r, ROW_ROUTINE);
            self.write8(r.a4 + OBJ_KIND, KIND_ROW);
            self.write8(r.a4 + OBJ_DIRECTION, 0);
            self.p_keep_object(&r);
            r.c = false;
            self.r_011d98(&mut r);
            let screen = self.read8(SCREEN);
            r.d0 = low_byte(r.d0, screen);
            if screen < SCREEN_MARKED {
                (r.d5, r.d6, r.a5, r.a6) =
                    self.r_011f64(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            }
            r.d4 = low_byte(kept, (kept as u8).wrapping_add(1));
            if r.d4 as u8 == 4 {
                return (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6);
            }
        }
    }

    /// `func_011BC0`: the objects set up with `func_011C92` as their
    /// routine (kind 0xF, direction 0), each given a place's x and y:
    /// by side, the side's first place by its count (`func_011F1C`) plus
    /// the object's among them, past `STILL_SIDE_PLACES` (kind 0xD)
    /// when not on `ONE_SIDE`'s side; else place 0 for the object
    /// `ONE_SIDE`, the others (kind 0xD) from the first place by the
    /// objects' number on, past `STILL_SIDE_PLACES`.  Then placed
    /// (`func_011C92`).  Gives back d0, d1, d2, d4, d5, a0, a4, a5 and
    /// X.
    pub fn r_011bc0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4: low_word(d4, 0), d5, d6, d7, a1, a2, a3, a5, a6, ..Regs::default() };
        self.write8(OTHERS, 0);
        loop {
            self.write8(SETTING_UP, r.d4 as u8);
            (_, r.a4, r.x) = self.r_001cfa(r.d4);
            self.p_set_up(&mut r, STILL_ROUTINE);
            self.p_keep_object(&r);
            self.write8(r.a4 + OBJ_DIRECTION, 0);
            self.write8(r.a4 + OBJ_KIND, KIND_STILL);
            r.d0 = low_word(r.d0, 0);
            let other = if self.read8(FLAGS) & BY_SIDE != 0 {
                self.r_011f1c(r.d0, r.d1, r.d4);
                r.a0 = STILL_FIRST;
                // d0's word: the count unsigned
                let count = self.read8(STEPS);
                let first = self.read8(STILL_FIRST - 1 + u32::from(count));
                let place;
                (place, r.x) = first.overflowing_add(self.read8(STEP));
                r.d0 = low_word(r.d0, place as i8 as u16);
                r.d2 = low_byte(r.d2, self.read8(ONE_SIDE));
                r.d1 = low_byte(r.d1, self.read8(r.a4 + OBJ_FLAGS) ^ r.d2 as u8);
                r.d1 as u8 & SIDE != 0
            } else if r.d4 as u8 == self.read8(ONE_SIDE) {
                false
            } else {
                r.a0 = STILL_FIRST;
                let objects = self.read8(OBJECTS);
                let first = self.read8(at(STILL_FIRST - 2, objects));
                let others = self.read8(OTHERS);
                self.write8(OTHERS, others.wrapping_add(1));
                r.d0 = low_word(r.d0, first.wrapping_add(others) as i8 as u16);
                true
            };
            if other {
                self.write8(r.a4 + OBJ_KIND, KIND_SIDE);
                let place;
                (place, r.x) = (r.d0 as u8).overflowing_add(STILL_SIDE_PLACES);
                r.d0 = low_byte(r.d0, place);
            }
            let place = r.d0 as u8;
            r.d0 = low_word(r.d0, place as i8 as u16);
            let x = self.read8(at(STILL_X, place));
            self.write8(r.a4 + OBJ_X, x);
            let y = self.read8(at(STILL_Y, place));
            self.write8(r.a4 + OBJ_Y, y);
            self.write8(r.a4 + OBJ_X_HIGH, 0);
            self.write8(r.a4 + OBJ_Y_HIGH, 0);
            r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
            r.a5 = self.read32(SLOT_ADDR);
            r.c = false;
            self.r_011c92(&mut r);
            let index;
            (index, r.x) = self.read8(SETTING_UP).overflowing_add(1);
            r.d4 = low_byte(r.d4, index);
            if index == self.read8(OBJECTS) {
                return (r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, r.x);
            }
        }
    }

    /// `func_011C92`, an object's routine: its position (`func_001B32`)
    /// and its sprite (`func_001B5C`), its animation (`func_00130E`,
    /// `func_001336`).
    pub fn r_011c92(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.z, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
    }

    /// `func_011CCA`: for each object a task with a sprite
    /// (`func_011D4C`'s; 0x18 3, its index the object's), its position
    /// 0x80 lower (`func_00152A`), its frame the object's animation's
    /// second byte, its flags the object's sprite's flipped, its tile
    /// the object's sprite's + 0x40, its frames `BADGE_FRAMES`.  Gives
    /// back d4, d5, d6, a0, a4, a5 and a6 (the last object's sprite).
    pub fn r_011cca(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        let (mut d0, mut d4, mut d5, mut d6, mut a5) = (d0, low_word(d4, 0), d5, d6, a5);
        loop {
            let (_, a4, _) = self.r_001cfa(d4);
            self.write8(SETTING_UP, d4 as u8);
            let (routine, priority, depth) = BADGE;
            let task;
            (d5, d6, a5, task, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, depth, priority, routine);
            self.write8(task + TASK_18, 3);
            let index = self.read8(SETTING_UP);
            d0 = low_byte(d0, index);
            self.write8(task + TASK_INDEX, index);
            self.write8(a5 + SPRITE_INDEX, index);
            self.r_00152a(task);
            let y = self.read16(task + TASK_Y);
            self.write16(task + TASK_Y, y.wrapping_add(0x80));
            d4 = low_byte(d4, self.read8(SETTING_UP));
            let (_, a4, _) = self.r_001cfa(d4);
            let a0 = self.read32(a4 + OBJ_ANIM);
            self.r_011d8e(a0, a5);
            d6 = low_byte(d6, self.read8(a4 + OBJ_SPRITE));
            let sprite = self.read32(a4 + OBJ_SPRITE_ADDR);
            let flags = self.read8(sprite + SPRITE_FLAGS) | FLIPPED;
            d0 = low_byte(d0, flags);
            self.write8(a5 + SPRITE_FLAGS, flags);
            let tile = self.read16(sprite + SPRITE_TILE).wrapping_add(0x40);
            d0 = low_word(d0, tile);
            self.write16(a5 + SPRITE_TILE, tile);
            self.write32(a5 + SPRITE_FRAMES, BADGE_FRAMES);
            let index = self.read8(SETTING_UP).wrapping_add(1);
            d4 = low_byte(d4, index);
            if index == self.read8(OBJECTS) {
                return (d4, d5, d6, a0, a4, a5, sprite);
            }
        }
    }

    /// `func_011D4C`, a task's routine (unless `func_0109FA` ends it):
    /// its animation's low byte by its object's index, its y 0x54; on
    /// side 1, 8 and 0x39 more; then moved (`loc_001BA4`) with the
    /// kept sprite record.
    pub fn r_011d4c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let unwound;
        (r.d0, r.d6, r.x, r.c, unwound) = self.r_0109fa(r.d0, r.d5, r.d6, r.a6, r.x, 0x011D52);
        if unwound {
            return;
        }
        let a6 = r.a6;
        let index = self.read8(a6 + TASK_INDEX);
        r.d4 = low_word(r.d4, index as i8 as u16);
        (_, r.a4, r.x) = self.r_001cfa(r.d4);
        let anim = self.read8(at(BADGE_ANIMS, index));
        self.write8(a6 + TASK_ANIM_LOW, anim);
        self.write8(a6 + TASK_Y_LOW, 0x54);
        if self.read8(r.a4 + OBJ_FLAGS) & SIDE != 0 {
            self.write8(a6 + TASK_ANIM_LOW, anim.wrapping_add(8));
            self.write8(a6 + TASK_Y_LOW, 0x54 + 0x39);
        }
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.x, r.c) = self.r_001ba4(r.a5, a6);
    }

    /// `func_011D8E`: the sprite's frame (a5) the second byte at a0.
    pub fn r_011d8e(&mut self, a0: u32, a5: u32) {
        let frame = self.read8(a0 + 1);
        self.write8(a5 + SPRITE_FRAME, frame);
        self.r_001334();
    }

    /// `func_011D98`, an object's routine: its kind the direction (the
    /// sprite marked changed when it isn't), its animation
    /// (`func_00130E`, `func_001336`), then its sprite placed: away
    /// past the objects in play; on the screens below 5 (y 0x34) and
    /// 7 (y 0x65) in a row, its x `ROW_START` and a step per object
    /// before it (by the objects' number, less `ROW_LESS`), walked to
    /// a pixel a frame while `WALK`; on screen 5 by `X_5`/`Y_5`; on
    /// others by side, by the side's count and the object's place
    /// among them (`func_011F1C`; side 1 0x40 lower, `func_001B5C`),
    /// else by `X_5`/`Y_5` past 5.
    pub fn r_011d98(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let direction = self.read8(DIRECTION);
        r.d0 = low_byte(r.d0, direction);
        if direction != self.read8(r.a4 + OBJ_KIND) {
            self.write8(r.a4 + OBJ_KIND, direction);
            let flags = self.read8(r.a5 + SPRITE_FLAGS) | CHANGED;
            self.write8(r.a5 + SPRITE_FLAGS, flags);
        }
        (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.z, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        let last;
        (last, r.x) = self.read8(OBJECTS).overflowing_sub(1);
        r.d0 = low_byte(r.d0, last);
        r.c = false;
        if last < self.read8(OBJ_INDEX) {
            self.write16(r.a5 + SPRITE_X, AWAY);
            return;
        }
        r.d1 = ROW_Y_HIGH;
        let screen = self.read8(SCREEN);
        r.d0 = low_byte(r.d0, screen);
        if screen < SCREEN_ROW_LOW || screen == SCREEN_ROW_HIGH {
            if screen < SCREEN_ROW_LOW {
                r.d1 = ROW_Y_LOW;
            }
            self.p_row(r);
            return;
        }
        if screen == SCREEN_ROW_LOW {
            r.d4 = low_byte(r.d4, self.read8(OBJ_INDEX));
            r.a4 = self.read32(OBJ_ADDR);
        } else if self.read8(FLAGS) & BY_SIDE == 0 {
            r.d4 = low_byte(r.d4, self.read8(OBJ_INDEX).wrapping_add(5));
            (_, r.a4, r.x) = self.r_001cfa(r.d4);
        } else {
            self.p_by_side(r);
            return;
        }
        let index = r.d4 as u8;
        r.d4 = low_word(r.d4, index as i8 as u16);
        let x = self.read8(at(X_5, index));
        self.write8(r.a5 + SPRITE_X_LOW, x);
        let y = self.read8(at(Y_5, index));
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        self.write8(r.a5 + SPRITE_X, 0);
        self.write8(r.a5 + SPRITE_Y, 0);
    }

    /// `func_011D98`'s row: y `r.d1`, x as `ROW_X` comes to (or walked
    /// to).
    fn p_row(&mut self, r: &mut Regs) {
        self.write8(r.a5 + SPRITE_Y_LOW, r.d1 as u8);
        self.write8(r.a5 + SPRITE_Y, 0);
        let steps = self.read8(OBJ_INDEX).wrapping_add(1);
        self.write8(STEPS, steps);
        let objects = self.read8(OBJECTS);
        r.d0 = low_word(r.d0, objects as i8 as u16);
        let step = self.read8(at(ROW_STEPS, objects)).wrapping_sub(self.read8(ROW_LESS));
        self.write8(STEP, step);
        let start = self.read8(ROW_START);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        self.write8(ROW_X, start);
        self.write8(r.a5 + SPRITE_X, 0);
        loop {
            let step = self.read8(STEP);
            r.d0 = low_byte(r.d0, step);
            let row_x = self.read8(ROW_X).wrapping_add(step);
            self.write8(ROW_X, row_x);
            let steps = self.read8(STEPS);
            self.write8(STEPS, steps.wrapping_sub(1));
            r.x = steps == 0;
            if steps == 1 {
                break;
            }
        }
        r.c = false;
        let walk = self.read8(WALK);
        r.d0 = low_byte(r.d0, walk);
        let target = self.read8(ROW_X);
        if walk == 0 {
            self.write8(r.a5 + SPRITE_X_LOW, target);
            return;
        }
        let x = self.read8(r.a5 + SPRITE_X_LOW);
        r.d0 = low_byte(r.d0, x);
        if x == target {
            return;
        }
        let x = if x > target { x - 1 } else { x + 1 };
        // neither wraps: X clear
        r.x = false;
        r.d0 = low_byte(r.d0, x);
        self.write8(r.a5 + SPRITE_X_LOW, x);
    }

    /// `func_011D98` by side: the place the side's first by its count
    /// plus the object's among them (`func_011F1C`), side 1 0x40 lower;
    /// the sprite moved (`func_001B5C`).
    fn p_by_side(&mut self, r: &mut Regs) {
        r.d4 = low_byte(r.d4, self.read8(OBJ_INDEX));
        self.r_011f1c(r.d0, r.d1, r.d4);
        let count = self.read8(STEPS);
        let place = self.read8(at(SIDE_FIRST, count)).wrapping_add(self.read8(STEP));
        r.d0 = low_word(r.d0, place as i8 as u16);
        let x = self.read8(at(SIDE_X, place));
        self.write8(r.a5 + SPRITE_X_LOW, x);
        let y = self.read8(at(SIDE_Y, place));
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        self.write8(r.a5 + SPRITE_X, 0);
        self.write8(r.a5 + SPRITE_Y, 0);
        r.d4 = low_byte(r.d4, self.read8(OBJ_INDEX));
        r.a4 = self.read32(OBJ_ADDR);
        let flags = self.read8(r.a4 + OBJ_FLAGS);
        r.d0 = low_byte(r.d0, flags);
        if flags & SIDE != 0 {
            let y = self.read8(r.a5 + SPRITE_Y_LOW).wrapping_add(0x40);
            self.write8(r.a5 + SPRITE_Y_LOW, y);
        }
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_011F64`: a mark: a task with a sprite (`func_011F86`'s),
    /// the sprite's flags 0x84, the task's object `SETTING_UP`.  Gives
    /// back d5, d6, a5 and a6 (the sprite's and the task's).
    pub fn r_011f64(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32) {
        let (routine, priority, depth) = MARK;
        let (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, depth, priority, routine);
        self.write8(a5 + SPRITE_FLAGS, 0x84);
        let index = self.read8(SETTING_UP);
        self.write8(a6 + TASK_OBJECT, index);
        (d5, d6, a5, a6)
    }

    /// `func_011F86`, a mark's routine: its frame 0xB (0xC when its
    /// object's flags have bit 4), its position its object's sprite's.
    /// (On screen 2 it compares the object's index + 1 with
    /// 0xFFAD3B, to no effect.)
    pub fn r_011f86(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let screen = self.read8(SCREEN);
        r.d0 = low_byte(r.d0, screen);
        if screen == SCREEN_2 {
            r.d0 = low_byte(r.d0, self.read8(r.a6 + TASK_OBJECT).wrapping_add(1));
        }
        self.write8(r.a5 + SPRITE_FRAME, 0x0B);
        r.d4 = low_byte(r.d4, self.read8(r.a6 + TASK_OBJECT));
        (_, r.a4, r.x) = self.r_001cfa(r.d4);
        if self.read8(r.a4 + OBJ_FLAGS) & MARK_B != 0 {
            self.write8(r.a5 + SPRITE_FRAME, 0x0C);
        }
        r.d6 = low_byte(r.d6, self.read8(r.a4 + OBJ_SPRITE));
        r.a6 = self.read32(r.a4 + OBJ_SPRITE_ADDR);
        let y = self.read16(r.a6 + SPRITE_Y);
        self.write16(r.a5 + SPRITE_Y, y);
        let x = self.read16(r.a6 + SPRITE_X);
        self.write16(r.a5 + SPRITE_X, x);
        r.c = false;
    }

    /// `func_01179C`: the picker: `RUNNING` set, a task with a sprite
    /// (`func_01185A`'s; flags 0x84, animation `PICKER_ANIM`, count
    /// 0x90).  By side: its y 0x68 (0xA8 when `ONE_SIDE` is side 1) and
    /// the number of each object on that side counted up (the last of
    /// them picked: d1's low byte when none is); else its y by
    /// `ONE_SIDE` + 0x10 and that object's number counted up and picked.
    /// The pick (0-7) in `PICK`, its x `PICK_X` by it.  Gives back d0,
    /// d2, d4, d5, d6, a0, a4 and a5.
    pub fn r_01179c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        self.write8(RUNNING, 1);
        let (routine, priority, depth) = PICKER;
        let (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, depth, priority, routine);
        self.write32(a6 + TASK_ANIM, PICKER_ANIM);
        self.write8(a5 + SPRITE_FLAGS, 0x84);
        self.write8(a6 + TASK_COUNT, 0x90);
        let (mut d0, mut d1, mut d2, mut d4, mut a0) = (d0, d1, d2, d4, a0);
        let a4;
        if self.read8(FLAGS) & BY_SIDE != 0 {
            let y = if self.read8(ONE_SIDE) != 0 { 0xA8 } else { 0x68 };
            self.write8(a6 + TASK_Y_LOW, y);
            d4 = low_word(d4, 0);
            let mut object;
            loop {
                (_, object, _) = self.r_001cfa(d4);
                d2 = low_byte(d2, self.read8(ONE_SIDE));
                let flags = self.read8(object + OBJ_FLAGS);
                d0 = low_byte(d0, flags ^ d2 as u8);
                if (flags ^ d2 as u8) & SIDE == 0 {
                    let number = flags.wrapping_add(1);
                    self.write8(object + OBJ_FLAGS, number);
                    d1 = low_byte(d1, number);
                }
                let index = (d4 as u8).wrapping_add(1);
                d4 = low_byte(d4, index);
                if index == self.read8(OBJECTS) {
                    break;
                }
            }
            d0 = low_byte(d0, d1 as u8);
            a4 = object;
        } else {
            let index = self.read8(ONE_SIDE);
            (_, a4, _) = self.r_001cfa(low_byte(d4, index));
            d4 = low_word(d4, index as i8 as u16);
            a0 = PICKER_Y;
            let y = self.read8(at(PICKER_Y, index)).wrapping_add(0x10);
            self.write8(a6 + TASK_Y_LOW, y);
            let number = self.read8(a4 + OBJ_FLAGS).wrapping_add(1);
            self.write8(a4 + OBJ_FLAGS, number);
            d0 = low_byte(d0, number);
        }
        let pick = d0 as u16 & 7;
        d0 = low_word(d0, pick);
        self.write8(PICK, pick as u8);
        let x = self.read8(PICK_X + u32::from(pick));
        self.write8(a6 + TASK_X_LOW, x);
        self.write8(a6 + TASK_X, 0);
        self.write8(a6 + TASK_Y, 1);
        (d0, d2, d4, d5, d6, a0, a4, a5)
    }

    /// `func_01185A`, the picker's routine: its animation
    /// (`func_0013D6`), moved up (`VELOCITY_Y`, `func_001AA2`), its
    /// sprite placed (`func_001B7E`); when its count runs out its
    /// sprite is let go (`func_00153A`), the sound sent, a task with a
    /// sprite made in its place (`func_0118B4`'s; flags 0x85,
    /// animation `PICKED_ANIM`), `RUNNING` cleared and the task ended.
    pub fn r_01185a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        self.write16(VELOCITY_Y, 0xFE00);
        (r.d0, r.d1, r.x, r.n, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        let count = self.read8(r.a6 + TASK_COUNT);
        self.write8(r.a6 + TASK_COUNT, count.wrapping_sub(1));
        r.x = count == 0;
        r.c = r.x;
        if count != 1 {
            return;
        }
        self.r_00153a(r.a6);
        r.d0 = low_byte(r.d0, PICKED_SOUND);
        self.r_0034b6(r.d0);
        let (routine, priority, depth) = PICKED;
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, priority, routine);
        self.r_00152a(r.a6);
        self.write32(r.a6 + TASK_ANIM, PICKED_ANIM);
        self.write8(r.a5 + SPRITE_FLAGS, 0x85);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        self.write8(RUNNING, 0);
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }

    /// `func_0118C0`, `func_0118B4`'s routine after its yield: its
    /// animation (`func_0013D6`).
    pub fn r_0118c0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }
}
