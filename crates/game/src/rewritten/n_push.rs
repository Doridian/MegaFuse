//! A push given to the objects (`func_00E6D6`, from the table at
//! `D_00A2F4`): the routines `PER_OBJECT` of the first four objects are
//! `func_00E6F8`, which gives the object being run (one of the first 6)
//! the routine `PUSHED` with a speed (`SPEED`) that runs down each frame
//! (`func_00E7B2`, faster while object 0 asks for that way) and moves it
//! that way (`PUSH_X`, `PUSH_Y`) as it walks (`func_001FE6`).

use crate::{Game, Regs};

/// Four routines (longs) `func_00E6D6` sets to `func_00E6F8`.
const PER_OBJECT: u32 = 0xFFFF_A996;
const PER_OBJECT_COUNT: u32 = 4;
const GIVE_PUSH: u32 = 0x00E6F8;
/// The object being run (index, address).
const CURRENT: u32 = 0xFFA42C;
const CURRENT_ADDR: u32 = 0xFFA42E;
/// The push's speed (a word; while its high byte isn't 0 the object
/// keeps `PUSHED`) and way (0-3).
const SPEED: u32 = 0xFFACEE;
const WAY: u32 = 0xFFACF0;
const START_SPEED: u16 = 0x600;
/// What is added to the object's velocity each frame (words, x then
/// y; `b_walk.rs`).
const PUSH_X: u32 = 0xFF9EF2;
const PUSH_Y: u32 = 0xFF9EF4;
/// The objects' requests (8 bytes an object; object 0's byte 2), and the
/// request bit for each way (`D_009C6C`, bytes in the ROM).
const PADS: u32 = 0xFFFF_AD0A;
const PAD_REQUESTS: u32 = 2;
const WAY_BITS: u32 = 0x009C6C;
/// How much the speed runs down a frame: while object 0 asks for the
/// push's way, and else.
const SLOWER: u32 = 0x40;
const SLOW: u32 = 0x20;
/// The routines that set the push by way (`D_00E7F0`, longs in the
/// ROM).
const BY_WAY: u32 = 0x00E7F0;
/// Bytes `func_00E752` sets before `func_001FE6`: an object's index,
/// "no slide" and "fast" (`b_walk.rs`, `b_control.rs`).
const WALKER: u32 = 0xFFA93F;
const NO_SLIDE: u32 = 0xFFA93E;
const FAST: u32 = 0xFFA940;
/// What `func_002B1A` found (`func_0017B8`'s cell kind).
const FOUND: u32 = 0xFF9EF6;
/// The running object's sprite record (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;

/// An object's fields: its routine, the routine it goes back to, a
/// byte 0x15 cleared.
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_NEXT: u32 = 0x06;
const OBJ_15: u32 = 0x15;
/// The routine of a pushed object, and what it goes back to: for the
/// first 5 objects `func_001F7E`, else `loc_0045E2`.
const PUSHED: u32 = 0x00E752;
const BACK_FIRST: u32 = 0x001F7E;
const BACK_OTHER: u32 = 0x0045E2;
/// The Z80 command when the push starts.
const PUSH_SOUND: u8 = 0x5C;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_00E6D6`: the first four objects' `PER_OBJECT` routines set
    /// to `GIVE_PUSH`.  Gives back d0 ((d0 less 9) times 16, or 6), d1,
    /// a0 (after the routines), X (from the shift) and C (clear).
    pub fn r_00e6d6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let less = (r.d0 as u8).wrapping_sub(9);
        r.x = less & 0x10 != 0;
        let shifted = less << 4;
        r.d1 = GIVE_PUSH;
        for i in 0..PER_OBJECT_COUNT {
            self.write32(PER_OBJECT + 4 * i, GIVE_PUSH);
        }
        r.a0 = PER_OBJECT + 4 * PER_OBJECT_COUNT;
        r.d0 = low_byte(r.d0, shifted | 6);
        r.c = false;
        r.unwound = false;
    }

    /// `func_00E6F8`, `GIVE_PUSH`: the object being run, if one of the
    /// first 6, is pushed: its routine `PUSHED` (going back to
    /// `BACK_FIRST` or `BACK_OTHER`), the Z80 sent `PUSH_SOUND`,
    /// `SPEED` `START_SPEED`, `WAY` d0 less one.  Gives back d0, d4,
    /// a4, X, N, Z, C (clear).
    pub fn r_00e6f8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let index = self.read8(CURRENT);
        r.d4 = low_byte(r.d4, index);
        r.a4 = self.read32(CURRENT_ADDR);
        r.c = false;
        if index >= 6 {
            r.n = (index.wrapping_sub(6) as i8) < 0;
            r.z = index == 6;
            return;
        }
        let (way, borrow) = (r.d0 as u8).overflowing_sub(1);
        r.x = borrow;
        let kept = low_byte(r.d0, way);
        self.write32(r.a4 + OBJ_ROUTINE, PUSHED);
        self.r_0034b6(low_byte(kept, PUSH_SOUND));
        let back = if index < 5 { BACK_FIRST } else { BACK_OTHER };
        self.write32(r.a4 + OBJ_NEXT, back);
        self.write16(SPEED, START_SPEED);
        self.write8(r.a4 + OBJ_15, 0);
        r.d0 = kept;
        self.write8(WAY, way);
        r.n = (way as i8) < 0;
        r.z = way == 0;
    }

    /// `func_00E752`, `PUSHED`: once the speed's high byte is 0 the
    /// object goes back to its routine `OBJ_NEXT` (from the next
    /// frame).  The push is set (`func_00E7B2`) and the object walks
    /// with it (`func_004BD8`, `func_001FE6`, "no slide", not "fast");
    /// unless that stops it, `func_002B1A` with the way: if it found a
    /// kind below 3 the object goes back to `OBJ_NEXT`; else, for an
    /// object of 5 on, `func_0047C6` and `func_004828`.
    pub fn r_00e752(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(SPEED) == 0 {
            let next = self.read32(r.a4 + OBJ_NEXT);
            self.write32(r.a4 + OBJ_ROUTINE, next);
        }
        (r.d1, r.d2) = self.r_00e7b2(r.d1);
        self.write8(WALKER, 0);
        r.d0 = low_byte(r.d0, 0);
        (r.d4, r.a0, r.a4) = self.r_004bd8(r.d0, r.d4);
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
        self.write8(NO_SLIDE, 1);
        self.write8(FAST, 0);
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a4, r.a5, r.a6, r.x, r.c) = self.r_001fe6(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        if r.c {
            return;
        }
        r.d0 = low_byte(r.d0, self.read8(WAY));
        (r.d0, r.d1, r.d4, r.a4, r.x, r.c) = self.r_002b1a(r.d0, r.d1, r.d4);
        let found = self.read8(FOUND);
        r.d0 = low_byte(r.d0, found);
        if found < 3 {
            let next = self.read32(r.a4 + OBJ_NEXT);
            self.write32(r.a4 + OBJ_ROUTINE, next);
            r.c = false;
            return;
        }
        r.c = (r.d4 as u8) < 5;
        if r.c {
            return;
        }
        (r.d0, r.d1, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c) = self.r_0047c6(r.d0, r.d1, r.d4, r.d5, r.a4);
        (r.d0, r.d1, r.d4, r.d5, r.a4, r.a5, r.x, r.c) = self.r_004828(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3);
    }

    /// `func_00E7B2`: the speed `SLOW` less a frame (`SLOWER` while
    /// object 0 asks for the push's way) and the push set by way
    /// (`BY_WAY`).  Gives back d1 (the way, a word) and d2 (how much
    /// less).
    pub fn r_00e7b2(&mut self, d1: u32) -> (u32, u32) {
        let way = self.read8(WAY);
        let d1 = low_word(d1, way as i8 as i16 as u16);
        let requests = self.read8(PADS + PAD_REQUESTS);
        let bit = self.read8(WAY_BITS.wrapping_add(way as i8 as i32 as u32));
        let d2 = if requests & bit != 0 { SLOWER } else { SLOW };
        let speed = self.read16(SPEED);
        self.write16(SPEED, speed.wrapping_sub(d2 as u16));
        let index = way.wrapping_mul(4) as i8 as i32 as u32;
        let routine = self.read32(BY_WAY.wrapping_add(index));
        let d0 = low_word(u32::from(requests & bit), index as u16);
        let mut r = Regs { d0, d1, d2, a0: routine, ..Regs::default() };
        self.call_family_20(routine, &mut r);
        (d1, d2)
    }

    /// `func_00E800` (way 0): `PUSH_Y` the speed taken away.  Gives back
    /// d0.
    pub fn r_00e800(&mut self, regs: &mut Regs) {
        let speed = self.read16(SPEED).wrapping_neg();
        regs.d0 = low_word(regs.d0, speed);
        self.write16(PUSH_Y, speed);
        regs.unwound = false;
    }

    /// `func_00E80C` (way 1): `PUSH_X` the speed.
    pub fn r_00e80c(&mut self, regs: &mut Regs) {
        let speed = self.read16(SPEED);
        self.write16(PUSH_X, speed);
        regs.unwound = false;
    }

    /// `func_00E814` (way 2): `PUSH_Y` the speed.
    pub fn r_00e814(&mut self, regs: &mut Regs) {
        let speed = self.read16(SPEED);
        self.write16(PUSH_Y, speed);
        regs.unwound = false;
    }

    /// `func_00E81C` (way 3): `PUSH_X` the speed taken away.  Gives back
    /// d0.
    pub fn r_00e81c(&mut self, regs: &mut Regs) {
        let speed = self.read16(SPEED).wrapping_neg();
        regs.d0 = low_word(regs.d0, speed);
        self.write16(PUSH_X, speed);
        regs.unwound = false;
    }
}
