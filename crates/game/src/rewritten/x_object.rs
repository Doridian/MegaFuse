//! An object `func_01982C` sets up (entry 2 of the table at 0x1924E,
//! through `func_019824`; scripts stage-3-boss and exit-3-boss get
//! there) and its routines: it comes down (`func_0198E0`), bobs
//! (`func_019914`), plays an animation (`func_019938`), then walks
//! (`func_019988`): on a cell mark it either plays another animation
//! that sends four tasks out round it (`func_019A12`, `func_019A48`,
//! the tasks moved by `func_019B4E`), or, level with object 0, turns
//! to it (`func_01A022`), or makes a task over a cell of 1 near it
//! (`func_019E26`, `func_019FB6`).

use crate::{Game, Regs};

/// An object's fields: byte 0 (`OBJ_STATE`: 2 while its task over a
/// cell is out, back to 1 by that task), its routine (a long), the
/// long with a mark in its high nibble (`OBJ_SAVED`), its number
/// (0x0F), a byte set to 6 (0x11), its direction (bit 0 read here),
/// its position's low bytes, its animations (a long, `OBJ_ANIMS`) and
/// the one running (`OBJ_ANIM`), and two bytes (0x26, 0x27) the walk
/// reads.
const OBJ_STATE: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_SAVED: u32 = 0x06;
const OBJ_NUMBER: u32 = 0x0F;
const OBJ_11: u32 = 0x11;
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_X_LOW: u32 = 0x19;
const OBJ_Y: u32 = 0x1A;
const OBJ_Y_LOW: u32 = 0x1B;
const OBJ_ANIMS: u32 = 0x1C;
const OBJ_ANIM: u32 = 0x20;
const OBJ_26: u32 = 0x26;
const OBJ_27: u32 = 0x27;
/// An object's sprite record (its address, a long; its index).
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;

/// A task's fields: its position (words, x then y; the high bytes and
/// y's low byte read by `func_019B4E`), its angle and radius
/// (`func_019B4E`; the angle is also the frame `func_019FB6` gives a
/// sprite), the point it goes round (bytes), a sprite record's index
/// (`func_019FB6`) and its animation (a long).
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_Y_LOW: u32 = 0x11;
const TASK_ANGLE: u32 = 0x16;
const TASK_RADIUS: u32 = 0x17;
const TASK_CENTRE_X: u32 = 0x19;
const TASK_CENTRE_Y: u32 = 0x1A;
const TASK_SPRITE: u32 = 0x1B;
const TASK_ANIM: u32 = 0x20;

/// A sprite record's frame and flags, its position (words).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;

/// Sprite set 1's frames' table (`func_000776`).
const FRAMES_DC3D8: u32 = 0x0DC3D8;
const SET: u32 = 1;
/// A long kept across the load (`func_0094D8`), and the load's list
/// (`D_0198D0`, in the ROM).
const LONG_5810: u32 = 0xFF_5810;
const LOAD_LIST: u32 = 0xFFFF_92FA;
const LOADED: u32 = 0x0198D0;
/// What an object is set up with (`anim.rs`): its routine, and its
/// position (words, x then y).
const OBJ_INIT_ROUTINE: u32 = 0xFFFF_80D0;
const POSITION_X: u32 = 0xFFFF_9F04;
const POSITION_Y: u32 = 0xFFFF_9F06;
const START_X: u16 = 0x88;
const START_Y: u16 = 0xFFD0;
/// The object record set up (`func_001CFA`).
const OBJECT: u32 = 6;

/// A count: 0x10 when set up, counted down while it bobs and by
/// `func_01A022`, 0x40 when it turns; while it isn't 0 the walk goes
/// faster and the task `func_0198C2` stays.
const COUNT: u32 = 0xFFFF_ADE4;

/// The velocity's y (a word, `a_motion.rs`), and the speed (a word,
/// 8.8) the walk goes by (`func_004096`).
const VELOCITY_Y: u32 = 0xFFFF_9EC6;
const FALL: u16 = 0x400;
const SPEED: u32 = 0xFFFF_A960;
const SPEED_SLOW: u16 = 0x60;
const SPEED_FAST: u16 = 0x90;
/// Set to 1 while it comes down; cleared elsewhere (`q_object.rs`).
const BYTE_80DF: u32 = 0xFFFF_80DF;
/// Set to 0x80 when it starts to walk, and bit 0 of `SPARED` cleared.
const MODE: u32 = 0xFFFF_9540;
const SPARED: u32 = 0xFFFF_9548;

/// Object 0's position's low bytes (records 0x40 apart from 0xFFA432).
const OBJECT_0_X_LOW: u32 = 0xFFFF_A44B;
const OBJECT_0_Y_LOW: u32 = 0xFFFF_A44D;
/// Byte 0 of object 6 (0xFFA5B2): while it isn't 0, the tasks
/// `func_019B4E` moves stay on the screen.
const OBJECT_6_STATE: u32 = 0xFFFF_A5B2;
/// The object running (index, address; `a_objects.rs`) and the sprite
/// record kept (index, address).
const RUNNING: u32 = 0xFFFF_A42C;
const RUNNING_ADDR: u32 = 0xFFFF_A42E;
const SLOT_INDEX: u32 = 0xFFFF_8A57;
const SLOT_ADDR: u32 = 0xFFFF_8A5A;

/// A map cell (column, row; `anim.rs`), and the cell map's value the
/// search looks for.
const CELL_X: u32 = 0xFFFF_9F08;
const CELL_Y: u32 = 0xFFFF_9F09;
const CELL_WANTED: u8 = 1;
/// Two points (words, `d_angle.rs`): A, and B (`a_motion.rs`'s
/// position worked out apart).  The search keeps its first cell in B's
/// high bytes and how far it is in their low bytes; `func_019A48` puts
/// its centre in B's low bytes.
const A_X: u32 = 0xFFFF_9EE6;
const A_X_LOW: u32 = 0xFFFF_9EE7;
const A_Y: u32 = 0xFFFF_9EE8;
const A_Y_LOW: u32 = 0xFFFF_9EE9;
const B_X: u32 = 0xFFFF_9EFA;
const B_X_LOW: u32 = 0xFFFF_9EFB;
const B_Y: u32 = 0xFFFF_9EFE;
const B_Y_LOW: u32 = 0xFFFF_9EFF;
/// Where the cell's task is going (words, x then y; `func_019FB6` puts
/// its sprite half way).
const TARGET_X: u32 = 0xFFFF_ADE6;
const TARGET_Y: u32 = 0xFFFF_ADE8;
/// Bytes used for counting: the cell found (column x 2 + row), and
/// `func_019A48`'s tasks to make and their next angle.
const BYTE_80C4: u32 = 0xFFFF_80C4;
const BYTE_80C5: u32 = 0xFFFF_80C5;

/// Z80 commands (`func_0034B6`, `func_0034DE`).
const SOUND_64: u32 = 0x64;
const SOUND_67: u32 = 0x67;
const SOUND_0F: u32 = 0x0F;
const SOUND_68: u32 = 0x68;
const SOUND_84: u32 = 0x84;
const SOUND_1C: u32 = 0x1C;

/// Animations and frames.
const ANIM_DE288: u32 = 0x0DE288;
const ANIM_DE2AA: u32 = 0x0DE2AA;
const ANIM_DE342: u32 = 0x0DE342;
const ANIM_DE35A: u32 = 0x0DE35A;
const ANIMS_1A40A: u32 = 0x01A40A;
const FRAME_SOUND: u8 = 0x60;
const FRAME_TURN_LEFT: u8 = 0xAA;
const FRAME_TURN_RIGHT: u8 = 0xAB;

/// The routines.
const COMING_DOWN: u32 = 0x0198E0;
const BOBBING: u32 = 0x019914;
const PLAYING: u32 = 0x019938;
const WALKING: u32 = 0x019988;
const SENDING: u32 = 0x019A12;
const TURNING: u32 = 0x01A022;
const WAITING: u32 = 0x019E26;
const COUNT_TASK: u32 = 0x0198C2;
const ROUND_TASK: u32 = 0x019ACE;
const CELL_TASK: u32 = 0x019E46;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_019824`: `func_01982C`.
    pub fn r_019824(&mut self, regs: &mut Regs) {
        let r = regs;
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_01982c(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        r.unwound = false;
    }

    /// `func_01982C`: sprite set `SET`'s frames `FRAMES_DC3D8`
    /// (`func_000776`), the list `LOADED` loaded (`func_0094D8`,
    /// `LONG_5810` kept), `func_0094B2`; object `OBJECT` set up at
    /// `START_X`, `START_Y` with `func_0198E0` (`func_001CFA`,
    /// `func_0012A0`): its sprite's flags 0x81, its number 0x21,
    /// `ANIM_DE288` a frame on (`func_001336`); `COUNT` 0x10 and the
    /// task `func_0198C2` with a sprite (`func_001222`): flags 0xC1,
    /// frame 0xA9, at 0x80, 0x60 (the bytes).  Gives back every
    /// register, X and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01982c(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let d0 = self.r_000776(SET, FRAMES_DC3D8, a1);
        let kept = self.read32(LONG_5810);
        self.write32(LOAD_LIST, LOADED);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c) =
            self.r_0094d8(d0, d1, d2, d3, d4, d5, d6, d7, FRAMES_DC3D8, a1, a2, a3, a4, a5, a6, x);
        if self.restarted.is_some() {
            return (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c);
        }
        self.write32(LONG_5810, kept);
        self.r_0094b2();
        self.write16(POSITION_X, START_X);
        self.write16(POSITION_Y, START_Y);
        self.write32(OBJ_INIT_ROUTINE, COMING_DOWN);
        let (d4, obj, x) = self.r_001cfa(OBJECT);
        let (d0, d5, sprite, x) = self.r_0012a0(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, obj, a5, x);
        self.write8(sprite + SPRITE_FLAGS, 0x81);
        self.write8(obj + OBJ_NUMBER, 0x21);
        self.write32(obj + OBJ_ANIM, ANIM_DE288);
        let (d0, d1, d2, d5, a0, a5, _, _, _) = self.r_001336(d0, d1, d2, d5, obj, x);
        self.write8(COUNT, 0x10);
        let (d5, d6, sprite, task, x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, obj, a5, 0xF0, 0x32, COUNT_TASK);
        self.write8(sprite + SPRITE_FLAGS, 0xC1);
        self.write8(sprite + SPRITE_FRAME, 0xA9);
        self.write8(sprite + SPRITE_X_LOW, 0x80);
        self.write8(sprite + SPRITE_Y_LOW, 0x60);
        (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, obj, sprite, task, x, false)
    }

    /// `func_001B32` (as `func_019934`, the routines' end): the sprite
    /// placed for the object.
    fn x_place(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
    }

    /// `func_019934`: `func_001B32`.
    pub fn r_019934(&mut self, d1: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        self.r_001b32(d1, a4, a5)
    }

    /// `func_0198E0` (an object's routine): `BYTE_80DF` 1, moved down
    /// by `FALL` (`func_0019DA`); once its y is 0x40 or more,
    /// `func_019914` from now on (and now) with sound `SOUND_64`.  Else
    /// placed.
    pub fn r_0198e0(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        self.write8(BYTE_80DF, 1);
        self.write16(VELOCITY_Y, FALL);
        (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        if self.read8(r.a4 + OBJ_Y) == 0 {
            let y = self.read8(r.a4 + OBJ_Y_LOW);
            r.d0 = low_byte(r.d0, y);
            if y >= 0x40 {
                self.write32(r.a4 + OBJ_ROUTINE, BOBBING);
                r.d0 = low_byte(r.d0, SOUND_64 as u8);
                self.r_0034b6(r.d0);
                self.r_019914(r);
                return;
            }
        }
        self.x_place(r);
    }

    /// `func_019914` (an object's routine): its y 0x40 or 0x41 by
    /// `COUNT`'s bit 0, `COUNT` counted down; at 0, `func_019938` from
    /// now on.  Placed.
    pub fn r_019914(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let count = self.read8(COUNT);
        let y = (count & 1) + 0x40;
        r.d0 = low_byte(r.d0, y);
        self.write8(r.a4 + OBJ_Y_LOW, y);
        let count = count.wrapping_sub(1);
        self.write8(COUNT, count);
        if count == 0 {
            self.write32(r.a4 + OBJ_ROUTINE, PLAYING);
        }
        self.x_place(r);
    }

    /// `func_019938` (an object's routine): sound `SOUND_67` while its
    /// frame is `FRAME_SOUND`; the animation a frame on
    /// (`func_001336`).  When it ends, `MODE` 0x80 and bit 0 of
    /// `SPARED` cleared, its animations `ANIMS_1A40A`, `OBJ_11` 6, the
    /// mark 0x90, `OBJ_27` 1, sound `SOUND_0F`, and `func_019988` from
    /// now on (and now).  Else placed.
    pub fn r_019938(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let frame = self.read8(r.a5 + SPRITE_FRAME);
        r.d0 = low_byte(r.d0, frame);
        if frame == FRAME_SOUND {
            r.d0 = low_byte(r.d0, SOUND_67 as u8);
            self.r_0034b6(r.d0);
        }
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if !r.c {
            self.x_place(r);
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, WALKING);
        self.write8(MODE, 0x80);
        let spared = self.read8(SPARED);
        self.write8(SPARED, spared & !1);
        self.write32(r.a4 + OBJ_ANIMS, ANIMS_1A40A);
        self.write8(r.a4 + OBJ_11, 6);
        self.write8(r.a4 + OBJ_SAVED, 0x90);
        self.write8(r.a4 + OBJ_27, 1);
        r.d0 = low_byte(r.d0, SOUND_0F as u8);
        r.c = false;
        self.r_0034b6(r.d0);
        self.r_019988(r);
    }

    /// `func_019988` (an object's routine), the walk: `SPEED`
    /// `SPEED_FAST` or `SPEED_SLOW` by its direction's bit 0, doubled
    /// (and `OBJ_26` 0, else 0xC) while `COUNT` isn't 0; a frame of it
    /// (`func_004096`; C: `func_01A04C`), sound `SOUND_1C` for it
    /// (`func_0034DE`), its sprite record at depth 8 (`func_0007D2`).
    /// Without the mark: `ANIM_DE2AA`, sound `SOUND_68` and
    /// `func_019A12` from now on (and now).  With it, when `COUNT` is
    /// 0 and its y is object 0's (low bytes): `COUNT` 0x40, frame
    /// `FRAME_TURN_LEFT` (`FRAME_TURN_RIGHT` when its x is under object
    /// 0's) and `func_01A022` from now on (and now).  Else the cell
    /// search (`x_search`).
    pub fn r_019988(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        let speed = if self.read8(r.a4 + OBJ_DIRECTION) & 1 != 0 { SPEED_FAST } else { SPEED_SLOW };
        self.write16(SPEED, speed);
        self.write8(r.a4 + OBJ_26, 0x0C);
        let count = self.read8(COUNT);
        r.d0 = low_byte(r.d0, count);
        if count != 0 {
            self.write8(r.a4 + OBJ_26, 0);
            self.write16(SPEED, speed << 1);
        }
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c) =
            self.r_004096(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        if r.c {
            (r.d6, r.a6, r.x, r.c) = self.r_01a04c(r.d4, r.d5, r.d6, r.a4);
            return;
        }
        r.d0 = SOUND_1C;
        (r.d1, r.d2, r.x) = self.r_0034de(r.d0, r.d1, r.d2, r.a4);
        r.d0 = 8;
        r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
        let mark = self.read8(r.a4 + OBJ_SAVED) & 0xF0;
        r.d0 = low_byte(r.d0, mark);
        if mark == 0 {
            self.write32(r.a4 + OBJ_ANIM, ANIM_DE2AA);
            self.write32(r.a4 + OBJ_ROUTINE, SENDING);
            r.d0 = low_byte(r.d0, SOUND_68 as u8);
            r.c = false;
            self.r_0034b6(r.d0);
            self.r_019a12(r);
            return;
        }
        let count = self.read8(COUNT);
        r.d0 = low_byte(r.d0, count);
        if count == 0 {
            let y = self.read8(r.a4 + OBJ_Y_LOW);
            r.d0 = low_byte(r.d0, y);
            if y == self.read8(OBJECT_0_Y_LOW) {
                self.write8(COUNT, 0x40);
                self.write8(r.a5 + SPRITE_FRAME, FRAME_TURN_LEFT);
                let x = self.read8(r.a4 + OBJ_X_LOW);
                r.d0 = low_byte(r.d0, x);
                if x < self.read8(OBJECT_0_X_LOW) {
                    self.write8(r.a5 + SPRITE_FRAME, FRAME_TURN_RIGHT);
                }
                self.write32(r.a4 + OBJ_ROUTINE, TURNING);
                r.c = false;
                self.r_01a022(r);
                return;
            }
        }
        self.x_search(r);
    }

    /// The cell search (0x19CAC): from its cell (`func_0015E6`) one
    /// left and one down, the 3 x 2 cells to the right and down from
    /// there, column by column, for the first of `CELL_WANTED` in the
    /// cell map (`func_00167A`): then `x_found`.  None: d0's low byte
    /// 3, X and C clear.
    fn x_search(&mut self, r: &mut Regs) {
        (r.d0, r.x) = self.r_0015e6(r.a4);
        let column = self.read8(CELL_X).wrapping_sub(1);
        self.write8(CELL_X, column);
        let row = self.read8(CELL_Y).wrapping_add(1);
        self.write8(CELL_Y, row);
        self.write8(B_X, column);
        self.write8(B_Y, row);
        for across in 0..3u8 {
            self.write8(B_X_LOW, across);
            for down in 0..2u8 {
                self.write8(B_Y_LOW, down);
                let column = self.read8(B_X).wrapping_add(across);
                self.write8(CELL_X, column);
                let row = self.read8(B_Y).wrapping_add(down);
                self.write8(CELL_Y, row);
                (r.a0, r.x) = self.r_00167a();
                let cell = self.read8(r.a0);
                r.d0 = low_byte(r.d0, cell);
                if cell == CELL_WANTED {
                    self.x_found(r, across * 2 + down);
                    return;
                }
                r.d0 = low_byte(r.d0, down + 1);
            }
            self.write8(B_Y_LOW, 2);
        }
        self.write8(B_X_LOW, 3);
        r.d0 = low_byte(r.d0, 3);
        r.x = false;
        r.c = false;
    }

    /// The cell found (0x19D10, `which` its column x 2 + row, kept in
    /// `BYTE_80C4`): a task `func_019E46` with a sprite (`func_001222`)
    /// and a second sprite record (`func_000796`, depth 9; its index
    /// the task's `TASK_SPRITE`), both with flags 0x81.  It starts at
    /// the object (`func_001A10`, `func_001A18`), 0x17 right of it
    /// (frame 0xAC, `ANIM_DE342`) for the third column, else 0x16 left
    /// (0xAD, `ANIM_DE35A`), 0x1E down, kept in `TARGET_X`,
    /// `TARGET_Y`, and goes round the cell's middle; placed
    /// (`func_001B7E`), a frame of its animation (`func_0013D6`),
    /// `func_019FB6`.  The object running again (`RUNNING`), its frame
    /// 0x56 (0x57 left), `OBJ_STATE` 2 and `func_019E26` from now on.
    fn x_found(&mut self, r: &mut Regs, which: u8) {
        r.d0 = low_byte(r.d0, which);
        self.write8(BYTE_80C4, which);
        self.r_001a10(r.a4);
        self.r_001a18(r.a4);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 7, 0x32, CELL_TASK);
        let (d5, a5) = (r.d5, r.a5);
        r.d0 = 9;
        let (index, sprite, _, _) =
            self.r_000796(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
        self.write8(r.a6 + TASK_SPRITE, index as u8);
        self.write8(sprite + SPRITE_FLAGS, 0x81);
        (r.d5, r.a5) = (d5, a5);
        self.write8(r.a5 + SPRITE_FLAGS, 0x81);
        let y = self.read16(B_Y).wrapping_add(0x1E);
        self.write16(TARGET_Y, y);
        self.write16(r.a6 + TASK_Y, y);
        let column = self.read8(CELL_X);
        self.write8(r.a6 + TASK_CENTRE_X, (column << 4) | 8);
        let row = self.read8(CELL_Y);
        r.d0 = low_byte(r.d0, (row << 4) | 8);
        self.write8(r.a6 + TASK_CENTRE_Y, (row << 4) | 8);
        let (x, angle, anim, frame) = if self.read8(BYTE_80C4) < 3 {
            (self.read16(B_X).wrapping_sub(0x16), 0xAD, ANIM_DE35A, 0x57)
        } else {
            (self.read16(B_X).wrapping_add(0x17), 0xAC, ANIM_DE342, 0x56)
        };
        r.d0 = low_word(r.d0, x);
        self.write16(TARGET_X, x);
        self.write16(r.a6 + TASK_X, x);
        self.write8(r.a6 + TASK_ANGLE, angle);
        self.write32(r.a6 + TASK_ANIM, anim);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        (r.d0, r.x) = self.r_019fb6(r.d0, r.d5, r.a6);
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        r.d5 = low_byte(r.d5, self.read8(r.a4 + OBJ_SPRITE));
        r.a5 = self.read32(r.a4 + OBJ_SPRITE_ADDR);
        self.write8(r.a5 + SPRITE_FRAME, frame);
        self.write32(r.a4 + OBJ_ROUTINE, WAITING);
        self.write8(r.a4 + OBJ_STATE, 2);
        r.c = false;
    }

    /// `func_019A12` (an object's routine): its cell (`func_0015E6`),
    /// `func_006E7C` (C: `func_01A04C`), the animation a frame on
    /// (`func_001336`); when it ends, sound `SOUND_84`, the four tasks
    /// (`func_019A48`), the mark 0x90, `COUNT` 0 and `func_019988`
    /// from now on.
    pub fn r_019a12(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        (r.d0, r.x) = self.r_0015e6(r.a4);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e7c(r.d1, r.d2, r.a0, r.a4, r.a5);
        if r.c {
            (r.d6, r.a6, r.x, r.c) = self.r_01a04c(r.d4, r.d5, r.d6, r.a4);
            return;
        }
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
        if r.c {
            r.d0 = low_byte(r.d0, SOUND_84 as u8);
            self.r_0034b6(r.d0);
            (r.d0, r.x) = self.r_019a48(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            self.write32(r.a4 + OBJ_ROUTINE, WALKING);
            self.write8(r.a4 + OBJ_SAVED, 0x90);
            self.write8(COUNT, 0);
            r.c = false;
        }
    }

    /// `func_019A48`: four tasks `func_019ACE` with sprites
    /// (`func_001222`, flags 0x81) going round the object (its x, its
    /// y + 8: `B_X_LOW`, `B_Y_LOW`) from radius 0, their angles 10
    /// apart from 0x10 - 1 - the angle of object 0 from it
    /// (`func_0041D0`).  The registers are kept but d0 (its low byte
    /// the next angle) and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_019a48(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, bool) {
        self.r_001a10(a4);
        let y = self.read8(a4 + OBJ_Y_LOW).wrapping_add(8);
        self.write8(B_Y_LOW, y);
        self.write8(B_Y, 0);
        let x0 = self.read8(OBJECT_0_X_LOW);
        self.write8(A_X_LOW, x0);
        let y0 = self.read8(OBJECT_0_Y_LOW);
        self.write8(A_Y_LOW, y0);
        self.write8(A_X, 0);
        self.write8(A_Y, 0);
        let d0 = self.r_0041d0(low_byte(d0, y), d1);
        let mut angle = (!(d0 as u8)).wrapping_add(0x10);
        let mut d0 = low_byte(d0, angle);
        self.write8(BYTE_80C5, angle);
        self.write8(BYTE_80C4, 4);
        let (mut d5, mut d6, mut a5) = (d5, d6, a5);
        loop {
            let task;
            (d5, d6, a5, task, _, _) =
                self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 6, 0x32, ROUND_TASK);
            self.write8(a5 + SPRITE_FLAGS, 0x81);
            self.write8(task + TASK_RADIUS, 0);
            angle = self.read8(BYTE_80C5);
            self.write8(task + TASK_ANGLE, angle);
            angle = angle.wrapping_sub(0x0A);
            d0 = low_byte(d0, angle);
            self.write8(BYTE_80C5, angle);
            let x = self.read8(B_X_LOW);
            self.write8(task + TASK_CENTRE_X, x);
            let y = self.read8(B_Y_LOW);
            self.write8(task + TASK_CENTRE_Y, y);
            let left = self.read8(BYTE_80C4);
            self.write8(BYTE_80C4, left.wrapping_sub(1));
            if left == 1 {
                return (d0, false);
            }
        }
    }

    /// `func_019B4E` (from `func_019ACE`): the task at the point round
    /// its centre by its angle and radius (`func_017780` for x,
    /// `func_01777C` for y, added to the centre's bytes).  Placed
    /// (`func_001B7E`, with the kept sprite record `SLOT_INDEX`,
    /// `SLOT_ADDR`) while both high bytes are 0, y's low byte under
    /// 0xE0 and `OBJECT_6_STATE` isn't 0; else the caller's frame is
    /// dropped (d0 its return address) and the task ends
    /// (`func_00121A`).  Gives back d0, d1, d5, d6, a5, X, C and
    /// whether it was dropped.
    #[allow(clippy::too_many_arguments)]
    pub fn r_019b4e(&mut self, d0: u32, d1: u32, d3: u32, d5: u32, d6: u32, a5: u32, a6: u32, ret: u32) -> (u32, u32, u32, u32, u32, bool, bool, bool) {
        let angle = self.read8(a6 + TASK_ANGLE);
        let radius = self.read8(a6 + TASK_RADIUS);
        let d1 = low_byte(d1, radius);
        let d0 = self.r_017780(low_byte(d0, angle), d1, d3);
        let centre = self.read8(a6 + TASK_CENTRE_X);
        let d1 = low_word(d1, u16::from(centre));
        let x = (d0 as u16).wrapping_add(u16::from(centre));
        let d0 = low_word(d0, x);
        self.write16(a6 + TASK_X, x);
        let angle = self.read8(a6 + TASK_ANGLE);
        let radius = self.read8(a6 + TASK_RADIUS);
        let d1 = low_byte(d1, radius);
        let d0 = self.r_01777c(low_byte(d0, angle), d1, d3);
        let centre = self.read8(a6 + TASK_CENTRE_Y);
        let d1 = low_word(d1, u16::from(centre));
        let (y, x) = (d0 as u16).overflowing_add(u16::from(centre));
        self.write16(a6 + TASK_Y, y);
        let high = self.read8(a6 + TASK_Y) | self.read8(a6 + TASK_X);
        if high == 0 {
            let y_low = self.read8(a6 + TASK_Y_LOW);
            if y_low < 0xE0 && self.read8(OBJECT_6_STATE) != 0 {
                let d5 = low_byte(d5, self.read8(SLOT_INDEX));
                let a5 = self.read32(SLOT_ADDR);
                let (d0, x, c) = self.r_001b7e(a5, a6);
                return (d0, d1, d5, d6, a5, x, c, false);
            }
        }
        let (d6, x, c) = self.r_00121a(d5, d6, x);
        (ret, d1, d5, d6, a5, x, c, true)
    }

    /// `func_019E26` (an object's routine): its cell (`func_0015E6`),
    /// `func_006E7C` (C: `func_01A04C`); once `OBJ_STATE` is back to 1,
    /// `func_019988` from now on.  C: `OBJ_STATE` was 0.
    pub fn r_019e26(&mut self, regs: &mut Regs) {
        let r = regs;
        r.unwound = false;
        (r.d0, r.x) = self.r_0015e6(r.a4);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e7c(r.d1, r.d2, r.a0, r.a4, r.a5);
        if r.c {
            (r.d6, r.a6, r.x, r.c) = self.r_01a04c(r.d4, r.d5, r.d6, r.a4);
            return;
        }
        let state = self.read8(r.a4 + OBJ_STATE);
        r.c = state < 1;
        if state == 1 {
            self.write32(r.a4 + OBJ_ROUTINE, WALKING);
            r.c = false;
        }
    }

    /// `func_019FB6`: the task's sprite record `TASK_SPRITE`
    /// (`func_000B5A`) half way between the task and the target
    /// (`TARGET_X`, `TARGET_Y`), 8 left and 0x20 down (kept in `B_X`,
    /// `B_Y`), its frame the task's `TASK_ANGLE`.  d5 and a5 are kept.
    /// Gives back d0 (its low word the y) and X.
    pub fn r_019fb6(&mut self, d0: u32, d5: u32, a6: u32) -> (u32, bool) {
        let x = (self.read16(a6 + TASK_X).wrapping_add(self.read16(TARGET_X)) >> 1).wrapping_sub(8);
        self.write16(B_X, x);
        let y = (self.read16(a6 + TASK_Y).wrapping_add(self.read16(TARGET_Y)) >> 1).wrapping_add(0x20);
        self.write16(B_Y, y);
        let index = self.read8(a6 + TASK_SPRITE);
        let (_, sprite, x_flag) = self.r_000b5a(low_byte(d5, index));
        let x = self.read16(B_X);
        self.write16(sprite + SPRITE_X, x);
        let y = self.read16(B_Y);
        self.write16(sprite + SPRITE_Y, y);
        let frame = self.read8(a6 + TASK_ANGLE);
        self.write8(sprite + SPRITE_FRAME, frame);
        (low_word(d0, y), x_flag)
    }
}
