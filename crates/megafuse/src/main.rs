//! MegaFuse: the translated game on a ROM.  `megafuse ROM` plays it in a
//! window (window.rs: arrows, Z X C, Enter; F11 full screen).  With
//! --frames it runs headless instead: a pad script (tools/mdtrace's
//! format), RAM dumps, screenshots, hashes, timelines, so it can be
//! compared with the original running in Genesis Plus GX
//! (tools/port_check.py, tools/timeline_check.py).
//!
//!   megafuse ROM [--host ROOM [--players N] [--delay D] | --join ROOM]
//!                [--signal ws://HOST:3536]
//!   megafuse ROM --loopback N --input FILE --frames N
//!   megafuse ROM [--input FILE] [--frames N] [--every N]
//!                [--ram PREFIX] [--shot PREFIX] [--wav FILE]
//!                [--timeline FILE] [--ztimeline FILE] [--timeline-at F]
//!                [--hashes FILE] [--vram PREFIX] [--restore-check]
//!
//! --restore-check: every so many steps a snapshot, some steps run, the
//! snapshot restored and the same steps run again: the clock and RAM
//! after each have to be the same (Game::save, Game::restore).

#![cfg_attr(target_arch = "wasm32", allow(dead_code, unused_imports))]

#[cfg(not(target_arch = "wasm32"))]
mod netplay;
mod png;
#[cfg(target_arch = "wasm32")]
mod web;
mod window;

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
    /// VRAM, VSRAM and the registers, as tools/mdtrace --vram writes them.
    vram_prefix: Option<String>,
    /// Each frame's CRC-32s of work RAM and the picture (--hashes).
    hashes: Option<std::io::BufWriter<std::fs::File>>,
    /// The script wants a Team Player, a 4 Way Play.
    teamplayer: bool,
    wayplay: bool,
    /// The sound so far (16-bit stereo), for --wav.
    wav: Option<(String, Vec<i16>)>,
}

impl Headless {
    fn apply(&mut self, frame: u64, ram: &mut [u8; 0x10000], pads: &mut [u16; 4]) {
        // each pad as the script's last change by this frame has it (a
        // function of the frame: a restored run gets the same pads)
        let mut held = [0u16; 4];
        let mut set = [false; 4];
        for (_, changes) in self.script.pads.range(..=frame).rev() {
            for &(p, m) in changes.iter().rev() {
                if p < 4 && !set[p] {
                    held[p] = m;
                    set[p] = true;
                }
            }
            if set.iter().all(|&s| s) {
                break;
            }
        }
        *pads = held;
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
    fn frame(&mut self, frame: u64, picture: &[u32], width: usize, height: usize, ram: &mut [u8; 0x10000], pads: &mut [u16; 4]) {
        // the frame that just ended is self.frame + 1 (mdtrace counts
        // frames run)
        self.frame = frame;
        let f = frame;
        if let Some(out) = &mut self.hashes {
            // as tools/mdtrace --hashes: RAM to the stack, the picture's RGB
            use std::io::Write as _;
            let mut rgb = Vec::with_capacity(width * height * 3);
            for y in 0..height {
                for &p in &picture[y * md::vdp::WIDTH..y * md::vdp::WIDTH + width] {
                    rgb.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, p as u8]);
                }
            }
            let _ = writeln!(out, "{} {:08X} {:08X}", f - 1, png::crc32(&ram[..0xFE00]), png::crc32(&rgb));
            let _ = out.flush();
        }
        if self.every > 0 && (f.is_multiple_of(self.every) || f == self.frames) {
            if let Some(p) = &self.ram_prefix {
                let _ = std::fs::write(format!("{p}{f:06}.bin"), &ram[..]);
            }
            if let Some(p) = &self.shot_prefix {
                let _ = png::write(&format!("{p}{f:06}.png"), width, md::vdp::WIDTH, height, picture);
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

    fn vdp(&mut self, frame: u64, vdp: &md::vdp::Vdp) {
        let f = frame;
        if let Some(p) = &self.vram_prefix
            && self.every > 0
            && (f.is_multiple_of(self.every) || f == self.frames)
        {
            let _ = std::fs::write(format!("{p}{f:06}.vram"), &vdp.vram[..]);
            let vs: Vec<u8> = vdp.vsram.iter().flat_map(|w| w.to_be_bytes()).chain(std::iter::repeat(0)).take(0x80).collect();
            let _ = std::fs::write(format!("{p}{f:06}.vsram"), vs);
            let _ = std::fs::write(format!("{p}{f:06}.sat"), &vdp.sat[..]);
            let mut regs = vdp.regs.to_vec();
            regs.resize(0x20, 0);
            let _ = std::fs::write(format!("{p}{f:06}.regs"), regs);
        }
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
    fn frame(&mut self, _: u64, _: &[u32], _: usize, _: usize, _: &mut [u8; 0x10000], _: &mut [u16; 4]) {}
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

/// In the browser: the page calls `start` with the ROM (web.rs).
#[cfg(target_arch = "wasm32")]
fn main() {
    web::main();
}

#[cfg(not(target_arch = "wasm32"))]
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
    let mut ztimeline = None;
    let mut headless = false;
    let mut restore_check = false;
    let mut join: Option<netplay::Join> = None;
    let mut players = 2;
    let mut delay = netplay::DELAY;
    let mut signal = "ws://localhost:3536".to_owned();
    let mut loopback = None;
    let mut h = Headless {
        script: Script { pads: BTreeMap::new(), pokes: BTreeMap::new() },
        frames: 3600,
        every: 600,
        ram_prefix: None,
        shot_prefix: None,
        frame: 0,
        wav: None,
        teamplayer: false,
        wayplay: false,
        hashes: None,
        vram_prefix: None,
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
            "--frames" => {
                h.frames = next(&mut i).parse().unwrap_or(3600);
                headless = true;
            }
            "--every" => h.every = next(&mut i).parse().unwrap_or(600),
            "--ram" => h.ram_prefix = Some(next(&mut i)),
            "--shot" => h.shot_prefix = Some(next(&mut i)),
            "--wav" => h.wav = Some((next(&mut i), Vec::new())),
            "--timeline" => timeline = Some(next(&mut i)),
            "--timeline-at" => timeline_at = next(&mut i).parse().ok(),
            "--ztimeline" => ztimeline = Some(next(&mut i)),
            "--vram" => h.vram_prefix = Some(next(&mut i)),
            "--restore-check" => restore_check = true,
            "--host" | "--join" => {
                let host = a == "--host";
                join = Some(netplay::Join { room: next(&mut i), host, players: 2, delay: 0, signal: String::new() });
            }
            "--players" => players = next(&mut i).parse().unwrap_or(2),
            "--delay" => delay = next(&mut i).parse().unwrap_or(netplay::DELAY),
            "--signal" => signal = next(&mut i),
            "--loopback" => loopback = next(&mut i).parse::<usize>().ok(),
            "--hashes" => {
                let p = next(&mut i);
                let f = std::fs::File::create(&p).unwrap_or_else(|e| {
                    eprintln!("{p}: {e}");
                    std::process::exit(1)
                });
                h.hashes = Some(std::io::BufWriter::new(f));
            }
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
    if let Some(j) = &mut join {
        j.players = players;
        j.delay = delay;
        j.signal.clone_from(&signal);
    }
    if let Some(n) = loopback {
        let text = input.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
        netplay::loopback(rom, parse_script(&text).pads, n.clamp(2, 4), h.frames);
    }
    if !headless {
        window::play(rom, join);
    }
    if let Some(j) = &join {
        let text = input.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
        netplay::headless(rom, j, parse_script(&text).pads, h.frames);
    }
    if let Some(p) = input {
        let text = std::fs::read_to_string(&p).unwrap_or_default();
        h.script = parse_script(&text);
        // tools/mdtrace's marks: a Team Player on port A, a 4 Way Play
        h.teamplayer = text.lines().any(|l| l.trim() == "# teamplayer");
        h.wayplay = text.lines().any(|l| l.trim() == "# wayplay");
    }
    let mut ram = [0u8; 0x10000];
    let mut pads = [0u16; 4];
    h.apply(0, &mut ram, &mut pads);
    let (teamplayer, wayplay) = (h.teamplayer, h.wayplay);
    let frames_wanted = h.frames;
    if restore_check {
        // the frontend must not end the run (the check does)
        h.frames = u64::MAX;
    }
    let mut md = Md::new(rom.clone(), Box::new(h));
    if teamplayer {
        md.plug_teamplayer();
    }
    if wayplay {
        md.plug_wayplay();
    }
    if let Some(p) = ztimeline {
        let f = std::fs::File::create(&p).unwrap_or_else(|e| {
            eprintln!("{p}: {e}");
            std::process::exit(1)
        });
        md.sound.timeline = Some(std::sync::Arc::new(std::sync::Mutex::new(game::Timeline::new(
            Box::new(std::io::BufWriter::new(f)),
            timeline_at,
        ))));
    }
    let h_frames = frames_wanted;
    let mut game = game::Game::new(rom, Box::new(md));
    if let Some(p) = timeline {
        let f = std::fs::File::create(&p).unwrap_or_else(|e| {
            eprintln!("{p}: {e}");
            std::process::exit(1)
        });
        game.timeline = Some(game::Timeline::new(Box::new(std::io::BufWriter::new(f)), timeline_at));
    }
    if restore_check {
        let frames = h_frames;
        check_restore(&mut game, frames);
    }
    game.run();
}

/// Snapshots, taken back: every 37th step, 25 steps run, restored and
/// run again, the clock and RAM after each compared.
fn check_restore(game: &mut game::Game, frames: u64) -> ! {
    game.boot();
    let mut checks = 0;
    let mut step = 0u64;
    while game.clock / game::FRAME < frames.saturating_sub(40) {
        step += 1;
        if !step.is_multiple_of(37) {
            game.step();
            continue;
        }
        let snap = game.save();
        let run = |g: &mut game::Game| -> Vec<(u64, u32)> {
            (0..25)
                .map(|_| {
                    g.step();
                    (g.clock, png::crc32(&g.ram[..]))
                })
                .collect()
        };
        let first = run(game);
        game.restore(&snap);
        let again = run(game);
        if let Some(k) = (0..first.len()).find(|&k| first[k] != again[k]) {
            eprintln!(
                "restore-check: step {} + {k}: clock {} / {}, RAM {:08X} / {:08X}",
                step, first[k].0, again[k].0, first[k].1, again[k].1
            );
            std::process::exit(1);
        }
        checks += 1;
    }
    println!("restore-check: {checks} snapshots restored, the same");
    std::process::exit(0)
}
