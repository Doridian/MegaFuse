//! Network play (docs/NETPLAY.md, crates/net): in a window, as the host
//! of a room or a player joining one, through the signalling server
//! (`megafuse-server`); and `--loopback`, peers in one process over the
//! in-memory transport, for testing that they stay the same.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, PoisonError};

use md::Frontend;
use net::{Session, Transport};

/// The input delay (frames) unless asked otherwise.
pub const DELAY: u64 = 3;

/// A frame's pads from the session: this peer's pad goes out, the four
/// come back; the state's hash goes along now and then.
pub type Pads = Box<dyn FnMut(u64, u16, &[u8; 0x10000]) -> Result<[u16; 4], String> + Send>;

/// The session as a [`Pads`].
pub fn pads<T: Transport + Send + 'static>(mut s: Session<T>) -> Pads {
    Box::new(move |frame, local, ram| {
        let hash = frame.is_multiple_of(net::session::HASH_EVERY).then(|| crate::png::crc32(&ram[..]));
        s.pads(frame, local, hash).map_err(|e| e.to_string())
    })
}

/// How a window's game meets the others.
pub struct Join {
    pub room: String,
    pub host: bool,
    pub players: usize,
    pub delay: u64,
    pub signal: String,
}

/// Connects (waiting for the players): the pads, this peer's seat and
/// how many play.
pub fn connect(j: &Join, rom: &[u8]) -> Result<(Pads, usize, usize), String> {
    let crc = crate::png::crc32(rom);
    let t = net::matchbox::MatchboxTransport::new(&j.signal, net::matchbox::IceServers::default());
    eprintln!("{} room {} on {}...", if j.host { "hosting" } else { "joining" }, j.room, j.signal);
    let s = if j.host {
        Session::host(t, &j.room, j.players, j.delay, crc)
    } else {
        Session::join(t, &j.room, crc)
    }
    .map_err(|e| e.to_string())?;
    let (slot, players) = s.seat();
    eprintln!("player {} of {players}", slot + 1);
    Ok((pads(s), slot, players))
}

/// A headless peer of the loopback test: its pad is the script's for its
/// seat; it ends at `frames`, with its RAM's hash.
struct Peer {
    pads: Pads,
    slot: usize,
    script: Arc<BTreeMap<u64, Vec<(usize, u16)>>>,
    frames: u64,
    done: Arc<Mutex<Option<u32>>>,
}

impl Frontend for Peer {
    fn frame(&mut self, frame: u64, _: &[u32], _: usize, _: usize, ram: &mut [u8; 0x10000], pads: &mut [u16; 4]) {
        if frame >= self.frames {
            let mut d = self.done.lock().unwrap_or_else(PoisonError::into_inner);
            if d.is_none() {
                *d = Some(crate::png::crc32(&ram[..]));
            }
            *pads = [0; 4];
            return;
        }
        // this seat's pad, as the script has it by this frame
        let local = self
            .script
            .range(..=frame)
            .rev()
            .flat_map(|(_, c)| c.iter().rev())
            .find(|&&(p, _)| p == self.slot)
            .map_or(0, |&(_, m)| m);
        match (self.pads)(frame, local, ram) {
            Ok(p) => *pads = p,
            Err(e) => {
                eprintln!("peer {}: frame {frame}: {e}", self.slot + 1);
                std::process::exit(1);
            }
        }
    }
}

/// A headless peer of a network game (`--host`/`--join` with --frames):
/// its seat's pads from the script; at the end, its RAM's hash.
pub fn headless(rom: Vec<u8>, j: &Join, script: BTreeMap<u64, Vec<(usize, u16)>>, frames: u64) -> ! {
    let (pads, slot, players) = connect(j, &rom).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1)
    });
    let done = Arc::new(Mutex::new(None));
    let peer = Peer { pads, slot, script: Arc::new(script), frames, done: Arc::clone(&done) };
    let mut md = md::Md::new(rom.clone(), Box::new(peer));
    if players > 2 {
        md.plug_teamplayer();
    }
    let mut game = game::Game::new(rom, Box::new(md));
    game.boot();
    while done.lock().unwrap_or_else(PoisonError::into_inner).is_none() {
        game.step();
    }
    let h = done.lock().unwrap_or_else(PoisonError::into_inner).unwrap_or(0);
    println!("player {}: {frames} frames: RAM {h:08X}", slot + 1);
    // the others may still want this frame's pads
    std::thread::sleep(std::time::Duration::from_millis(500));
    std::process::exit(0)
}

/// `players` peers in one process, each in a thread of its own, playing
/// the script's pads 1 to `players` over the in-memory transport for
/// `frames` frames: their states have to agree all along (the session
/// compares their hashes) and at the end.
pub fn loopback(rom: Vec<u8>, script: BTreeMap<u64, Vec<(usize, u16)>>, players: usize, frames: u64) -> ! {
    let hub = net::memory::Hub::new();
    let script = Arc::new(script);
    let crc = crate::png::crc32(&rom);
    let mut runs = Vec::new();
    for k in 0..players {
        let t = hub.transport();
        let rom = rom.clone();
        let script = Arc::clone(&script);
        let done = Arc::new(Mutex::new(None));
        let out = Arc::clone(&done);
        runs.push((
            std::thread::spawn(move || {
                let s = if k == 0 {
                    Session::host(t, "loopback", players, DELAY, crc)
                } else {
                    // let the host be there first
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    Session::join(t, "loopback", crc)
                };
                let s = s.unwrap_or_else(|e| {
                    eprintln!("peer {}: {e}", k + 1);
                    std::process::exit(1)
                });
                let slot = s.seat().0;
                let peer = Peer { pads: pads(s), slot, script, frames, done: Arc::clone(&done) };
                let mut md = md::Md::new(rom.clone(), Box::new(peer));
                if players > 2 {
                    md.plug_teamplayer();
                }
                let mut game = game::Game::new(rom, Box::new(md));
                game.boot();
                while done.lock().unwrap_or_else(PoisonError::into_inner).is_none() {
                    game.step();
                }
            }),
            out,
        ));
    }
    let mut hashes = Vec::new();
    for (run, out) in runs {
        let _ = run.join();
        hashes.push(*out.lock().unwrap_or_else(PoisonError::into_inner));
    }
    let same = hashes.iter().all(|h| h.is_some() && *h == hashes[0]);
    println!(
        "loopback: {players} peers, {frames} frames: RAM {}",
        hashes.iter().map(|h| h.map_or("-".to_owned(), |x| format!("{x:08X}"))).collect::<Vec<_>>().join(" ")
    );
    std::process::exit(if same { 0 } else { 1 })
}
