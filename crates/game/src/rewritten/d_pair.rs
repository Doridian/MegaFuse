//! An object made to go with another (`func_004480`): the made one
//! (`MADE`, from index 5 on) carries the other (`OTHER`, by the table
//! `OTHERS`): each frame the other is put where the made one is
//! (`func_0047C6`), and their sprite records are put in depth one
//! before the other by the made one's direction (`func_004828`).  The
//! made one runs one of a few routines (its byte 2, a long): standing
//! (`loc_0045E2`), each kind's own move when `BITS_A933`'s bit 4 says
//! so (the table D_0048F6), and the moves those start.
//!
//! Most of these keep the registers the objects' routines share
//! (`Regs`): a4 and d4 the object and its index, a5 and d5 a sprite
//! record and its index, as the original leaves them.

use crate::{Game, Regs};

/// An object's fields (`anim.rs` for more).
/// The routine it runs (a long), and one `func_004E5C` puts back.
const OBJ_ROUTINE: u32 = 0x02;
const OBJ_SAVED_ROUTINE: u32 = 0x06;
/// Its sprite record (a long) and the record's index.
const OBJ_SPRITE_ADDR: u32 = 0x0A;
const OBJ_SPRITE: u32 = 0x0E;
/// The made one's index - 5 (also its sprite record's palette byte).
const OBJ_NUMBER: u32 = 0x0F;
/// Flags: bit 7 set while it moves on its own (`func_004568`,
/// `func_004A0E`), cleared when it lands (`func_004E5C`).
const OBJ_FLAGS: u32 = 0x10;
const OBJ_FLAG_MOVING: u8 = 0x80;
/// The other's bits 5-6, set for a while (`func_004DFE`).
const OBJ_FLAGS_60: u8 = 0x60;
/// The direction it is to take (`func_0047C6` sets it with the
/// direction).
const OBJ_NEXT_DIRECTION: u32 = 0x13;
/// Its direction (bits 0-1) and a flag (bit 7: busy with a move; 0xFF
/// as a whole: `func_0046FC`'s mark).
const OBJ_DIRECTION: u32 = 0x14;
const DIRECTION_BITS: u8 = 0x03;
const DIRECTION_BUSY: u8 = 0x80;
/// What its animation is by (added to the direction, `func_00130E`).
const OBJ_KIND: u32 = 0x15;
const OBJ_16: u32 = 0x16;
const OBJ_17: u32 = 0x17;
/// Its position (words), and their low bytes.
const OBJ_X: u32 = 0x18;
const OBJ_X_LOW: u32 = 0x19;
const OBJ_Y: u32 = 0x1A;
const OBJ_Y_LOW: u32 = 0x1B;
/// The animation running (a long).
const OBJ_ANIM: u32 = 0x20;
/// The animation's frames to go and position; `func_00490C`'s moves
/// count frames in `OBJ_TIMER` too.
const OBJ_TIMER: u32 = 0x24;
const OBJ_STEP: u32 = 0x25;
/// A word `func_004EA6` moves by (its low byte from the table
/// `SPEEDS`).
const OBJ_SPEED: u32 = 0x26;
const OBJ_SPEED_LOW: u32 = 0x27;
/// A word added to while it moves on its own (`func_004424`; set to
/// 0xFC00, 0xFD00 to start); its high byte's sign is looked at.
const OBJ_RISE: u32 = 0x28;
/// A height: what the sprite records' depth is by, copied to the
/// other.
const OBJ_HEIGHT: u32 = 0x2A;
/// The other's: the made one's index; a byte `func_004D7E` clears.
const OBJ_MADE: u32 = 0x32;
const OBJ_3A: u32 = 0x3A;

/// A sprite record's fields (`sprite.rs`): its palette, its frame, its
/// flags, a word of tiles, the low byte of y, and a long `func_004480`
/// sets.
const SPRITE_PALETTE: u32 = 0x03;
const SPRITE_FRAME: u32 = 0x04;
const SPRITE_FLAGS: u32 = 0x05;
const SPRITE_TILES: u32 = 0x06;
const SPRITE_Y_LOW: u32 = 0x0B;
const SPRITE_0C: u32 = 0x0C;
/// The flag that keeps a sprite record's frame (`sprite.rs`).
const BUDGETED: u8 = 1 << 4;
const ALTERNATE: u8 = 1 << 6;

/// A task's fields (`task.rs`): frames to go, a byte, and the sprite
/// record a shadow copies the frame of.
const TASK_FRAMES: u32 = 0x12;
const TASK_13: u32 = 0x13;
const TASK_COPIED: u32 = 0x19;

/// The made object: its index (5 on) and address.
const MADE: u32 = 0xFFA42C;
const MADE_ADDR: u32 = 0xFFA42E;
/// The other object's index.
const OTHER: u32 = 0xFFA93F;
/// By the made one's index - 5: the other object, the kind it was made
/// as (`d0` of `func_004480`), a count from 3 (by its index - 0, so 6
/// on).
const OTHERS: u32 = 0xFFFF_A968;
const KINDS: u32 = 0xFFFF_A96E;
const COUNTS: u32 = 0xFFFF_A96F;
/// The sprite record the task runs with (`task.rs`'s `SLOT_INDEX`,
/// `SLOT_ADDR`); here the made one's.
const SPRITE: u32 = 0xFF8A57;
const SPRITE_ADDR: u32 = 0xFF8A5A;
/// Records of 8 bytes by the other's index: bytes 2 and 3 are looked
/// at (low four bits) and cleared.
const RECORDS: u32 = 0xFFFF_AD0A;
/// A byte set to 1, then 8 or 3 if the other's record's byte 3 has a
/// low bit.
const BYTE_A942: u32 = 0xFFA942;
/// Bit 4: the made one's move starts (`BITS_A933`), goes on
/// (`BITS_A932`, cleared by `func_004EA6`).
const BITS_A933: u32 = 0xFFA933;
const BITS_A932: u32 = 0xFFA932;
const MOVE_BIT: u8 = 1 << 4;
/// Bits: 2 with 0 or 4 (0x11) makes `loc_0045E2` show both still;
/// 6 stops `func_004BF6`.
const BITS_9540: u32 = 0xFF9540;
/// When this is 0x37 kind 3 is made as 4 (`func_004568`).
const BYTE_953F: u32 = 0xFF953F;
/// Looked at before the sound `loc_0045E2` makes.
const BYTE_A93C: u32 = 0xFFA93C;
const BYTE_A936: u32 = 0xFFA936;
/// A pointer to a byte `func_004BF6` compares with 3.
const POINTER_9ED6: u32 = 0xFF9ED6;
/// What an object is set up with (`anim.rs`): a long, its animations,
/// a position (words).
const OBJ_INIT_2: u32 = 0xFF80D0;
const OBJ_INIT_ANIMS: u32 = 0xFF80C8;
const POSITION: u32 = 0xFF9F04;
const POSITION_Y: u32 = 0xFF9F06;
/// What `func_004EA6` sets up for the move (`func_002846` and the
/// like): the position's low bytes, the speed negated and not, and a
/// step by the direction (0 and 2 set `MOVE_Y`, 1 and 3 `MOVE_X`; the
/// long `MOVE_X` covers both).
const MOVE_X_LOW: u32 = 0xFF9F02;
const MOVE_Y_LOW: u32 = 0xFF9F03;
const SPEED_NEG: u32 = 0xFF9EEA;
const SPEED_POS: u32 = 0xFF9EEC;
const MOVE_X: u32 = 0xFF9EEE;
const MOVE_Y: u32 = 0xFF9EF0;
/// The new task's index (`task.rs`).
const NEW_TASK: u32 = 0xFF80EF;

/// Tables in the ROM.
/// The made one's routines by kind (D_0048F6).
const KIND_ROUTINES: u32 = 0x0048F6;
/// Two colours (by their form) by kind (D_004554).
const KIND_COLOURS: u32 = 0x004554;
/// `func_004A0E`'s routines by direction (D_004A7E).
const JUMP_ROUTINES: u32 = 0x004A7E;
/// Speeds by the position's low four bits (D_004AEA).
const SPEEDS: u32 = 0x004AEA;
/// `func_004ED8`'s routines by direction (D_004EF0).
const MOVE_ROUTINES: u32 = 0x004EF0;
/// A direction by sprite frame: the code reads `func_004B26` + 0x8F
/// + the frame, the bytes after `func_004B26` (0x4BCA) being frames
/// 0x15 on.
const DIRECTION_BY_FRAME: u32 = 0x004BB5;
/// Bits by direction or'd into the other's record (D_009C68).
const BITS_BY_DIRECTION: u32 = 0x009C68;
/// The sprite record's long 0x0C (D_0093B4) and the animations
/// (D_00B558) the made one starts with.
const MADE_SPRITE_0C: u32 = 0x0093B4;
const MADE_ANIMS: u32 = 0x00B558;
/// An animation (D_0EF8EE) `func_00495A` starts.
const ANIM_EF8EE: u32 = 0x0EF8EE;

/// Routines.
const STANDING: u32 = 0x0045E2;
const FALLING: u32 = 0x004D7E;
const LANDED: u32 = 0x004F6C;
const TIMED: u32 = 0x004AFA;
const CARRYING: u32 = 0x004B26;
const PUSHING: u32 = 0x004BF6;
const SHADOW: u32 = 0x004D24;
const SPRITE_TASK_A: u32 = 0x005038;
const SPRITE_TASK_B: u32 = 0x00505C;

/// Sounds (`func_0034B6`).
const SOUND_MADE: u32 = 0x3C;
const SOUND_JUMP: u32 = 0x3E;
const SOUND_PUSH: u32 = 0x3F;

fn low_byte(reg: u32, b: u8) -> u32 {
    (reg & 0xFFFF_FF00) | u32::from(b)
}

fn low_word(reg: u32, w: u16) -> u32 {
    (reg & 0xFFFF_0000) | u32::from(w)
}

/// An entry of a table by the made one's index - 5 (`d4`'s low word).
fn by_made(table: u32, d4: u32) -> u32 {
    table.wrapping_sub(5).wrapping_add(d4 as u16 as i16 as u32)
}

impl Game {
    /// d4 and a4 back to the made object (`MADE`); C clear (the move).
    fn d_made(&mut self, r: &mut Regs) {
        r.d4 = low_byte(r.d4, self.read8(MADE));
        r.a4 = self.read32(MADE_ADDR);
        r.c = false;
    }

    /// d4 and a4 to the other object (`OTHER`, `func_001CFA`).
    fn d_other(&mut self, r: &mut Regs) {
        r.d4 = low_byte(r.d4, self.read8(OTHER));
        (r.d4, r.a4, r.x) = self.r_001cfa(r.d4);
        r.c = false;
    }

    /// d5 and a5 to the task's sprite record (`SPRITE`); C clear.
    fn d_task_sprite(&mut self, r: &mut Regs) {
        r.d5 = low_byte(r.d5, self.read8(SPRITE));
        r.a5 = self.read32(SPRITE_ADDR);
        r.c = false;
    }

    /// d5 and a5 to the object's sprite record; C clear.
    fn d_obj_sprite(&mut self, r: &mut Regs) {
        r.d5 = low_byte(r.d5, self.read8(r.a4 + OBJ_SPRITE));
        r.a5 = self.read32(r.a4 + OBJ_SPRITE_ADDR);
        r.c = false;
    }

    /// `OTHER` from `OTHERS` by the made one's index (d4's low word
    /// the index sign-extended, a0 `OTHERS`).  Gives back the other's
    /// index.
    fn d_note_other(&mut self, r: &mut Regs) -> u8 {
        r.a0 = OTHERS;
        r.d4 = low_word(r.d4, r.d4 as u8 as i8 as u16);
        let other = self.read8(by_made(OTHERS, r.d4));
        self.write8(OTHER, other);
        r.c = false;
        other
    }

    /// The made one's (d4, a4) record of `RECORDS`' byte at `at`, by
    /// `index` (x 8, a word).
    fn d_record(index: u8, at: u32) -> u32 {
        let offset = (index as i8 as u16).wrapping_shl(3);
        RECORDS.wrapping_add(at).wrapping_add(offset as i16 as u32)
    }

    fn d_anim(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d2, r.d5, r.a0, r.a5, r.x, r.z, r.c) = self.r_001336(r.d0, r.d1, r.d2, r.d5, r.a4, r.x);
    }

    fn d_set_anim(&mut self, r: &mut Regs) {
        (r.a0, r.x, r.c) = self.r_00130e(r.d0, r.a4);
    }

    fn d_1b32(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.x, r.c) = self.r_001b32(r.d1, r.a4, r.a5);
    }

    fn d_2f70(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.a0, r.a1, r.x) = self.r_002f70(r.d0, r.d1, r.d4, r.a0, r.a1);
    }

    fn d_15e6(&mut self, r: &mut Regs) {
        (r.d0, r.x) = self.r_0015e6(r.a4);
    }

    fn d_167a(&mut self, r: &mut Regs) {
        (r.a0, r.x) = self.r_00167a();
    }

    fn d_3032(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c) = self.r_003032(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.x);
    }

    fn d_follow(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c) = self.r_0047c6(r.d0, r.d1, r.d4, r.d5, r.a4);
    }

    fn d_depths(&mut self, r: &mut Regs) {
        (r.d0, r.d1, r.d4, r.d5, r.a4, r.a5, r.x, r.c) = self.r_004828(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3);
    }

    /// `func_001FD4` (or `func_001FD6`, `at_d6`), all the registers.
    /// Gives back whether the game restarted in it.
    fn d_1fd4(&mut self, r: &mut Regs, at_d6: bool) -> bool {
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6, r.x, r.c) = if at_d6 {
            self.r_001fd6(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6)
        } else {
            self.r_001fd4(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6)
        };
        self.restarted.is_some()
    }

    fn d_sound(&mut self, r: &mut Regs, sound: u32) {
        r.d0 = (r.d0 & 0xFFFF_FF00) | sound;
        self.r_0034b6(r.d0);
    }

    /// `func_004480`: an object made to go with the other (`OTHER`):
    /// the first free from index 5 (its byte 0 is 0), set up
    /// (`func_0012B6`), the made one (`MADE`, its sprite record
    /// `SPRITE`'s, flags 0xA2) of kind `d0` (`KINDS`; its colours,
    /// `func_004526`), direction `d1`, `OBJ_KIND` `d2`; the two know
    /// each other (`OTHERS`, `OBJ_MADE`), the sprite record's tiles by
    /// the index, its animation, `func_001B32`, the depths
    /// (`func_004828`).
    #[allow(clippy::too_many_arguments)]
    pub fn r_004480(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut d4 = 5;
        let obj = loop {
            let (index, obj, _) = self.r_001cfa(d4);
            if self.read8(obj) == 0 {
                break obj;
            }
            d4 = low_byte(index, (index as u8).wrapping_add(1));
        };
        self.r_0012b6(obj);
        let d5 = low_byte(d5, self.read8(obj + OBJ_SPRITE));
        let sprite = self.read32(obj + OBJ_SPRITE_ADDR);
        self.write8(SPRITE, d5 as u8);
        self.write32(SPRITE_ADDR, sprite);
        self.write8(MADE, d4 as u8);
        self.write32(MADE_ADDR, obj);
        self.write8(sprite + SPRITE_FLAGS, 0xA2);
        let d4 = low_word(d4, d4 as u8 as i8 as u16);
        self.write8(by_made(KINDS, d4), d0 as u8);
        self.r_004526(d0, d4);
        let number = (d4 as u8).wrapping_sub(5);
        let d0 = low_byte(d0, number);
        self.write8(sprite + SPRITE_PALETTE, number);
        self.write8(obj + OBJ_NUMBER, number);
        self.write8(obj + OBJ_DIRECTION, d1 as u8);
        self.write8(obj + OBJ_KIND, d2 as u8);
        // the other knows the made one; d3 ends with the other's index
        let other_index = self.read8(OTHER);
        let (other_index, other, _) = self.r_001cfa(low_byte(d4, other_index));
        self.write8(other + OBJ_MADE, d4 as u8);
        let d3 = low_word(d3, other_index as u16);
        self.write8(by_made(OTHERS, d4), d3 as u8);
        let tiles = ((d4 as u16) << 4).wrapping_sub(0x10).wrapping_add(0x260);
        let d0 = low_word(d0, tiles);
        self.write16(sprite + SPRITE_TILES, tiles);
        self.write32(sprite + SPRITE_0C, MADE_SPRITE_0C);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3: obj, a4: obj, a5: sprite, ..Regs::default() };
        self.d_set_anim(&mut r);
        self.d_1b32(&mut r);
        self.d_depths(&mut r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.a0, r.a3, r.a4, r.a5, r.x, r.c)
    }

    /// `func_004526`: kind `d0`'s two colours (`KIND_COLOURS`) by
    /// `func_00946E` at 0x20 x (the index `d4` - 5) + 0x10 (that word
    /// shifted with what the kind's offset left in its high byte), then
    /// `func_0094B2`.
    pub fn r_004526(&mut self, d0: u32, d4: u32) {
        let offset = (d0 as u8 as i8 as u16).wrapping_mul(4);
        let colours = KIND_COLOURS.wrapping_add(offset as i16 as u32);
        let w = (offset & 0xFF00) | u16::from((d4 as u8).wrapping_sub(5));
        let d0 = low_word(d0, (w << 5).wrapping_add(0x10));
        self.r_00946e(d0, 2, colours);
        self.r_0094b2();
    }

    /// `func_004568`: the object at a4 makes one to go with it
    /// (`func_004480`) where it is, facing its way (+ 0x80), `OBJ_KIND`
    /// 8, of a random kind 0-3 (`func_001BC0`; 3 is 4 when `BYTE_953F`
    /// is 0x37), standing (`STANDING`) but moving on its own first
    /// (`FALLING`, `OBJ_RISE` 0xFC00), with a sprite task
    /// (`func_004FD0`) and the sound `SOUND_MADE`.
    #[allow(clippy::too_many_arguments)]
    pub fn r_004568(&mut self, d1: u32, d2: u32, d3: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a4: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        let d1 = low_byte(d1, self.read8(a4 + OBJ_DIRECTION) | 0x80);
        let d2 = low_byte(d2, 8);
        let x = self.read16(a4 + OBJ_X);
        self.write16(POSITION, x);
        let y = self.read16(a4 + OBJ_Y);
        self.write16(POSITION_Y, y);
        self.write32(OBJ_INIT_2, STANDING);
        self.write32(OBJ_INIT_ANIMS, MADE_ANIMS);
        let mut d0 = self.r_001bc0(4, d1);
        if self.read8(BYTE_953F) == 0x37 && d0 as u8 == 3 {
            d0 = low_byte(d0, 4);
        }
        let (d0, d1, d2, d3, d4, d5, _, a3, obj, a5, _, _) = self.r_004480(d0, d1, d2, d3, d5, d6, d7, a1, a2);
        let flags = self.read8(obj + OBJ_FLAGS);
        self.write8(obj + OBJ_FLAGS, flags | OBJ_FLAG_MOVING);
        self.write8(obj + OBJ_KIND, 0);
        self.write32(obj + OBJ_ROUTINE, FALLING);
        self.write16(obj + OBJ_RISE, 0xFC00);
        let (d0, _, _) = self.r_004fd0(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, obj, a5);
        let d0 = low_byte(d0, SOUND_MADE as u8);
        self.r_0034b6(d0);
        (d0, d1, d2, d3, d4, d5, a3, obj)
    }

    /// The kind's routine (`KIND_ROUTINES`, family 22), from
    /// `loc_0045E2`.
    fn d_kind_routine(&mut self, r: &mut Regs) {
        r.d4 = low_word(r.d4, r.d4 as u8 as i8 as u16);
        r.a0 = KINDS;
        let kind = self.read8(by_made(KINDS, r.d4));
        let (twice, _) = kind.overflowing_add(kind);
        let (offset, carry) = twice.overflowing_add(twice);
        r.d0 = low_word(r.d0, offset as i8 as u16);
        // the ext.w clears C
        r.x = carry;
        r.c = false;
        r.a0 = self.read32(KIND_ROUTINES.wrapping_add(offset as i8 as u32));
        let mut sub = *r;
        self.call_family_22(r.a0, &mut sub);
        (r.d0, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6, r.x, r.c) = (sub.d0, sub.d4, sub.d5, sub.d6, sub.a0, sub.a4, sub.a5, sub.a6, sub.x, sub.c);
    }

    /// `loc_0045E2` (the made one's routine, standing): while
    /// `BITS_9540` has bit 2 and 0 or 4 both are shown still (the
    /// other's `OBJ_KIND` 0x10, the made one's 0, facing 2, once), their
    /// animations a frame on; else (a sound first, as `BYTE_A93C` and
    /// `BYTE_A936` say, when its animation is at step 1) unless
    /// `func_001FD4` sets C, `func_001B32`, the other follows it, the
    /// depths, and the kind's routine (`KIND_ROUTINES`).
    pub fn r_0045e2(&mut self, regs: &mut Regs) {
        let r = regs;
        self.d_note_other(r);
        let bits = self.read8(BITS_9540);
        if bits & 0x04 != 0 {
            r.d0 = low_byte(r.d0, self.read8(BITS_9540) & 0x11);
            if r.d0 as u8 != 0 {
                self.d_other(r);
                r.d0 = low_byte(r.d0, 0x10);
                if self.read8(r.a4 + OBJ_KIND) != 0x10 {
                    self.write8(r.a4 + OBJ_KIND, 0x10);
                    self.write8(r.a4 + OBJ_DIRECTION, 0);
                    self.d_set_anim(r);
                    self.write8(r.a4 + OBJ_DIRECTION, 2);
                    self.write8(r.a4 + OBJ_TIMER, 0);
                    self.write8(r.a4 + OBJ_STEP, 0);
                    self.d_made(r);
                    self.d_task_sprite(r);
                    self.write8(r.a4 + OBJ_DIRECTION, 2);
                    self.write8(r.a4 + OBJ_KIND, 0);
                    self.write8(r.a4 + OBJ_TIMER, 0);
                    self.write8(r.a4 + OBJ_STEP, 0);
                    self.d_set_anim(r);
                    let number = self.read8(r.a4 + OBJ_NUMBER);
                    self.write8(r.a5 + SPRITE_PALETTE, number);
                }
                self.d_other(r);
                self.d_obj_sprite(r);
                self.write8(r.a5 + SPRITE_PALETTE, r.d4 as u8);
                self.d_anim(r);
                self.d_made(r);
                self.d_task_sprite(r);
                self.d_anim(r);
                self.d_1b32(r);
                self.d_follow(r);
                self.d_depths(r);
                return;
            }
        }
        if self.read8(BYTE_A93C) != 0 && self.read8(r.a4 + OBJ_TIMER) == 1 && self.read8(r.a4 + OBJ_STEP) & 0x02 == 0 {
            let sound = if self.read8(BYTE_A936) != 0 { 0x35 } else { 0x3B };
            self.d_sound(r, sound);
        }
        if self.d_1fd4(r, false) || r.c {
            return;
        }
        self.d_task_sprite(r);
        self.d_1b32(r);
        self.d_follow(r);
        self.d_depths(r);
        self.d_kind_routine(r);
    }

    /// `BYTE_A942` 1, or `high` if the other's record's byte 3 has a low
    /// bit (d0, a0 as the original leaves them).
    fn d_a942(&mut self, r: &mut Regs, high: u8) {
        self.write8(BYTE_A942, 1);
        let other = self.read8(OTHER);
        r.d0 = low_word(r.d0, (other as i8 as u16).wrapping_shl(3));
        r.a0 = RECORDS;
        let bits = self.read8(Self::d_record(other, 3)) & 0x0F;
        r.d0 = low_byte(r.d0, bits);
        if bits != 0 {
            self.write8(BYTE_A942, high);
        }
    }

    /// `func_0046FC` (a routine of the made one): its animation a frame
    /// on, `BYTE_A942` (8), `func_002F70`; if `func_002E24` sets C the
    /// other's direction 0xFF; the depths as facing 2, then
    /// `func_0015E6`, `func_00167A`, `func_003032`.
    /// No script calls it: reviewed by reading.
    pub fn r_0046fc(&mut self, regs: &mut Regs) {
        let r = regs;
        self.d_note_other(r);
        self.d_anim(r);
        self.d_a942(r, 8);
        self.d_made(r);
        self.d_2f70(r);
        if self.r_002e24(r.a4) {
            self.d_other(r);
            self.write8(r.a4 + OBJ_DIRECTION, 0xFF);
            self.d_made(r);
        }
        let direction = self.read8(r.a4 + OBJ_DIRECTION);
        self.write8(r.a4 + OBJ_DIRECTION, 2);
        self.d_depths(r);
        self.write8(r.a4 + OBJ_DIRECTION, direction);
        r.c = false;
        self.d_15e6(r);
        self.d_167a(r);
        self.d_3032(r);
    }

    /// `func_00477A` (a routine of the made one): `BYTE_A942` (3),
    /// `func_002F70`, `func_002E24`, `func_0015E6`, `func_00167A`,
    /// `func_003032`.
    /// No script calls it: reviewed by reading.
    pub fn r_00477a(&mut self, regs: &mut Regs) {
        let r = regs;
        self.d_note_other(r);
        self.d_a942(r, 3);
        self.d_made(r);
        self.d_2f70(r);
        r.c = self.r_002e24(r.a4);
        self.d_15e6(r);
        self.d_167a(r);
        self.d_3032(r);
    }

    /// `func_0047C6`: the other (by `OTHERS`, `func_001D0C`) follows the
    /// made one at a4: its position, its height (the made one's, + 1
    /// at its animation's odd pairs), its direction (if that changed,
    /// `OBJ_NEXT_DIRECTION` too and `func_0020E2`); then with a4 the
    /// other and a5 its sprite record, `func_001B32`.
    pub fn r_0047c6(&mut self, d0: u32, d1: u32, d4: u32, d5: u32, a4: u32) -> (u32, u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d4, d5, a4, ..Regs::default() };
        let other_index = self.d_note_other(&mut r);
        let (d5, other) = self.r_001d0c(low_byte(r.d5, other_index));
        r.d5 = d5;
        r.a5 = other;
        let x = self.read16(r.a4 + OBJ_X);
        self.write16(other + OBJ_X, x);
        let y = self.read16(r.a4 + OBJ_Y);
        self.write16(other + OBJ_Y, y);
        let odd = (self.read8(r.a4 + OBJ_STEP) >> 1) & 1;
        let height = odd.wrapping_add(self.read8(r.a4 + OBJ_HEIGHT));
        r.d0 = low_byte(r.d0, height);
        self.write8(other + OBJ_HEIGHT, height);
        let direction = self.read8(r.a4 + OBJ_DIRECTION) & DIRECTION_BITS;
        r.d0 = low_byte(r.d0, direction);
        if direction != self.read8(other + OBJ_DIRECTION) {
            self.write8(other + OBJ_DIRECTION, direction);
            self.write8(other + OBJ_NEXT_DIRECTION, direction);
            (r.d0, r.d5, r.a0, r.a5, r.x, r.c) = self.r_0020e2(r.d0, r.d4, r.d5);
        }
        self.d_other(&mut r);
        self.d_obj_sprite(&mut r);
        self.d_1b32(&mut r);
        (r.d0, r.d1, r.d4, r.d5, r.a0, r.a4, r.a5, r.x, r.c)
    }

    /// `func_004828`: the two sprite records' depth (`func_0007D2`):
    /// both by the made one's sprite's y - its height - 0x10 (not,
    /// even, | 2), the made one's put first when it faces 2, else the
    /// other's.  d5 and a5 are kept (`SPRITE`'s at the start); d4 and
    /// a4 the made one.  Gives back d0 and d1 (the depth in the low
    /// bytes), X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_004828(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32) -> (u32, u32, u32, u32, u32, u32, bool, bool) {
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, ..Regs::default() };
        self.d_task_sprite(&mut r);
        self.d_made(&mut r);
        let (kept5, kept_a5) = (r.d5, r.a5);
        let above = self.read8(r.a5 + SPRITE_Y_LOW).wrapping_sub(self.read8(r.a4 + OBJ_HEIGHT));
        let (above, borrow) = above.overflowing_sub(0x10);
        r.x = borrow;
        let depth = (!above & 0xFE) | 2;
        r.d1 = low_byte(r.d1, depth);
        let made_first = self.read8(r.a4 + OBJ_DIRECTION) & DIRECTION_BITS == 2;
        let depth_of = |game: &mut Game, r: &mut Regs| {
            r.d0 = low_byte(r.d0, depth);
            r.c = game.r_0007d2(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.x);
        };
        if made_first {
            depth_of(self, &mut r);
            self.d_other(&mut r);
            self.d_obj_sprite(&mut r);
            self.write8(SPRITE, r.d5 as u8);
            self.write32(SPRITE_ADDR, r.a5);
            depth_of(self, &mut r);
            self.d_made(&mut r);
        } else {
            self.d_other(&mut r);
            self.d_obj_sprite(&mut r);
            self.write8(SPRITE, r.d5 as u8);
            self.write32(SPRITE_ADDR, r.a5);
            depth_of(self, &mut r);
            self.d_made(&mut r);
            self.d_obj_sprite(&mut r);
            self.write8(SPRITE, r.d5 as u8);
            self.write32(SPRITE_ADDR, r.a5);
            depth_of(self, &mut r);
        }
        (r.d0, r.d1, r.d4, kept5, r.a4, kept_a5, r.x, r.c)
    }

    /// Whether the made one's move starts (`BITS_A933`'s bit 4), d4 and
    /// a4 back to it first.
    fn d_move_starts(&mut self, r: &mut Regs) -> bool {
        self.d_made(r);
        self.read8(BITS_A933) & MOVE_BIT != 0
    }

    /// A move that shows frame 0x23 + the direction for 0xC frames
    /// (`TIMED`), its sprite record `BUDGETED`; d0 the direction.
    fn d_timed_move(&mut self, r: &mut Regs) {
        self.write8(r.a4 + OBJ_TIMER, 0x0C);
        let direction = self.read8(r.a4 + OBJ_DIRECTION) & DIRECTION_BITS;
        r.d0 = low_byte(r.d0, direction.wrapping_add(0x23));
        self.d_obj_sprite(r);
        self.write8(r.a5 + SPRITE_FRAME, r.d0 as u8);
        let flags = self.read8(r.a5 + SPRITE_FLAGS);
        self.write8(r.a5 + SPRITE_FLAGS, flags | BUDGETED);
        r.d0 = low_byte(r.d0, direction);
    }

    /// `func_00490C` (kind 0's routine): when the move starts, frame
    /// 0x23 + the direction for 0xC frames (`TIMED`) and
    /// `func_005B8C` that way.
    pub fn r_00490c(&mut self, regs: &mut Regs) {
        let r = regs;
        if !self.d_move_starts(r) {
            return;
        }
        self.d_timed_move(r);
        (r.d0, r.d6, r.a0, r.a6, r.x) = self.r_005b8c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        self.write32(r.a4 + OBJ_ROUTINE, TIMED);
        r.c = false;
    }

    /// `func_00495A` (kind 1's routine): when the move starts, busy,
    /// the animation `ANIM_EF8EE` from its start, `CARRYING`.
    pub fn r_00495a(&mut self, regs: &mut Regs) {
        let r = regs;
        if !self.d_move_starts(r) {
            return;
        }
        let direction = self.read8(r.a4 + OBJ_DIRECTION);
        self.write8(r.a4 + OBJ_DIRECTION, direction | DIRECTION_BUSY);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.write32(r.a4 + OBJ_ANIM, ANIM_EF8EE);
        self.write32(r.a4 + OBJ_ROUTINE, CARRYING);
        r.c = false;
    }

    /// `func_00498E` (kind 2's routine): when the move starts, its
    /// count (`COUNTS`) 3, `PUSHING`, the sound `SOUND_PUSH`.
    pub fn r_00498e(&mut self, regs: &mut Regs) {
        let r = regs;
        if !self.d_move_starts(r) {
            return;
        }
        r.a0 = COUNTS;
        r.d4 = low_word(r.d4, r.d4 as u8 as i8 as u16);
        self.write8(COUNTS.wrapping_add(r.d4 as u16 as i16 as u32), 3);
        self.write32(r.a4 + OBJ_ROUTINE, PUSHING);
        r.c = false;
        self.d_sound(r, SOUND_PUSH);
    }

    /// `func_0049C0` (kind 3's routine): as kind 0's, with
    /// `func_00555E`.
    pub fn r_0049c0(&mut self, regs: &mut Regs) {
        let r = regs;
        if !self.d_move_starts(r) {
            return;
        }
        self.d_timed_move(r);
        r.x = self.r_00555e(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        self.write32(r.a4 + OBJ_ROUTINE, TIMED);
        r.c = false;
    }

    /// `func_004A0E` (kind 4's routine): when the move starts, it moves
    /// on its own (`OBJ_FLAGS` 0x80, its animation that of `OBJ_KIND`
    /// 8, `OBJ_RISE` 0xFD00), its speed by the direction
    /// (`JUMP_ROUTINES`) unless busy, `FALLING`, a sprite task
    /// (`func_004FD0`), the sound `SOUND_JUMP`.
    pub fn r_004a0e(&mut self, regs: &mut Regs) {
        let r = regs;
        if !self.d_move_starts(r) {
            return;
        }
        let obj = r.a4;
        self.write8(obj + OBJ_TIMER, 0);
        self.write8(obj + OBJ_STEP, 0);
        self.write8(obj + OBJ_FLAGS, OBJ_FLAG_MOVING);
        self.write8(obj + OBJ_KIND, 8);
        self.write8(obj + OBJ_16, 0);
        self.write8(obj + OBJ_17, 0);
        self.d_set_anim(r);
        self.write8(obj + OBJ_KIND, 0);
        self.write16(obj + OBJ_RISE, 0xFD00);
        let direction = self.read8(obj + OBJ_DIRECTION);
        r.d0 = low_byte(r.d0, direction);
        if direction & 0x80 == 0 {
            let offset = (direction & DIRECTION_BITS) * 4;
            r.d0 = low_word(r.d0, u16::from(offset));
            r.x = false;
            r.c = false;
            r.a0 = self.read32(JUMP_ROUTINES + u32::from(offset));
            let mut sub = *r;
            self.call_family_18(r.a0, &mut sub);
            (r.d0, r.d4, r.a4) = (sub.d0, sub.d4, sub.a4);
        }
        self.write32(r.a4 + OBJ_ROUTINE, FALLING);
        (r.d0, r.x, r.c) = self.r_004fd0(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5);
        r.c = false;
        self.d_sound(r, SOUND_JUMP);
    }

    /// The made one's speed (`OBJ_SPEED`) by `low` (the low four bits
    /// of its x or y, those negated, d0's low byte), from `SPEEDS`.
    fn d_speed(&mut self, r: &mut Regs, field: u32, negate: bool) {
        self.d_made_quiet(r);
        let low = self.read8(r.a4 + field) & 0x0F;
        let low = if negate { low ^ 0x0F } else { low };
        r.d0 = low_byte(r.d0, low);
        r.d0 = self.r_004aa2(r.d0, r.a4);
    }

    /// d4 and a4 back to the made one, the flags left alone (family
    /// 18 doesn't give them back).
    fn d_made_quiet(&mut self, r: &mut Regs) {
        r.d4 = low_byte(r.d4, self.read8(MADE));
        r.a4 = self.read32(MADE_ADDR);
    }

    /// `func_004A8E` (direction 0's speed): by y's low bits, negated.
    /// No script calls it: reviewed by reading.
    pub fn r_004a8e(&mut self, regs: &mut Regs) {
        self.d_speed(regs, OBJ_Y_LOW, true);
    }

    /// `loc_004AA2`: the made one's speed `SPEEDS[d0]` (d0's low byte,
    /// sign-extended to its low word).
    pub fn r_004aa2(&mut self, d0: u32, obj: u32) -> u32 {
        let d0 = low_word(d0, d0 as u8 as i8 as u16);
        let speed = self.read8(SPEEDS.wrapping_add(d0 as u16 as i16 as u32));
        self.write8(obj + OBJ_SPEED_LOW, speed);
        self.write8(obj + OBJ_SPEED, 0);
        d0
    }

    /// `func_004AB0` (direction 1's speed): by x's low bits.
    pub fn r_004ab0(&mut self, regs: &mut Regs) {
        self.d_speed(regs, OBJ_X_LOW, false);
    }

    /// `func_004AC2` (direction 2's speed): by y's low bits.
    pub fn r_004ac2(&mut self, regs: &mut Regs) {
        self.d_speed(regs, OBJ_Y_LOW, false);
    }

    /// `func_004AD4` (direction 3's speed): by x's low bits, negated.
    pub fn r_004ad4(&mut self, regs: &mut Regs) {
        self.d_speed(regs, OBJ_X_LOW, true);
    }

    /// `func_004AFA` (`TIMED`): `func_002F70` while `OBJ_TIMER` counts
    /// down; at 0 the made one stands again (`STANDING`, not busy).
    /// Gives back X and C (the count's borrow, clear at the end).
    pub fn r_004afa(&mut self, regs: &mut Regs) {
        let r = regs;
        self.d_note_other(r);
        self.d_2f70(r);
        let obj = r.a4;
        let (left, borrow) = self.read8(obj + OBJ_TIMER).overflowing_sub(1);
        self.write8(obj + OBJ_TIMER, left);
        r.x = borrow;
        r.c = borrow;
        if left == 0 {
            self.write8(obj + OBJ_STEP, 0);
            let direction = self.read8(obj + OBJ_DIRECTION);
            self.write8(obj + OBJ_DIRECTION, direction & !DIRECTION_BUSY);
            self.write32(obj + OBJ_ROUTINE, STANDING);
            r.c = false;
        }
    }

    /// `func_004B26` (`CARRYING`): the other's record cleared
    /// (`func_004BD8`); unless `func_001FD4` sets C: a sound (0x41 or
    /// 0x42 at random) at the animation's start, `func_001B32`, the
    /// animation a frame on, the other follows and the depths as the
    /// direction `DIRECTION_BY_FRAME` of the frame says; when
    /// `BITS_A932`'s bit 4 is clear it stands again.
    /// No script calls it: reviewed by reading.
    pub fn r_004b26(&mut self, regs: &mut Regs) {
        let r = regs;
        let other = self.d_note_other(r);
        r.d0 = low_byte(r.d0, other);
        (r.d4, r.a0, r.a4) = self.r_004bd8(r.d0, r.d4);
        if self.d_1fd4(r, false) || r.c {
            return;
        }
        if self.read8(r.a4 + OBJ_TIMER) == 0 && self.read8(r.a4 + OBJ_STEP) & 0x02 == 0 {
            r.d0 = self.r_001bc0(2, r.d1);
            let sound = (r.d0 as u8).wrapping_add(0x41);
            self.d_sound(r, u32::from(sound));
        }
        self.d_task_sprite(r);
        self.d_1b32(r);
        self.d_anim(r);
        let direction = self.read8(r.a4 + OBJ_DIRECTION);
        let frame = self.read8(r.a5 + SPRITE_FRAME);
        r.d0 = low_word(r.d0, frame as i8 as u16);
        let shown = self.read8(DIRECTION_BY_FRAME.wrapping_add(frame as i8 as u32));
        self.write8(r.a4 + OBJ_DIRECTION, shown);
        self.d_task_sprite(r);
        self.d_follow(r);
        self.d_depths(r);
        self.d_made(r);
        self.write8(r.a4 + OBJ_DIRECTION, direction);
        r.c = false;
        if self.read8(BITS_A932) & MOVE_BIT == 0 {
            let direction = self.read8(r.a4 + OBJ_DIRECTION);
            self.write8(r.a4 + OBJ_DIRECTION, direction & !DIRECTION_BUSY);
            self.write8(r.a4 + OBJ_TIMER, 0);
            self.write8(r.a4 + OBJ_STEP, 0);
            self.d_set_anim(r);
            self.write32(r.a4 + OBJ_ROUTINE, STANDING);
            r.c = false;
        }
    }

    /// `func_004BD8`: the other's (`d0`) record's bytes 2 and 3 cleared
    /// but for the high four bits; d4 and a4 back to the made one.
    /// Gives back d4, a0 (`RECORDS`) and a4.
    pub fn r_004bd8(&mut self, d0: u32, d4: u32) -> (u32, u32, u32) {
        for at in [2, 3] {
            let record = Self::d_record(d0 as u8, at);
            let b = self.read8(record);
            self.write8(record, b & 0xF0);
        }
        let d4 = low_byte(d4, self.read8(MADE));
        (d4, RECORDS, self.read32(MADE_ADDR))
    }

    /// `func_004BF6` (`PUSHING`): unless `BITS_9540`'s bit 6 is set
    /// (then it stands again), the other's record gets the bits of the
    /// direction (`BITS_BY_DIRECTION`); unless `func_001FD6` sets C:
    /// `func_0015E6`, `func_00173E` that way, and if the byte at
    /// `POINTER_9ED6` is 3 or more, every third time (`COUNTS`)
    /// `func_001542` and a shadow (`func_004CA8`), then `func_001B32`,
    /// the other follows and the depths; else it stands again.
    pub fn r_004bf6(&mut self, regs: &mut Regs) {
        let r = regs;
        if self.read8(BITS_9540) & 0x40 == 0 {
            let other = self.d_note_other(r);
            r.d0 = low_byte(r.d0, other);
            (r.d4, r.a0, r.a4) = self.r_004bd8(r.d0, r.d4);
            let direction = self.read8(r.a4 + OBJ_DIRECTION) & DIRECTION_BITS;
            r.d0 = low_word(r.d0, u16::from(direction));
            let bits = self.read8(BITS_BY_DIRECTION + u32::from(direction));
            r.d0 = low_byte(r.d0, bits);
            r.a0 = OTHERS;
            r.d4 = low_word(r.d4, r.d4 as u8 as i8 as u16);
            let other = self.read8(by_made(OTHERS, r.d4));
            r.d1 = low_word(r.d1, (other as i8 as u16).wrapping_shl(3));
            r.a0 = RECORDS;
            let record = Self::d_record(other, 2);
            let b = self.read8(record);
            self.write8(record, b | bits);
            self.d_task_sprite(r);
            if self.d_1fd4(r, true) || r.c {
                return;
            }
            self.d_15e6(r);
            let direction = self.read8(r.a4 + OBJ_DIRECTION) & DIRECTION_BITS;
            r.d0 = low_byte(r.d0, direction);
            r.x = self.r_00173e(r.d0, r.a4);
            r.a0 = self.read32(POINTER_9ED6);
            let b = self.read8(r.a0);
            r.d0 = low_byte(r.d0, b);
            if b >= 3 {
                r.d4 = low_word(r.d4, r.d4 as u8 as i8 as u16);
                r.a0 = COUNTS;
                let count = COUNTS.wrapping_add(r.d4 as u16 as i16 as u32);
                let left = self.read8(count).wrapping_sub(1);
                self.write8(count, left);
                if left == 0 {
                    self.write8(count, 3);
                    self.r_001542(r.a4);
                    (r.d4, r.d5, r.d6, r.a4, r.a6) = self.r_004ca8(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
                }
                self.d_task_sprite(r);
                self.d_1b32(r);
                self.d_follow(r);
                self.d_depths(r);
                return;
            }
        }
        self.write32(r.a4 + OBJ_ROUTINE, STANDING);
        r.c = false;
    }

    /// `func_004CA8`: a shadow of the made one: a task (`SHADOW`) with a
    /// sprite record (`func_001222`, depth 0xF8, priority 0xC2) for
    /// 0xB frames (`func_00152A`), the record like the made one's
    /// (`SPRITE`: palette, tiles, flags + `ALTERNATE`, frame 0), the
    /// made one's sprite index in the task's `TASK_COPIED`, and its
    /// first frame now (`SPRITE` the shadow's meanwhile).  d4 and a4
    /// back to the made one.  Gives back d4, d5, d6, a4, a6.
    #[allow(clippy::too_many_arguments)]
    pub fn r_004ca8(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, u32, u32, u32, u32) {
        let (d5, d6, a5, task, x, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0xF8, 0xC2, SHADOW);
        let mut r = Regs { d0, d1, d2, d3, d4, d5, d6, d7, a0, a1, a2, a3, a4, a5, a6: task, x, ..Regs::default() };
        if !failed {
            self.write8(task + TASK_FRAMES, 0x0B);
            self.r_00152a(task);
            let made_index = self.read8(SPRITE);
            r.d6 = low_byte(r.d6, made_index);
            let made = self.read32(SPRITE_ADDR);
            self.write8(a5 + SPRITE_FRAME, 0);
            let palette = self.read8(made + SPRITE_PALETTE);
            self.write8(a5 + SPRITE_PALETTE, palette);
            let tiles = self.read16(made + SPRITE_TILES);
            self.write16(a5 + SPRITE_TILES, tiles);
            let flags = self.read8(made + SPRITE_FLAGS) | ALTERNATE;
            r.d0 = low_byte(r.d0, flags);
            self.write8(a5 + SPRITE_FLAGS, flags);
            r.d0 = low_byte(r.d0, made_index);
            r.d6 = low_byte(r.d6, self.read8(NEW_TASK));
            (r.d6, r.a6, r.x) = self.r_0005f8(r.d6);
            self.write8(r.a6 + TASK_COPIED, made_index);
            let kept = (self.read8(SPRITE), self.read32(SPRITE_ADDR));
            self.write8(SPRITE, d5 as u8);
            self.write32(SPRITE_ADDR, a5);
            r.c = false;
            self.r_004d24(&mut r);
            self.write32(SPRITE_ADDR, kept.1);
            self.write8(SPRITE, kept.0);
        }
        self.d_made_quiet(&mut r);
        (r.d4, r.d5, r.d6, r.a4, r.a6)
    }

    /// `func_004D24` (`SHADOW`, a task): for its frames its sprite
    /// record shows the frame of the one at `TASK_COPIED` (none while
    /// that one is `BUDGETED`), then `func_001B7E` and `func_001B5C`; at
    /// the end the task and its record go (`func_00121A`).
    pub fn r_004d24(&mut self, regs: &mut Regs) {
        let r = regs;
        let task = r.a6;
        let (left, borrow) = self.read8(task + TASK_FRAMES).overflowing_sub(1);
        self.write8(task + TASK_FRAMES, left);
        r.x = borrow;
        r.c = borrow;
        if left == 0 {
            (r.d6, r.x, r.c) = self.r_00121a(r.d5, r.d6, r.x);
            return;
        }
        r.d5 = low_byte(r.d5, self.read8(task + TASK_COPIED));
        let (d5, copied, x) = self.r_000b5a(r.d5);
        r.x = x;
        let own = self.read32(SPRITE_ADDR);
        self.write8(own + SPRITE_FRAME, 0);
        if self.read8(copied + SPRITE_FLAGS) & BUDGETED == 0 {
            let frame = self.read8(copied + SPRITE_FRAME);
            self.write8(own + SPRITE_FRAME, frame);
        }
        r.d5 = d5;
        self.d_task_sprite(r);
        (r.d0, r.x, r.c) = self.r_001b7e(r.a5, r.a6);
        (r.d0, r.x, r.c) = self.r_001b5c(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a5);
    }

    /// `func_004D6C` (an object's routine): its animation a frame on,
    /// `func_004E5C`, the other follows, the depths.
    /// No script calls it: reviewed by reading.
    pub fn r_004d6c(&mut self, regs: &mut Regs) {
        self.d_anim(regs);
        self.r_004e5c(regs);
        self.d_follow(regs);
        self.d_depths(regs);
    }

    /// `func_004D7E` (`FALLING`): its animation a frame on; at step 4
    /// it moves (`func_004DFE`, unless busy), `func_001AF6`, and once
    /// `func_001ABC` and `OBJ_RISE` aren't negative it has landed
    /// (height, timer, flags 0; the other's `OBJ_3A` 0; `LANDED`).
    /// Then `func_001B32`, `func_002F70`, the other follows, the depths.
    pub fn r_004d7e(&mut self, regs: &mut Regs) {
        let r = regs;
        self.d_note_other(r);
        self.d_anim(r);
        if self.read8(r.a4 + OBJ_STEP) == 4 {
            if self.read8(r.a4 + OBJ_DIRECTION) & 0x80 == 0 {
                (r.d1, r.d4, r.d5, r.d6, r.a4, r.a5, r.a6) = self.r_004dfe(r.d1, r.d2, r.d4, r.d5, r.d6, r.a6);
            }
            r.d0 = self.r_001af6(r.a4);
            let (d0, x, negative) = self.r_001abc(r.d0, r.a4);
            (r.d0, r.x) = (d0, x);
            if !negative && self.read8(r.a4 + OBJ_RISE) & 0x80 == 0 {
                self.write8(r.a4 + OBJ_HEIGHT, 0);
                self.write8(r.a4 + OBJ_TIMER, 0);
                self.write8(r.a4 + OBJ_FLAGS, 0);
                self.d_other(r);
                self.write8(r.a4 + OBJ_3A, 0);
                self.d_made(r);
                self.write32(r.a4 + OBJ_ROUTINE, LANDED);
            }
        }
        self.d_1b32(r);
        self.d_note_other(r);
        self.d_2f70(r);
        self.d_follow(r);
        self.d_depths(r);
    }

    /// `func_004DFE`: the made one moves (`func_004EA6`); while its
    /// `OBJ_RISE` is negative and `func_0016CE` that way gives 4 or more
    /// the other's flags have `OBJ_FLAGS_60` meanwhile (put back after).
    /// d4 and a4 back to the made one.  Gives back d1, d4, d5, d6, a4,
    /// a5, a6.
    pub fn r_004dfe(&mut self, d1: u32, d2: u32, d4: u32, d5: u32, d6: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
        let other_index = self.read8(OTHER);
        let (_, other) = self.r_001d0c(low_byte(d5, other_index));
        let flags = self.read8(other + OBJ_FLAGS);
        let d4 = low_byte(d4, self.read8(MADE));
        let obj = self.read32(MADE_ADDR);
        let mut d1 = d1;
        if self.read8(obj + OBJ_RISE) & 0x80 != 0 {
            let (d0, _) = self.r_0015e6(obj);
            let direction = self.read8(obj + OBJ_DIRECTION) & DIRECTION_BITS;
            let (d0, d1b) = self.r_0016ce(low_byte(d0, direction), d1);
            d1 = d1b;
            if d0 as u8 >= 4 {
                let b = self.read8(other + OBJ_FLAGS);
                self.write8(other + OBJ_FLAGS, b | OBJ_FLAGS_60);
            }
        }
        let d5 = low_byte(d5, self.read8(SPRITE));
        let (d1, d4, d5, d6, _, _, a5, a6) = self.r_004ea6(d1, d2, d4, d5, d6, obj, a6);
        let other_index = self.read8(OTHER);
        let (d4, other, _) = self.r_001cfa(low_byte(d4, other_index));
        self.write8(other + OBJ_FLAGS, flags);
        let d4 = low_byte(d4, self.read8(MADE));
        (d1, d4, d5, d6, self.read32(MADE_ADDR), a5, a6)
    }

    /// `func_004E5C` (an object's routine, `d4` its index, kept in
    /// `OTHER`): it moves (`func_004EA6`) unless busy, `func_001AF6`,
    /// and once `func_001ABC` isn't negative it has landed (height 0,
    /// not busy, not moving, its routine back from
    /// `OBJ_SAVED_ROUTINE`); `func_002F70` for indexes under 5, then
    /// `func_001B32` and `func_001B50`.
    pub fn r_004e5c(&mut self, regs: &mut Regs) {
        let r = regs;
        self.write8(OTHER, r.d4 as u8);
        if self.read8(r.a4 + OBJ_DIRECTION) & 0x80 == 0 {
            (r.d1, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6) = self.r_004ea6(r.d1, r.d2, r.d4, r.d5, r.d6, r.a4, r.a6);
        }
        r.d0 = self.r_001af6(r.a4);
        let (d0, x, negative) = self.r_001abc(r.d0, r.a4);
        (r.d0, r.x) = (d0, x);
        let obj = r.a4;
        if !negative {
            self.write8(obj + OBJ_HEIGHT, 0);
            let direction = self.read8(obj + OBJ_DIRECTION);
            self.write8(obj + OBJ_DIRECTION, direction & !DIRECTION_BUSY);
            let flags = self.read8(obj + OBJ_FLAGS);
            self.write8(obj + OBJ_FLAGS, flags & !OBJ_FLAG_MOVING);
            let saved = self.read32(obj + OBJ_SAVED_ROUTINE);
            self.write32(obj + OBJ_ROUTINE, saved);
        }
        if self.read8(OTHER) < 5 {
            self.d_2f70(r);
        }
        self.d_1b32(r);
        (r.d0, r.x, r.c) = self.r_001b50(r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5);
    }

    /// `func_004EA6`: the object at a4 moves a step its way
    /// (`func_004ED8`) at its speed: `BITS_A932` cleared,
    /// `func_0015E6`, `func_00157A`, the position's low bytes, the speed
    /// and its negation, `MOVE_X` and `MOVE_Y` 0.  Gives back d1, d4,
    /// d5, d6, a0, a4, a5, a6 as `func_004ED8` does.
    #[allow(clippy::too_many_arguments)]
    pub fn r_004ea6(&mut self, d1: u32, d2: u32, d4: u32, d5: u32, d6: u32, a4: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        self.write8(BITS_A932, 0);
        let (d0, _) = self.r_0015e6(a4);
        self.r_00157a();
        let x = self.read8(a4 + OBJ_X_LOW);
        self.write8(MOVE_X_LOW, x);
        let y = self.read8(a4 + OBJ_Y_LOW);
        self.write8(MOVE_Y_LOW, y);
        let speed = self.read16(a4 + OBJ_SPEED);
        self.write16(SPEED_POS, speed);
        self.write16(SPEED_NEG, speed.wrapping_neg());
        self.write32(MOVE_X, 0);
        let direction = self.read8(a4 + OBJ_DIRECTION) & DIRECTION_BITS;
        let d0 = (d0 & 0xFFFF_0000) | u32::from(speed.wrapping_neg() & 0xFF00) | u32::from(direction);
        self.r_004ed8(d0, d1, d2, d4, d5, d6, a6)
    }

    /// `func_004ED8`: the move by the direction `d0` (its low byte;
    /// `MOVE_ROUTINES`, family 19), then d5 and a5 `SPRITE`'s.
    #[allow(clippy::too_many_arguments)]
    pub fn r_004ed8(&mut self, d0: u32, d1: u32, d2: u32, d4: u32, d5: u32, d6: u32, a6: u32) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        let (twice, _) = (d0 as u8).overflowing_add(d0 as u8);
        let (offset, carry) = twice.overflowing_add(twice);
        let d0 = low_word(d0, u16::from(offset));
        let a0 = self.read32(MOVE_ROUTINES + u32::from(offset));
        // the andi.w clears C
        let mut r = Regs { d0, d1, d2, d4, d5, d6, a0, a6, x: carry, ..Regs::default() };
        self.call_family_19(a0, &mut r);
        self.d_task_sprite(&mut r);
        (r.d1, r.d4, r.d5, r.d6, r.a0, r.a4, r.a5, r.a6)
    }

    /// `func_004F00` (direction 0's move): `MOVE_Y` the speed negated,
    /// `func_002846`; then if y's high byte is 0 and its low one under
    /// 8, y is 8.
    pub fn r_004f00(&mut self, regs: &mut Regs) {
        let r = regs;
        self.d_made(r);
        let speed = self.read16(SPEED_NEG);
        self.write16(MOVE_Y, speed);
        (r.d0, r.d1, r.d4, r.d6, r.a0, r.a4, r.a6, r.x, r.c) = self.r_002846(r.d0, r.d1, r.d2, r.d4, r.d6, r.a0, r.a4, r.a6, r.x);
        self.d_made(r);
        if self.read8(r.a4 + OBJ_Y) == 0 {
            let low = self.read8(r.a4 + OBJ_Y_LOW);
            r.d0 = low_byte(r.d0, low);
            r.c = low < 8;
            if low < 8 {
                self.write16(r.a4 + OBJ_Y, 8);
                r.c = false;
            }
        }
    }

    /// `func_004F36` (direction 1's move): `MOVE_X` the speed,
    /// `func_002596`.
    pub fn r_004f36(&mut self, regs: &mut Regs) {
        let r = regs;
        self.d_made(r);
        let speed = self.read16(SPEED_POS);
        self.write16(MOVE_X, speed);
        (r.d0, r.d1, r.d4, r.d6, r.a0, r.a4, r.a6, r.x, r.c) = self.r_002596(r.d0, r.d1, r.d2, r.d4, r.d6, r.a0, r.a4, r.a6, r.x);
    }

    /// `func_004F48` (direction 2's move): `MOVE_Y` the speed,
    /// `func_00299E`.
    pub fn r_004f48(&mut self, regs: &mut Regs) {
        let r = regs;
        self.d_made(r);
        let speed = self.read16(SPEED_POS);
        self.write16(MOVE_Y, speed);
        (r.d0, r.d1, r.d4, r.d6, r.a0, r.a4, r.a6, r.x, r.c) = self.r_00299e(r.d0, r.d1, r.d2, r.d4, r.d6, r.a0, r.a4, r.a6, r.x);
    }

    /// `func_004F5A` (direction 3's move): `MOVE_X` the speed negated,
    /// `func_0026EE`.
    pub fn r_004f5a(&mut self, regs: &mut Regs) {
        let r = regs;
        self.d_made(r);
        let speed = self.read16(SPEED_NEG);
        self.write16(MOVE_X, speed);
        (r.d0, r.d1, r.d4, r.d6, r.a0, r.a4, r.a6, r.x, r.c) = self.r_0026ee(r.d0, r.d1, r.d2, r.d4, r.d6, r.a0, r.a4, r.a6, r.x);
    }

    /// `func_004F6C` (`LANDED`): `func_0053A6`, `func_00211C`; while
    /// `BITS_A932` is clear and its animation hasn't ended:
    /// `func_001B32`, `func_002F70`, `func_0015E6`, `func_00167A`,
    /// `func_003032`, the other follows.  Else it stands again
    /// (`OBJ_KIND` 0, its animation from the start, `STANDING`, run
    /// now).
    pub fn r_004f6c(&mut self, regs: &mut Regs) {
        let r = regs;
        let direction = self.read8(r.a4 + OBJ_DIRECTION) & DIRECTION_BITS;
        self.write8(r.a4 + OBJ_DIRECTION, direction);
        self.d_note_other(r);
        (r.d0, r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a0, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6) = self.r_0053a6(r.d1, r.d2, r.d3, r.d4, r.d5, r.d6, r.d7, r.a1, r.a2, r.a3, r.a4, r.a5, r.a6);
        if self.restarted.is_some() {
            return;
        }
        (r.d0, r.d1, r.d4, r.d5, r.d7, r.a0, r.a4, r.a5, r.x) = self.r_00211c(r.d0, r.d1, r.d2, r.d4, r.d5, r.d7, r.a0, r.a5);
        if self.read8(BITS_A932) == 0 {
            self.d_anim(r);
            if !r.c {
                self.d_1b32(r);
                self.d_2f70(r);
                self.d_15e6(r);
                self.d_167a(r);
                self.d_3032(r);
                self.d_follow(r);
                return;
            }
        }
        self.write8(r.a4 + OBJ_KIND, 0);
        self.d_set_anim(r);
        self.write8(r.a4 + OBJ_TIMER, 0);
        self.write8(r.a4 + OBJ_STEP, 0);
        self.d_anim(r);
        self.write32(r.a4 + OBJ_ROUTINE, STANDING);
        r.c = false;
        self.r_0045e2(r);
    }

    /// A task (`func_001222`, depth 0xFC, priority 0xC3) running
    /// `routine` with a sprite record (flags 4), 3 frames, its byte
    /// 0x13 `value`'s low byte.  Gives back d0 (`value` if made), X and
    /// C (set if it couldn't be).
    #[allow(clippy::too_many_arguments)]
    fn d_sprite_task(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, routine: u32, value: u32) -> (u32, bool, bool) {
        let (_, _, sprite, task, x, failed) = self.r_001222(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, 0xFC, 0xC3, routine);
        if failed {
            return (d0, x, true);
        }
        self.write8(sprite + SPRITE_FLAGS, 4);
        self.write8(task + TASK_FRAMES, 3);
        self.write8(task + TASK_13, value as u8);
        (value, x, false)
    }

    /// `func_004FD0`: the task `SPRITE_TASK_A` (`d_sprite_task`) with
    /// d4's low byte.  Gives back d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_004fd0(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, bool, bool) {
        self.d_sprite_task(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, SPRITE_TASK_A, d4)
    }

    /// `func_004FF4`: the task `SPRITE_TASK_B` (`d_sprite_task`) with
    /// d6's low byte.  Gives back d0, X and C.
    #[allow(clippy::too_many_arguments)]
    pub fn r_004ff4(&mut self, d0: u32, d1: u32, d2: u32, d3: u32, d4: u32, d5: u32, d6: u32, d7: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> (u32, bool, bool) {
        self.d_sprite_task(d0, d1, d2, d3, d4, d5, d6, d7, a1, a2, a3, a4, a5, SPRITE_TASK_B, d6)
    }
}
