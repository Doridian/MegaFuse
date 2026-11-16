//! Network play (docs/NETPLAY.md, crates/net): in a window, as the host
//! of a room or a player joining one, through the signalling server
//! (`megafuse-server`); and `--loopback`, peers in one process over the
//! in-memory transport, for testing that they stay the same.
//!
//! The session never waits, and neither does the game: [`Driver`] runs
//! it a frame at a time on the pads the session has, guessed where they
//! aren't in yet, keeps a snapshot after every step, and when a guess
//! turns out wrong goes back to the snapshot before it and runs the
//! frames again (silently: the window has shown them). It doesn't run
//! more than [`AHEAD`] frames past the ones whose pads are all in. A
//! peer coming into a game under way runs it from the start on the pads
//! so far, silently, and then plays.

#[cfg(not(target_arch = "wasm32"))]
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use md::Frontend;
use net::session::{HASH_EVERY, Seat};
use net::{Session, Transport};
use platform::{Duration, Instant};

/// The input delay (frames) unless asked otherwise: a pad goes out this
/// far ahead, so that it is mostly there in time and a guess is rare.
pub const DELAY: u64 = 2;
/// The frames at the start with no pads: the boot (some 155 frames,
/// which [`game::Game::boot`] runs at once) and a little more.
pub const QUIET: u64 = 240;
/// How far the game may run past the frames whose pads are all in.
pub const AHEAD: u64 = 8;

/// The session over whichever transport.
pub type Link = Session<Box<dyn Transport + Send>>;

/// What the game's frontend and the driver share.
pub struct Net {
    pub session: Link,
    /// The frames ended so far (the frontend's count).
    label: u64,
    /// This peer's pad now.
    pub local: u16,
    /// Frames run again, or a game caught up with: not shown or heard.
    pub silent: bool,
}

impl Net {
    /// Whether the state at the end of frame `label` is final: every pad
    /// before it is in (and right: a wrong guess is rolled back before
    /// the game runs on, and nothing comes in during a step).
    #[must_use]
    pub fn finished(&self, label: u64) -> bool {
        label <= self.session.confirmed() + 1
    }

    /// From the frontend at the end of frame `label`: the pads for the
    /// next one. This peer's pad goes out `delay` frames ahead; the
    /// state's hash, if final, goes for the check.
    pub fn pads(&mut self, label: u64, ram: &[u8; 0x10000]) -> [u16; 4] {
        self.label = label;
        if self.finished(label) && label.is_multiple_of(HASH_EVERY) {
            self.session.hash(label, crate::png::crc32(&ram[..]));
        }
        if let Some(seat) = self.session.seat() {
            self.session.local(label + seat.delay, self.local);
        }
        self.session.pads(label).0
    }
}

/// The game and the session together.
pub struct Driver {
    pub net: Arc<Mutex<Net>>,
    game: Option<game::Game>,
    /// The game after each step, by the frames ended then.
    snaps: VecDeque<(u64, game::Snapshot)>,
    /// A game for so many players, its frontend sharing [`Driver::net`].
    make: Make,
    /// Rollbacks so far, and the frames they ran again.
    pub rollbacks: (u64, u64),
}

/// How a [`Driver`] makes its game.
pub type Make = Box<dyn FnMut(usize, Arc<Mutex<Net>>) -> game::Game + Send>;

/// What a [`Driver::tick`] did.
pub enum Tick {
    /// The game ran a step (its frames went to the frontend).
    Stepped,
    /// It didn't: the pads aren't in (or it is catching up).
    Waiting,
    /// The session ended.
    Ended(String),
}

impl Driver {
    #[must_use]
    pub fn new(session: Link, make: Make) -> Self {
        let net = Net {
            session,
            label: 0,
            local: 0,
            silent: false,
        };
        Driver {
            net: Arc::new(Mutex::new(net)),
            game: None,
            snaps: VecDeque::new(),
            make,
            rollbacks: (0, 0),
        }
    }

    fn net(&self) -> MutexGuard<'_, Net> {
        self.net.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// This peer's seat, once it has one.
    #[must_use]
    pub fn seat(&self) -> Option<Seat> {
        self.net().session.seat()
    }

    /// What the player should know while the game isn't simply running
    /// (empty when it is).
    #[must_use]
    pub fn status(&self) -> String {
        let n = self.net();
        let (label, confirmed) = (n.label, n.session.confirmed());
        match n.session.seat() {
            None if n.session.peers() == 0 => "connecting to the room...".to_owned(),
            None => "waiting for the host to let this player in...".to_owned(),
            Some(seat) if seat.from == 0 => {
                format!("catching up with the game: frame {label} of {confirmed}...")
            }
            Some(_) if label >= confirmed + AHEAD => "waiting for the other players...".to_owned(),
            Some(_) => String::new(),
        }
    }

    /// The frames ended so far.
    #[must_use]
    pub fn frames(&self) -> u64 {
        self.net().label
    }

    fn step(&mut self) {
        if let Some(g) = &mut self.game {
            g.step();
            let label = self
                .net
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .label;
            self.snaps.push_back((label, g.save()));
        }
    }

    /// One turn of the loop: what came from the network, a rollback if a
    /// guess was wrong, then a step if the game may run on. `budget`: how
    /// long a peer coming in may spend catching up in this turn.
    pub fn tick(&mut self, budget: Duration) -> Tick {
        let (seat, rollback) = {
            let mut n = self.net();
            if let Err(e) = n.session.poll() {
                return Tick::Ended(e.to_string());
            }
            // this peer's pad out as far as it goes, stepping or not (a
            // seat that has just become this peer's may start behind it)
            if let Some(seat) = n.session.seat() {
                let (label, local) = (n.label, n.local);
                n.session.local(label + seat.delay, local);
            }
            (n.session.seat(), n.session.take_rollback())
        };
        let Some(seat) = seat else {
            return Tick::Waiting;
        };
        if self.game.is_none() {
            let mut g = (self.make)(seat.players, Arc::clone(&self.net));
            self.net().silent = seat.from == 0;
            g.boot();
            self.net().silent = false;
            let label = self.net().label;
            assert!(
                label < QUIET,
                "the boot ran past the quiet frames ({label})"
            );
            self.snaps.push_back((label, g.save()));
            self.game = Some(g);
            return Tick::Stepped;
        }
        if let Some(r) = rollback {
            self.rollback(r);
        }
        let (label, confirmed) = {
            let n = self.net();
            (n.label, n.session.confirmed())
        };
        // a peer coming in: the game so far, silently
        if seat.from == 0 {
            if label + AHEAD / 2 < confirmed {
                self.net().silent = true;
                let start = Instant::now();
                while self.frames() + 1 < confirmed && start.elapsed() < budget {
                    self.step();
                }
                self.net().silent = false;
                self.prune();
                return Tick::Waiting;
            }
            self.net().session.ready();
        }
        if label >= confirmed + AHEAD {
            return Tick::Waiting;
        }
        self.step();
        self.prune();
        Tick::Stepped
    }

    /// A guess for frame `r` was wrong: back to before it, and the frames
    /// since again, silently.
    fn rollback(&mut self, r: u64) {
        let now = self.frames();
        if r > now {
            // (the frame hasn't run yet)
            return;
        }
        let Some(k) = self.snaps.iter().rposition(|&(f, _)| f < r) else {
            panic!("no snapshot before frame {r}");
        };
        self.snaps.truncate(k + 1);
        let (f, snap) = &self.snaps[k];
        let f = *f;
        if let Some(g) = &mut self.game {
            g.restore(snap);
        }
        {
            let mut n = self.net();
            n.label = f;
            n.silent = true;
        }
        self.rollbacks.0 += 1;
        self.rollbacks.1 += now - f;
        while self.frames() < now {
            self.step();
        }
        self.net().silent = false;
    }

    /// The snapshots no rollback can go back to: all but the last one
    /// before the first frame whose pads aren't all in.
    fn prune(&mut self) {
        let confirmed = self.net().session.confirmed();
        while self.snaps.len() > 1 && self.snaps[1].0 <= confirmed {
            self.snaps.pop_front();
        }
    }
}

/// How a window's game meets the others.
pub struct Join {
    pub room: String,
    pub host: bool,
    pub players: usize,
    pub delay: u64,
    pub signal: String,
    /// STUN and TURN servers (none: the peers have to reach each other
    /// directly, as on a LAN).
    pub ice: net::matchbox::IceServers,
}

/// The session for `j`: the room hosted or joined (it doesn't wait).
pub fn connect(j: &Join, rom: &[u8]) -> Result<Link, String> {
    let crc = crate::png::crc32(rom);
    j.ice.check()?;
    let t: Box<dyn Transport + Send> = Box::new(net::matchbox::MatchboxTransport::new(
        &j.signal,
        j.ice.clone(),
    ));
    eprintln!(
        "{} room {} on {}",
        if j.host { "hosting" } else { "joining" },
        j.room,
        j.signal
    );
    if j.host {
        Session::host(t, &j.room, j.players, j.delay, QUIET, crc)
    } else {
        Session::join(t, &j.room, crc)
    }
    .map_err(|e| e.to_string())
}

/// A headless peer: its pad is the script's for its seat; its RAM's
/// hash at frame `frames` (the last run of it: a rollback may run it
/// again).
#[cfg(not(target_arch = "wasm32"))]
struct Peer {
    net: Arc<Mutex<Net>>,
    script: Arc<BTreeMap<u64, Vec<(usize, u16)>>>,
    frames: u64,
    at: Arc<Mutex<Option<u32>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Frontend for Peer {
    fn frame(
        &mut self,
        frame: u64,
        picture: &[u32],
        width: usize,
        height: usize,
        ram: &mut [u8; 0x10000],
        pads: &mut [u16; 4],
    ) {
        let mut n = self.net.lock().unwrap_or_else(PoisonError::into_inner);
        let slot = n.session.seat().map_or(0, |s| s.slot);
        // MEGAFUSE_NETSHOT=N: each peer's picture at frame N
        if std::env::var("MEGAFUSE_NETSHOT")
            .ok()
            .and_then(|v| v.parse().ok())
            == Some(frame)
            && !n.silent
        {
            let _ = crate::png::write(
                &format!("netshot-{}-{frame}.png", slot + 1),
                width,
                md::vdp::WIDTH,
                height,
                picture,
            );
        }
        if frame == self.frames {
            *self.at.lock().unwrap_or_else(PoisonError::into_inner) =
                Some(crate::png::crc32(&ram[..]));
        }
        // this seat's pad, as the script has it by this frame
        n.local = self
            .script
            .range(..=frame)
            .rev()
            .flat_map(|(_, c)| c.iter().rev())
            .find(|&&(p, _)| p == slot)
            .map_or(0, |&(_, m)| m);
        *pads = n.pads(frame, ram);
    }
}

/// A headless peer's driver: a game whose frontend is a [`Peer`].
#[cfg(not(target_arch = "wasm32"))]
fn peer(
    rom: Vec<u8>,
    session: Link,
    script: Arc<BTreeMap<u64, Vec<(usize, u16)>>>,
    frames: u64,
) -> (Driver, Arc<Mutex<Option<u32>>>) {
    let done = Arc::new(Mutex::new(None));
    let d2 = Arc::clone(&done);
    let driver = Driver::new(
        session,
        Box::new(move |players, net| {
            let p = Peer {
                net,
                script: Arc::clone(&script),
                frames,
                at: Arc::clone(&d2),
            };
            let mut md = md::Md::new(rom.clone(), Box::new(p));
            if players > 2 {
                md.plug_teamplayer();
            }
            game::Game::new(rom.clone(), Box::new(md))
        }),
    );
    (driver, done)
}

/// A headless peer's hash at its last frame, once final.
#[cfg(not(target_arch = "wasm32"))]
fn result(driver: &Driver, at: &Mutex<Option<u32>>, frames: u64) -> Option<u32> {
    let at = *at.lock().unwrap_or_else(PoisonError::into_inner);
    let n = driver.net();
    (n.label >= frames && n.finished(frames))
        .then_some(at)
        .flatten()
}

/// Runs a headless driver until `stop` says so (or the session ends:
/// why); `pace`: from when, how many times the console's speed (a game
/// in a window runs at 1).
#[cfg(not(target_arch = "wasm32"))]
fn run(
    driver: &mut Driver,
    who: &str,
    pace: Option<(Instant, u32)>,
    mut stop: impl FnMut(&Driver) -> bool,
) -> Result<(), String> {
    let debug = std::env::var_os("MEGAFUSE_NETDEBUG").is_some();
    let mut shown = Instant::now();
    while !stop(driver) {
        if debug && shown.elapsed() > Duration::from_secs(1) {
            shown = Instant::now();
            let n = driver.net();
            eprintln!(
                "{who}: frame {}, confirmed {} (missing {:?}), seat {:?}",
                n.label,
                n.session.confirmed(),
                n.session.missing(),
                n.session.seat()
            );
        }
        if let Some((start, speed)) = pace {
            let due = start.elapsed().as_secs_f64() * 60.0 * f64::from(speed);
            let seated = driver.seat().is_some_and(|s| s.from != 0);
            if seated && driver.frames() as f64 > due {
                std::thread::sleep(std::time::Duration::from_micros(500));
                continue;
            }
        }
        match driver.tick(Duration::from_millis(20)) {
            Tick::Stepped => {}
            Tick::Waiting => std::thread::sleep(std::time::Duration::from_micros(500)),
            Tick::Ended(e) => return Err(format!("{who}: frame {}: {e}", driver.frames())),
        }
    }
    Ok(())
}

/// What a peer that has to go on can't do without.
#[cfg(not(target_arch = "wasm32"))]
fn or_exit(r: Result<(), String>) {
    if let Err(e) = r {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

/// A headless peer of a network game (`--host`/`--join` with --frames):
/// its seat's pads from the script; at the end, its RAM's hash.
#[cfg(not(target_arch = "wasm32"))]
pub fn headless(
    rom: Vec<u8>,
    j: &Join,
    script: BTreeMap<u64, Vec<(usize, u16)>>,
    frames: u64,
) -> ! {
    let session = connect(j, &rom).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1)
    });
    let (mut driver, at) = peer(rom, session, Arc::new(script), frames);
    let pace = Some((Instant::now(), 1));
    or_exit(run(&mut driver, "peer", pace, |d| {
        result(d, &at, frames).is_some()
    }));
    let slot = driver.seat().map_or(0, |s| s.slot);
    let h = result(&driver, &at, frames).unwrap_or(0);
    println!("player {}: {frames} frames: RAM {h:08X}", slot + 1);
    // the others may still want this peer's pads (and the host's relays)
    let until = Instant::now() + Duration::from_secs(2);
    // (the others may be done and gone first)
    let _ = run(&mut driver, "peer", pace, |_| Instant::now() > until);
    std::process::exit(0)
}

/// `players` peers in one process, each in a thread of its own, playing
/// the script's pads 1 to `players` over the in-memory transport (each
/// message `latency` late) for `frames` frames: their states have to
/// agree all along (the session compares their hashes) and at the end.
/// `late`: the last peer comes in that many milliseconds after the
/// start, into the game under way; `speed`: the peers keep to that many
/// times the console's pace (a peer coming in catches up faster).
#[cfg(not(target_arch = "wasm32"))]
pub fn loopback(
    rom: Vec<u8>,
    script: BTreeMap<u64, Vec<(usize, u16)>>,
    players: usize,
    frames: u64,
    latency: u64,
    late: u64,
    speed: u32,
) -> ! {
    let start = Instant::now();
    let hub = net::memory::Hub::new().with_latency(Duration::from_millis(latency));
    let script = Arc::new(script);
    let crc = crate::png::crc32(&rom);
    let finished = Arc::new(Mutex::new(0usize));
    let mut runs = Vec::new();
    for k in 0..players {
        let t: Box<dyn Transport + Send> = Box::new(hub.transport());
        let rom = rom.clone();
        let script = Arc::clone(&script);
        let finished = Arc::clone(&finished);
        let out = Arc::new(Mutex::new(None));
        let out2 = Arc::clone(&out);
        runs.push((
            std::thread::spawn(move || {
                let s = if k == 0 {
                    Session::host(t, "loopback", players, DELAY, QUIET, crc)
                } else {
                    // let the host be there first; the last one later still
                    let wait = if k == players - 1 && late > 0 {
                        late
                    } else {
                        50
                    };
                    std::thread::sleep(std::time::Duration::from_millis(wait));
                    Session::join(t, "loopback", crc)
                };
                let s = s.unwrap_or_else(|e| {
                    eprintln!("peer {}: {e}", k + 1);
                    std::process::exit(1)
                });
                let (mut driver, at) = peer(rom, s, script, frames);
                let who = format!("peer {}", k + 1);
                let pace = (speed > 0).then_some((start, speed));
                or_exit(run(&mut driver, &who, pace, |d| {
                    result(d, &at, frames).is_some()
                }));
                *out2.lock().unwrap_or_else(PoisonError::into_inner) = result(&driver, &at, frames);
                let seat = driver.seat().map(|s| (s.slot, s.from));
                eprintln!(
                    "{who}: seat {seat:?}, {} rollbacks, {} frames run again",
                    driver.rollbacks.0, driver.rollbacks.1
                );
                *finished.lock().unwrap_or_else(PoisonError::into_inner) += 1;
                // on until every peer is done (the host passes the pads on)
                or_exit(run(&mut driver, &who, pace, |_| {
                    *finished.lock().unwrap_or_else(PoisonError::into_inner) == players
                }));
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
        hashes
            .iter()
            .map(|h| h.map_or("-".to_owned(), |x| format!("{x:08X}")))
            .collect::<Vec<_>>()
            .join(" ")
    );
    std::process::exit(if same { 0 } else { 1 })
}
