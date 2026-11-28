//! The commands of the script the ending's scenes run by (`func_015FAE`
//! reads a command byte from the script, D_016FFA on, and calls the
//! routine its table has for it, commands 1-0x10: `func_016048`-
//! `func_016130`).  Each reads its operand with `func_015F54` (a byte),
//! `func_015F62` (a word) or `func_015F80` (a long); before a word or a
//! long a byte is passed over.  Most set a value the scripted task (`func_015EB4`,
//! `func_015ED0`) or the scene's other task (`func_0161C0`) moves by
//! each frame.  Also the reads of a made task's place and speed
//! (`func_016EFE`, `func_016F08`).

use crate::{Game, Regs};

/// What `func_015ED0` moves the scripted task by: its speed (a word,
/// to `SPEED`; set to 0 when it goes below 0) and the byte added to it
/// each frame; its angle (a word: the high byte to `func_00410C`), the
/// word added to that each frame and the byte added to that word; its
/// frame (a byte: the high nibble + 0x33 is the sprite's frame), the
/// byte added to that each frame and the byte added to that one.
const SPEED_A: u32 = 0xFFAD6E;
const SPEED_A_STEP: u32 = 0xFFAD74;
const ANGLE_A: u32 = 0xFFAD70;
const ANGLE_A_LOW: u32 = 0xFFAD71;
const TURN_A: u32 = 0xFFAD72;
const TURN_A_STEP: u32 = 0xFFAD75;
const FRAME_A: u32 = 0xFFAD76;
const FRAME_A_STEP: u32 = 0xFFAD77;
const FRAME_A_STEP_STEP: u32 = 0xFFAD78;
/// What `func_0161C0` moves by: a speed (a word, to `SPEED`) and the
/// byte added to it each frame, an angle (a word: the high byte to
/// `func_00410C`), the word added to it each frame and the byte added
/// to that word.
const SPEED_B: u32 = 0xFFAD66;
const SPEED_B_STEP: u32 = 0xFFAD6C;
const ANGLE_B: u32 = 0xFFAD68;
const ANGLE_B_LOW: u32 = 0xFFAD69;
const TURN_B: u32 = 0xFFAD6A;
const TURN_B_STEP: u32 = 0xFFAD6D;
/// How many more calls of `func_015FAE` return before it reads the
/// script again.
const WAIT: u32 = 0xFFAD7F;

/// The task running (its index).
const CURRENT_TASK: u32 = 0xFF80EE;
/// A task's routine (a long).
const TASK_ROUTINE: u32 = 0x04;

/// The place a task is made at (words, x then y; `func_00152A`), the
/// speed `func_00410C`'s velocity is made for, and a byte
/// `func_016E7E` picks the made task's animation by (0-3).
const POSITION_X: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
const SPEED: u32 = 0xFF9F00;
const CHOICE: u32 = 0xFF9EF6;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

impl Game {
    /// A byte passed over and a word read (`func_015F54`,
    /// `func_015F62`); gives it back.
    fn u_word(&mut self, r: &mut Regs) -> u16 {
        (r.d0, r.a0, _, _) = self.r_015f54(r.d0);
        (r.d0, r.a0) = self.r_015f62(r.d0, r.d1);
        r.d0 as u16
    }

    /// The next byte read (`func_015F54`); gives it back.
    fn u_byte(&mut self, r: &mut Regs) -> u8 {
        (r.d0, r.a0, _, _) = self.r_015f54(r.d0);
        r.d0 as u8
    }

    /// `func_016048`, command 1: the scripted task's speed.
    pub fn r_016048(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let w = self.u_word(regs);
        self.write16(SPEED_A, w);
    }

    /// `func_01605A`, command 2: what its speed changes by.
    pub fn r_01605a(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let b = self.u_byte(regs);
        self.write8(SPEED_A_STEP, b);
    }

    /// `func_016066`, command 3: its angle (the high byte; the low one
    /// cleared).
    pub fn r_016066(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let b = self.u_byte(regs);
        self.write8(ANGLE_A, b);
        self.write8(ANGLE_A_LOW, 0);
    }

    /// `func_016076`, command 4: what its angle changes by.
    pub fn r_016076(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let w = self.u_word(regs);
        self.write16(TURN_A, w);
    }

    /// `func_016088`, command 5: what that changes by.
    pub fn r_016088(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let b = self.u_byte(regs);
        self.write8(TURN_A_STEP, b);
    }

    /// `func_016094`, command 6: its frame.
    pub fn r_016094(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let b = self.u_byte(regs);
        self.write8(FRAME_A, b);
    }

    /// `func_0160A0`, command 7: what its frame changes by.
    pub fn r_0160a0(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let b = self.u_byte(regs);
        self.write8(FRAME_A_STEP, b);
    }

    /// `func_0160AC`, command 8: what that changes by.
    /// No script calls it: reviewed by reading.
    pub fn r_0160ac(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let b = self.u_byte(regs);
        self.write8(FRAME_A_STEP_STEP, b);
    }

    /// `func_0160B8`, command 9: `func_0161C0`'s speed.
    pub fn r_0160b8(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let w = self.u_word(regs);
        self.write16(SPEED_B, w);
    }

    /// `func_0160CA`, command 0x0A: what that speed changes by.
    pub fn r_0160ca(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let b = self.u_byte(regs);
        self.write8(SPEED_B_STEP, b);
    }

    /// `func_0160D6`, command 0x0B: `func_0161C0`'s angle (the high
    /// byte; the low one cleared).
    pub fn r_0160d6(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let b = self.u_byte(regs);
        self.write8(ANGLE_B, b);
        self.write8(ANGLE_B_LOW, 0);
    }

    /// `func_0160E6`, command 0x0C: what that angle changes by.
    pub fn r_0160e6(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let w = self.u_word(regs);
        self.write16(TURN_B, w);
    }

    /// `func_0160F8`, command 0x0D: what that changes by.
    pub fn r_0160f8(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let b = self.u_byte(regs);
        self.write8(TURN_B_STEP, b);
    }

    /// `func_016104`, command 0x0E: the running task's routine becomes
    /// the long read.  Gives back a6 (the task) and d6.
    pub fn r_016104(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        r.unwound = false;
        (r.d0, r.a0, _, _) = self.r_015f54(r.d0);
        let current = self.read8(CURRENT_TASK);
        (r.d6, r.a6, r.x) = self.r_0005f8(low_byte(r.d6, current));
        (r.d0, r.a0) = self.r_015f80(r.d0, r.d1);
        self.write32(r.a6 + TASK_ROUTINE, r.d0);
    }

    /// `func_01611E`, command 0x0F: the routine at the long read is
    /// called (the makers of the ending's tasks: `func_016240` and the
    /// others of its family), with a0 the routine.
    pub fn r_01611e(&mut self, regs: &mut Regs) {
        let r = &mut *regs;
        (r.d0, r.a0, _, _) = self.r_015f54(r.d0);
        (r.d0, r.a0) = self.r_015f80(r.d0, r.d1);
        r.a0 = r.d0;
        self.call_family_10(r.a0, r);
        r.unwound = false;
    }

    /// `func_016130`, command 0x10: how many calls of `func_015FAE` to
    /// wait.
    pub fn r_016130(&mut self, regs: &mut Regs) {
        regs.unwound = false;
        let b = self.u_byte(regs);
        self.write8(WAIT, b);
    }

    /// `func_016EFE`: the place's x read (a word: `POSITION_X`), then
    /// `func_016F08`.  Gives back d0.
    /// No script calls it (`func_016F74` does): reviewed by reading.
    pub fn r_016efe(&mut self, d0: u32, d1: u32) -> u32 {
        let (d0, _) = self.r_015f62(d0, d1);
        self.write16(POSITION_X, d0 as u16);
        self.r_016f08(d0, d1)
    }

    /// `func_016F08`: the place's y and the speed read (words:
    /// `POSITION_Y`, `SPEED`), then `CHOICE` (a byte) and a byte passed
    /// over.  Gives back d0.
    pub fn r_016f08(&mut self, d0: u32, d1: u32) -> u32 {
        let (d0, _) = self.r_015f62(d0, d1);
        self.write16(POSITION_Y, d0 as u16);
        let (d0, _) = self.r_015f62(d0, d1);
        self.write16(SPEED, d0 as u16);
        let (d0, _, _, _) = self.r_015f54(d0);
        self.write8(CHOICE, d0 as u8);
        let (d0, _, _, _) = self.r_015f54(d0);
        d0
    }
}
