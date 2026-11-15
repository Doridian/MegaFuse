//! The YM2612 (OPN2): six four-operator FM channels, the DAC, the LFO and
//! the timers, a sample every 144 master clocks (some 53.3 kHz).  The
//! envelope follows the chip's rate tables; levels are worked out in
//! floating point from its attenuation steps (0.09375 dB an envelope
//! step, 0.75 dB a total-level step).
//!
//! The Model 1's YM2612 puts each channel out through a 9-bit DAC: the
//! carriers lose their low 5 bits of 14, and the DAC's "ladder effect"
//! pushes every channel 4 steps (of 256) away from zero, 3 on the
//! negative side when the channel is on that speaker.  Quiet notes come
//! out louder and harsher for it ([`Ym2612::model1_dac`]; Genesis Plus
//! GX's default, "mame (ym2612)").

use std::f32::consts::TAU;

const ATTACK: u8 = 0;
const DECAY: u8 = 1;
const SUSTAIN: u8 = 2;
const RELEASE: u8 = 3;
const MAX_ATT: u32 = 1023;

// how far the envelope moves at each step, by (rate & 3) and the step
const EG_INC: [[u32; 8]; 4] = [
    [0, 1, 0, 1, 0, 1, 0, 1],
    [0, 1, 0, 1, 1, 1, 0, 1],
    [0, 1, 1, 1, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 1],
];

const DT_TABLE: [[u8; 4]; 32] = [
    [0, 0, 1, 2], [0, 0, 1, 2], [0, 0, 1, 2], [0, 0, 1, 2],
    [0, 1, 2, 2], [0, 1, 2, 3], [0, 1, 2, 3], [0, 1, 2, 3],
    [0, 1, 2, 4], [0, 1, 3, 4], [0, 1, 3, 4], [0, 1, 3, 5],
    [0, 2, 4, 5], [0, 2, 4, 6], [0, 2, 4, 6], [0, 2, 5, 7],
    [0, 2, 5, 8], [0, 3, 6, 8], [0, 3, 6, 9], [0, 3, 7, 10],
    [0, 4, 8, 11], [0, 4, 8, 12], [0, 4, 9, 13], [0, 5, 10, 14],
    [0, 5, 11, 16], [0, 6, 12, 17], [0, 6, 13, 19], [0, 7, 14, 20],
    [0, 8, 16, 22], [0, 8, 16, 22], [0, 8, 16, 22], [0, 8, 16, 22],
];

// LFO frequencies in samples per step (of 128 steps a cycle)
const LFO_PERIOD: [u32; 8] = [108, 77, 71, 67, 62, 44, 8, 5];
const AMS_SHIFT: [u32; 4] = [8, 3, 1, 0];
// phase modulation depth, cents-ish fractions of the frequency
const PMS_DEPTH: [f32; 8] = [0.0, 0.034, 0.067, 0.10, 0.14, 0.20, 0.40, 0.80];

#[derive(Clone, Copy, Default)]
struct Op {
    dt: u8,
    mul: u8,
    tl: u8,
    ks: u8,
    ar: u8,
    am: bool,
    d1r: u8,
    d2r: u8,
    d1l: u8,
    rr: u8,
    ssg: u8,
    phase: u32,
    att: u32,
    stage: u8,
    key: bool,
}

#[derive(Clone, Copy, Default)]
struct Channel {
    op: [Op; 4],
    fnum: u16,
    block: u8,
    fb: u8,
    alg: u8,
    left: bool,
    right: bool,
    ams: u8,
    pms: u8,
    fb_out: [f32; 2],
}

#[derive(Clone)]
pub struct Ym2612 {
    ch: [Channel; 6],
    /// Channel 3's special mode: its operators 1-3 have their own
    /// frequencies.
    ch3_fnum: [u16; 3],
    ch3_block: [u8; 3],
    ch3_special: bool,
    addr: [u8; 2],
    latch_hi: u8,
    latch_hi3: u8,
    dac_on: bool,
    dac: f32,
    lfo_on: bool,
    lfo_freq: u8,
    lfo_cnt: u32,
    lfo_step: u32,
    eg_counter: u32,
    eg_div: u32,
    timer_a: u16,
    timer_b: u8,
    timer_a_cnt: i32,
    timer_b_cnt: i32,
    timer_ctrl: u8,
    pub status: u8,
    /// The Model 1's DAC: 9 bits and the ladder effect (see above).
    pub model1_dac: bool,
    sin: Vec<f32>,
}

impl Default for Ym2612 {
    fn default() -> Self {
        Self::new()
    }
}

impl Ym2612 {
    #[must_use]
    pub fn new() -> Self {
        let mut ch = [Channel::default(); 6];
        for c in &mut ch {
            c.left = true;
            c.right = true;
            for o in &mut c.op {
                o.att = MAX_ATT;
                o.stage = RELEASE;
            }
        }
        Ym2612 {
            ch,
            ch3_fnum: [0; 3],
            ch3_block: [0; 3],
            ch3_special: false,
            addr: [0; 2],
            latch_hi: 0,
            latch_hi3: 0,
            dac_on: false,
            dac: 0.0,
            lfo_on: false,
            lfo_freq: 0,
            lfo_cnt: 0,
            lfo_step: 0,
            eg_counter: 0,
            eg_div: 0,
            timer_a: 0,
            timer_b: 0,
            timer_a_cnt: 0,
            timer_b_cnt: 0,
            timer_ctrl: 0,
            status: 0,
            model1_dac: true,
            sin: (0..1024).map(|i| (i as f32 / 1024.0 * TAU).sin()).collect(),
        }
    }

    /// A write to 0x4000-0x4003 (port 0/1 address and data).
    pub fn write(&mut self, port: u8, value: u8) {
        let part = usize::from((port >> 1) & 1);
        if port & 1 == 0 {
            self.addr[part] = value;
            return;
        }
        let reg = self.addr[part];
        if part == 0 && reg < 0x30 {
            self.write_global(reg, value);
        } else {
            self.write_channel(part, reg, value);
        }
    }

    fn write_global(&mut self, reg: u8, v: u8) {
        match reg {
            0x22 => {
                self.lfo_on = v & 8 != 0;
                self.lfo_freq = v & 7;
                if !self.lfo_on {
                    self.lfo_step = 0;
                }
            }
            0x24 => self.timer_a = (self.timer_a & 3) | (u16::from(v) << 2),
            0x25 => self.timer_a = (self.timer_a & 0x3FC) | u16::from(v & 3),
            0x26 => self.timer_b = v,
            0x27 => {
                if v & 1 != 0 && self.timer_ctrl & 1 == 0 {
                    self.timer_a_cnt = 1024 - i32::from(self.timer_a);
                }
                if v & 2 != 0 && self.timer_ctrl & 2 == 0 {
                    self.timer_b_cnt = (256 - i32::from(self.timer_b)) * 16;
                }
                if v & 0x10 != 0 {
                    self.status &= !1;
                }
                if v & 0x20 != 0 {
                    self.status &= !2;
                }
                self.timer_ctrl = v;
                self.ch3_special = v & 0xC0 != 0;
            }
            0x28 => {
                let c = usize::from(v & 3);
                if c == 3 {
                    return;
                }
                let c = c + if v & 4 != 0 { 3 } else { 0 };
                for k in 0..4 {
                    // the key bits are op1 op2 op3 op4 in 4-7
                    let on = v & (0x10 << k) != 0;
                    let o = &mut self.ch[c].op[k];
                    if on && !o.key {
                        o.phase = 0;
                        o.stage = ATTACK;
                        if Self::attack_rate(o, self.ch[c].fnum, self.ch[c].block) >= 62 {
                            o.att = 0;
                            o.stage = DECAY;
                        }
                    } else if !on && o.key {
                        o.stage = RELEASE;
                    }
                    self.ch[c].op[k].key = on;
                }
            }
            0x2A => self.dac = (f32::from(v) - 128.0) / 128.0,
            0x2B => self.dac_on = v & 0x80 != 0,
            _ => {}
        }
    }

    fn write_channel(&mut self, part: usize, reg: u8, v: u8) {
        let c = usize::from(reg & 3);
        if c == 3 {
            return;
        }
        let ch = c + part * 3;
        // register order of operators: 1 3 2 4
        let k = [0, 2, 1, 3][usize::from((reg >> 2) & 3)];
        match reg & 0xF0 {
            0x30 => {
                let o = &mut self.ch[ch].op[k];
                o.dt = (v >> 4) & 7;
                o.mul = v & 15;
            }
            0x40 => self.ch[ch].op[k].tl = v & 0x7F,
            0x50 => {
                let o = &mut self.ch[ch].op[k];
                o.ks = v >> 6;
                o.ar = v & 31;
            }
            0x60 => {
                let o = &mut self.ch[ch].op[k];
                o.am = v & 0x80 != 0;
                o.d1r = v & 31;
            }
            0x70 => self.ch[ch].op[k].d2r = v & 31,
            0x80 => {
                let o = &mut self.ch[ch].op[k];
                o.d1l = v >> 4;
                o.rr = v & 15;
            }
            0x90 => self.ch[ch].op[k].ssg = v & 15,
            0xA0 => match reg & 0x0C {
                0x00 => {
                    self.ch[ch].fnum = u16::from(self.latch_hi & 7) << 8 | u16::from(v);
                    self.ch[ch].block = (self.latch_hi >> 3) & 7;
                }
                0x04 => self.latch_hi = v & 0x3F,
                0x08 => {
                    if part == 0 {
                        self.ch3_fnum[c] = u16::from(self.latch_hi3 & 7) << 8 | u16::from(v);
                        self.ch3_block[c] = (self.latch_hi3 >> 3) & 7;
                    }
                }
                _ => {
                    if part == 0 {
                        self.latch_hi3 = v & 0x3F;
                    }
                }
            },
            0xB0 => match reg & 0x0C {
                0x00 => {
                    self.ch[ch].fb = (v >> 3) & 7;
                    self.ch[ch].alg = v & 7;
                }
                0x04 => {
                    let c = &mut self.ch[ch];
                    c.left = v & 0x80 != 0;
                    c.right = v & 0x40 != 0;
                    c.ams = (v >> 4) & 3;
                    c.pms = v & 7;
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// The key code (block and the top of the frequency number) that
    /// key scaling and detune go by.
    fn keycode(fnum: u16, block: u8) -> u8 {
        let f11 = (fnum >> 10) & 1;
        let f10 = (fnum >> 9) & 1;
        let f9 = (fnum >> 8) & 1;
        let f8 = (fnum >> 7) & 1;
        let n3 = f11 & (f10 | f9 | f8) | (!f11 & f10 & f9 & f8 & 1);
        (block << 2) | (f11 as u8) << 1 | (n3 & 1) as u8
    }

    fn rate(r: u8, o: &Op, fnum: u16, block: u8) -> u32 {
        if r == 0 {
            return 0;
        }
        let kc = u32::from(Self::keycode(fnum, block));
        let rks = kc >> (3 - u32::from(o.ks));
        (2 * u32::from(r) + rks).min(63)
    }

    fn attack_rate(o: &Op, fnum: u16, block: u8) -> u32 {
        Self::rate(o.ar, o, fnum, block)
    }

    fn envelope(o: &mut Op, fnum: u16, block: u8, counter: u32) {
        let r = match o.stage {
            ATTACK => Self::rate(o.ar, o, fnum, block),
            DECAY => Self::rate(o.d1r, o, fnum, block),
            SUSTAIN => Self::rate(o.d2r, o, fnum, block),
            _ => Self::rate(o.rr * 2 + 1, o, fnum, block),
        };
        if r < 4 {
            return;
        }
        let shift = 11u32.saturating_sub(r / 4);
        if counter & ((1 << shift) - 1) != 0 {
            return;
        }
        let mut inc = EG_INC[(r & 3) as usize][((counter >> shift) & 7) as usize];
        if r >= 48 {
            inc <<= (r / 4) - 12;
        }
        match o.stage {
            ATTACK => {
                if inc > 0 {
                    // exponential: the attenuation shrinks by a fraction
                    let a = o.att as i64;
                    let d = ((!a) * i64::from(inc)) >> 4;
                    let n = (a + d).max(0) as u32;
                    o.att = n;
                }
                if o.att == 0 {
                    o.stage = DECAY;
                }
            }
            DECAY => {
                o.att = (o.att + inc).min(MAX_ATT);
                let sl = if o.d1l == 15 { 31 } else { u32::from(o.d1l) } << 5;
                if o.att >= sl {
                    o.stage = SUSTAIN;
                }
            }
            _ => o.att = (o.att + inc).min(MAX_ATT),
        }
    }

    /// The step of the phase (of 2^20 a cycle) a sample.
    fn phase_inc(o: &Op, fnum: u16, block: u8, pm: f32) -> u32 {
        let f = f32::from(fnum) * (1.0 + pm);
        let base = (f * f32::from(1u16 << block)) / 2.0;
        let kc = usize::from(Self::keycode(fnum, block));
        let dt = i32::from(DT_TABLE[kc][usize::from(o.dt & 3)]);
        let dt = if o.dt & 4 != 0 { -dt } else { dt };
        let base = (base as i32 + dt).max(0) as f32;
        let mul = if o.mul == 0 { 0.5 } else { f32::from(o.mul) };
        (base * mul) as u32
    }

    fn op_out(&self, o: &Op, modulation: f32, am: u32) -> f32 {
        let att = o.att + (u32::from(o.tl) << 3) + if o.am { am } else { 0 };
        if att >= MAX_ATT {
            return 0.0;
        }
        let gain = 10f32.powf(-(att as f32) * 0.09375 / 20.0);
        let p = (o.phase >> 10) as f32 / 1024.0 + modulation;
        let i = ((p * 1024.0) as i32).rem_euclid(1024) as usize;
        // 14 bits, sign and magnitude: what is under a step is 0
        (self.sin[i] * gain * 8192.0).trunc() / 8192.0
    }

    /// One sample: (left, right), the channels summed, each -1..1.
    pub fn sample(&mut self) -> (f32, f32) {
        // the LFO
        let (am, pm_amount) = if self.lfo_on {
            self.lfo_cnt += 1;
            if self.lfo_cnt >= LFO_PERIOD[usize::from(self.lfo_freq)] {
                self.lfo_cnt = 0;
                self.lfo_step = (self.lfo_step + 1) & 127;
            }
            let tri = if self.lfo_step < 64 { self.lfo_step } else { 127 - self.lfo_step };
            let pm = (self.lfo_step as f32 / 128.0 * TAU).sin();
            (tri * 2, pm)
        } else {
            (0, 0.0)
        };
        // the envelopes, a step every 3 samples
        self.eg_div += 1;
        if self.eg_div >= 3 {
            self.eg_div = 0;
            self.eg_counter = self.eg_counter.wrapping_add(1);
            let counter = self.eg_counter;
            for c in 0..6 {
                for k in 0..4 {
                    let (fnum, block) = self.op_freq(c, k);
                    let o = &mut self.ch[c].op[k];
                    Self::envelope(o, fnum, block, counter);
                }
            }
        }
        // the timers (in samples: A counts every sample, B every 16)
        if self.timer_ctrl & 1 != 0 {
            self.timer_a_cnt -= 1;
            if self.timer_a_cnt <= 0 {
                self.timer_a_cnt += 1024 - i32::from(self.timer_a);
                if self.timer_ctrl & 4 != 0 {
                    self.status |= 1;
                }
            }
        }
        if self.timer_ctrl & 2 != 0 {
            self.timer_b_cnt -= 1;
            if self.timer_b_cnt <= 0 {
                self.timer_b_cnt += (256 - i32::from(self.timer_b)) * 16;
                if self.timer_ctrl & 8 != 0 {
                    self.status |= 2;
                }
            }
        }
        let mut left = 0.0;
        let mut right = 0.0;
        for c in 0..6 {
            let out = if c == 5 && self.dac_on {
                self.dac
            } else {
                self.channel(c, am, pm_amount)
            };
            let ch = &self.ch[c];
            if ch.left {
                left += out;
            }
            if ch.right {
                right += out;
            }
            if self.model1_dac {
                let step = 1.0 / 256.0;
                if out < 0.0 {
                    left -= if ch.left { 3.0 } else { 4.0 } * step;
                    right -= if ch.right { 3.0 } else { 4.0 } * step;
                } else {
                    left += 4.0 * step;
                    right += 4.0 * step;
                }
            }
        }
        (left, right)
    }

    fn op_freq(&self, c: usize, k: usize) -> (u16, u8) {
        if c == 2 && self.ch3_special && k < 3 {
            // ops 1 2 3 of channel 3 have 0xA9 0xAA 0xA8's frequencies
            let i = [1, 2, 0][k];
            (self.ch3_fnum[i], self.ch3_block[i])
        } else {
            (self.ch[c].fnum, self.ch[c].block)
        }
    }

    fn channel(&mut self, c: usize, am: u32, pm_amount: f32) -> f32 {
        let pm = pm_amount * PMS_DEPTH[usize::from(self.ch[c].pms)];
        let am = am >> AMS_SHIFT[usize::from(self.ch[c].ams)];
        for k in 0..4 {
            let (fnum, block) = self.op_freq(c, k);
            let inc = Self::phase_inc(&self.ch[c].op[k], fnum, block, pm);
            let o = &mut self.ch[c].op[k];
            o.phase = o.phase.wrapping_add(inc) & 0xFFFFF;
        }
        let ch = self.ch[c];
        let fb = if ch.fb > 0 {
            (ch.fb_out[0] + ch.fb_out[1]) / 2.0 * f32::from(1u16 << ch.fb) / 64.0
        } else {
            0.0
        };
        // an operator's 14-bit output, halved, moves the next one's 10-bit
        // phase: full scale is 4 cycles
        let m = 4.0;
        let dac9 = self.model1_dac;
        let q = |x: f32| if dac9 { (x * 256.0).floor() / 256.0 } else { x };
        let o1 = self.op_out(&ch.op[0], fb, am);
        let (o2, o3, o4);
        let out = match ch.alg {
            0 => {
                o2 = self.op_out(&ch.op[1], o1 * m, am);
                o3 = self.op_out(&ch.op[2], o2 * m, am);
                q(self.op_out(&ch.op[3], o3 * m, am))
            }
            1 => {
                o2 = self.op_out(&ch.op[1], 0.0, am);
                o3 = self.op_out(&ch.op[2], (o1 + o2) * m, am);
                q(self.op_out(&ch.op[3], o3 * m, am))
            }
            2 => {
                o2 = self.op_out(&ch.op[1], 0.0, am);
                o3 = self.op_out(&ch.op[2], o2 * m, am);
                q(self.op_out(&ch.op[3], (o1 + o3) * m, am))
            }
            3 => {
                o2 = self.op_out(&ch.op[1], o1 * m, am);
                o3 = self.op_out(&ch.op[2], 0.0, am);
                q(self.op_out(&ch.op[3], (o2 + o3) * m, am))
            }
            4 => {
                o2 = self.op_out(&ch.op[1], o1 * m, am);
                o3 = self.op_out(&ch.op[2], 0.0, am);
                o4 = self.op_out(&ch.op[3], o3 * m, am);
                q(o2) + q(o4)
            }
            5 => {
                o2 = self.op_out(&ch.op[1], o1 * m, am);
                o3 = self.op_out(&ch.op[2], o1 * m, am);
                o4 = self.op_out(&ch.op[3], o1 * m, am);
                q(o2) + q(o3) + q(o4)
            }
            6 => {
                o2 = self.op_out(&ch.op[1], o1 * m, am);
                o3 = self.op_out(&ch.op[2], 0.0, am);
                o4 = self.op_out(&ch.op[3], 0.0, am);
                q(o2) + q(o3) + q(o4)
            }
            _ => {
                o2 = self.op_out(&ch.op[1], 0.0, am);
                o3 = self.op_out(&ch.op[2], 0.0, am);
                o4 = self.op_out(&ch.op[3], 0.0, am);
                q(o1) + q(o2) + q(o3) + q(o4)
            }
        };
        let chm = &mut self.ch[c];
        chm.fb_out = [chm.fb_out[1], o1];
        out.clamp(-1.0, 1.0)
    }
}
