//! The routines of a task an object puts at a cell (`func_0058F0`,
//! `e_placed.rs`): `func_005D46` while it stays or is pushed (its way of
//! moving in flag bits 0-2, `func_005E50`), `func_005C10` and
//! `loc_005C32` once thrown (`func_005BD2`), `func_005E02` once stopped
//! (`func_005E0E`); a count of frames (`TASK_TIMER`) runs down to its
//! end (`func_005DC8`).

use crate::{Game, Regs};

/// A put task's fields (see `e_placed.rs`): its position (words, x
/// then y; the high byte of x at `TASK_X_HIGH`), the object that moved
/// it last, a direction, the count of frames, a byte for `TIMER_OUT`,
/// flags, its owner, its cell or rate (a word).
const TASK_X_HIGH: u32 = 0x0E;
const TASK_Y: u32 = 0x10;
const TASK_MOVED_BY: u32 = 0x14;
const TASK_DIRECTION: u32 = 0x15;
const TASK_TIMER: u32 = 0x16;
const TASK_17: u32 = 0x17;
const TASK_FLAGS: u32 = 0x18;
const TASK_OWNER: u32 = 0x19;
const TASK_CELL: u32 = 0x1A;
/// Flag bits: its way of moving (0-2), what `loc_005C32` keeps
/// (`THROWN_KEEPS`), what keeps `func_005FC4` from drawing it in its
/// cell (`NOT_STILL`), bit 6 (its sprite shown: the cell cleared on
/// the plane), bit 7 (`func_005E8A` stops it in the middle of a cell).
const MOVING: u8 = 0x07;
const THROWN_KEEPS: u8 = 0xD8;
const PUSHED_KEEPS: u8 = 0x18;
const NOT_STILL: u8 = 0x27;
const SHOWN: u8 = 0x40;
const STOP_IN_MIDDLE: u8 = 0x80;
/// A sprite record's frame.
const SPRITE_FRAME: u32 = 0x04;

/// The running task's index.
const CURRENT_TASK: u32 = 0xFF80EE;
/// The slot (`func_000B5A`'s records) the running task has: index and
/// address.
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// The cell (column, row; one word) and its address in the cell map;
/// the cell's second byte, 0x200 on.
const CELL: u32 = 0xFF9F08;
const CELL_AT: u32 = 0xFF9ED6;
const SECOND: u32 = 0x200;
/// The map's width and height (in cells, bytes).
const WIDTH: u32 = 0xFF9EB2;
const HEIGHT: u32 = 0xFF9EB3;
/// A velocity (words, x then y; each a byte of pixels, then a byte of
/// fraction) for `func_001A88`, `func_001AA2`.
const VELOCITY: u32 = 0xFF9EC4;
const VELOCITY_X: u32 = 0xFF9EC4;
const VELOCITY_X_FRACTION: u32 = 0xFF9EC5;
const VELOCITY_Y: u32 = 0xFF9EC6;
const VELOCITY_Y_FRACTION: u32 = 0xFF9EC7;
/// Settings bits (`func_005D7C`: bits 2-3, bit 6) and another byte
/// whose bit 2 it tests.
const SETTINGS: u32 = 0xFF9540;
const BITS_A442: u32 = 0xFFA442;
/// Where `TASK_17` goes when the count runs out.
const TIMER_OUT: u32 = 0xFFAABE;
/// A table by `TASK_MOVED_BY` x 8 whose byte 3's bit 6 sets
/// `STOP_IN_MIDDLE`.
const BY_MOVER: u32 = 0xFFFF_AD0A;
/// A table of bytes from 0xFFAC00 by the index `func_0066BE` gives.
const TABLE_AC00: u32 = 0xFFFF_AC00;

/// A thrown task's pixels a frame by direction (`D_005CF2` x,
/// `D_005CF6` y; bytes in the ROM), its rate when it lands on a cell
/// it can't stay on.
const THROWN_X: u32 = 0x005CF2;
const THROWN_Y: u32 = 0x005CF6;
const BOUNCE_RATE: u16 = 0xFE80;
/// The routines a task goes on with: `loc_005C32` once its sprite is
/// shown, `func_005D46` when it lands, `func_005E02` once stopped.
const FLYING: u32 = 0x005C32;
const PUT: u32 = 0x005D46;
const STOPPED: u32 = 0x005E02;
/// The routines by way of moving (1-3; longs in the ROM right after
/// `func_005E50`, from `func_005E50` + 0x18 by the way x 4).
const WAYS: u32 = 0x005E68;
/// Velocities by direction (longs: x, y): pushed (`D_005EEC`) and
/// `func_005EFC`'s (`D_005F2A`).
const PUSHED: u32 = 0x005EEC;
const SLOW: u32 = 0x005F2A;
/// The tiles by `func_006074`'s frame (`D_0060C0`, words in the ROM).
const TILES: u32 = 0x0060C0;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

impl Game {
    /// `func_005C10`, a thrown task's routine: once its sprite is shown
    /// (`SHOWN`) its routine is `FLYING` from the next frame; now it is
    /// drawn (`func_005FC4`).
    /// No script calls it: reviewed by reading.
    pub fn r_005c10(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        if self.read8(r.a6 + TASK_FLAGS) & SHOWN != 0 {
            self.r_00053e(r.d6, FLYING);
        }
        let d0 = self.r_0015f8(r.a6);
        let x = self.r_001652();
        (r.d0, r.d5, r.a0, r.a5, r.x, r.c) = self.r_005fc4(d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6, x);
        r.unwound = false;
    }

    /// `loc_005C32`, a thrown task's routine: it moves (`func_005CD0`)
    /// and falls (`func_001B14`, `func_001ACA`).  When it comes down on
    /// a cell whose byte is 5 or more (and, for 5, whose byte in
    /// `TABLE_AC00` is not 0x8E) it lands: in the middle of the cell,
    /// an object there hit (`func_002D3E`), its routine `PUT` from the
    /// next frame.  Otherwise it bounces (`BOUNCE_RATE`) and its sprite
    /// is drawn (`func_00601E`, `func_001B94`, `func_001B74`).
    /// No script calls it: reviewed by reading.
    pub fn r_005c32(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d1, r.d2) = self.r_005cd0(r.d0, r.d1, r.d2, r.a6);
        let d0 = self.r_001b14(r.a6);
        let (d0, x, up) = self.r_001aca(d0, r.a6);
        (r.d0, r.x) = (d0, x);
        if !up {
            if self.e_lands(r) {
                return;
            }
            self.write16(r.a6 + TASK_CELL, BOUNCE_RATE);
        }
        r.d5 = low_byte(r.d5, self.read8(SLOT_INDEX));
        (r.d0, r.d5, r.a0, r.a5, r.x, r.c) = self.r_00601e(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a6);
        (r.d0, _, _) = self.r_001b94(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b74(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
    }

    /// `loc_005C32` coming down: true if the task lands (its routine
    /// `PUT` from now on), false if it bounces.
    fn e_lands(&mut self, r: &mut Regs) -> bool {
        self.r_00187c(r.a6);
        r.d0 = self.r_0015f8(r.a6);
        let (cell, x) = self.r_00167a();
        (r.a0, r.x) = (cell, x);
        let byte = self.read8(cell);
        r.d0 = low_byte(r.d0, byte);
        if byte < 5 {
            return false;
        }
        if byte == 5 {
            r.d0 = self.r_0015f8(r.a6);
            (r.d1, _, _) = self.r_0066be(r.d1, r.d2);
            r.a0 = TABLE_AC00;
            let entry = self.read8(TABLE_AC00.wrapping_add(r.d1 as u16 as i16 as u32));
            r.d0 = low_byte(r.d0, entry);
            let current = self.read8(CURRENT_TASK);
            (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
            if entry == 0x8E {
                return false;
            }
        }
        let (d0, hit) = self.r_00191c(r.a6, r.x);
        r.d0 = d0;
        if hit {
            (r.d0, r.a0) = self.r_002d3e(r.d0, r.d4, r.d5, r.a0);
        }
        r.d0 = low_byte(r.d0, THROWN_KEEPS);
        let flags = self.read8(r.a6 + TASK_FLAGS);
        self.write8(r.a6 + TASK_FLAGS, flags & THROWN_KEEPS);
        let cell = self.read16(CELL);
        self.write16(r.a6 + TASK_CELL, cell);
        self.r_00167a();
        (r.d0, r.a0) = self.r_00168c(r.d0);
        (r.x, r.c) = self.r_00053e(r.d6, PUT);
        true
    }

    /// `func_005CD0`: the task at a6 moved 2 pixels a frame in the
    /// direction of its flag bits 0-1 (`THROWN_X`, `THROWN_Y`), coming
    /// round at the map's edges.  Gives back d1 and d2.
    /// No script calls it: reviewed by reading.
    pub fn r_005cd0(&mut self, d0: u32, d1: u32, d2: u32, a6: u32) -> (u32, u32) {
        let direction = self.read8(a6 + TASK_FLAGS) & 3;
        let d0 = low_word(d0, u16::from(direction));
        let x = self.read8(THROWN_X + u32::from(direction));
        self.write8(VELOCITY_X, x);
        let y = self.read8(THROWN_Y + u32::from(direction));
        self.write8(VELOCITY_Y, y);
        self.write8(VELOCITY_X_FRACTION, 0);
        self.write8(VELOCITY_Y_FRACTION, 0);
        // x's high byte against the width in 256 pixels
        let width = self.read8(WIDTH) >> 4;
        let d2 = low_byte(d2, width);
        let (d0, d1, _, before, _) = self.r_001a88(d0, d1, a6);
        let high = self.read8(a6 + TASK_X_HIGH);
        let mut d0 = d0;
        if before {
            self.write8(a6 + TASK_X_HIGH, high.wrapping_add(width));
        } else {
            d0 = low_byte(d0, high);
            if high >= width {
                self.write8(a6 + TASK_X_HIGH, 0);
            }
        }
        let height = u16::from(self.read8(HEIGHT)) << 4;
        let d2 = low_word(d2, height);
        let (_, d1, _, before, _) = self.r_001aa2(d0, d1, a6);
        let y = self.read16(a6 + TASK_Y);
        if before {
            self.write16(a6 + TASK_Y, y.wrapping_add(height));
        } else if y >= height {
            self.write16(a6 + TASK_Y, y & 0xF);
        }
        (d1, d2)
    }

    /// `func_005D46`, a put task's routine: gone when its cell's byte
    /// is 0 (`func_005DC8`); else it moves by its way (`func_005E50`),
    /// its cell's routine runs (`func_005378`), and on a cell whose
    /// byte is 7 or more it stops (timer 3 at most, routine `STOPPED`);
    /// otherwise the cell's byte is 1 and the count runs
    /// (`func_005D7C`).
    pub fn r_005d46(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = self.r_0015f8(r.a6);
        let (cell, x) = self.r_00167a();
        (r.a0, r.x) = (cell, x);
        let byte = self.read8(cell);
        r.d0 = low_byte(r.d0, byte);
        if byte == 0 {
            return self.e_gone(r);
        }
        (r.d0, r.d1, r.d6, r.a2, r.a6, r.x) = self.r_005e50(r.d0, r.d1, r.d2, r.d6, r.a2, r.a6, r.x);
        (r.d1, r.d6, r.a6, r.x) = self.r_005378(r.d0, r.d1, r.d6, r.a6, r.x);
        r.a0 = self.read32(CELL_AT);
        let byte = self.read8(r.a0);
        r.d0 = low_byte(r.d0, byte);
        if byte >= 7 {
            if self.read8(r.a6 + TASK_TIMER) >= 3 {
                self.write8(r.a6 + TASK_TIMER, 3);
            }
            self.r_00187c(r.a6);
            self.write8(r.a6 + TASK_FLAGS, 0);
            (r.x, r.c) = self.r_00053e(r.d6, STOPPED);
            return;
        }
        if byte == 0 {
            return self.e_gone(r);
        }
        r.a0 = self.r_00197a();
        self.e_counted(r);
    }

    /// `func_005DC8` on the family's registers.
    fn e_gone(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x, r.c) = self.r_005dc8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a5, r.a6);
    }

    /// `func_005D7C` on the family's registers.
    fn e_counted(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x, r.c) = self.r_005d7c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x);
    }

    /// `func_005D7C` (`.L005D7C`): the task's cell kept, it is drawn
    /// (`func_005FC4`), and its count runs down a frame, to its end
    /// (`TIMER_OUT` = `TASK_17`, `func_005DC8`).  The count stops at 1
    /// when `SETTINGS` bits 2-3 are set (it goes on from 1 only), and
    /// at 4 when `BITS_A442` bit 2 is set, `SETTINGS` bit 6 clear and
    /// the owner is object 0.  Gives back d0, d1, d2, d4, d5, d6, a0,
    /// a4, a5, a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_005d7c(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let cell = self.read16(CELL);
        self.write16(a6 + TASK_CELL, cell);
        let (d0, d5, a0, a5, x, _) = self.r_005fc4(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a5, a6, x);
        let bits = self.read8(SETTINGS) & 0x0C;
        let d0 = low_byte(d0, bits);
        let timer = self.read8(a6 + TASK_TIMER);
        if bits != 0 {
            if timer != 1 {
                return (d0, d1, d2, d4, d5, d6, a0, a4, a5, a6, x, timer < 1);
            }
        } else if self.read8(BITS_A442) & 0x04 != 0
            && self.read8(SETTINGS) & 0x40 == 0
            && self.read8(a6 + TASK_OWNER) == 0
            && timer >= 4
        {
            return (d0, d1, d2, d4, d5, d6, a0, a4, a5, a6, x, false);
        }
        let left = timer.wrapping_sub(1);
        self.write8(a6 + TASK_TIMER, left);
        if left != 0 {
            // subq's borrow
            let borrow = timer == 0;
            return (d0, d1, d2, d4, d5, d6, a0, a4, a5, a6, borrow, borrow);
        }
        let t17 = self.read8(a6 + TASK_17);
        self.write8(TIMER_OUT, t17);
        self.r_005dc8(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a5, a6)
    }

    /// `func_005DC8` (`.L005DC8`): the task taken out and ended
    /// (`func_005A4E`), then `func_0060CC`.  Gives back d0, d1, d2, d4,
    /// d5, d6, a0, a4, a5, a6, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_005dc8(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let (d0, d4, d6, a0, a4, x, c) = self.r_005a4e(d0, d4, d5, d6, a6);
        let (d0, d1, d2, d5, d6, a0, a5, a6, x, c) = self.r_0060cc(d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6, x, c);
        (d0, d1, d2, d4, d5, d6, a0, a4, a5, a6, x, c)
    }

    /// `func_005E02`, a stopped task's routine: its count runs
    /// (`func_005D7C`).
    pub fn r_005e02(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.d0 = self.r_0015f8(r.a6);
        (r.a0, r.x) = self.r_00167a();
        self.e_counted(r);
    }

    /// `func_005E50`: the task's way of moving (its flag bits 0-2; none
    /// for 0) run from `WAYS`.  Gives back d0, d1, d6, a2, a6 and X.
    #[allow(clippy::too_many_arguments)]
    pub fn r_005e50(&mut self, d0: u32, d1: u32, d2: u32, d6: u32, a2: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, u32, bool) {
        let way = self.read8(a6 + TASK_FLAGS) & MOVING;
        if way == 0 {
            return (low_byte(d0, 0), d1, d6, a2, a6, x);
        }
        let offset = u16::from(way) * 4;
        let routine = self.read32(WAYS + u32::from(offset));
        let mut r = Regs { d0: low_word(d0, offset), d1, d2, d6, a0: routine, a2, a6, ..Regs::default() };
        self.call_family_21(routine, &mut r);
        (r.d0, r.d1, r.d6, r.a2, r.a6, r.x)
    }

    /// `func_005E78` (way 3): a6 the running task's record again, the
    /// cell its own.
    /// No script calls it: reviewed by reading.
    pub fn r_005e78(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, _) = self.r_0005f8(low_byte(r.d6, current));
        r.d0 = self.r_0015f8(r.a6);
        (r.a0, r.x) = self.r_00167a();
    }

    /// `func_005E8A` (way 1, pushed): it moves at `PUSHED` in its
    /// direction (`func_005F3A`) and stops when it can't (in the middle
    /// of its cell); with `STOP_IN_MIDDLE` (set when the mover's entry
    /// in `BY_MOVER` has bit 6 of its byte 3) it stops at the next
    /// middle of a cell.
    /// No script calls it: reviewed by reading.
    pub fn r_005e8a(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.a2 = PUSHED;
        let blocked;
        (r.d0, r.d1, r.d6, r.a6, blocked) = self.r_005f3a(r.d0, r.d1, r.d2, r.d6, r.a2);
        r.c = blocked;
        if blocked {
            self.r_00187c(r.a6);
            r.d0 = self.r_0015f8(r.a6);
            let flags = self.read8(r.a6 + TASK_FLAGS);
            self.write8(r.a6 + TASK_FLAGS, flags & PUSHED_KEEPS);
            (r.a0, r.x) = self.r_00167a();
            return;
        }
        let mover = u16::from(self.read8(r.a6 + TASK_MOVED_BY)) << 3;
        r.d0 = low_word(r.d0, mover);
        r.x = false;
        r.a0 = BY_MOVER;
        let flags = self.read8(r.a6 + TASK_FLAGS);
        if self.read8(BY_MOVER.wrapping_add(3).wrapping_add(mover as i16 as u32)) & 0x40 != 0 {
            self.write8(r.a6 + TASK_FLAGS, flags | STOP_IN_MIDDLE);
        }
        if self.read8(r.a6 + TASK_FLAGS) & STOP_IN_MIDDLE == 0 {
            return;
        }
        let middle;
        (r.d0, middle) = self.r_001896(r.d0, r.a6);
        if middle {
            (r.d0, r.x) = self.r_005edc(r.a6);
        }
    }

    /// `func_005EDC` (`.L005EDC`): the task stops: its flags but bits
    /// 3-4 cleared, the cell its own.  Gives back d0 and X.
    /// No script calls it: reviewed by reading.
    pub fn r_005edc(&mut self, a6: u32) -> (u32, bool) {
        let d0 = self.r_0015f8(a6);
        let flags = self.read8(a6 + TASK_FLAGS);
        self.write8(a6 + TASK_FLAGS, flags & PUSHED_KEEPS);
        let (_, x) = self.r_00167a();
        self.r_005eea();
        (d0, x)
    }

    /// `func_005EEA` (`.L005EEA`): returns.
    /// No script calls it: reviewed by reading.
    pub fn r_005eea(&mut self) {}

    /// `func_005EFC` (way 2): the running task moves at `SLOW` in its
    /// direction while the next cell's second byte is not 0 and it can
    /// (`func_005F4E`), else it stops (`func_005EDC`).
    /// No script calls it: reviewed by reading.
    pub fn r_005efc(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        r.a2 = SLOW;
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, _) = self.r_0005f8(low_byte(r.d6, current));
        (r.d0, _) = self.r_00168c(r.d0);
        let direction = self.read8(r.a6 + TASK_DIRECTION);
        r.d0 = low_byte(r.d0, direction);
        let x = self.r_001726(r.d0, r.a6);
        r.a0 = self.read32(CELL_AT);
        let second = self.read8(r.a0 + SECOND);
        r.d0 = low_byte(r.d0, second);
        if second != 0 {
            let blocked;
            (r.d0, r.d1, r.x, blocked) = self.r_005f4e(r.d0, r.d1, r.d2, r.a2, r.a6, x);
            r.c = blocked;
            if !blocked {
                return;
            }
        }
        (r.d0, r.x) = self.r_005edc(r.a6);
    }

    /// `func_005F3A`: the running task (a6 from `CURRENT_TASK`), the
    /// cell's byte made again from its source (`func_00168C`), the cell
    /// next to it in its direction (`func_001726`), and
    /// `func_005F4E`.  Gives back d0, d1, d6, a6 and C.
    /// No script calls it: reviewed by reading.
    pub fn r_005f3a(&mut self, d0: u32, d1: u32, d2: u32, d6: u32, a2: u32) -> (u32, u32, u32, u32, bool) {
        let current = self.read8(CURRENT_TASK);
        let (d6, a6, _) = self.r_0005f8(low_byte(d6, current));
        let (d0, _) = self.r_00168c(d0);
        let direction = self.read8(a6 + TASK_DIRECTION);
        let d0 = low_byte(d0, direction);
        let x = self.r_001726(d0, a6);
        let (d0, d1, _, blocked) = self.r_005f4e(d0, d1, d2, a2, a6, x);
        (d0, d1, d6, a6, blocked)
    }

    /// `func_005F4E`: the task at a6 moves a frame in its direction at
    /// the velocity from table `a2` (`func_005FAC`), unless the cell at
    /// `CELL_AT` has a byte below 5, an object next to it that way
    /// (`func_0018C4`), or a 5 that `func_0067C8` says no to.  Gives
    /// back d0, d1, X and C (set when it can't).
    /// No script calls it: reviewed by reading.
    pub fn r_005f4e(&mut self, d0: u32, d1: u32, d2: u32, a2: u32, a6: u32, x: bool) -> (u32, u32, bool, bool) {
        let at = self.read32(CELL_AT);
        let byte = self.read8(at);
        let d0 = low_byte(d0, byte);
        if byte < 5 {
            return (d0, d1, x, true);
        }
        let direction = self.read8(a6 + TASK_DIRECTION);
        let (d0, d1, _, x, taken) = self.r_0018c4(low_byte(d0, direction), d1, at, a6);
        if taken {
            return (d0, d1, x, true);
        }
        let at = self.read32(CELL_AT);
        let byte = self.read8(at);
        let mut d0 = low_byte(d0, byte);
        if byte == 5 {
            let (d0b, x, no) = self.r_0067c8(d0, d1, d2);
            if no {
                return (d0b, d1, x, true);
            }
            d0 = d0b;
        }
        let d1 = self.r_005fac(d0, d1, a2, a6);
        let d0 = self.r_0015f8(a6);
        let (_, x) = self.r_00167a();
        (d0, d1, x, false)
    }

    /// `func_005FAC`: the velocity for the task's direction from table
    /// `a2` (longs: x, y) to `VELOCITY`, and the task moved by it, y
    /// then x (`func_001AA2`, `func_001A88`).  Gives back d1.
    /// No script calls it: reviewed by reading.
    pub fn r_005fac(&mut self, d0: u32, d1: u32, a2: u32, a6: u32) -> u32 {
        let direction = self.read8(a6 + TASK_DIRECTION) as i8 as i16 as u16;
        let offset = direction.wrapping_mul(4);
        let velocity = self.read32(a2.wrapping_add(offset as i16 as u32));
        self.write32(VELOCITY, velocity);
        let (d0, d1, ..) = self.r_001aa2(low_word(d0, offset), d1, a6);
        let (_, d1, ..) = self.r_001a88(d0, d1, a6);
        d1
    }

    /// `func_005FC4`: unless the objects are stopped (`func_001BB4`;
    /// then it returns from its caller too), the task's sprite: when it
    /// is shown and `SHOWN` is set the cell is cleared on the plane;
    /// while the task moves (`NOT_STILL`) or is off the middle of its
    /// cell the sprite is drawn (`func_00601E`), else the sprite is
    /// hidden (frame 0) and the cell drawn with the tiles for
    /// `func_006074`'s frame (`TILES`, `func_005294`).  Gives back d0,
    /// d5, a0, a5, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_005fc4(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32, x: bool) -> (u32, u32, u32, u32, bool, bool) {
        let (c, stop) = self.r_001bb4();
        if stop {
            return (d0, d5, a0, a5, x, c);
        }
        self.r_001278();
        let slot = self.read8(SLOT_INDEX);
        let d5 = low_byte(d5, slot);
        let a5 = self.read32(SLOT_ADDR);
        // (the X and C this leaves are overwritten on every path)
        if self.read8(a5 + SPRITE_FRAME) != 0 && self.read8(a6 + TASK_FLAGS) & SHOWN != 0 {
            self.r_0052f0(d0);
        }
        let moving = self.read8(a6 + TASK_FLAGS) & NOT_STILL;
        let d0 = low_byte(d0, moving);
        if moving != 0 {
            return self.r_00601e(d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a6);
        }
        let (d0, middle) = self.r_001896(d0, a6);
        if !middle {
            return self.r_00601e(d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a6);
        }
        self.write8(a5 + SPRITE_FRAME, 0);
        let (d0, _) = self.r_006074(d0, a6);
        let frame = (d0 as u8).wrapping_add(d0 as u8);
        let offset = frame as i8 as i16 as u16;
        let tiles = self.read16(TILES.wrapping_add(offset as i16 as u32));
        let d0 = low_word(d0, tiles);
        let (x, c) = self.r_005294(d0, d1);
        (d0, d5, TILES, a5, x, c)
    }
}
