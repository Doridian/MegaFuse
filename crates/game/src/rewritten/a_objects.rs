//! The objects: 20 records of 0x40 bytes from `OBJECTS` (their
//! addresses also in the ROM's table `D_001D1E`, `func_001CFA`), each
//! with a routine that a task runs once a frame (`func_001CA0`), and a
//! search for the one at a cell (`func_00192A`).

use crate::{Game, Regs};

/// The records, `OBJECT_SIZE` bytes each, and how many.
const OBJECTS: u32 = 0xFFFF_A432;
const OBJECT_SIZE: u32 = 0x40;
const COUNT: u8 = 20;
/// How many bytes of a record `func_001C56` clears.
const CLEARED: u32 = 0x2C;

/// An object's fields: not 0 while it is in use; its routine (a long);
/// its sprite record's address (a long) and index; a byte whose sign
/// bit keeps `func_00192A` from finding it; its position (words, x then
/// y; one long).
const OBJ_USED: u32 = 0x00;
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
const OBJ_FLAGS: u32 = 0x10;
const OBJ_POSITION: u32 = 0x18;
/// A task's position (a long, as an object's).
const TASK_POSITION: u32 = 0x0E;

/// The object being run (its index, a byte, and its address).
const RUNNING: u32 = 0xFFA42C;
const RUNNING_ADDR: u32 = 0xFFA42E;
/// The sprite record of the routine being run (index, address:
/// `task.rs`, `sprite.rs`).
const SLOT_INDEX: u32 = 0xFF8A57;
const SLOT_ADDR: u32 = 0xFF8A5A;
/// Cleared each time the objects are run.
const CLEARED_EACH_FRAME: u32 = 0xFFA943;
/// While this isn't 0, the objects aren't run.
const STOPPED: u32 = 0xFFA3FC;
/// The cell looked for (`func_00192A`): a position, its low nibbles
/// cleared (a long).
const LOOKED_FOR: u32 = 0xFF9ED2;
/// Clears the low nibble of both of a position's words.
const CELL_MASK: u32 = 0xFFF0_FFF0;
/// A position (a long, x then y).
const POSITION: u32 = 0xFF9F04;

/// Pixels moved by direction, x (D_0017A6) and y (D_0017AF), signed
/// bytes (`a_cells.rs`).
const MOVE_X: u32 = 0x0017A6;
const MOVE_Y: u32 = 0x0017AF;

/// The object task's routine (`func_001C96`) and priority.
const OBJECT_TASK: u32 = 0x001C96;
const OBJECT_TASK_PRIORITY: u8 = 0x40;

/// The backdrop's colour while the objects run (`func_000296`), and
/// the default after.
const BACKDROP_BUSY: u16 = 0xEE;
const BACKDROP_DEFAULT: u16 = 0xFFFF;

impl Game {
    /// `func_0018C4`: the object at the cell next to the task at a6 in
    /// direction `d0` (`MOVE_X`, `MOVE_Y`), `func_00192A`.  d1 and a0 are
    /// kept.  Gives back d0, d1, a0, X and C (set when one is there).
    pub fn r_0018c4(&mut self, d0: u32, d1: u32, a0: u32, a6: u32) -> (u32, u32, u32, bool, bool) {
        let direction = d0 as u8 as i8 as i32 as u32;
        let position = self.read32(a6 + TASK_POSITION);
        let step_x = self.read8(MOVE_X.wrapping_add(direction)) as i8 as i16 as u16;
        let x = ((position >> 16) as u16).wrapping_add(step_x) & 0xFFF0;
        let step_y = self.read8(MOVE_Y.wrapping_add(direction)) as i8 as i16 as u16;
        let (y, carry) = (position as u16).overflowing_add(step_y);
        self.write32(LOOKED_FOR, u32::from(x) << 16 | u32::from(y & 0xFFF0));
        let (d0, x, c) = self.r_00192a(d0, carry);
        (d0, d1, a0, x, c)
    }

    /// `func_0018F8`: the object at the cell of `POSITION`
    /// (`func_00192A`).  Gives back d0 and C.
    pub fn r_0018f8(&mut self, x: bool) -> (u32, bool) {
        let cell = self.read32(POSITION) & CELL_MASK;
        let (d0, _, c) = self.look_for(cell, x);
        (d0, c)
    }

    /// `func_00190A`: the object at the cell of the object at a4.  Gives
    /// back d0, X and C.
    pub fn r_00190a(&mut self, a4: u32, x: bool) -> (u32, bool, bool) {
        let cell = self.read32(a4 + OBJ_POSITION) & CELL_MASK;
        self.look_for(cell, x)
    }

    /// `func_00191C`: the object at the cell of the task at a6.  Gives
    /// back d0 and C.
    pub fn r_00191c(&mut self, a6: u32, x: bool) -> (u32, bool) {
        let cell = self.read32(a6 + TASK_POSITION) & CELL_MASK;
        let (d0, _, c) = self.look_for(cell, x);
        (d0, c)
    }

    /// `LOOKED_FOR` = `cell`, then `func_00192A`.
    fn look_for(&mut self, cell: u32, x: bool) -> (u32, bool, bool) {
        self.write32(LOOKED_FOR, cell);
        self.r_00192a(cell, x)
    }

    /// `func_00192A`: the first object in use (and its `OBJ_FLAGS` not
    /// negative) whose position is in the cell `LOOKED_FOR`.  Gives back
    /// d0 (its index; when there is none, the last position compared, or
    /// `d0` if none was), X (clear once past the first object) and C
    /// (set when one is found).
    pub fn r_00192a(&mut self, d0: u32, x: bool) -> (u32, bool, bool) {
        let cell = self.read32(LOOKED_FOR);
        let mut d0 = d0;
        let mut x = x;
        for index in 0..COUNT {
            let object = OBJECTS + OBJECT_SIZE * u32::from(index);
            if self.read8(object + OBJ_USED) != 0 && self.read8(object + OBJ_FLAGS) & 0x80 == 0 {
                d0 = self.read32(object + OBJ_POSITION) & CELL_MASK;
                if d0 == cell {
                    return (u32::from(index), x, true);
                }
            }
            // addq.b #1 on the index, which never carries
            x = false;
        }
        (d0, x, false)
    }

    /// `func_001BB4`: whether the objects are stopped (`STOPPED`); when
    /// they are, it returns from its caller too.  Gives back C (clear)
    /// and that.
    pub fn r_001bb4(&mut self) -> (bool, bool) {
        (false, self.read8(STOPPED) != 0)
    }

    /// `func_001C56`: the objects cleared (their first `CLEARED` bytes),
    /// none running, and the task that runs them (`func_001C96`) made.
    /// Gives back d6, a6 (the task's index and record) and X.
    pub fn r_001c56(&mut self, d4: u32, d5: u32, d6: u32) -> (u32, u32, bool) {
        self.write16(RUNNING, 0);
        for index in 0..u32::from(COUNT) {
            let object = OBJECTS + OBJECT_SIZE * index;
            for offset in 0..CLEARED {
                self.write8(object + offset, 0);
            }
        }
        let (d6, a6, x, _) = self.r_0003e8(d4, d5, d6, OBJECT_TASK_PRIORITY, OBJECT_TASK);
        (d6, a6, x)
    }

    /// `func_001C96`, a task: the objects run (`func_001CA0`) unless
    /// they are stopped.
    pub fn r_001c96(&mut self, regs: &mut Regs) {
        if self.read8(STOPPED) == 0 {
            let r = &mut *regs;
            (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) =
                self.r_001ca0(r.d0, r.d1, r.d2, r.d3, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5, r.a6);
        } else {
            regs.c = false;
        }
        regs.unwound = false;
    }

    /// `func_001CA0`: each object in use has its routine run, with d4
    /// its index, a4 its address, d5 and a5 its sprite record's index
    /// and address (kept in `SLOT_INDEX`, `SLOT_ADDR`; the object's in
    /// `RUNNING`, `RUNNING_ADDR`); the other registers go from one to
    /// the next.  The backdrop is `BACKDROP_BUSY` meanwhile.  Gives back
    /// every register, X and C.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn r_001ca0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a5: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d5, d6, d7, a0, a1, a2, a3, a5, a6, ..Regs::default() };
        self.r_000296(BACKDROP_BUSY);
        self.write8(CLEARED_EACH_FRAME, 0);
        let mut d4 = 0u32;
        loop {
            let (_, object, x) = self.r_001cfa(d4);
            r.a4 = object;
            r.x = x;
            if self.read8(object + OBJ_USED) != 0 {
                let sprite = self.read8(object + OBJ_SPRITE);
                r.d5 = (r.d5 & 0xFFFF_FF00) | u32::from(sprite);
                r.a5 = self.read32(object + OBJ_SPRITE_ADDR);
                self.write8(SLOT_INDEX, sprite);
                self.write32(SLOT_ADDR, r.a5);
                self.write8(RUNNING, d4 as u8);
                self.write32(RUNNING_ADDR, object);
                r.d4 = d4;
                r.a0 = self.read32(object + OBJ_ROUTINE);
                // the flags of move.l a4,(RUNNING_ADDR).w
                (r.n, r.z, r.v, r.c) = (object & 0x8000_0000 != 0, object == 0, false, false);
                r.unwound = false;
                let routine = r.a0;
                self.call_family_4(routine, &mut r);
                if r.unwound || self.restarted.is_some() {
                    return (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c);
                }
            }
            // addq.b #1 on the index, which never carries
            d4 += 1;
            r.x = false;
            if d4 == u32::from(COUNT) {
                break;
            }
        }
        let c = self.r_000296(BACKDROP_DEFAULT);
        (r.d0, r.d1, r.d2, r.d3, d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, c)
    }
}
