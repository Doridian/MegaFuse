//! MegaFuse, for now headless: runs the translated game on a ROM with a
//! pad script (tools/mdtrace's format) and writes RAM dumps and
//! screenshots, so it can be compared with the original running in
//! Genesis Plus GX (tools/port_check.py).
//!
//!   megafuse ROM [--input FILE] [--frames N] [--every N]
//!                [--ram PREFIX] [--shot PREFIX]

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
        if self.every > 0 && (f % self.every == 0 || f == self.frames) {
            if let Some(p) = &self.ram_prefix {
                let _ = std::fs::write(format!("{p}{f:06}.bin"), &ram[..]);
            }
            if let Some(p) = &self.shot_prefix {
                let _ = png::write(&format!("{p}{f:06}.png"), width, md::vdp::WIDTH, md::vdp::HEIGHT, picture);
            }
        }
        if f >= self.frames {
            std::process::exit(0);
        }
        self.apply(f, ram, pads);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut rom_path = None;
    let mut input = None;
    let mut h = Headless {
        script: Script { pads: BTreeMap::new(), pokes: BTreeMap::new() },
        frames: 3600,
        every: 600,
        ram_prefix: None,
        shot_prefix: None,
        frame: 0,
        held: [0; 4],
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
            _ => rom_path = Some(a.to_string()),
        }
        i += 1;
    }
    let Some(rom_path) = rom_path else {
        eprintln!("usage: megafuse ROM [--input FILE] [--frames N] [--every N] [--ram PREFIX] [--shot PREFIX]");
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
    let md = Md::new(Box::new(h));
    let mut game = game::Game::new(rom, Box::new(md));
    game.run();
}
