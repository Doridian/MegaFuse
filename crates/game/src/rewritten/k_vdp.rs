//! The VDP from 0xB49A on: the window plane's registers, a VRAM copy
//! kept for the vertical blank (asked for by a task that steps
//! through a table of sources, `func_00B8C0`; done by `func_00B954`),
//! and a DMA from the 68000's memory to VRAM with the Z80's bus taken
//! (`func_00B9D0`, `func_00B9F6`).

use crate::{Game, Regs};

/// The VDP's control port and data port.
const CONTROL: u32 = 0xC00004;
const DATA: u32 = 0xC00000;
/// The status bit that stays up while a DMA runs.
const DMA_BUSY: u16 = 1 << 1;

/// The VDP's registers as the game keeps them in RAM, each a control
/// port word (`0x8rvv`), register r at 0xFFF004 + 2r (`vdp.rs`): 1
/// (bit 4 lets DMA run), 3 (the window's table), 15 (the address
/// step), 17 and 18 (the window's position across and down).
const REG1: u32 = 0xFFFF_F006;
const REG3: u32 = 0xFFFF_F00A;
const REG15: u32 = 0xFFFF_F022;
const REG17: u32 = 0xFFFF_F026;
const REG18: u32 = 0xFFFF_F028;
const DMA_ON: u16 = 1 << 4;

/// The Z80's bus request (bit 8 written: the 68000 wants the bus; bit
/// 0 read: set until the Z80 has let it go).
const BUSREQ: u32 = 0xA11100;

/// The VRAM copy `func_00B954` makes: it runs while `COPY_ON` (set by
/// `func_00B8C0`, cleared by `func_00BE96`) and `COPY_ASKED` (set by
/// `func_00B8C0`, cleared once copied) are both set; from `COPY_FROM`
/// to `COPY_TO` (VRAM addresses), `COPY_LENGTH` bytes (words).
const COPY_ASKED: u32 = 0xFFFF_A944;
const COPY_ON: u32 = 0xFFFF_A945;
const COPY_TO: u32 = 0xFFFF_A946;
const COPY_FROM: u32 = 0xFFFF_A948;
const COPY_LENGTH: u32 = 0xFFFF_A94A;
/// A count whose low 2 bits say which task's turn it is to step
/// (`func_00B8C0`, against its `TURN`); cleared by `func_00BE96`.
const TURNS: u32 = 0xFFFF_A94C;
/// Set while the play is stopped (`func_00B378`): the tasks of
/// `func_00B8C0` ask for their copy without stepping.
const STOPPED: u32 = 0xFFFF_A3FC;

/// `func_00B9D0`'s DMA: from the long at `DMA_FROM` (an address in
/// the 68000's memory) to the VRAM address `DMA_TO` (a word: its bits
/// 14-15 the command's), `DMA_LENGTH` words.
const DMA_FROM: u32 = 0xFFFF_A94E;
const DMA_TO: u32 = 0xFFFF_A952;
const DMA_LENGTH: u32 = 0xFFFF_A954;

/// A `func_00B8C0` task's fields.
/// The table of sources (words: a VRAM address halved, plus 0x1000),
/// a long.
const TABLE: u32 = 0x0C;
/// Where the copy goes (a word, as the table's), and its length in
/// words less 1 (a word).
const TO: u32 = 0x10;
const LENGTH: u32 = 0x12;
/// The turns left to the next step, and what it starts again at.
const WAIT: u32 = 0x14;
const DELAY: u32 = 0x15;
/// The table entry copied, and the number of entries.
const STEP: u32 = 0x16;
const STEPS: u32 = 0x17;
/// The task's turn (compared with `TURNS`' low 2 bits).
const TURN: u32 = 0x18;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

/// A table address halved, plus 0x1000, as a VRAM address.
fn vram(word: u16) -> u16 {
    word.wrapping_sub(0x1000).wrapping_mul(2)
}

impl Game {
    /// The VDP's register kept at `reg` (in RAM) set to `value` and
    /// written to it; gives back the word written.
    fn k_vdp_reg(&mut self, reg: u32, value: u8) -> u16 {
        self.write8(reg + 1, value);
        let word = self.read16(reg);
        self.write16(CONTROL, word);
        word
    }

    /// The window's registers set to `table`, 0, `down` (17 and 18
    /// after 3 if `table` is given); the flags of the last word's
    /// write (N, Z; C clear) to `regs`.
    fn k_window(&mut self, regs: &mut Regs, table: Option<u8>, down: u8) {
        if let Some(table) = table {
            self.k_vdp_reg(REG3, table);
        }
        self.k_vdp_reg(REG17, 0);
        let word = self.k_vdp_reg(REG18, down);
        regs.n = word & 0x8000 != 0;
        regs.z = word == 0;
        regs.c = false;
        regs.unwound = false;
    }

    /// `func_00B49A`: the window's table at 0xE000 (register 3 0x38),
    /// across 0, down 3 rows.  Gives back the flags of the last write.
    pub fn r_00b49a(&mut self, regs: &mut Regs) {
        self.k_window(regs, Some(0x38), 3);
    }

    /// `func_00B4C6`: no window (registers 17 and 18 0).  Gives back
    /// the flags of the last write.
    pub fn r_00b4c6(&mut self, regs: &mut Regs) {
        self.k_window(regs, None, 0);
    }

    /// `func_00B4E4`: the window's table at 0xE000, across 0, down
    /// 0x10 rows.
    /// No script runs it (checked by reading).
    pub fn r_00b4e4(&mut self) {
        self.k_vdp_reg(REG3, 0x38);
        self.k_vdp_reg(REG17, 0);
        self.k_vdp_reg(REG18, 0x10);
    }

    /// `func_00B8C0`, a task: on its turn (`TURN` equal to `TURNS`'
    /// low 2 bits) it steps through its table, an entry every `DELAY`
    /// turns (`WAIT` counts them down; not while `STOPPED`), and asks
    /// for the VRAM copy of the entry's source to `TO`, `LENGTH` + 1
    /// words (with interrupts masked), and sets `COPY_ON`.  Gives back
    /// d0, d1, a0, X and C (as the status register put back leaves X;
    /// on another's turn, C is the compare's).
    pub fn r_00b8c0(&mut self, regs: &mut Regs) {
        let task = regs.a6;
        regs.unwound = false;
        regs.d0 = 1;
        let turn = self.read8(TURNS) & 3;
        regs.d1 = low_byte(regs.d1, turn);
        let mine = self.read8(task + TURN);
        if turn != mine {
            regs.c = turn < mine;
            return;
        }
        let mut x = regs.x;
        if self.read8(STOPPED) == 0 {
            let wait = self.read8(task + WAIT).wrapping_sub(1);
            if wait != 0 && (wait as i8) > 0 {
                self.write8(task + WAIT, wait);
                regs.d1 = low_byte(regs.d1, wait);
                regs.x = false;
                regs.c = false;
                return;
            }
            let delay = self.read8(task + DELAY);
            self.write8(task + WAIT, delay);
            let (mut step, carry) = self.read8(task + STEP).overflowing_add(1);
            x = carry;
            if step == self.read8(task + STEPS) {
                step = 0;
            }
            self.write8(task + STEP, step);
            regs.d1 = low_byte(regs.d1, step);
        }
        let ipl = self.ipl();
        self.set_ipl(7);
        let table = self.read32(task + TABLE);
        let step = u32::from(self.read8(task + STEP));
        let from = vram(self.read16(table + 2 * step));
        self.write16(COPY_FROM, from);
        let length = self.read16(task + LENGTH).wrapping_add(1).wrapping_mul(2);
        self.write16(COPY_LENGTH, length);
        let to = vram(self.read16(task + TO));
        self.write16(COPY_TO, to);
        self.write8(COPY_ASKED, 1);
        self.set_ipl(ipl);
        self.write8(COPY_ON, 1);
        regs.d0 = u32::from(to);
        regs.a0 = table;
        regs.x = x;
        regs.c = false;
    }

    /// `func_00B954`: the VRAM copy asked for (`COPY_ASKED`, while
    /// `COPY_ON`), by DMA (step 1); waits for it to end.
    pub fn r_00b954(&mut self) {
        if self.read8(COPY_ON) == 0 || self.read8(COPY_ASKED) == 0 {
            return;
        }
        let reg15 = self.read16(REG15);
        self.write16(CONTROL, (reg15 & 0xFF00) | 1);
        let reg1 = self.read16(REG1);
        self.write16(CONTROL, reg1 | DMA_ON);
        // the length (registers 20 and 19)
        let length = self.read16(COPY_LENGTH);
        self.write32(CONTROL, 0x9400_9300 | u32::from(length >> 8) << 16 | u32::from(length & 0xFF));
        // the source (registers 21 and 22), then the copy mode (23)
        let from = self.read16(COPY_FROM);
        self.write32(CONTROL, 0x9500_9600 | u32::from(from & 0xFF) << 16 | u32::from(from >> 8));
        self.write16(CONTROL, 0x97C0);
        let to = u32::from(self.read16(COPY_TO));
        self.write32(CONTROL, (to & 0x3FFF) << 16 | to >> 14 | 0xC0);
        while self.read16(CONTROL) & DMA_BUSY != 0 {
            self.pass(26);
        }
        let reg1 = self.read16(REG1);
        self.write16(CONTROL, reg1);
        let reg15 = self.read16(REG15);
        self.write16(CONTROL, reg15);
        self.write8(COPY_ASKED, 0);
    }

    /// `func_00B9D0`: `func_00B9F6` with interrupts masked, from
    /// `DMA_FROM` to `DMA_TO`, `DMA_LENGTH` words.  Gives back a0 (the
    /// source).
    pub fn r_00b9d0(&mut self) -> u32 {
        let ipl = self.ipl();
        self.set_ipl(7);
        let length = self.read16(DMA_LENGTH);
        let to = self.read16(DMA_TO);
        let from = self.read32(DMA_FROM);
        let a0 = self.r_00b9f6(from, to, (length >> 8) as u8, length as u8);
        self.set_ipl(ipl);
        a0
    }

    /// `func_00B9F6` (arguments on the stack): a DMA of `length_high`,
    /// `length_low` words from `from` (in the 68000's memory) to the
    /// VRAM address `to` (its bits 14-15 read from `DMA_TO`), the Z80's
    /// bus taken meanwhile (and left as it was); its first word written
    /// again by hand after it.  Gives back a0 (`from`).
    pub fn r_00b9f6(&mut self, from: u32, to: u16, length_high: u8, length_low: u8) -> u32 {
        let reg1 = self.read16(REG1);
        self.write16(CONTROL, reg1 | DMA_ON);
        let bus = self.read16(BUSREQ) ^ 0x100;
        self.write16(BUSREQ, 0x100);
        self.write32(CONTROL, 0x9400_9300 | u32::from(length_high) << 16 | u32::from(length_low));
        // the source in words (registers 21, 22 and 23)
        let source = from >> 1;
        self.write32(CONTROL, 0x9500_9600 | (source & 0xFF) << 16 | (source >> 8 & 0xFF));
        self.write16(CONTROL, 0x9700 | (source >> 16 & 0x7F) as u16);
        let first = (to & 0x3FFF) | 0x4000;
        self.write16(CONTROL, first);
        let high = self.read16(DMA_TO).rotate_left(2) & 3;
        while self.read8(BUSREQ) & 1 != 0 {
            self.pass(30);
        }
        // the second word, with the DMA bit: it starts
        self.write16(CONTROL, high | 0x80);
        while self.read16(CONTROL) & DMA_BUSY != 0 {
            self.pass(26);
        }
        let reg1 = self.read16(REG1);
        self.write16(CONTROL, reg1);
        self.write32(CONTROL, u32::from(first) << 16 | u32::from(high));
        let word = self.read16(from);
        self.write16(DATA, word);
        self.write16(BUSREQ, bus);
        from
    }
}
