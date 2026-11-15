//! The Z80's side: the queue of commands the 68000 leaves in its RAM.

use crate::Game;

/// The Z80's bus request (bit 8 written: the 68000 wants the bus; bit
/// 0 read: set until the Z80 has let it go).
const BUSREQ: u32 = 0xA11100;
/// The Z80's RAM, as the 68000 sees it.
const Z80: u32 = 0xA00000;
/// While this byte isn't 0 the 68000 leaves the queue alone (the Z80 is
/// at it).
const QUEUE_BUSY: u32 = Z80 + 0x1C00;
/// A byte the 68000 reads back (`func_00B89E`).
const STATUS: u32 = Z80 + 0x1C1C;
/// The queue: 16 bytes from `QUEUE`, the first at `QUEUE_FIRST`, so
/// many after it as `QUEUE_COUNT` says.
const QUEUE_FIRST: u32 = Z80 + 0x1C2E;
const QUEUE_COUNT: u32 = Z80 + 0x1C2F;
const QUEUE: u32 = Z80 + 0x1C30;

impl Game {
    /// The Z80's bus, waited for.
    fn z80_bus(&mut self) {
        self.write16(BUSREQ, 0x100);
        while self.read8(BUSREQ) & 1 != 0 {
            self.pass(30);
        }
    }

    fn z80_release(&mut self) {
        self.write16(BUSREQ, 0);
    }

    /// `func_00B84E`: `command` (its low byte) to the end of the Z80's
    /// queue, once the Z80 isn't at it.
    pub fn r_00b84e(&mut self, command: u32) {
        loop {
            self.z80_bus();
            if self.read8(QUEUE_BUSY) == 0 {
                break;
            }
            self.z80_release();
            // a pause (32 turns of a `dbra`)
            self.pass(32 * 10 + 4);
        }
        let first = self.read8(QUEUE_FIRST);
        let count = self.read8(QUEUE_COUNT);
        let at = u32::from(first.wrapping_add(count) & 0xF);
        self.write8(QUEUE + at, command as u8);
        self.write8(QUEUE_COUNT, count.wrapping_add(1));
        self.z80_release();
    }

    /// `func_00B84C`: 0xFF to the Z80's queue.
    pub fn r_00b84c(&mut self) {
        self.r_00b84e(0xFFFF_FFFF);
    }

    /// `func_00349E`, `func_0034B6`: `func_00B84E` with interrupts
    /// masked.
    pub fn r_00349e(&mut self, command: u32) {
        let ipl = self.ipl();
        self.set_ipl(7);
        self.r_00b84e(command);
        self.set_ipl(ipl);
    }

    pub fn r_0034b6(&mut self, command: u32) {
        self.r_00349e(command);
    }

    /// `func_00B89E`: the Z80's byte at 0x1C1C, into `d0`'s low byte.
    pub fn r_00b89e(&mut self, d0: u32) -> u32 {
        self.z80_bus();
        let status = self.read8(STATUS);
        self.z80_release();
        (d0 & !0xFF) | u32::from(status)
    }

    /// `func_0034CE`: `func_00B89E` with interrupts masked.
    pub fn r_0034ce(&mut self, d0: u32) -> u32 {
        let ipl = self.ipl();
        self.set_ipl(7);
        let d0 = self.r_00b89e(d0);
        self.set_ipl(ipl);
        d0
    }
}
