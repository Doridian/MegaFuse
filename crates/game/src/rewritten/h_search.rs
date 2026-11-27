//! A request (pad bits) chosen for an object by looking at the cells
//! around it (0x87F6-0x8F71): a grid of the map's cells, a byte each
//! (`GRID`: the high nibble a level, the low nibble a kind), searched
//! from the object's cell in the four ways, as deep as 5 cells; and
//! the grid's levels spread out from objects along the four ways
//! (`func_008EEE`, `func_008F72`).

use crate::Game;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// `base` + the low word of `index` (signed).
fn at(base: u32, index: u32) -> u32 {
    base.wrapping_add(index as u16 as i16 as u32)
}

/// The grid: a byte a cell, by `y * 16 + x`: the high nibble a level,
/// the low nibble a kind (1, 2, 4-6, 7 by how the code treats them).
const GRID: u32 = 0xFFFF_9C9C;
const LEVEL: u8 = 0xF0;
const KIND: u8 = 0x0F;
/// The map's cells (`a_cells.rs`'s `CELL_MAP`): the kinds
/// `func_008F72` spreads levels through.
const CELL_MAP: u32 = 0xFFFF_989C;
/// A grid `func_0066BE` gives the index into.
const GRID_AC00: u32 = 0xFFFF_AC00;
/// The four ways' steps in the grid (signed bytes in the ROM: -16,
/// +1, +16, -1), the way back from each (bytes: 2, 3, 0, 1), and
/// `func_008F72`'s copy of the steps.
const STEPS: u32 = 0x008DDA;
const BACK: u32 = 0x008DDE;
const STEPS_2: u32 = 0x009052;
const WAYS: u8 = 4;

/// The object the request is for (an index, `func_001CFA`'s) and the
/// one being run (index and address: `a_objects.rs`).
const WALKER: u32 = 0xFFFF_A93F;
const CURRENT: u32 = 0xFFFF_A42C;
const CURRENT_ADDR: u32 = 0xFFFF_A42E;
/// The request chosen (bits 0-3 the ways: `b_walk.rs`) and a second
/// byte set with it (0x10, 0x20 here).
const REQUEST: u32 = 0xFFFF_A932;
const REQUEST_2: u32 = 0xFFFF_A933;
/// A byte compared with the nearest object's distance (`b_control.rs`).
const ARG_A934: u32 = 0xFFFF_A934;
/// The current object's byte 0x13, copied (`b_walk.rs`'s facing before
/// it moved).
const OLD_FACING: u32 = 0xFFFF_A93B;
/// The cell `func_0015E6` gives (x, y), saved and restored by
/// `func_0087F6`.
const CELL_X: u32 = 0xFFFF_9F08;
const CELL_Y: u32 = 0xFFFF_9F09;
/// Flags of the game: bit 1 objects whose byte 0x2D's bit 6 are the
/// same are passed over in the search for the nearest.
const MODE: u32 = 0xFFFF_9540;
const MODE_SIDES: u8 = 1 << 1;
/// How many objects there are (an index stops at it).
const OBJECT_COUNT: u32 = 0xFFFF_9541;
/// The frame count: an object searches on frames whose parity is its
/// index's.
const FRAME: u32 = 0xFFFF_80D8;
/// Bit 0 set by the first search (only one while it stays set).
const SEARCHED: u32 = 0xFFFF_806A;

/// An object's bytes (`func_001CFA`'s records).
const OBJ_13: u32 = 0x13;
const OBJ_X: u32 = 0x19;
const OBJ_Y: u32 = 0x1B;
const OBJ_2C: u32 = 0x2C;
/// Bit 5: passed over by the search for the nearest; bit 6 its side.
const OBJ_FLAGS: u32 = 0x2D;
const OBJ_2F: u32 = 0x2F;
/// Where it was at its last search, and frames to keep the request
/// it chose (`OBJ_KEPT`) while within 8 pixels of it.
const OBJ_SEARCH_X: u32 = 0x33;
const OBJ_SEARCH_Y: u32 = 0x34;
const OBJ_KEEP: u32 = 0x35;
const OBJ_KEPT: u32 = 0x39;

/// Two bytes by the object's byte 0x2C (x 2, a long read: `LIMITS`),
/// `LIMIT` a level (a way's first cell below it is closed) and
/// `LIMIT_2` (compared with the object's byte 0x2F).
const LIMITS: u32 = 0x0088E8;
const LIMIT: u32 = 0xFFFF_8072;
const LIMIT_2: u32 = 0xFFFF_8073;
const LIMIT_COPY: u32 = 0xFFFF_80B4;
/// The object's cell (a word, `y * 16 + x`) and its level.
const HERE: u32 = 0xFFFF_8076;
const HERE_LEVEL: u32 = 0xFFFF_8098;
/// By way (4 bytes each): the highest level found and its depth, the
/// lowest and its depth, the first cell's level, whether it is open
/// (a long of 0xFF bytes to begin with, cleared by way).
const HIGHEST: u32 = 0xFFFF_8078;
const HIGHEST_DEPTH: u32 = 0xFFFF_807C;
const LOWEST: u32 = 0xFFFF_8080;
const LOWEST_DEPTH: u32 = 0xFFFF_8084;
const FIRST: u32 = 0xFFFF_8088;
const OPEN: u32 = 0xFFFF_808C;
/// The ways in order (4 words), best first.
const ORDER: u32 = 0xFFFF_8090;
/// The request for each way (4 bytes, from `DEFAULT_REQUESTS`: up,
/// right, down, left).
const REQUESTS: u32 = 0xFFFF_809A;
const REQUEST_UP: u32 = REQUESTS;
const REQUEST_RIGHT: u32 = REQUESTS + 1;
const REQUEST_DOWN: u32 = REQUESTS + 2;
const REQUEST_LEFT: u32 = REQUESTS + 3;
const DEFAULT_REQUESTS: u32 = 0x0088E4;
/// Ties counted by the sort of the ways.
const TIES: u32 = 0xFFFF_80AA;
/// The search's depth, and the best and the worst level met in the
/// way being searched with their depths.
const DEPTH: u32 = 0xFFFF_80AC;
const MAX_DEPTH: u8 = 5;
const HIGH: u32 = 0xFFFF_80B7;
const HIGH_DEPTH: u32 = 0xFFFF_80B8;
const LOW: u32 = 0xFFFF_80B9;
const LOW_DEPTH: u32 = 0xFFFF_80BA;
/// A cell of kind 2 or 5 found: its way (a word, -1 none), depth and
/// kind.
const FOUND_WAY: u32 = 0xFFFF_80AE;
const FOUND_DEPTH: u32 = 0xFFFF_80B0;
const FOUND_KIND: u32 = 0xFFFF_80B1;
/// The nearest other object: its distance (x + y in cells), index,
/// cell and distances by x and y; then the scratch for each object
/// looked at, and the flags of the object searching.
const NEAREST: u32 = 0xFFFF_809E;
const NEAREST_X: u32 = 0xFFFF_809F;
const NEAREST_Y: u32 = 0xFFFF_80A0;
const NEAREST_DX: u32 = 0xFFFF_80A1;
const NEAREST_DY: u32 = 0xFFFF_80A2;
const OWN_FLAGS: u32 = 0xFFFF_80A3;
const SEEN_DX: u32 = 0xFFFF_806C;
const SEEN_INDEX: u32 = 0xFFFF_806D;
const SEEN_Y: u32 = 0xFFFF_806E;
const SEEN_X: u32 = 0xFFFF_806F;
const SEEN_DY: u32 = 0xFFFF_8070;
const SEEN_DX_2: u32 = 0xFFFF_8071;

/// `func_008EEE`'s place in each object's list (4 bytes; 0xFF done)
/// and the lists: 0x40 bytes an object from `LISTS`, 8 an entry (a
/// byte, 0 the end; a long at 2, the object spreading).
const PLACES: u32 = 0xFFFF_80BE;
const LISTS: u32 = 0xFFFF_AAC0;
const ENTRY_OBJECT: u32 = 2;
const ENTRY_SIZE: u8 = 8;
const LIST_SIZE: u8 = 0x40;
/// A spreading object's bytes: its level (high nibble), its reach,
/// its cell (x, y), and its flags (bit 5: it doesn't spread).
const SPREAD_LEVEL: u32 = 0x16;
const SPREAD_REACH: u32 = 0x17;
const SPREAD_FLAGS: u32 = 0x18;
const SPREAD_X: u32 = 0x1A;
const SPREAD_Y: u32 = 0x1B;
/// `func_008F72`'s reach for the way, and the way (3 to 0).
const REACH: u32 = 0xFFFF_80BC;
const SPREAD_WAY: u32 = 0xFFFF_AABD;

impl Game {
    /// `func_0087F6`: the request (`REQUEST`) for the object `WALKER`,
    /// `REQUEST_2` cleared.  While its `OBJ_KEEP` isn't 0 and it is
    /// within 8 pixels (x + y) of where it last searched, it is counted
    /// down and the request is `OBJ_KEPT`; else on frames of its
    /// parity, the first of them (`SEARCHED`), a search from its cell
    /// (`func_00890C`) with `LIMIT` and `LIMIT_2` by its byte 0x2C, the
    /// request kept in `OBJ_KEPT`; else `OBJ_KEPT`.  `CELL_X`, `CELL_Y`
    /// are kept.  Gives back d4 (`WALKER` in its low byte), d7 (as the
    /// search leaves it) and X (the carry of x + y, if that was added
    /// and nothing searched; else clear).
    pub fn r_0087f6(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d7: u32) -> (u32, u32, bool) {
        let cell = self.read16(CELL_X);
        self.write8(REQUEST, 0);
        self.write8(REQUEST_2, 0);
        let current = self.read32(CURRENT_ADDR);
        let facing = self.read8(current + OBJ_13);
        self.write8(OLD_FACING, facing);
        let walker = self.read8(WALKER);
        let (mut d4, obj, mut x) = self.r_001cfa(low_byte(d4, walker));
        let (mut d0, mut d1, mut d7) = (d0, d1, d7);
        let keep = self.read8(obj + OBJ_KEEP);
        let mut kept = false;
        if keep != 0 {
            let dx = self.read8(obj + OBJ_SEARCH_X).abs_diff(self.read8(obj + OBJ_X));
            let dy = self.read8(obj + OBJ_SEARCH_Y).abs_diff(self.read8(obj + OBJ_Y));
            d1 = low_byte(d1, dy);
            let (sum, carry) = dx.overflowing_add(dy);
            d0 = low_byte(d0, sum);
            x = carry;
            if sum < 8 {
                // (`keep` isn't 0: no borrow)
                self.write8(obj + OBJ_KEEP, keep - 1);
                x = false;
                kept = true;
            }
        }
        if !kept {
            let frame = self.read8(FRAME);
            d1 = low_byte(d1, frame);
            d0 = low_byte(d0, d4 as u8 ^ frame);
            if d0 & 1 != 0 {
                let searched = self.read8(SEARCHED);
                self.write8(SEARCHED, searched | 1);
                if searched & 1 == 0 {
                    (d4, d7) = self.search(d0, d1, d2, d4, d7, obj);
                    self.write16(CELL_X, cell);
                    return (d4, d7, false);
                }
            }
        }
        let request = self.read8(obj + OBJ_KEPT);
        self.write8(REQUEST, request);
        self.write16(CELL_X, cell);
        (d4, d7, x)
    }

    /// `func_0087F6`'s search, for the object at `obj`.
    fn search(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d7: u32, obj: u32) -> (u32, u32) {
        self.write8(obj + OBJ_KEEP, 8);
        let x = self.read8(obj + OBJ_X);
        self.write8(obj + OBJ_SEARCH_X, x);
        let y = self.read8(obj + OBJ_Y);
        self.write8(obj + OBJ_SEARCH_Y, y);
        let walker = self.read8(WALKER);
        let (_, obj, _) = self.r_001cfa(low_byte(d4, walker));
        let twice = self.read8(obj + OBJ_2C).wrapping_mul(2);
        let limits = self.read32(at(LIMITS, u32::from(twice as i8 as u16)));
        self.write32(LIMIT, limits);
        let requests = self.read32(DEFAULT_REQUESTS);
        self.write32(REQUESTS, requests);
        let d4 = low_byte(d4, self.read8(CURRENT));
        let current = self.read32(CURRENT_ADDR);
        let x = self.read8(current + OBJ_X) >> 4;
        self.write16(HERE, u16::from(x));
        let y = self.read8(current + OBJ_Y) & 0xF0;
        let here = self.read16(HERE);
        self.write16(HERE, here | u16::from(y));
        let d0 = low_word(d0, u16::from(y));
        let (d4, d7) = self.r_00890c(d0, d1, d2, d4, d7);
        let walker = self.read8(WALKER);
        let (d4, obj, _) = self.r_001cfa(low_byte(d4, walker));
        let request = self.read8(REQUEST);
        self.write8(obj + OBJ_KEPT, request);
        (d4, d7)
    }

    /// `func_00890C`: the search from `HERE`: `FOUND_DEPTH` 8, no way
    /// found, every way open; `HERE_LEVEL` its cell's level (0: bit 4
    /// of `REQUEST_2` set); the ways searched (`func_008C6A`) and
    /// ordered (`func_008B62`).  At a level of 0xF0, the closed ways'
    /// requests cleared, then a found way's request (a kind 2 found at
    /// depth 1: none, and `REQUEST_2` 0x20), or with none found the
    /// way to the nearest other object (`nearest`).  Else the best
    /// way's request (or, if it is 0 and the level is 0, its default).
    /// Gives back d4 and d7.
    pub fn r_00890c(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d7: u32) -> (u32, u32) {
        self.write8(FOUND_DEPTH, 8);
        self.write16(FOUND_WAY, 0xFFFF);
        self.write8(FOUND_KIND, 0);
        self.write32(OPEN, 0xFFFF_FFFF);
        let here = self.read16(HERE);
        let d1 = low_word(d1, here);
        let level = self.read8(at(GRID, d1)) & LEVEL;
        let d0 = low_byte(d0, level);
        self.write8(HERE_LEVEL, level);
        if level == 0 {
            let r2 = self.read8(REQUEST_2);
            self.write8(REQUEST_2, r2 | 0x10);
        }
        let (d1, d7) = self.r_008c6a(d0, d1, d7);
        let (d2, d4) = self.r_008b62(d1, d2, d4);
        if self.read8(HERE_LEVEL) != 0xF0 {
            if d2 & 0x8000 == 0 {
                self.r_00895e(d2);
            }
            return (d4, d7);
        }
        let open = self.read32(OPEN);
        let requests = self.read32(REQUESTS);
        self.write32(REQUESTS, requests & open);
        let found = self.read16(FOUND_WAY);
        if found & 0x8000 == 0 {
            let request = self.read8(at(REQUESTS, u32::from(found)));
            self.write8(REQUEST, request);
            if self.read8(FOUND_KIND) == 5 || self.read8(FOUND_DEPTH) != 1 {
                return (d4, d7);
            }
            let walker = self.read8(WALKER);
            let (d4, obj, _) = self.r_001cfa(low_byte(d4, walker));
            self.write8(obj + OBJ_KEEP, 0);
            self.write8(REQUEST, 0);
            self.write8(REQUEST_2, 0x20);
            return (d4, d7);
        }
        (self.nearest(d4), d7)
    }

    /// `func_00890C` with no way found: the nearest other object (x + y
    /// in cells; not one of `OBJ_FLAGS` bit 5, nor of the same side
    /// with `MODE_SIDES`), and the request toward it: by x if it is as
    /// far by x as by y, else by y, else the other, else left or
    /// right.  In line with it, if `ARG_A934` isn't below its distance
    /// and the object's byte 0x2F is below `LIMIT_2`, `REQUEST_2` 0x20
    /// and `OBJ_KEEP` 0.  Gives back d4.
    fn nearest(&mut self, d4: u32) -> u32 {
        let walker = self.read8(WALKER);
        let (_, obj, _) = self.r_001cfa(low_byte(d4, walker));
        self.r_0015e6(obj);
        self.write8(NEAREST, 0xFF);
        let own = self.read8(obj + OBJ_FLAGS);
        self.write8(OWN_FLAGS, own);
        let mut d4 = low_word(d4, 0);
        loop {
            let (_, obj, _) = self.r_001cfa(d4);
            let index = d4 as u8;
            let flags = self.read8(obj + OBJ_FLAGS);
            let skip = index == self.read8(WALKER)
                || flags & (1 << 5) != 0
                || (self.read8(MODE) & MODE_SIDES != 0 && (flags ^ self.read8(OWN_FLAGS)) & 0x40 == 0);
            if !skip {
                let x = self.read8(obj + OBJ_X) >> 4;
                self.write8(SEEN_X, x);
                let dx = x.abs_diff(self.read8(CELL_X));
                self.write8(SEEN_DX, dx);
                self.write8(SEEN_DX_2, dx);
                let y = self.read8(obj + OBJ_Y) >> 4;
                self.write8(SEEN_Y, y);
                let dy = y.abs_diff(self.read8(CELL_Y));
                self.write8(SEEN_DY, dy);
                let distance = dy.wrapping_add(self.read8(SEEN_DX));
                if distance < self.read8(NEAREST) {
                    self.write8(NEAREST, distance);
                    self.write8(SEEN_INDEX, index);
                    for (from, to) in [(SEEN_X, NEAREST_X), (SEEN_Y, NEAREST_Y), (SEEN_DX_2, NEAREST_DX), (SEEN_DY, NEAREST_DY)] {
                        let b = self.read8(from);
                        self.write8(to, b);
                    }
                }
            }
            d4 = low_byte(d4, index.wrapping_add(1));
            if d4 as u8 == self.read8(OBJECT_COUNT) {
                break;
            }
        }
        let by_x = if self.read8(CELL_X) < self.read8(NEAREST_X) { REQUEST_RIGHT } else { REQUEST_LEFT };
        let by_y = if self.read8(CELL_Y) < self.read8(NEAREST_Y) { REQUEST_DOWN } else { REQUEST_UP };
        let mut chosen = None;
        if self.read8(NEAREST_DX) >= self.read8(NEAREST_DY) {
            chosen = self.request_if_any(by_x);
        }
        if chosen.is_none() {
            chosen = self.request_if_any(by_y);
        }
        if chosen.is_none() {
            let request = self.read8(by_x);
            self.write8(REQUEST, request);
        }
        if self.read8(CELL_X) != self.read8(NEAREST_X) && self.read8(CELL_Y) != self.read8(NEAREST_Y) {
            return d4;
        }
        let walker = self.read8(WALKER);
        let (d4, obj, _) = self.r_001cfa(low_byte(d4, walker));
        if self.read8(ARG_A934) >= self.read8(NEAREST) && self.read8(obj + OBJ_2F) < self.read8(LIMIT_2) {
            self.write8(REQUEST_2, 0x20);
            self.write8(obj + OBJ_KEEP, 0);
        }
        d4
    }

    /// The request at `from` to `REQUEST`, if it isn't 0.
    fn request_if_any(&mut self, from: u32) -> Option<()> {
        let request = self.read8(from);
        if request == 0 {
            return None;
        }
        self.write8(REQUEST, request);
        Some(())
    }

    /// `func_00890C`'s end (`.L00895E`): the request of the way `d2`
    /// (its low word), or if it is 0 and `HERE_LEVEL` is 0, the way's
    /// default.
    pub fn r_00895e(&mut self, d2: u32) {
        let mut request = self.read8(at(REQUESTS, d2));
        if request == 0 {
            if self.read8(HERE_LEVEL) != 0 {
                return;
            }
            request = self.read8(at(DEFAULT_REQUESTS, d2));
        }
        self.r_008980(u32::from(request));
    }

    /// `.L008980`: `REQUEST` the low byte of `d0`.
    pub fn r_008980(&mut self, d0: u32) {
        self.write8(REQUEST, d0 as u8);
    }

    /// `.L008984`: an `rts`.
    pub fn r_008984(&mut self) {}

    /// `func_00899E`: `REQUEST` the low byte of `d0`.  No script calls
    /// it (nothing in the ROM does): reviewed by reading.
    pub fn r_00899e(&mut self, d0: u32) {
        self.r_008980(d0);
    }

    /// `func_008986`: the way with the lowest level (`func_008C3A`); if
    /// there is one and its level isn't the low byte of `d0`, its
    /// request (`func_00890C`'s end).  No script calls it (nothing in
    /// the ROM does): reviewed by reading.
    pub fn r_008986(&mut self, d0: u32, d1: u32, d2: u32) {
        let (d1, d2, _) = self.r_008c3a(d1, d2);
        if d2 & 0x8000 != 0 || d1 as u8 == d0 as u8 {
            return;
        }
        self.r_00895e(d2);
    }

    /// `func_008B62`: the ways ordered (`ORDER`): the found way (or,
    /// with none, `OLD_FACING`) first, then sorted by highest level and
    /// least depth of it, the highest first (ties of level counted in
    /// `TIES`).  The best way is given back in d2 unless its first
    /// cell's level is 0xF0, or above `HERE_LEVEL`, or equal to it with
    /// `HERE_LEVEL` below its highest, or else (then the way of the
    /// lowest level, `func_008C3A`, if there is one) its way is open;
    /// else -1.  Gives back d2 (its low word, the way) and d4.
    pub fn r_008b62(&mut self, d1: u32, d2: u32, d4: u32) -> (u32, u32) {
        for way in 0..4 {
            self.write16(ORDER + 2 * way, way as u16);
        }
        let found = self.read16(FOUND_WAY);
        let first = if found & 0x8000 == 0 { found } else { u16::from(self.read8(OLD_FACING)) };
        let to = at(ORDER, u32::from(first.wrapping_add(first)));
        let swapped = self.read16(to);
        let way0 = self.read16(ORDER);
        self.write16(to, way0);
        self.write16(ORDER, swapped);
        self.write8(TIES, 0);
        for i in 0..3 {
            for j in (i + 1)..4 {
                let a = self.read16(ORDER + 2 * i);
                let b = self.read16(ORDER + 2 * j);
                let high_b = self.read8(at(HIGHEST, u32::from(b)));
                let high_a = self.read8(at(HIGHEST, u32::from(a)));
                let swap = if high_b != high_a {
                    high_b > high_a
                } else {
                    let ties = self.read8(TIES);
                    self.write8(TIES, ties.wrapping_add(1));
                    let depth_a = self.read8(at(HIGHEST_DEPTH, u32::from(a)));
                    let depth_b = self.read8(at(HIGHEST_DEPTH, u32::from(b)));
                    depth_a > depth_b
                };
                if swap {
                    self.write16(ORDER + 2 * i, b);
                    self.write16(ORDER + 2 * j, a);
                }
            }
        }
        let d4 = low_word(d4, 6);
        let best = self.read16(ORDER);
        let mut d2 = low_word(d2, best);
        let mut way = u32::from(best);
        let first = self.read8(at(FIRST, way));
        if first == 0xF0 {
            return (d2, d4);
        }
        let here = self.read8(HERE_LEVEL);
        if first >= here {
            if first != here || here < self.read8(at(HIGHEST, way)) {
                return (d2, d4);
            }
            let (_, lowest, _) = self.r_008c3a(d1, d2);
            d2 = lowest;
            if d2 & 0x8000 != 0 {
                return (d2, d4);
            }
            way = d2;
        }
        if self.read8(at(OPEN, way)) == 0 {
            d2 = low_word(d2, 0xFFFF);
        }
        (d2, d4)
    }

    /// `func_008C3A`: the way whose lowest level is least (at most
    /// 0xF0; the last of equals).  Gives back d1 (that level in its low
    /// byte, 0xF0 if none), d2 (the way in its low word, -1 if none)
    /// and a2 (the way, sign-extended).
    pub fn r_008c3a(&mut self, d1: u32, d2: u32) -> (u32, u32, u32) {
        let mut least = 0xF0;
        let mut way: u16 = 0xFFFF;
        for w in 0..4u16 {
            let level = self.read8(LOWEST + u32::from(w));
            if level <= least {
                way = w;
                least = level;
            }
        }
        (low_byte(d1, least), low_word(d2, way), way as i16 as u32)
    }

    /// `func_008C6A`: the four ways from `HERE` (d1's low word) searched:
    /// each way's first cell's level (`FIRST`; below `LIMIT`, the way
    /// closed), then by its kind: 2 found at depth 1 (unless a 5 was
    /// at depth 1), 5 (with `HERE_LEVEL` at least 0x18 and the cell's
    /// `GRID_AC00` byte 0x88, or 0x8E with `CURRENT` 5 on, as 2; else
    /// found at depth 1) and 4-6 searched on (`func_008DE2`); others a
    /// level 0 at the most and 0xF0 at the least, and a closed way of
    /// no request.  Gives back d1 and d7.
    pub fn r_008c6a(&mut self, _d0: u32, d1: u32, d7: u32) -> (u32, u32) {
        let mut d7 = d7;
        for way in 0..WAYS {
            self.write8(DEPTH, 1);
            let limit = self.read8(LIMIT);
            self.write8(LIMIT_COPY, limit);
            self.write8(HIGH_DEPTH, 0xF);
            self.write8(LOW_DEPTH, 0xF);
            let w = u32::from(way);
            let step = self.read8(STEPS + w);
            let next = low_byte(d1, (d1 as u8).wrapping_add(step));
            let cell = self.read8(at(GRID, next));
            let level = cell & LEVEL;
            self.write8(FIRST + w, level);
            self.write8(HIGH, level);
            self.write8(LOW, level);
            if level < self.read8(LIMIT_COPY) {
                self.write8(OPEN + w, 0);
            }
            let kind = cell & KIND;
            let found = |g: &mut Game, kind: u8| {
                g.write8(FOUND_KIND, kind);
                g.write16(FOUND_WAY, u16::from(way));
                g.write8(FOUND_DEPTH, 1);
            };
            let mut closed = true;
            match kind {
                2 => {
                    if !(self.read8(FOUND_DEPTH) == 1 && self.read8(FOUND_KIND) == 5) {
                        found(self, 2);
                    }
                }
                5 => {
                    let mut as_2 = false;
                    if self.read8(HERE_LEVEL) >= 0x18 {
                        self.write8(CELL_X, next as u8 & 0xF);
                        self.write8(CELL_Y, next as u8 >> 4);
                        let (index, _, _) = self.r_0066be(next, u32::from(way));
                        let b = self.read8(at(GRID_AC00, index));
                        as_2 = b == 0x88 || (self.read8(CURRENT) >= 5 && b == 0x8E);
                    }
                    if as_2 {
                        found(self, 2);
                    } else {
                        found(self, 5);
                        self.search_on(cell & LEVEL, next, way, &mut d7);
                        closed = false;
                    }
                }
                4 | 6 => {
                    self.search_on(level, next, way, &mut d7);
                    closed = false;
                }
                _ => {
                    self.write8(HIGH, 0);
                    self.write8(LOW, 0xF0);
                }
            }
            if closed {
                self.r_008d4e(way as i16 as u32);
                self.write8(OPEN + w, 0);
            }
            for (from, to) in [(HIGH, HIGHEST), (HIGH_DEPTH, HIGHEST_DEPTH), (LOW, LOWEST), (LOW_DEPTH, LOWEST_DEPTH)] {
                let b = self.read8(from);
                self.write8(to + w, b);
            }
        }
        (d1, d7)
    }

    /// A way's cell of `level` at `cell` taken in (`func_008E9E`,
    /// `func_008EC6`) and searched on from (`func_008DE2`).
    fn search_on(&mut self, level: u8, cell: u32, way: u8, d7: &mut u32) {
        self.r_008e9e(u32::from(level));
        self.r_008ec6(u32::from(level));
        *d7 = self.r_008de2(cell, u32::from(way), u32::from(way), *d7, GRID);
    }

    /// `func_008D4E`: the request of the way `a2` 0.
    pub fn r_008d4e(&mut self, way: u32) {
        self.write8(REQUESTS.wrapping_add(way), 0);
    }

    /// `func_008DE2`: one deeper (`DEPTH`), the three ways from `cell`
    /// (d1's low word) but back (`BACK` of the way `came`, d3) searched
    /// in the grid at a1: a kind 2 found if shallower than the found
    /// depth, a 5 found if not deeper (and searched on), 4-6 taken in
    /// (`func_008E9E`, `func_008EC6`) and, short of `MAX_DEPTH`,
    /// searched on.  `found_way` (d2's low word) is the way searched.
    /// Gives back d7 (its low byte the way back).
    pub fn r_008de2(&mut self, cell: u32, found_way: u32, came: u32, d7: u32, grid: u32) -> u32 {
        let depth = self.read8(DEPTH).wrapping_add(1);
        self.write8(DEPTH, depth);
        let back = self.read8(at(BACK, came));
        let d7 = low_byte(d7, back);
        for way in 0..WAYS {
            if way == back {
                continue;
            }
            let step = self.read8(STEPS + u32::from(way));
            let next = low_byte(cell, (cell as u8).wrapping_add(step));
            let at_next = at(grid, next);
            let cell_byte = self.read8(at_next);
            let level = match cell_byte & KIND {
                2 => {
                    let depth = self.read8(DEPTH);
                    if depth < self.read8(FOUND_DEPTH) {
                        self.write8(FOUND_DEPTH, depth);
                        self.write8(FOUND_KIND, 2);
                        self.write16(FOUND_WAY, found_way as u16);
                    }
                    continue;
                }
                5 => {
                    let depth = self.read8(DEPTH);
                    if self.read8(FOUND_DEPTH) < depth {
                        continue;
                    }
                    self.write8(FOUND_DEPTH, depth);
                    self.write8(FOUND_KIND, 5);
                    self.write16(FOUND_WAY, found_way as u16);
                    self.read8(at_next) & LEVEL
                }
                4..=6 => cell_byte & LEVEL,
                _ => continue,
            };
            self.r_008e9e(u32::from(level));
            self.r_008ec6(u32::from(level));
            if self.read8(DEPTH) != MAX_DEPTH {
                self.r_008de2(next, found_way, low_word(came, u16::from(way)), d7, grid);
            }
        }
        let depth = self.read8(DEPTH);
        self.write8(DEPTH, depth.wrapping_sub(1));
        d7
    }

    /// `func_008E9E`: the level `d0` (its low byte) above `HIGH`, or
    /// equal to it at a lesser depth: `HIGH` and `HIGH_DEPTH` it.
    pub fn r_008e9e(&mut self, d0: u32) {
        let level = d0 as u8;
        let high = self.read8(HIGH);
        if level < high {
            return;
        }
        if level > high {
            self.write8(HIGH, level);
        } else if self.read8(DEPTH) >= self.read8(HIGH_DEPTH) {
            return;
        }
        let depth = self.read8(DEPTH);
        self.write8(HIGH_DEPTH, depth);
    }

    /// `func_008EC6`: the level `d0` (its low byte) below `LOW`, or
    /// equal to it at a lesser depth: `LOW` and `LOW_DEPTH` it.
    pub fn r_008ec6(&mut self, d0: u32) {
        let level = d0 as u8;
        let low = self.read8(LOW);
        if level > low {
            return;
        }
        if level < low {
            self.write8(LOW, level);
        } else if self.read8(DEPTH) >= self.read8(LOW_DEPTH) {
            return;
        }
        let depth = self.read8(DEPTH);
        self.write8(LOW_DEPTH, depth);
    }

    /// `func_008EEE`: the grid's levels spread out (`func_008F72`) from
    /// the objects in every object's list (`func_009056` first), level
    /// by level (0, 0x10, ... 0xF0) as long as some list isn't done;
    /// each list a pass takes up where it stopped (`PLACES`), at an
    /// object of a higher level.  Gives back d0 (the object count in
    /// its low word), d1, d2, d3 (the last level), d4, d6, d7 (the
    /// lists not done in the last pass), a0, a1 (`PLACES`), a6 (the last
    /// object) and X (set if it went past level 0xF0).
    pub fn r_008eee(&mut self, d1: u32, d3: u32, d4: u32, d6: u32, d7: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let (mut d0, mut d2, mut a0) = self.r_009056();
        self.write32(PLACES, 0);
        let mut d1 = low_word(d1, 0);
        let mut d6 = low_word(d6, 0);
        let mut d3 = low_byte(d3, 0);
        let (mut d4, mut d7, mut a6) = (d4, d7, a6);
        loop {
            let level = d3 as u8;
            d7 = low_word(d7, 0);
            d0 = low_word(d0, 0);
            loop {
                let list = d0 as u16;
                let place = at(PLACES, d0);
                let mut offset = self.read8(place);
                if offset & 0x80 == 0 {
                    d7 = low_word(d7, (d7 as u16).wrapping_add(1));
                    d2 = low_word(d2, list << 6);
                    a0 = at(at(LISTS, d2), u32::from(offset));
                    loop {
                        let first = self.read8(a0);
                        d6 = low_byte(d6, first);
                        if first == 0 {
                            offset = 0xFF;
                            break;
                        }
                        a6 = self.read32(a0 + ENTRY_OBJECT);
                        let its = self.read8(a6 + SPREAD_LEVEL) & LEVEL;
                        d2 = low_byte(d2, its);
                        if its > level {
                            break;
                        }
                        if its == level {
                            (d4, d6) = self.r_008f72(d4, d6, a6);
                        }
                        a0 = a0.wrapping_add(u32::from(ENTRY_SIZE));
                        offset = offset.wrapping_add(ENTRY_SIZE);
                        if offset >= LIST_SIZE {
                            offset = 0xFF;
                            break;
                        }
                    }
                }
                d1 = low_byte(d1, offset);
                self.write8(place, offset);
                d0 = low_word(d0, list.wrapping_add(1));
                if d0 as u8 == self.read8(OBJECT_COUNT) {
                    break;
                }
            }
            if d7 as u16 == 0 {
                return (d0, d1, d2, d3, d4, d6, d7, a0, PLACES, a6, false);
            }
            d3 = low_byte(d3, level.wrapping_add(0x10));
            if d3 as u8 == 0 {
                return (d0, d1, d2, d3, d4, d6, d7, a0, PLACES, a6, true);
            }
        }
    }

    /// `func_008F72`: unless its flags' bit 5 is set, the object at a6
    /// spreads its level (or, if its cell's kind isn't 1, its cell's)
    /// through the grid: its cell kind 7 of that level, then each way
    /// (from the last) as far as its reach: by the cells' kinds in
    /// `CELL_MAP`, 6 the level if lower (on), 7 and up that kind (on),
    /// 1 kind 7 of the level, others (2 as 0) that kind of the level
    /// (each of those last two stops the way).  Gives back d4 (the
    /// last way's step in its low word) and d6 (the reach left in its
    /// low byte).
    pub fn r_008f72(&mut self, d4: u32, d6: u32, a6: u32) -> (u32, u32) {
        if self.read8(a6 + SPREAD_FLAGS) & (1 << 5) != 0 {
            return (d4, d6);
        }
        let mut spread = (self.read8(a6 + SPREAD_LEVEL) & LEVEL) | 6;
        let reach = self.read8(a6 + SPREAD_REACH);
        self.write8(REACH, reach);
        let y = self.read8(a6 + SPREAD_Y);
        let x = self.read8(a6 + SPREAD_X);
        let here = u16::from((y << 4) | x);
        self.write16(HERE, here);
        let cell = self.read8(at(GRID, u32::from(here)));
        if cell & KIND != 1 {
            spread = (cell & LEVEL) | 6;
        }
        let level = spread & LEVEL;
        self.write8(at(GRID, u32::from(here)), 7 | level);
        self.write8(SPREAD_WAY, 3);
        let (mut d4, mut d6) = (d4, d6);
        loop {
            d6 = low_byte(d6, self.read8(REACH));
            let way = self.read8(SPREAD_WAY);
            let step = self.read8(STEPS_2 + u32::from(way)) as i8 as u16;
            d4 = low_word(d4, step);
            let mut cell = step.wrapping_add(self.read16(HERE));
            loop {
                let kind = self.read8(at(CELL_MAP, u32::from(cell)));
                let to = at(GRID, u32::from(cell));
                match kind {
                    6 => {
                        if spread < self.read8(to) {
                            self.write8(to, spread);
                        }
                    }
                    7.. => self.write8(to, kind),
                    1 => {
                        self.write8(to, 7 | level);
                        break;
                    }
                    _ => {
                        let kind = if kind == 2 { 0 } else { kind };
                        self.write8(to, kind | level);
                        break;
                    }
                }
                cell = cell.wrapping_add(step);
                let left = (d6 as u8).wrapping_sub(1);
                d6 = low_byte(d6, left);
                if left == 0 {
                    break;
                }
            }
            let way = self.read8(SPREAD_WAY).wrapping_sub(1);
            self.write8(SPREAD_WAY, way);
            if way & 0x80 != 0 {
                break;
            }
        }
        (d4, d6)
    }
}
