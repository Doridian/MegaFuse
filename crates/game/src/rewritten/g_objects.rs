//! Object routines of 0x7000-0x7AAE: the routines an object (a4, its
//! sprite record a5) runs once a frame from its `OBJ_ROUTINE`
//! (`func_001CA0`), and the tasks (a6) some of them make.  Each sets the
//! routine of the next frame by writing `OBJ_ROUTINE` (and most fall
//! into it at once).
//!
//! Most end the same way: the sprite placed for the object
//! (`func_001B32`), its depth (`func_001B5C`, or `func_001B50` with the
//! object's offset), the object's cell (`func_0015E6`), then
//! `func_006E28`.
//!
//! Two kinds of objects work in pairs: some routines reach the fields of
//! the object before (`OBJ_PREV_OFFSET`) or after (`OBJ_NEXT_FLAGS`) the
//! one running, or run it too (`func_00725A`).
//!
//! The routines keep every register the original leaves, which their
//! callers (the families' generated code) may read: each passes the
//! registers on as the 68000 does.

use crate::{Game, Regs};

/// An object's fields.
/// Byte 0: not 0 while it is in use (`a_objects.rs`); compared here
/// with `ID_PAIRED` and `ID_SPLIT` to choose what to run.
const OBJ_ID: u32 = 0x00;
/// The routine it runs (a long).
const OBJ_ROUTINE: u32 = 0x02;
/// A long `func_006E28` keeps the routine in when it changes it (so its
/// first byte is 0 then); the routines here also use that byte's high
/// nibble as a mark that they are under way.
const OBJ_SAVED: u32 = 0x06;
/// Counted down (`func_007774`, `func_0077D2`); its low 3 bits pick
/// the frame `func_007834` makes a task in.
const OBJ_COUNTDOWN: u32 = 0x07;
/// Bit 7 keeps `func_00192A` from finding the object; bit 0 is set
/// from `OBJ_NEXT_DIRECTION`'s (`func_00762C`).
const OBJ_FLAGS: u32 = 0x10;
/// When not 0, `func_001486` is asked first (`func_007020`); also read
/// by `func_006E28`.
const OBJ_11: u32 = 0x11;
/// Set to 1 when its routine becomes `func_006EE0`.
const OBJ_12: u32 = 0x12;
/// The direction it takes next (negative: none).
const OBJ_NEXT_DIRECTION: u32 = 0x13;
/// Its direction (bits 0-1; bit 7 kept apart) and the kind of its
/// animation (`anim.rs`).
const OBJ_DIRECTION: u32 = 0x14;
const OBJ_KIND: u32 = 0x15;
/// Its animation (a long) and the animation's frames to go and
/// position (`anim.rs`); the routines here use the two bytes as
/// counters of their own too.
const OBJ_ANIM: u32 = 0x20;
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
/// A word (`a_motion.rs`).
const OBJ_RATE: u32 = 0x28;
/// Its offset (the high byte of a word, `a_motion.rs`) and a byte
/// after it, counted up (`func_007744`).
const OBJ_OFFSET: u32 = 0x2A;
const OBJ_PHASE: u32 = 0x2B;
/// The previous object's `OBJ_OFFSET` (0x40 bytes back), and the next
/// object's `OBJ_FLAGS` (0x40 bytes on).
const OBJ_PREV_OFFSET: u32 = 0x2A_u32.wrapping_sub(0x40);
const OBJ_NEXT_FLAGS: u32 = 0x40 + 0x10;

/// The `OBJ_ID`s some routines here look for.
const ID_PAIRED: u8 = 0x0D;
const ID_SPLIT: u8 = 0x14;

/// A sprite record's frame and flags (`sprite.rs`).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;

/// A task's fields (`task.rs`, `anim.rs`): its position's low bytes
/// (its cell within the map's), its animation's counters (cleared),
/// three bytes of the routines here (0x16 counted down to its end) and
/// its animation.
const TASK_X_LOW: u32 = 0x0F;
const TASK_Y_LOW: u32 = 0x11;
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_15: u32 = 0x15;
const TASK_COUNT: u32 = 0x16;
const TASK_17: u32 = 0x17;
const TASK_ANIM: u32 = 0x20;

/// A map cell (column, row) and the one `func_003E44` keeps.
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
const KEPT_X: u32 = 0xFFA966;
const KEPT_Y: u32 = 0xFFA967;
/// The cell's address in the cell map (`func_00167A`).
const CELL_AT: u32 = 0xFF9ED6;
/// The speed (a word) `func_001620` makes a velocity of (`a_motion.rs`).
const SPEED: u32 = 0xFF9F00;
const SPEED_LOW: u32 = 0xFF9F01;
/// The object running (its index and address) and its sprite record's
/// (`a_objects.rs`): reloaded after making a task.
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// Its low 3 bits are compared with a task's index (`func_0075D6`).
const TURN: u32 = 0xFF80D8;
/// Written with 1, or the object's index, before `func_0053A6`.
const FOR_53A6: u32 = 0xFFA93F;
/// Counted up when an object's routine becomes `func_006EE0`.
const ENDED: u32 = 0xFFA956;
/// Set (0x10), counted up and down by `func_007020`.
const SET_AC7D: u32 = 0xFFAC7D;
const UP_AC7C: u32 = 0xFFAC7C;
const DOWN_A959: u32 = 0xFFA959;
/// A word `func_0073E4` sets to 0x133.
const WORD_A960: u32 = 0xFFA960;
/// A word `func_007834` sets to 0xA0.
const WORD_9EEC: u32 = 0xFF9EEC;
/// Where `func_007834` keeps a direction while it makes a task.
const KEPT_DIRECTION: u32 = 0xFFAABD;
/// 10 entries of 8 bytes: a byte (0: unused) and a long, the address of
/// a record whose bytes `ENTRY_X`, `ENTRY_Y` are compared with a cell
/// (`func_007438`).
const ENTRIES: u32 = 0xFFFF_AAC0;
const ENTRY_SIZE: u32 = 8;
const ENTRY_COUNT: u32 = 10;
const ENTRY_ADDR: u32 = 2;
const ENTRY_X: u32 = 0x1A;
const ENTRY_Y: u32 = 0x1B;

/// Tables in the ROM: offsets (`func_00720A`, `func_007602`,
/// `func_007774`) and `func_007744`'s.
const OFFSETS_722A: u32 = 0x00722A;
const OFFSETS_762A: u32 = 0x00762A;
const OFFSETS_7726: u32 = 0x007726;
const OFFSETS_7770: u32 = 0x007770;

/// Animations in the ROM.
const ANIM_E263A: u32 = 0x0E263A;
const ANIM_E26E8: u32 = 0x0E26E8;
const ANIM_E26F2: u32 = 0x0E26F2;
const ANIM_E2752: u32 = 0x0E2752;
const ANIM_E282E: u32 = 0x0E282E;
const ANIM_E3A4A: u32 = 0x0E3A4A;
const ANIM_E3A66: u32 = 0x0E3A66;
const ANIM_E3B62: u32 = 0x0E3B62;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    // The callees on the registers, as the original calls them.

    /// `func_001B32`: the sprite placed for the object.
    fn g_place(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
    }

    /// `func_001B5C`: the sprite's depth.
    fn g_depth(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_001B50`: the sprite's depth, less the object's offset.
    fn g_depth_offset(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b50(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_0015E6`: the object's cell.
    fn g_cell(&mut self, r: &mut Regs) {
        (r.d0, r.x) = self.r_0015e6(r.a4);
    }

    /// `func_00130E`: the object's animation by its direction and kind.
    fn g_set_anim(&mut self, r: &mut Regs) {
        (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
    }

    /// `func_001336`: the object's animation a frame on (C: it ended).
    fn g_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
    }

    /// `func_0013D6`: the task's animation a frame on (C: it ended).
    fn g_task_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `func_001B7E`: the sprite placed for the task.
    fn g_task_place(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_00121A`: the task ends.
    fn g_task_end(&mut self, r: &mut Regs) {
        (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
    }

    /// `func_001A88`, `func_001AA2`: the task moved by the velocity.
    fn g_task_move(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, _, r.c) = self.r_001a88(r.d0, r.d1, r.a6);
        (r.d0, r.d1, r.x, _, r.c) = self.r_001aa2(r.d0, r.d1, r.a6);
    }

    /// `func_003988`.
    fn g_3988(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d4, r.a4) = self.r_003988(r.d0, r.d1, r.d4, r.a4, r.a5);
    }

    /// `func_006E28`.
    fn g_6e28(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e28(r.d0, r.d1, r.d2, r.a0, r.a4, r.a5);
    }

    /// `func_0053A6` (with `FOR_53A6` written first).
    fn g_53a6(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6) =
            self.r_0053a6(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `func_004FD0`.
    fn g_4fd0(&mut self, r: &mut Regs) {
        (r.d0, r.x, r.c) = self.r_004fd0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_001222`: a task running `routine` with a sprite record
    /// (d5, d6, a5, a6 then the sprite's and the task's; C: none).
    fn g_make_task(&mut self, r: &mut Regs, depth: u8, priority: u8, routine: u32) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, depth, priority, routine);
    }

    /// d4, a4, d5 and a5 back to the object running and its sprite
    /// record.
    fn g_reload(&mut self, r: &mut Regs) {
        r.d4 = low_byte(r.d4, self.read8(RUNNING));
        r.a4 = self.read32(RUNNING_ADDR);
        self.g_reload_sprite(r);
    }

    /// d5 and a5 back to the running object's sprite record.
    fn g_reload_sprite(&mut self, r: &mut Regs) {
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
    }

    /// The object's routine from the next frame on.
    fn g_next(&mut self, obj: u32, routine: u32) {
        self.write32(obj + OBJ_ROUTINE, routine);
    }

    /// Counts the byte at `addr` down; whether it got to 0 (X and C,
    /// the borrow, are never set: the counters here don't start at 0).
    fn g_count_down(&mut self, r: &mut Regs, addr: u32) -> bool {
        let n = self.read8(addr);
        self.write8(addr, n.wrapping_sub(1));
        r.x = n == 0;
        r.c = r.x;
        n == 1
    }

    /// `g_count_down` on a field of the object (a4) or of the task (a6).
    fn g_count_down_obj(&mut self, r: &mut Regs, field: u32) -> bool {
        let addr = r.a4 + field;
        self.g_count_down(r, addr)
    }

    fn g_count_down_task(&mut self, r: &mut Regs, field: u32) -> bool {
        let addr = r.a6 + field;
        self.g_count_down(r, addr)
    }

    /// The direction taken: `OBJ_NEXT_DIRECTION`, if there is one, into
    /// `OBJ_DIRECTION` (its bit 7 kept).  d0 and d1 are left as the
    /// original leaves them.
    fn g_turn(&mut self, r: &mut Regs) {
        let obj = r.a4;
        let next = self.read8(obj + OBJ_NEXT_DIRECTION);
        r.d0 = low_byte(r.d0, next);
        if next & 0x80 == 0 {
            let kept = self.read8(obj + OBJ_DIRECTION) & 0x80;
            r.d1 = low_byte(r.d1, kept);
            r.d0 = low_byte(r.d0, next | kept);
            self.write8(obj + OBJ_DIRECTION, next | kept);
        }
    }

    /// The common end: the sprite placed, its depth, the object's cell
    /// and `func_006E28` (`func_00730C`, `func_007428`).
    fn g_finish(&mut self, r: &mut Regs) {
        self.g_place(r);
        self.g_depth(r);
        self.g_cell(r);
        self.g_6e28(r);
    }

    /// Whether the mark in `OBJ_SAVED` is set (d0: its high nibble).
    fn g_marked(&mut self, r: &mut Regs) -> bool {
        let mark = self.read8(r.a4 + OBJ_SAVED) & 0xF0;
        r.d0 = low_byte(r.d0, mark);
        mark != 0
    }

    /// The mark set: `OBJ_SAVED`'s high nibble all ones (d0 with it).
    fn g_mark_all(&mut self, r: &mut Regs) {
        let mark = (r.d0 as u8) | 0xF0;
        r.d0 = low_byte(r.d0, mark);
        self.write8(r.a4 + OBJ_SAVED, mark);
    }

    /// `OBJ_TIMER` and `OBJ_STEP` cleared.
    fn g_clear_counters(&mut self, obj: u32) {
        self.write8(obj + OBJ_TIMER, 0);
        self.write8(obj + OBJ_STEP, 0);
    }

    // The routines.

    /// `func_007000`: the sprite's frame 0xF8 and flags 4, the cell at
    /// the object's (`func_00167A`) made 0 (`func_0019B6`), then
    /// `func_007020` from now on.
    pub fn r_007000(&mut self, regs: &mut Regs) {
        self.write8(regs.a5 + SPRITE_FRAME, 0xF8);
        self.write8(regs.a5 + SPRITE_FLAGS, 4);
        self.g_cell(regs);
        (regs.a0, regs.x) = self.r_00167a();
        regs.a0 = self.r_0019b6();
        self.g_next(regs.a4, 0x007020);
        regs.c = false;
        self.r_007020(regs);
        regs.unwound = false;
    }

    /// `func_007020`: the sprite placed; unless `OBJ_11` is set and
    /// `func_001486` finds nothing, `SET_AC7D` is set, `UP_AC7C` counted
    /// up, `DOWN_A959` down, the cell's byte split (`func_00168C`),
    /// `func_001278`, `func_0010D6`, `func_0052F0`, `ENDED` counted,
    /// `func_006F1A` and `func_006766`.
    pub fn r_007020(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_cell(r);
        self.g_place(r);
        self.g_depth(r);
        let v = self.read8(r.a4 + OBJ_11);
        r.d0 = low_byte(r.d0, v);
        let go = if v == 0 {
            true
        } else {
            (r.x, r.c) = self.r_001486(r.d0, r.d1, r.a0, r.x);
            r.c
        };
        if go {
            self.write8(SET_AC7D, 0x10);
            let n = self.read8(UP_AC7C);
            self.write8(UP_AC7C, n.wrapping_add(1));
            let n = self.read8(DOWN_A959);
            self.write8(DOWN_A959, n.wrapping_sub(1));
            (r.a0, r.x) = self.r_00167a();
            (r.d0, r.a0) = self.r_00168c(r.d0);
            self.r_001278();
            r.x = self.r_0010d6(r.d0);
            (r.x, r.c) = self.r_0052f0(r.d0);
            let n = self.read8(ENDED);
            self.write8(ENDED, n.wrapping_add(1));
            (r.d0, r.d1, r.x, r.c) = self.r_006f1a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006766(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
        r.unwound = false;
    }

    /// `func_00706C`: the sprite's frame 0xF9 and flags 4 (0x1E and 1
    /// when the next object's `OBJ_FLAGS` has bit 0), placed, the
    /// object's `OBJ_FLAGS` 0xFF; when the next object's `OBJ_FLAGS` is
    /// negative, the object is set going (`OBJ_DIRECTION` 0xFF,
    /// `OBJ_RATE` 0xFE00, `OBJ_FLAGS` 0x80, `func_007000` saved,
    /// `func_0070C8` next) and `func_004FD0`.
    pub fn r_00706c(&mut self, regs: &mut Regs) {
        let r = regs;
        let (frame, flags) = if self.read8(r.a4 + OBJ_NEXT_FLAGS) & 1 != 0 { (0x1E, 1) } else { (0xF9, 4) };
        self.write8(r.a5 + SPRITE_FRAME, frame);
        self.write8(r.a5 + SPRITE_FLAGS, flags);
        self.g_place(r);
        self.write8(r.a4 + OBJ_FLAGS, 0xFF);
        let next = self.read8(r.a4 + OBJ_NEXT_FLAGS);
        r.d0 = low_byte(r.d0, next);
        r.c = false;
        if next & 0x80 != 0 {
            self.write8(r.a4 + OBJ_DIRECTION, 0xFF);
            self.write16(r.a4 + OBJ_RATE, 0xFE00);
            self.write8(r.a4 + OBJ_FLAGS, 0x80);
            self.write32(r.a4 + OBJ_SAVED, 0x007000);
            self.g_next(r.a4, 0x0070C8);
            self.g_4fd0(r);
        }
        r.unwound = false;
    }

    /// `func_0070C8`: the sprite's frame 0xF9, `func_0020B0`, then
    /// `func_004E5C`.
    pub fn r_0070c8(&mut self, regs: &mut Regs) {
        self.write8(regs.a5 + SPRITE_FRAME, 0xF9);
        (regs.d0, regs.x, regs.c) = self.r_0020b0(regs.d0, regs.a4, regs.x);
        self.r_004e5c(regs);
        regs.unwound = false;
    }

    /// `func_0070D6`: the animation a frame on; when it ends,
    /// `OBJ_KIND` one more for its next animation, then 3 (counted down
    /// by `func_0070FC`, next).  Then `func_0070F4`.
    pub fn r_0070d6(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_animate(r);
        if r.c {
            let kind = self.read8(r.a4 + OBJ_KIND);
            self.write8(r.a4 + OBJ_KIND, kind.wrapping_add(1));
            self.g_set_anim(r);
            self.g_next(r.a4, 0x0070FC);
            self.write8(r.a4 + OBJ_KIND, 3);
            r.c = false;
        }
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_0070f4(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        r.unwound = false;
    }

    /// `func_0070F4`: the object's cell, then `func_0071A0`.  Gives back
    /// d0, d1, d2, a0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0070f4(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        self.r_0015e6(a4);
        self.r_0071a0(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5)
    }

    /// `func_0070FC`: the animation a frame on; each time it ends
    /// `OBJ_KIND` is counted down, and at 0 `func_007110` runs (and is
    /// next).  Otherwise `func_0070F4`.
    pub fn r_0070fc(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_animate(r);
        if r.c && self.g_count_down_obj(r, OBJ_KIND) {
            self.g_next(r.a4, 0x007110);
            r.c = false;
            self.r_007110(r);
        } else {
            (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_0070f4(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
        r.unwound = false;
    }

    /// `func_007110`: the cell kept (`func_003E44`) and the object's;
    /// when they are in the same row (`func_003E68`) and less than 3
    /// columns apart, the object faces it (`OBJ_DIRECTION` 1, or 3 when
    /// the object's column is not less), `OBJ_KIND` 4, `func_0070D6`
    /// next, and `func_0071A0`.  Otherwise `func_007174`.
    pub fn r_007110(&mut self, regs: &mut Regs) {
        let r = regs;
        r.d0 = self.r_003e44();
        self.g_cell(r);
        (r.d0, r.c) = self.r_003e68(r.d0);
        let mut near = false;
        if r.c {
            let (x, kept) = (self.read8(CELL_X), self.read8(KEPT_X));
            let diff = x.wrapping_sub(kept);
            r.x = x < kept;
            let mut d = diff;
            if diff & 0x80 != 0 {
                d = diff.wrapping_neg();
                r.x = diff != 0;
            }
            r.d0 = low_byte(r.d0, d);
            r.c = d < 3;
            near = r.c;
        }
        if near {
            self.g_next(r.a4, 0x0070D6);
            self.write8(r.a4 + OBJ_DIRECTION, 1);
            self.write8(r.a4 + OBJ_KIND, 4);
            if self.read8(CELL_X) >= self.read8(KEPT_X) {
                self.write8(r.a4 + OBJ_DIRECTION, 3);
            }
            let direction = self.read8(r.a4 + OBJ_DIRECTION);
            r.d0 = low_byte(r.d0, direction);
            self.g_clear_counters(r.a4);
            self.g_set_anim(r);
            self.g_cell(r);
            (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_0071a0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        } else {
            self.r_007174(r);
        }
        r.unwound = false;
    }

    /// `func_007174`: `func_003988`, then `func_007178`.
    pub fn r_007174(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_3988(r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_007178(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a6);
        r.unwound = false;
    }

    /// `.L007178` (in `func_007174`): the direction taken
    /// (`OBJ_NEXT_DIRECTION`), then `func_00718E`.  Gives back every
    /// register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_007178(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a6, ..Regs::default() };
        self.g_turn(&mut r);
        self.r_00718e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a6)
    }

    /// `func_00718E`: the object's animation set and a frame on,
    /// `func_0053A6` (`FOR_53A6` 1), then `func_0071A0`.  Gives back
    /// every register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00718e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a6, ..Regs::default() };
        self.g_set_anim(&mut r);
        self.g_animate(&mut r);
        self.write8(FOR_53A6, 1);
        r.c = false;
        self.g_53a6(&mut r);
        if self.restarted.is_none() {
            (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_0071a0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_0071A0`: the sprite placed, its depth less the offset, the
    /// object's cell, then `func_006E28`.  Gives back d0, d1, d2, a0, X
    /// and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0071a0(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, ..Regs::default() };
        self.g_place(&mut r);
        self.g_depth_offset(&mut r);
        self.g_cell(&mut r);
        self.g_6e28(&mut r);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c)
    }

    /// `func_0071B0`: the sprite's frame 0x47, placed, its depth, the
    /// cell at the object's made 0 (`func_0019B6`), then `func_0071D2`
    /// from now on.
    pub fn r_0071b0(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write8(r.a5 + SPRITE_FRAME, 0x47);
        self.g_place(r);
        self.g_depth(r);
        self.g_cell(r);
        (r.a0, r.x) = self.r_00167a();
        r.a0 = self.r_0019b6();
        self.g_next(r.a4, 0x0071D2);
        r.c = false;
        self.r_0071d2(r);
        r.unwound = false;
    }

    /// `func_0071D2`: the sprite placed, its depth; when `func_001486`
    /// finds something, the cell's byte split (`func_00168C`),
    /// `func_006EE0` next with `OBJ_12` 1 and the animation `ANIM_E263A`,
    /// and `ENDED` counted.
    pub fn r_0071d2(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_place(r);
        self.g_depth(r);
        self.g_cell(r);
        (r.x, r.c) = self.r_001486(r.d0, r.d1, r.a0, r.x);
        if r.c {
            (r.a0, r.x) = self.r_00167a();
            (r.d0, r.a0) = self.r_00168c(r.d0);
            self.g_next(r.a4, 0x006EE0);
            self.write8(r.a4 + OBJ_12, 1);
            self.write32(r.a4 + OBJ_ANIM, ANIM_E263A);
            let n = self.read8(ENDED);
            self.write8(ENDED, n.wrapping_add(1));
            r.x = n == 0xFF;
            r.c = r.x;
        }
        r.unwound = false;
    }

    /// The previous object's offset from `table` (by `OBJ_STEP` less 1,
    /// at least 0, shifted right `shift`) plus the object's own
    /// (`func_00720A`, `func_007602`); d0 as the original leaves it.
    fn g_prev_offset(&mut self, r: &mut Regs, table: u32, shift: u32) {
        let step = self.read8(r.a4 + OBJ_STEP).wrapping_sub(1);
        let index = (if step & 0x80 != 0 { 0 } else { step }) >> shift;
        // ext.w: the index is below 0x80
        r.d0 = low_word(r.d0, u16::from(index));
        let offset = self.read8(table + u32::from(index)).wrapping_add(self.read8(r.a4 + OBJ_OFFSET));
        r.d0 = low_byte(r.d0, offset);
        self.write8(r.a4.wrapping_add(OBJ_PREV_OFFSET), offset);
    }

    /// `func_00720A`: the previous object's offset from `OFFSETS_722A`,
    /// then `func_00722E`.
    pub fn r_00720a(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_prev_offset(r, OFFSETS_722A, 1);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_00722e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        r.unwound = false;
    }

    /// `func_00722E`: as `func_007174` (`func_003988`, the direction
    /// taken, the animation, `func_0053A6`), but ends with
    /// `func_00725A`.  Gives back every register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00722e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        self.g_3988(&mut r);
        self.g_turn(&mut r);
        self.g_set_anim(&mut r);
        self.g_animate(&mut r);
        self.write8(FOR_53A6, 1);
        r.c = false;
        self.g_53a6(&mut r);
        if self.restarted.is_none() {
            (r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c) =
                self.r_00725a(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_00725A`: the sprite placed; the previous object (index d4
    /// less 1) put at this one's position and its sprite placed and
    /// given a depth; then this one's sprite its depth, and
    /// `func_006E28`.  Gives back d0, d1, d2, d4, d5, a0, a4, a5 (this
    /// object and its sprite record again), X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00725a(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, ..Regs::default() };
        self.g_place(&mut r);
        self.g_cell(&mut r);
        self.r_001542(r.a4);
        let previous = (r.d4 as u8).wrapping_sub(1);
        self.g_object(&mut r, previous);
        self.r_001300(r.a4);
        self.g_object_sprite(&mut r);
        self.g_place(&mut r);
        self.g_depth_offset(&mut r);
        let this = (r.d4 as u8).wrapping_add(1);
        self.g_object(&mut r, this);
        self.g_object_sprite(&mut r);
        self.g_depth_offset(&mut r);
        self.g_6e28(&mut r);
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c)
    }

    /// d4 and a4 the object of index `index` (`func_001CFA`).
    fn g_object(&mut self, r: &mut Regs, index: u8) {
        (r.d4, r.a4, r.x) = self.r_001cfa(low_byte(r.d4, index));
    }

    /// d5 and a5 the object's sprite record, kept in `SLOT_INDEX`,
    /// `SLOT_ADDR`.
    fn g_object_sprite(&mut self, r: &mut Regs) {
        /// An object's sprite record's address (a long) and index.
        const OBJ_SPRITE_ADDR: u32 = 0x0A;
        const OBJ_SPRITE: u32 = 0x0E;
        let index = self.read8(r.a4 + OBJ_SPRITE);
        r.d5 = low_byte(r.d5, index);
        r.a5 = self.read32(r.a4 + OBJ_SPRITE_ADDR);
        self.write8(SLOT_INDEX, index);
        self.write32(SLOT_ADDR, r.a5);
    }

    /// `func_0072A6`: unless marked (then `func_007308`), marked (bit
    /// 7), the counters cleared, the animation `ANIM_E26E8`, and
    /// `func_0072D0` from now on.
    pub fn r_0072a6(&mut self, regs: &mut Regs) {
        let r = regs;
        let marked = self.g_marked(r);
        r.c = false;
        if marked {
            self.g_7308(r);
        } else {
            let mark = self.read8(r.a4 + OBJ_SAVED);
            self.write8(r.a4 + OBJ_SAVED, mark | 0x80);
            self.g_next(r.a4, 0x0072D0);
            self.g_clear_counters(r.a4);
            self.write32(r.a4 + OBJ_ANIM, ANIM_E26E8);
            r.c = false;
            self.r_0072d0(r);
        }
        r.unwound = false;
    }

    /// `func_0072D0`: the animation a frame on; when it ends,
    /// `func_0072E0` (and from now on).  Otherwise `func_00730C`.
    /// (No script runs it: reviewed by reading.)
    pub fn r_0072d0(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_animate(r);
        if r.c {
            self.g_next(r.a4, 0x0072E0);
            r.c = false;
            self.r_0072e0(r);
        } else {
            self.g_finish(r);
        }
        r.unwound = false;
    }

    /// `func_0072E0`: `OBJ_TIMER` counted down; at 0 the animation
    /// `ANIM_E26F2` and `func_0072F8` (from now on).  Otherwise
    /// `func_00730C`.  (No script runs it: reviewed by reading.)
    pub fn r_0072e0(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.g_count_down_obj(r, OBJ_TIMER) {
            self.write32(r.a4 + OBJ_ANIM, ANIM_E26F2);
            self.g_next(r.a4, 0x0072F8);
            r.c = false;
            self.r_0072f8(r);
        } else {
            self.g_finish(r);
        }
        r.unwound = false;
    }

    /// `func_0072F8`: the animation a frame on; when it ends,
    /// `func_0072A6` next and `func_007308`.  Otherwise `func_00730C`.
    /// (No script runs it: reviewed by reading.)
    pub fn r_0072f8(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_animate(r);
        if r.c {
            self.g_next(r.a4, 0x0072A6);
            r.c = false;
            self.g_7308(r);
        } else {
            self.g_finish(r);
        }
        r.unwound = false;
    }

    /// `func_007308` on the registers.
    fn g_7308(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_007308(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
    }

    /// `.L007308` (in `func_0072F8`): `func_007174`.  Gives back every
    /// register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_007308(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c, ..Regs::default() };
        self.r_007174(&mut r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `.L00730C` (in `func_0072F8`): the sprite placed, its depth, the
    /// object's cell and `func_006E28`.  Gives back d0, d1, d2, a0, X
    /// and C.  (No script runs it on its own: reviewed by reading.)
    #[allow(clippy::too_many_arguments)]
    pub fn r_00730c(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, ..Regs::default() };
        self.g_finish(&mut r);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c)
    }

    /// `func_00731C`: `func_007174`; when that gives C, a task running
    /// `func_007368` (depth 0x10, priority 0x42) at the object's
    /// position, its animation `ANIM_E2752`, 0x40 frames to wait, its
    /// sprite's frame 0 and flags 0x41.
    pub fn r_00731c(&mut self, regs: &mut Regs) {
        let r = regs;
        self.r_007174(r);
        if self.restarted.is_none() && r.c {
            self.r_001542(r.a4);
            self.g_make_task(r, 0x10, 0x42, 0x007368);
            if !r.c {
                self.r_00152a(r.a6);
                self.write32(r.a6 + TASK_ANIM, ANIM_E2752);
                self.write8(r.a6 + TASK_TIMER, 0);
                self.write8(r.a6 + TASK_STEP, 0);
                self.write8(r.a6 + TASK_COUNT, 0x40);
                self.write8(r.a5 + SPRITE_FRAME, 0);
                self.write8(r.a5 + SPRITE_FLAGS, 0x41);
                r.c = false;
                self.r_007368(r);
            }
        }
        r.unwound = false;
    }

    /// `func_007368`, a task: waits `TASK_COUNT` frames, then its sprite
    /// is placed and its animation run; the task ends with it.
    pub fn r_007368(&mut self, regs: &mut Regs) {
        let r = regs;
        let wait = self.read8(r.a6 + TASK_COUNT);
        r.d0 = low_byte(r.d0, wait);
        if wait == 0 {
            self.g_task_place(r);
            self.g_task_animate(r);
            if r.c {
                self.g_task_end(r);
            }
        } else {
            self.g_count_down_task(r, TASK_COUNT);
        }
        r.unwound = false;
    }

    /// `.L007386` (in `func_007368`): only a return.
    pub fn r_007386(&mut self) {}

    /// `func_00738C`: when `OBJ_DIRECTION` is negative, it is the turn
    /// of the object's index (`func_00442C`) and an entry is in line
    /// with the object's cell (`func_007438`): the object faces it,
    /// put in the middle of its cell, `OBJ_KIND` 5 and `func_0073CA`
    /// (from now on).  Otherwise `func_007174`.
    pub fn r_00738c(&mut self, regs: &mut Regs) {
        let r = regs;
        let direction = self.read8(r.a4 + OBJ_DIRECTION);
        r.d0 = low_byte(r.d0, direction);
        r.c = false;
        let mut found = false;
        if direction & 0x80 != 0 {
            let (z, c) = self.r_00442c();
            r.c = c;
            if z {
                self.g_cell(r);
                (r.d0, r.x, r.c) = self.r_007438(r.d0, r.x);
                found = r.c;
            }
        }
        if found {
            self.write8(r.a4 + OBJ_NEXT_DIRECTION, r.d0 as u8);
            self.write8(r.a4 + OBJ_DIRECTION, r.d0 as u8);
            self.r_001852(r.a4);
            self.write8(r.a4 + OBJ_KIND, 5);
            self.g_clear_counters(r.a4);
            self.g_set_anim(r);
            self.g_next(r.a4, 0x0073CA);
            r.c = false;
            self.r_0073ca(r);
        } else {
            self.r_007174(r);
        }
        r.unwound = false;
    }

    /// `func_0073CA`: the animation a frame on; when it ends, `OBJ_KIND`
    /// 9 and `func_0073E4` (from now on).  Otherwise `func_007428`.
    /// (No script runs it: reviewed by reading.)
    pub fn r_0073ca(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_animate(r);
        if r.c {
            self.write8(r.a4 + OBJ_KIND, 9);
            self.g_set_anim(r);
            self.g_next(r.a4, 0x0073E4);
            r.c = false;
            self.r_0073e4(r);
        } else {
            self.g_finish(r);
        }
        r.unwound = false;
    }

    /// `func_0073E4`: `WORD_A960` 0x133, the animation a frame on, the
    /// cell next to the object's in `OBJ_NEXT_DIRECTION` (`func_00173E`)
    /// and its byte through `func_003AC4` (and `func_003A04` when that
    /// gives no C); when no entry is in line any more (`func_007438`),
    /// `OBJ_KIND` 0 and `func_00738C` next.  Then `func_007428`.  (No
    /// script runs it: reviewed by reading.)
    pub fn r_0073e4(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_cell(r);
        self.write16(WORD_A960, 0x133);
        self.g_animate(r);
        let direction = self.read8(r.a4 + OBJ_NEXT_DIRECTION);
        r.d0 = low_byte(r.d0, direction);
        r.x = self.r_00173e(r.d0, r.a4);
        r.a0 = self.read32(CELL_AT);
        let cell = self.read8(r.a0);
        r.d0 = low_byte(r.d0, cell);
        if !self.r_003ac4(r.d0, r.a4) {
            (r.d0, r.d1, r.x) = self.r_003a04(r.d0, r.d1, r.a4);
        }
        (r.d0, r.x, r.c) = self.r_007438(r.d0, r.x);
        if !r.c {
            self.g_next(r.a4, 0x00738C);
            self.write8(r.a4 + OBJ_KIND, 0);
            self.g_set_anim(r);
        }
        self.g_finish(r);
        r.unwound = false;
    }

    /// `.L007428` (in `func_0073E4`): the sprite placed, its depth, the
    /// object's cell and `func_006E28`.  Gives back d0, d1, d2, a0, X
    /// and C.  (No script runs it on its own: reviewed by reading.)
    #[allow(clippy::too_many_arguments)]
    pub fn r_007428(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, ..Regs::default() };
        self.g_finish(&mut r);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c)
    }

    /// `func_007438`: the first entry in use of `ENTRIES` whose record's
    /// cell is in the column or the row of `CELL_X`, `CELL_Y`: C set,
    /// d0's low byte the direction it is in (0 up, 1 right, 2 down, 3
    /// left; the column compared first).  The other registers are kept.
    /// Gives back d0, X (clear once past the first entry: `addq`) and C.
    pub fn r_007438(&mut self, d0: u32, x: bool) -> (u32, bool, bool) {
        let (cell_x, cell_y) = (self.read8(CELL_X), self.read8(CELL_Y));
        let mut x = x;
        for i in 0..ENTRY_COUNT {
            let entry = ENTRIES + ENTRY_SIZE * i;
            if self.read8(entry) != 0 {
                let record = self.read32(entry + ENTRY_ADDR);
                if self.read8(record + ENTRY_X) == cell_x {
                    let direction = if cell_y < self.read8(record + ENTRY_Y) { 2 } else { 0 };
                    return (low_byte(d0, direction), x, true);
                }
                if self.read8(record + ENTRY_Y) == cell_y {
                    let direction = if cell_x < self.read8(record + ENTRY_X) { 1 } else { 3 };
                    return (low_byte(d0, direction), x, true);
                }
            }
            x = false;
        }
        (d0, x, false)
    }

    /// `loc_0074B8`: unless marked (then `func_00722E` for an object
    /// `ID_PAIRED`, else `func_007174`), marked (bits 7 and 5),
    /// `OBJ_TIMER` 0x28, `OBJ_STEP` 10 and `func_0074DC` (from now on).
    pub fn r_0074b8(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.g_marked(r) {
            if self.read8(r.a4 + OBJ_ID) == ID_PAIRED {
                (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
                    self.r_00722e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            } else {
                self.r_007174(r);
            }
        } else {
            let mark = self.read8(r.a4 + OBJ_SAVED);
            self.write8(r.a4 + OBJ_SAVED, mark | 0xA0);
            self.g_next(r.a4, 0x0074DC);
            self.write8(r.a4 + OBJ_TIMER, 0x28);
            self.write8(r.a4 + OBJ_STEP, 10);
            r.c = false;
            self.r_0074dc(r);
        }
        r.unwound = false;
    }

    /// `func_0074DC`: every 0x28 frames (then 12) `OBJ_TIMER` gets to 0:
    /// a task running `func_0075A6` (depth 0x80, priority 0xCF) at the
    /// object's position, going in `OBJ_NEXT_DIRECTION` (kept in its
    /// `TASK_17`) at `SPEED` 0x800, its animation `ANIM_E282E`, and
    /// sound 0x5D; after `OBJ_STEP` (10) of them, `loc_0074B8` next (or
    /// `func_007602` for an object `ID_PAIRED`).  Then `func_00725A`
    /// for an object `ID_PAIRED`, else the sprite placed, its depth,
    /// the object's cell and `func_006E28`.  (No script runs it:
    /// reviewed by reading.)
    pub fn r_0074dc(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.g_count_down_obj(r, OBJ_TIMER) {
            self.write8(r.a4 + OBJ_TIMER, 12);
            self.r_001542(r.a4);
            let direction = self.read8(r.a4 + OBJ_NEXT_DIRECTION) & 3;
            r.d0 = low_byte(r.d0, direction);
            let saved = r.d0;
            self.g_make_task(r, 0x80, 0xCF, 0x0075A6);
            r.d0 = saved;
            if !r.c {
                self.write8(r.a6 + TASK_17, r.d0 as u8);
                self.write8(r.a5 + SPRITE_FLAGS, 1);
                self.r_00152a(r.a6);
                self.write32(r.a6 + TASK_ANIM, ANIM_E282E);
                self.write16(SPEED, 0x800);
                let direction = self.read8(r.a6 + TASK_17);
                r.d0 = low_byte(r.d0, direction);
                self.r_001620(r.d0);
                self.g_task_move(r);
                self.g_reload_sprite(r);
                r.d0 = low_byte(r.d0, 0x5D);
                r.c = self.r_0013c0(r.d0, r.a5);
            }
            self.g_reload(r);
            if self.g_count_down_obj(r, OBJ_STEP) {
                self.write8(r.a4 + OBJ_TIMER, 0);
                self.g_next(r.a4, 0x0074B8);
                if self.read8(r.a4 + OBJ_ID) == ID_PAIRED {
                    self.g_next(r.a4, 0x007602);
                }
            }
        }
        if self.read8(r.a4 + OBJ_ID) == ID_PAIRED {
            (r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c) =
                self.r_00725a(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        } else {
            self.g_finish(r);
        }
        r.unwound = false;
    }

    /// `func_0075A6`, a task: its animation a frame on (it ends with
    /// it); else moved at `SPEED` 0x200 in `TASK_17`'s direction from
    /// its position (`func_003554`), its sprite placed and given a
    /// depth, then `func_0075D6`.  (No script runs it: reviewed by
    /// reading.)
    pub fn r_0075a6(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_task_animate(r);
        if r.c {
            self.g_task_end(r);
        } else {
            r.d0 = 8;
            (r.d1, r.d2, r.x, r.c) = self.r_003554(r.d0, r.d1, r.d2, r.a6);
            self.write16(SPEED, 0x200);
            let direction = self.read8(r.a6 + TASK_17);
            r.d0 = low_byte(r.d0, direction);
            self.r_001620(r.d0);
            self.g_task_move(r);
            self.g_task_place(r);
            self.g_depth(r);
            (r.d0, r.d1, r.a0, r.x, r.c) = self.r_0075d6(r.d0, r.d1, r.d2, r.d6, r.a0, r.a6, r.x);
        }
        r.unwound = false;
    }

    /// `func_0075D6`: on the frames whose `TURN` (low 3 bits) is the
    /// task's index d6's, when the task's cell holds 1, `func_005E0E`.
    /// Gives back d0, d1, a0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0075d6(&mut self, d0: u32, d1: u32, d2: u32, d6: u32, a0: u32, a6: u32, x: bool) -> (u32, u32, u32, bool, bool) {
        let mine = d6 as u8 & 7;
        let turn = self.read8(TURN) & 7;
        let (mut d0, d1, mut a0, mut x) = (low_byte(d0, mine), low_byte(d1, turn), a0, x);
        let mut c = turn < mine;
        if turn == mine {
            d0 = self.r_0015f8(a6);
            (a0, x) = self.r_00167a();
            let cell = self.read8(a0);
            d0 = low_byte(d0, cell);
            c = cell < 1;
            if cell == 1 {
                (x, _, c) = self.r_005e0e(d2, d6, a6);
            }
        }
        (d0, d1, a0, x, c)
    }

    /// `func_007602`: the previous object's offset from `OFFSETS_762A`
    /// (which has 2 bytes; `OBJ_STEP` above 16 reads past it, into
    /// `func_00762C`), the sprite record reloaded, then `loc_0074B8`.
    pub fn r_007602(&mut self, regs: &mut Regs) {
        let r = regs;
        r.d0 &= 0xFFFF_0000;
        self.g_prev_offset(r, OFFSETS_762A, 3);
        self.g_reload_sprite(r);
        r.c = false;
        self.r_0074b8(r);
        r.unwound = false;
    }

    /// `func_00762C`: bit 0 of `OBJ_FLAGS` from `OBJ_NEXT_DIRECTION`'s,
    /// then `loc_007762`.
    pub fn r_00762c(&mut self, regs: &mut Regs) {
        let r = regs;
        let mut flags = self.read8(r.a4 + OBJ_FLAGS) & 0xFE;
        if self.read8(r.a4 + OBJ_NEXT_DIRECTION) & 1 != 0 {
            flags |= 1;
        }
        r.d0 = low_byte(r.d0, flags);
        self.write8(r.a4 + OBJ_FLAGS, flags);
        r.c = false;
        self.r_007762(r);
        r.unwound = false;
    }

    /// `func_00764A`: unless marked (then `loc_007762`), marked (bit 7),
    /// the counters cleared, the animation `ANIM_E3A4A` and
    /// `func_007674` (from now on).
    pub fn r_00764a(&mut self, regs: &mut Regs) {
        let r = regs;
        let marked = self.g_marked(r);
        r.c = false;
        if marked {
            self.g_76f0(r);
        } else {
            let mark = self.read8(r.a4 + OBJ_SAVED);
            self.write8(r.a4 + OBJ_SAVED, mark | 0x80);
            self.g_next(r.a4, 0x007674);
            self.g_clear_counters(r.a4);
            self.write32(r.a4 + OBJ_ANIM, ANIM_E3A4A);
            r.c = false;
            self.r_007674(r);
        }
        r.unwound = false;
    }

    /// `func_007674`: the animation a frame on; when it ends,
    /// `func_00764A` next and `loc_007762`.  Otherwise, on its pair
    /// 4 sound 0x52, on its pair 0x14 a task running `func_007708`
    /// (depth 0x80, priority 0x42; not checked for) at the object's
    /// position with the animation `ANIM_E3A66` for 0x80 frames; then
    /// the sprite placed, its depth, the object's cell, `func_006E28`
    /// and `func_007766`.
    pub fn r_007674(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_animate(r);
        if r.c {
            self.g_next(r.a4, 0x00764A);
            r.c = false;
            self.g_76f0(r);
        } else {
            let timer = self.read8(r.a4 + OBJ_TIMER);
            r.d0 = low_byte(r.d0, timer);
            if timer == 0 {
                if self.read8(r.a4 + OBJ_STEP) == 4 {
                    r.d0 = low_byte(r.d0, 0x52);
                    r.c = self.r_0013c0(r.d0, r.a5);
                }
                if self.read8(r.a4 + OBJ_STEP) == 0x14 {
                    self.r_001542(r.a4);
                    // the original doesn't look at C here
                    self.g_make_task(r, 0x80, 0x42, 0x007708);
                    self.r_00152a(r.a6);
                    self.write32(r.a6 + TASK_ANIM, ANIM_E3A66);
                    self.write8(r.a6 + TASK_COUNT, 0x80);
                    self.write8(r.a5 + SPRITE_FLAGS, 1);
                    self.g_reload(r);
                }
            }
            self.g_finish(r);
            r.c = self.r_007766(r.a4, r.c);
        }
        r.unwound = false;
    }

    /// `func_0076F0` on the registers.
    fn g_76f0(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0076f0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
    }

    /// `.L0076F0` (in `func_007674`): `loc_007762`.  Gives back every
    /// register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0076f0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c, ..Regs::default() };
        self.r_007762(&mut r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_007708`, a task: its animation a frame on, `TASK_COUNT`
    /// counted down (the task ends at 0); else moved from its position
    /// (`func_003554`, 0x14) and its sprite placed and given a depth.
    pub fn r_007708(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_task_animate(r);
        if self.g_count_down_task(r, TASK_COUNT) {
            self.g_task_end(r);
        } else {
            r.d0 = 0x14;
            (r.d1, r.d2, r.x, r.c) = self.r_003554(r.d0, r.d1, r.d2, r.a6);
            self.g_task_place(r);
            self.g_depth(r);
        }
        r.unwound = false;
    }

    /// `func_00772E`: `OBJ_OFFSET` 1, `OBJ_PHASE` 0, `func_004FD0`, then
    /// `func_007744` (from now on).
    pub fn r_00772e(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write8(r.a4 + OBJ_OFFSET, 1);
        self.write8(r.a4 + OBJ_PHASE, 0);
        self.g_next(r.a4, 0x007744);
        self.g_4fd0(r);
        self.r_007744(r);
        r.unwound = false;
    }

    /// `func_007744`: `OBJ_PHASE` counted up; `OBJ_OFFSET` from
    /// `OFFSETS_7726` by its bits 4-6; the sprite record reloaded, then
    /// `loc_007762`.
    pub fn r_007744(&mut self, regs: &mut Regs) {
        let r = regs;
        let phase = self.read8(r.a4 + OBJ_PHASE).wrapping_add(1);
        self.write8(r.a4 + OBJ_PHASE, phase);
        let index = (phase >> 4) & 7;
        r.d0 = low_word(r.d0, u16::from(index));
        r.x = phase & 8 != 0;
        let offset = self.read8(OFFSETS_7726 + u32::from(index));
        self.write8(r.a4 + OBJ_OFFSET, offset);
        self.g_reload_sprite(r);
        r.c = false;
        self.r_007762(r);
        r.unwound = false;
    }

    /// `loc_007762`: `func_007174`, then `func_007766`.
    pub fn r_007762(&mut self, regs: &mut Regs) {
        let r = regs;
        self.r_007174(r);
        if self.restarted.is_none() {
            r.c = self.r_007766(r.a4, r.c);
        }
        r.unwound = false;
    }

    /// `.L007766` (in `loc_007762`): `OBJ_OFFSET` cleared when C is set.
    /// Gives back C (clear).
    pub fn r_007766(&mut self, a4: u32, c: bool) -> bool {
        if c {
            self.write8(a4 + OBJ_OFFSET, 0);
        }
        false
    }

    /// `func_007774`: for an object `ID_SPLIT`, the previous object's
    /// offset from `OFFSETS_7770` by the direction.  Unless marked (then
    /// `func_00781A`), marked (all 4 bits), sound 0x53, `OBJ_KIND` 5,
    /// `OBJ_COUNTDOWN` 0x80, and `func_0077D2` (from now on).
    pub fn r_007774(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.read8(r.a4 + OBJ_ID) == ID_SPLIT {
            let direction = self.read8(r.a4 + OBJ_DIRECTION) & 3;
            r.d0 = low_word(r.d0, u16::from(direction));
            let offset = self.read8(OFFSETS_7770 + u32::from(direction));
            self.write8(r.a4.wrapping_add(OBJ_PREV_OFFSET), offset);
            self.g_reload_sprite(r);
        }
        let marked = self.g_marked(r);
        r.c = false;
        if marked {
            self.g_781a(r);
        } else {
            self.g_mark_all(r);
            self.g_next(r.a4, 0x0077D2);
            r.d0 = low_byte(r.d0, 0x53);
            r.c = self.r_0013c0(r.d0, r.a5);
            self.g_clear_counters(r.a4);
            self.write8(r.a4 + OBJ_KIND, 5);
            self.g_set_anim(r);
            self.write8(r.a4 + OBJ_COUNTDOWN, 0x80);
            r.c = false;
            self.r_0077d2(r);
        }
        r.unwound = false;
    }

    /// `func_0077D2`: the animation a frame on, `OBJ_COUNTDOWN` counted
    /// down; at 0, `OBJ_KIND` 0, `func_007774` next and `func_00781A`.
    /// Otherwise `func_007834`, then `func_00725A` (an object
    /// `ID_SPLIT`) or `func_0071A0`, and `func_007A56`.
    pub fn r_0077d2(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_animate(r);
        let count = self.read8(r.a4 + OBJ_COUNTDOWN).wrapping_sub(1);
        self.write8(r.a4 + OBJ_COUNTDOWN, count);
        if count == 0 {
            self.g_next(r.a4, 0x007774);
            self.g_clear_counters(r.a4);
            self.write8(r.a4 + OBJ_KIND, 0);
            self.g_set_anim(r);
            self.g_781a(r);
        } else {
            (r.d1, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6) =
                self.r_007834(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            if self.read8(r.a4 + OBJ_ID) == ID_SPLIT {
                (r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c) =
                    self.r_00725a(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            } else {
                self.g_cell(r);
                (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_0071a0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            }
            self.g_7a56(r);
        }
        r.unwound = false;
    }

    /// `func_00781A` on the registers.
    fn g_781a(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_00781a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
    }

    /// `func_007A56` on the registers.
    fn g_7a56(&mut self, r: &mut Regs) {
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) =
            self.r_007a56(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
    }

    /// `.L00781A` (in `func_0077D2`): `func_00722E` (an object
    /// `ID_SPLIT`) or `func_007174`, then `func_007A56`.  Gives back
    /// every register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00781a(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        if self.read8(a4 + OBJ_ID) == ID_SPLIT {
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
                self.r_00722e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        } else {
            self.r_007174(&mut r);
        }
        if self.restarted.is_none() {
            self.g_7a56(&mut r);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_007834`: when `OBJ_COUNTDOWN`'s low 3 bits are 4, a task
    /// running `func_00793A` (depth 0x80, priority 0xC2) at the object's
    /// position: its sprite's flags 4, the direction (`OBJ_DIRECTION`'s
    /// bits 0-1, through `KEPT_DIRECTION`) in `TASK_15`, 0x20 frames in
    /// `TASK_COUNT`, three random numbers (`func_001BE2`): one in
    /// `TASK_17`, one (0-7, plus 4) and one (0-7) the low nibbles of
    /// its x and y.  Then, the object running reloaded and the cell
    /// kept (`func_003E44`): when that cell is in the object's line, in
    /// front of it within 4 cells (by `OBJ_DIRECTION`: bit 0 the row
    /// else the column, bit 1 whether the distance is counted up or
    /// down), `WORD_9EEC` 0xA0 and `func_00334C`.  Gives back d1, d4,
    /// d5, d6, a0, a4, a5 and a6.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_007834(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        if self.read8(a4 + OBJ_COUNTDOWN) & 7 == 4 {
            self.r_001542(a4);
            let direction = self.read8(a4 + OBJ_DIRECTION) & 3;
            r.d0 = low_byte(r.d0, direction);
            self.write8(KEPT_DIRECTION, direction);
            self.g_make_task(&mut r, 0x80, 0xC2, 0x00793A);
            if !r.c {
                let task = r.a6;
                self.r_00152a(task);
                self.write8(r.a5 + SPRITE_FLAGS, 4);
                let direction = self.read8(KEPT_DIRECTION);
                self.write8(task + TASK_15, direction);
                self.write8(task + TASK_COUNT, 0x20);
                (r.d0, r.d1, _) = self.r_001be2(r.d1);
                self.write8(task + TASK_17, r.d0 as u8);
                (r.d0, r.d1, _) = self.r_001be2(r.d1);
                let high = self.read8(task + TASK_X_LOW) & 0xF0;
                r.d1 = low_byte(r.d1, high);
                self.write8(task + TASK_X_LOW, ((r.d0 as u8 & 7) + 4) | high);
                (r.d0, r.d1, _) = self.r_001be2(r.d1);
                let high = self.read8(task + TASK_Y_LOW) & 0xF0;
                r.d1 = low_byte(r.d1, high);
                self.write8(task + TASK_Y_LOW, (r.d0 as u8 & 7) | high);
            }
        }
        self.g_reload(&mut r);
        r.d0 = self.r_003e44();
        self.g_cell(&mut r);
        let facing = self.read8(r.a4 + OBJ_DIRECTION);
        let (same, distance) = if facing & 1 != 0 {
            (self.read8(CELL_Y) == self.read8(KEPT_Y), self.read8(CELL_X).wrapping_sub(self.read8(KEPT_X)))
        } else {
            (self.read8(CELL_X) == self.read8(KEPT_X), self.read8(CELL_Y).wrapping_sub(self.read8(KEPT_Y)))
        };
        // bit 1 and bit 0 the same: the kept cell is at 0-4 cells on;
        // else at 1-5 cells back (the difference 0xFB-0xFF)
        let ahead = if (facing & 2 != 0) == (facing & 1 != 0) { distance < 5 } else { distance >= 0xFB };
        if same && ahead {
            self.write16(WORD_9EEC, 0xA0);
            let direction = self.read8(r.a4 + OBJ_DIRECTION) & 3;
            (r.d1, r.a0) = self.r_00334c(low_byte(r.d0, direction), r.d1, r.d2, r.d5, r.d6, r.a0, r.a6);
        }
        (r.d1, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6)
    }

    /// `func_00793A`, a task: its sprite's frame 0x74, moved at `SPEED`
    /// 0x02, `TASK_17` in direction `TASK_15`; `TASK_COUNT` counted down
    /// (the task ends at 0); else its sprite placed and given a depth.
    pub fn r_00793a(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write8(r.a5 + SPRITE_FRAME, 0x74);
        self.write8(SPEED, 2);
        let low = self.read8(r.a6 + TASK_17);
        self.write8(SPEED_LOW, low);
        let direction = self.read8(r.a6 + TASK_15);
        r.d0 = low_byte(r.d0, direction);
        self.r_001620(r.d0);
        self.g_task_move(r);
        if self.g_count_down_task(r, TASK_COUNT) {
            self.g_task_end(r);
        } else {
            self.g_task_place(r);
            self.g_depth(r);
        }
        r.unwound = false;
    }

    /// `func_007970`: unless marked (then `func_007174` and
    /// `func_007A56`), marked (all 4 bits), `OBJ_KIND` 5 and
    /// `func_00799E` (from now on).
    pub fn r_007970(&mut self, regs: &mut Regs) {
        let r = regs;
        let marked = self.g_marked(r);
        r.c = false;
        if marked {
            self.r_007174(r);
            if self.restarted.is_none() {
                self.g_7a56(r);
            }
        } else {
            self.g_mark_all(r);
            self.g_next(r.a4, 0x00799E);
            self.g_clear_counters(r.a4);
            self.write8(r.a4 + OBJ_KIND, 5);
            self.g_set_anim(r);
            self.r_00799e(r);
        }
        r.unwound = false;
    }

    /// `func_00799E`: the animation a frame on; when it ends, `OBJ_KIND`
    /// 9 and `func_0079D2` (from now on).  Else from its pair 10 on
    /// `func_007A18`; before, the sprite placed, the object's cell,
    /// `func_006E28` and `func_007A56`.
    pub fn r_00799e(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_animate(r);
        if r.c {
            self.write8(r.a4 + OBJ_KIND, 9);
            self.g_set_anim(r);
            self.g_next(r.a4, 0x0079D2);
            r.c = false;
            self.r_0079d2(r);
        } else if self.read8(r.a4 + OBJ_STEP) >= 10 {
            self.g_7a18(r);
        } else {
            self.g_place(r);
            self.g_cell(r);
            self.g_6e28(r);
            self.g_7a56(r);
        }
        r.unwound = false;
    }

    /// `func_0079D2`: if marked, as `func_007174` without its end:
    /// `func_003988`, the direction taken, the animation set and a frame
    /// on, `FOR_53A6` the object's index, the sprite placed and given a
    /// depth less the offset.  Else marked (all 4 bits), `OBJ_KIND` 13
    /// and `func_007A00` (from now on).
    pub fn r_0079d2(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.g_marked(r) {
            self.g_3988(r);
            self.g_turn(r);
            self.g_set_anim(r);
            self.g_animate(r);
            self.write8(FOR_53A6, r.d4 as u8);
            self.g_place(r);
            self.g_depth_offset(r);
        } else {
            self.g_mark_all(r);
            self.g_next(r.a4, 0x007A00);
            self.g_clear_counters(r.a4);
            self.write8(r.a4 + OBJ_KIND, 13);
            self.g_set_anim(r);
            self.r_007a00(r);
        }
        r.unwound = false;
    }

    /// `func_007A00`: the animation a frame on; when it ends, `OBJ_KIND`
    /// 0 and `func_007970` next.  Then `func_007A18`.  (No script runs
    /// it: reviewed by reading.)
    pub fn r_007a00(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_animate(r);
        if r.c {
            self.write8(r.a4 + OBJ_KIND, 0);
            self.g_set_anim(r);
            self.g_next(r.a4, 0x007970);
            r.c = false;
        }
        self.g_7a18(r);
        r.unwound = false;
    }

    /// `func_007A18` on the registers.
    fn g_7a18(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_007a18(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `.L007A18` (in `func_007A00`): `FOR_53A6` the object's index, the
    /// sprite placed and given a depth.  Gives back d0, d1, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_007a18(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let mut r = Regs { d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, ..Regs::default() };
        self.write8(FOR_53A6, d4 as u8);
        self.g_place(&mut r);
        self.g_depth(&mut r);
        (r.d0, r.d1, r.x, r.c)
    }

    /// `.L007A56` (in `func_007970`): when C is set, a task running
    /// `func_007A90` (depth 2, priority 0xC2) at the object's position,
    /// its sprite's flags 1, the animation `ANIM_E3B62` and 0x4A in
    /// `TASK_COUNT`.  Gives back d5, d6, a5, a6, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_007a56(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, x, c, ..Regs::default() };
        if c {
            self.r_001542(a4);
            self.g_make_task(&mut r, 2, 0xC2, 0x007A90);
            if !r.c {
                self.r_00152a(r.a6);
                self.write8(r.a5 + SPRITE_FLAGS, 1);
                self.write32(r.a6 + TASK_ANIM, ANIM_E3B62);
                self.write8(r.a6 + TASK_COUNT, 0x4A);
                r.c = false;
            }
        }
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c)
    }

    /// `func_007A90`, a task: its sprite placed; `TASK_COUNT` counted
    /// down, and once at 0 (kept at 1 from then on) its animation run;
    /// the task ends with it.
    pub fn r_007a90(&mut self, regs: &mut Regs) {
        let r = regs;
        self.g_task_place(r);
        if self.g_count_down_task(r, TASK_COUNT) {
            self.write8(r.a6 + TASK_COUNT, 1);
            self.g_task_animate(r);
            if r.c {
                self.g_task_end(r);
            }
        }
        r.unwound = false;
    }
}
