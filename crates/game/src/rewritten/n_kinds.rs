//! Routines of objects (a4) from the table at `D_00A334` (longs: a
//! routine, its animations, two bytes), most going on to the default
//! walk `func_007174` (`loc_007178`, `func_0071A0`, ...): one that
//! sinks and comes up again (`func_00EB62`), one that blinks
//! (`func_00EC82`), one that turns to object 0 now and then
//! (`func_00ECFA`), ones that leave a task behind (`loc_00EE12`), one
//! that makes four more (`func_00EE4C`), one that is knocked out once
//! (`func_00EEE8`) and so on.

use crate::{Game, Regs};

/// The frame count (bit 0 tested by `func_00EC26`).
const FRAME: u32 = 0xFF80D8;
/// The object being run (index).
const CURRENT: u32 = 0xFFA42C;
/// The running object's sprite record (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// An argument to `func_007174` (a direction 1-4, `func_00ECFA`).
const ARG_AABD: u32 = 0xFFAABD;
/// The objects knocked out (counted down by `func_00EEF2`).
const FALLING: u32 = 0xFFA956;
/// Set when an object's fall is over (`f_objects.rs`).
const LANDED: u32 = 0xFFAC7D;
/// A speed (a word, 8.8; `c_chase.rs`) for `func_00398C`.
const SPEED: u32 = 0xFFA960;
const SPEED_LOW: u32 = 0xFFA961;
/// How many objects `func_00EE76` makes, less one (it counts down).
const LEFT_TO_MAKE: u32 = 0xFFACF2;
/// What `func_0012A0` makes an object with: its animations and routine
/// (longs).
const OBJ_INIT_ANIMS: u32 = 0xFF80C8;
const OBJ_INIT_ROUTINE: u32 = 0xFF80D0;

/// An object's fields: its kind, routine, a byte whose high nibble
/// is a timer (`func_00EB62`, `func_00EC82`), flags (bit 7),
/// a byte set to 1 (`func_00EEF2`), a step, a direction byte (bit 7:
/// none), its facing, a frame (`func_00ECFA`), the two bytes of its
/// animation (step and timer), its animation (a long), an offset (a
/// word; its high byte the sprite's y offset), two bytes `func_00EE76`
/// sets.
const OBJ_KIND: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_TIMER: u32 = 0x06;
const OBJ_FLAGS: u32 = 0x10;
const OBJ_11: u32 = 0x11;
const OBJ_STEP: u32 = 0x12;
const OBJ_DIRECTION: u32 = 0x13;
const OBJ_FACING: u32 = 0x14;
const OBJ_FRAME: u32 = 0x15;
const OBJ_16: u32 = 0x16;
const OBJ_17: u32 = 0x17;
const OBJ_ANIM: u32 = 0x20;
const OBJ_ANIM_STEP: u32 = 0x24;
const OBJ_ANIM_TIMER: u32 = 0x25;
const OBJ_26: u32 = 0x26;
const OBJ_27: u32 = 0x27;
const OBJ_OFFSET: u32 = 0x2A;
const OBJ_OFFSET_LOW: u32 = 0x2B;
/// The byte `func_00ED9A` writes 0x16 bytes before the object (in the
/// one before it; `(-0x16,a4)` in the original).
const BEFORE: u32 = 0x16;

/// A task's fields: count, a byte, its animation.
const TASK_COUNT: u32 = 0x16;
const TASK_17: u32 = 0x17;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's flags.
const SPRITE_FLAGS: u32 = 0x05;

/// `func_00EB62`'s routines and animations: waiting, sinking, under
/// (its offset bobbing by `BOB`, `D_00EC76`, bytes in the ROM), rising
/// back, done.
const SINKER: u32 = 0x00EB62;
const SINK_WAIT: u32 = 0x00EB8C;
const SINK_WAIT_ANIM: u32 = 0x0E49DC;
const SINKING: u32 = 0x00EBB6;
const SINKING_ANIM: u32 = 0x0E49E6;
const UNDER: u32 = 0x00EBDE;
const UNDER_ANIM: u32 = 0x0E4A00;
const BOB: u32 = 0x00EC76;
const RISING: u32 = 0x00EC26;
const RISING_ANIM: u32 = 0x0E49EE;
const RISEN: u32 = 0x00EC56;
const RISEN_ANIM: u32 = 0x0E49F6;
/// How far down it sinks (the offset's high byte).
const SUNK: u8 = 0xE0;

/// `func_00EC82`'s routines and frames.
const BLINK_A: u32 = 0x00EC82;
const BLINK_B: u32 = 0x00ECA4;
const BLINK_C: u32 = 0x00ECBE;
const BLINK_D: u32 = 0x00ECE0;
const OPEN_FRAME: u8 = 5;
const SHUT_FRAME: u8 = 9;

/// `func_00ECFA`'s routines; its turn by facing (`D_00ED8A`, signed
/// bytes in the ROM; 0 none, else the sign picks the frame).
const TURNER: u32 = 0x00ECFA;
const TURNING: u32 = 0x00ED5E;
const TURNS: u32 = 0x00ED8A;
/// `func_00ED9A`'s bytes by animation step (`D_00ED92`, in the ROM).
const BY_STEP: u32 = 0x00ED92;
/// The kind `func_00EDB8` treats apart.
const KIND_1C: u8 = 0x1C;

/// The task `loc_00EE12` leaves behind: routine, priority, depth,
/// animation, count.
const LEFT_TASK: u32 = 0x007A90;
const LEFT_ANIM: u32 = 0x0E4B1E;
const LEFT_COUNT: u8 = 0x50;

/// `func_00EE4C`'s task (`MAKER`, its count `MAKER_COUNT`), the objects
/// it makes (kind `MADE_KIND`, routine `MADE_ROUTINE`, animations
/// `MADE_ANIMS`).
const MAKER: u32 = 0x00EE76;
const MAKER_COUNT: u8 = 0x76;
const MADE_KIND: u8 = 0x1E;
const MADE_ROUTINE: u32 = 0x00EEE4;
const MADE_ANIMS: u32 = 0x00FF7C;
const TO_MAKE: u8 = 3;

/// `func_00EEE8`'s routines and the animation once knocked out.
const KNOCKABLE: u32 = 0x00EEE8;
const KNOCKED: u32 = 0x00EF16;
const KNOCKED_ANIM: u32 = 0x0E54EA;
/// The animation step from which it can be hit again (`func_00EF28`).
const HITTABLE_FROM: u8 = 0x1A;

/// `func_00EF44`'s routine and animation once `LANDED` is set.
const WAKER: u32 = 0x00EF44;
const WAKING: u32 = 0x00EF68;
const WAKING_ANIM: u32 = 0x0E5556;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_00EB5A`: the default walk (`func_007174`).
    pub fn r_00eb5a(&mut self, regs: &mut Regs) {
        self.r_007174(regs);
    }

    /// `func_00EB62`, `SINKER`: walks (`func_007174`) while its timer
    /// runs; then (its timer 0xF0) `SINK_WAIT` with `SINK_WAIT_ANIM`.
    pub fn r_00eb62(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let timer = self.read8(r.a4 + OBJ_TIMER) & 0xF0;
        r.d0 = low_byte(r.d0, timer);
        r.c = false;
        if timer != 0 {
            self.r_007174(r);
            return;
        }
        r.d0 = low_byte(r.d0, 0xF0);
        self.write8(r.a4 + OBJ_TIMER, 0xF0);
        self.write32(r.a4 + OBJ_ROUTINE, SINK_WAIT);
        self.n_new_anim(r.a4, SINK_WAIT_ANIM);
        self.r_00eb8c(r);
    }

    /// The object's animation `anim` from its start.
    fn n_new_anim(&mut self, a4: u32, anim: u32) {
        self.write8(a4 + OBJ_ANIM_STEP, 0);
        self.write8(a4 + OBJ_ANIM_TIMER, 0);
        self.write32(a4 + OBJ_ANIM, anim);
    }

    /// `func_00EB8C`, `SINK_WAIT`: at the end of its animation
    /// (`func_001336`) it sinks: `SINKING` with `SINKING_ANIM`, its
    /// offset 0, step 0, flag bit 7 set, `func_004FD0`.  Else drawn
    /// (`loc_00EC66`).
    pub fn r_00eb8c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.n_animate(r);
        if !r.c {
            (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_00ec66(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, SINKING);
        self.write32(r.a4 + OBJ_ANIM, SINKING_ANIM);
        self.write16(r.a4 + OBJ_OFFSET, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        let flags = self.read8(r.a4 + OBJ_FLAGS);
        self.write8(r.a4 + OBJ_FLAGS, flags | 0x80);
        (r.d0, r.x, r.c) = self.r_004fd0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.r_00ebb6(r);
    }

    /// The object's animation a frame on (`func_001336`).
    fn n_animate(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.z, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
    }

    /// `func_00EBB6`, `SINKING`: animated, its offset's high byte one
    /// less a frame down to `SUNK`; then `UNDER` with `UNDER_ANIM` for
    /// 0x80 frames.  Drawn (`loc_00EC0A`).
    pub fn r_00ebb6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.n_animate(r);
        let (offset, borrow) = self.read8(r.a4 + OBJ_OFFSET).overflowing_sub(1);
        self.write8(r.a4 + OBJ_OFFSET, offset);
        r.x = borrow;
        if offset != SUNK {
            (r.d0, r.d1, r.x, r.c) = self.r_00ec0a(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            return;
        }
        self.write8(r.a4 + OBJ_OFFSET_LOW, 0x80);
        self.write32(r.a4 + OBJ_ANIM, UNDER_ANIM);
        self.write32(r.a4 + OBJ_ROUTINE, UNDER);
        r.c = false;
        self.r_00ebde(r);
    }

    /// `func_00EBDE`, `UNDER`: animated; its offset's high byte bobs
    /// (`BOB` by the count in its low byte); at the count's end
    /// `RISING` with `RISING_ANIM`.  Drawn (`loc_00EC0A`).
    pub fn r_00ebde(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.n_animate(r);
        let (count, borrow) = self.read8(r.a4 + OBJ_OFFSET_LOW).overflowing_sub(1);
        self.write8(r.a4 + OBJ_OFFSET_LOW, count);
        r.x = borrow;
        if count == 0 {
            self.write8(r.a4 + OBJ_OFFSET_LOW, 0);
            self.write32(r.a4 + OBJ_ANIM, RISING_ANIM);
            self.write32(r.a4 + OBJ_ROUTINE, RISING);
            r.c = false;
            self.r_00ec26(r);
            return;
        }
        r.x = count & 2 != 0;
        let index = (count >> 2) & 7;
        r.d0 = low_word(r.d0, u16::from(index));
        r.a0 = BOB;
        let bob = self.read8(BOB + u32::from(index));
        self.write8(r.a4 + OBJ_OFFSET, bob);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        (r.d0, r.d1, r.x, r.c) = self.r_00ec0a(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `loc_00EC0A`: the sprite placed with the object's offset
    /// (`func_001B32`) and its depth (`func_001B50`).  Gives back d0,
    /// d1, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00ec0a(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, bool, bool) {
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        let (d0, x, c) = self.r_001b50(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        (d0, d1, x, c)
    }

    /// `func_00EC26`, `RISING`: animated; every other frame its
    /// offset's high byte one more, up to 0: then `RISEN` with
    /// `RISEN_ANIM`, flag bit 7 clear.  Drawn (`loc_00EC0A`).
    pub fn r_00ec26(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.n_animate(r);
        if self.read8(FRAME) & 1 != 0 {
            let (offset, carry) = self.read8(r.a4 + OBJ_OFFSET).overflowing_add(1);
            self.write8(r.a4 + OBJ_OFFSET, offset);
            r.x = carry;
            if offset == 0 {
                self.n_new_anim(r.a4, RISEN_ANIM);
                self.write32(r.a4 + OBJ_ROUTINE, RISEN);
                r.c = false;
                let flags = self.read8(r.a4 + OBJ_FLAGS);
                self.write8(r.a4 + OBJ_FLAGS, flags & 0x7F);
                self.r_00ec56(r);
                return;
            }
        }
        (r.d0, r.d1, r.x, r.c) = self.r_00ec0a(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00EC56`, `RISEN`: animated; at its end `SINKER` again.
    /// Drawn (`loc_00EC66`).
    pub fn r_00ec56(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.n_animate(r);
        if r.c {
            self.write32(r.a4 + OBJ_ROUTINE, SINKER);
            r.c = false;
        }
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_00ec66(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `loc_00EC66`: the sprite placed (`func_001B32`) with its depth
    /// (`func_001B5C`), the object's cell (`func_0015E6`) and what hits
    /// it there (`func_006E28`).  Gives back d0, d1, d2, a0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00ec66(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, bool, bool) {
        let (d0, d1, _, _) = self.r_001b32(d1, a4, a5);
        self.r_001b5c(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5);
        let (d0, _) = self.r_0015e6(a4);
        self.r_006e28(d0, d1, d2, a0, a4, a5)
    }

    /// `loc_00EC7E`: the default walk (`func_007174`).  Gives back every
    /// register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00ec7e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c, ..Regs::default() };
        self.r_007174(&mut r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// The default walk through `loc_00EC7E`, on the registers `r`.
    fn n_walk(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_00ec7e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
        r.unwound = false;
    }

    /// `func_00EC82`, `BLINK_A`: once its timer is out, frame
    /// `OPEN_FRAME` from its animation's start, `BLINK_B`.  It walks
    /// (`loc_00EC7E`).
    pub fn r_00ec82(&mut self, regs: &mut Regs) {
        self.n_blink_timer(regs, BLINK_B);
    }

    /// `func_00ECBE`, `BLINK_C`: as `BLINK_A`, going on to `BLINK_D`.
    pub fn r_00ecbe(&mut self, regs: &mut Regs) {
        self.n_blink_timer(regs, BLINK_D);
    }

    /// `func_00EC82` and `func_00ECBE`: once the timer is out, frame
    /// `OPEN_FRAME` from the animation's start, routine `next`; then the
    /// walk.
    fn n_blink_timer(&mut self, r: &mut Regs, next: u32) {
        let timer = self.read8(r.a4 + OBJ_TIMER) & 0xF0;
        r.d0 = low_byte(r.d0, timer);
        r.c = false;
        if timer == 0 {
            self.write8(r.a4 + OBJ_FRAME, OPEN_FRAME);
            self.write8(r.a4 + OBJ_ANIM_STEP, 0);
            self.write8(r.a4 + OBJ_ANIM_TIMER, 0);
            self.write32(r.a4 + OBJ_ROUTINE, next);
        }
        self.n_walk(r);
    }

    /// `func_00ECA4`, `BLINK_B`: once its animation's timer is 0, frame
    /// `SHUT_FRAME`, `BLINK_C` (its timer's bit 7 set) from now on.
    /// Else it walks (`loc_00EC7E`).
    pub fn r_00eca4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let timer = self.read8(r.a4 + OBJ_ANIM_TIMER);
        r.d0 = low_byte(r.d0, timer);
        r.c = false;
        if timer != 0 {
            self.n_walk(r);
            return;
        }
        self.write8(r.a4 + OBJ_FRAME, SHUT_FRAME);
        self.write32(r.a4 + OBJ_ROUTINE, BLINK_C);
        let held = self.read8(r.a4 + OBJ_TIMER);
        self.write8(r.a4 + OBJ_TIMER, held | 0x80);
        self.r_00ecbe(r);
    }

    /// `func_00ECE0`, `BLINK_D`: once its animation's timer is 0, frame
    /// 0, `BLINK_A` again, its timer 0xF0 more.  It walks
    /// (`loc_00EC7E`).
    pub fn r_00ece0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let timer = self.read8(r.a4 + OBJ_ANIM_TIMER);
        r.d0 = low_byte(r.d0, timer);
        r.c = false;
        if timer == 0 {
            self.write8(r.a4 + OBJ_FRAME, 0);
            self.write32(r.a4 + OBJ_ROUTINE, BLINK_A);
            let held = self.read8(r.a4 + OBJ_TIMER);
            self.write8(r.a4 + OBJ_TIMER, held | 0xF0);
        }
        self.n_walk(r);
    }

    /// `func_00ECFA`, `TURNER`: it walks (`func_007174`, its facing's
    /// way plus one in `ARG_AABD`); unless that has it leave a task
    /// (C, `loc_00ED86`), if it has a direction whose turn from the way
    /// (`TURNS`) isn't 0 it turns: frame 5 (or 9 for a turn the other
    /// way, `func_00130E`), its animation from the start (`func_001852`)
    /// and `TURNING` from now on.  Then drawn (`loc_00ED7A`).
    pub fn r_00ecfa(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (way, carry) = (self.read8(r.a4 + OBJ_FACING) & 3).overflowing_add(1);
        r.d0 = low_byte(r.d0, way);
        r.x = carry;
        self.write8(ARG_AABD, way);
        r.c = false;
        self.r_007174(r);
        if self.restarted.is_some() {
            return;
        }
        r.unwound = false;
        if r.c {
            (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00ed86(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
            return;
        }
        let direction = self.read8(r.a4 + OBJ_DIRECTION);
        r.d0 = low_byte(r.d0, direction);
        r.c = false;
        if (direction as i8) < 0 {
            self.n_ed7a(r);
            return;
        }
        let from = direction.wrapping_add(1).wrapping_sub(self.read8(ARG_AABD));
        let (index, carry) = from.overflowing_add(3);
        r.x = carry;
        let turn = self.read8(TURNS.wrapping_add(index as i8 as i32 as u32));
        r.d0 = low_byte(low_word(r.d0, index as i8 as i16 as u16), turn);
        r.c = false;
        if turn == 0 {
            self.n_ed7a(r);
            return;
        }
        let frame = if (turn as i8) < 0 { SHUT_FRAME } else { OPEN_FRAME };
        self.write8(r.a4 + OBJ_FRAME, frame);
        (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
        self.write8(r.a4 + OBJ_ANIM_STEP, 0);
        self.write8(r.a4 + OBJ_ANIM_TIMER, 0);
        self.r_001852(r.a4);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        self.write32(r.a4 + OBJ_ROUTINE, TURNING);
        r.c = false;
        self.r_00ed5e(r);
    }

    /// `loc_00ED7A` on the registers `r`.
    fn n_ed7a(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.d6, r.a0, r.a5, r.a6, r.x, r.c) = self.r_00ed7a(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a6);
    }

    /// `func_00ED5E`, `TURNING`: animated; at its end frame 0
    /// (`func_00130E`) and `TURNER` again.  The object's cell
    /// (`func_0015E6`), drawn (`loc_00ED7A`).
    pub fn r_00ed5e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.n_animate(r);
        if r.c {
            self.write8(r.a4 + OBJ_FRAME, 0);
            (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
            self.write32(r.a4 + OBJ_ROUTINE, TURNER);
            r.c = false;
        }
        (r.d0, r.x) = self.r_0015e6(r.a4);
        self.n_ed7a(r);
    }

    /// `loc_00ED7A`: the object drawn and hit (`func_0071A0`), then
    /// `loc_00ED86`.  Gives back d0, d1, d2, d5, d6, a0, a5, a6, X, C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00ed7a(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let d5 = low_byte(d5, self.read8(SLOT_INDEX));
        let a5 = self.read32(SLOT_ADDR);
        let (d0, d1, d2, a0, x, c) = self.r_0071a0(d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5);
        let (d5, d6, a5, a6, x, c) = self.r_00ed86(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, x, c);
        (d0, d1, d2, d5, d6, a0, a5, a6, x, c)
    }

    /// `loc_00ED86`: `loc_00EE12`.  Gives back d5, d6, a5, a6, X, C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00ed86(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, bool, bool) {
        self.r_00ee12(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, a6, x, c)
    }

    /// `func_00ED9A`: the byte `BEFORE` bytes before the object set from
    /// `BY_STEP` by its animation's timer over 2, less one (not below
    /// 0); then `func_00EDB8`.
    pub fn r_00ed9a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let half = self.read8(r.a4 + OBJ_ANIM_TIMER) >> 1;
        let (less, borrow) = half.overflowing_sub(1);
        r.x = borrow;
        let index = if (less as i8) < 0 { 0 } else { less };
        r.d0 = low_word(r.d0, index as i8 as i16 as u16);
        let byte = self.read8(BY_STEP.wrapping_add(index as i8 as i32 as u32));
        self.write8(r.a4.wrapping_sub(BEFORE), byte);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.c = false;
        r.a5 = self.read32(SLOT_ADDR);
        self.r_00edb8(r);
    }

    /// `func_00EDB8`: with its animation's timer 6 or more (but not
    /// 0x10) it walks (`func_007174`, or `func_00722E` for kind
    /// `KIND_1C`); else its animation from the start (`func_001852`,
    /// `OBJ_16`, `OBJ_17` cleared) and `func_00718E` (or, for kind
    /// `KIND_1C`, frame `func_00130E`, `func_001336` and
    /// `func_00725A`).  Then `loc_00EE12`.
    pub fn r_00edb8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let timer = self.read8(r.a4 + OBJ_ANIM_TIMER);
        r.d0 = low_byte(r.d0, timer);
        if timer >= 6 && timer != 0x10 {
            let kind = self.read8(r.a4 + OBJ_KIND);
            r.d0 = low_byte(r.d0, kind);
            r.z = kind == KIND_1C;
            r.c = kind < KIND_1C;
            if kind == KIND_1C {
                (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_00722e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            } else {
                self.r_007174(r);
                r.unwound = false;
            }
            if self.restarted.is_some() {
                return;
            }
        } else {
            self.r_001852(r.a4);
            self.write8(r.a4 + OBJ_16, 0);
            self.write8(r.a4 + OBJ_17, 0);
            if self.read8(r.a4 + OBJ_KIND) == KIND_1C {
                (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
                self.n_animate(r);
                (r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c) = self.r_00725a(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
            } else {
                (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_00718e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a6);
                if self.restarted.is_some() {
                    return;
                }
            }
        }
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00ee12(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
    }

    /// `loc_00EE12`: with C set, the object's position (`func_001542`)
    /// gets a task `LEFT_TASK` (`func_001222`; animation `LEFT_ANIM`,
    /// sprite flags 1, count `LEFT_COUNT`).  Gives back d5, d6, a5, a6,
    /// X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00ee12(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool, c: bool) -> (u32, u32, u32, u32, bool, bool) {
        if !c {
            return (d5, d6, a5, a6, x, c);
        }
        self.r_001542(a4);
        let (d5, d6, a5, a6, x, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 2, 0xC2, LEFT_TASK);
        if failed {
            return (d5, d6, a5, a6, x, true);
        }
        self.write32(a6 + TASK_ANIM, LEFT_ANIM);
        self.r_00152a(a6);
        self.write8(a5 + SPRITE_FLAGS, 1);
        self.write8(a6 + TASK_COUNT, LEFT_COUNT);
        (d5, d6, a5, a6, x, false)
    }

    /// `func_00EE4C`: it walks (`func_007174`); if that has it leave
    /// (C), a task `MAKER` (`func_0003E8`) with count `MAKER_COUNT` and
    /// the object's index.
    pub fn r_00ee4c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.r_007174(r);
        if self.restarted.is_some() {
            return;
        }
        r.unwound = false;
        if !r.c {
            return;
        }
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, 0x42, MAKER);
        if !r.c {
            self.write8(r.a6 + TASK_COUNT, MAKER_COUNT);
            let index = self.read8(CURRENT);
            self.write8(r.a6 + TASK_17, index);
            r.c = false;
        }
    }

    /// `loc_00EE74`: it returns.
    pub fn r_00ee74(&mut self) {}

    /// `func_00EE76`, `MAKER`: at the end of its count, from the
    /// position of the object it was made for (`func_001542`) up to four
    /// objects (`func_001456`, `func_0012A0`) of kind `MADE_KIND`,
    /// numbered 3 down to 0 in their direction, facing and timer; then
    /// the task ends (`func_000560`).
    pub fn r_00ee76(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (count, borrow) = self.read8(r.a6 + TASK_COUNT).overflowing_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        r.x = borrow;
        if count != 0 {
            return;
        }
        self.write8(LEFT_TO_MAKE, TO_MAKE);
        let index = self.read8(r.a6 + TASK_17);
        (r.d4, r.a4, r.x) = self.r_001cfa(low_byte(r.d4, index));
        self.r_001542(r.a4);
        loop {
            let full;
            (r.d0, r.x, full) = self.r_001456(r.d0);
            r.c = full;
            if full {
                break;
            }
            (r.d4, r.a4, r.x) = self.r_001cfa(low_byte(r.d4, r.d0 as u8));
            self.write32(OBJ_INIT_ANIMS, MADE_ANIMS);
            self.write32(OBJ_INIT_ROUTINE, MADE_ROUTINE);
            (r.d0, r.d5, r.a5, r.x) = self.r_0012a0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
            self.write8(r.a5 + SPRITE_FLAGS, 1);
            let number = self.read8(LEFT_TO_MAKE);
            r.d0 = low_byte(r.d0, number);
            self.write8(r.a4 + OBJ_DIRECTION, number);
            self.write8(r.a4 + OBJ_FACING, number);
            self.write8(r.a4 + OBJ_TIMER, number);
            self.write8(r.a4 + OBJ_26, 0x0C);
            self.write8(r.a4 + OBJ_27, 1);
            self.write8(r.a4 + OBJ_KIND, MADE_KIND);
            let (left, borrow) = number.overflowing_sub(1);
            self.write8(LEFT_TO_MAKE, left);
            r.x = borrow;
            r.c = borrow;
            if (left as i8) < 0 {
                break;
            }
        }
        (r.d6, r.x, r.c) = self.r_000560(r.d5, r.d6, r.x);
    }

    /// `func_00EEE4`, `MADE_ROUTINE`: the default walk (`func_007174`).
    pub fn r_00eee4(&mut self, regs: &mut Regs) {
        self.r_007174(regs);
    }

    /// `func_00EEE8`, `KNOCKABLE`: it walks (`func_007174`); when hit
    /// (C) it is knocked out (`func_00EEF2`).
    pub fn r_00eee8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.n = false;
        r.z = false;
        r.v = false;
        self.r_007174(r);
        if self.restarted.is_some() {
            return;
        }
        r.unwound = false;
        if r.c {
            self.r_00eef2(r);
        }
    }

    /// `func_00EEF2`: knocked out: frame 0, step 0, `OBJ_11` 1,
    /// `FALLING` one less, `KNOCKED` with `KNOCKED_ANIM`.
    pub fn r_00eef2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.write8(r.a4 + OBJ_FRAME, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.write8(r.a4 + OBJ_11, 1);
        let (falling, borrow) = self.read8(FALLING).overflowing_sub(1);
        self.write8(FALLING, falling);
        r.x = borrow;
        self.write32(r.a4 + OBJ_ROUTINE, KNOCKED);
        self.write32(r.a4 + OBJ_ANIM, KNOCKED_ANIM);
        r.n = false;
        r.z = false;
        r.c = false;
        self.r_00ef16(r);
    }

    /// `func_00EF16`, `KNOCKED`: animated; at its end `KNOCKABLE` again
    /// (run now), else `func_00EF28`.
    pub fn r_00ef16(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.n_animate(r);
        if !r.c {
            self.r_00ef28(r);
            return;
        }
        self.write32(r.a4 + OBJ_ROUTINE, KNOCKABLE);
        r.c = false;
        self.r_00eee8(r);
    }

    /// `func_00EF28`: the sprite placed (`func_001B32`, `func_001B50`);
    /// from its animation's timer `HITTABLE_FROM` on, the object's cell
    /// (`func_0015E6`) and what hits it there (`func_006E28`).  Gives
    /// back N and Z (from that comparison) and C.
    pub fn r_00ef28(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
        (r.d0, r.x, r.c) = self.r_001b50(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
        let timer = self.read8(r.a4 + OBJ_ANIM_TIMER);
        r.n = (timer.wrapping_sub(HITTABLE_FROM) as i8) < 0;
        r.z = timer == HITTABLE_FROM;
        r.c = timer < HITTABLE_FROM;
        if r.c {
            return;
        }
        (r.d0, r.x) = self.r_0015e6(r.a4);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_006e28(r.d0, r.d1, r.d2, r.a0, r.a4, r.a5);
    }

    /// `func_00EF42`: it returns.
    pub fn r_00ef42(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `func_00EF44`, `WAKER`: it walks (`func_007174`) until `LANDED`
    /// is set; then `WAKING` with `WAKING_ANIM`.
    pub fn r_00ef44(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let landed = self.read8(LANDED);
        r.d0 = low_byte(r.d0, landed);
        r.z = landed == 0;
        r.c = false;
        if landed == 0 {
            self.r_007174(r);
            return;
        }
        self.n_new_anim(r.a4, WAKING_ANIM);
        self.write32(r.a4 + OBJ_ROUTINE, WAKING);
        self.r_00ef68(r);
    }

    /// `func_00EF68`, `WAKING`: animated; at its end `WAKER` again.  The
    /// object's cell (`func_0015E6`), drawn and hit (`func_0071A0`).
    pub fn r_00ef68(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.n_animate(r);
        if r.c {
            self.write32(r.a4 + OBJ_ROUTINE, WAKER);
            r.c = false;
        }
        (r.d0, r.x) = self.r_0015e6(r.a4);
        (r.d0, r.d1, r.d2, r.a0, r.x, r.c) = self.r_0071a0(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_00EF80`: its speed (`OBJ_OFFSET_LOW`, 0 when its animation
    /// is at its first step) 2 less a frame, but not below 0x20, is the
    /// speed `func_00398C` turns it by; then `loc_007178`.
    pub fn r_00ef80(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let step = self.read8(r.a4 + OBJ_ANIM_STEP);
        r.d0 = low_byte(r.d0, step);
        if step == 0 {
            self.write8(r.a4 + OBJ_OFFSET_LOW, 0);
        }
        let (speed, borrow) = self.read8(r.a4 + OBJ_OFFSET_LOW).overflowing_sub(2);
        r.x = borrow;
        r.d0 = if speed < 0x20 { 0x20 } else { low_byte(r.d0, speed) };
        self.write8(r.a4 + OBJ_OFFSET_LOW, r.d0 as u8);
        self.write8(SPEED_LOW, r.d0 as u8);
        self.write8(SPEED, 0);
        r.c = false;
        (r.d0, r.d1, r.d4, r.a4) = self.r_00398c(r.d0, r.d1, r.d4, r.a4, r.a5);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_007178(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a6);
        r.unwound = false;
    }

    /// `func_00EFB0`: it walks (`func_007174`), then `loc_00F1D0`.
    pub fn r_00efb0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        self.r_007174(r);
        if self.restarted.is_some() {
            return;
        }
        r.unwound = false;
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_00f1d0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
    }
}
