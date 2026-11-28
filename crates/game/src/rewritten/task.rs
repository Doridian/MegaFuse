//! The tasks: the game's routines run once a frame, in a list ordered by
//! priority (the scheduler, `func_0005C0`, runs it from the main loop).
//!
//! A task is a record of 0x30 bytes in work RAM, found by its index in
//! the ROM's table of their addresses (`D_00062E`, 50 of them).  Record 0
//! is the list's head and record 1 its tail; both have routines too
//! (`func_0004FE`, `func_000510`), run first and last.  The list is
//! linked both ways, by index and (forwards) by address too.

use crate::{Game, Regs};

/// The records' addresses by index (`D_00062E`, in the ROM).
const TASK_TABLE: u32 = 0x00062E;
/// How many records there are.
const TASKS: u8 = 50;
/// The first record a new task may take (0 and 1 are the head and the
/// tail).
const FIRST_FREE: u8 = 2;

/// The index of the task running.
const CURRENT: u32 = 0xFF80EE;
/// The priority of the task being made, then its index.
const NEW: u32 = 0xFF80EF;
/// How many tasks there are (the head and the tail not counted).
const COUNT: u32 = 0xFF80F0;
/// The list's head (record 0) and tail (record 1).
const HEAD: u32 = 0xFFFF80F2;
const TAIL: u32 = 0xFFFF8122;
/// Counts the tasks made (one more at each try, a byte).
const MADE: u32 = 0xFFA3FE;
/// The task area cleared at the start: from `COUNT`, 0x259 longs.
const AREA: u32 = 0xFF80F0;
const AREA_LONGS: u32 = 0x259;

/// The slot (`func_000B5A`'s records) the task running has: its index
/// and its address.
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;

/// A record's fields.
/// The next task's index (0xFF in the tail).
const NEXT: u32 = 0x00;
/// The previous task's index.
const PREV: u32 = 0x01;
/// Its priority: 0 the record is free, 0xFF the tail; the list is kept
/// in order of the low 7 bits; bit 7 set, the task may be ended to make
/// room for another.
const PRIORITY: u32 = 0x02;
/// Its slot's index (0xFF: none; a free record has 0).
const SLOT: u32 = 0x03;
/// The routine the scheduler calls (a long).
const ROUTINE: u32 = 0x04;
/// The next task's address (0 in the tail).
const NEXT_ADDR: u32 = 0x08;
/// A byte cleared when the task is made and when it ends.
const FLAG_1B: u32 = 0x1B;
/// A routine kept to be the task's routine again (`loc_000558`).
const KEPT_ROUTINE: u32 = 0x1C;

/// The flags a `func_0005F8` look-up leaves: X clear (the index x 4
/// doesn't carry out of the word).
const LOOKUP_X: bool = false;

impl Game {
    /// The address of task `index`'s record.
    fn task(&mut self, index: u8) -> u32 {
        self.read32(TASK_TABLE + 4 * u32::from(index))
    }

    /// `func_000372`: no tasks but the head and the tail, their routines
    /// the `rts` at 0x3D8; then `func_001C3A`.  Gives back the registers
    /// as it leaves them.
    #[allow(clippy::too_many_arguments)]
    pub fn r_000372(&mut self, _d2: u32, _d3: u32, _d4: u32, _d5: u32, _d7: u32, _a1: u32, _a2: u32, _a3: u32, _a4: u32, _a5: u32) -> (u32, u32, u32, u32, u32) {
        const NOTHING: u32 = 0x3D8;
        for k in 0..AREA_LONGS {
            self.write32(AREA + 4 * k, 0);
        }
        // the head: index 0, its next the tail
        self.write8(HEAD + PRIORITY, 1);
        self.write8(HEAD + NEXT, 1);
        self.write32(HEAD + NEXT_ADDR, TAIL);
        self.write8(HEAD + PREV, 0);
        // the tail: index 1, nothing after it
        self.write8(TAIL + PRIORITY, 0xFF);
        self.write8(TAIL + NEXT, 0xFF);
        self.write32(TAIL + NEXT_ADDR, 0);
        self.write8(TAIL + PREV, 0);
        self.r_0006f6(0, NOTHING);
        self.r_0006f6(1, NOTHING);
        self.write8(COUNT, 0);
        self.r_001c3a();
        // d0, d1, d6, a0, a6
        (0xFF, 0, 1, NOTHING, TAIL)
    }

    /// `func_0003D8` (the `rts` that ends `func_000372`): the routine of
    /// the list's head and tail; it does nothing.
    pub fn r_0003d8(&mut self, _regs: &mut Regs) {}

    /// `func_0003DA`: `func_000372`, the head's routine kept.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0003da(&mut self, d2: u32, d3: u32, d4: u32, d5: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32) {
        let head_routine = self.read32(HEAD + ROUTINE);
        let out = self.r_000372(d2, d3, d4, d5, d7, a1, a2, a3, a4, a5);
        self.write32(HEAD + ROUTINE, head_routine);
        out
    }

    /// `func_0003E8`: `func_000404` with its arguments on the stack (the
    /// priority, a byte, then the routine).
    pub fn r_0003e8(&mut self, d4: u32, d5: u32, d6: u32, priority: u8, routine: u32) -> (u32, u32, bool, bool) {
        self.r_000404(u32::from(priority), d4, d5, d6, routine)
    }

    /// `func_000404`: a new task running `routine` (a0) at `priority`
    /// (d0's low byte), linked in before the first task of the same
    /// priority or more (bit 7 aside).  A free record is taken, else
    /// the last task whose priority has bit 7 is ended for it (its slot
    /// freed, `func_000870`); if there is none, C is set and d6 is 0xFF.
    /// Gives back d6 (its low byte the new task's index), a6 (its
    /// record), X and C.
    pub fn r_000404(&mut self, d0: u32, _d4: u32, d5: u32, d6: u32, routine: u32) -> (u32, u32, bool, bool) {
        let priority = d0 as u8;
        let made = self.read8(MADE);
        self.write8(MADE, made.wrapping_add(1));
        self.write8(NEW, priority);

        let mut found = None;
        if (self.read8(COUNT) as i8) < TASKS as i8 {
            for index in FIRST_FREE..TASKS {
                let record = self.task(index);
                if self.read8(record + PRIORITY) == 0 {
                    found = Some((index, record));
                    break;
                }
            }
        }
        let (index, record) = match found {
            Some(free) => free,
            None => {
                // none free: the last task that may be ended, from the
                // tail back
                let mut index = self.read8(TAIL + PREV);
                loop {
                    let record = self.task(index);
                    if self.read8(record + PRIORITY) & 0x80 != 0 {
                        let slot = self.read8(record + SLOT);
                        if slot & 0x80 == 0 {
                            let slot_index = self.read8(SLOT_INDEX);
                            let slot_addr = self.read32(SLOT_ADDR);
                            self.r_000870(u32::from(slot), d5);
                            self.write32(SLOT_ADDR, slot_addr);
                            self.write8(SLOT_INDEX, slot_index);
                        }
                        self.end_task(index);
                        break (index, record);
                    }
                    index = self.read8(record + PREV);
                    if index == 0 {
                        return (0xFF, record, LOOKUP_X, true);
                    }
                }
            }
        };

        // the task it goes before
        let mut next = self.read8(HEAD + NEXT);
        let mut next_record = self.read32(HEAD + NEXT_ADDR);
        loop {
            let p = self.read8(next_record + PRIORITY);
            if p == 0xFF || p & 0x7F >= priority {
                break;
            }
            next = self.read8(next_record + NEXT);
            next_record = self.read32(next_record + NEXT_ADDR);
        }
        self.write8(record + NEXT, next);
        self.write32(record + NEXT_ADDR, next_record);
        let prev = self.read8(next_record + PREV);
        let prev_record = self.task(prev);
        self.write8(prev_record + NEXT, index);
        self.write32(prev_record + NEXT_ADDR, record);
        self.write8(record + PREV, prev);
        self.write8(next_record + PREV, index);

        self.write8(record + PRIORITY, priority);
        self.write32(record + ROUTINE, routine);
        self.write8(record + SLOT, 0xFF);
        self.write8(record + FLAG_1B, 0);
        let count = self.read8(COUNT);
        self.write8(COUNT, count.wrapping_add(1));
        self.write8(NEW, index);
        // X: the count's carry
        ((d6 & !0xFF) | u32::from(index), record, count == 0xFF, false)
    }

    /// `func_0004FE`: the head's routine.
    pub fn r_0004fe(&mut self, routine: u32) {
        self.r_0006f6(0, routine);
    }

    /// `func_000510`: the tail's routine.
    pub fn r_000510(&mut self, routine: u32) {
        self.r_0006f6(1, routine);
    }

    /// `func_000522`, the yield: the rest of the routine that calls it
    /// (from `resume`, its return address) is the running task's routine
    /// from now on, and the routine returns at once (the unwind, the
    /// last flag).  Gives back X and C (clear) too.
    pub fn r_000522(&mut self, _d6: u32, resume: u32) -> (bool, bool, bool) {
        let current = self.read8(CURRENT);
        let record = self.task(current);
        self.write32(record + ROUTINE, resume);
        (LOOKUP_X, false, true)
    }

    /// `func_00053E`: `routine` is the running task's routine from now
    /// on.  Gives back X and C (clear).
    pub fn r_00053e(&mut self, _d6: u32, routine: u32) -> (bool, bool) {
        let current = self.read8(CURRENT);
        let record = self.task(current);
        self.write32(record + ROUTINE, routine);
        (LOOKUP_X, false)
    }

    /// `loc_000558`: the routine kept in the task at `record` is its
    /// routine again.  Gives back C (clear).
    pub fn r_000558(&mut self, record: u32) -> bool {
        let routine = self.read32(record + KEPT_ROUTINE);
        self.write32(record + ROUTINE, routine);
        false
    }

    /// `func_000560`: the running task ends (`func_000564`).  Gives back
    /// d6 (its low byte the task's index), X and C.
    pub fn r_000560(&mut self, d5: u32, d6: u32, x: bool) -> (u32, bool, bool) {
        let d6 = (d6 & !0xFF) | u32::from(self.read8(CURRENT));
        let (x, c) = self.r_000564(d5, d6, x);
        (d6, x, c)
    }

    /// `func_000564`: task `d6` (its low byte; none if negative) ends:
    /// out of the list, its record cleared.  If it is the one running,
    /// the scheduler goes on from the one before.  Gives back X (the
    /// count's borrow; `x` if no task ended) and C (clear).
    pub fn r_000564(&mut self, _d5: u32, d6: u32, x: bool) -> (bool, bool) {
        let index = d6 as u8;
        if index & 0x80 != 0 {
            return (x, false);
        }
        (self.end_task(index), false)
    }

    /// Task `index` ends; true if the count was 0 (its borrow).
    fn end_task(&mut self, index: u8) -> bool {
        let record = self.task(index);
        self.write8(record + FLAG_1B, 0);
        let prev = self.read8(record + PREV);
        let prev_record = self.task(prev);
        let next = self.read8(record + NEXT);
        let next_record = self.read32(record + NEXT_ADDR);
        self.write8(prev_record + NEXT, next);
        self.write32(prev_record + NEXT_ADDR, next_record);
        self.write8(next_record + PREV, prev);
        if self.read8(CURRENT) == index {
            self.write8(CURRENT, prev);
        }
        let count = self.read8(COUNT);
        self.write8(COUNT, count.wrapping_sub(1));
        self.write8(record + PRIORITY, 0);
        self.write32(record + ROUTINE, 0);
        self.write8(record + PREV, 0);
        self.write8(record + NEXT, 0);
        self.write32(record + NEXT_ADDR, 0);
        count == 0
    }

    /// `func_0005C0`, the scheduler: each task's routine in turn, from
    /// the head to the tail.  A routine is called with d0 0, d5 its
    /// slot's index and a5 its address (kept in `SLOT_INDEX`,
    /// `SLOT_ADDR` too), a6 its record and d6 its index; the other
    /// registers go from one task to the next.
    #[allow(clippy::too_many_arguments)]
    pub fn r_0005c0(&mut self, d1: u32, d2: u32, d3: u32, d4: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) {
        let mut regs = Regs { d1, d2, d3, d4, d7, a1, a2, a3, a4, ..Regs::default() };
        let mut index = 0u8;
        loop {
            self.write8(CURRENT, index);
            let record = self.task(index);
            let routine = self.read32(record + ROUTINE);
            let slot = self.read8(record + SLOT);
            let (_, slot_addr, _) = self.r_000b5a(u32::from(slot));
            self.write8(SLOT_INDEX, slot);
            self.write32(SLOT_ADDR, slot_addr);
            regs.d0 = 0;
            regs.d5 = u32::from(slot);
            regs.d6 = (regs.d6 & !0xFF) | u32::from(index);
            regs.a0 = routine;
            regs.a5 = slot_addr;
            regs.a6 = record;
            // moveq #0,d0's flags; X from the slot's look-up
            (regs.x, regs.n, regs.z, regs.v, regs.c) = (false, false, true, false, false);
            self.call_family_16(routine, &mut regs);
            if regs.unwound || self.restarted.is_some() {
                break;
            }
            // from the one running now (the one before, if it ended)
            let current = self.read8(CURRENT);
            regs.a6 = self.task(current);
            let next = self.read8(regs.a6 + NEXT);
            regs.d6 = (regs.d6 & !0xFF) | u32::from(next);
            if next & 0x80 != 0 {
                break;
            }
            index = next;
        }
        let r = regs;
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d6, r.d7, r.a3, r.a4, r.a6)
    }

    /// `func_00060A`: a5 = task `d5`'s record; d5 is kept.
    pub fn r_00060a(&mut self, d5: u32) -> (u32, u32) {
        let a5 = self.task(d5 as u8);
        (d5, a5)
    }

    /// `func_00061C`: a4 = task `d4`'s record; d4 is kept.
    pub fn r_00061c(&mut self, d4: u32) -> (u32, u32) {
        let a4 = self.task(d4 as u8);
        (d4, a4)
    }

    /// `func_0006F6`: task `d6`'s routine; d6 is kept.  Gives back X.
    pub fn r_0006f6(&mut self, d6: u32, routine: u32) -> (u32, bool) {
        let record = self.task(d6 as u8);
        self.write32(record + ROUTINE, routine);
        (d6, LOOKUP_X)
    }
    /// `func_00121A`: the running task ends (`func_000560`) and its
    /// sprite record is freed (`func_000890`).  Gives back d6, X and C.
    pub fn r_00121a(&mut self, d5: u32, d6: u32, x: bool) -> (u32, bool, bool) {
        let (d6, x, _) = self.r_000560(d5, d6, x);
        let c = self.r_000890();
        (d6, x, c)
    }

    /// `func_001212` (a task's routine): unless `func_001BB4` says to
    /// stop here, the task's kept routine is its routine again
    /// (`loc_000558`).  Gives back C.
    pub fn r_001212(&mut self, regs: &mut Regs) {
        let (c, stop) = self.r_001bb4();
        regs.c = if stop { c } else { self.r_000558(regs.a6) };
    }

    /// `func_001222`: a task running `routine` at `priority`
    /// (`func_000404`) with a sprite record at `depth` (`func_000796`),
    /// its frame 0, the task's byte 3 its index; C set if either can't
    /// be had (the task ended again).  Gives back d5 and a5 (the
    /// sprite's index and record), d6 and a6 (the task's), X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_001222(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, depth: u8, priority: u8, routine: u32) -> (u32, u32, u32, u32, bool, bool) {
        /// The sprite record's frame, and the long written at its y.
        const FRAME: u32 = 0x04;
        const Y: u32 = 0x0A;
        let (d6, task, x, failed) = self.r_000404((d0 & !0xFF) | u32::from(priority), d4, d5, d6, routine);
        if failed {
            return (d5, d6, a5, task, x, true);
        }
        let (d5, sprite, x, failed) = self.r_000796((d0 & !0xFF) | u32::from(depth), d1, d2, d3, d4, d5, d6, d7, routine, a1, a2, a3, a4, a5, x);
        if failed {
            let (x, _) = self.r_000564(d5, d6, x);
            return (d5, d6, sprite, task, x, true);
        }
        self.write8(task + SLOT, d5 as u8);
        self.write8(sprite + FRAME, 0);
        // y 0, and 0x100 in the word after it
        self.write32(sprite + Y, 0x100);
        self.write8(task + 0x12, 0);
        self.write8(task + 0x13, 0);
        (d5, d6, sprite, task, x, false)
    }
}
