//! The sprites: what the game shows on the sprite plane, a list of
//! records kept in order of depth, and the sprite table made from it
//! once a frame (`func_0008B2`, from the main loop) for the vertical
//! blank to copy to the VDP.
//!
//! A record is 0x20 bytes in work RAM, found by its index in the ROM's
//! table of their addresses (`D_000B7E`, 66 of them).  Record 0 is the
//! list's head and record 1 its tail.  A task has one (its slot, the
//! task record's byte 3: the scheduler puts its index in `CURRENT` and
//! its address in `CURRENT_ADDR`).  The list is linked both ways, by
//! index and by address.
//!
//! What a record shows is a frame of one of 8 sets (`SETS`): a set has
//! a table of frames, each a list of pieces (a 16x16 or 32x16 sprite
//! each, at an offset from the record's position).

use crate::Game;

/// The records' addresses by index (`D_000B7E`, in the ROM; `D_000B86`
/// is its third entry).
const SPRITE_TABLE: u32 = 0x000B7E;
/// The sets' addresses (`D_000C9A`, in the ROM): 8 of `SETS`.
const SET_TABLE: u32 = 0x000C9A;

/// The records: 0x210 longs from the head (record 0) on.
const HEAD: u32 = 0xFFFF8AB4;
const TAIL: u32 = 0xFFFF8AD4;
const AREA_LONGS: u32 = 0x210;
/// The sets: two longs each, the frames' table and a table of each
/// frame's cost (`func_000776`).
const SETS: u32 = 0xFF8A74;

/// Compared with 0x44 before a record is taken (`func_000796`): set to
/// 2 at the start and, as far as the code goes, never again.
const LIMIT: u32 = 0xFF8A58;
/// The record the task running has: its index and its address (the
/// scheduler's, `func_0005C0`).
const CURRENT: u32 = 0xFF8A57;
const CURRENT_ADDR: u32 = 0xFF8A5A;
/// The depth of the record being linked in, then its index.
const DEPTH: u32 = 0xFF8A59;
/// The frame count: a record with `ALTERNATE` is drawn on every other
/// frame, by its index's parity.
const FRAME: u32 = 0xFF80D8;

/// The sprite table being made (in RAM; the vertical blank copies it).
const SPRITE_BUFFER: u32 = 0xFFFFD800;
/// While it is made: the sprites still free (of 64), the cost still
/// allowed (`BUDGETED`, of 12), the next sprite's link, the pieces of
/// the frame being drawn still to go, the priority and palette bits of
/// its pieces.
const SPRITES_LEFT: u32 = 0xFF8A60;
const BUDGET_LEFT: u32 = 0xFF8A61;
const NEXT_LINK: u32 = 0xFF8A73;
const PIECES_LEFT: u32 = 0xFF8A72;
const PIECE_BITS: u32 = 0xFF8A70;
const SPRITES: u8 = 0x40;
const BUDGET: u8 = 0x0C;

/// `func_000752`'s table: words.
const TILE_ROWS: u32 = 0xFF8A62;

/// A record's fields.
/// The next record's index (0xFF in the tail), the previous one's.
const NEXT: u32 = 0x00;
const PREV: u32 = 0x01;
/// Its depth: 0 free, 0xFF the tail; the list is in order of it, the
/// deepest drawn last (under the others).
const DEPTH_FIELD: u32 = 0x02;
/// The palette (bits 0-1).
const PALETTE: u32 = 0x03;
/// The frame shown (0: none).
const FRAME_FIELD: u32 = 0x04;
/// Flags: bits 0-2 the set, and the bits below.
const FLAGS: u32 = 0x05;
/// Tile offset added to each piece's (a `TWO_TABLES` record's).
const TILE_OFFSET: u32 = 0x06;
/// Its position (words): x, y.
const X: u32 = 0x08;
const Y: u32 = 0x0A;
/// The frame last drawn with the cost paid (`BUDGETED`).
const DRAWN_FRAME: u32 = 0x0C;
/// The next record's address, the previous one's.
const NEXT_ADDR: u32 = 0x10;
const PREV_ADDR: u32 = 0x14;

/// The flags.
const SET_BITS: u8 = 0x07;
/// Set when the cost was paid for the frame in `DRAWN_FRAME`.
const PAID: u8 = 1 << 3;
/// The frame costs (the set's second table): drawn while the frame's
/// budget lasts, else the frame last paid for.
const BUDGETED: u8 = 1 << 4;
/// Drawn with `func_000A1E`: pieces mirrored vertically, the tile
/// offset added.
const TWO_TABLES: u8 = 1 << 5;
const ALTERNATE: u8 = 1 << 6;
/// The sprites' priority bit.
const PRIORITY: u8 = 1 << 7;

/// The window a piece must be in to be drawn (sprite coordinates, 128
/// the screen's edge): y 0x70-0x16F, x 0x60-0x19F.
const Y_RANGE: std::ops::Range<u16> = 0x70..0x170;
const X_RANGE: std::ops::Range<u16> = 0x60..0x1A0;

/// What the drawing leaves in the registers the main loop's next pass
/// reads (they go on to the tasks: tools/recomp sees them read there).
struct Left {
    d0: u32,
    d1: u32,
    d3: u32,
    d6: u32,
    d7: u32,
    a3: u32,
    a4: u32,
    a6: u32,
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// The address of sprite record `index` (its low byte).
    fn sprite(&mut self, index: u32) -> u32 {
        self.read32(SPRITE_TABLE + 4 * (index & 0xFF))
    }

    /// `func_000704`: no records but the head and the tail (after
    /// `func_0103D4`), then `func_00977E`.  Gives back d0, d1 and a0 as
    /// the clearing leaves them, and X.
    pub fn r_000704(&mut self, d0: u32) -> (u32, u32, u32, bool) {
        let x = self.r_0103d4();
        self.write8(LIMIT, 2);
        for k in 0..AREA_LONGS {
            self.write32(HEAD + 4 * k, 0);
        }
        self.write8(HEAD + DEPTH_FIELD, 0);
        self.write8(HEAD + NEXT, 1);
        self.write32(HEAD + NEXT_ADDR, TAIL);
        self.write8(TAIL + DEPTH_FIELD, 0xFF);
        self.write8(TAIL + NEXT, 0xFF);
        self.write8(TAIL + PREV, 0);
        self.write32(TAIL + PREV_ADDR, HEAD);
        self.r_00977e();
        (low_word(d0, 0xFFFF), 0, HEAD + 4 * AREA_LONGS, x)
    }

    /// `func_000752`: `count` (d2's low byte) words of `TILE_ROWS` from
    /// the `first` (d1's low word): `tile` (d0's low word) / 32, then
    /// 0x100 more each.
    pub fn r_000752(&mut self, tile: u32, first: u32, count: u32) {
        let mut word = (tile as u16) >> 5;
        let at = TILE_ROWS.wrapping_add((first as u16).wrapping_mul(2) as i16 as u32);
        let n = (count as u16 & 0xFF).wrapping_sub(1);
        // a dbra: n + 1 words (0x10000 for a count of 0)
        for k in 0..=u32::from(n) {
            self.write16(at.wrapping_add(2 * k), word);
            word = word.wrapping_add(0x100);
        }
    }

    /// `func_000776`: set `set` (d0's low byte): its frames' table
    /// (a0) and its costs' (a1).  Gives back d0, the set x 8.
    pub fn r_000776(&mut self, set: u32, frames: u32, costs: u32) -> u32 {
        let offset = (set & 0xFF) * 8;
        self.write32(SETS + offset, frames);
        self.write32(SETS + offset + 4, costs);
        offset
    }

    /// `func_000796`: a new record at depth `depth` (d0's low byte), the
    /// first free one from 2; C set (and nothing done) when `LIMIT` is
    /// 0x44.  Gives back d5 (its index), a5 (its address), X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_000796(&mut self, depth: u32, _d1: u32, _d2: u32, _d3: u32, _d4: u32, d5: u32, _d6: u32, _d7: u32, _a0: u32, _a1: u32, _a2: u32, _a3: u32, _a4: u32, a5: u32, x: bool) -> (u32, u32, bool, bool) {
        self.write8(DEPTH, depth as u8);
        if self.read8(LIMIT) == 0x44 {
            return (d5, a5, x, true);
        }
        let mut index = 2u32;
        let mut record = self.sprite(index);
        let mut x = x;
        while self.read8(record + DEPTH_FIELD) != 0 {
            index += 1;
            record += 0x20;
            x = false;
        }
        let index = self.link_sprite(index, record);
        (index, record, x, false)
    }

    /// `func_0007D2`: the task's record at depth `depth` (d0's low
    /// byte), moved in the list.  Gives back C (clear).
    #[allow(clippy::too_many_arguments)]
    pub fn r_0007d2(&mut self, depth: u32, _d1: u32, _d2: u32, _d3: u32, _d4: u32, _d5: u32, _d6: u32, _d7: u32, _a0: u32, _a1: u32, _a2: u32, _a3: u32, _x: bool) -> bool {
        let index = self.read8(CURRENT);
        let record = self.read32(CURRENT_ADDR);
        self.write8(DEPTH, depth as u8);
        self.r_00084e(record);
        self.link_sprite(u32::from(index), record);
        false
    }

    /// `func_0007F4`: record `d5` (its low byte), at a5, linked in at
    /// the depth in `DEPTH`.  Gives back d5 (the index) and C (clear).
    #[allow(clippy::too_many_arguments)]
    pub fn r_0007f4(&mut self, _d0: u32, _d1: u32, _d2: u32, _d3: u32, _d4: u32, d5: u32, _d6: u32, _d7: u32, _a0: u32, _a1: u32, _a2: u32, _a3: u32, _a4: u32, a5: u32, _x: bool) -> (u32, bool) {
        (self.link_sprite(d5, a5), false)
    }

    /// Record `index` (its low byte), at `record`, linked in before the
    /// first record deeper than `DEPTH`; its index goes to `DEPTH`.
    /// Gives back the index.
    fn link_sprite(&mut self, index: u32, record: u32) -> u32 {
        let index = index as u8;
        let depth = self.read8(DEPTH);
        let mut next = self.read8(HEAD + NEXT);
        let mut next_record = self.read32(HEAD + NEXT_ADDR);
        while depth >= self.read8(next_record + DEPTH_FIELD) {
            next = self.read8(next_record + NEXT);
            next_record = self.read32(next_record + NEXT_ADDR);
        }
        self.write8(record + DEPTH_FIELD, depth);
        self.write8(record + NEXT, next);
        self.write32(record + NEXT_ADDR, next_record);
        let prev = self.read8(next_record + PREV);
        let prev_record = self.read32(next_record + PREV_ADDR);
        self.write8(prev_record + NEXT, index);
        self.write32(prev_record + NEXT_ADDR, record);
        self.write8(record + PREV, prev);
        self.write32(record + PREV_ADDR, prev_record);
        self.write8(next_record + PREV, index);
        self.write32(next_record + PREV_ADDR, record);
        self.write8(DEPTH, index);
        u32::from(index)
    }

    /// `func_00084E`: the record at `record` out of the list.  Gives
    /// back a4, the one before it.
    pub fn r_00084e(&mut self, record: u32) -> u32 {
        let prev = self.read8(record + PREV);
        let prev_record = self.read32(record + PREV_ADDR);
        let next = self.read8(record + NEXT);
        let next_record = self.read32(record + NEXT_ADDR);
        self.write8(next_record + PREV, prev);
        self.write32(next_record + PREV_ADDR, prev_record);
        self.write8(prev_record + NEXT, next);
        self.write32(prev_record + NEXT_ADDR, next_record);
        prev_record
    }

    /// `func_000870`: record `d0` (its low byte) freed: it becomes the
    /// current one (`CURRENT`) and `func_000890` frees it.  Gives back X
    /// (clear).
    pub fn r_000870(&mut self, index: u32, _d5: u32) -> bool {
        let record = self.sprite(index);
        self.write8(CURRENT, index as u8);
        self.write32(CURRENT_ADDR, record);
        self.r_000890();
        false
    }

    /// `func_000890`: the current record freed: out of the list, its
    /// palette and depth 0; its index to `DEPTH`.  Gives back C (clear).
    pub fn r_000890(&mut self) -> bool {
        let index = self.read8(CURRENT);
        let record = self.read32(CURRENT_ADDR);
        self.write8(record + PALETTE, 0);
        self.r_00084e(record);
        self.write8(DEPTH, index);
        self.write8(record + DEPTH_FIELD, 0);
        false
    }

    /// `func_0008B2`: the sprite table made from the list, from the
    /// head on (the backdrop colour switched around it, `func_000296`,
    /// which does nothing).  The registers it is given and gives back
    /// are those the drawing leaves ([`Left`]).
    #[allow(clippy::too_many_arguments)]
    pub fn r_0008b2(&mut self, d0: u32, d1: u32, d3: u32, d6: u32, d7: u32, a3: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        let mut left = Left { d0, d1, d3, d6, d7, a3, a4, a6 };
        self.r_000296(0xE0);
        let mut at = SPRITE_BUFFER;
        self.write8(SPRITES_LEFT, SPRITES);
        self.write8(BUDGET_LEFT, BUDGET);
        self.write8(NEXT_LINK, 1);
        let mut record = HEAD;
        loop {
            let index = self.read8(record + NEXT);
            if index == 1 {
                break;
            }
            record = self.read32(record + NEXT_ADDR);
            if self.read8(record + FRAME_FIELD) == 0 {
                continue;
            }
            let flags = self.read8(record + FLAGS);
            left.d0 = low_byte(left.d0, flags);
            if flags & ALTERNATE != 0 {
                let parity = self.read8(FRAME) ^ index;
                left.d1 = low_byte(left.d1, parity);
                if parity & 1 != 0 {
                    continue;
                }
            }
            if flags & TWO_TABLES != 0 {
                self.draw_two_tables(&mut left, &mut at, record);
            } else {
                self.draw_one_table(&mut left, &mut at, record);
            }
        }
        let sprites_left = self.read8(SPRITES_LEFT);
        if sprites_left != 0 {
            if sprites_left == SPRITES {
                // nothing drawn: one sprite, at y 0
                self.write16(at, 0);
                at += 8;
            }
            // the last sprite's link: the end
            self.write8(at.wrapping_sub(5), 0);
        }
        self.r_000296(0xFFFF);
        let l = left;
        (l.d0, l.d1, l.d3, l.d6, l.d7, l.a3, l.a4, l.a6)
    }

    /// The start of drawing a record (both kinds): the priority bit to
    /// `PIECE_BITS`, the set (a6); gives back the frame x 4.
    fn draw_start(&mut self, left: &mut Left, flags: u8, frame: u8) -> u16 {
        let bits = u16::from(flags & PRIORITY) << 8;
        left.d3 = low_word(low_byte(left.d3, flags), bits);
        self.write16(PIECE_BITS, bits);
        let set = u16::from(flags & SET_BITS) * 4;
        left.d6 = low_word(left.d6, set);
        left.a6 = self.read32(SET_TABLE + u32::from(set));
        let by_frame = u16::from(frame) * 4;
        left.d0 = low_word(left.d0, by_frame);
        by_frame
    }

    /// Fewer sprites left than `pieces`? (d0's low byte the sprites
    /// left.)
    fn too_few_sprites(&mut self, left: &mut Left, pieces: u8) -> bool {
        let sprites_left = self.read8(SPRITES_LEFT);
        left.d0 = low_byte(left.d0, sprites_left);
        sprites_left < pieces
    }

    /// The pieces of a frame from `pieces` (4 bytes each: y, x, then
    /// the sprite's word: size bit 15, tile and flip bits), at the
    /// record's position, those in the window to the table at `at`.
    /// `mirrored`: y negated, and the record's tile offset added.
    fn draw_pieces(&mut self, left: &mut Left, at: &mut u32, record: u32, mut pieces: u32, mirrored: bool) {
        let palette = u16::from(self.read8(record + PALETTE) & 3) << 13;
        left.d0 = low_word(left.d0, palette);
        let bits = self.read16(PIECE_BITS) | palette;
        self.write16(PIECE_BITS, bits);
        let x0 = self.read16(record + X).wrapping_add(0x80);
        let y0 = self.read16(record + Y).wrapping_add(0x80);
        left.d6 = low_word(left.d6, x0);
        left.d7 = low_word(left.d7, y0);
        let tile_offset = if mirrored { self.read16(record + TILE_OFFSET) } else { 0 };
        let mut count = self.read8(PIECES_LEFT);
        loop {
            let dy = self.read8(pieces);
            let dy = if mirrored { dy.wrapping_neg() } else { dy };
            let y = (dy as i8 as u16).wrapping_add(y0);
            left.d0 = low_word(left.d0, y);
            if Y_RANGE.contains(&y) {
                self.write16(*at, y);
                let x = (self.read8(pieces + 1) as i8 as u16).wrapping_add(x0);
                left.d0 = low_word(left.d0, x);
                if X_RANGE.contains(&x) {
                    self.write16(*at + 6, x);
                    let word = self.read16(pieces + 2);
                    let tile = if mirrored {
                        word.wrapping_add(tile_offset) & 0x7FFF
                    } else {
                        word & 0x7FFC
                    };
                    self.write16(*at + 4, tile.wrapping_add(bits));
                    let link = self.read8(NEXT_LINK);
                    self.write8(NEXT_LINK, link.wrapping_add(1));
                    let size = ((word & 0x8000) >> 4) | 0x500;
                    let size_link = (size & 0xFF00) | u16::from(link);
                    left.d0 = low_word(left.d0, size_link);
                    self.write16(*at + 2, size_link);
                    let sprites_left = self.read8(SPRITES_LEFT);
                    self.write8(SPRITES_LEFT, sprites_left.wrapping_sub(1));
                    *at += 8;
                }
            }
            pieces += 4;
            count = count.wrapping_sub(1);
            if count == 0 {
                break;
            }
        }
        self.write8(PIECES_LEFT, 0);
        left.a3 = pieces;
    }

    /// `func_000946`, a record drawn from its set's frames' table: the
    /// frame (a count, a byte unused, the pieces) if there are sprites
    /// enough left for all its pieces.
    fn draw_one_table(&mut self, left: &mut Left, at: &mut u32, record: u32) {
        let flags = self.read8(record + FLAGS);
        let frame = self.read8(record + FRAME_FIELD);
        let by_frame = self.draw_start(left, flags, frame);
        let frames = self.read32(left.a6);
        let mut pieces = self.read32(frames.wrapping_add(u32::from(by_frame)));
        let count = self.read8(pieces);
        pieces += 2;
        left.a3 = pieces;
        self.write8(PIECES_LEFT, count);
        if self.too_few_sprites(left, count) {
            return;
        }
        self.draw_pieces(left, at, record, pieces, false);
    }

    /// `func_000A1E`, a record drawn mirrored, from its set's frames'
    /// table and costs' table: with `BUDGETED`, the frame if its cost is
    /// in the budget left (paid: the frame kept in `DRAWN_FRAME`, `PAID`
    /// set), else the frame last paid for.
    fn draw_two_tables(&mut self, left: &mut Left, at: &mut u32, record: u32) {
        let flags = self.read8(record + FLAGS);
        let frame = self.read8(record + FRAME_FIELD);
        let by_frame = self.draw_start(left, flags, frame);
        let frames = self.read32(left.a6);
        let mut pieces = self.read32(frames.wrapping_add(u32::from(by_frame)));
        let costs = self.read32(left.a6 + 4);
        let cost_at = self.read32(costs.wrapping_add(u32::from(by_frame)));
        left.a3 = pieces;
        left.a4 = cost_at;
        let count = self.read8(pieces);
        self.write8(PIECES_LEFT, count);
        if self.too_few_sprites(left, count) {
            return;
        }
        if self.read8(record + FLAGS) & BUDGETED != 0 {
            let budget = self.read8(BUDGET_LEFT);
            let cost = self.read8(cost_at);
            let rest = budget.wrapping_sub(cost);
            left.d0 = low_byte(left.d0, rest);
            if budget >= cost {
                self.write8(BUDGET_LEFT, rest);
                let flags = self.read8(record + FLAGS);
                self.write8(record + FLAGS, (flags | PAID) & !BUDGETED);
                self.write32(record + DRAWN_FRAME, pieces);
            } else {
                pieces = self.read32(record + DRAWN_FRAME);
                left.a3 = pieces;
            }
            let count = self.read8(pieces);
            self.write8(PIECES_LEFT, count);
            if count == 0 || self.too_few_sprites(left, count) {
                return;
            }
        }
        self.draw_pieces(left, at, record, pieces + 2, true);
    }

    /// `func_000946` (above), with the registers it is given and gives
    /// back.
    pub fn r_000946(&mut self, d0: u32, d3: u32, d6: u32, d7: u32, a2: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        let mut left = Left { d0, d1: 0, d3, d6, d7, a3: 0, a4: 0, a6: 0 };
        let mut at = a2;
        self.draw_one_table(&mut left, &mut at, a5);
        let l = left;
        (l.d0, l.d3, l.d6, l.d7, at, l.a3, l.a6)
    }

    /// `func_000A1E` (above), with the registers it is given and gives
    /// back.
    pub fn r_000a1e(&mut self, d0: u32, d3: u32, d6: u32, d7: u32, a2: u32, a5: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        let mut left = Left { d0, d1: 0, d3, d6, d7, a3: 0, a4: 0, a6: 0 };
        let mut at = a2;
        self.draw_two_tables(&mut left, &mut at, a5);
        let l = left;
        (l.d0, l.d3, l.d6, l.d7, at, l.a3, l.a4, l.a6)
    }

    /// `func_000B48`, `func_000B5A`: a6 (a5) = record `d6` (`d5`); the
    /// index is kept.  Gives back X (clear: the index x 4 doesn't carry
    /// out of the word).
    pub fn r_000b48(&mut self, d6: u32) -> (u32, u32, bool) {
        let a6 = self.sprite(d6);
        (d6, a6, false)
    }

    pub fn r_000b5a(&mut self, d5: u32) -> (u32, u32, bool) {
        let a5 = self.sprite(d5);
        (d5, a5, false)
    }

    /// `func_000B6C`: a4 = record `d4`, which no caller reads.
    pub fn r_000b6c(&mut self, _d4: u32) {}

    /// `func_000C86`: a0 = set `d0` (its low word, not masked); d0 is
    /// left x 4, which no caller reads.
    pub fn r_000c86(&mut self, set: u32) -> u32 {
        let by_set = (set as u16).wrapping_mul(4);
        self.read32(SET_TABLE.wrapping_add(by_set as i16 as u32))
    }

    /// `func_000C90`: a6 = set `d6` (its low word, not masked); gives
    /// back d6, its low word x 4.
    pub fn r_000c90(&mut self, set: u32) -> (u32, u32) {
        let by_set = (set as u16).wrapping_mul(4);
        let a6 = self.read32(SET_TABLE.wrapping_add(by_set as i16 as u32));
        (low_word(set, by_set), a6)
    }
}
