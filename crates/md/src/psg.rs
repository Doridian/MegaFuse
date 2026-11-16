//! The SN76489 (the VDP's PSG, Sega's version): three square-wave tones
//! and a noise channel, clocked at the Z80's 3.58 MHz over 16.

/// Attenuation in 2 dB steps; 15 is off.
const VOLUME: [f32; 16] = {
    let mut t = [0.0; 16];
    let mut v = 1.0f32;
    let mut i = 0;
    while i < 15 {
        t[i] = v;
        v *= 0.794_328_2; // -2 dB
        i += 1;
    }
    t
};

#[derive(Clone)]
pub struct Psg {
    /// Tone periods (10 bits) and the noise control (3 bits).
    period: [u16; 4],
    att: [u8; 4],
    count: [u16; 4],
    /// The tones' outputs (high or low); the noise's is the shift
    /// register's bit 0.
    level: [bool; 3],
    lfsr: u16,
    noise_edge: bool,
    /// The register the last latch byte named.
    latch: usize,
}

impl Default for Psg {
    fn default() -> Self {
        Self::new()
    }
}

impl Psg {
    #[must_use]
    pub fn new() -> Self {
        Psg {
            period: [0; 4],
            att: [15; 4],
            count: [0; 4],
            level: [false; 3],
            lfsr: 0x8000,
            noise_edge: false,
            latch: 0,
        }
    }

    pub fn write(&mut self, v: u8) {
        if v & 0x80 != 0 {
            self.latch = usize::from((v >> 4) & 7);
            let ch = self.latch >> 1;
            if self.latch & 1 != 0 {
                self.att[ch] = v & 15;
            } else if ch == 3 {
                self.period[3] = u16::from(v & 7);
                self.lfsr = 0x8000;
            } else {
                self.period[ch] = (self.period[ch] & 0x3F0) | u16::from(v & 15);
            }
        } else {
            let ch = self.latch >> 1;
            if self.latch & 1 != 0 {
                self.att[ch] = v & 15;
            } else if ch == 3 {
                self.period[3] = u16::from(v & 7);
                self.lfsr = 0x8000;
            } else {
                self.period[ch] = (self.period[ch] & 0xF) | (u16::from(v & 0x3F) << 4);
            }
        }
    }

    /// One step of the counters (every 16 clocks).
    fn step(&mut self) {
        for c in 0..3 {
            if self.count[c] > 0 {
                self.count[c] -= 1;
            }
            if self.count[c] == 0 {
                self.count[c] = self.period[c].max(1);
                self.level[c] = !self.level[c];
                if c == 2 && self.period[3] & 3 == 3 {
                    self.clock_noise();
                }
            }
        }
        if self.period[3] & 3 != 3 {
            if self.count[3] > 0 {
                self.count[3] -= 1;
            }
            if self.count[3] == 0 {
                self.count[3] = 0x10 << (self.period[3] & 3);
                self.clock_noise();
            }
        }
    }

    /// The noise's counter flips its output; the shift register moves
    /// on every other flip (the rising edges).
    fn clock_noise(&mut self) {
        self.noise_edge = !self.noise_edge;
        if !self.noise_edge {
            return;
        }
        let fb = if self.period[3] & 4 != 0 {
            (self.lfsr ^ (self.lfsr >> 3)) & 1
        } else {
            self.lfsr & 1
        };
        self.lfsr = (self.lfsr >> 1) | (fb << 15);
    }

    /// `steps` steps of the counters; the output averaged over them, the
    /// four channels summed, each -1..1.
    pub fn run(&mut self, steps: u32) -> f32 {
        let mut sum = 0.0;
        for _ in 0..steps {
            self.step();
            for c in 0..3 {
                // a period of 0 or 1 holds the output high (the sample trick)
                let high = self.level[c] || self.period[c] <= 1;
                sum += if high {
                    VOLUME[usize::from(self.att[c])]
                } else {
                    -VOLUME[usize::from(self.att[c])]
                };
            }
            sum += if self.lfsr & 1 != 0 {
                VOLUME[usize::from(self.att[3])]
            } else {
                -VOLUME[usize::from(self.att[3])]
            };
        }
        if steps == 0 { 0.0 } else { sum / steps as f32 }
    }
}
