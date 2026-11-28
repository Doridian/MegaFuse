//! A chain of objects 8 to 0xD (0x147B4-0x15000): `func_014886` sets
//! them up, object 8 runs the chain (`func_014B80` and the routines it
//! goes on as), the others follow it (`func_0093B8`).  Each link has an
//! angle (`OBJ_ANGLE`, 0x6B-0x95) and is put at the `REACH` from the
//! one before (`func_0149CA`, `func_014B14`; the first from object 7's
//! position 0x18 up), the angles turning together through a ring of
//! the last 8 steps (`func_014932`).  `REACH` grows to 0xA00 and
//! shrinks while `SHRINK` lasts (`func_014B46`).  `STOPPED` (the walker
//! of `s_walker.rs`) makes it throw pieces down its links
//! (`func_014C5E`-`func_014D88`) or shrink (`func_014E0E`); when
//! object 7 is hit (`func_006E7C`) through it often enough
//! (`func_014F46`), object 7 starts `func_014380` (`s_pieces.rs`) and
//! the links fall (`func_014E3A`-`func_014F2C`).  Also two tasks:
//! `func_0147B4`'s, which shows a sprite at an object
//! (`func_0147DC`), and `func_014842`'s for `func_00B8C0`.

use crate::{Game, Regs};

/// The objects' fields: the routine (a long), the sprite record (its
/// address, a long, and index), a count of hits (`func_014F46`), a
/// count (`func_014CCC`'s pieces), the angle, the base frame, the
/// position (words: x, y; the y's low byte), the velocity
/// `func_014EA4` keeps (y: bytes 0x20-0x21, x: 0x26-0x27, the order of
/// `VELOCITY_*`), the timer, the turn (+1 or -1), the rate (a word, its
/// two bytes) and the offset.
const OBJ_ID: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_HITS: u32 = 0x11;
const OBJ_COUNT: u32 = 0x14;
const OBJ_ANGLE: u32 = 0x16;
const OBJ_FRAME: u32 = 0x17;
const OBJ_X: u32 = 0x18;
const OBJ_Y: u32 = 0x1A;
const OBJ_Y_LOW: u32 = 0x1B;
const OBJ_VELOCITY_Y: u32 = 0x20;
const OBJ_VELOCITY_Y_FRACTION: u32 = 0x21;
const OBJ_TIMER: u32 = 0x24;
const OBJ_TURN: u32 = 0x25;
const OBJ_VELOCITY_X: u32 = 0x26;
const OBJ_VELOCITY_X_FRACTION: u32 = 0x27;
const OBJ_RATE: u32 = 0x28;
const OBJ_RATE_LOW: u32 = 0x29;
const OBJ_OFFSET: u32 = 0x2A;
/// Tasks' fields: words 0x10 and 0x12, a long 0xC (`func_014842`'s
/// for `func_00B8C0`); bytes 0x14-0x18; a frame shown on odd frames
/// and the object (`func_0147DC`); the velocity (bytes 0x18-0x1B, as
/// the objects' in `s_walker.rs`); the offset (a byte, then the
/// animation: a long).
const TASK_10: u32 = 0x10;
const TASK_12: u32 = 0x12;
const TASK_0C: u32 = 0x0C;
const TASK_14: u32 = 0x14;
const TASK_15: u32 = 0x15;
const TASK_16: u32 = 0x16;
const TASK_17: u32 = 0x17;
const TASK_18: u32 = 0x18;
const TASK_SHOWN: u32 = 0x16;
const TASK_OBJECT: u32 = 0x17;
const TASK_VELOCITY_X_FRACTION: u32 = 0x18;
const TASK_VELOCITY_X: u32 = 0x19;
const TASK_VELOCITY_Y_FRACTION: u32 = 0x1A;
const TASK_VELOCITY_Y: u32 = 0x1B;
const TASK_OFFSET: u32 = 0x20;
/// A sprite record's byte 3, frame, flags, position (words).
const SPRITE_BYTE_3: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_Y: u32 = 0x0A;

/// The frame count.
const FRAME: u32 = 0xFF80D8;
/// The running task's index (`task.rs`).
const CURRENT_TASK: u32 = 0xFF80EE;
/// What `func_0012A0` sets an object up with (a long).
const OBJ_INIT_2: u32 = 0xFF80D0;
/// The object being run (its index and address) and its sprite
/// record's (index, address).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// A sprite record (index, address): the walker's (`s_walker.rs`);
/// the chain is left running with it.
const LEADER: u32 = 0xFFA640;
const LEADER_ADDR: u32 = 0xFFA63C;
/// A sprite record whose byte 3 the links' get (`func_0149CA`).
const BYTE_3_FROM: u32 = 0xFFA5FC;
/// The velocity (bytes: x, its fraction, y, its fraction) and the
/// speed (a word) `func_00410C` makes it of.
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_X_FRACTION: u32 = 0xFF9EC5;
const VELOCITY_Y: u32 = 0xFF9EC6;
const VELOCITY_Y_FRACTION: u32 = 0xFF9EC7;
const SPEED: u32 = 0xFF9F00;
/// A position (words, x then y): `func_001542` and `func_001300`.
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
/// A word set to 0xD8 by `func_014FE6`.
const WORD_9EE8: u32 = 0xFF9EE8;
/// How far the links are apart (a word, the speed of `func_00410C`;
/// its high byte read too); a count of links (`func_014886`,
/// `func_014932`); the ring's place (0-7); the frames left to shrink.
const REACH: u32 = 0xFF9674;
const LINKS: u32 = 0xFF9676;
const RING_AT: u32 = 0xFF9677;
const SHRINK: u32 = 0xFF9678;
/// The y step added to each link's y (`func_0149CA`: 0, 4, 8...).
const SWAY: u32 = 0xFF967A;
/// The ring of the last 8 turns (bytes).
const RING: u32 = 0xFFFF96BA;
const RING_SIZE: u32 = 8;
/// Bytes: the links' offset, `func_014C5E`'s task's.
const BYTE_A61C: u32 = 0xFFA61C;
const BYTE_A65C: u32 = 0xFFA65C;
/// `s_walker.rs`'s count of stops: 1 makes the chain throw pieces, more
/// shrink it; cleared after.
const STOPPED: u32 = 0xFFAD56;
/// `s_pieces.rs`'s count of gone tasks: cleared when object 7 starts;
/// the links fall when it is negative (0xFF).
const GONE: u32 = 0xFFAD57;
/// `func_014E3A`'s angles (by 0x2A).
const FALL_ANGLE: u32 = 0xFFAABD;
/// `func_01481A`'s count (it ends at 0x20) and the byte it moves.
const COUNT_9EB9: u32 = 0xFF9EB9;
const SHAKE_Y: u32 = 0xFFA60D;
/// Bit 6 of one, bit 0 of the other, set when object 7 starts.
const BITS_9540: u32 = 0xFF9540;
const SPARED: u32 = 0xFF9548;

/// The objects: 2, 7, the links 8 (the first) to 0xD; the one past.
const OBJECT_2: u8 = 2;
const OBJECT_7: u8 = 7;
const FIRST: u8 = 8;
const LAST: u8 = 0xD;
const PAST: u8 = 0xE;
/// What a link is set up with: `OBJ_INIT_2`, flags, angle, frame; the
/// first's frame and hits; the others' routine, their task's.
const LINK_INIT: u32 = 0x013F58;
const LINK_FLAGS: u8 = 0x81;
const MIDDLE: u8 = 0x80;
const LINK_FRAME: u8 = 0x73;
const FIRST_FRAME: u8 = 0x70;
const FIRST_HITS: u8 = 6;
const FOLLOW: u32 = 0x0093B8;
const SHOW_FLAGS: u8 = 0x84;
const SHOW_FRAME: u8 = 0xE0;
const START_SOUND: u8 = 0x72;
/// The angles' range.
const LOWEST: u8 = 0x6B;
const HIGHEST: u8 = 0x95;
/// A link's frame: its base plus 1 under `FRAME_1_UNDER`, plus 0 under
/// `FRAME_0_UNDER`, else plus 2.
const FRAME_1_UNDER: u8 = 0x76;
const FRAME_0_UNDER: u8 = 0x8A;
/// The first link is this far up from object 7.
const FIRST_UP: u16 = 0x18;
/// The links' depth (`func_0007D2`) and sway step.
const LINK_DEPTH: u32 = 5;
const SWAY_STEP: u8 = 4;
/// `REACH`: its most, the steps up and down.
const REACH_MOST: u16 = 0xA00;
const REACH_GROW: u16 = 0x30;
const REACH_SHRINK: u16 = 0x60;
/// The first link's frames: turning, swinging, shrinking, falling.
const TURN_FRAME: u8 = 0x76;
const SWING_FRAME: u8 = 0x70;
const SHRINK_FRAME: u8 = 0x7A;
const FALL_FRAME: u8 = 0x7A;
/// `func_014BC4`'s wait at either end, the pieces' wait and count
/// (`func_014C5E`, `func_014CCC`).
const END_WAIT: u8 = 8;
const THROW_WAIT: u8 = 0x20;
const THROW_FRAME: u8 = 0x76;
const PIECE_WAIT: u8 = 2;
const PIECES: u8 = 0x40;
/// `func_014C5E`'s task: routine, depth, priority, flags.
const ROUTINE_14DD4: u32 = 0x014DD4;
const FOLLOWER_DEPTH: u8 = 3;
const FOLLOWER_PRIORITY: u8 = 0x43;
const FOLLOWER_FLAGS: u8 = 0x81;
/// The follower's frame is the walker's plus 7.
const FOLLOWER_FRAME: u8 = 7;
/// `func_014D18`'s pieces: speed, routine, depth, priority, flags,
/// animation.
const PIECE_SPEED: u16 = 0x400;
const ROUTINE_14D88: u32 = 0x014D88;
const PIECE_DEPTH: u8 = 4;
const PIECE_PRIORITY: u8 = 0x42;
const PIECE_FLAGS: u8 = 0x81;
const PIECE_ANIM: u32 = 0x0E863E;
/// What `func_014D88` tells `func_003554`, the cell it stops a put
/// task on (`func_005E0E`).
const CELL_HIT: u32 = 8;
const CELL_PUT: u8 = 1;
/// `func_014E3A`'s fall: speed, angle step, wait; the rates by link
/// (the bytes at 0x14E98, read from `func_014E3A` + 0x4E by the link
/// times 2; they go to `OBJ_RATE` swapped, the first byte into the
/// low one).
const FALL_SPEED: u16 = 0xA0;
const FALL_ANGLE_STEP: u8 = 0x2A;
const FALL_WAIT: u8 = 2;
const FALL_RATES: u32 = 0x014E88;
/// `func_014ED2`: the rate step, the bounce's rate, the wait after,
/// the flags.
const FALL_RATE_STEP: u32 = 0x20;
const BOUNCE_RATE: u16 = 0xFE00;
const GONE_WAIT: u8 = 0x40;
const GONE_FLAGS: u8 = 0xC1;
/// `func_014F46`: what `func_0034DE` is told; the shrink on a hit;
/// the Z80's command, object 7's turn, wait and count when it starts;
/// the task then (routine, priority).
const HIT_CELL: u32 = 0x10;
const HIT_SHRINK: u8 = 0x20;
const LOST_SOUND: u8 = 0xF9;
const LOST_TURN: u8 = 0x0A;
const SEVEN_TIMER: u8 = 1;
const SEVEN_COUNT: u8 = 0x30;
const ROUTINE_14FE6: u32 = 0x014FE6;
const LOST_PRIORITY: u8 = 0x42;
/// `func_0147B4`'s task: routine, depth, priority, flags, frame, object.
const ROUTINE_147DC: u32 = 0x0147DC;
const SHOW_DEPTH: u8 = 0xFE;
const SHOW_PRIORITY: u8 = 0x43;
const SHOW_1_FLAGS: u8 = 0x81;
const SHOW_1_FRAME: u8 = 0x79;
/// `func_014842`'s task: routine, priority, words, the list (D_014880),
/// bytes.
const ROUTINE_B8C0: u32 = 0x00B8C0;
const B8C0_PRIORITY: u8 = 0x20;
const B8C0_10: u16 = 0x4100;
const B8C0_12: u16 = 0x003F;
const B8C0_LIST: u32 = 0x014880;
/// `func_01481A`'s end.
const COUNT_9EB9_END: u8 = 0x20;
/// The routines the chain goes on as.
const ROUTINE_14B80: u32 = 0x014B80;
const ROUTINE_14BC4: u32 = 0x014BC4;
const ROUTINE_14C5E: u32 = 0x014C5E;
const ROUTINE_14CCC: u32 = 0x014CCC;
const ROUTINE_14E0E: u32 = 0x014E0E;
const ROUTINE_14E3A: u32 = 0x014E3A;
const ROUTINE_14ED2: u32 = 0x014ED2;
const ROUTINE_14F2C: u32 = 0x014F2C;
const ROUTINE_14380: u32 = 0x014380;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// The byte at `at` one less; true when it got to 0.
    fn s_tick(&mut self, at: u32) -> bool {
        let left = self.read8(at).wrapping_sub(1);
        self.write8(at, left);
        left == 0
    }

    /// `func_014B46` on the family's registers.
    fn s_reach(&mut self, r: &mut Regs) {
        r.d0 = self.r_014b46(r.d0);
    }

    /// `func_014932` on the family's registers, with the turn in d0
    /// (Z: it is 0).
    fn s_turn(&mut self, r: &mut Regs, z: bool) {
        (r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5) = self.r_014932(r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, z);
    }

    /// The turn (`OBJ_TURN`) into d0 and `func_014932` with it.
    fn s_turn_by_own(&mut self, r: &mut Regs) {
        let turn = self.read8(r.a4 + OBJ_TURN);
        r.d0 = low_byte(r.d0, turn);
        self.s_turn(r, turn == 0);
    }

    /// `func_0149CA` on the family's registers.
    fn s_links(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.a4, r.a5, r.x, r.c) = self.r_0149ca(r.d0, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_014F46` on the family's registers.
    fn s_hits(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d4, r.d6, r.a0, r.a4, r.a6, r.x, r.c) = self.r_014f46(r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6);
    }

    /// `func_014F3E` on the family's registers.
    fn s_place_fallen(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_014f3e(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// The sprite record of the object at `obj` as the slot
    /// (`SLOT_INDEX`, `SLOT_ADDR`).
    fn s_slot_of(&mut self, obj: u32) {
        let index = self.read8(obj + OBJ_SPRITE);
        let addr = self.read32(obj + OBJ_SPRITE_ADDR);
        self.write8(SLOT_INDEX, index);
        self.write32(SLOT_ADDR, addr);
    }

    /// `func_0147B4`: a task of `func_0147DC` (`func_001222`) for
    /// object 7 with frame `SHOW_1_FRAME`.  Gives back d5, d6 and a5.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0147b4(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32) {
        let (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, SHOW_DEPTH, SHOW_PRIORITY, ROUTINE_147DC);
        self.write8(a5 + SPRITE_FLAGS, SHOW_1_FLAGS);
        self.write8(a6 + TASK_SHOWN, SHOW_1_FRAME);
        self.write8(a6 + TASK_OBJECT, OBJECT_7);
        (d5, d6, a5)
    }

    /// `func_0147DC`, a task's routine: its sprite at its object's
    /// position (`func_001542`, `func_00152A`, `func_001B7E`), its
    /// frame `TASK_SHOWN` on odd frames, else 0; when the object's
    /// offset is 0 the task ends (`func_00121A`).
    pub fn r_0147dc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a5 + SPRITE_FRAME, 0);
        if self.read8(FRAME) & 1 != 0 {
            let shown = self.read8(r.a6 + TASK_SHOWN);
            self.write8(r.a5 + SPRITE_FRAME, shown);
        }
        r.d4 = low_byte(r.d4, self.read8(r.a6 + TASK_OBJECT));
        (r.d4, r.a4, r.x) = self.r_001cfa(r.d4);
        self.r_001542(r.a4);
        let offset = self.read8(r.a4 + OBJ_OFFSET);
        r.d0 = low_byte(r.d0, offset);
        if offset == 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        r.d6 = low_byte(r.d6, self.read8(CURRENT_TASK));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        self.r_00152a(r.a6);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_01481A`, a task's routine: every 4 frames `SHAKE_Y` goes
    /// up and so does `COUNT_9EB9`; at 0x20 the task ends
    /// (`func_000560`).
    pub fn r_01481a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = low_byte(r.d0, self.read8(FRAME) & 3);
        r.c = false;
        if r.d0 as u8 != 0 {
            return;
        }
        let v = self.read8(SHAKE_Y).wrapping_add(1);
        self.write8(SHAKE_Y, v);
        let was = self.read8(COUNT_9EB9);
        let count = was.wrapping_add(1);
        r.d0 = low_byte(r.d0, count);
        r.x = was == 0xFF;
        self.write8(COUNT_9EB9, count);
        if count == COUNT_9EB9_END {
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
            return;
        }
        r.c = count < COUNT_9EB9_END;
    }

    /// `func_014842`: a task of `func_00B8C0` (`func_0003E8`) with the
    /// words 0x4100, 0x3F, the list `B8C0_LIST`, bytes 0x15 1, 0x17 3,
    /// 0x14, 0x16 and 0x18 0.  Gives back d6.
    pub fn r_014842(&mut self, d4: u32, d5: u32, d6: u32) -> u32 {
        let (d6, a6, _, _) = self.r_0003e8(d4, d5, d6, B8C0_PRIORITY, ROUTINE_B8C0);
        self.write16(a6 + TASK_10, B8C0_10);
        self.write16(a6 + TASK_12, B8C0_12);
        self.write32(a6 + TASK_0C, B8C0_LIST);
        self.write8(a6 + TASK_15, 1);
        self.write8(a6 + TASK_17, 3);
        self.write8(a6 + TASK_16, 0);
        self.write8(a6 + TASK_14, 0);
        self.write8(a6 + TASK_18, 0);
        d6
    }

    /// `func_014886`: the chain set up.  `REACH`, `RING_AT`, `SHRINK`
    /// and the ring cleared; each link 8-0xD set up (`func_014912`),
    /// the first with frame `FIRST_FRAME`, `FIRST_HITS` hits and routine
    /// `func_014B80` (and the Z80's `START_SOUND`), the others
    /// following (`func_0093B8`), each with a task of `func_0147DC`
    /// showing frame `SHOW_FRAME` at it.  Gives back d0, d1, d4, d5,
    /// d6, a0, a6 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_014886(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, bool) {
        self.write16(REACH, 0);
        self.write8(RING_AT, 0);
        self.write8(SHRINK, 0);
        let (d0, d1, a0, _) = self.r_0011d2(RING_SIZE, d1, RING);
        self.write8(LINKS, (LAST - FIRST) as u8);
        let (mut d4, a4, x) = self.r_001cfa(low_byte(d4, FIRST));
        let (mut d0, mut d5, mut a5) = self.r_014912(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        self.write8(a4 + OBJ_FRAME, FIRST_FRAME);
        self.write8(a4 + OBJ_HITS, FIRST_HITS);
        self.write32(a4 + OBJ_ROUTINE, ROUTINE_14B80);
        d0 = low_byte(d0, START_SOUND);
        self.r_0034b6(d0);
        d4 = low_byte(d4, (d4 as u8).wrapping_add(1));
        let mut d6 = d6;
        let mut a6;
        loop {
            let (d4_, a4, x) = self.r_001cfa(d4);
            d4 = d4_;
            (d0, d5, a5) = self.r_014912(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
            self.write32(a4 + OBJ_ROUTINE, FOLLOW);
            (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, SHOW_DEPTH, SHOW_PRIORITY, ROUTINE_147DC);
            self.write8(a5 + SPRITE_FLAGS, SHOW_FLAGS);
            self.write8(a6 + TASK_SHOWN, SHOW_FRAME);
            self.write8(a6 + TASK_OBJECT, d4 as u8);
            d4 = low_byte(d4, (d4 as u8).wrapping_add(1));
            if self.s_tick(LINKS) {
                break;
            }
        }
        (d0, d1, d4, d5, d6, a0, a6, false)
    }

    /// `func_014912`: the object at a4 set up (`func_0012A0` with
    /// `LINK_INIT`), its sprite's flags `LINK_FLAGS`, its angle the
    /// middle, its frame `LINK_FRAME`.  Gives back d0, d5 and a5.
    #[allow(clippy::too_many_arguments)]
    pub fn r_014912(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, x: bool) -> (u32, u32, u32) {
        self.write32(OBJ_INIT_2, LINK_INIT);
        let (d0, d5, a5, _) = self.r_0012a0(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, x);
        self.write8(a5 + SPRITE_FLAGS, LINK_FLAGS);
        self.write8(a4 + OBJ_ANGLE, MIDDLE);
        self.write8(a4 + OBJ_FRAME, LINK_FRAME);
        (d0, d5, a5)
    }

    /// `func_014932`: the chain turned by `d0`'s low byte, unless Z is
    /// set (the callers' flags: of the turn's byte, but `func_014B80`'s
    /// come from its btst).  The running link's angle goes by it (kept
    /// to 0x6B-0x95), what it really went is put in the ring at
    /// `RING_AT`, and each link after it goes by the ring's entry at
    /// (`RING_AT` + 4 * `LINKS`) & 7, `LINKS` counting 7 down; `RING_AT`
    /// goes on.  The
    /// running object and sprite record are put back.  Gives back d1,
    /// d2, d4, d5, a0 (`RING`), a4 and a5.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_014932(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d5: u32, a0: u32, a4: u32, a5: u32, z: bool) -> (u32, u32, u32, u32, u32, u32, u32) {
        if z {
            return (d1, d2, d4, d5, a0, a4, a5);
        }
        let angle = self.read8(a4 + OBJ_ANGLE);
        let wanted = (d0 as u8).wrapping_add(angle);
        let new = if wanted >= HIGHEST {
            HIGHEST
        } else if wanted < LOWEST {
            LOWEST
        } else {
            wanted
        };
        let ring_at = self.read8(RING_AT);
        let d2 = low_word(d2, ring_at as i8 as u16);
        self.write8(RING.wrapping_add(ring_at as i8 as u32), new.wrapping_sub(angle));
        self.write8(a4 + OBJ_ANGLE, new);
        self.write8(LINKS, 7);
        let mut d4 = low_byte(d4, (d4 as u8).wrapping_add(1));
        let mut d1 = low_byte(d1, new);
        loop {
            let a4;
            (d4, a4, _) = self.r_001cfa(d4);
            let links = self.read8(LINKS);
            let at = (links << 2).wrapping_add(self.read8(RING_AT)) & 7;
            d1 = low_word(d1, u16::from(at));
            let turned = self.read8(a4 + OBJ_ANGLE).wrapping_add(self.read8(RING + u32::from(at)));
            self.write8(a4 + OBJ_ANGLE, turned);
            self.write8(LINKS, links.wrapping_sub(1));
            d4 = low_byte(d4, (d4 as u8).wrapping_add(1));
            if d4 as u8 == PAST {
                break;
            }
        }
        let next = self.read8(RING_AT).wrapping_add(1) & 7;
        self.write8(RING_AT, next);
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        (d1, d2, d4, d5, RING, a4, a5)
    }

    /// `func_0149CA`: the links placed, 0xD down to 8: each one's frame
    /// by its angle (its base plus 1, 0 or 2), its position the `REACH`
    /// at its angle from the next link's (the first's: from object 7's,
    /// 0x18 up; `func_014B14`), its offset `BYTE_A61C`, its sprite
    /// placed (`func_001B32`) with byte 3 of `BYTE_3_FROM`'s; then the
    /// links' depths (`func_0007D2`), their y's low bytes swayed by 0,
    /// 4, 8... (`SWAY`), their sprites' frames 0 while `REACH`'s high
    /// byte is 0 (but the first's).  The running object is left the
    /// first link and the sprite record `LEADER`.  Gives back d0, d1,
    /// d2, d4, d5, a4, a5, X and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0149ca(&mut self, d0: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (mut d0, mut d2, mut d4, mut d5) = (d0, d2, d4, d5);
        // a5 is the sprite record's before it is read
        let _ = a5;
        let mut a5;
        let mut d1;
        let mut a4;
        let mut x;
        d4 = low_byte(d4, LAST);
        loop {
            (d4, a4, _) = self.r_001cfa(d4);
            self.write8(RUNNING, d4 as u8);
            self.write32(RUNNING_ADDR, a4);
            self.s_slot_of(a4);
            let angle = self.read8(a4 + OBJ_ANGLE);
            d0 = low_byte(d0, angle);
            d1 = if angle < FRAME_1_UNDER {
                1
            } else if angle < FRAME_0_UNDER {
                0
            } else {
                2
            };
            d1 = low_byte(d1, (d1 as u8).wrapping_add(self.read8(a4 + OBJ_FRAME)));
            d5 = low_byte(d5, self.read8(SLOT_INDEX));
            a5 = self.read32(SLOT_ADDR);
            self.write8(a5 + SPRITE_FRAME, d1 as u8);
            if d4 as u8 == LAST {
                d4 = low_byte(d4, OBJECT_7);
                (d4, a4, _) = self.r_001cfa(d4);
                self.r_001542(a4);
                let y = self.read16(POSITION_Y).wrapping_sub(FIRST_UP);
                self.write16(POSITION_Y, y);
                d4 = low_byte(d4, self.read8(RUNNING));
                a4 = self.read32(RUNNING_ADDR);
            } else {
                d4 = low_byte(d4, (d4 as u8).wrapping_add(1));
                (d4, a4, _) = self.r_001cfa(d4);
                self.r_001542(a4);
                d4 = low_byte(d4, (d4 as u8).wrapping_sub(1));
                (d4, a4, _) = self.r_001cfa(d4);
            }
            d1 = low_byte(d1, self.read8(REACH));
            d0 = low_byte(d0, self.read8(a4 + OBJ_ANGLE));
            (d1, d2) = self.r_014b14(d0, d1, d2, a4);
            let v = self.read8(BYTE_A61C);
            self.write8(a4 + OBJ_OFFSET, v);
            d5 = low_byte(d5, self.read8(SLOT_INDEX));
            a5 = self.read32(SLOT_ADDR);
            (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
            let from = self.read32(BYTE_3_FROM);
            let byte = self.read8(from + SPRITE_BYTE_3);
            d0 = low_byte(d0, byte);
            d5 = low_byte(d5, self.read8(SLOT_INDEX));
            a5 = self.read32(SLOT_ADDR);
            self.write8(a5 + SPRITE_BYTE_3, byte);
            d4 = low_byte(d4, (d4 as u8).wrapping_sub(1));
            if d4 as u8 == OBJECT_7 {
                break;
            }
        }
        d4 = low_byte(d4, FIRST);
        (d4, a4, x) = self.r_001cfa(d4);
        self.write8(SWAY, 0);
        loop {
            self.s_slot_of(a4);
            d0 = LINK_DEPTH;
            self.r_0007d2(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, x);
            let sway = self.read8(SWAY);
            let y = self.read8(a4 + OBJ_Y_LOW).wrapping_add(sway);
            self.write8(a4 + OBJ_Y_LOW, y);
            d0 = low_byte(d0, SWAY_STEP);
            self.write8(SWAY, sway.wrapping_add(SWAY_STEP));
            d4 = low_byte(d4, (d4 as u8).wrapping_add(1));
            if d4 as u8 == PAST {
                break;
            }
            (d4, a4, x) = self.r_001cfa(d4);
            if self.read8(REACH) == 0 {
                d5 = low_byte(d5, self.read8(a4 + OBJ_SPRITE));
                a5 = self.read32(a4 + OBJ_SPRITE_ADDR);
                self.write8(a5 + SPRITE_FRAME, 0);
            }
        }
        d4 = low_byte(d4, FIRST);
        (d4, a4, x) = self.r_001cfa(d4);
        self.write8(RUNNING, d4 as u8);
        self.write32(RUNNING_ADDR, a4);
        d5 = low_byte(d5, self.read8(LEADER));
        a5 = self.read32(LEADER_ADDR);
        self.write8(SLOT_INDEX, d5 as u8);
        self.write32(SLOT_ADDR, a5);
        (d0, d1, d2, d4, d5, a4, a5, x, false)
    }

    /// `func_014B14`: the object at a4 put the `REACH` (as the speed)
    /// from `POSITION` at the angle -`d0` (`func_00410C`).  Gives back
    /// d1 and d2.
    pub fn r_014b14(&mut self, d0: u32, d1: u32, d2: u32, a4: u32) -> (u32, u32) {
        let reach = self.read16(REACH);
        self.write16(SPEED, reach);
        let angle = d0 as u8;
        let (_, d1, d2, _) = self.r_00410c(low_byte(d0, angle.wrapping_neg()), d1, d2, angle != 0);
        let x = (self.read8(VELOCITY_X) as i8 as u16).wrapping_add(self.read16(POSITION_X));
        self.write16(a4 + OBJ_X, x);
        let y = (self.read8(VELOCITY_Y) as i8 as u16).wrapping_add(self.read16(POSITION_Y));
        self.write16(a4 + OBJ_Y, y);
        (d1, d2)
    }

    /// `func_014B46`: `REACH` shrinks by `REACH_SHRINK` (to 0) while
    /// `SHRINK` lasts (it goes down), else it grows by `REACH_GROW` (to
    /// `REACH_MOST`).  Gives back d0 (its low word the new `REACH`).
    pub fn r_014b46(&mut self, d0: u32) -> u32 {
        let shrink = self.read8(SHRINK);
        let reach = self.read16(REACH);
        let new = if shrink != 0 {
            self.write8(SHRINK, shrink - 1);
            let less = reach.wrapping_sub(REACH_SHRINK);
            if (less as i16) < 0 {
                0
            } else {
                less
            }
        } else {
            reach.wrapping_add(REACH_GROW).min(REACH_MOST)
        };
        self.write16(REACH, new);
        low_word(d0, new)
    }

    /// `func_014B80`, the first link's routine at the start: `REACH`
    /// grows (`func_014B46`; at its most, `func_014BC4` from now on, the
    /// timer 0x10 and the turn 1); on frames with `FRAME`'s bit 5 the
    /// chain turns by -1 (`func_014932`, which looks at the btst's Z:
    /// with the bit clear it doesn't turn by 1), the links placed
    /// (`func_0149CA`), the first's frame `TURN_FRAME`.
    pub fn r_014b80(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.s_reach(r);
        if r.d0 as u16 == REACH_MOST {
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14BC4);
            self.write8(r.a4 + OBJ_TIMER, 0x10);
            self.write8(r.a4 + OBJ_TURN, 1);
        }
        r.d0 = 1;
        let bit_clear = self.read8(FRAME) & 0x20 == 0;
        if !bit_clear {
            r.d0 = low_byte(r.d0, 0xFF);
        }
        self.s_turn(r, bit_clear);
        self.s_links(r);
        self.write8(r.a5 + SPRITE_FRAME, TURN_FRAME);
        r.c = false;
    }

    /// `func_014BC4`, the first link's routine: it swings.  `REACH`
    /// (`func_014B46`); at either end of the angles it waits
    /// `END_WAIT` frames, then the turn is the other way; the chain
    /// turns (`func_014932`), the links are placed (`func_0149CA`), the
    /// first's frame `SWING_FRAME` (`SHRINK_FRAME` while shrinking).
    /// When the walker has stopped once (`STOPPED` 1) the Z80 is told
    /// `START_SOUND` and it throws pieces (`func_014C5E`, after
    /// `THROW_WAIT` frames, frame `THROW_FRAME`); more and it shrinks
    /// (`func_014E0E`, not looking at hits this frame).  Hits
    /// (`func_014F46`).
    pub fn r_014bc4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.s_reach(r);
        let timer = self.read8(r.a4 + OBJ_TIMER);
        r.d0 = low_byte(r.d0, timer);
        if timer == 0 {
            let angle = self.read8(r.a4 + OBJ_ANGLE);
            r.d0 = low_byte(r.d0, angle);
            if angle == HIGHEST || angle == LOWEST {
                self.write8(r.a4 + OBJ_TIMER, END_WAIT);
            }
        } else if self.s_tick(r.a4 + OBJ_TIMER) {
            let turn = self.read8(r.a4 + OBJ_TURN).wrapping_neg();
            self.write8(r.a4 + OBJ_TURN, turn);
        }
        self.s_turn_by_own(r);
        self.s_links(r);
        self.write8(r.a5 + SPRITE_FRAME, SWING_FRAME);
        let shrink = self.read8(SHRINK);
        r.d0 = low_byte(r.d0, shrink);
        if shrink != 0 {
            self.write8(r.a5 + SPRITE_FRAME, SHRINK_FRAME);
        }
        let stopped = self.read8(STOPPED);
        r.d0 = low_byte(r.d0, stopped);
        if stopped > 1 {
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14E0E);
            r.c = false;
            return;
        }
        if stopped == 1 {
            r.d0 = low_byte(r.d0, START_SOUND);
            self.r_0034b6(r.d0);
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14C5E);
            self.write8(r.a4 + OBJ_TIMER, THROW_WAIT);
            self.write8(r.a4 + OBJ_FRAME, THROW_FRAME);
        }
        self.s_hits(r);
    }

    /// `func_014C5E`, the first link's routine: it swings on
    /// (`func_014B46`, `func_014932`, `func_0149CA`) for its timer;
    /// then the turn is the other way, a task of `func_014DD4` made
    /// (`func_001222`, its offset `BYTE_A65C`) and it throws
    /// `PIECES`/2 pieces (`func_014CCC` from now on, run now).  Hits
    /// (`func_014F46`).
    pub fn r_014c5e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.s_reach(r);
        self.s_turn_by_own(r);
        self.s_links(r);
        if !self.s_tick(r.a4 + OBJ_TIMER) {
            self.s_hits(r);
            return;
        }
        let turn = self.read8(r.a4 + OBJ_TURN).wrapping_neg();
        self.write8(r.a4 + OBJ_TURN, turn);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14CCC);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, FOLLOWER_DEPTH, FOLLOWER_PRIORITY, ROUTINE_14DD4);
        self.write8(r.a5 + SPRITE_FLAGS, FOLLOWER_FLAGS);
        let v = self.read8(BYTE_A65C);
        self.write8(r.a6 + TASK_OFFSET, v);
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        self.write8(r.a4 + OBJ_TIMER, END_WAIT);
        self.write8(r.a4 + OBJ_COUNT, PIECES);
        r.c = false;
        self.r_014ccc(r);
    }

    /// `func_014CCC`, the first link's routine: `REACH`
    /// (`func_014B46`); every `PIECE_WAIT` frames the chain turns
    /// (`func_014932`) and, every other time, throws a piece
    /// (`func_014D18`); when its count is out it swings again
    /// (`func_014BC4`) and `STOPPED` is cleared.  The links placed
    /// (`func_0149CA`), hits (`func_014F46`).
    pub fn r_014ccc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.s_reach(r);
        if self.s_tick(r.a4 + OBJ_TIMER) {
            self.write8(r.a4 + OBJ_TIMER, PIECE_WAIT);
            self.s_turn_by_own(r);
            if self.read8(r.a4 + OBJ_COUNT) & 1 == 0 {
                (r.d0, r.d2, r.d5, r.d6, r.a4, r.a5, r.a6) = self.r_014d18(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            }
            if self.s_tick(r.a4 + OBJ_COUNT) {
                self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14BC4);
                self.write8(STOPPED, 0);
            }
        }
        self.s_links(r);
        self.s_hits(r);
    }

    /// `func_014D18`: a piece (a task of `func_014D88`, `func_001222`)
    /// at the link's position, its velocity at `PIECE_SPEED` by the
    /// angle -`OBJ_ANGLE` (`func_00410C`), its animation `PIECE_ANIM`,
    /// if one could be had.  The running object and sprite record put
    /// back.  Gives back d0, d2, d5, d6, a4, a5 and a6.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_014d18(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        self.r_001542(a4);
        self.write16(SPEED, PIECE_SPEED);
        let angle = self.read8(a4 + OBJ_ANGLE).wrapping_neg();
        let mut d0 = low_byte(d0, angle);
        let mut d2 = d2;
        let (d5, d6, a5, a6, x, c) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, PIECE_DEPTH, PIECE_PRIORITY, ROUTINE_14D88);
        if !c {
            (d0, _, d2, _) = self.r_00410c(d0, d1, d2, x);
            self.r_00152a(a6);
            self.s_keep_velocity(a6);
            self.write8(a5 + SPRITE_FLAGS, PIECE_FLAGS);
            self.write32(a6 + TASK_OFFSET, PIECE_ANIM);
        }
        let a4 = self.read32(RUNNING_ADDR);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        (d0, d2, d5, d6, a4, a5, a6)
    }

    /// The velocity kept in the task at `a6`.
    fn s_keep_velocity(&mut self, a6: u32) {
        let v = self.read8(VELOCITY_X_FRACTION);
        self.write8(a6 + TASK_VELOCITY_X_FRACTION, v);
        let v = self.read8(VELOCITY_X);
        self.write8(a6 + TASK_VELOCITY_X, v);
        let v = self.read8(VELOCITY_Y_FRACTION);
        self.write8(a6 + TASK_VELOCITY_Y_FRACTION, v);
        let v = self.read8(VELOCITY_Y);
        self.write8(a6 + TASK_VELOCITY_Y, v);
    }

    /// `func_014D88`, a piece's routine: its animation run
    /// (`func_0013D6`; when it ends the task ends, `func_00121A`); it
    /// moves by its velocity (`func_001A88`, `func_001AA2`), the Z80
    /// told of its cell (`func_003554`), the put task there stopped if
    /// the cell is `CELL_PUT` (`func_005E0E`), its sprite placed
    /// (`func_001B7E`).
    pub fn r_014d88(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        let v = self.read8(r.a6 + TASK_VELOCITY_X_FRACTION);
        self.write8(VELOCITY_X_FRACTION, v);
        let v = self.read8(r.a6 + TASK_VELOCITY_X);
        self.write8(VELOCITY_X, v);
        let v = self.read8(r.a6 + TASK_VELOCITY_Y_FRACTION);
        self.write8(VELOCITY_Y_FRACTION, v);
        let v = self.read8(r.a6 + TASK_VELOCITY_Y);
        self.write8(VELOCITY_Y, v);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        r.d0 = CELL_HIT;
        (r.d1, r.d2, r.x, r.c) = self.r_003554(r.d0, r.d1, r.d2, r.a6);
        r.d0 = self.r_0015f8(r.a6);
        (r.a0, r.x) = self.r_00167a();
        let cell = self.read8(r.a0);
        r.d0 = low_byte(r.d0, cell);
        if cell == CELL_PUT {
            let _ = self.r_005e0e(r.d2, r.d6, r.a6);
        }
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_014DD4`, `func_014C5E`'s task's routine: while `STOPPED`
    /// isn't 0 its sprite record gets the walker's (`LEADER`) byte 3,
    /// frame + `FOLLOWER_FRAME` and position; then it ends
    /// (`func_00121A`).
    pub fn r_014dd4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let stopped = self.read8(STOPPED);
        r.d0 = low_byte(r.d0, stopped);
        if stopped == 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        r.d5 = low_byte(r.d5, self.read8(LEADER));
        r.a5 = self.read32(LEADER_ADDR);
        r.d6 = low_byte(r.d6, self.read8(SLOT_INDEX));
        r.a6 = self.read32(SLOT_ADDR);
        let v = self.read8(r.a5 + SPRITE_BYTE_3);
        self.write8(r.a6 + SPRITE_BYTE_3, v);
        let (frame, carry) = self.read8(r.a5 + SPRITE_FRAME).overflowing_add(FOLLOWER_FRAME);
        r.d0 = low_byte(r.d0, frame);
        r.x = carry;
        self.write8(r.a6 + SPRITE_FRAME, frame);
        let v = self.read16(r.a5 + SPRITE_X);
        self.write16(r.a6 + SPRITE_X, v);
        let v = self.read16(r.a5 + SPRITE_Y);
        self.write16(r.a6 + SPRITE_Y, v);
        r.c = false;
    }

    /// `func_014E0E`, the first link's routine: the chain shrinks
    /// (`SHRINK` 1 each frame, `func_014B46`) until `STOPPED` is 0
    /// (then it swings, `func_014BC4`); the links placed
    /// (`func_0149CA`), the first's frame `SWING_FRAME`; hits
    /// (`func_014F46`).
    pub fn r_014e0e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let stopped = self.read8(STOPPED);
        r.d0 = low_byte(r.d0, stopped);
        if stopped == 0 {
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14BC4);
        }
        self.write8(SHRINK, 1);
        self.s_reach(r);
        self.s_links(r);
        self.write8(r.a5 + SPRITE_FRAME, SWING_FRAME);
        r.c = false;
        self.s_hits(r);
    }

    /// `func_014E3A`, the first link's routine once object 7 started
    /// (`func_014F46`): `REACH` grows (`func_014B46`); the links are
    /// placed (`func_0149CA`, the first's frame `FALL_FRAME`) until
    /// `GONE` is negative; then each link from the running one falls:
    /// its velocity at `FALL_SPEED` by angles 0x2A apart
    /// (`func_014EA4`), its timer `FALL_WAIT`, its rate from
    /// `FALL_RATES`.
    pub fn r_014e3a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(SHRINK, 0);
        self.s_reach(r);
        let gone = self.read8(GONE);
        r.d0 = low_byte(r.d0, gone);
        if (gone as i8) >= 0 {
            self.write8(r.a4 + OBJ_FRAME, FALL_FRAME);
            r.c = false;
            self.s_links(r);
            return;
        }
        self.write8(FALL_ANGLE, 0);
        self.write16(SPEED, FALL_SPEED);
        loop {
            let (angle, carry) = self.read8(FALL_ANGLE).overflowing_add(FALL_ANGLE_STEP);
            r.d0 = low_byte(r.d0, angle);
            self.write8(FALL_ANGLE, angle);
            (r.d0, r.d1, r.d2) = self.r_014ea4(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, carry);
            self.write8(r.a4 + OBJ_TIMER, FALL_WAIT);
            let at = (r.d4 as u8).wrapping_mul(2);
            r.d0 = low_word(r.d0, at as i8 as u16);
            let table = FALL_RATES.wrapping_add(at as i8 as u32);
            let v = self.read8(table);
            self.write8(r.a4 + OBJ_RATE_LOW, v);
            let v = self.read8(table + 1);
            self.write8(r.a4 + OBJ_RATE, v);
            r.d4 = low_byte(r.d4, (r.d4 as u8).wrapping_add(1));
            (r.d4, r.a4, r.x) = self.r_001cfa(r.d4);
            if r.d4 as u8 == PAST {
                break;
            }
        }
        r.c = false;
    }

    /// `func_014EA4`: the object's velocity at the angle `d0`
    /// (`func_00410C`) kept in it, its offset 0, its routine
    /// `func_014ED2`, and a task with its index (`func_004FD0`).  Gives
    /// back d0, d1 and d2.
    #[allow(clippy::too_many_arguments)]
    pub fn r_014ea4(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, x: bool) -> (u32, u32, u32) {
        let (d0, d1, d2, _) = self.r_00410c(d0, d1, d2, x);
        let v = self.read8(VELOCITY_X_FRACTION);
        self.write8(a4 + OBJ_VELOCITY_X_FRACTION, v);
        let v = self.read8(VELOCITY_X);
        self.write8(a4 + OBJ_VELOCITY_X, v);
        let v = self.read8(VELOCITY_Y_FRACTION);
        self.write8(a4 + OBJ_VELOCITY_Y_FRACTION, v);
        let v = self.read8(VELOCITY_Y);
        self.write8(a4 + OBJ_VELOCITY_Y, v);
        self.write8(a4 + OBJ_OFFSET, 0);
        self.write32(a4 + OBJ_ROUTINE, ROUTINE_14ED2);
        let (d0, _, _) = self.r_004fd0(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5);
        (d0, d1, d2)
    }

    /// `func_014ED2`, a falling link's routine: it moves by its velocity
    /// (`func_0019C0`, `func_0019DA`), its rate up by `FALL_RATE_STEP`
    /// (`func_004424`) and its offset by it (`func_001ABC`); when that
    /// isn't negative it bounces (offset 0, rate `BOUNCE_RATE`) and
    /// after its timer's bounces it stops: rate 0, flags `GONE_FLAGS`,
    /// `func_014F2C` from now on (run now).  Its sprite is placed
    /// (`func_014F3E`).
    pub fn r_014ed2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let v = self.read8(r.a4 + OBJ_VELOCITY_X_FRACTION);
        self.write8(VELOCITY_X_FRACTION, v);
        let v = self.read8(r.a4 + OBJ_VELOCITY_X);
        self.write8(VELOCITY_X, v);
        let v = self.read8(r.a4 + OBJ_VELOCITY_Y_FRACTION);
        self.write8(VELOCITY_Y_FRACTION, v);
        let v = self.read8(r.a4 + OBJ_VELOCITY_Y);
        self.write8(VELOCITY_Y, v);
        (r.d0, r.d1, r.x) = self.r_0019c0(r.d0, r.d1, r.a4);
        (r.d0, r.d1, r.x) = self.r_0019da(r.d0, r.d1, r.a4);
        r.d0 = FALL_RATE_STEP;
        r.c = false;
        let n;
        (r.d0, r.x, _) = self.r_004424(r.d0, r.a4);
        (r.d0, r.x, n) = self.r_001abc(r.d0, r.a4);
        if n {
            self.s_place_fallen(r);
            return;
        }
        self.write8(r.a4 + OBJ_OFFSET, 0);
        self.write16(r.a4 + OBJ_RATE, BOUNCE_RATE);
        if !self.s_tick(r.a4 + OBJ_TIMER) {
            self.s_place_fallen(r);
            return;
        }
        self.write16(r.a4 + OBJ_RATE, 0);
        self.write8(r.a4 + OBJ_TIMER, GONE_WAIT);
        self.write8(r.a5 + SPRITE_FLAGS, GONE_FLAGS);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14F2C);
        r.x = false;
        r.c = false;
        self.r_014f2c(r);
    }

    /// `func_014F2C`, a fallen link's routine: its sprite placed
    /// (`func_014F3E`) for its timer's frames; then its sprite record
    /// is freed (`func_000890`) and the object ended (its first byte
    /// 0).
    pub fn r_014f2c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.s_tick(r.a4 + OBJ_TIMER) {
            self.s_place_fallen(r);
            return;
        }
        r.x = false;
        self.r_000890();
        self.write8(r.a4 + OBJ_ID, 0);
        r.c = false;
    }

    /// `func_014F3E` (`.L014F3E`): the object's sprite placed
    /// (`func_001B32`) and its depth (`func_001B50`).  Gives back d0,
    /// d1, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_014f3e(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        let (d0, x, c) = self.r_001b50(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        (d0, d1, x, c)
    }

    /// `func_014F46`: the Z80 told of the link's cell (`func_0034DE`,
    /// `func_0015E6`), and whether object 7 is hit (`func_006E7C`):
    /// then its hits are 1, `SHRINK` `HIT_SHRINK`, and one of the
    /// running link's hits goes; at the last `GONE` is cleared, bits
    /// set in `BITS_9540` and `SPARED`, the Z80 told `LOST_SOUND`, the
    /// link goes on as `func_014E3A` (turn `LOST_TURN`), object 7 as
    /// `func_014380` (`s_pieces.rs`) where it is, its timer
    /// `SEVEN_TIMER` and count `SEVEN_COUNT`, object 2 is put there
    /// (`func_001300`) and a task of `func_014FE6` made
    /// (`func_0003E8`).  The running object put back.  Gives back d0,
    /// d1, d2, d4, d6, a0, a4, a6, X and C (clear).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_014f46(&mut self, d1: u32, d2: u32, d4: u32, d5: u32, d6: u32, a0: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (mut d6, mut a6) = (d6, a6);
        let (d1, d2, _) = self.r_0034de(HIT_CELL, d1, d2, a4);
        let _ = self.r_0015e6(a4);
        let (mut d4, mut a4, _) = self.r_001cfa(low_byte(d4, OBJECT_7));
        let (mut d0, d1, d2, a0, mut x, c) = self.r_006e7c(d1, d2, a0, a4, a5);
        if c {
            self.write8(a4 + OBJ_HITS, 1);
            self.write8(SHRINK, HIT_SHRINK);
            d4 = low_byte(d4, self.read8(RUNNING));
            a4 = self.read32(RUNNING_ADDR);
            let hits = self.read8(a4 + OBJ_HITS);
            self.write8(a4 + OBJ_HITS, hits.wrapping_sub(1));
            x = hits == 0;
            if hits == 1 {
                self.write8(GONE, 0);
                let v = self.read8(BITS_9540) | 0x40;
                self.write8(BITS_9540, v);
                let v = self.read8(SPARED) | 0x01;
                self.write8(SPARED, v);
                d0 = low_byte(d0, LOST_SOUND);
                self.r_00349e(d0);
                self.write32(a4 + OBJ_ROUTINE, ROUTINE_14E3A);
                self.write8(a4 + OBJ_TURN, LOST_TURN);
                d4 = low_byte(d4, (d4 as u8).wrapping_sub(1));
                (d4, a4, _) = self.r_001cfa(d4);
                self.write32(a4 + OBJ_ROUTINE, ROUTINE_14380);
                self.r_001542(a4);
                self.write8(a4 + OBJ_TIMER, SEVEN_TIMER);
                self.write8(a4 + OBJ_HITS, SEVEN_COUNT);
                d4 = low_byte(d4, OBJECT_2);
                (d4, a4, _) = self.r_001cfa(d4);
                self.r_001300(a4);
                (d6, a6, x, _) = self.r_0003e8(d4, d5, d6, LOST_PRIORITY, ROUTINE_14FE6);
            }
        }
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        (d0, d1, d2, d4, d6, a0, a4, a6, x, false)
    }

    /// `func_014FE6`, a task's routine: `WORD_9EE8` 0xD8, then
    /// `.L013756`.
    pub fn r_014fe6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write16(WORD_9EE8, 0xD8);
        (r.d0, r.d4, r.a0, r.a4, r.x, r.c) = self.r_013756(r.d4, r.d6, r.a0, r.a4);
    }
}
