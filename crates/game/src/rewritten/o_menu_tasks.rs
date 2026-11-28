//! The tasks of the menu's pages (`o_menu.rs`), run by the scheduler:
//! the page's mark and bar, the arrows, the pointer, and the tasks that
//! read the pads (`o_input.rs`'s `INPUT`) on each page: page 0
//! (`func_010A18`) and 1 (`func_010BCE`) step settings, page 2 (the
//! lines, `func_010CC6`) and its cell chooser (`func_010EA2`), page 3
//! (`func_010FEE`).  While `LEAVING` is set the tasks of other pages
//! than `PAGE` end (`func_0109FA`).  The routines leaving a page
//! (`func_010A9A`, `func_010C04`, `loc_01109A`) wait for the frame.

use crate::{Game, Regs};

/// The page shown, the line chosen (1 on), and set while the tasks of
/// other pages are to end.
const PAGE: u32 = 0xFFAD3A;
const LINE: u32 = 0xFFAD3B;
const LEAVING: u32 = 0xFFAD3C;
/// Cleared by page 0's task (and counted by `func_010A7E`).
const BYTE_AD40: u32 = 0xFFAD40;
/// How many page changes are going on (`func_010BB6`'s waits).
const CHANGING: u32 = 0xFFAD44;
/// The menus' input (`o_input.rs`): bit 4 of its high byte goes back,
/// bits 5-7 go on (bit 7 at once in `func_010BB6`), bits 0-3 the
/// directions.
const INPUT: u32 = 0xFFAD02;
const BACK: u8 = 1 << 4;
const ON: u8 = 0xE0;
const ON_NOW: u8 = 1 << 7;
const DIRECTIONS_UP_DOWN: u8 = 0x03;
/// Settings: `COUNT` (2-4), `COUNT_2` (1-5), and bits (bit 1: page 2
/// goes on to page 3; it picks the arrows' animation).
const COUNT: u32 = 0xFF9541;
const COUNT_2: u32 = 0xFF9542;
const SETTINGS: u32 = 0xFF9540;
/// The settings' lists `func_01031A` steps (page 0's, page 1's,
/// page 2's from `LINES_LIST` and page 3's from `LINES_LIST_3`, copied
/// to `LIST` with `COUNT` as the line's highest; the cell chooser's).
const PAGE_0_LIST: u32 = 0x010B28;
const PAGE_1_LIST: u32 = 0x010C6C;
const LINES_LIST: u32 = 0x010E92;
const LINES_LIST_3: u32 = 0x01110E;
const LIST: u32 = 0xFFFF_AD46;
const LIST_END: u32 = 0xFFFF_AD56;
const LIST_LINE_MAX: u32 = 0xFFAD4F;
const CELL_LIST: u32 = 0x010FA6;
/// A line's object's flag kept across `func_01031A` (its bit 4 on page
/// 2, bit 6 on page 3) and the object's index.
const KEPT_FLAG: u32 = 0xFFAC90;
const KEPT_INDEX: u32 = 0xFFAC91;
/// The cell chosen (0-0x11; `func_010EA2`).
const CELL: u32 = 0xFF9E9C;
/// The sprite record of the routine being run (index, address).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// The running task's index.
const CURRENT_TASK: u32 = 0xFF80EE;

/// Z80 commands: a tick, a step, back, on, the line, can't.
const TICK: u8 = 0xFD;
const STEP_SOUND: u8 = 0x20;
const ON_SOUND: u8 = 0x21;
const BACK_SOUND: u8 = 0x22;
const LINE_SOUND: u8 = 0x23;
const NO_SOUND: u8 = 0x31;

/// A task's fields: its animation's frames to go and step, its y's low
/// byte, a count, its page (or a number), the routine `loc_000558`
/// gives it back, its animation and its second byte.
const TASK_TIMER: u32 = 0x12;
const TASK_STEP: u32 = 0x13;
const TASK_Y_LOW: u32 = 0x11;
const TASK_COUNT: u32 = 0x16;
const TASK_PAGE: u32 = 0x18;
const TASK_NEXT: u32 = 0x1C;
const TASK_ANIM_1: u32 = 0x21;
/// A sprite record's frame, its position (x then y, words; bytes 8 on)
/// and its flags.
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_X_HIGH: u32 = 0x08;
const SPRITE_X_LOW: u32 = 0x09;
const SPRITE_Y_HIGH: u32 = 0x0A;
const SPRITE_Y_LOW: u32 = 0x0B;

/// An object's fields: its sprite record's address and index, its
/// animations (a long), the cell it is given (`CELL`); and, from the
/// object after it (- 0x40), its sprite record's index, `CELL` and
/// flags.
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_ANIMS: u32 = 0x1C;
const OBJ_CELL: u32 = 0x2C;
const BEFORE_SPRITE: u32 = 0xFFFF_FFCE;
const BEFORE_CELL: u32 = 0xFFFF_FFEC;
const BEFORE_FLAGS: u32 = 0xFFFF_FFED;
/// The animations by cell (longs by the cell x 2, D_00B534).
const ANIMS_BY_CELL: u32 = 0x00B534;

/// The routines the tasks go on with.
const PAGE_0: u32 = 0x010A18;
const PAGE_0_ON: u32 = 0x010A7E;
const PAGE_1: u32 = 0x010BCE;
const PAGE_1_ON: u32 = 0x010C54;
const PAGE_2: u32 = 0x010CC6;
const PAGE_3: u32 = 0x010FEE;
const WAIT: u32 = 0x010BB6;
const WAIT_FRAMES: u8 = 0x1C;
const CHOOSER: u32 = 0x010EA2;
const CELL_MARK: u32 = 0x010FB0;
const CELL_MARK_PRIORITY: u8 = 0x50;
const CELL_MARK_DEPTH: u8 = 0x80;
const CELL_MARK_FRAME: u8 = 0x0A;
const SHOWN: u8 = 0x84;
/// The arrows' animations' second byte by bit 1 of `SETTINGS`
/// (D_010ADC), their y by `COUNT` (D_010CC0) and by `COUNT_2` (from
/// D_010CC0 - 1).
const ARROW_ANIMS: u32 = 0x010ADC;
const ARROW_Y: u32 = 0x010CC0;
const ARROW_2_Y: u32 = 0x010CBF;
/// The cell mark's position by `CELL` / 2 (x D_010FDA, y D_010FE4).
const CELL_X: u32 = 0x010FDA;
const CELL_Y: u32 = 0x010FE4;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    fn o_sound(&mut self, r: &mut Regs, command: u8) {
        r.d0 = low_byte(r.d0, command);
        self.r_0034b6(r.d0);
    }

    fn o_menu_slot(&mut self, r: &mut Regs) {
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        r.a5 = self.read32(SLOT_ADDR);
    }

    /// `func_0109FA` on the routines' registers: true if the task ended.
    fn o_leaves(&mut self, r: &mut Regs, ret: u32) -> bool {
        let ended;
        (r.d0, r.d6, r.x, r.c, ended) = self.r_0109fa(r.d0, r.d5, r.d6, r.a6, r.x, ret);
        ended
    }

    /// `func_010974`, the page's mark: its frame `PAGE` + 4 when
    /// `LEAVING` (its page then `PAGE`); placed (`func_010996`).
    pub fn r_010974(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let page = self.read8(r.a6 + TASK_PAGE);
        r.d0 = low_byte(r.d0, page);
        if page != self.read8(PAGE) {
            let leaving = self.read8(LEAVING);
            r.d0 = low_byte(r.d0, leaving);
            if leaving != 0 {
                let page = self.read8(PAGE);
                self.write8(r.a6 + TASK_PAGE, page);
                let (frame, carry) = page.overflowing_add(4);
                r.d0 = low_byte(r.d0, frame);
                r.x = carry;
                self.write8(r.a5 + SPRITE_FRAME, frame);
            }
        }
        r.c = self.r_010996(r.a5);
    }

    /// `func_010996`: the sprite record at a5 at (0x80, 0x56).  C clear.
    pub fn r_010996(&mut self, a5: u32) -> bool {
        self.write8(a5 + SPRITE_Y_LOW, 0x56);
        self.write8(a5 + SPRITE_X_LOW, 0x80);
        self.write8(a5 + SPRITE_X_HIGH, 0);
        self.write8(a5 + SPRITE_Y_HIGH, 0);
        false
    }

    /// `func_0109AC`, the page's bar: its count grows by 2 to 0x20
    /// while its page is `PAGE` (it takes `PAGE` when `LEAVING`), else
    /// shrinks by 2 to 0; placed (`func_010996`) that much lower or
    /// higher.
    pub fn r_0109ac(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(LEAVING) != 0 {
            let page = self.read8(PAGE);
            self.write8(r.a6 + TASK_PAGE, page);
        }
        let page = self.read8(r.a6 + TASK_PAGE);
        r.d0 = low_byte(r.d0, page);
        let count = self.read8(r.a6 + TASK_COUNT);
        r.d0 = low_byte(r.d0, count);
        r.c = false;
        if page == self.read8(PAGE) {
            if count == 0x20 {
                return;
            }
            let count = count.wrapping_add(2);
            self.write8(r.a6 + TASK_COUNT, count);
            self.r_010996(r.a5);
            r.d0 = low_byte(r.d0, count);
            let (y, carry) = self.read8(r.a5 + SPRITE_Y_LOW).overflowing_add(count);
            self.write8(r.a5 + SPRITE_Y_LOW, y);
            (r.x, r.c) = (carry, carry);
        } else {
            if count == 0 {
                return;
            }
            let count = count.wrapping_sub(2);
            self.write8(r.a6 + TASK_COUNT, count);
            self.r_010996(r.a5);
            r.d0 = low_byte(r.d0, count);
            let (y, borrow) = self.read8(r.a5 + SPRITE_Y_LOW).overflowing_sub(count);
            self.write8(r.a5 + SPRITE_Y_LOW, y);
            (r.x, r.c) = (borrow, borrow);
        }
    }

    /// `func_0109FA`: while `LEAVING`, a task whose page isn't `PAGE`
    /// ends (`func_00121A`), and its caller with it: d0 the caller's
    /// return address `ret`, and true.
    pub fn r_0109fa(&mut self, d0: u32, d5: u32, d6: u32, a6: u32, x: bool, ret: u32) -> (u32, u32, bool, bool, bool) {
        let leaving = self.read8(LEAVING);
        let d0 = low_byte(d0, leaving);
        if leaving == 0 {
            return (d0, d6, x, false, false);
        }
        let page = self.read8(PAGE);
        let d0 = low_byte(d0, page);
        let own = self.read8(a6 + TASK_PAGE);
        if page == own {
            return (d0, d6, x, false, false);
        }
        let (d6, x, c) = self.r_00121a(d5, d6, x);
        (ret, d6, x, c, true)
    }

    /// `func_010A18`, page 0's task: unless the objects are stopped
    /// (`func_001BB4`), the Z80 sent `TICK`, `COUNT` and bit 1 of
    /// `SETTINGS` stepped (`PAGE_0_LIST`; when one changes its text is
    /// drawn, `func_010B38`, `STEP_SOUND`); going on (`ON`), the task
    /// waits (`func_010BB6`) then goes to the next page
    /// (`func_010A7E`).
    pub fn r_010a18(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(BYTE_AD40, 0);
        self.write8(LEAVING, 0);
        let stopped;
        (r.c, stopped) = self.r_001bb4();
        if stopped {
            return;
        }
        r.d0 = low_byte(r.d0, TICK);
        self.r_00349e(r.d0);
        let changed;
        (r.x, changed) = self.r_01031a(r.a1, PAGE_0_LIST);
        if changed {
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) = self.r_0106b2(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            if self.restarted.is_some() {
                return;
            }
            self.r_010b38(r.d0, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6);
            self.o_sound(r, STEP_SOUND);
        }
        self.o_go_on(r, PAGE_0_ON);
    }

    /// Going on (`ON` in `INPUT`): the task waits `WAIT_FRAMES`
    /// (`func_010BB6`), then runs `next`; `CHANGING` one more.
    fn o_go_on(&mut self, r: &mut Regs, next: u32) {
        let on = self.read8(INPUT) & ON;
        r.d0 = low_byte(r.d0, on);
        r.c = false;
        if on == 0 {
            return;
        }
        self.write32(r.a6 + TASK_NEXT, next);
        self.write8(r.a6 + TASK_COUNT, WAIT_FRAMES);
        self.r_00053e(r.d6, WAIT);
        let (changing, carry) = self.read8(CHANGING).overflowing_add(1);
        self.write8(CHANGING, changing);
        (r.x, r.c) = (carry, carry);
    }

    /// `func_010AB8`, the left arrow: its animation's second byte by
    /// bit 1 of `SETTINGS` (`ARROW_ANIMS`), drawn (`loc_001BA4`).
    pub fn r_010ab8(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.o_leaves(r, 0x010ABE) {
            return;
        }
        let index = (self.read8(SETTINGS) & 2) >> 1;
        r.d0 = low_word(r.d0, u16::from(index));
        r.x = false;
        let anim = self.read8(ARROW_ANIMS + u32::from(index));
        self.write8(r.a6 + TASK_ANIM_1, anim);
        self.o_menu_slot(r);
        (r.d0, r.x, r.c) = self.r_001ba4(r.a5, r.a6);
    }

    /// An arrow animated while a page changes (`ON_SOUND` at its step
    /// 8), its y by a setting (`table` from the setting), drawn
    /// (`func_001B7E`).
    fn o_arrow(&mut self, r: &mut Regs, setting: u32, table: u32) {
        let changing = self.read8(CHANGING);
        r.d0 = low_byte(r.d0, changing);
        if changing != 0 {
            (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0013d6(r.d0, r.d1, r.d2, r.d5, r.a6);
            if self.read8(r.a6 + TASK_TIMER) == 0 && self.read8(r.a6 + TASK_STEP) == 8 {
                self.o_sound(r, ON_SOUND);
            }
        }
        let index = self.read8(setting) as i8 as i16 as u16;
        r.d0 = low_word(r.d0, index);
        let y = self.read8(table.wrapping_add(index as i16 as u32));
        self.write8(r.a6 + TASK_Y_LOW, y);
        self.o_menu_slot(r);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
    }

    /// `func_010ADE`, the right arrow on page 0: y by `COUNT`.
    pub fn r_010ade(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.o_leaves(r, 0x010AE4) {
            return;
        }
        self.o_arrow(r, COUNT, ARROW_Y);
        r.a0 = ARROW_Y;
    }

    /// `func_010BB6`, the wait before a page changes: `WAIT_FRAMES`
    /// frames (`TASK_COUNT`), or none with `ON_NOW`; then the task's
    /// kept routine (`loc_000558`).
    pub fn r_010bb6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(INPUT) & ON_NOW == 0 {
            let count = self.read8(r.a6 + TASK_COUNT);
            self.write8(r.a6 + TASK_COUNT, count.wrapping_sub(1));
            r.x = count == 0;
            r.c = r.x;
            if count != 1 {
                return;
            }
        }
        r.c = self.r_000558(r.a6);
    }

    /// `func_010BCE`, page 1's task: `COUNT_2` and `DIRECTION` stepped
    /// (`PAGE_1_LIST`; when one changes, the text and `STEP_SOUND`);
    /// back (`BACK`) to page 0 (`func_010C04`), or on (`ON`) after a
    /// wait to the next page (`func_010C54`).
    pub fn r_010bce(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(LEAVING, 0);
        let changed;
        (r.x, changed) = self.r_01031a(r.a1, PAGE_1_LIST);
        r.c = changed;
        if changed {
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) = self.r_010718(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            if self.restarted.is_some() {
                return;
            }
            self.o_sound(r, STEP_SOUND);
        }
        if self.read8(INPUT) & BACK != 0 {
            self.write32(r.a6 + TASK_NEXT, PAGE_0);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_010c04(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            return;
        }
        self.o_go_on(r, PAGE_1_ON);
    }

    /// `func_010C7C`, the arrow on page 1: y by `COUNT_2`.
    pub fn r_010c7c(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.o_leaves(r, 0x010C82) {
            return;
        }
        self.o_arrow(r, COUNT_2, ARROW_2_Y);
    }

    /// The lines' list (`from`) copied to `LIST`, its line's highest
    /// `COUNT`.  d0, a0, a1 as the copy leaves them.
    fn o_lines_list(&mut self, r: &mut Regs, from: u32) {
        for i in 0..4 {
            let long = self.read32(from + 4 * i);
            self.write32(LIST + 4 * i, long);
        }
        r.a0 = from + 0x10;
        r.a1 = LIST_END;
        r.d0 = low_word(r.d0, 0xFFFF);
        let count = self.read8(COUNT);
        self.write8(LIST_LINE_MAX, count);
    }

    /// The object of the line `LINE` (`func_001CFA`; its flags are the
    /// object before's, `BEFORE_FLAGS`).
    fn o_line_object(&mut self, r: &mut Regs, index: u8) {
        (r.d4, r.a4, r.x) = self.r_001cfa(low_byte(r.d4, index));
    }

    /// `func_010CC6`, page 2's task: `LINE` (0-`COUNT`) and the
    /// direction `DIRECTION` stepped (`LINES_LIST`), bit 4 of the line's
    /// object's flags kept; when one changes, the line is read back
    /// (`func_0107B6`) with `LINE_SOUND` (up, down) or `STEP_SOUND`.
    /// Back (`BACK`): page 1 (`func_010C04`).  On (`ON`): with a line
    /// and bit 5 or 6, the cell chooser (`func_010EA2`, its mark
    /// `func_010FB0`) from the line's object's cell; else page 3 with
    /// bit 1 of `SETTINGS` (`func_010A92`), or the next page
    /// (`loc_01109A`).
    pub fn r_010cc6(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(LEAVING, 0);
        self.write8(CHANGING, 0);
        self.o_lines_list(r, LINES_LIST);
        let line = self.read8(LINE);
        self.o_line_object(r, line);
        let index = r.d4 as u8;
        self.write8(KEPT_INDEX, index);
        if index != 0 {
            let flag = self.read8(r.a4.wrapping_add(BEFORE_FLAGS)) & 0x10;
            r.d0 = low_byte(r.d0, flag);
            self.write8(KEPT_FLAG, flag);
        }
        let (x, changed) = self.r_01031a(r.a1, LIST);
        let index = self.read8(KEPT_INDEX);
        self.o_line_object(r, index);
        if index != 0 {
            let flags = (self.read8(r.a4.wrapping_add(BEFORE_FLAGS)) & 0xEF) | self.read8(KEPT_FLAG);
            r.d0 = low_byte(r.d0, flags);
            self.write8(r.a4.wrapping_add(BEFORE_FLAGS), flags);
        }
        (r.x, r.c) = (x, changed);
        if changed {
            (r.d0, r.d1, r.a0, r.x) = self.r_0107b6(r.d0, r.d1, r.a0, r.x);
            let sound = if self.read8(INPUT) & DIRECTIONS_UP_DOWN != 0 { LINE_SOUND } else { STEP_SOUND };
            self.o_sound(r, sound);
        }
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
        let input = self.read8(INPUT);
        r.d0 = low_byte(r.d0, input);
        if input & BACK != 0 {
            self.write32(r.a6 + TASK_NEXT, PAGE_1);
            self.write8(LINE, 0);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_010c04(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            if self.restarted.is_none() {
                r.c = self.r_011a82();
            }
            return;
        }
        r.d0 = low_byte(r.d0, input & ON);
        r.c = false;
        if input & ON == 0 {
            return;
        }
        if self.read8(LINE) != 0 && self.read8(INPUT) & 0x60 != 0 {
            self.o_choose_cell(r);
            return;
        }
        if self.read8(SETTINGS) & 2 != 0 {
            self.write32(r.a6 + TASK_NEXT, PAGE_3);
            self.write8(LINE, 0);
            r.c = false;
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x) = self.r_010a92(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            if self.restarted.is_none() {
                r.c = self.r_011a82();
            }
            return;
        }
        self.r_011a82();
        let (page, carry) = self.read8(PAGE).overflowing_add(1);
        self.write8(PAGE, page);
        r.x = carry;
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_01109a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
    }

    /// `func_010CC6`'s cell chooser: `CELL` from the line's object, the
    /// task's routine `func_010EA2`, `ON_SOUND`, and the cell's mark
    /// (`func_010FB0`, run at once).
    fn o_choose_cell(&mut self, r: &mut Regs) {
        let line = self.read8(LINE);
        self.o_line_object(r, line);
        let cell = self.read8(r.a4.wrapping_add(BEFORE_CELL));
        self.write8(CELL, cell);
        (r.x, r.c) = self.r_00053e(r.d6, CHOOSER);
        self.o_sound(r, ON_SOUND);
        (r.d5, r.d6, r.a5, r.a6, r.x, r.c) = self.r_001222(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, CELL_MARK_DEPTH, CELL_MARK_PRIORITY, CELL_MARK);
        self.write8(r.a5 + SPRITE_FRAME, CELL_MARK_FRAME);
        self.write8(r.a5 + SPRITE_FLAGS, SHOWN);
        r.c = false;
        self.r_010fb0(r);
    }

    /// `func_010E20`, the pointer: hidden unless its page is `PAGE`
    /// (`func_010E64`); with a line chosen, frame 0x11 at the x of the
    /// line's object's sprite record, else frame 0x10 at x 0xD8.
    pub fn r_010e20(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.o_leaves(r, 0x010E26) {
            return;
        }
        r.d0 = self.r_010e64(r.d0, r.a5, r.a6);
        self.write8(r.a5 + SPRITE_X_HIGH, 0);
        let line = self.read8(LINE);
        r.c = false;
        if line == 0 {
            self.write8(r.a5 + SPRITE_X_LOW, 0xD8);
            self.write8(r.a5 + SPRITE_FRAME, 0x10);
            return;
        }
        self.write8(r.a5 + SPRITE_FRAME, 0x11);
        self.o_line_object(r, line);
        let sprite = self.read8(r.a4.wrapping_add(BEFORE_SPRITE));
        (r.d6, r.a6, r.x) = self.r_000b48(low_byte(r.d6, sprite));
        let x = self.read8(r.a6 + SPRITE_X_LOW);
        self.write8(r.a5 + SPRITE_X_LOW, x);
    }

    /// `func_010E64`: a task's sprite record (a5) at y 0x8080 unless the
    /// task's page is `PAGE`; then at y 0x20 when `LEAVING`.
    pub fn r_010e64(&mut self, d0: u32, a5: u32, a6: u32) -> u32 {
        let page = self.read8(PAGE);
        let d0 = low_byte(d0, page);
        if page != self.read8(a6 + TASK_PAGE) {
            self.write8(a5 + SPRITE_Y_HIGH, 0x80);
            self.write8(a5 + SPRITE_Y_LOW, 0x80);
            return low_byte(d0, 0x80);
        }
        let leaving = self.read8(LEAVING);
        let d0 = low_byte(d0, leaving);
        if leaving != 0 {
            self.write8(a5 + SPRITE_Y_HIGH, 0);
            self.write8(a5 + SPRITE_Y_LOW, 0x20);
        }
        d0
    }

    /// `func_010EA2`, the cell chooser: `CELL` stepped by 2 (0-0x10;
    /// `CELL_LIST`, `STEP_SOUND`), or by 10 round 18 with up and down
    /// (0x10 in place of 0 going down).  Back (`BACK`): `BACK_SOUND`,
    /// page 2 again.  On: page 2 again, `ON_SOUND`, the line's object
    /// given the cell (`OBJ_CELL`) and its animations by the cell
    /// (`ANIMS_BY_CELL`), its animation from them (`func_00130E`,
    /// `func_001E86`, `func_001336`), its sprite record's bit 4 set, and
    /// `func_0094B2`.
    pub fn r_010ea2(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let changed;
        (r.x, changed) = self.r_01031a(r.a1, CELL_LIST);
        if changed {
            r.c = false;
            self.o_sound(r, STEP_SOUND);
            return;
        }
        let input = self.read8(INPUT);
        r.d0 = low_byte(r.d0, input & 0xF0);
        if input & 0xF0 == 0 {
            r.d0 = low_byte(r.d0, input & DIRECTIONS_UP_DOWN);
            r.c = false;
            if input & DIRECTIONS_UP_DOWN == 0 {
                return;
            }
            self.o_sound(r, STEP_SOUND);
            let cell = self.read8(CELL);
            if input & 1 == 0 {
                let (mut moved, carry) = cell.overflowing_add(0x0A);
                r.x = carry;
                if moved >= 0x12 {
                    moved = moved.wrapping_sub(0x12);
                    r.x = false;
                }
                r.d0 = low_byte(r.d0, moved);
            } else {
                let (moved, borrow) = cell.overflowing_sub(0x0A);
                r.x = borrow;
                r.d0 = low_byte(r.d0, moved);
                if moved & 0x80 != 0 {
                    let (moved, carry) = moved.overflowing_add(0x12);
                    r.x = carry;
                    r.d0 = if moved != 0 { low_byte(r.d0, moved) } else { 0x10 };
                }
            }
            self.write8(CELL, r.d0 as u8);
            r.c = false;
            return;
        }
        if input & BACK != 0 {
            let changing = self.read8(CHANGING).wrapping_add(1);
            self.write8(CHANGING, changing);
            self.o_sound(r, BACK_SOUND);
            (r.x, r.c) = self.r_00053e(r.d6, PAGE_2);
            return;
        }
        self.r_00053e(r.d6, PAGE_2);
        let changing = self.read8(CHANGING).wrapping_add(1);
        self.write8(CHANGING, changing);
        self.o_sound(r, ON_SOUND);
        let line = self.read8(LINE).wrapping_sub(1);
        self.o_line_object(r, line);
        let cell = self.read8(CELL);
        self.write8(r.a4 + OBJ_CELL, cell);
        r.d0 = low_word(r.d0, (cell as i8 as i16 as u16).wrapping_mul(2));
        r.a0 = ANIMS_BY_CELL;
        let anims = self.read32(ANIMS_BY_CELL.wrapping_add(r.d0 as u16 as i16 as u32));
        self.write32(r.a4 + OBJ_ANIMS, anims);
        r.d5 = low_byte(r.d5, self.read8(r.a4 + OBJ_SPRITE));
        r.a5 = self.read32(r.a4 + OBJ_SPRITE_ADDR);
        (r.a0, _, _) = self.r_00130e(r.d0, r.a4);
        let x = self.r_001e86(r.a4, r.a5);
        let flags = self.read8(r.a5 + SPRITE_FLAGS) | 0x10;
        self.write8(r.a5 + SPRITE_FLAGS, flags);
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, _, _) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, x);
        r.c = self.r_0094b2();
    }

    /// `func_010FB0`, the cell's mark: ends while a page changes
    /// (`func_00121A`); else at the cell's place (`CELL_X`, `CELL_Y` by
    /// `CELL` / 2).
    pub fn r_010fb0(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        if self.read8(CHANGING) != 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        self.write8(r.a5 + SPRITE_Y_HIGH, 0);
        self.write8(r.a5 + SPRITE_X_HIGH, 0);
        let cell = self.read8(CELL);
        r.x = cell & 1 != 0;
        let index = cell >> 1;
        r.d0 = low_word(r.d0, u16::from(index));
        let x = self.read8(CELL_X + u32::from(index));
        self.write8(r.a5 + SPRITE_X_LOW, x);
        let y = self.read8(CELL_Y + u32::from(index));
        self.write8(r.a5 + SPRITE_Y_LOW, y);
        r.c = false;
    }

    /// `func_010FEE`, page 3's task: `LINE` (0-`COUNT`) and
    /// `DIRECTION` stepped (`LINES_LIST_3`; `STEP_SOUND`), bit 6 of the
    /// line's object's flags kept.  Back (`BACK`): page 2
    /// (`func_010C04`, `func_011A6E`).  On (`ON`): the next page
    /// (`loc_01109A`) if `func_0110CA` lets it, else `NO_SOUND`.
    /// No script calls it: reviewed by reading.
    pub fn r_010fee(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        self.write8(LEAVING, 0);
        self.o_lines_list(r, LINES_LIST_3);
        let line = self.read8(LINE);
        self.write8(KEPT_INDEX, line);
        self.o_line_object(r, line);
        let flag = self.read8(r.a4.wrapping_add(BEFORE_FLAGS)) & 0x40;
        r.d0 = low_byte(r.d0, flag);
        self.write8(KEPT_FLAG, flag);
        let (_, changed) = self.r_01031a(r.a1, LIST);
        let index = self.read8(KEPT_INDEX);
        self.o_line_object(r, index);
        let flags = (self.read8(r.a4.wrapping_add(BEFORE_FLAGS)) & !0x40) | self.read8(KEPT_FLAG);
        r.d0 = low_byte(r.d0, flags);
        self.write8(r.a4.wrapping_add(BEFORE_FLAGS), flags);
        if changed {
            self.o_sound(r, STEP_SOUND);
        }
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
        let input = self.read8(INPUT);
        r.d0 = low_byte(r.d0, input);
        if input & BACK != 0 {
            self.write32(r.a6 + TASK_NEXT, PAGE_2);
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_010c04(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            if self.restarted.is_none() {
                (r.d0, r.c) = self.r_011a6e(r.d0);
            }
            return;
        }
        r.d0 = low_byte(r.d0, input & ON);
        r.c = false;
        if input & ON == 0 {
            return;
        }
        let refused;
        (r.x, refused) = self.r_0110ca(r.d4);
        if !refused {
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = self.r_01109a(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
            return;
        }
        r.c = false;
        self.o_sound(r, NO_SOUND);
    }
}
