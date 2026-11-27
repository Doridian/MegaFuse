//! Random picks from `func_001BE2`'s numbers, and a task that sets two
//! bytes at random each frame for a while.

use crate::{Game, Regs};

/// What `func_004050` and `func_004070` make of a random number's low
/// three bits (D_004068, D_004080): a number from 0 to 3, the middle
/// ones likelier or the ends.
const PICKS_4068: u32 = 0x004068;
const PICKS_4080: u32 = 0x004080;
/// One of `func_001BE2`'s bytes of state; `loc_004088` adds 1 to it.
const RANDOM_STATE: u32 = 0xFFA400;

/// The task `func_0043A8` starts and the frames it is to run (task
/// byte 0x16).
const TASK_PRIORITY: u8 = 0x31;
const TASK_ROUTINE: u32 = 0x0043DA;
const TASK_FRAMES: u32 = 0x16;
/// The sound `func_0043A8` starts with (`func_0034B6`).
const SOUND: u32 = 0x64;
/// Bytes the task clears but for bit 5, each frame (the 3rd and 4th of
/// the 8-byte record at `RECORDS`).
const RECORDS: u32 = 0xFFFF_AD0A;
/// The two bytes the task sets: 6-9 and 0-3 at random, 8 and 0 in the
/// end.
const BYTE_9EBD: u32 = 0xFF9EBD;
const BYTE_9EBF: u32 = 0xFF9EBF;

impl Game {
    /// A random number's low three bits through the table `picks`, to
    /// d0's low byte (d0's low word: the bits, sign-extended).
    fn d_pick(&mut self, regs: &mut Regs, picks: u32) {
        let (d0, d1, x) = self.r_001be2(regs.d1);
        let bits = d0 & 7;
        let pick = self.read8(picks + bits);
        regs.d0 = (d0 & 0xFFFF_0000) | u32::from(pick);
        regs.d1 = d1;
        regs.x = x;
        (regs.n, regs.z, regs.v, regs.c) = (pick & 0x80 != 0, pick == 0, false, false);
    }

    /// `func_004050` (one of a family): a random number from 0 to 3 to
    /// d0's low byte by `PICKS_4068` (1 and 3 likelier); d1 is kept.
    pub fn r_004050(&mut self, regs: &mut Regs) {
        let d1 = regs.d1;
        self.d_pick(regs, PICKS_4068);
        regs.d1 = d1;
    }

    /// `func_004070` (one of a family): a random number from 0 to 3 to
    /// d0's low byte by `PICKS_4080` (0 and 2 likelier).
    /// No script calls it: reviewed by reading.
    pub fn r_004070(&mut self, regs: &mut Regs) {
        self.d_pick(regs, PICKS_4080);
    }

    /// `loc_004088` (one of a family): a random number's low two bits to
    /// d0's low byte, and `RANDOM_STATE` 1 on.
    pub fn r_004088(&mut self, regs: &mut Regs) {
        let (d0, d1, _) = self.r_001be2(regs.d1);
        let (state, x) = self.read8(RANDOM_STATE).overflowing_add(1);
        self.write8(RANDOM_STATE, state);
        let bits = d0 as u8 & 3;
        regs.d0 = (d0 & 0xFFFF_FF00) | u32::from(bits);
        regs.d1 = d1;
        regs.x = x;
        (regs.n, regs.z, regs.v, regs.c) = (false, bits == 0, false, false);
    }

    /// `func_0043A8`: the task `TASK_ROUTINE` (`func_0003E8`) to run
    /// `d0` frames (its low byte), and the sound `SOUND`, unless no task
    /// could be had.  Gives back d0 (its low byte `SOUND` then) and X.
    pub fn r_0043a8(&mut self, d0: u32, d4: u32, d5: u32, d6: u32) -> (u32, bool) {
        let (_, task, x, failed) = self.r_0003e8(d4, d5, d6, TASK_PRIORITY, TASK_ROUTINE);
        if failed {
            return (d0, x);
        }
        self.write8(task + TASK_FRAMES, d0 as u8);
        let d0 = (d0 & 0xFFFF_FF00) | SOUND;
        self.r_0034b6(d0);
        (d0, x)
    }

    /// `func_0043DA`, `func_0043A8`'s task (one of a family): each frame
    /// the two bytes after `RECORDS`' first two cleared but for bit 5,
    /// and, while its frames last (none set: for ever), `BYTE_9EBD` 6-9
    /// and `BYTE_9EBF` 0-3 at random; on the last one 8 and 0 and the
    /// task ends (`func_000560`).
    pub fn r_0043da(&mut self, regs: &mut Regs) {
        regs.a0 = RECORDS;
        for at in [2, 3] {
            let b = self.read8(RECORDS + at);
            self.write8(RECORDS + at, b & 0x20);
        }
        let task = regs.a6;
        let frames = self.read8(task + TASK_FRAMES);
        regs.d0 = (regs.d0 & 0xFFFF_FF00) | u32::from(frames);
        if frames != 0 {
            let left = frames - 1;
            self.write8(task + TASK_FRAMES, left);
            regs.x = false;
            if left == 0 {
                self.write8(BYTE_9EBD, 8);
                self.write8(BYTE_9EBF, 0);
                (regs.d6, regs.x, regs.c) = self.r_000560(regs.d5, regs.d6, regs.x);
                return;
            }
        }
        let (d0, d1, _) = self.r_001be2(regs.d1);
        let (b, _) = (d0 as u8 & 3).overflowing_add(6);
        self.write8(BYTE_9EBD, b);
        let (d0, d1, x) = self.r_001be2(d1);
        let b = d0 as u8 & 3;
        self.write8(BYTE_9EBF, b);
        regs.d0 = (d0 & 0xFFFF_FF00) | u32::from(b);
        regs.d1 = d1;
        regs.x = x;
        regs.c = false;
    }
}
