//! The pads: the I/O ports read (port 1 alone, or four pads through
//! it when `func_009086` found the four-way adaptor), and the Z80 held
//! for good (`func_00922C`).
//!
//! No script plays with the adaptor, so `func_00912A` and the pads'
//! routines under it, and `func_00922C`, are reviewed by reading only.

use crate::Game;

/// The I/O ports: data of port 1 and 2, control of port 2 and 3.
const DATA_1: u32 = 0xA10003;
const DATA_2: u32 = 0xA10005;
const CTRL_2: u32 = 0xA10009;
const CTRL_3: u32 = 0xA1000B;
/// The Z80's bus request and reset (`z80.rs`).
const BUSREQ: u32 = 0xA11100;
const RESET: u32 = 0xA11200;

/// Whether `func_009086` looks for the adaptor (a byte, not 0: it does).
const PROBE: u32 = 0xFFFF_80C3;
/// What it found: 0xFF the adaptor (four pads on port 1), 0 not.
const ADAPTOR: u32 = 0xFFFF_80C2;
/// Each pad's record (10 bytes): a word cleared, then what
/// `func_007F4A` keeps of the buttons read.
const PAD_1: u32 = 0xFFFF_800A;
const PAD_2: u32 = 0xFFFF_8014;
const PAD_3: u32 = 0xFFFF_801E;
const PAD_4: u32 = 0xFFFF_8028;

/// The TH line of a port's data (written: which half of the buttons
/// is read next).
const TH: u8 = 0x40;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// `func_009086`: with `PROBE` set, interrupts masked and the Z80's
    /// bus taken, port 2 and 3 set up and the adaptor looked for
    /// (`func_0090F2`): `ADAPTOR` 0xFF if it answers, else 0 (and port
    /// 2 and 3 set up again).  The bus request goes back to the
    /// opposite of what was read of it.
    pub fn r_009086(&mut self) {
        if self.read8(PROBE) == 0 {
            return;
        }
        let ipl = self.ipl();
        self.set_ipl(7);
        let bus = self.read16(BUSREQ) ^ 0x100;
        self.write16(BUSREQ, 0x100);
        self.write8(CTRL_2, TH);
        self.write8(CTRL_3, TH);
        if self.r_0090f2() {
            self.write8(ADAPTOR, 0xFF);
        } else {
            self.write8(ADAPTOR, 0);
            self.write8(CTRL_2, TH);
            self.write8(CTRL_3, TH);
        }
        self.write16(BUSREQ, bus);
        self.set_ipl(ipl);
    }

    /// `func_0090F2`: port 3 and 2 set the way the adaptor answers to;
    /// gives back Z: port 1's low two bits read 0 (the adaptor).
    pub fn r_0090f2(&mut self) -> bool {
        self.write8(CTRL_3, 0x43);
        self.write8(DATA_2, 0x7C);
        self.write8(CTRL_3, 0x7F);
        self.write8(DATA_2, 0x7C);
        self.read8(DATA_1) & 3 == 0
    }

    /// `func_00912A`: the pads read: with the adaptor four from port 1
    /// (`func_00918E` .. `func_0091B8`), else port 1's and port 2's
    /// (`func_0091E2`, `func_0091D4`).  d1 is kept.  Gives back d0 (the
    /// last pad's buttons).
    pub fn r_00912a(&mut self, d1: u32) -> u32 {
        self.write8(CTRL_2, TH);
        if self.read8(ADAPTOR) == 0 {
            self.r_0091e2(d1, PAD_1);
            self.r_0091d4(d1, PAD_2)
        } else {
            self.r_00918e(d1, PAD_1);
            self.r_00919c(d1, PAD_2);
            self.r_0091aa(d1, PAD_3);
            self.r_0091b8(d1, PAD_4)
        }
    }

    /// `func_00918E`: the adaptor's pad 1 (port 2 says which) read into
    /// the record at `a1` (`func_0091C6`).
    pub fn r_00918e(&mut self, d1: u32, a1: u32) {
        self.write8(DATA_2, 0x0C);
        self.r_0091c6(d1, a1);
    }

    /// `func_00919C`: the adaptor's pad 2.
    pub fn r_00919c(&mut self, d1: u32, a1: u32) {
        self.write8(DATA_2, 0x1C);
        self.r_0091c6(d1, a1);
    }

    /// `func_0091AA`: the adaptor's pad 3.
    pub fn r_0091aa(&mut self, d1: u32, a1: u32) {
        self.write8(DATA_2, 0x2C);
        self.r_0091c6(d1, a1);
    }

    /// `func_0091B8`: the adaptor's pad 4.  Gives back d0 (its buttons).
    pub fn r_0091b8(&mut self, d1: u32, a1: u32) -> u32 {
        self.write8(DATA_2, 0x3C);
        self.r_0091c6(d1, a1)
    }

    /// `func_0091C6`: a pad read from port 1 into the record at `a1`.
    pub fn r_0091c6(&mut self, d1: u32, a1: u32) -> u32 {
        self.i_read_pad(DATA_1, d1, a1)
    }

    /// `func_0091D4`: port 2's pad into the record at `a1`.  Gives back
    /// d0 (its buttons).
    pub fn r_0091d4(&mut self, d1: u32, a1: u32) -> u32 {
        self.i_read_pad(DATA_2, d1, a1)
    }

    /// `func_0091E2`: port 1's pad into the record at `a1`.
    pub fn r_0091e2(&mut self, d1: u32, a1: u32) {
        self.i_read_pad(DATA_1, d1, a1);
    }

    /// The pad on `port` read (TH low: start and A in bits 4-5, TH
    /// high: C, B and the pad), the buttons a byte, 1 for down
    /// (start, A, C, B, right, left, down, up from bit 7); the record's
    /// first word cleared and the buttons given to `func_007F4A` with
    /// the rest of it.  Gives back d0 (the buttons).
    fn i_read_pad(&mut self, port: u32, d1: u32, record: u32) -> u32 {
        self.write8(port, 0);
        let low = self.read8(port);
        self.write8(port, TH);
        let high = self.read8(port) & 0x3F;
        let buttons = !((low << 2) & 0xC0 | high);
        let d0 = u32::from(buttons);
        self.write16(record, 0);
        self.r_007f4a(d0, low_byte(d1, high), record + 2);
        d0
    }

    /// `func_00922C`: the Z80 out of reset and its bus taken, waited
    /// for.
    pub fn r_00922c(&mut self) {
        self.write16(RESET, 0x100);
        self.write16(BUSREQ, 0x100);
        while self.read8(BUSREQ) & 1 != 0 {
            self.pass(30);
        }
    }
}
