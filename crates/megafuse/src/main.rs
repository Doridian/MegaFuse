//! MegaFuse, for now headless: runs the translated game on a ROM with a
//! pad script (tools/mdtrace's format) and writes RAM dumps and
//! screenshots, so it can be compared with the original running in
//! Genesis Plus GX (tools/port_check.py).
//!
//!   megafuse ROM [--input FILE] [--frames N] [--every N]
//!                [--ram PREFIX] [--shot PREFIX] [--wav FILE]
//!                [--timeline FILE [--timeline-at F]]

mod png;

use md::{Frontend, Md, button};
use std::collections::BTreeMap;

struct Script {
    /// frame -> [(pad, buttons)] from then on
    pads: BTreeMap<u64, Vec<(usize, u16)>>,
    /// frame -> [(address, value, bytes)] written before it runs
    pokes: BTreeMap<u64, Vec<(u32, u32, usize)>>,
}

fn parse_script(text: &str) -> Script {
    let mut s = Script { pads: BTreeMap::new(), pokes: BTreeMap::new() };
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let mut it = line.split_whitespace();
        let (Some(frame), Some(what)) = (it.next(), it.next()) else { continue };
        let Ok(frame) = frame.parse::<u64>() else { continue };
        if let Some(addr) = what.strip_prefix('=') {
            let value = it.next().unwrap_or("0");
            let a = u32::from_str_radix(addr, 16).unwrap_or(0);
            let v = u32::from_str_radix(value, 16).unwrap_or(0);
            s.pokes.entry(frame).or_default().push((a, v, value.len() / 2));
            continue;
        }
        let (pad, buttons) = match what.split_once(':') {
            Some((p, b)) => (p.parse::<usize>().unwrap_or(1), b),
            None => (1, what),
        };
        let mut m = 0;
        for ch in buttons.chars() {
            m |= match ch {
                'U' => button::UP,
                'D' => button::DOWN,
                'L' => button::LEFT,
                'R' => button::RIGHT,
                'A' => button::A,
                'B' => button::B,
                'C' => button::C,
                'S' => button::START,
                _ => 0,
            };
        }
        s.pads.entry(frame).or_default().push((pad.saturating_sub(1), m));
    }
    s
}

struct Headless {
    script: Script,
    frames: u64,
    every: u64,
    ram_prefix: Option<String>,
    shot_prefix: Option<String>,
    frame: u64,
    held: [u16; 4],
    /// The sound so far (16-bit stereo), for --wav.
    wav: Option<(String, Vec<i16>)>,
}

impl Headless {
    fn apply(&mut self, frame: u64, ram: &mut [u8; 0x10000], pads: &mut [u16; 4]) {
        if let Some(changes) = self.script.pads.get(&frame) {
            for &(p, m) in changes {
                if p < 4 {
                    self.held[p] = m;
                }
            }
        }
        *pads = self.held;
        if let Some(pokes) = self.script.pokes.get(&frame) {
            for &(a, v, n) in pokes {
                for k in 0..n {
                    let byte = (v >> (8 * (n - 1 - k))) as u8;
                    ram[((a as usize) + k) & 0xFFFF] = byte;
                }
            }
        }
    }
}

impl Frontend for Headless {
    fn frame(&mut self, picture: &[u32], width: usize, ram: &mut [u8; 0x10000], pads: &mut [u16; 4]) {
        // the frame that just ended is self.frame + 1 (mdtrace counts
        // frames run)
        self.frame += 1;
        let f = self.frame;
        if self.every > 0 && (f.is_multiple_of(self.every) || f == self.frames) {
            if let Some(p) = &self.ram_prefix {
                let _ = std::fs::write(format!("{p}{f:06}.bin"), &ram[..]);
            }
            if let Some(p) = &self.shot_prefix {
                let _ = png::write(&format!("{p}{f:06}.png"), width, md::vdp::WIDTH, md::vdp::HEIGHT, picture);
            }
        }
        if f >= self.frames {
            if let Some((path, samples)) = &self.wav
                && let Err(e) = write_wav(path, samples)
            {
                eprintln!("{path}: {e}");
            }
            std::process::exit(0);
        }
        self.apply(f, ram, pads);
    }

    fn audio(&mut self, samples: &[f32]) {
        if let Some((_, out)) = &mut self.wav {
            out.extend(samples.iter().map(|&v| (v.clamp(-1.0, 1.0) * 32767.0) as i16));
        }
    }
}

/// A 16-bit stereo WAV at the YM2612's rate.
fn write_wav(path: &str, samples: &[i16]) -> std::io::Result<()> {
    let rate = md::sound::SAMPLE_RATE;
    let data = u32::try_from(samples.len() * 2).unwrap_or(u32::MAX);
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 4).to_le_bytes());
    out.extend_from_slice(&4u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for v in samples {
        out.extend_from_slice(&v.to_le_bytes());
    }
    std::fs::write(path, out)
}

struct Silent;

impl Frontend for Silent {
    fn frame(&mut self, _: &[u32], _: usize, _: &mut [u8; 0x10000], _: &mut [u16; 4]) {}
}

/// `megafuse --routine ROM ENTRY IN OUT`: one routine from a state of the
/// original's (tools/mdtrace --snap: D0-D7 A0-A7 PC SR as big-endian
/// longs, then work RAM), the result written the same way
/// (tools/routine_check.py compares it with the original's).
fn routine(args: &[String]) {
    let rom = std::fs::read(&args[2]).expect("the ROM");
    let entry = u32::from_str_radix(&args[3], 16).expect("ENTRY in hex");
    let snap = std::fs::read(&args[4]).expect("IN");
    let reg = |k: usize| u32::from_be_bytes([snap[4 * k], snap[4 * k + 1], snap[4 * k + 2], snap[4 * k + 3]]);
    let mut regs = game::Regs {
        d0: reg(0), d1: reg(1), d2: reg(2), d3: reg(3), d4: reg(4), d5: reg(5), d6: reg(6), d7: reg(7),
        a0: reg(8), a1: reg(9), a2: reg(10), a3: reg(11), a4: reg(12), a5: reg(13), a6: reg(14),
        ..game::Regs::default()
    };
    let sr = reg(17);
    regs.c = sr & 1 != 0;
    regs.v = sr & 2 != 0;
    regs.z = sr & 4 != 0;
    regs.n = sr & 8 != 0;
    regs.x = sr & 16 != 0;
    let mut game = game::Game::new(rom.clone(), Box::new(Md::new(rom, Box::new(Silent))));
    game.ram.copy_from_slice(&snap[72..72 + 0x10000]);
    game.mask_interrupts();
    if !game.call_by_address(entry, &mut regs) {
        eprintln!("{entry:06X}: not callable on its own");
        std::process::exit(3);
    }
    let mut out = Vec::new();
    for v in [regs.d0, regs.d1, regs.d2, regs.d3, regs.d4, regs.d5, regs.d6, regs.d7,
              regs.a0, regs.a1, regs.a2, regs.a3, regs.a4, regs.a5, regs.a6, 0, 0,
              u32::from(regs.c) | u32::from(regs.v) << 1 | u32::from(regs.z) << 2
                  | u32::from(regs.n) << 3 | u32::from(regs.x) << 4] {
        out.extend_from_slice(&v.to_be_bytes());
    }
    out.extend_from_slice(&game.ram[..]);
    std::fs::write(&args[5], out).expect("OUT");
    println!("{}", game::Game::routine_outputs(entry).join(" "));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--routine") {
        routine(&args);
        return;
    }
    let mut rom_path = None;
    let mut input = None;
    let mut timeline = None;
    let mut timeline_at = None;
    let mut h = Headless {
        script: Script { pads: BTreeMap::new(), pokes: BTreeMap::new() },
        frames: 3600,
        every: 600,
        ram_prefix: None,
        shot_prefix: None,
        frame: 0,
        held: [0; 4],
        wav: None,
    };
    let mut i = 1;
    while i < args.len() {
        let a = args[i].as_str();
        let next = |i: &mut usize| {
            *i += 1;
            args.get(*i).cloned().unwrap_or_default()
        };
        match a {
            "--input" => input = Some(next(&mut i)),
            "--frames" => h.frames = next(&mut i).parse().unwrap_or(3600),
            "--every" => h.every = next(&mut i).parse().unwrap_or(600),
            "--ram" => h.ram_prefix = Some(next(&mut i)),
            "--shot" => h.shot_prefix = Some(next(&mut i)),
            "--wav" => h.wav = Some((next(&mut i), Vec::new())),
            "--timeline" => timeline = Some(next(&mut i)),
            "--timeline-at" => timeline_at = next(&mut i).parse().ok(),
            _ => rom_path = Some(a.to_string()),
        }
        i += 1;
    }
    let Some(rom_path) = rom_path else {
        eprintln!("usage: megafuse ROM [--input FILE] [--frames N] [--every N] [--ram PREFIX] [--shot PREFIX] [--wav FILE]");
        std::process::exit(2);
    };
    let rom = std::fs::read(&rom_path).unwrap_or_else(|e| {
        eprintln!("{rom_path}: {e}");
        std::process::exit(1)
    });
    if let Some(p) = input {
        let text = std::fs::read_to_string(&p).unwrap_or_default();
        h.script = parse_script(&text);
    }
    let mut ram = [0u8; 0x10000];
    let mut pads = [0u16; 4];
    h.apply(0, &mut ram, &mut pads);
    let md = Md::new(rom.clone(), Box::new(h));
    let mut game = game::Game::new(rom, Box::new(md));
    if let Some(p) = timeline {
        let f = std::fs::File::create(&p).unwrap_or_else(|e| {
            eprintln!("{p}: {e}");
            std::process::exit(1)
        });
        game.timeline = Some(game::Timeline::new(Box::new(std::io::BufWriter::new(f)), timeline_at));
    }
    game.run();
}
