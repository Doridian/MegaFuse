//! What the translated code calls besides the routines: the bus, and the
//! instructions whose result and flags are easier written once than
//! inline (shifts and rotates, `divu`).

use crate::Game;

const RAM_FROM: u32 = 0xE0_0000;
const ROM_TO: u32 = 0x40_0000;

impl Game {
    #[inline]
    #[must_use]
    pub fn read8(&mut self, addr: u32) -> u8 {
        let a = addr & 0xFF_FFFF;
        if a >= RAM_FROM {
            self.ram[(a & 0xFFFF) as usize]
        } else if a < ROM_TO {
            self.rom.get(a as usize).copied().unwrap_or(0xFF)
        } else {
            self.hw.read8(a)
        }
    }

    #[inline]
    #[must_use]
    pub fn read16(&mut self, addr: u32) -> u16 {
        let a = addr & 0xFF_FFFF;
        if a >= RAM_FROM || a < ROM_TO {
            u16::from_be_bytes([self.read8(a), self.read8(a + 1)])
        } else {
            self.hw.read16(a)
        }
    }

    #[inline]
    #[must_use]
    pub fn read32(&mut self, addr: u32) -> u32 {
        let hi = self.read16(addr);
        let lo = self.read16(addr.wrapping_add(2));
        u32::from(hi) << 16 | u32::from(lo)
    }

    /// Debugging: MEGAFUSE_TRAP=ADDR (hex) panics, with a backtrace,
    /// at the first write of something other than zero that covers the
    /// byte at ADDR (after MEGAFUSE_TRAP_AFTER cycles).
    fn trap(&self, a: u32, n: u32, value: u32) {
        static TRAP: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
        let t = TRAP.get_or_init(|| {
            std::env::var("MEGAFUSE_TRAP").ok().and_then(|v| u32::from_str_radix(&v, 16).ok())
        });
        static AFTER: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
        let after = *AFTER.get_or_init(|| {
            std::env::var("MEGAFUSE_TRAP_AFTER").ok().and_then(|v| v.parse().ok()).unwrap_or(0)
        });
        if let Some(t) = *t {
            if self.cycles >= after && value != 0 && (t & 0xFF_FFFF).wrapping_sub(a) < n {
                panic!("a write to {a:06X} (cycle {})", self.cycles);
            }
        }
    }

    #[inline]
    pub fn write8(&mut self, addr: u32, value: u8) {
        let a = addr & 0xFF_FFFF;
        self.trap(a, 1, u32::from(value));
        if a >= RAM_FROM {
            self.ram[(a & 0xFFFF) as usize] = value;
        } else if a >= ROM_TO {
            self.hw.write8(a, value);
        }
    }

    #[inline]
    pub fn write16(&mut self, addr: u32, value: u16) {
        let a = addr & 0xFF_FFFF;
        self.trap(a, 2, u32::from(value));
        if a >= RAM_FROM {
            let [hi, lo] = value.to_be_bytes();
            self.ram[(a & 0xFFFF) as usize] = hi;
            self.ram[((a + 1) & 0xFFFF) as usize] = lo;
        } else if a >= ROM_TO {
            self.hw.write16(a, value);
            if a & 0xFF_FFF8 == 0xC0_0000 {
                self.dma();
            }
        }
    }

    #[inline]
    pub fn write32(&mut self, addr: u32, value: u32) {
        self.write16(addr, (value >> 16) as u16);
        self.write16(addr.wrapping_add(2), value as u16);
    }

    /// `move.w sr,...`: supervisor mode, the interrupt mask and the flags.
    #[must_use]
    pub fn sr_word(&self, x: bool, n: bool, z: bool, v: bool, c: bool) -> u16 {
        0x2000
            | u16::from(self.ipl) << 8
            | u16::from(x) << 4
            | u16::from(n) << 3
            | u16::from(z) << 2
            | u16::from(v) << 1
            | u16::from(c)
    }

    /// `divu.w`: the 32-bit dividend by a 16-bit divisor, the remainder
    /// in the high word; overflow (a quotient over 16 bits) leaves the
    /// dividend as it was and sets V.
    #[must_use]
    pub fn divu(&self, dividend: u32, divisor: u16) -> (u32, bool) {
        if divisor == 0 {
            panic!("divu by zero");
        }
        let q = dividend / u32::from(divisor);
        let r = dividend % u32::from(divisor);
        if q > 0xFFFF { (dividend, true) } else { (r << 16 | q, false) }
    }
}

/// Shifts and rotates: (result, C, V, X out); `count` is already reduced.
macro_rules! shifts {
    ($t:ty, $asl:ident, $asr:ident, $lsl:ident, $lsr:ident, $rol:ident,
     $ror:ident, $roxl:ident, $roxr:ident) => {
        impl Game {
            #[must_use]
            pub fn $asl(&self, d: $t, count: u32) -> ($t, bool, bool, bool) {
                let msb = <$t>::MAX ^ (<$t>::MAX >> 1);
                let (mut r, mut c, mut v) = (d, false, false);
                for _ in 0..count {
                    c = r & msb != 0;
                    let next = r << 1;
                    v |= (next & msb != 0) != c;
                    r = next;
                }
                (r, c, v, c)
            }
            #[must_use]
            pub fn $asr(&self, d: $t, count: u32) -> ($t, bool, bool, bool) {
                let msb = <$t>::MAX ^ (<$t>::MAX >> 1);
                let (mut r, mut c) = (d, false);
                for _ in 0..count {
                    c = r & 1 != 0;
                    r = (r >> 1) | (r & msb);
                }
                (r, c, false, c)
            }
            #[must_use]
            pub fn $lsl(&self, d: $t, count: u32) -> ($t, bool, bool, bool) {
                let msb = <$t>::MAX ^ (<$t>::MAX >> 1);
                let (mut r, mut c) = (d, false);
                for _ in 0..count {
                    c = r & msb != 0;
                    r <<= 1;
                }
                (r, c, false, c)
            }
            #[must_use]
            pub fn $lsr(&self, d: $t, count: u32) -> ($t, bool, bool, bool) {
                let (mut r, mut c) = (d, false);
                for _ in 0..count {
                    c = r & 1 != 0;
                    r >>= 1;
                }
                (r, c, false, c)
            }
            #[must_use]
            pub fn $rol(&self, d: $t, count: u32) -> ($t, bool, bool, bool) {
                let r = d.rotate_left(count % <$t>::BITS);
                (r, count > 0 && r & 1 != 0, false, false)
            }
            #[must_use]
            pub fn $ror(&self, d: $t, count: u32) -> ($t, bool, bool, bool) {
                let msb = <$t>::MAX ^ (<$t>::MAX >> 1);
                let r = d.rotate_right(count % <$t>::BITS);
                (r, count > 0 && r & msb != 0, false, false)
            }
            #[must_use]
            pub fn $roxl(&self, d: $t, count: u32, x: bool) -> ($t, bool, bool, bool) {
                let msb = <$t>::MAX ^ (<$t>::MAX >> 1);
                let (mut r, mut x) = (d, x);
                for _ in 0..count {
                    let out = r & msb != 0;
                    r = (r << 1) | <$t>::from(x);
                    x = out;
                }
                (r, x, false, x)
            }
            #[must_use]
            pub fn $roxr(&self, d: $t, count: u32, x: bool) -> ($t, bool, bool, bool) {
                let msb = <$t>::MAX ^ (<$t>::MAX >> 1);
                let (mut r, mut x) = (d, x);
                for _ in 0..count {
                    let out = r & 1 != 0;
                    r = (r >> 1) | if x { msb } else { 0 };
                    x = out;
                }
                (r, x, false, x)
            }
        }
    };
}

shifts!(u8, asl8, asr8, lsl8, lsr8, rol8, ror8, roxl8, roxr8);
shifts!(u16, asl16, asr16, lsl16, lsr16, rol16, ror16, roxl16, roxr16);
shifts!(u32, asl32, asr32, lsl32, lsr32, rol32, ror32, roxl32, roxr32);
