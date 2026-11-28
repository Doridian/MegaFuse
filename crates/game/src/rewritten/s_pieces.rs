//! An object's routines (0x14380-0x147B4): `func_014380` and the ones it
//! goes on as.  While its count `OBJ_COUNT` lasts it makes, every 16
//! frames, a task that shakes `SHAKE_X`/`SHAKE_Y` about
//! `CENTRE_X`/`CENTRE_Y` (`func_0146CE`, `func_0146F2`), moves
//! `CENTRE_X` toward 0x88, and every 8 frames throws a pair of pieces
//! (`func_014528`: a task that runs an animation where it is,
//! `func_014622`, and one that falls from it, `func_014638`).  Then it
//! rises (`func_0143FC`), waits (`func_014460`-`func_0144D6`), sends a
//! task up (`func_01472A`, `func_014782`), and when that has gone ends
//! with more pieces (`func_014422`, `func_014436`) and `.L012B6A`.

use crate::{Game, Regs};

/// The object's fields: its routine and the routine after
/// `func_014436`'s pieces (longs), a count (`func_014380` throws pieces
/// and shakes while it isn't 0), its animation (a long), its timer and
/// a count of pieces (bytes), its rate (a word) and a byte 0x2A.
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_THEN: u32 = 0x06;
const OBJ_COUNT: u32 = 0x11;
const OBJ_ANIM: u32 = 0x20;
const OBJ_TIMER: u32 = 0x24;
const OBJ_PIECES: u32 = 0x25;
const OBJ_RATE: u32 = 0x28;
const OBJ_2A: u32 = 0x2A;
/// The object's first byte: cleared when it ends.
const OBJ_ID: u32 = 0x00;
/// Tasks' fields: a count (`func_014622`'s, down each frame), a base
/// frame (and 1 on frames with `FRAME`'s bit 2), a count (frames), an
/// index into `FALL_X`, the offset (a byte; `func_01472A`'s, the
/// pieces') and the rate (a word).
const TASK_11: u32 = 0x11;
const TASK_FRAME: u32 = 0x13;
const TASK_COUNT: u32 = 0x16;
const TASK_WAY: u32 = 0x17;
const TASK_RATE: u32 = 0x1A;
const TASK_OFFSET: u32 = 0x20;
/// A sprite record's frame and flags.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// The frame count.
const FRAME: u32 = 0xFF80D8;
/// The new task's index (`task.rs`).
const NEW_TASK: u32 = 0xFF80EF;
/// The object being run (its index and address) and its sprite
/// record's (index, address); put back after making tasks.
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// The velocity (bytes: x, its fraction, y, its fraction; or words)
/// `func_001A88` and `func_001AA2` move by.
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;
/// A byte: the offset of `func_014528`'s first task.
const BYTE_A61C: u32 = 0xFFA61C;
/// The pieces' count `& 7` (`func_014528`) and `func_0146F2`'s range
/// (both use the byte).
const INDEX_AABD: u32 = 0xFFAABD;
/// Bytes `func_0146F2` sets to `CENTRE_X`, `CENTRE_Y` plus a random
/// number up to its range, and the bytes they are about;
/// `func_014380` moves `CENTRE_X` toward `CENTRE_X_TO`.
const SHAKE_X: u32 = 0xFFA60B;
const SHAKE_Y: u32 = 0xFFA60D;
const CENTRE_X: u32 = 0xFFA4CB;
const CENTRE_Y: u32 = 0xFFA4CD;
const CENTRE_X_TO: u8 = 0x88;
/// Cleared by `func_014380` (`s_walker.rs`'s count of stops).
const STOPPED: u32 = 0xFFAD56;
/// Counts the tasks of `func_014782` that went (`func_0144EE` waits
/// for it), then 0xFF.
const GONE: u32 = 0xFFAD57;
/// A byte set to 2, 0 and 1 on the way (`func_01446E` on), cleared by
/// `func_01451A`.
const BYTE_954F: u32 = 0xFF954F;

/// The object's sprite frames: while throwing, rising, waiting, at the
/// end.
const THROW_FRAME: u8 = 0x89;
const RISE_FRAME: u8 = 0x4F;
const WAIT_FRAME: u8 = 0x50;
/// `func_0143FC`'s rate step (`func_004424`), and the frames of its
/// task (`func_0043A8`).
const RISE_STEP: u32 = 0x10;
const RISE_TASK: u32 = 0x20;
/// The waits: `func_014460`'s, `func_01446E`'s, `func_01448A`'s.
const WAIT_1: u8 = 0xF0;
const WAIT_2: u8 = 0x80;
const WAIT_3: u8 = 0xF0;
/// `func_014422`'s pieces and the frames between them; `func_01448A`'s.
const END_PIECES: u8 = 5;
const PIECE_WAIT: u8 = 4;
const WAIT_PIECES: u8 = 6;
/// The Z80's commands: a piece of `func_014436`, `func_01448A`'s, a
/// thrown pair (`func_014528`).
const PIECE_SOUND: u8 = 0x5F;
const WAIT_SOUND: u8 = 0x44;
const THROW_SOUND: u8 = 0x5E;
/// The frame of the task `func_01448A` finds (the new task, a6).
const WAIT_TASK_FRAME: u8 = 0x51;

/// `func_014528`'s tasks: their depths, priority, flags; the falling
/// one's rate, its frames (0x51 + 2 * 0-4) and count; a piece every 8
/// frames.
const ROUTINE_14622: u32 = 0x014622;
const ROUTINE_14638: u32 = 0x014638;
const STILL_DEPTH: u8 = 2;
const FALL_DEPTH: u8 = 1;
const PIECE_PRIORITY: u8 = 0xC2;
const PIECE_FLAGS: u8 = 0x81;
const STILL_ANIM: u32 = 0x0E8604;
const FALL_RATE: u16 = 0xFC00;
const FALL_FRAMES: u8 = 0x51;
const FALL_FRAME_KINDS: u32 = 4;
const FALL_BOUNCES: u8 = 2;
const PIECE_EVERY: u8 = 8;
/// The pieces' velocities by the count & 7 (x, then y 8 on: D_01460A)
/// and the falling ones' x by `TASK_WAY` (D_01461A).
const PIECE_VELOCITIES: u32 = 0x01460A;
const FALL_X: u32 = 0x01461A;
/// How `func_014638` falls: its velocity's y, its gravity
/// (`func_00C742`), the rate a bounce gives, the flags and count once
/// done (then `func_0146C4`).
const FALL_VELOCITY_Y: u16 = 0x100;
const GRAVITY: u32 = 0x20;
const BOUNCE_RATE: u16 = 0xFF00;
const DONE_FLAGS: u8 = 0xC1;
const DONE_COUNT: u8 = 0x10;
const ROUTINE_146C4: u32 = 0x0146C4;
/// `func_0146CE`'s task: routine, priority, frames.
const ROUTINE_146F2: u32 = 0x0146F2;
const SHAKE_PRIORITY: u8 = 0x43;
const SHAKE_FRAMES: u8 = 0x10;
/// `func_01472A`'s task: velocity y, routine, depth, priority, flags,
/// frame, offset, rate.
const UP_VELOCITY: u16 = 0xE000;
const ROUTINE_14782: u32 = 0x014782;
const UP_DEPTH: u8 = 1;
const UP_PRIORITY: u8 = 0x42;
const UP_FLAGS: u8 = 0x84;
const UP_FRAME: u8 = 0xEA;
const UP_OFFSET: u8 = 0xF0;
const UP_RATE: u16 = 0xFB00;
/// What `.L012B6A` sets: the routine, the animation, the timer.
const ROUTINE_12B8E: u32 = 0x012B8E;
const END_ANIM: u32 = 0x0E874A;
const END_TIMER: u8 = 0xC0;
/// The routines the object goes on as.
const ROUTINE_143FC: u32 = 0x0143FC;
const ROUTINE_14436: u32 = 0x014436;
const ROUTINE_14460: u32 = 0x014460;
const ROUTINE_1446E: u32 = 0x01446E;
const ROUTINE_1448A: u32 = 0x01448A;
const ROUTINE_144D6: u32 = 0x0144D6;
const ROUTINE_144EE: u32 = 0x0144EE;
const ROUTINE_1451A: u32 = 0x01451A;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_014528` on the family's registers.
    fn s_throw(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.c) = self.r_014528(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `func_001B32` (`.L0143E2`, `.L01445C`) on the family's
    /// registers: the object's sprite placed.
    fn s_place(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
    }

    /// `func_001B94` (`.L0146BE`) on the family's registers: the task's
    /// sprite placed.
    fn s_place_task(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b94(r.a5, r.a6);
    }

    /// The byte at `at` one less; true when it got to 0.
    fn s_count_down(&mut self, at: u32) -> bool {
        let left = self.read8(at).wrapping_sub(1);
        self.write8(at, left);
        left == 0
    }

    /// `func_014380`, an object's routine: `STOPPED` cleared, its frame
    /// `THROW_FRAME`.  While its count isn't 0, every 16 frames a
    /// shaking task (`func_0146CE`); every 4 frames `CENTRE_X` moves a
    /// step toward 0x88 (when it is there and the count is 0, the
    /// object rises: its rate 0, `func_0143FC` from now on).  A pair of
    /// pieces every 8 frames (`func_014528`) takes 1 off the count.
    /// Its sprite is placed (`func_001B32`).
    pub fn r_014380(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(STOPPED, 0);
        self.write8(r.a5 + SPRITE_FRAME, THROW_FRAME);
        let count = self.read8(r.a4 + OBJ_COUNT);
        r.d0 = low_byte(r.d0, count);
        if count != 0 {
            r.d0 = low_byte(r.d0, self.read8(FRAME) & 0x0F);
            if r.d0 as u8 == 0 {
                (r.d4, r.d6, r.a4, r.a6, r.x) = self.r_0146ce(r.d4, r.d5, r.d6);
                r.d0 = low_byte(r.d0, self.read8(FRAME));
            }
        }
        r.d0 = low_byte(r.d0, r.d0 as u8 & 3);
        if r.d0 as u8 == 0 {
            let centre = self.read8(CENTRE_X);
            r.d0 = low_byte(r.d0, centre);
            if centre == CENTRE_X_TO {
                let count = self.read8(r.a4 + OBJ_COUNT);
                r.d0 = low_byte(r.d0, count);
                if count == 0 {
                    self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_143FC);
                    self.write16(r.a4 + OBJ_RATE, 0);
                    r.c = false;
                    self.r_0143fc(r);
                    return;
                }
            } else if centre > CENTRE_X_TO {
                self.write8(CENTRE_X, centre - 1);
                r.x = false;
            } else {
                self.write8(CENTRE_X, centre + 1);
                r.x = false;
            }
        }
        self.s_throw(r);
        if r.c {
            let count = self.read8(r.a4 + OBJ_COUNT);
            r.d0 = low_byte(r.d0, count);
            if count != 0 {
                self.write8(r.a4 + OBJ_COUNT, count - 1);
                r.x = false;
            }
        }
        self.s_place(r);
    }

    /// `func_0143E2` (`.L0143E2`): the object's sprite placed
    /// (`func_001B32`).  Gives back d0, d1, X and C.
    pub fn r_0143e2(&mut self, d1: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        self.r_001b32(d1, a4, a5)
    }

    /// `func_0143FC`, an object's routine: it rises (its rate up by
    /// `RISE_STEP` a frame, `func_004424`, its offset by that,
    /// `func_001ABC`; its sprite placed, `func_001B32`) until the
    /// offset is no more negative; then `OBJ_2A` is cleared, a task of
    /// `RISE_TASK` frames made (`func_0043A8`), its frame `RISE_FRAME`
    /// and it waits (`func_014460` from now on).
    pub fn r_0143fc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = RISE_STEP;
        let n;
        (r.d0, r.x, _) = self.r_004424(r.d0, r.a4);
        (r.d0, r.x, n) = self.r_001abc(r.d0, r.a4);
        if n {
            self.s_place(r);
            return;
        }
        self.write8(r.a4 + OBJ_2A, 0);
        r.d0 = RISE_TASK;
        (r.d0, r.x) = self.r_0043a8(r.d0, r.d4, r.d5, r.d6);
        self.write8(r.a5 + SPRITE_FRAME, RISE_FRAME);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_14460);
        r.c = false;
    }

    /// `func_014422`: the object throws `END_PIECES` pairs of pieces,
    /// `PIECE_WAIT` frames apart (`func_014436` from now on, run now),
    /// then goes on as its `OBJ_THEN`.  Gives back d0, d1, d5, d6, a0,
    /// a5 and a6.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_014422(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32) {
        self.write32(a4 + OBJ_ROUTINE, ROUTINE_14436);
        self.write8(a4 + OBJ_PIECES, END_PIECES);
        self.write8(a4 + OBJ_TIMER, PIECE_WAIT);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, n: false, z: false, v: false, c: false, unwound: false };
        self.r_014436(&mut r);
        (r.d0, r.d1, r.d5, r.d6, r.a0, r.a5, r.a6)
    }

    /// `func_014436`, an object's routine: a pair of pieces when the
    /// timer runs out (`func_014528`), with the Z80's `PIECE_SOUND`,
    /// the next `PIECE_WAIT` frames on; after the last the object goes
    /// on as its `OBJ_THEN`.  Its sprite is placed (`func_001B32`).
    pub fn r_014436(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.s_throw(r);
        if r.c {
            r.d0 = low_byte(r.d0, PIECE_SOUND);
            self.r_0034b6(r.d0);
            self.write8(r.a4 + OBJ_TIMER, PIECE_WAIT);
            let pieces = self.read8(r.a4 + OBJ_PIECES);
            r.d0 = low_byte(r.d0, pieces);
            if pieces == 0 {
                let then = self.read32(r.a4 + OBJ_THEN);
                self.write32(r.a4 + OBJ_ROUTINE, then);
            }
        }
        self.s_place(r);
    }

    /// `func_01445C` (`.L01445C`): the object's sprite placed
    /// (`func_001B32`).  Gives back d0, d1, X and C.
    pub fn r_01445c(&mut self, d1: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        self.r_001b32(d1, a4, a5)
    }

    /// `func_014460`, an object's routine: it waits `WAIT_1` frames
    /// (`func_01446E` from now on, run now).
    pub fn r_014460(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(r.a4 + OBJ_TIMER, WAIT_1);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_1446E);
        r.c = false;
        self.r_01446e(r);
    }

    /// `func_01446E`, an object's routine: when its timer runs out,
    /// `BYTE_954F` is 2 and it waits `WAIT_2` frames more
    /// (`func_01448A`).  Its sprite is placed (`func_001B32`).
    pub fn r_01446e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.s_count_down(r.a4 + OBJ_TIMER) {
            self.write8(BYTE_954F, 2);
            self.write8(r.a4 + OBJ_TIMER, WAIT_2);
            self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_1448A);
        }
        self.s_place(r);
    }

    /// `func_01448A`, an object's routine: `BYTE_954F` cleared; when
    /// its timer runs out its frame is `WAIT_FRAME`, it throws a pair
    /// of pieces (`func_014528`, `WAIT_PIECES` of them counted), the
    /// new task gets frame `WAIT_TASK_FRAME`, the Z80 `WAIT_SOUND`, and
    /// it waits `WAIT_3` frames (`func_0144D6` from now on, run now).
    /// Else its sprite is placed (`func_001B32`).
    pub fn r_01448a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(BYTE_954F, 0);
        if !self.s_count_down(r.a4 + OBJ_TIMER) {
            self.s_place(r);
            return;
        }
        self.write8(r.a5 + SPRITE_FRAME, WAIT_FRAME);
        self.write8(r.a4 + OBJ_PIECES, WAIT_PIECES);
        let v = self.read8(r.a4 + OBJ_TIMER).wrapping_add(1);
        self.write8(r.a4 + OBJ_TIMER, v);
        self.s_throw(r);
        r.d6 = low_byte(r.d6, self.read8(NEW_TASK));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        self.write8(r.a6 + TASK_FRAME, WAIT_TASK_FRAME);
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        r.d0 = low_byte(r.d0, WAIT_SOUND);
        self.r_0034b6(r.d0);
        self.write8(r.a4 + OBJ_TIMER, WAIT_3);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_144D6);
        r.c = false;
        self.r_0144d6(r);
    }

    /// `func_0144D6`, an object's routine: when its timer runs out,
    /// `GONE` is cleared, a task sent up (`func_01472A`), and it waits
    /// for that to go (`func_0144EE` from now on, run now).  Else its
    /// sprite is placed (`func_001B32`).
    pub fn r_0144d6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if !self.s_count_down(r.a4 + OBJ_TIMER) {
            self.s_place(r);
            return;
        }
        self.write8(GONE, 0);
        (r.d0, r.d1, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6, r.x) = self.r_01472a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_144EE);
        r.c = false;
        self.r_0144ee(r);
    }

    /// `func_0144EE`, an object's routine: its sprite placed
    /// (`func_001B32`) until `GONE` isn't 0; then `GONE` is 0xFF,
    /// `BYTE_954F` 1, its frame 0, it throws its last pieces
    /// (`func_014422`, then `func_01451A`), and `.L012B6A`:
    /// `func_01288E`, its routine `func_012B8E`, frame 0, animation
    /// `END_ANIM`, timer `END_TIMER`.
    pub fn r_0144ee(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let gone = self.read8(GONE);
        r.d0 = low_byte(r.d0, gone);
        if gone == 0 {
            self.s_place(r);
            return;
        }
        self.write32(r.a4 + OBJ_THEN, ROUTINE_1451A);
        self.write8(BYTE_954F, 1);
        self.write8(GONE, 0xFF);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        (r.d0, r.d1, r.d5, r.d6, r.a0, r.a5, r.a6) = self.r_014422(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        (r.d0, r.d4, r.d5, r.a4, r.a5, r.x) = self.r_01288e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
        self.write32(r.a4 + OBJ_ROUTINE, ROUTINE_12B8E);
        self.write8(r.a5 + SPRITE_FRAME, 0);
        self.write32(r.a4 + OBJ_ANIM, END_ANIM);
        self.write8(r.a4 + OBJ_TIMER, END_TIMER);
        r.c = false;
    }

    /// `func_01451A`, an object's routine (`func_0144EE`'s
    /// `OBJ_THEN`): `BYTE_954F` cleared, its sprite record freed
    /// (`func_000890`) and the object ended (its first byte 0).
    pub fn r_01451a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(BYTE_954F, 0);
        self.r_000890();
        self.write8(r.a4 + OBJ_ID, 0);
        r.c = false;
    }

    /// `func_014528`: the object's timer down; when it runs out (C set
    /// then; else clear) its count of pieces goes down and a pair is
    /// thrown from its position (`func_001542`), the next in 8 frames:
    /// a task of `func_014622` moved once by the velocity from
    /// `PIECE_VELOCITIES` by the count & 7 (`func_001A88`,
    /// `func_001AA2`; placed with the offset `BYTE_A61C`, then running
    /// `STILL_ANIM`), and one of `func_014638` at the new position
    /// with that index, rate `FALL_RATE`, frame 0x51-0x59 at random and
    /// the Z80's `THROW_SOUND` (if either task can't be had, the rest
    /// isn't done).  The running object and sprite record are put
    /// back.  Gives back d0, d1, d4, d5, d6, a0, a4, a5, a6 and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_014528(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        if !self.s_count_down(a4 + OBJ_TIMER) {
            return (d0, d1, d4, d5, d6, a0, a4, a5, a6, false);
        }
        let (mut d0, mut d1, mut d5, mut d6, mut a5) = (d0, d1, d5, d6, a5);
        let mut a6;
        let pieces = self.read8(a4 + OBJ_PIECES).wrapping_sub(1);
        self.write8(a4 + OBJ_PIECES, pieces);
        let index = pieces & 7;
        self.write8(INDEX_AABD, index);
        d0 = low_word(d0, u16::from(index));
        let v = self.read8(PIECE_VELOCITIES + u32::from(index));
        self.write8(VELOCITY_X, v);
        let v = self.read8(PIECE_VELOCITIES + 8 + u32::from(index));
        self.write8(VELOCITY_Y, v);
        self.write8(a4 + OBJ_TIMER, PIECE_EVERY);
        self.r_001542(a4);
        'made: {
            let c;
            (d5, d6, a5, a6, _, c) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, STILL_DEPTH, PIECE_PRIORITY, ROUTINE_14622);
            if c {
                break 'made;
            }
            self.r_00152a(a6);
            (d0, d1, _, _, _) = self.r_001a88(d0, d1, a6);
            (_, d1, _, _, _) = self.r_001aa2(d0, d1, a6);
            self.r_00153a(a6);
            self.write8(a5 + SPRITE_FLAGS, PIECE_FLAGS);
            let v = self.read8(BYTE_A61C);
            self.write8(a6 + TASK_OFFSET, v);
            (d0, _, _) = self.r_001b94(a5, a6);
            self.write32(a6 + TASK_OFFSET, STILL_ANIM);
            let c;
            (d5, d6, a5, a6, _, c) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, FALL_DEPTH, PIECE_PRIORITY, ROUTINE_14638);
            if c {
                break 'made;
            }
            self.r_00152a(a6);
            let v = self.read8(INDEX_AABD);
            self.write8(a6 + TASK_WAY, v);
            self.write8(a6 + TASK_OFFSET, 0);
            self.write16(a6 + TASK_RATE, FALL_RATE);
            d0 = self.r_001bc0(FALL_FRAME_KINDS, d1);
            d0 = low_byte(d0, (d0 as u8).wrapping_mul(2).wrapping_add(FALL_FRAMES));
            self.write8(a6 + TASK_FRAME, d0 as u8);
            self.write8(a6 + TASK_COUNT, FALL_BOUNCES);
            self.write8(a5 + SPRITE_FLAGS, PIECE_FLAGS);
            d0 = low_byte(d0, THROW_SOUND);
            self.r_0034b6(d0);
        }
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        d5 = low_byte(d5, self.read8(SLOT_INDEX));
        a5 = self.read32(SLOT_ADDR);
        (d0, d1, d4, d5, d6, PIECE_VELOCITIES, a4, a5, a6, true)
    }

    /// `func_014622`, a piece's routine (`func_014528`'s first): its
    /// byte 0x11 down, its sprite placed (`func_001B94`), its animation
    /// run (`func_0013D6`); when that ends the task ends
    /// (`func_00121A`).
    pub fn r_014622(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let v = self.read8(r.a6 + TASK_11).wrapping_sub(1);
        self.write8(r.a6 + TASK_11, v);
        self.s_place_task(r);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
        if r.c {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
        }
    }

    /// `func_014638`, a piece's routine (`func_014528`'s second): every
    /// 4 frames its frame (its base, one more on frames with `FRAME`'s
    /// bit 2); it moves by x `FALL_X[TASK_WAY]` and y
    /// `FALL_VELOCITY_Y` (`func_001A88`, `func_001AA2`), its rate
    /// pulled down (`func_00C742`) and its offset by it
    /// (`func_001ACA`); when that isn't negative it bounces (rate
    /// `BOUNCE_RATE`, offset 0) and after its count of bounces it is
    /// done: flags `DONE_FLAGS`, `func_0146C4` from now on.  Its sprite
    /// is placed (`func_001B94`).
    pub fn r_014638(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = low_byte(r.d0, self.read8(FRAME) & 3);
        if r.d0 as u8 == 0 {
            let odd = (self.read8(FRAME) & 4) >> 2;
            r.d0 = low_byte(r.d0, odd.wrapping_add(self.read8(r.a6 + TASK_FRAME)));
            self.write8(r.a5 + SPRITE_FRAME, r.d0 as u8);
        }
        let way = self.read8(r.a6 + TASK_WAY);
        r.d1 = low_word(r.d1, way as i8 as u16);
        r.a0 = FALL_X;
        let fall_x = self.read8(FALL_X.wrapping_add(way as i8 as u32)) as i8 as u16;
        r.d0 = low_word(r.d0, fall_x);
        self.write16(VELOCITY_X, fall_x);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        self.write16(VELOCITY_Y, FALL_VELOCITY_Y);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
        r.d0 = low_byte(r.d0, GRAVITY as u8);
        self.r_00c742(r.d0, r.a6);
        let n;
        (r.d0, r.x, n) = self.r_001aca(r.d0, r.a6);
        if !n {
            self.write16(r.a6 + TASK_RATE, BOUNCE_RATE);
            self.write8(r.a6 + TASK_OFFSET, 0);
            if self.s_count_down(r.a6 + TASK_COUNT) {
                self.write8(r.a6 + TASK_COUNT, DONE_COUNT);
                (r.x, r.c) = self.r_00053e(r.d6, ROUTINE_146C4);
                self.write8(r.a5 + SPRITE_FLAGS, DONE_FLAGS);
            }
        }
        self.s_place_task(r);
    }

    /// `func_0146BE` (`.L0146BE`): the task's sprite placed
    /// (`func_001B94`).  Gives back d0, X and C.
    pub fn r_0146be(&mut self, a5: u32, a6: u32) -> (u32, bool, bool) {
        self.r_001b94(a5, a6)
    }

    /// `func_0146C4`, a piece's routine once done: its sprite placed
    /// (`func_001B94`) for `DONE_COUNT` frames, then the task ends
    /// (`func_00121A`).
    pub fn r_0146c4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.s_count_down(r.a6 + TASK_COUNT) {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, false);
        } else {
            self.s_place_task(r);
        }
    }

    /// `func_0146CE`: a task of `func_0146F2` (`func_0003E8`, no
    /// sprite) for `SHAKE_FRAMES` frames, if one could be had; the
    /// running object put back.  Gives back d4, d6, a4, a6 and X.
    pub fn r_0146ce(&mut self, d4: u32, d5: u32, d6: u32) -> (u32, u32, u32, u32, bool) {
        let (d6, a6, x, c) = self.r_0003e8(d4, d5, d6, SHAKE_PRIORITY, ROUTINE_146F2);
        if !c {
            self.write8(a6 + TASK_COUNT, SHAKE_FRAMES);
        }
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        (d4, d6, a4, a6, x)
    }

    /// `func_0146F2`, a task's routine: for its count of frames
    /// `SHAKE_X` and `SHAKE_Y` are `CENTRE_X` and `CENTRE_Y` plus a
    /// random number from 0 to (count / 4) | 1 (`func_001BC0`); then
    /// the task ends (`func_000560`).
    pub fn r_0146f2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.s_count_down(r.a6 + TASK_COUNT) {
            (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, false);
            return;
        }
        let range = (self.read8(r.a6 + TASK_COUNT) >> 2) | 1;
        r.d0 = low_byte(r.d0, range);
        self.write8(INDEX_AABD, range);
        r.d0 = self.r_001bc0(r.d0, r.d1);
        r.d0 = low_byte(r.d0, (r.d0 as u8).wrapping_add(self.read8(CENTRE_X)));
        self.write8(SHAKE_X, r.d0 as u8);
        r.d0 = low_byte(r.d0, self.read8(INDEX_AABD));
        r.d0 = self.r_001bc0(r.d0, r.d1);
        let (y, carry) = (r.d0 as u8).overflowing_add(self.read8(CENTRE_Y));
        r.d0 = low_byte(r.d0, y);
        r.x = carry;
        self.write8(SHAKE_Y, y);
        r.c = false;
    }

    /// `func_01472A`: a task of `func_014782` made (`func_001222`; it
    /// isn't looked at whether it could be) at the object's position
    /// moved once by velocity y `UP_VELOCITY` (`func_001A88`,
    /// `func_001AA2`), with flags `UP_FLAGS`, frame `UP_FRAME`, offset
    /// `UP_OFFSET` and rate `UP_RATE`, its sprite placed
    /// (`func_001B7E`).  The running object and sprite record are put
    /// back.  Gives back d0, d1, d4, d5, d6, a4, a5, a6 and X.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01472a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        self.write16(VELOCITY_Y, UP_VELOCITY);
        self.r_001542(a4);
        let (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, UP_DEPTH, UP_PRIORITY, ROUTINE_14782);
        self.r_00152a(a6);
        let (d0, d1, _, _, _) = self.r_001a88(d0, d1, a6);
        let (_, d1, _, _, _) = self.r_001aa2(d0, d1, a6);
        self.write8(a5 + SPRITE_FLAGS, UP_FLAGS);
        self.write8(a5 + SPRITE_FRAME, UP_FRAME);
        self.write8(a6 + TASK_OFFSET, UP_OFFSET);
        self.write16(a6 + TASK_RATE, UP_RATE);
        let (d0, x, _) = self.r_001b7e(a5, a6);
        let d4 = low_byte(d4, self.read8(RUNNING));
        let a4 = self.read32(RUNNING_ADDR);
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        (d0, d1, d4, d5, d6, a4, a5, a6, x)
    }

    /// `func_014782`, the routine of `func_01472A`'s task: its rate
    /// pulled down (`func_001B14`) and its offset by it
    /// (`func_001ACA`); while that is negative its sprite is placed
    /// (`func_001B94`); then `GONE` goes up and the task ends
    /// (`func_00121A`).
    pub fn r_014782(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = self.r_001b14(r.a6);
        let n;
        (r.d0, r.x, n) = self.r_001aca(r.d0, r.a6);
        if n {
            self.s_place_task(r);
            return;
        }
        let was = self.read8(GONE);
        self.write8(GONE, was.wrapping_add(1));
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, was == 0xFF);
    }
}
