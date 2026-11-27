//! From 0xBBC0 to 0xC000: a stage's set-up (`func_00BE96`) and the task
//! that runs with it (`func_00BDD8`), the tiles a count shows and hides
//! (`func_00BBC0`), a sprite that follows an object's
//! (`func_00BCD4`, `func_00BD14`), and what the stage's tables call
//! for its kinds of cell (`func_00BF28`, `func_00BF86`, with
//! `func_00BFE4`).

use crate::{Game, Regs};

/// Counts down a frame (`func_00BDD8`) once set: at 0xE0
/// `func_00BBC0` puts its three tasks up, at 6 takes them down.
const COUNTDOWN: u32 = 0xFFFF_AC7D;
/// When set, `func_00BDD8` calls `func_00AF26` (and clears it).
const ASK_AF26: u32 = 0xFFFF_AC7C;
/// `func_00AA00`'s arguments: where (a word, its low byte stepped).
const PLANE_AT: u32 = 0xFFFF_80C8;
const PLANE_AT_LOW: u32 = 0xFFFF_80C9;
/// The three things `func_00BBC0` puts up (`D_0D90B2`, `D_0D90BA`,
/// `D_0D90C2`, in the ROM).
const SHOWN: [u32; 3] = [0x0D90B2, 0x0D90BA, 0x0D90C2];

/// Flags of the game (`b_control.rs`): bit 0 the effects' mode, bit
/// 3 set by `loc_00B312`, bits 4 and 6 set by `func_00C028`.
const MODE: u32 = 0xFFFF_9540;
const MODE_EFFECTS: u8 = 1 << 0;
const MODE_ENDED: u8 = 1 << 3;
/// `k_vdp.rs`: set while the play is stopped.
const STOPPED: u32 = 0xFFFF_A3FC;
/// `b_control.rs`: counts the frames `func_00211C` calls `func_0087F6`.
const DRIVEN: u32 = 0xFFFF_A943;
/// `k_vdp.rs`: the VRAM copy's switch, and the count of turns.
const COPY_ON: u32 = 0xFFFF_A945;
const TURNS: u32 = 0xFFFF_A94C;

/// A number `func_00BE96` makes `STAGE` from (adding 0x2B), and the
/// one `func_00BF28` and `func_00BFE4` compare it with.
const NUMBER: u32 = 0xFFFF_953A;
const UPTO: u32 = 0xFFFF_953B;
/// An index into `MASKS` (bits 0-3), and the bits tested with it.
const MASK_INDEX: u32 = 0xFFFF_953C;
const BITS: u32 = 0xFFFF_953D;
const MASKS: u32 = 0x00C5AA;
/// `b_control.rs`: 0x39 and 0x3A are tested.
const STAGE: u32 = 0xFFFF_953F;
const STAGE_FOLLOWER: u8 = 0x3A;
const STAGE_OFFSET: u8 = 0x2B;
/// The list `func_0094D8` works through (a long).
const LIST_94D8: u32 = 0xFFFF_92FA;
/// `func_00BE96`'s list (`D_00BF1A`, in the ROM).
const SETUP_LIST: u32 = 0x00BF1A;
/// `func_00BF28`'s list, made in RAM from the ROM's `D_00BF7C`,
/// `D_00BF80`, `D_00BF84`: two 4-byte entries and the end; each
/// entry's first byte goes up by 2 for each of `BITS`' low bits set.
const BUILT_LIST: u32 = 0xFFFF_ACE0;
const BUILT_SECOND: u32 = BUILT_LIST + 4;
const BUILT_END: u32 = BUILT_LIST + 8;
const LIST_ROM: u32 = 0x00BF7C;
/// Set to 0xFF00 by `func_00BE96`, cleared by `func_00BE96`.
const WORD_D002: u32 = 0xFFFF_D002;
const BYTE_80DF: u32 = 0xFFFF_80DF;
/// A byte `func_00BF86` sets to 0xED for `func_00C054`'s task.
const BYTE_AC91: u32 = 0xFFFF_AC91;

/// The index of the task running (`task.rs`).
const CURRENT: u32 = 0xFFFF_80EE;

/// Routines (code addresses).
const FOLLOWER: u32 = 0x00BD14;
const ROUTINE_C028: u32 = 0x00C028;
const ROUTINE_C054: u32 = 0x00C054;
/// What `func_00BCD4`'s task animates (`D_0DA5B2`, in the ROM).
const FOLLOWER_ANIM: u32 = 0x0DA5B2;

/// A task's fields.
/// `func_00BBC0`'s three tasks' indices (bytes; the first 0: none).
const SHOWN_TASKS: u32 = 0x0C;
/// `FOLLOWER`'s: its animation (a long), a byte set to 1, and the
/// object it follows.
const TASK_ANIM: u32 = 0x20;
const TASK_12: u32 = 0x12;
const TASK_OBJECT: u32 = 0x1B;
/// A sprite record's byte 0x1B (`func_00BBC0` sets it to 0x80).
const SPRITE_1B: u32 = 0x1B;

/// An object's fields (`b_state.rs`): in use, its sprite record's
/// address and index.
const OBJ_USED: u32 = 0x00;
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;

/// A sprite record's fields (`sprite.rs`).
const SPRITE_DEPTH: u32 = 0x02;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X: u32 = 0x08;
const SPRITE_Y: u32 = 0x0A;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_00BBC0`: at `COUNTDOWN` 0xE0, the three of `SHOWN` put up
    /// (`func_00AA00`, at 0x1D, then a step on each) and their tasks'
    /// indices kept in the task's (a6) `SHOWN_TASKS`, unless they are
    /// already; at 6, their tasks' sprite records' byte 0x1B set to
    /// 0x80 and they are forgotten.
    /// No script runs it (checked by reading).
    pub fn r_00bbc0(&mut self, d4: u32, d5: u32, d6: u32, a6: u32) {
        match self.read8(COUNTDOWN) {
            0xE0 => {
                if self.read8(a6 + SHOWN_TASKS) != 0 {
                    return;
                }
                self.write16(PLANE_AT, 0x1D);
                for (k, &shown) in SHOWN.iter().enumerate() {
                    if k > 0 {
                        let at = self.read8(PLANE_AT_LOW);
                        self.write8(PLANE_AT_LOW, at.wrapping_add(1));
                    }
                    let (d0, _, _, _) = self.r_00aa00(d4, d5, d6, shown);
                    self.write8(a6 + SHOWN_TASKS + k as u32, d0 as u8);
                }
            }
            6 => {
                let mut d5 = d5;
                for k in 0..3 {
                    d5 = low_byte(d5, self.read8(a6 + SHOWN_TASKS + k));
                    let (_, sprite) = self.r_00060a(d5);
                    self.write8(sprite + SPRITE_1B, 0x80);
                }
                self.write8(a6 + SHOWN_TASKS, 0);
            }
            _ => {}
        }
    }

    /// `func_00BCD4`: on stage `STAGE_FOLLOWER`, a task running
    /// `FOLLOWER` (priority 0x42, sprite depth 4) for object d4,
    /// animating `FOLLOWER_ANIM`.  Gives back d6, a6 and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_00bcd4(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, bool) {
        if self.read8(STAGE) != STAGE_FOLLOWER {
            return (d6, a6, x);
        }
        let (_, d6, sprite, task, x, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 4, 0x42, FOLLOWER);
        self.write8(sprite + SPRITE_FLAGS, 4);
        self.write32(task + TASK_ANIM, FOLLOWER_ANIM);
        self.write8(task + TASK_12, 1);
        self.write8(task + TASK_OBJECT, d4 as u8);
        (d6, task, x)
    }

    /// `func_00BD14`, `FOLLOWER`: while its object is in use, its
    /// sprite at the object's sprite's place, a step in front of it
    /// (`func_0007D2`), animated (`func_0013D6`); else the task ends.
    /// Gives back the registers as they are left, X and C.
    /// No script runs it (checked by reading).
    pub fn r_00bd14(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        r.d4 = low_byte(r.d4, self.read8(r.a6 + TASK_OBJECT));
        (r.d4, r.a4, r.x) = self.r_001cfa(r.d4);
        if self.read8(r.a4 + OBJ_USED) == 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        r.d6 = low_byte(r.d6, self.read8(r.a4 + OBJ_SPRITE));
        let other = self.read32(r.a4 + OBJ_SPRITE_ADDR);
        r.a6 = other;
        let x = self.read16(other + SPRITE_X);
        self.write16(r.a5 + SPRITE_X, x);
        let y = self.read16(other + SPRITE_Y);
        self.write16(r.a5 + SPRITE_Y, y);
        let depth = self.read8(other + SPRITE_DEPTH);
        r.d0 = low_byte(r.d0, depth.wrapping_sub(1));
        r.x = depth == 0;
        r.c = self.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
        r.d6 = low_byte(r.d6, self.read8(CURRENT));
        (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
    }

    /// `func_00BDD8`, the stage's task: `func_00ACA4`; `func_00AF26`
    /// if asked (`ASK_AF26`); `COUNTDOWN` one less; in the effects'
    /// mode, not stopped and with `DRIVEN`, `func_008EEE` (the
    /// backdrop 0xE meanwhile); `func_0062C0` unless `MODE_ENDED`.
    /// Gives back the registers as they are left, X and C.
    pub fn r_00bdd8(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        (r.d0, r.d1, r.d5, r.d6, r.a0, r.a1, r.a5, r.a6, r.x) = self.r_00aca4(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.read8(ASK_AF26) != 0 {
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x) = self.r_00af26(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6);
            self.write8(ASK_AF26, 0);
        }
        // tst, or the count's borrow (never): C clear
        r.c = false;
        let count = self.read8(COUNTDOWN);
        if count != 0 {
            self.write8(COUNTDOWN, count - 1);
            r.x = false;
        }
        if self.read8(MODE) & MODE_EFFECTS != 0 && self.read8(STOPPED) == 0 && self.read8(DRIVEN) != 0 {
            self.r_000296(0xE);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d6, r.d7, r.a0, r.a1, r.a6, r.x) = self.r_008eee(r.d1, r.d3, r.d4, r.d6, r.d7, r.a6);
            r.c = self.r_000296(0xFFFF);
        }
        if self.read8(MODE) & MODE_ENDED == 0 {
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.a0, r.a1, r.a2, r.a3, r.x, r.c) = self.r_0062c0(r.d1, r.d2, r.d3, r.d4, r.d5, r.a0, r.a1, r.x);
        }
    }

    /// `func_00BE96`: a stage set up: no VRAM copy, `STAGE` from
    /// `NUMBER`; `func_009874`, the picture off, plane 0xE000 cleared,
    /// `func_000E3E`, `func_001030` (with `NUMBER` 6 meanwhile),
    /// `SETUP_LIST` through `func_0094D8`, `func_000E9E`,
    /// `func_00ABE0`, `func_00ABA6`, the window (`func_00B49A`),
    /// `func_0010EC`, `func_009B0A`, `func_00AF26`; then `func_00A506`
    /// with 4.  Gives back the registers as they are left.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_00be96(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        self.write8(COPY_ON, 0);
        self.write8(TURNS, 0);
        let number = self.read8(NUMBER);
        self.write8(STAGE, number.wrapping_add(STAGE_OFFSET));
        let mut r = Regs { d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, ..Regs::default() };
        macro_rules! all {
            ($r:expr) => {
                ($r.d0, $r.d1, $r.d2, $r.d3, $r.d4, $r.d5, $r.d6, $r.d7, $r.a0, $r.a1, $r.a2, $r.a3, $r.a4, $r.a5, $r.a6)
            };
        }
        (r.d0, r.a0, r.a1, r.x) = self.r_009874(r.a1);
        self.r_0002e0();
        self.r_0002e6();
        (r.d0, r.d1, r.d6, r.a6, r.x) = self.r_000e3e(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        let number = self.read8(NUMBER);
        self.write8(NUMBER, 6);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) = self.r_001030(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        if self.restarted.is_some() {
            return all!(r);
        }
        self.write8(NUMBER, number);
        self.write32(LIST_94D8, SETUP_LIST);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return all!(r);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) = self.r_000e9e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return all!(r);
        }
        (r.d0, r.d1, r.x) = self.r_00abe0(r.d0, r.d1, r.x);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) = self.r_00aba6(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        if self.restarted.is_some() {
            return all!(r);
        }
        self.write16(WORD_D002, 0xFF00);
        self.r_00b49a(&mut r);
        self.write8(BYTE_80DF, 0);
        self.r_0010ec();
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6) = self.r_009b0a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        if self.restarted.is_some() {
            return all!(r);
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x) = self.r_00af26(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6);
        self.write8(COUNTDOWN, 0);
        r.d0 = low_byte(r.d0, 4);
        (r.d0, r.a1, _, _) = self.r_00a506(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        all!(r)
    }

    /// `func_00BF28`: unless `NUMBER` is below `UPTO` (not 0),
    /// `BUILT_LIST` made (its entries' first bytes 2 more for each of
    /// `BITS`' low bits set, from bit 0 to the first clear) and worked
    /// through (`func_0094D8`).  Gives back the registers as they are
    /// left, X and C.
    pub fn r_00bf28(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let upto = self.read8(UPTO);
        if upto != 0 {
            let number = self.read8(NUMBER);
            regs.d0 = low_byte(regs.d0, number);
            if number < upto {
                regs.c = true;
                return;
            }
        }
        let first = self.read32(LIST_ROM);
        self.write32(BUILT_LIST, first);
        let second = self.read32(LIST_ROM + 4);
        self.write32(BUILT_SECOND, second);
        let end = self.read8(LIST_ROM + 8);
        self.write8(BUILT_END, end);
        let mut bits = self.read8(BITS) & 0x0F;
        loop {
            let out = bits & 1 != 0;
            bits >>= 1;
            if !out {
                break;
            }
            for entry in [BUILT_LIST, BUILT_SECOND] {
                let byte = self.read8(entry);
                self.write8(entry, byte.wrapping_add(2));
            }
        }
        regs.d0 = low_byte(regs.d0, bits);
        self.write32(LIST_94D8, BUILT_LIST);
        let r = &mut *regs;
        // lsr's X (the bit out: clear) goes in
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c) = self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, false);
        if self.restarted.is_some() {
            return;
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c);
    }

    /// `func_00BF86`: if `func_00BFE4` gives C clear, a task running
    /// `func_00C054` (priority 0x42, sprite depth 3; `BYTE_AC91` 0xED
    /// first) and `func_00CA06`; else `func_00C766` and a task running
    /// `func_00C028` (priority 0x42, sprite depth 0x40) at its
    /// position (`func_00152A`), frame 0xFE.  Gives back d0 4, d5, d6,
    /// a5, a6, X and C clear.
    pub fn r_00bf86(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let r = &mut *regs;
        let (_, c) = self.r_00bfe4();
        if !c {
            self.write8(BYTE_AC91, 0xED);
            (r.d5, r.d6, r.a5, r.a6, r.x, _) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 3, 0x42, ROUTINE_C054);
            self.r_00ca06(r.d0, r.a5, r.a6);
        } else {
            (r.d0, r.d5, r.d6, r.a5, r.a6, _, _) = self.r_00c766(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
            (r.d5, r.d6, r.a5, r.a6, r.x, _) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, 0x40, 0x42, ROUTINE_C028);
            self.r_00152a(r.a6);
            self.write8(r.a5 + SPRITE_FLAGS, 4);
            self.write8(r.a5 + SPRITE_FRAME, 0xFE);
        }
        r.d0 = 4;
        r.c = false;
    }

    /// `func_00BFE4`: C and X clear if `NUMBER` is `UPTO` and the bit
    /// of `BITS` that `MASK_INDEX` picks (`MASKS`) is clear; else both
    /// set.
    pub fn r_00bfe4(&mut self) -> (bool, bool) {
        let set = self.read8(UPTO) != self.read8(NUMBER) || {
            let index = self.read8(MASK_INDEX) as i8 as u32;
            self.read8(BITS) & self.read8(MASKS.wrapping_add(index)) != 0
        };
        (set, set)
    }
}
