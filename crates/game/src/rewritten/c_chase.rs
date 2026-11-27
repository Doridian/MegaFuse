//! Objects that pick their direction by the cell of object 0 (the
//! object `func_001CFA` gives for 0, at `OBJECT_0`): how far they are
//! from it, the ways open to them, and a routine by their kind
//! (`func_003B94`'s table) that picks one.
//!
//! A direction is 0-3: a step of (0, -1), (1, 0), (0, 1), (-1, 0) in
//! cells (`STEP_X`, `STEP_Y`); `REVERSE` turns one around.

use crate::{Game, Regs};

/// An object's fields (`anim.rs` has more).
/// A timer (bits 0-3) and what it starts again at (bits 4-7,
/// `func_006FA0`).
const OBJ_TIMER: u32 = 0x06;
/// A count of turns that keeps the routine of `func_003B94` on its
/// course (`func_003C48`, `func_003D3C`).
const OBJ_HOLD: u32 = 0x07;
/// Bit 0: set while `OBJ_HOLD` counts (`func_003C48`), and the speed
/// is half again (`func_003928`); bit 6: ways 2 and 3 are closed to it
/// (`func_003AC4`).
const OBJ_FLAGS: u32 = 0x10;
/// Its direction (bit 7: none).
const OBJ_DIRECTION: u32 = 0x13;
/// Its direction for its animation (`anim.rs`); bit 7 is cleared while
/// it moves, set while it picks.
const OBJ_ANIM_DIRECTION: u32 = 0x14;
/// What `func_003B94` picks a routine by (bits 0-3).
const OBJ_KIND: u32 = 0x26;
/// What a change of direction adds to `OBJ_TIMER`.
const OBJ_TIMER_ADD: u32 = 0x27;
/// A cell (column, row) it compares with `CELL` (`func_003FAA`).
const OBJ_HOME: u32 = 0x28;
const OBJ_HOME_Y: u32 = 0x29;

/// Object 0 (`func_001CFA`'s first) and its fields read here: its
/// position (x, y: words), its direction.
const OBJECT_0: u32 = 0xFFA432;
const OBJECT_0_POSITION: u32 = OBJECT_0 + 0x18;
const OBJECT_0_DIRECTION: u32 = OBJECT_0 + 0x13;

/// The object being run: its index (`func_001CFA`'s) and address.
const CURRENT_INDEX: u32 = 0xFFA42C;
const CURRENT: u32 = 0xFFA42E;

/// A map cell (column, row): the object's own here (`func_0015E6`), the
/// one looked at in `func_003EC4`, moved in `func_003F70`.
const CELL: u32 = 0xFF9F08;
const CELL_X: u32 = 0xFF9F08;
const CELL_Y: u32 = 0xFF9F09;
/// Object 0's cell (`func_003E44`): the cell the objects go by.
const TARGET: u32 = 0xFFA966;
const TARGET_X: u32 = 0xFFA966;
const TARGET_Y: u32 = 0xFFA967;
/// The column and row differences to `TARGET` (`func_003F1A`).
const DIFF_X: u32 = 0xFFAC8A;
const DIFF_Y: u32 = 0xFFAC8C;
/// The direction the object had before it picked (`func_00398C`).
const LAST_DIRECTION: u32 = 0xFFAC8B;
/// A speed (a word, 8.8): what a move goes by along its axis
/// (`VELOCITY_X` or `VELOCITY_Y`, negated for directions 0 and 3).
const SPEED: u32 = 0xFFA960;
const SPEED_LOW: u32 = 0xFFA961;
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_Y: u32 = 0xFF9EC6;
/// While this isn't 0, `func_00398C` does nothing.
const STOPPED: u32 = 0xFFA957;
/// One byte by direction: 1 where the object may go (`func_003AEA`).
const OPEN: u32 = 0xFFA962;
/// The map cell `func_00167A` last looked at (a pointer; `func_00173E`
/// calls it for the cell ahead).
const CELL_POINTER: u32 = 0xFF9ED6;

/// The tables in the ROM.
/// The speed by the object's kind (its byte 0, from 2: a byte, the
/// speed's low byte; 0 for 0x100).
const SPEEDS: u32 = 0x003960;
/// The direction opposite each (D_009C70).
const REVERSE: u32 = 0x009C70;
/// The step of each direction (bytes): x from `STEP_X`, y from `STEP_Y`.
const STEP_X: u32 = 0x003F0C;
const STEP_Y: u32 = 0x003F0F;
/// `func_003F70`'s steps (x: by object 0's direction; y: see there).
const STEP2_X: u32 = 0x003FA2;
const STEP2_Y: u32 = 0x003FA5;
/// A direction by object 0's (`func_003D6A`).
const AHEAD: u32 = 0x004048;
/// The routine by kind (D_003BBE, longs).
const BY_KIND: u32 = 0x003BBE;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// `neg.b` of a byte that `sub.b` left negative: its distance.
fn distance(d: u8) -> u8 {
    if d & 0x80 != 0 {
        d.wrapping_neg()
    } else {
        d
    }
}

impl Game {
    /// `func_003928`: `SPEED` by the object's kind (its byte 0), half
    /// again while bit 0 of `OBJ_FLAGS` is set; then d4 and a4 back to
    /// the object being run (`CURRENT_INDEX`, `CURRENT`).
    pub fn r_003928(&mut self, d0: u32, d4: u32, obj: u32) -> (u32, u32, u32) {
        let kind = self.read8(obj).wrapping_sub(2);
        let mut d0 = low_word(d0, kind as i8 as i16 as u16);
        self.write8(SPEED, 0);
        let speed = self.read8(SPEEDS.wrapping_add(kind as i8 as i32 as u32));
        self.write8(SPEED_LOW, speed);
        if speed == 0 {
            self.write8(SPEED, 1);
        }
        if self.read8(obj + OBJ_FLAGS) & 1 != 0 {
            let speed = self.read16(SPEED);
            d0 = low_word(d0, speed >> 1);
            self.write16(SPEED, speed.wrapping_add(speed >> 1));
        }
        let d4 = low_byte(d4, self.read8(CURRENT_INDEX));
        let a4 = self.read32(CURRENT);
        (d0, d4, a4)
    }

    /// `func_003988`: `func_003928`, then `func_00398C`.
    pub fn r_003988(&mut self, d0: u32, d1: u32, d4: u32, obj: u32, a5: u32) -> (u32, u32, u32, u32) {
        let (d0, d4, obj) = self.r_003928(d0, d4, obj);
        self.r_00398c(d0, d1, d4, obj, a5)
    }

    /// `func_00398C`: the object's move.  On a cell's middle (both
    /// `func_00182E` and `func_001840` give 0 or 9), the first time
    /// there (bit 7 of `OBJ_ANIM_DIRECTION` set for it), it picks: the
    /// ways open (`func_003AEA`), its routine (`func_003B94`) when
    /// `OBJ_TIMER` runs out (a change of direction adds `OBJ_TIMER_ADD`
    /// to it), a way open (`func_003B28`); then it moves
    /// (`func_0039C0`).  Gives back d0, d1, d4 and a4 (`func_003B94`
    /// sets them to the object being run).
    pub fn r_00398c(&mut self, d0: u32, d1: u32, d4: u32, obj: u32, a5: u32) -> (u32, u32, u32, u32) {
        if self.read8(STOPPED) != 0 {
            return (d0, d1, d4, obj);
        }
        let (d0, _) = self.r_0015e6(obj);
        let (d0, z, _) = self.r_00182e(d0, obj);
        let (mut d0, mut on_middle) = (d0, z || d0 as u8 == 9);
        if on_middle {
            let (d0b, z, _) = self.r_001840(d0, obj);
            d0 = d0b;
            on_middle = z || d0 as u8 == 9;
        }
        if !on_middle {
            let anim = self.read8(obj + OBJ_ANIM_DIRECTION);
            self.write8(obj + OBJ_ANIM_DIRECTION, anim & 0x7F);
            let (d0, d1) = self.r_0039c0(d0, d1, obj);
            return (d0, d1, d4, obj);
        }
        let anim = self.read8(obj + OBJ_ANIM_DIRECTION);
        if anim & 0x80 != 0 {
            let (d0, d1) = self.r_0039c0(low_byte(d0, anim), d1, obj);
            return (d0, d1, d4, obj);
        }
        self.write8(obj + OBJ_ANIM_DIRECTION, anim | 0x80);
        self.r_003aea(obj);
        let value = self.read8(obj + OBJ_DIRECTION) & 3;
        self.write8(LAST_DIRECTION, value);
        let (mut d1, mut d4, mut obj) = (d1, d4, obj);
        let timer = self.read8(obj + OBJ_TIMER);
        let mut pick = timer & 0xF == 0;
        if !pick {
            let timer = timer.wrapping_sub(1);
            self.write8(obj + OBJ_TIMER, timer);
            pick = timer & 0xF == 0;
        }
        if pick {
            self.r_003e44();
            let (d0, _) = self.r_0015e6(obj);
            let picked;
            (picked, d1, d4, obj) = self.r_003b94(d0, d1, d4, obj);
            if picked as u8 != self.read8(LAST_DIRECTION) {
                self.write8(LAST_DIRECTION, picked as u8);
                let add = self.read8(obj + OBJ_TIMER_ADD);
                let timer = self.read8(obj + OBJ_TIMER);
                self.write8(obj + OBJ_TIMER, timer | add);
            }
        }
        let (d0, none) = self.r_003b28(d1, obj, a5);
        self.write8(obj + OBJ_DIRECTION, d0 as u8);
        if !none {
            self.r_006fa0(obj);
        }
        let (d0, _) = self.r_0015e6(obj);
        let (d0, d1) = self.r_0039c0(d0, d1, obj);
        (d0, d1, d4, obj)
    }

    /// `func_0039C0`: the object moves on in its direction (none: bit
    /// 7 of `OBJ_ANIM_DIRECTION` cleared), turning back where the cell
    /// it goes into is closed to it (`func_00173E`, `func_003AC4`).
    /// Gives back d0 and d1.
    pub fn r_0039c0(&mut self, d0: u32, d1: u32, obj: u32) -> (u32, u32) {
        let direction = self.read8(obj + OBJ_DIRECTION);
        let mut d0 = low_byte(d0, direction);
        let mut d1 = d1;
        if direction & 0x80 != 0 {
            let anim = self.read8(obj + OBJ_ANIM_DIRECTION);
            self.write8(obj + OBJ_ANIM_DIRECTION, anim & 0x7F);
            return (d0, d1);
        }
        self.r_00173e(d0, obj);
        let cell = self.read32(CELL_POINTER);
        d0 = low_byte(d0, self.read8(cell));
        if self.r_003ac4(d0, obj) {
            let direction = self.read8(obj + OBJ_DIRECTION);
            d1 = low_word(d1, direction as i8 as i16 as u16);
            let back = self.read8(REVERSE.wrapping_add(direction as i8 as i32 as u32));
            self.write8(obj + OBJ_DIRECTION, back);
            let anim = self.read8(obj + OBJ_ANIM_DIRECTION);
            self.write8(obj + OBJ_ANIM_DIRECTION, anim & 0x7F);
        }
        let (d0, d1, _) = self.r_003a04(d0, d1, obj);
        (d0, d1)
    }

    /// `func_003A04`: the object's velocity from `SPEED` by its
    /// direction (negated for 0 and 3; `SPEED` keeps it), the other
    /// axis's cleared (`func_001858`, `func_00186A`), and the move
    /// (`func_0019DA` up or down, `func_0019C0` across).  Gives back d0,
    /// d1 and X.
    pub fn r_003a04(&mut self, d0: u32, d1: u32, obj: u32) -> (u32, u32, bool) {
        let direction = self.read8(obj + OBJ_DIRECTION);
        let d0 = low_byte(d0, direction >> 2);
        let backward = direction & 2 != 0;
        if direction & 1 == 0 {
            // 0 and 2: up and down
            if !backward {
                self.negate_speed();
            }
            let value = self.read16(SPEED);
            self.write16(VELOCITY_Y, value);
            self.r_001858(obj);
            self.r_0019da(d0, d1, obj)
        } else {
            if backward {
                self.negate_speed();
            }
            self.r_00186a(obj);
            let value = self.read16(SPEED);
            self.write16(VELOCITY_X, value);
            self.r_0019c0(d0, d1, obj)
        }
    }

    fn negate_speed(&mut self) {
        let speed = self.read16(SPEED);
        self.write16(SPEED, speed.wrapping_neg());
    }

    /// `func_003AC4`: whether the cell of kind `d0` (its low byte) is
    /// closed to the object (C): kinds 0 and 1 are, 4 and on aren't, 2
    /// and 3 are unless bit 6 of `OBJ_FLAGS` is set.
    pub fn r_003ac4(&mut self, d0: u32, obj: u32) -> bool {
        match d0 as u8 {
            0 | 1 => true,
            2 | 3 => self.read8(obj + OBJ_FLAGS) & 0x40 == 0,
            _ => false,
        }
    }

    /// `func_003AEA`: `OPEN` for the four directions: 1 where the cell
    /// that way (`func_0016F6`) is closed to the object (`func_003AC4`),
    /// 0 where not.
    pub fn r_003aea(&mut self, obj: u32) {
        for direction in (0..4u32).rev() {
            self.write8(OPEN + direction, 0);
            let (d0, _) = self.r_0015e6(obj);
            let (d0, ..) = self.r_0016f6(low_byte(d0, direction as u8), direction);
            if self.r_003ac4(d0, obj) {
                self.write8(OPEN + direction, 1);
            }
        }
    }

    /// `func_003B28`: the way the object goes: from `LAST_DIRECTION`,
    /// turning one way or the other (a random number's sign,
    /// `func_001BE2`), the first not marked in `OPEN` (the way back is
    /// marked first, unless the way ahead already is); 0xFF where all
    /// are.  Gives back d0 (its low byte the way) and N (none).
    pub fn r_003b28(&mut self, _d1: u32, obj: u32, _a5: u32) -> (u32, bool) {
        let ahead = self.read8(obj + OBJ_DIRECTION) & 3;
        if self.read8(OPEN + u32::from(ahead)) == 0 {
            let back = self.read8(REVERSE + u32::from(ahead));
            self.write8(OPEN.wrapping_add(back as i8 as i32 as u32), 1);
        }
        let mut way = self.read8(LAST_DIRECTION) & 3;
        let (random, ..) = self.r_001be2(u32::from(way));
        let turn: u8 = if random & 0x80 != 0 { 0xFF } else { 1 };
        let mut tries = 4;
        while self.read8(OPEN + u32::from(way)) != 0 {
            way = way.wrapping_add(turn) & 3;
            tries -= 1;
            if tries == 0 {
                way = 0xFF;
                break;
            }
        }
        (low_byte(random, way), way & 0x80 != 0)
    }

    /// `func_003B94`: the routine of the object's kind (`BY_KIND` by
    /// bits 0-3 of `OBJ_KIND`), run for the object being run
    /// (`CURRENT`), picks a direction.  Gives back d0 (its low byte the
    /// direction), d1, d4 and a4.
    pub fn r_003b94(&mut self, d0: u32, d1: u32, d4: u32, obj: u32) -> (u32, u32, u32, u32) {
        let offset = (self.read8(obj + OBJ_KIND) & 0xF) * 4;
        let d0 = low_word(d0, u16::from(offset));
        let routine = self.read32(BY_KIND + u32::from(offset));
        let d4 = low_byte(d4, self.read8(CURRENT_INDEX));
        let obj = self.read32(CURRENT);
        let mut regs = Regs { d0, d1, d4, a0: routine, a4: obj, ..Regs::default() };
        self.call_family_11(routine, &mut regs);
        if regs.unwound {
            return (regs.d0, regs.d1, d4, obj);
        }
        self.r_0015e6(obj);
        (regs.d0, regs.d1, d4, obj)
    }

    /// The tail of several routines of `func_003B94`: `OBJ_TIMER` to 1
    /// (its start kept).
    fn timer_to_1(&mut self, regs: &mut Regs) {
        let obj = regs.a4;
        let timer = (self.read8(obj + OBJ_TIMER) & 0xF0) | 1;
        self.write8(obj + OBJ_TIMER, timer);
        regs.d0 = low_byte(regs.d0, timer);
    }

    /// `func_003BFE` (kind 0): toward object 0 (`func_003F1A`); on its
    /// row or column, or within 4 cells of it, for the next 4 turns too
    /// (`func_003C48`).
    pub fn r_003bfe(&mut self, regs: &mut Regs) {
        if !self.r_003c48(regs.a4) {
            let c;
            (regs.d0, c) = self.r_003e5c(regs.d0);
            let hold = c || {
                regs.d0 = low_byte(regs.d0, 4);
                let (d1, _, c) = self.r_003e90(regs.d0, regs.d1);
                regs.d1 = d1;
                c
            };
            if hold {
                self.write8(regs.a4 + OBJ_HOLD, 4);
            }
        }
        self.r_003f1a(regs);
    }

    /// `func_003C2A` (kind 1): toward object 0 (`func_003F1A`); on its
    /// row or column, for the next 4 turns too (`func_003C48`).
    pub fn r_003c2a(&mut self, regs: &mut Regs) {
        if !self.r_003c48(regs.a4) {
            let c;
            (regs.d0, c) = self.r_003e5c(regs.d0);
            if c {
                self.write8(regs.a4 + OBJ_HOLD, 4);
            }
        }
        self.r_003f1a(regs);
    }

    /// `func_003C48`: while `OBJ_HOLD` counts down, C and bit 0 of
    /// `OBJ_FLAGS` set; once it's 0, both clear.
    pub fn r_003c48(&mut self, obj: u32) -> bool {
        let hold = self.read8(obj + OBJ_HOLD);
        let flags = self.read8(obj + OBJ_FLAGS);
        if hold != 0 {
            self.write8(obj + OBJ_HOLD, hold - 1);
            self.write8(obj + OBJ_FLAGS, flags | 1);
            true
        } else {
            self.write8(obj + OBJ_FLAGS, flags & 0xFE);
            false
        }
    }

    /// `func_003C6C` (kind 4), `func_003CA2` (kind 5): within 4 cells of
    /// object 0 (`func_003E90`), for 4 (or 2) turns, toward a cell near
    /// it (`func_003F70`); else by `OBJ_HOME` (`func_003FAA`).
    pub fn r_003c6c(&mut self, regs: &mut Regs) {
        self.near_then_step(regs, 4);
    }

    pub fn r_003ca2(&mut self, regs: &mut Regs) {
        self.near_then_step(regs, 2);
    }

    fn near_then_step(&mut self, regs: &mut Regs, turns: u8) {
        if !self.r_003d3c(regs.a4) {
            if !self.near(regs, 4) {
                (regs.d0, regs.d1) = self.r_003faa(regs.d0, regs.d1, regs.a4);
                return;
            }
            self.write8(regs.a4 + OBJ_HOLD, turns);
        }
        self.timer_to_1(regs);
        self.step_toward(regs);
    }

    /// 0x3F70 with a0 as it leaves it (it points a0 at `STEP2_X`).
    fn step_toward(&mut self, regs: &mut Regs) {
        (regs.d0, regs.d1) = self.r_003f70(regs.d0, regs.d1, regs.a4);
        regs.a0 = STEP2_X;
    }

    /// `func_004088`, a0 kept (it doesn't change it; its translation
    /// gives back 0).
    fn random_way(&mut self, regs: &mut Regs) {
        let a0 = regs.a0;
        self.r_004088(regs);
        regs.a0 = a0;
    }

    /// `func_003E90` with d0 = `cells`, d0 and d1 kept in `regs`.
    fn near(&mut self, regs: &mut Regs, cells: u8) -> bool {
        regs.d0 = low_byte(regs.d0, cells);
        let (d1, _, c) = self.r_003e90(regs.d0, regs.d1);
        regs.d1 = d1;
        c
    }

    /// `func_003CD8` (kind 6): in object 0's column, for 4 turns,
    /// toward a cell near it (`func_003F70`); else by `OBJ_HOME`'s
    /// column (`func_00400C`).
    pub fn r_003cd8(&mut self, regs: &mut Regs) {
        if !self.r_003d3c(regs.a4) {
            if !self.r_003e80() {
                regs.d0 = self.by_home_column(regs.d0, regs.d1, regs.a4);
                return;
            }
            self.write8(regs.a4 + OBJ_HOLD, 4);
        }
        self.timer_to_1(regs);
        self.step_toward(regs);
    }

    /// `func_003D0A` (kind 7): in object 0's row, for 4 turns, toward a
    /// cell near it (`func_003F70`); else by `OBJ_HOME`'s row
    /// (`func_004024`).
    pub fn r_003d0a(&mut self, regs: &mut Regs) {
        if !self.r_003d3c(regs.a4) {
            let c;
            (regs.d0, c) = self.r_003e68(regs.d0);
            if !c {
                regs.d0 = self.by_home_row(regs.d0, regs.d1, regs.a4);
                return;
            }
            self.write8(regs.a4 + OBJ_HOLD, 4);
        }
        self.timer_to_1(regs);
        self.step_toward(regs);
    }

    /// 0x400C: in `OBJ_HOME`'s column, a random direction, 1 or 3;
    /// else `func_003FC0`.
    fn by_home_column(&mut self, d0: u32, d1: u32, obj: u32) -> u32 {
        if self.read8(CELL_X) != self.read8(obj + OBJ_HOME) {
            return self.r_003fc0(d0, obj);
        }
        let (random, ..) = self.r_001be2(d1);
        low_byte(random, (random as u8 & 3) | 1)
    }

    /// 0x4024: in `OBJ_HOME`'s row, a random direction, 1 or 3; else
    /// `func_003FE8`.
    fn by_home_row(&mut self, d0: u32, d1: u32, obj: u32) -> u32 {
        if self.read8(CELL_Y) != self.read8(obj + OBJ_HOME_Y) {
            return self.r_003fe8(d0, obj);
        }
        let (random, ..) = self.r_001be2(d1);
        low_byte(random, (random as u8 & 3) | 1)
    }

    /// `func_003D3C`: while `OBJ_HOLD` counts down, C set.
    pub fn r_003d3c(&mut self, obj: u32) -> bool {
        let hold = self.read8(obj + OBJ_HOLD);
        if hold != 0 {
            self.write8(obj + OBJ_HOLD, hold - 1);
        }
        hold != 0
    }

    /// `func_003D54` (kind 8): within 4 cells of object 0, by
    /// `OBJ_HOME` (`func_003FAA`); else toward it (`func_003F1A`).
    pub fn r_003d54(&mut self, regs: &mut Regs) {
        if self.near(regs, 4) {
            (regs.d0, regs.d1) = self.r_003faa(regs.d0, regs.d1, regs.a4);
        } else {
            self.r_003f1a(regs);
        }
    }

    /// `func_003D6A` (kind 9): within 4 cells of object 0, for 4 turns,
    /// the way `AHEAD` gives for its direction; else toward it
    /// (`func_003F1A`).
    pub fn r_003d6a(&mut self, regs: &mut Regs) {
        if !self.r_003d3c(regs.a4) {
            if !self.near(regs, 4) {
                self.r_003f1a(regs);
                return;
            }
            self.write8(regs.a4 + OBJ_HOLD, 4);
        }
        self.timer_to_1(regs);
        let direction = self.read8(OBJECT_0_DIRECTION) as i8 as i16;
        let way = self.read8(AHEAD.wrapping_add(direction as i32 as u32));
        regs.d0 = low_byte(low_word(regs.d0, direction as u16), way);
    }

    /// `func_003D9E` (kind 10): in object 0's row, for 8 turns, toward
    /// it (`func_003F1A`); else by `OBJ_HOME` (`func_003FAA`).
    pub fn r_003d9e(&mut self, regs: &mut Regs) {
        if !self.r_003d3c(regs.a4) {
            let c;
            (regs.d0, c) = self.r_003e68(regs.d0);
            if !c {
                (regs.d0, regs.d1) = self.r_003faa(regs.d0, regs.d1, regs.a4);
                return;
            }
            self.write8(regs.a4 + OBJ_HOLD, 8);
        }
        self.timer_to_1(regs);
        self.r_003f1a(regs);
    }

    /// `func_003DCE` (kind 11): in object 0's column, for 8 turns,
    /// toward it (`func_003F1A`); else by `OBJ_HOME` (`func_003FAA`).
    pub fn r_003dce(&mut self, regs: &mut Regs) {
        if !self.r_003d3c(regs.a4) {
            if !self.r_003e80() {
                (regs.d0, regs.d1) = self.r_003faa(regs.d0, regs.d1, regs.a4);
                return;
            }
            self.write8(regs.a4 + OBJ_HOLD, 8);
        }
        self.timer_to_1(regs);
        self.r_003f1a(regs);
    }

    /// `func_003DFE` (kind 13): within 4 cells of object 0 and on its
    /// row or column, for 5 turns, a direction by object 0's
    /// (`func_004050`); else a random one (`func_004088`).
    pub fn r_003dfe(&mut self, regs: &mut Regs) {
        let obj = regs.a4;
        let hold = self.read8(obj + OBJ_HOLD);
        regs.d0 = low_byte(regs.d0, hold);
        if hold != 0 {
            self.write8(obj + OBJ_HOLD, hold - 1);
        } else {
            if !self.near(regs, 4) {
                self.random_way(regs);
                return;
            }
            let c;
            (regs.d0, c) = self.r_003e5c(regs.d0);
            if !c {
                self.random_way(regs);
                return;
            }
            self.write8(obj + OBJ_HOLD, 5);
        }
        self.timer_to_1(regs);
        regs.d0 = low_byte(regs.d0, self.read8(OBJECT_0_DIRECTION));
        // a0 kept, as in `random_way`
        let a0 = regs.a0;
        self.r_004050(regs);
        regs.a0 = a0;
    }

    /// `func_003E44`: object 0's cell (its position / 16) to `CELL` and
    /// `TARGET`.  Gives back d0 (the position / 16, its words swapped).
    pub fn r_003e44(&mut self) -> u32 {
        let d0 = self.read32(OBJECT_0_POSITION) >> 4;
        self.write8(CELL_Y, d0 as u8);
        let d0 = d0.rotate_left(16);
        self.write8(CELL_X, d0 as u8);
        let value = self.read16(CELL);
        self.write16(TARGET, value);
        d0
    }

    /// `func_003E5C`: whether `CELL` is in `TARGET`'s column or row (C).
    /// Gives back d0 (its low byte the last read).
    pub fn r_003e5c(&mut self, d0: u32) -> (u32, bool) {
        let x = self.read8(CELL_X);
        if x == self.read8(TARGET_X) {
            return (low_byte(d0, x), true);
        }
        self.r_003e68(d0)
    }

    /// `func_003E68`: whether `CELL` is in `TARGET`'s row (C).  Gives
    /// back d0 (its low byte the row).
    pub fn r_003e68(&mut self, d0: u32) -> (u32, bool) {
        let y = self.read8(CELL_Y);
        (low_byte(d0, y), y == self.read8(TARGET_Y))
    }

    /// `func_003E7A`: C set.
    pub fn r_003e7a(&mut self) -> bool {
        true
    }

    /// `func_003E80`: whether `CELL` is in `TARGET`'s column (C).
    pub fn r_003e80(&mut self) -> bool {
        self.read8(CELL_X) == self.read8(TARGET_X)
    }

    /// `func_003E90`: whether `CELL` is less than `d0` (its low byte)
    /// cells from `TARGET` both ways (C).  Gives back d1 (its low byte
    /// the last distance) and N (of the last compare).
    pub fn r_003e90(&mut self, d0: u32, d1: u32) -> (u32, bool, bool) {
        let cells = d0 as u8;
        let dx = distance(self.read8(CELL_X).wrapping_sub(self.read8(TARGET_X)));
        let n = dx.wrapping_sub(cells) & 0x80 != 0;
        if dx >= cells {
            return (low_byte(d1, dx), n, false);
        }
        let dy = distance(self.read8(CELL_Y).wrapping_sub(self.read8(TARGET_Y)));
        let n = dy.wrapping_sub(cells) & 0x80 != 0;
        (low_byte(d1, dy), n, dy < cells)
    }

    /// `func_003EC4`: whether a cell of kind 1 is within 3 steps of
    /// `CELL` in the object's direction (`func_00167A`'s cell); `CELL`
    /// kept.  What it finds isn't given back.  (No script calls it; and
    /// the bytes after its table, `move.b #1,d1; bra.s`, are another
    /// entry, for 2 steps, nothing calls.)
    pub fn r_003ec4(&mut self, _d1: u32, obj: u32) {
        let cell = self.read16(CELL);
        let direction = u32::from(self.read8(obj + OBJ_DIRECTION) & 3);
        let step_x = self.read8(STEP_X + direction);
        let step_y = self.read8(STEP_Y + direction);
        for _ in 0..3 {
            let value = self.read8(CELL_X).wrapping_add(step_x);
            self.write8(CELL_X, value);
            let value = self.read8(CELL_Y).wrapping_add(step_y);
            self.write8(CELL_Y, value);
            let (at, _) = self.r_00167a();
            if self.read8(at) == 1 {
                break;
            }
        }
        self.write16(CELL, cell);
    }

    /// `func_003F1A` (kinds 2 and 3): the direction toward `TARGET`
    /// along the axis it is farther on (up or down where the distances
    /// are equal); the differences to `DIFF_X`, `DIFF_Y`.  Gives back d0
    /// (its low byte the direction) and d1 (its low byte |dy| - |dx|).
    pub fn r_003f1a(&mut self, regs: &mut Regs) {
        (regs.d0, regs.d1) = self.toward_target(regs.d0, regs.d1);
    }

    fn toward_target(&mut self, d0: u32, d1: u32) -> (u32, u32) {
        let diff_x = self.read8(CELL_X).wrapping_sub(self.read8(TARGET_X));
        self.write8(DIFF_X, diff_x);
        let dx = distance(diff_x);
        let diff_y = self.read8(CELL_Y).wrapping_sub(self.read8(TARGET_Y));
        self.write8(DIFF_Y, diff_y);
        let dy = distance(diff_y);
        let direction = if dy < dx {
            if self.read8(DIFF_X) & 0x80 != 0 { 1 } else { 3 }
        } else if self.read8(DIFF_Y) & 0x80 != 0 {
            2
        } else {
            0
        };
        (low_byte(d0, direction), low_byte(d1, dy.wrapping_sub(dx)))
    }

    /// 0x3F70: `CELL` moved by `STEP2_X` of object 0's direction and by
    /// `STEP2_Y` of that step (the step taken for an index, d0's low
    /// word, its high byte what d0 had); then, unless it is `TARGET`,
    /// the direction toward `TARGET` (`func_003F1A`).  Gives back d0
    /// and d1.
    pub fn r_003f70(&mut self, d0: u32, d1: u32, _obj: u32) -> (u32, u32) {
        let direction = self.read8(OBJECT_0_DIRECTION) as i8 as i16;
        let d1 = low_word(d1, direction as u16);
        let step_x = self.read8(STEP2_X.wrapping_add(direction as i32 as u32));
        let value = self.read8(CELL_X).wrapping_add(step_x);
        self.write8(CELL_X, value);
        let index = low_byte(d0, step_x) as u16 as i16;
        let step_y = self.read8(STEP2_Y.wrapping_add(index as i32 as u32));
        let value = self.read8(CELL_Y).wrapping_add(step_y);
        self.write8(CELL_Y, value);
        let cell = self.read16(CELL);
        let d0 = low_word(d0, cell);
        if cell == self.read16(TARGET) {
            return (d0, d1);
        }
        self.toward_target(d0, d1)
    }

    /// 0x3FAA: on `OBJ_HOME`, a random direction; else the way back
    /// (`func_003FC0`).  Gives back d0 and d1.
    pub fn r_003faa(&mut self, d0: u32, d1: u32, obj: u32) -> (u32, u32) {
        let cell = self.read16(CELL);
        if cell != self.read16(obj + OBJ_HOME) {
            return (self.r_003fc0(low_word(d0, cell), obj), d1);
        }
        let (random, d1, _) = self.r_001be2(d1);
        (low_byte(random, random as u8 & 3), d1)
    }

    /// 0x3FC0, 0x3FE8: the object's way back (its direction turned
    /// around).  They compare `CELL` with `OBJ_HOME` and the distance
    /// with 3 to keep the direction where it is near, but the `andi`
    /// after the compare clears C: they always turn back.  Gives back d0
    /// (its low byte the direction).
    pub fn r_003fc0(&mut self, d0: u32, obj: u32) -> u32 {
        self.r_003fe8(d0, obj)
    }

    pub fn r_003fe8(&mut self, d0: u32, obj: u32) -> u32 {
        low_byte(d0, (self.read8(obj + OBJ_DIRECTION) & 3) ^ 2)
    }
}
