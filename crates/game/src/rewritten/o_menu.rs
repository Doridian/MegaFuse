//! A menu of pages (`PAGE`, 0-4): the screen set up (`func_01052E`),
//! each page's text lists (D_0105D2, drawn by `func_0094D8` from the
//! list at `TEXT`) and its setup routine (D_0105E6: `func_010662`,
//! `func_0106EC`, `func_010764`, `func_0107E0`, `func_010844`), the
//! tasks they make (`o_menu_tasks.rs`), and bits of the first five
//! objects' `OBJ_FLAGS` cleared (`func_010B38`..`func_010B8C`).

use crate::{Game, Regs};

/// The page shown (an index into `TEXTS` and `SETUPS`).
const PAGE: u32 = 0xFFAD3A;
/// The line chosen on page 2-3 (1 on; 0 for none: `o_tiles.rs`).
const LINE: u32 = 0xFFAD3B;
/// A number added to the first byte of `func_010846`'s lists (also
/// `f_records.rs`'s `LIST_INDEX`).
const LIST_INDEX: u32 = 0xFFAD3F;
/// The text list `func_0094D8` draws (a long).
const TEXT: u32 = 0xFF92FA;
/// Settings stepped on the pages: a count (2-4: `func_0106B2`'s
/// entry), another (1-5: `func_010718`'s), and bits (bit 1 keeps
/// `func_010B38` from clearing).
const COUNT: u32 = 0xFF9541;
const COUNT_2: u32 = 0xFF9542;
const SETTINGS: u32 = 0xFF9540;
/// The texts by `COUNT` (bytes from D_0106D0 - 2: offsets from
/// `COUNT_TEXTS`) and by `COUNT_2` (from D_010736 - 1: offsets from
/// `COUNT_2_TEXTS`).
const COUNT_OFFSETS: u32 = 0x0106CE;
const COUNT_TEXTS: u32 = 0x0106D0;
const COUNT_2_OFFSETS: u32 = 0x010735;
const COUNT_2_TEXTS: u32 = 0x010736;
/// `func_0107E0`'s texts by `COUNT` (longs from D_01082C, so from
/// 0x10834 for the counts 2-4).
const LINES_TEXTS: u32 = 0x01082C;
/// `func_010846`'s two lists (5 bytes each, copied to `LIST` with
/// `LIST_INDEX` added to the first byte), and `func_0108AE`'s.
const LIST_1: u32 = 0x0108A2;
const LIST_2: u32 = 0x0108A8;
const LIST: u32 = 0xFFFF_AD46;
const OTHER_TEXT: u32 = 0x0108BE;
/// A word `func_010846` sets to 0x0EEE between its lists.
const WORD_585E: u32 = 0x00FF_585E;
/// Cleared by `func_01052E`.
const BYTE_A945: u32 = 0xFFA945;

/// The plane's position (words, x and y), where it is as a VRAM offset
/// (a word, `func_0105B0`; its low byte the column part), and the two
/// words `func_010566` copies (x negated).
const PLANE_X: u32 = 0xFF9F04;
const PLANE_Y: u32 = 0xFF9F06;
const PLACE: u32 = 0xFF9ECE;
const PLACE_LOW: u32 = 0xFF9ECF;
const SCROLL_X: u32 = 0xFFD002;
const SCROLL_Y: u32 = 0xFFD402;
const SCROLL_X_COPY: u32 = 0xFF80E6;
const SCROLL_Y_COPY: u32 = 0xFF80EA;

/// The runs of `func_0101C0` (`o_tiles.rs`): where they start, their
/// word and a byte its start is made with.
const START: u32 = 0xFF80C8;
const WORD: u32 = 0xFF80D0;
const BYTE_80CD: u32 = 0xFF80CD;

/// A task's fields: its position (words, x and y), a count, the page
/// it belongs to (or a number), its animation.
const TASK_X: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_COUNT: u32 = 0x16;
const TASK_PAGE: u32 = 0x18;
const TASK_ANIM: u32 = 0x20;
/// A sprite record's frame, flags and position (x then y, words; as
/// bytes from 8).
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_Y_HIGH: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;
/// The sprite records' flags the menu's tasks get.
const SHOWN: u8 = 0x84;

/// The tasks of the pages: routines (`o_menu_tasks.rs`), priority and
/// depths; the arrows' animation.
const ARROW_LEFT: u32 = 0x010AB8;
const ARROW_RIGHT: u32 = 0x010ADE;
const ARROW_2: u32 = 0x010C7C;
const POINTER: u32 = 0x010E20;
const PAGE_MARK: u32 = 0x010974;
const PAGE_BAR: u32 = 0x0109AC;
const TASK_PRIORITY: u8 = 0x40;
const ARROW_DEPTH: u8 = 0x90;
const POINTER_DEPTH: u8 = 0x79;
const MARK_DEPTH: u8 = 0xF2;
const BAR_DEPTH: u8 = 0xF1;
const ARROW_ANIM: u32 = 0x0DA4A0;
/// The frame `func_01091C`'s bar gets.
const BAR_FRAME: u8 = 0x0F;

/// The objects' flags `func_010B38`.. clear (`b_control.rs`).
const OBJ_FLAGS: u32 = 0x2D;
/// How many objects they go through.
const OBJECTS: u8 = 5;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_01052E`: the screen cleared and set up for the menu
    /// (`func_0002E0`, `func_00B4C6`, `func_008624`, the tasks and
    /// sprite records afresh: `func_0003DA`, `func_000704`,
    /// `func_001C56`).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_01052e(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, bool) {
        self.write8(BYTE_A945, 0);
        self.r_0002e0();
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, ..Regs::default() };
        self.r_00b4c6(&mut r);
        self.r_008624();
        let (d0, _, d6, _, _) = self.r_0003da(r.d2, r.d3, r.d4, r.d5, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        let (d0, d1, a0, _) = self.r_000704(d0);
        let (d6, a6, x) = self.r_001c56(r.d4, r.d5, d6);
        (d0, d1, d6, a0, a6, x)
    }

    /// `func_010566`, a task's routine: `SCROLL_X` negated and
    /// `SCROLL_Y` copied for the frame.
    pub fn r_010566(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        let x = self.read16(SCROLL_X);
        let negated = x.wrapping_neg();
        r.d0 = low_word(r.d0, negated);
        r.x = x != 0;
        self.write16(SCROLL_X_COPY, negated);
        let y = self.read16(SCROLL_Y);
        self.write16(SCROLL_Y_COPY, y);
        r.c = false;
        r.unwound = false;
    }

    /// `func_0105B0`: `PLACE` from the plane's position: its column
    /// (x / 8, 0-0x3F) and row (y / 8, x 0x40).  X clear.
    pub fn r_0105b0(&mut self, d0: u32) -> (u32, bool) {
        let column = ((self.read16(PLANE_X) >> 3) as u8) & 0x3F;
        self.write8(PLACE_LOW, column);
        let place = ((self.read16(PLANE_Y) & 0x1F8) << 3) | u16::from(column);
        self.write16(PLACE, place);
        (low_word(d0, place), false)
    }

    /// `func_010662`, page 0's setup: two arrows (`ARROW_LEFT` with
    /// frame 5 + 4, `ARROW_RIGHT` with frame 0xFF + 4 and
    /// `ARROW_ANIM`), then `COUNT`'s text (`func_0106B2`).
    pub fn r_010662(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, ARROW_DEPTH, TASK_PRIORITY, ARROW_LEFT);
        (r.d0, r.x) = self.r_0108f2(5, r.a5, r.a6);
        self.write8(r.a6 + TASK_PAGE, 0);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, ARROW_DEPTH, TASK_PRIORITY, ARROW_RIGHT);
        (r.d0, r.x) = self.r_0108f2(low_byte(r.d0, 0xFF), r.a5, r.a6);
        self.write32(r.a6 + TASK_ANIM, ARROW_ANIM);
        self.write8(r.a5 + SPRITE_FLAGS, SHOWN);
        self.write8(r.a6 + TASK_PAGE, 0);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) = self.r_0106b2(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// A text by a setting: `offsets` + the setting gives the offset of
    /// the text from `texts`; drawn (`func_0094D8`).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn o_text_by(&mut self, setting: u32, offsets: u32, texts: u32, r: Regs) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let index = self.read8(setting) as i8 as i32 as u32;
        let offset = self.read8(offsets.wrapping_add(index)) as i8 as i32 as u32;
        let (d0, x) = offset.overflowing_add(texts);
        self.write32(TEXT, d0);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, _) = self.r_0094d8(d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, x);
        (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x)
    }

    /// `func_0106B2`: `COUNT`'s text drawn.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0106b2(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let r = Regs { d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        self.o_text_by(COUNT, COUNT_OFFSETS, COUNT_TEXTS, r)
    }

    /// `func_0106EC`, page 1's setup: an arrow (`ARROW_2`, frame 0xFF +
    /// 4, `ARROW_ANIM`, page 1), then `COUNT_2`'s text
    /// (`func_010718`).
    pub fn r_0106ec(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, ARROW_DEPTH, TASK_PRIORITY, ARROW_2);
        (r.d0, r.x) = self.r_0108f2(low_byte(r.d0, 0xFF), r.a5, r.a6);
        self.write32(r.a6 + TASK_ANIM, ARROW_ANIM);
        self.write8(r.a6 + TASK_PAGE, 1);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) = self.r_010718(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `func_010718`: `COUNT_2`'s text drawn.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_010718(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool) {
        let r = Regs { d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, ..Regs::default() };
        self.o_text_by(COUNT_2, COUNT_2_OFFSETS, COUNT_2_TEXTS, r)
    }

    /// `func_010764`, page 2's setup: 5 runs of 2 words (`WORD`
    /// 0x8200 counted up) from `PLACE`'s row + 3, column + 0x1B (as a
    /// VRAM address: - 0x1000, x 2; `func_0101C0`), the pointer
    /// (`func_0108CA`, page 2), `LINE` 1 and its line read back
    /// (`func_0107B6`).
    pub fn r_010764(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d1 = low_word(r.d1, 0x0502);
        self.write16(WORD, 0x8200);
        let column = (self.read8(PLACE_LOW) & 0x3F).wrapping_add(0x1B);
        self.write8(BYTE_80CD, column);
        let place = (self.read16(PLACE) & 0xFFC0).wrapping_add(0xC0) | u16::from(column);
        let start = place.wrapping_sub(0x1000).wrapping_mul(2);
        self.write16(START, start);
        (r.d0, r.d1, r.a0, r.a6, r.x) = self.r_0101c0(low_word(r.d0, start), r.d1, r.a0, r.a6);
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x) = self.r_0108ca(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.write8(r.a6 + TASK_PAGE, 2);
        self.write8(LINE, 1);
        (r.d0, r.d1, r.a0, r.x) = self.r_0107b6(r.d0, r.d1, r.a0, r.x);
    }

    /// `func_0107E0`, page 3's setup: the pointer (`func_0108CA`, page
    /// 3), `LINE` 1, `func_011CCA`, and the text by `COUNT`
    /// (`LINES_TEXTS`).
    /// No script calls it: reviewed by reading.
    pub fn r_0107e0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.d5, r.d6, r.a5, r.a6, r.x) = self.r_0108ca(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.write8(r.a6 + TASK_PAGE, 3);
        self.write8(LINE, 1);
        (r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6) = self.r_011cca(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5);
        let twice = self.read8(COUNT).wrapping_mul(2);
        let (offset, x) = twice.overflowing_add(twice);
        r.d0 = low_word(r.d0, offset as i8 as i16 as u16);
        let text = self.read32(LINES_TEXTS.wrapping_add(offset as i8 as i32 as u32));
        self.write32(TEXT, text);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_0094d8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, x);
    }

    /// `func_010844`, page 4's setup: nothing.
    pub fn r_010844(&mut self, regs: &mut Regs) {
        regs.unwound = false;
    }

    /// `func_010846`: `LIST_1` and `LIST_2` drawn (copied to `LIST`,
    /// `LIST_INDEX` added to their first byte), `WORD_585E` 0x0EEE and
    /// `func_0094B2` between them.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_010846(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let mut regs = (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6);
        for (i, list) in [LIST_1, LIST_2].into_iter().enumerate() {
            if i == 1 {
                self.write16(WORD_585E, 0x0EEE);
                self.r_0094b2();
            }
            let head = self.read32(list);
            self.write32(LIST, head);
            let end = self.read8(list + 4);
            self.write8(LIST + 4, end);
            let index = self.read8(LIST_INDEX);
            let (first, x) = self.read8(LIST).overflowing_add(index);
            self.write8(LIST, first);
            self.write32(TEXT, LIST);
            let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6) = regs;
            let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, _, _) = self.r_0094d8(low_byte(d0, index), d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x);
            regs = (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6);
            if self.restarted.is_some() {
                break;
            }
        }
        regs
    }

    /// `func_0108AE`: the text at `OTHER_TEXT` drawn.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0108ae(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        self.write32(TEXT, OTHER_TEXT);
        let (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, _, _) = self.r_0094d8(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x);
        (d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6)
    }

    /// `func_0108CA`: the pointer task (`POINTER`, frame 1 + 4), its
    /// sprite record's y 0x8080.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_0108ca(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32, bool) {
        let (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, POINTER_DEPTH, TASK_PRIORITY, POINTER);
        let (d0, x) = self.r_0108f2(1, a5, a6);
        self.write8(a5 + SPRITE_Y_HIGH, 0x80);
        self.write8(a5 + SPRITE_Y_LOW, 0x80);
        (low_byte(d0, 0x80), d5, d6, a5, a6, x)
    }

    /// `func_0108F2`: a menu task set up (`func_00152A`), moved by
    /// (0x80, 0x48), its `TASK_PAGE` d0's low byte, its sprite record's
    /// frame that + 4 and flags `SHOWN`.  Gives back d0 (+ 4) and X.
    pub fn r_0108f2(&mut self, d0: u32, a5: u32, a6: u32) -> (u32, bool) {
        self.r_00152a(a6);
        let x = self.read16(a6 + TASK_X).wrapping_add(0x80);
        self.write16(a6 + TASK_X, x);
        let y = self.read16(a6 + TASK_Y).wrapping_add(0x48);
        self.write16(a6 + TASK_Y, y);
        self.write8(a6 + TASK_PAGE, d0 as u8);
        let (frame, carry) = (d0 as u8).overflowing_add(4);
        self.write8(a5 + SPRITE_FRAME, frame);
        self.write8(a5 + SPRITE_FLAGS, SHOWN);
        (low_byte(d0, frame), carry)
    }

    /// `func_01091C`: the page's mark (`PAGE_MARK`, frame `PAGE` + 4)
    /// and bar (`PAGE_BAR`, frame `BAR_FRAME`, count 0), both of page
    /// `PAGE`.
    #[allow(clippy::too_many_arguments)]
    pub fn r_01091c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32) {
        let (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, MARK_DEPTH, TASK_PRIORITY, PAGE_MARK);
        let page = self.read8(PAGE);
        let d0 = low_byte(d0, page.wrapping_add(4));
        self.write8(a5 + SPRITE_FRAME, d0 as u8);
        self.write8(a6 + TASK_PAGE, page);
        self.write8(a5 + SPRITE_FLAGS, SHOWN);
        let (d5, d6, a5, a6, _, _) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, BAR_DEPTH, TASK_PRIORITY, PAGE_BAR);
        self.write8(a5 + SPRITE_FRAME, BAR_FRAME);
        self.write8(a5 + SPRITE_FLAGS, SHOWN);
        self.write8(a6 + TASK_COUNT, 0);
        let page = self.read8(PAGE);
        self.write8(a6 + TASK_PAGE, page);
        (d5, d6, a5, a6)
    }

    /// `func_010B38`: bit 6 of the first five objects' `OBJ_FLAGS`
    /// cleared, unless bit 1 of `SETTINGS`.
    pub fn r_010b38(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, a4: u32, a5: u32, a6: u32) {
        if self.read8(SETTINGS) & 2 == 0 {
            self.o_each_object(d0, d4, d5, d6, a4, a5, a6, 0x010B58, |flags| flags & !0x40);
        }
    }

    /// `func_010B5C`: the first five objects' `OBJ_FLAGS` but bits 4
    /// and 6 cleared.
    /// No script calls it: reviewed by reading.
    pub fn r_010b5c(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, a4: u32, a5: u32, a6: u32) {
        self.o_each_object(d0, d4, d5, d6, a4, a5, a6, 0x010B72, |flags| flags & 0x50);
    }

    /// `func_010B74`: the first five objects' `OBJ_FLAGS` bits 0-3
    /// cleared.  Gives back d0 (0x10B8A), d4-d6 and a4-a6 as they were,
    /// X clear.
    /// No script calls it: reviewed by reading.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_010b74(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, bool) {
        self.o_each_object(d0, d4, d5, d6, a4, a5, a6, 0x010B8A, |flags| flags & 0xF0)
    }

    /// `func_010B8C`: the first five objects' `OBJ_FLAGS` cleared.
    /// Gives back d0 (0x10BA0), d4-d6 and a4-a6 as they were, X clear.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_010b8c(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, a4: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, bool) {
        self.o_each_object(d0, d4, d5, d6, a4, a5, a6, 0x010BA0, |_| 0)
    }

    /// The loop of `func_010B38`..: `OBJ_FLAGS` of the objects 0-4
    /// changed, `func_010BA2` ending it (from `ret`, the loop's return
    /// address, which it leaves in d0).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn o_each_object(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, a4: u32, a5: u32, a6: u32, ret: u32, change: impl Fn(u8) -> u8) -> (u32, u32, u32, u32, u32, u32, u32, bool) {
        let kept = (d4, d5, d6, a4, a5, a6);
        let (mut d0, mut d4, mut d5, mut d6, mut a5, mut a6) = (d0, low_word(d4, 0), d5, d6, a5, a6);
        let mut a4;
        loop {
            (d4, a4, _) = self.r_001cfa(d4);
            let flags = self.read8(a4 + OBJ_FLAGS);
            self.write8(a4 + OBJ_FLAGS, change(flags));
            let (n0, n4, n5, n6, m4, m5, m6, x, done) = self.r_010ba2(d0, d4, d5, d6, a4, a5, a6, kept.0, kept.1, kept.2, kept.3, kept.4, kept.5, ret);
            if done {
                return (n0, n4, n5, n6, m4, m5, m6, x);
            }
            (d0, d4, d5, d6, a5, a6) = (n0, n4, n5, n6, m5, m6);
        }
    }

    /// `func_010BA2`: the next object (d4's low byte one more); after
    /// object 4 the loop that calls it ends (from its caller: d0 the
    /// return address `ret`, d4-d6 and a4-a6 the ones kept at its
    /// start).  Gives back X (the add's carry) and whether it ended.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_010ba2(&mut self, d0: u32, d4: u32, d5: u32, d6: u32, a4: u32, a5: u32, a6: u32, arg0: u32, arg4: u32, arg8: u32, arg12: u32, arg16: u32, arg20: u32, ret: u32) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (next, x) = (d4 as u8).overflowing_add(1);
        let d4 = low_byte(d4, next);
        if next != OBJECTS {
            return (d0, d4, d5, d6, a4, a5, a6, x, false);
        }
        (ret, arg0, arg4, arg8, arg12, arg16, arg20, x, true)
    }
}
