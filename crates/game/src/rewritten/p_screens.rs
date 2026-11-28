//! The screens of choices before a game (0x110C8-0x11A38): each set up
//! by a task routine (the screen cleared, `func_01052E`; its graphics
//! loaded from a list, `func_0094D8`; its objects and tasks made; then
//! `func_00A506`), and the routines of the tasks that wait on it for a
//! button or a count and then leave for the next (`func_001212` then
//! the kept routine, after `func_00A51A`).  Routines that wait for the
//! frame (`func_011150`, `func_01111E`, `func_0118B4`, `func_011A38`
//! and the slides', `func_011A9C`) are left translated.

use crate::{Game, Regs};

/// The screen shown (a byte).
const SCREEN: u32 = 0xFFAD3A;
/// The list `func_0094D8` loads graphics by (a long).
const LOAD_LIST: u32 = 0xFF92FA;
/// The lists of the screens (in the ROM: D_011386, D_011586,
/// D_0119E6) and one made in RAM (`func_01141C`: 7 bytes from
/// D_01146A, its first byte and the word at 4 set by object).
const LIST_5: u32 = 0x011386;
const LIST_7: u32 = 0x011586;
const LIST_8: u32 = 0x0119E6;
const OBJECT_LIST: u32 = 0xFFFF_AD46;
const OBJECT_LIST_WORD: u32 = 0xFFAD4A;
const OBJECT_LIST_FROM: u32 = 0x01146A;
const OBJECT_LIST_BYTES: u32 = 7;
/// `func_01141C`'s words by object (D_011472).
const OBJECT_LIST_WORDS: u32 = 0x011472;
/// The graphics of an object's side: 0x1F + its side.
const SIDE_GRAPHICS: u8 = 0x1F;

/// The number of objects in play (a byte) and flags (bit 0 set and
/// bits 2-7 cleared when a game starts, `func_0114C0`; bit 1, the
/// objects go by side).
const OBJECTS: u32 = 0xFF9541;
const FLAGS: u32 = 0xFF9540;
const BY_SIDE: u8 = 1 << 1;
/// A number (a byte): 1 draws a second block (`func_0113A2`), and it
/// is the pick `func_0118C4` waits for.
const NUMBER: u32 = 0xFF9542;
/// The pick `func_01179C` made.
const PICK: u32 = 0xFFAD42;
/// While not 0, `func_01179C`'s task runs.
const PICKING: u32 = 0xFFAD44;
/// The choice (0-9, `p_choice.rs`): 7 sets other numbers for the game.
const CHOICE: u32 = 0xFFAD3F;
const CHOICE_7: u8 = 7;
/// Bytes a game starts with (`func_0114C0`).
const GAME_A937: u32 = 0xFFA937;
const GAME_A938: u32 = 0xFFA938;
const GAME_A939: u32 = 0xFFA939;
const GAME_A93A: u32 = 0xFFA93A;
const GAME_9548: u32 = 0xFF9548;
/// While 0 the objects are put in their row, else walked
/// (`p_marks.rs`).
const WALK: u32 = 0xFFAD40;
/// A byte cleared by `func_01151A`.
const CLEARED_80DF: u32 = 0xFF80DF;
/// A long cleared by `func_011920`.
const CLEARED_80E6: u32 = 0xFF80E6;

/// The direction (`p_slide.rs`) and the scroll tables' first words
/// (`h_scroll.rs`).
const DIRECTION: u32 = 0xFFAABD;
const H_VALUE: u32 = 0xFFD002;
const V_VALUE: u32 = 0xFFD402;
/// The position (words) cleared after a block is drawn.
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
/// A velocity's x (a word; `a_motion.rs`).
const VELOCITY_X: u32 = 0xFF9EC4;

/// What `func_0101C0` draws: the tiles' word (the low byte counted up
/// as it goes), a VRAM address (its low byte apart); a second tile
/// word.
const TILE: u32 = 0xFF80D0;
const TILE_LOW: u32 = 0xFF80D1;
const AT: u32 = 0xFF80C8;
const AT_LOW: u32 = 0xFF80C9;
const TILE_2: u32 = 0xFF80D2;
/// `func_00AA00`'s: what it takes (a long at `TILE`) and a word at
/// `AT`.
const TEXT: u32 = TILE;
/// A block's size for `func_0101C0` (d1: columns, rows).
const BLOCK: u16 = 0x201;
const MARK_BLOCK: u16 = 0x304;
/// The plane's base for `func_0113A2`'s blocks, the second tile's
/// offset from the block, and the VRAM's second row (+ 0x80 in the
/// command).
const PLANE: u16 = 0xE000;
const SECOND_AT: u16 = 0x0C;
const NEXT_ROW: u32 = 0x80_0000;
/// The VDP's ports.
const DATA: u32 = 0xC00000;
const CONTROL: u32 = 0xC00004;
/// The objects' list kept by `func_00B126` (a byte each).
const SORTED: u32 = 0xFFFF_9E88;

/// The sprite record of the routine being run.
const SPRITE_X_LOW_OFF: u32 = 0x09;
const SPRITE_X_HIGH_OFF: u32 = 0x08;
const SPRITE_Y_LOW_OFF: u32 = 0x0B;

/// A task's fields: a word (`func_01151A`'s count), its count down (a
/// byte), the routine kept for after `func_001212`.
const TASK_WORD: u32 = 0x0E;
const TASK_COUNT: u32 = 0x16;
const TASK_KEPT: u32 = 0x1C;
/// An object's flags: bits 0-2 the marks drawn by it, bit 6 its side.
const OBJ_FLAGS: u32 = 0x2D;

/// The routines: the wait every screen leaves through, the screens'.
const WAIT: u32 = 0x001212;
const SCREEN_5: u32 = 0x011312;
const SCREEN_8: u32 = 0x011920;
const SCREEN_8_DONE: u32 = 0x011A38;
const MENU: u32 = 0x01111E;
const GAME: u32 = 0x0114C0;
const SCREEN_5_TASK: u32 = 0x01147C;
const SCREEN_7_TASK: u32 = 0x01159A;
const SCREEN_8_TASK: u32 = 0x0119FA;
/// The tasks' priority.
const TASK_PRIORITY: u8 = 8;
/// The sound commands sent: the screens' music, and a stop.
const SOUND_5: u8 = 0x17;
const SOUND_7: u8 = 0x1A;
const SOUND_8: u8 = 0x1B;
const SOUND_STOP: u8 = 0xFE;
/// What `func_00349E` gets with the stop (d1).
const STOP_ARG: u32 = 0x20;
/// The buttons a screen waits for: bits 4-7 (`func_0102B6`), bits
/// 5-7 (`func_0102E8`) of the pads' bytes gathered in d0's high byte.
const BUTTONS: u8 = 0xF0;
const BUTTONS_8: u8 = 0xE0;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// d0's low word shifted right 8 (`lsr.w #8`): the word and X (the
/// last bit out).
fn high_byte(d0: u32) -> (u32, bool) {
    let w = d0 as u16;
    (low_word(d0, w >> 8), w & 0x80 != 0)
}

impl Game {
    /// `func_0094D8` with the list `list`, on all the registers.  True
    /// when the game restarted meanwhile.
    fn p_load(&mut self, r: &mut Regs, list: u32) -> bool {
        self.write32(LOAD_LIST, list);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
        self.restarted.is_some()
    }

    /// `func_01052E` on all the registers.
    fn p_clear(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d6, r.a0, r.a6, r.x) =
            self.r_01052e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
    }

    /// `func_008664` with the scroll's first words `h` and `v`.
    fn p_scroll(&mut self, r: &Regs, h: u16, v: u16) {
        self.write16(H_VALUE, h);
        self.write16(V_VALUE, v);
        let mut copy = *r;
        self.r_008664(&mut copy);
    }

    /// `func_00A506` (d0 4).
    fn p_show(&mut self, r: &mut Regs) {
        r.d0 = 4;
        (r.d0, r.a1, r.x, r.c) = self.r_00a506(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// The way out of a screen: `next` kept in the task (a6), the
    /// task's routine `func_001212`, the sound stopped, then
    /// `func_00A51A` (d0 2).  d1 is left 0x20.
    fn p_leave(&mut self, r: &mut Regs, next: u32) {
        self.write32(r.a6 + TASK_KEPT, next);
        self.r_00053e(r.d6, WAIT);
        r.d1 = STOP_ARG;
        r.d0 = low_byte(r.d0, SOUND_STOP);
        self.r_00349e(r.d0);
        r.d0 = 2;
        (r.d0, r.a1, r.x, r.c) = self.r_00a51a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
    }

    /// Whether a button of `mask` is down (`read` is `func_0102B6` or
    /// `func_0102E8`): d0 as the test leaves it, and X.
    fn p_buttons(&mut self, r: &mut Regs, read: fn(&mut Self, u32) -> u32, mask: u8) -> bool {
        let d0 = read(self, r.d0);
        let (d0, x) = high_byte(d0);
        let pressed = d0 as u8 & mask;
        r.d0 = low_byte(d0, pressed);
        r.x = x;
        pressed != 0
    }

    /// `func_0110C8`: nothing.
    pub fn r_0110c8(&mut self) {}

    /// `func_0112E0` (unless `func_0109FA` ends the task): `func_010E64`,
    /// then the sprite (a5) at x 0x80, y's low byte 0xA0.
    /// No script calls it: reviewed by reading.
    pub fn r_0112e0(&mut self, d0: u32, d5: u32, d6: u32, a5: u32, a6: u32, x: bool) {
        let (d0, _, _, _, unwound) = self.r_0109fa(d0, d5, d6, a6, x, 0x0112E6);
        if unwound {
            return;
        }
        self.r_010e64(d0, a5, a6);
        self.write8(a5 + SPRITE_Y_LOW_OFF, 0xA0);
        self.write8(a5 + SPRITE_X_HIGH_OFF, 0);
        self.write8(a5 + SPRITE_X_LOW_OFF, 0x80);
    }

    /// `func_01124A` and `func_01127E` (`func_011150`'s after its
    /// yields): `func_010846`, the task's routine `func_01111E` again.
    fn p_back_to_menu(&mut self, r: &mut Regs) {
        r.unwound = false;
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6) =
            self.r_010846(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        if self.restarted.is_some() {
            return;
        }
        (r.x, r.c) = self.r_00053e(r.d6, MENU);
    }

    /// `func_01124A`: `p_back_to_menu`.
    pub fn r_01124a(&mut self, regs: &mut Regs) {
        self.p_back_to_menu(regs);
    }

    /// `func_01127E`: `p_back_to_menu`.
    pub fn r_01127e(&mut self, regs: &mut Regs) {
        self.p_back_to_menu(regs);
    }

    /// `func_011312`, a task's routine: screen 5: cleared, its list
    /// loaded, a task `func_01147C` made, a block drawn
    /// (`func_0113A2`), the objects' graphics (`func_01141C`), its
    /// music, then `func_011360`.
    pub fn r_011312(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.p_clear(r);
        self.write8(SCREEN, 5);
        if self.p_load(r, LIST_5) {
            return;
        }
        r.a0 = SCREEN_5_TASK;
        r.d0 = u32::from(TASK_PRIORITY);
        (r.d6, r.a6, r.x, r.c) = self.r_000404(r.d0, r.d4, r.d5, r.d6, r.a0);
        self.write8(r.a6 + TASK_COUNT, 0);
        self.write16(TILE, 0xA050);
        self.write16(TILE_2, 0xA0B2);
        self.write8(AT_LOW, 0x4C);
        (r.d0, r.d1, r.a0, r.a6, r.x) = self.r_0113a2(r.d0, r.d1, r.a0, r.a6);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) =
            self.r_01141c(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a5, r.a6);
        if self.restarted.is_some() {
            return;
        }
        r.d0 = low_byte(r.d0, SOUND_5);
        self.r_0034b6(r.d0);
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a1, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_011360(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
    }

    /// `func_011360`: direction 2, the scroll at (0xFE04, 0), the
    /// objects in their row (`func_011B62`), then `func_00A506`.
    /// Gives back d0, d1, d2, d4, d5, d6, a0, a1, a4, a5, a6, X and C.
    pub fn r_011360(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        self.write8(DIRECTION, 2);
        self.p_scroll(&r, 0xFE04, 0);
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6) =
            self.r_011b62(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5, r.a6);
        self.p_show(&mut r);
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a1, r.a4, r.a5, r.a6, r.x, r.c)
    }

    /// `func_0113A2`: a block (2 x 1, `func_0101C0`) at `AT_LOW`'s
    /// column on the plane, its tiles `TILE` moved 2 x `NUMBER`
    /// (signed) on; when `NUMBER` is 1 the tile word `TILE_2` written
    /// to the plane at the block + 0xC and on the row below (with the
    /// interrupts masked).  The position cleared.  Gives back d0 (its
    /// low word as it came, its low byte `NUMBER`), d1, a0, a6 and X.
    pub fn r_0113a2(&mut self, d0: u32, d1: u32, a0: u32, a6: u32) -> (u32, u32, u32, u32, bool) {
        let d1 = low_word(d1, BLOCK);
        let number = self.read8(NUMBER);
        let kept = low_byte(d0, number) as u16;
        let offset = (number as i8 as u16).wrapping_mul(2);
        let tile = self.read16(TILE).wrapping_add(offset);
        self.write16(TILE, tile);
        let at = (0x100 | u16::from(self.read8(AT_LOW))).wrapping_mul(2).wrapping_add(PLANE);
        self.write16(AT, at);
        let (d0, mut d1, a0, a6, mut x) = self.r_0101c0(low_word(d0, at), d1, a0, a6);
        self.write16(AT, at);
        let mut d0 = low_word(d0, kept);
        if kept as u8 == 1 {
            d1 = low_word(d1, self.read16(TILE_2));
            let (second, carry) = self.read16(AT).overflowing_add(SECOND_AT);
            x = carry;
            let ipl = self.ipl();
            self.set_ipl(7);
            let (command, _) = self.r_000282(low_word(d0, second));
            self.write32(CONTROL, command);
            self.write16(DATA, d1 as u16);
            let command = command.wrapping_add(NEXT_ROW);
            self.write32(CONTROL, command);
            self.write16(DATA, d1 as u16);
            self.set_ipl(ipl);
            d0 = command;
        }
        self.write16(POSITION_X, 0);
        self.write16(POSITION_Y, 0);
        (d0, d1, a0, a6, x)
    }

    /// `func_01141C`: the list at `OBJECT_LIST` made from
    /// `OBJECT_LIST_FROM`, then for each object its first byte
    /// `SIDE_GRAPHICS` + the object's side and its word by the
    /// object's index, and loaded (`func_0094D8`).  Gives back the
    /// registers and X.
    pub fn r_01141c(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        for i in 0..OBJECT_LIST_BYTES {
            let b = self.read8(OBJECT_LIST_FROM + i);
            self.write8(OBJECT_LIST + i, b);
        }
        let mut r = Regs {
            d0: 0xFFFF,
            d1,
            d2,
            d3,
            d4: low_word(d4, 0),
            d5,
            d6,
            d7,
            a0: OBJECT_LIST_FROM + OBJECT_LIST_BYTES,
            a1: OBJECT_LIST + OBJECT_LIST_BYTES,
            a2,
            a3,
            a5,
            a6,
            ..Regs::default()
        };
        loop {
            (_, r.a4, r.x) = self.r_001cfa(r.d4);
            let index = (r.d4 as u8).wrapping_mul(2) as i8;
            let word = self.read16(OBJECT_LIST_WORDS.wrapping_add(i32::from(index) as u32));
            self.write16(OBJECT_LIST_WORD, word);
            let side = (self.read8(r.a4 + OBJ_FLAGS) >> 6) & 1;
            let first = SIDE_GRAPHICS + side;
            r.d0 = low_byte(low_word(r.d0, index as u16), first);
            r.x = false;
            self.write8(OBJECT_LIST, first);
            if self.p_load(&mut r, OBJECT_LIST) {
                break;
            }
            let index;
            (index, r.x) = (r.d4 as u8).overflowing_add(1);
            r.d4 = low_byte(r.d4, index);
            if index == self.read8(OBJECTS) {
                break;
            }
        }
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x)
    }

    /// `func_01147C`, screen 5's task: when its count runs out or a
    /// button is down, the game (`func_0114C0`) after the screen.
    pub fn r_01147c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (c, stop) = self.r_001bb4();
        if stop {
            r.c = c;
            return;
        }
        let count = self.read8(r.a6 + TASK_COUNT).wrapping_sub(1);
        self.write8(r.a6 + TASK_COUNT, count);
        if count != 0 && !self.p_buttons(r, Self::r_0102b6, BUTTONS) {
            r.c = false;
            return;
        }
        self.p_leave(r, GAME);
    }

    /// `func_0114C0`, a task's routine: a game starts: `FLAGS` bit 0
    /// set and bits 2-7 cleared, `func_00B49A`, the game's bytes set
    /// (others for choice 7), the tasks and sprites cleared
    /// (`func_0003DA`, `func_000704`, `func_001C56`) and on to
    /// `loc_0099E6`.
    pub fn r_0114c0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let flags = self.read8(FLAGS) | 1;
        self.write8(FLAGS, flags);
        self.write8(FLAGS, flags & 3);
        let mut copy = *r;
        self.r_00b49a(&mut copy);
        let (a937, a938, a939) = if self.read8(CHOICE) == CHOICE_7 { (8, 5, 0x10) } else { (2, 1, 0) };
        self.write8(GAME_A937, a937);
        self.write8(GAME_A938, a938);
        self.write8(GAME_A939, a939);
        self.write8(GAME_A93A, 1);
        self.write8(GAME_9548, 0);
        (r.d0, r.d1, r.d6, r.a0, r.a6) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        (r.d0, r.d1, r.a0, r.x) = self.r_000704(r.d0);
        (r.d6, r.a6, r.x) = self.r_001c56(r.d4, r.d5, r.d6);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
            self.r_0099e6(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
    }

    /// `loc_01151A`, a task's routine: screen 7: cleared, its list
    /// loaded, a task `func_01159A` (its word 0x2D0), its music,
    /// direction 0xD, the scroll at 0, the objects put (not walked) in
    /// a row by the number of objects + 4 (`func_011A72`,
    /// `func_011B62`), then `func_00A506`.
    pub fn r_01151a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(CLEARED_80DF, 0);
        self.p_clear(r);
        self.write8(SCREEN, 7);
        if self.p_load(r, LIST_7) {
            return;
        }
        r.a0 = SCREEN_7_TASK;
        r.d0 = u32::from(TASK_PRIORITY);
        (r.d6, r.a6, r.x, r.c) = self.r_000404(r.d0, r.d4, r.d5, r.d6, r.a0);
        self.write16(r.a6 + TASK_WORD, 0x2D0);
        r.d0 = low_byte(r.d0, SOUND_7);
        self.r_0034b6(r.d0);
        self.write8(DIRECTION, 0x0D);
        self.p_scroll(r, 0, 0);
        self.write8(WALK, 0);
        let number = self.read8(OBJECTS).wrapping_add(4);
        (r.d0, r.c) = self.r_011a72(low_byte(r.d0, number));
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6) =
            self.r_011b62(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5, r.a6);
        self.p_show(r);
    }

    /// `func_01159A`, screen 7's task: moved left (`VELOCITY_X`,
    /// `func_001A88`) until its word runs out or a button is down;
    /// then screen 5 (`func_0115C2`).
    pub fn r_01159a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (c, stop) = self.r_001bb4();
        if stop {
            r.c = c;
            return;
        }
        if !self.p_buttons(r, Self::r_0102b6, BUTTONS) {
            self.write16(VELOCITY_X, 0xFF00);
            (r.d0, r.d1, r.x, _, _) = self.r_001a88(r.d0, r.d1, r.a6);
            r.c = false;
            if self.read16(r.a6 + TASK_WORD) != 0 {
                return;
            }
        }
        (r.d0, r.d1, r.a1, r.x, r.c) = self.r_0115c2(r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `func_0115C2`: on to screen 5 (`p_leave`).  Gives back d0, d1,
    /// a1, X and C.
    pub fn r_0115c2(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, bool, bool) {
        let mut r = Regs { d2, d3, d4, d5, d6, d7, a2, a3, a4, a5, a6, ..Regs::default() };
        self.p_leave(&mut r, SCREEN_5);
        (r.d0, r.d1, r.a1, r.x, r.c)
    }

    /// `func_0116E4`: the objects' marks (`func_01174C`): one by one,
    /// each object's from the plane at 0xE40E, two columns apart; by
    /// side, the first and the last of the objects `func_00B126` sorts,
    /// at 0xE48E and 0xE88E.  Gives back d0, d1, d3, d4, a0 and a4.
    pub fn r_0116e4(&mut self, d0: u32, d1: u32, d3: u32, d4: u32, a0: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32) {
        if self.read8(FLAGS) & BY_SIDE != 0 {
            self.r_00b126(d1, d4);
            let d4 = low_byte(d4, self.read8(SORTED));
            let (_, a4, _) = self.r_001cfa(d4);
            self.write16(AT, 0xE48E);
            let (d0, d1, _, a6) = self.r_01174c(d0, d1, d3, a0, a4, a6);
            self.write16(AT, 0xE88E);
            let objects = self.read8(OBJECTS);
            let d0 = low_word(d0, objects as i8 as u16);
            let offset = (i32::from(objects as i8) - 1) as u32;
            let d4 = low_byte(d4, self.read8(SORTED.wrapping_add(offset)));
            let (_, a4, _) = self.r_001cfa(d4);
            let (d0, d1, a0, _) = self.r_01174c(d0, d1, d3, SORTED, a4, a6);
            return (d0, d1, d3, d4, a0, a4);
        }
        let (mut d0, mut d1, mut d3, mut d4, mut a0, mut a6) = (d0, d1, d3, d4, a0, a6);
        self.write16(AT, 0xE40E);
        d3 = low_word(d3, 0);
        loop {
            d4 = low_word(d4, d3 as u16);
            let (_, a4, _) = self.r_001cfa(d4);
            (d0, d1, a0, a6) = self.r_01174c(d0, d1, d3, a0, a4, a6);
            self.write8(AT_LOW, 0x0E);
            let at = self.read8(AT).wrapping_add(2);
            self.write8(AT, at);
            let index = (d3 as u8).wrapping_add(1);
            d3 = low_byte(d3, index);
            if index == self.read8(OBJECTS) {
                return (d0, d1, d3, d4, a0, a4);
            }
        }
    }

    /// `func_01174C`: as many marks (3 x 4 blocks, `func_0101C0`) as
    /// the object's (a4) flags' bits 0-2 say, 8 columns apart from
    /// `AT_LOW`, their tiles from 0x5E (0x3A by side).  Gives back d0,
    /// d1, a0 and a6.
    pub fn r_01174c(&mut self, d0: u32, d1: u32, _d3: u32, a0: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32) {
        let marks = u16::from(self.read8(a4 + OBJ_FLAGS)) & 7;
        let (mut d0, mut d1, mut a0, mut a6) = (low_word(d0, marks), d1, a0, a6);
        for _ in 0..marks {
            self.write16(TILE, 0x5E);
            if self.read8(FLAGS) & BY_SIDE != 0 {
                self.write8(TILE_LOW, 0x3A);
            }
            d1 = low_word(d1, MARK_BLOCK);
            (d0, d1, a0, a6, _) = self.r_0101c0(d0, d1, a0, a6);
            let at = self.read8(AT_LOW).wrapping_add(8);
            self.write8(AT_LOW, at);
        }
        (d0, d1, a0, a6)
    }

    /// `func_0118C4`, a task's routine (`loc_0115E8`'s): once the
    /// picker is done (`PICKING`), when its count runs out or a button
    /// is down: screen 8 (`func_011920`) when the pick is `NUMBER`,
    /// else screen 5.
    pub fn r_0118c4(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.c = false;
        if self.read8(PICKING) != 0 {
            return;
        }
        let (c, stop) = self.r_001bb4();
        if stop {
            r.c = c;
            return;
        }
        if !self.p_buttons(r, Self::r_0102b6, BUTTONS) {
            let count = self.read8(r.a6 + TASK_COUNT);
            self.write8(r.a6 + TASK_COUNT, count.wrapping_sub(1));
            r.x = count == 0;
            r.c = r.x;
            if count != 1 {
                return;
            }
        }
        let pick = self.read8(PICK);
        r.d0 = low_byte(r.d0, pick);
        let next = if pick == self.read8(NUMBER) { SCREEN_8 } else { SCREEN_5 };
        self.p_leave(r, next);
    }

    /// `func_011920`, a task's routine: screen 8: cleared, its music,
    /// the scroll at 0, the objects in their places (`func_011BC0`),
    /// its list loaded, a task `func_0119FA`, `func_00A506`, and six
    /// texts (`func_00AA00`).
    pub fn r_011920(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.p_clear(r);
        self.write8(SCREEN, 8);
        r.d0 = low_byte(r.d0, SOUND_8);
        self.r_0034b6(r.d0);
        self.write16(H_VALUE, 0);
        self.write32(CLEARED_80E6, 0);
        let mut copy = *r;
        self.r_008664(&mut copy);
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.a0, r.a4, r.a5, r.x) =
            self.r_011bc0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5, r.a6);
        if self.p_load(r, LIST_8) {
            return;
        }
        (r.d6, r.a6, r.x, r.c) = self.r_0003e8(r.d4, r.d5, r.d6, TASK_PRIORITY, SCREEN_8_TASK);
        self.p_show(r);
        const TEXTS: [(u32, u16); 6] =
            [(0x0D9152, 0x68), (0x0D915A, 0x69), (0x0D9162, 0x6A), (0x0D916C, 0x6C), (0x0D9174, 0x6D), (0x0D917C, 0x6E)];
        for (text, at) in TEXTS {
            self.write32(TEXT, text);
            self.write16(AT, at);
            (r.d0, r.a0, r.x, r.c) = self.r_00aa00(r.d4, r.d5, r.d6, r.a0);
        }
    }

    /// `func_0119FA`, screen 8's task: when a button is down, screen
    /// 8's end (`func_011A38`) after the screen.
    pub fn r_0119fa(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let (c, stop) = self.r_001bb4();
        if stop {
            r.c = c;
            return;
        }
        r.c = false;
        if !self.p_buttons(r, Self::r_0102e8, BUTTONS_8) {
            return;
        }
        self.write32(r.a6 + TASK_KEPT, SCREEN_8_DONE);
        self.r_00053e(r.d6, WAIT);
        r.d0 = 4;
        (r.d0, r.a1, r.x, r.c) = self.r_00a51a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a2, r.a3, r.a4, r.a5);
        r.d1 = STOP_ARG;
        r.d0 = low_byte(r.d0, SOUND_STOP);
        r.c = false;
        self.r_00349e(r.d0);
    }
}
