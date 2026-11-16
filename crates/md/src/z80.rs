//! A Z80, for the sound driver the game uploads (docs/PORT.md, "Sound"):
//! an interpreter, and the arithmetic the translated driver
//! (`driver.rs`) shares with it.
//! The documented instructions and the undocumented ones Z80 sound
//! drivers use (IXH/IXL/IYH/IYL, SLL); cycle counts from the Z80's
//! tables.

/// What the Z80 reaches: its RAM, the sound chips, the bank window.
pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

/// The cycles each instruction takes but for what depends on the data
/// (a branch taken, a block instruction going round again), by prefix:
/// what the Z80 spends before its accesses are made.
struct Base {
    main: [u8; 256],
    cb: [u8; 256],
    ed: [u8; 256],
    dd: [u8; 256],
    fd: [u8; 256],
    ddcb: [u8; 256],
    fdcb: [u8; 256],
}

/// 64 KB of scratch memory, for measuring.
struct Scratch(Vec<u8>);

impl Bus for Scratch {
    fn read(&mut self, addr: u16) -> u8 {
        self.0[usize::from(addr)]
    }
    fn write(&mut self, addr: u16, value: u8) {
        self.0[usize::from(addr)] = value;
    }
}

impl Base {
    /// Each instruction run once on a scratch Z80, its conditions false
    /// (the flags all clear, then all set; B and BC 1): the least it
    /// took.
    fn measure() -> Self {
        let one = |code: &[u8]| -> u8 {
            let mut least = u32::MAX;
            for f in [0x00, 0xFF] {
                for (b, c) in [(1, 0), (0, 1)] {
                    let mut mem = Scratch(vec![0; 0x10000]);
                    mem.0[..code.len()].copy_from_slice(code);
                    let mut z = Z80::new();
                    z.f = f;
                    z.b = b;
                    z.c = c;
                    z.pc = 0;
                    z.sp = 0x8000;
                    least = least.min(z.step(&mut mem));
                }
            }
            least as u8
        };
        let mut t = Base {
            main: [0; 256],
            cb: [0; 256],
            ed: [0; 256],
            dd: [0; 256],
            fd: [0; 256],
            ddcb: [0; 256],
            fdcb: [0; 256],
        };
        for op in 0..=255u8 {
            let k = usize::from(op);
            t.main[k] = one(&[op]);
            t.cb[k] = one(&[0xCB, op]);
            t.ed[k] = one(&[0xED, op]);
            t.dd[k] = one(&[0xDD, op]);
            t.fd[k] = one(&[0xFD, op]);
            t.ddcb[k] = one(&[0xDD, 0xCB, 0, op]);
            t.fdcb[k] = one(&[0xFD, 0xCB, 0, op]);
        }
        t
    }
}

static BASE: std::sync::OnceLock<Base> = std::sync::OnceLock::new();

const C: u8 = 0x01;
const N: u8 = 0x02;
const PV: u8 = 0x04;
const X3: u8 = 0x08;
const H: u8 = 0x10;
const X5: u8 = 0x20;
const Z: u8 = 0x40;
const S: u8 = 0x80;

#[derive(Default, Clone)]
pub struct Z80 {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    alt: [u8; 8],
    pub ix: u16,
    pub iy: u16,
    pub sp: u16,
    pub pc: u16,
    pub i: u8,
    pub r: u8,
    iff1: bool,
    iff2: bool,
    im: u8,
    halted: bool,
    /// EI's one-instruction delay.
    ei_delay: bool,
}

fn parity(v: u8) -> bool {
    v.count_ones().is_multiple_of(2)
}

fn szp(v: u8) -> u8 {
    let mut f = v & (S | X5 | X3);
    if v == 0 {
        f |= Z;
    }
    if parity(v) {
        f |= PV;
    }
    f
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Idx {
    Hl,
    Ix,
    Iy,
}

impl Z80 {
    #[must_use]
    pub fn new() -> Self {
        let mut z = Z80::default();
        z.reset();
        z
    }

    pub fn reset(&mut self) {
        self.pc = 0;
        self.i = 0;
        self.r = 0;
        self.iff1 = false;
        self.iff2 = false;
        self.im = 0;
        self.halted = false;
        self.sp = 0xFFFF;
        self.a = 0xFF;
        self.f = 0xFF;
    }

    pub(crate) fn bc(&self) -> u16 {
        u16::from(self.b) << 8 | u16::from(self.c)
    }
    pub(crate) fn de(&self) -> u16 {
        u16::from(self.d) << 8 | u16::from(self.e)
    }
    pub(crate) fn hl(&self) -> u16 {
        u16::from(self.h) << 8 | u16::from(self.l)
    }
    pub(crate) fn set_bc(&mut self, v: u16) {
        self.b = (v >> 8) as u8;
        self.c = v as u8;
    }
    pub(crate) fn set_de(&mut self, v: u16) {
        self.d = (v >> 8) as u8;
        self.e = v as u8;
    }
    pub(crate) fn set_hl(&mut self, v: u16) {
        self.h = (v >> 8) as u8;
        self.l = v as u8;
    }
    pub(crate) fn af(&self) -> u16 {
        u16::from(self.a) << 8 | u16::from(self.f)
    }
    pub(crate) fn set_af(&mut self, v: u16) {
        self.a = (v >> 8) as u8;
        self.f = v as u8;
    }

    fn idx(&self, x: Idx) -> u16 {
        match x {
            Idx::Hl => self.hl(),
            Idx::Ix => self.ix,
            Idx::Iy => self.iy,
        }
    }
    fn set_idx(&mut self, x: Idx, v: u16) {
        match x {
            Idx::Hl => self.set_hl(v),
            Idx::Ix => self.ix = v,
            Idx::Iy => self.iy = v,
        }
    }

    fn fetch(&mut self, bus: &mut dyn Bus) -> u8 {
        let v = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }
    fn fetch16(&mut self, bus: &mut dyn Bus) -> u16 {
        let lo = self.fetch(bus);
        let hi = self.fetch(bus);
        u16::from(hi) << 8 | u16::from(lo)
    }
    pub(crate) fn read16(bus: &mut dyn Bus, a: u16) -> u16 {
        u16::from(bus.read(a)) | u16::from(bus.read(a.wrapping_add(1))) << 8
    }
    pub(crate) fn write16(bus: &mut dyn Bus, a: u16, v: u16) {
        bus.write(a, v as u8);
        bus.write(a.wrapping_add(1), (v >> 8) as u8);
    }
    pub(crate) fn push(&mut self, bus: &mut dyn Bus, v: u16) {
        self.sp = self.sp.wrapping_sub(2);
        Self::write16(bus, self.sp, v);
    }
    pub(crate) fn pop(&mut self, bus: &mut dyn Bus) -> u16 {
        let v = Self::read16(bus, self.sp);
        self.sp = self.sp.wrapping_add(2);
        v
    }

    /// The maskable interrupt (the VDP's, once a frame).  Returns the
    /// cycles taken (0 when interrupts are off).
    pub fn interrupt(&mut self, bus: &mut dyn Bus) -> u32 {
        if !self.iff1 || self.ei_delay {
            return 0;
        }
        self.halted = false;
        self.iff1 = false;
        self.iff2 = false;
        match self.im {
            2 => {
                let v = u16::from(self.i) << 8 | 0xFF;
                self.push(bus, self.pc);
                self.pc = Self::read16(bus, v);
                19
            }
            _ => {
                self.push(bus, self.pc);
                self.pc = 0x38;
                13
            }
        }
    }

    // --- 8-bit registers by number (B C D E H L (HL) A) --------------------

    fn reg(&mut self, bus: &mut dyn Bus, n: u8, x: Idx, d: i8) -> u8 {
        match n {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => match x {
                Idx::Hl => self.h,
                _ => (self.idx(x) >> 8) as u8,
            },
            5 => match x {
                Idx::Hl => self.l,
                _ => self.idx(x) as u8,
            },
            6 => {
                let a = self.idx(x).wrapping_add(d as u16);
                bus.read(a)
            }
            _ => self.a,
        }
    }

    fn set_reg(&mut self, bus: &mut dyn Bus, n: u8, x: Idx, d: i8, v: u8) {
        match n {
            0 => self.b = v,
            1 => self.c = v,
            2 => self.d = v,
            3 => self.e = v,
            4 => match x {
                Idx::Hl => self.h = v,
                _ => {
                    let r = self.idx(x);
                    self.set_idx(x, (r & 0x00FF) | u16::from(v) << 8);
                }
            },
            5 => match x {
                Idx::Hl => self.l = v,
                _ => {
                    let r = self.idx(x);
                    self.set_idx(x, (r & 0xFF00) | u16::from(v));
                }
            },
            6 => {
                let a = self.idx(x).wrapping_add(d as u16);
                bus.write(a, v);
            }
            _ => self.a = v,
        }
    }

    fn rp(&self, n: u8, x: Idx) -> u16 {
        match n {
            0 => self.bc(),
            1 => self.de(),
            2 => self.idx(x),
            _ => self.sp,
        }
    }
    fn set_rp(&mut self, n: u8, x: Idx, v: u16) {
        match n {
            0 => self.set_bc(v),
            1 => self.set_de(v),
            2 => self.set_idx(x, v),
            _ => self.sp = v,
        }
    }

    pub(crate) fn cond(&self, n: u8) -> bool {
        match n {
            0 => self.f & Z == 0,
            1 => self.f & Z != 0,
            2 => self.f & C == 0,
            3 => self.f & C != 0,
            4 => self.f & PV == 0,
            5 => self.f & PV != 0,
            6 => self.f & S == 0,
            _ => self.f & S != 0,
        }
    }

    // --- the ALU -------------------------------------------------------------

    pub(crate) fn add8(&mut self, v: u8, carry: bool) {
        let c = u16::from(carry && self.f & C != 0);
        let a = self.a;
        let r = u16::from(a) + u16::from(v) + c;
        let rb = r as u8;
        let mut f = rb & (S | X5 | X3);
        if rb == 0 {
            f |= Z;
        }
        if r > 0xFF {
            f |= C;
        }
        if (a ^ v ^ rb) & 0x10 != 0 {
            f |= H;
        }
        if (a ^ rb) & (v ^ rb) & 0x80 != 0 {
            f |= PV;
        }
        self.a = rb;
        self.f = f;
    }

    pub(crate) fn sub8(&mut self, v: u8, carry: bool, store: bool) {
        let c = u16::from(carry && self.f & C != 0);
        let a = self.a;
        let r = u16::from(a).wrapping_sub(u16::from(v)).wrapping_sub(c);
        let rb = r as u8;
        let mut f = N | (rb & (S | X5 | X3));
        if !store {
            f = N | (rb & S) | (v & (X5 | X3));
        }
        if rb == 0 {
            f |= Z;
        }
        if r > 0xFF {
            f |= C;
        }
        if (a ^ v ^ rb) & 0x10 != 0 {
            f |= H;
        }
        if (a ^ v) & (a ^ rb) & 0x80 != 0 {
            f |= PV;
        }
        if store {
            self.a = rb;
        }
        self.f = f;
    }

    pub(crate) fn alu(&mut self, op: u8, v: u8) {
        match op {
            0 => self.add8(v, false),
            1 => self.add8(v, true),
            2 => self.sub8(v, false, true),
            3 => self.sub8(v, true, true),
            4 => {
                self.a &= v;
                self.f = szp(self.a) | H;
            }
            5 => {
                self.a ^= v;
                self.f = szp(self.a);
            }
            6 => {
                self.a |= v;
                self.f = szp(self.a);
            }
            _ => self.sub8(v, false, false),
        }
    }

    pub(crate) fn inc8(&mut self, v: u8) -> u8 {
        let r = v.wrapping_add(1);
        let mut f = (self.f & C) | (r & (S | X5 | X3));
        if r == 0 {
            f |= Z;
        }
        if v & 0x0F == 0x0F {
            f |= H;
        }
        if v == 0x7F {
            f |= PV;
        }
        self.f = f;
        r
    }

    pub(crate) fn dec8(&mut self, v: u8) -> u8 {
        let r = v.wrapping_sub(1);
        let mut f = (self.f & C) | N | (r & (S | X5 | X3));
        if r == 0 {
            f |= Z;
        }
        if v & 0x0F == 0 {
            f |= H;
        }
        if v == 0x80 {
            f |= PV;
        }
        self.f = f;
        r
    }

    pub(crate) fn add16(&mut self, a: u16, b: u16) -> u16 {
        let r = u32::from(a) + u32::from(b);
        let mut f = self.f & (S | Z | PV);
        if r > 0xFFFF {
            f |= C;
        }
        if (a ^ b ^ r as u16) & 0x1000 != 0 {
            f |= H;
        }
        f |= ((r >> 8) as u8) & (X5 | X3);
        self.f = f;
        r as u16
    }

    pub(crate) fn adc16(&mut self, a: u16, b: u16) -> u16 {
        let c = u32::from(self.f & C);
        let r = u32::from(a) + u32::from(b) + c;
        let rw = r as u16;
        let mut f = ((rw >> 8) as u8) & (S | X5 | X3);
        if rw == 0 {
            f |= Z;
        }
        if r > 0xFFFF {
            f |= C;
        }
        if (a ^ b ^ rw) & 0x1000 != 0 {
            f |= H;
        }
        if (a ^ rw) & (b ^ rw) & 0x8000 != 0 {
            f |= PV;
        }
        self.f = f;
        rw
    }

    pub(crate) fn sbc16(&mut self, a: u16, b: u16) -> u16 {
        let c = u32::from(self.f & C);
        let r = u32::from(a).wrapping_sub(u32::from(b)).wrapping_sub(c);
        let rw = r as u16;
        let mut f = N | (((rw >> 8) as u8) & (S | X5 | X3));
        if rw == 0 {
            f |= Z;
        }
        if r > 0xFFFF {
            f |= C;
        }
        if (a ^ b ^ rw) & 0x1000 != 0 {
            f |= H;
        }
        if (a ^ b) & (a ^ rw) & 0x8000 != 0 {
            f |= PV;
        }
        self.f = f;
        rw
    }

    pub(crate) fn rot(&mut self, op: u8, v: u8) -> u8 {
        let c = self.f & C != 0;
        let (r, out) = match op {
            0 => (v.rotate_left(1), v & 0x80 != 0),
            1 => (v.rotate_right(1), v & 1 != 0),
            2 => ((v << 1) | u8::from(c), v & 0x80 != 0),
            3 => ((v >> 1) | if c { 0x80 } else { 0 }, v & 1 != 0),
            4 => (v << 1, v & 0x80 != 0),
            5 => ((v >> 1) | (v & 0x80), v & 1 != 0),
            6 => ((v << 1) | 1, v & 0x80 != 0),
            _ => (v >> 1, v & 1 != 0),
        };
        self.f = szp(r) | u8::from(out);
        r
    }

    pub(crate) fn daa(&mut self) {
        let a = self.a;
        let mut corr = 0u8;
        let mut carry = self.f & C != 0;
        if self.f & H != 0 || a & 0x0F > 9 {
            corr |= 0x06;
        }
        if carry || a > 0x99 {
            corr |= 0x60;
            carry = true;
        }
        let r = if self.f & N != 0 { a.wrapping_sub(corr) } else { a.wrapping_add(corr) };
        let h = if self.f & N != 0 {
            self.f & H != 0 && a & 0x0F < 6
        } else {
            a & 0x0F > 9
        };
        self.a = r;
        self.f = szp(r) | (self.f & N) | u8::from(carry) | if h { H } else { 0 };
    }

    // --- what the translated driver shares with the interpreter -----------

    /// RLCA RRCA RLA RRA (y 0-3): A rotated, S Z P/V kept.
    pub(crate) fn rot_a(&mut self, y: u8) {
        let keep = self.f & (S | Z | PV);
        let r = self.rot(y, self.a);
        self.a = r;
        self.f = keep | (self.f & C) | (r & (X5 | X3));
    }

    pub(crate) fn cpl(&mut self) {
        self.a = !self.a;
        self.f = (self.f & (S | Z | PV | C)) | H | N | (self.a & (X5 | X3));
    }

    pub(crate) fn scf(&mut self) {
        self.f = (self.f & (S | Z | PV)) | C | (self.a & (X5 | X3));
    }

    pub(crate) fn ccf(&mut self) {
        let c = self.f & C;
        self.f = (self.f & (S | Z | PV)) | (self.a & (X5 | X3)) | if c != 0 { H } else { C };
    }

    pub(crate) fn ex_af(&mut self) {
        let af = self.af();
        let alt = u16::from(self.alt[6]) << 8 | u16::from(self.alt[7]);
        self.set_af(alt);
        self.alt[6] = (af >> 8) as u8;
        self.alt[7] = af as u8;
    }

    pub(crate) fn exx(&mut self) {
        for k in 0..6 {
            let v = [self.b, self.c, self.d, self.e, self.h, self.l][k];
            let w = self.alt[k];
            self.alt[k] = v;
            match k {
                0 => self.b = w,
                1 => self.c = w,
                2 => self.d = w,
                3 => self.e = w,
                4 => self.h = w,
                _ => self.l = w,
            }
        }
    }

    pub(crate) fn ex_de_hl(&mut self) {
        let de = self.de();
        let hl = self.hl();
        self.set_de(hl);
        self.set_hl(de);
    }

    /// BIT y of v: the flags.
    pub(crate) fn bit(&mut self, y: u8, v: u8) {
        let bit = v & (1 << y) != 0;
        let mut f = (self.f & C) | H;
        if !bit {
            f |= Z | PV;
        }
        if y == 7 && bit {
            f |= S;
        }
        f |= v & (X5 | X3);
        self.f = f;
    }

    pub(crate) fn neg(&mut self) {
        let v = self.a;
        self.a = 0;
        self.sub8(v, false, true);
    }

    pub(crate) fn di(&mut self) {
        self.iff1 = false;
        self.iff2 = false;
    }

    pub(crate) fn set_im(&mut self, im: u8) {
        self.im = im;
    }

    /// LDI or LDD (`down`) once: a byte from (HL) to (DE), BC counted
    /// down; whether BC is still not 0 (LDIR and LDDR go round again).
    pub(crate) fn ld_block(&mut self, bus: &mut dyn Bus, down: bool) -> bool {
        let v = bus.read(self.hl());
        bus.write(self.de(), v);
        let step: u16 = if down { 0xFFFF } else { 1 };
        self.set_hl(self.hl().wrapping_add(step));
        self.set_de(self.de().wrapping_add(step));
        self.set_bc(self.bc().wrapping_sub(1));
        let n = v.wrapping_add(self.a);
        self.f = (self.f & (S | Z | C)) | (n & X3) | ((n << 4) & X5) | if self.bc() != 0 { PV } else { 0 };
        self.bc() != 0
    }

    /// CPI or CPD once; whether CPIR and CPDR go round again (BC not 0,
    /// no match).
    pub(crate) fn cp_block(&mut self, bus: &mut dyn Bus, down: bool) -> bool {
        let v = bus.read(self.hl());
        let c = self.f & C;
        let r = self.a.wrapping_sub(v);
        let step: u16 = if down { 0xFFFF } else { 1 };
        self.set_hl(self.hl().wrapping_add(step));
        self.set_bc(self.bc().wrapping_sub(1));
        let h = (self.a ^ v ^ r) & H;
        let n = r.wrapping_sub(u8::from(h != 0));
        self.f = N | c | h | (r & S) | if r == 0 { Z } else { 0 } | (n & X3) | ((n << 4) & X5) | if self.bc() != 0 { PV } else { 0 };
        self.bc() != 0 && r != 0
    }

    /// Bumps R, as each instruction does.
    pub(crate) fn refresh(&mut self) {
        self.r = (self.r & 0x80) | (self.r.wrapping_add(1) & 0x7F);
    }

    // --- execution -------------------------------------------------------------

    /// One instruction; returns its cycles.
    /// The base time (cycles) of the instruction at PC: what it takes
    /// but for a branch taken or a block instruction going round again.
    pub fn base_cycles(&self, bus: &mut dyn Bus) -> u32 {
        if self.halted {
            return 4;
        }
        let t = BASE.get_or_init(Base::measure);
        let at = |k: u16| self.pc.wrapping_add(k);
        let op = bus.read(at(0));
        u32::from(match op {
            0xCB => t.cb[usize::from(bus.read(at(1)))],
            0xED => t.ed[usize::from(bus.read(at(1)))],
            0xDD | 0xFD => {
                let op2 = bus.read(at(1));
                let ix = op == 0xDD;
                if op2 == 0xCB {
                    let op4 = usize::from(bus.read(at(3)));
                    if ix { t.ddcb[op4] } else { t.fdcb[op4] }
                } else if ix {
                    t.dd[usize::from(op2)]
                } else {
                    t.fd[usize::from(op2)]
                }
            }
            _ => t.main[usize::from(op)],
        })
    }

    /// Will an interrupt be taken now (enabled, not just after EI)?
    #[must_use]
    pub fn takes_interrupt(&self) -> bool {
        self.iff1 && !self.ei_delay
    }

    pub fn step(&mut self, bus: &mut dyn Bus) -> u32 {
        self.ei_delay = false;
        if self.halted {
            return 4;
        }
        self.refresh();
        let op = self.fetch(bus);
        match op {
            0xCB => self.cb(bus, Idx::Hl, 0),
            0xED => self.ed(bus),
            0xDD => self.indexed(bus, Idx::Ix),
            0xFD => self.indexed(bus, Idx::Iy),
            _ => self.main(bus, op, Idx::Hl),
        }
    }

    fn indexed(&mut self, bus: &mut dyn Bus, x: Idx) -> u32 {
        let op = self.fetch(bus);
        match op {
            0xCB => {
                let d = self.fetch(bus) as i8;
                self.cb(bus, x, d) + 8
            }
            0xDD | 0xFD | 0xED => {
                // a prefix that does nothing: the next one counts
                self.pc = self.pc.wrapping_sub(1);
                4
            }
            _ => self.main(bus, op, x) + 4,
        }
    }

    /// The instruction set without prefix (or with DD/FD: x).
    #[allow(clippy::too_many_lines)]
    fn main(&mut self, bus: &mut dyn Bus, op: u8, x: Idx) -> u32 {
        let hi = op >> 6;
        let y = (op >> 3) & 7;
        let z = op & 7;
        // (HL) operands become (IX+d)/(IY+d), H and L become IXH/IXL
        let uses_mem = |n: u8| n == 6;
        match hi {
            1 => {
                if op == 0x76 {
                    self.halted = true;
                    return 4;
                }
                let mut d = 0i8;
                if x != Idx::Hl && (uses_mem(y) || uses_mem(z)) {
                    d = self.fetch(bus) as i8;
                }
                // with (IX+d), H and L stay H and L
                let (rx, wx) = if uses_mem(y) || uses_mem(z) {
                    if uses_mem(z) { (x, Idx::Hl) } else { (Idx::Hl, x) }
                } else {
                    (x, x)
                };
                let v = self.reg(bus, z, rx, d);
                self.set_reg(bus, y, wx, d, v);
                return if uses_mem(y) || uses_mem(z) {
                    if x == Idx::Hl { 7 } else { 15 }
                } else {
                    4
                };
            }
            2 => {
                let mut d = 0i8;
                if x != Idx::Hl && uses_mem(z) {
                    d = self.fetch(bus) as i8;
                }
                let v = self.reg(bus, z, x, d);
                self.alu(y, v);
                return if uses_mem(z) {
                    if x == Idx::Hl { 7 } else { 15 }
                } else {
                    4
                };
            }
            _ => {}
        }
        match op {
            0x00 => 4,
            0x01 | 0x11 | 0x21 | 0x31 => {
                let v = self.fetch16(bus);
                self.set_rp(y >> 1, x, v);
                10
            }
            0x02 => {
                bus.write(self.bc(), self.a);
                7
            }
            0x12 => {
                bus.write(self.de(), self.a);
                7
            }
            0x0A => {
                self.a = bus.read(self.bc());
                7
            }
            0x1A => {
                self.a = bus.read(self.de());
                7
            }
            0x22 => {
                let a = self.fetch16(bus);
                Self::write16(bus, a, self.idx(x));
                16
            }
            0x2A => {
                let a = self.fetch16(bus);
                let v = Self::read16(bus, a);
                self.set_idx(x, v);
                16
            }
            0x32 => {
                let a = self.fetch16(bus);
                bus.write(a, self.a);
                13
            }
            0x3A => {
                let a = self.fetch16(bus);
                self.a = bus.read(a);
                13
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                let v = self.rp(y >> 1, x).wrapping_add(1);
                self.set_rp(y >> 1, x, v);
                6
            }
            0x0B | 0x1B | 0x2B | 0x3B => {
                let v = self.rp(y >> 1, x).wrapping_sub(1);
                self.set_rp(y >> 1, x, v);
                6
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                let a = self.idx(x);
                let b = self.rp(y >> 1, x);
                let r = self.add16(a, b);
                self.set_idx(x, r);
                11
            }
            _ if hi == 0 && z == 4 => {
                let d = if y == 6 && x != Idx::Hl { self.fetch(bus) as i8 } else { 0 };
                let v = self.reg(bus, y, x, d);
                let r = self.inc8(v);
                self.set_reg(bus, y, x, d, r);
                if y == 6 { if x == Idx::Hl { 11 } else { 19 } } else { 4 }
            }
            _ if hi == 0 && z == 5 => {
                let d = if y == 6 && x != Idx::Hl { self.fetch(bus) as i8 } else { 0 };
                let v = self.reg(bus, y, x, d);
                let r = self.dec8(v);
                self.set_reg(bus, y, x, d, r);
                if y == 6 { if x == Idx::Hl { 11 } else { 19 } } else { 4 }
            }
            _ if hi == 0 && z == 6 => {
                let d = if y == 6 && x != Idx::Hl { self.fetch(bus) as i8 } else { 0 };
                let v = self.fetch(bus);
                self.set_reg(bus, y, x, d, v);
                if y == 6 { if x == Idx::Hl { 10 } else { 15 } } else { 7 }
            }
            0x07 | 0x0F | 0x17 | 0x1F => {
                self.rot_a(y);
                4
            }
            0x27 => {
                self.daa();
                4
            }
            0x2F => {
                self.cpl();
                4
            }
            0x37 => {
                self.scf();
                4
            }
            0x3F => {
                self.ccf();
                4
            }
            0x08 => {
                self.ex_af();
                4
            }
            0x10 => {
                let d = self.fetch(bus) as i8;
                self.b = self.b.wrapping_sub(1);
                if self.b != 0 {
                    self.pc = self.pc.wrapping_add(d as u16);
                    13
                } else {
                    8
                }
            }
            0x18 => {
                let d = self.fetch(bus) as i8;
                self.pc = self.pc.wrapping_add(d as u16);
                12
            }
            0x20 | 0x28 | 0x30 | 0x38 => {
                let d = self.fetch(bus) as i8;
                if self.cond(y - 4) {
                    self.pc = self.pc.wrapping_add(d as u16);
                    12
                } else {
                    7
                }
            }
            _ if hi == 3 && z == 0 => {
                if self.cond(y) {
                    self.pc = self.pop(bus);
                    11
                } else {
                    5
                }
            }
            0xC9 => {
                self.pc = self.pop(bus);
                10
            }
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                let v = self.pop(bus);
                match y >> 1 {
                    3 => self.set_af(v),
                    n => self.set_rp(n, x, v),
                }
                10
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                let v = match y >> 1 {
                    3 => self.af(),
                    n => self.rp(n, x),
                };
                self.push(bus, v);
                11
            }
            _ if hi == 3 && z == 2 => {
                let a = self.fetch16(bus);
                if self.cond(y) {
                    self.pc = a;
                }
                10
            }
            0xC3 => {
                self.pc = self.fetch16(bus);
                10
            }
            _ if hi == 3 && z == 4 => {
                let a = self.fetch16(bus);
                if self.cond(y) {
                    self.push(bus, self.pc);
                    self.pc = a;
                    17
                } else {
                    10
                }
            }
            0xCD => {
                let a = self.fetch16(bus);
                self.push(bus, self.pc);
                self.pc = a;
                17
            }
            _ if hi == 3 && z == 6 => {
                let v = self.fetch(bus);
                self.alu(y, v);
                7
            }
            _ if hi == 3 && z == 7 => {
                self.push(bus, self.pc);
                self.pc = u16::from(y) * 8;
                11
            }
            0xD9 => {
                self.exx();
                4
            }
            0xE3 => {
                let v = Self::read16(bus, self.sp);
                Self::write16(bus, self.sp, self.idx(x));
                self.set_idx(x, v);
                19
            }
            0xE9 => {
                self.pc = self.idx(x);
                4
            }
            0xEB => {
                self.ex_de_hl();
                4
            }
            0xF9 => {
                self.sp = self.idx(x);
                6
            }
            0xD3 => {
                let _port = self.fetch(bus);
                11
            }
            0xDB => {
                let _port = self.fetch(bus);
                self.a = 0xFF;
                11
            }
            0xF3 => {
                self.di();
                4
            }
            0xFB => {
                self.iff1 = true;
                self.iff2 = true;
                self.ei_delay = true;
                4
            }
            _ => 4,
        }
    }

    fn cb(&mut self, bus: &mut dyn Bus, x: Idx, d: i8) -> u32 {
        let op = self.fetch(bus);
        let y = (op >> 3) & 7;
        let z = op & 7;
        // with (IX+d) the operand is always memory
        let n = if x == Idx::Hl { z } else { 6 };
        let v = self.reg(bus, n, x, d);
        let mem = n == 6;
        match op >> 6 {
            0 => {
                let r = self.rot(y, v);
                self.set_reg(bus, n, x, d, r);
                if x != Idx::Hl && z != 6 {
                    self.set_reg(bus, z, Idx::Hl, 0, r);
                }
                if mem { 15 } else { 8 }
            }
            1 => {
                self.bit(y, v);
                if mem { 12 } else { 8 }
            }
            2 => {
                let r = v & !(1 << y);
                self.set_reg(bus, n, x, d, r);
                if mem { 15 } else { 8 }
            }
            _ => {
                let r = v | (1 << y);
                self.set_reg(bus, n, x, d, r);
                if mem { 15 } else { 8 }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn ed(&mut self, bus: &mut dyn Bus) -> u32 {
        let op = self.fetch(bus);
        let y = (op >> 3) & 7;
        match op {
            0x40 | 0x48 | 0x50 | 0x58 | 0x60 | 0x68 | 0x78 => {
                let v = 0xFF;
                if y != 6 {
                    self.set_reg(bus, y, Idx::Hl, 0, v);
                }
                self.f = szp(v) | (self.f & C);
                12
            }
            0x41 | 0x49 | 0x51 | 0x59 | 0x61 | 0x69 | 0x79 => 12,
            0x42 | 0x52 | 0x62 | 0x72 => {
                let r = self.sbc16(self.hl(), self.rp(y >> 1, Idx::Hl));
                self.set_hl(r);
                15
            }
            0x4A | 0x5A | 0x6A | 0x7A => {
                let r = self.adc16(self.hl(), self.rp(y >> 1, Idx::Hl));
                self.set_hl(r);
                15
            }
            0x43 | 0x53 | 0x63 | 0x73 => {
                let a = self.fetch16(bus);
                Self::write16(bus, a, self.rp(y >> 1, Idx::Hl));
                20
            }
            0x4B | 0x5B | 0x6B | 0x7B => {
                let a = self.fetch16(bus);
                let v = Self::read16(bus, a);
                self.set_rp(y >> 1, Idx::Hl, v);
                20
            }
            0x44 | 0x4C | 0x54 | 0x5C | 0x64 | 0x6C | 0x74 | 0x7C => {
                self.neg();
                8
            }
            0x45 | 0x55 | 0x5D | 0x65 | 0x6D | 0x75 | 0x7D | 0x4D => {
                self.pc = self.pop(bus);
                self.iff1 = self.iff2;
                14
            }
            0x46 | 0x4E | 0x66 | 0x6E => {
                self.im = 0;
                8
            }
            0x56 | 0x76 => {
                self.set_im(1);
                8
            }
            0x5E | 0x7E => {
                self.im = 2;
                8
            }
            0x47 => {
                self.i = self.a;
                9
            }
            0x4F => {
                self.r = self.a;
                9
            }
            0x57 | 0x5F => {
                self.a = if op == 0x57 { self.i } else { self.r };
                self.f = (self.f & C) | (self.a & (S | X5 | X3)) | if self.a == 0 { Z } else { 0 } | if self.iff2 { PV } else { 0 };
                9
            }
            0x67 => {
                let m = bus.read(self.hl());
                let a = self.a;
                bus.write(self.hl(), (m >> 4) | (a << 4));
                self.a = (a & 0xF0) | (m & 0x0F);
                self.f = szp(self.a) | (self.f & C);
                18
            }
            0x6F => {
                let m = bus.read(self.hl());
                let a = self.a;
                bus.write(self.hl(), (m << 4) | (a & 0x0F));
                self.a = (a & 0xF0) | (m >> 4);
                self.f = szp(self.a) | (self.f & C);
                18
            }
            0xA0 | 0xA8 | 0xB0 | 0xB8 => {
                // LDI LDD LDIR LDDR
                if self.ld_block(bus, op & 0x08 != 0) && op & 0x10 != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                    21
                } else {
                    16
                }
            }
            0xA1 | 0xA9 | 0xB1 | 0xB9 => {
                // CPI CPD CPIR CPDR
                if self.cp_block(bus, op & 0x08 != 0) && op & 0x10 != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                    21
                } else {
                    16
                }
            }
            _ => 8,
        }
    }
}
