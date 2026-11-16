//! The session without a game: peers over the in-memory transport (with
//! a delay, so that pads come late and get guessed), ticked in turn on
//! one thread, each running a frame when it may (no more than `AHEAD`
//! past the frames whose pads are all in) on pads made up from the frame
//! and its seat. Every peer has to end with the same pads for every
//! frame, each player's own the ones it gave `delay` frames before.

use std::time::{Duration, Instant};

use net::memory::Hub;
use net::{Session, Transport};

const DELAY: u64 = 2;
const QUIET: u64 = 10;
const AHEAD: u64 = 8;
const ROM: u32 = 0xABCD;

/// What a player presses at a frame: held for a while, as a pad is.
fn pressed(slot: usize, frame: u64) -> u16 {
    (((frame / 7) * 7 + slot as u64 * 13) % 256) as u16
}

struct Peer {
    s: Session<Box<dyn Transport>>,
    /// Frames run.
    frame: u64,
    rollbacks: usize,
}

impl Peer {
    /// A turn: what came, then a frame if it may.
    fn tick(&mut self) {
        self.s.poll().expect("the session goes on");
        if let Some(r) = self.s.take_rollback() {
            assert!(r > self.s.confirmed().min(r - 1));
            if r <= self.frame {
                self.rollbacks += 1;
            }
        }
        let Some(seat) = self.s.seat() else { return };
        if seat.from == 0 {
            // coming in: as far as the pads are in, then ready
            self.frame = self.frame.max(self.s.confirmed());
            self.s.ready();
        }
        if self.frame >= self.s.confirmed() + AHEAD {
            return;
        }
        self.frame += 1;
        self.s
            .local(self.frame + seat.delay, pressed(seat.slot, self.frame));
        let _ = self.s.pads(self.frame);
    }
}

fn host(hub: &Hub, players: usize) -> Peer {
    let t: Box<dyn Transport> = Box::new(hub.transport());
    Peer {
        s: Session::host(t, "room", players, DELAY, QUIET, ROM).expect("hosting"),
        frame: 0,
        rollbacks: 0,
    }
}

fn join(hub: &Hub, rom: u32) -> Peer {
    let t: Box<dyn Transport> = Box::new(hub.transport());
    Peer {
        s: Session::join(t, "room", rom).expect("joining"),
        frame: 0,
        rollbacks: 0,
    }
}

/// Ticks every peer until they have all run `frames` and every pad up to
/// it is in everywhere (or the time is up).
fn play(peers: &mut [Peer], frames: u64) {
    let until = Instant::now() + Duration::from_secs(20);
    while peers.iter().any(|p| p.s.confirmed() < frames) {
        assert!(
            Instant::now() < until,
            "the peers got stuck: {:?}",
            peers
                .iter()
                .map(|p| (p.frame, p.s.confirmed()))
                .collect::<Vec<_>>()
        );
        for p in peers.iter_mut() {
            p.tick();
        }
        std::thread::sleep(Duration::from_micros(200));
    }
}

/// Every peer's pads for frames 1..=frames.
fn pads(peers: &mut [Peer], frames: u64) -> Vec<Vec<[u16; 4]>> {
    peers
        .iter_mut()
        .map(|p| (1..=frames).map(|f| p.s.pads(f).0).collect())
        .collect()
}

#[test]
fn peers_get_the_same_pads() {
    let players = 3;
    let hub = Hub::new().with_latency(Duration::from_millis(5));
    let mut peers = vec![host(&hub, players)];
    for _ in 1..players {
        peers.push(join(&hub, ROM));
    }
    let frames = 400;
    play(&mut peers, frames);
    let got = pads(&mut peers, frames);
    for g in &got[1..] {
        assert_eq!(g, &got[0], "every peer has the same pads");
    }
    // the host's seat is its own from the start
    for (i, row) in got[0].iter().enumerate() {
        let f = i as u64 + 1;
        let want = if f <= QUIET || f <= DELAY {
            0
        } else {
            pressed(0, f - DELAY)
        };
        assert_eq!(row[0], want, "frame {f}, the host");
        assert_eq!(row[3], 0, "frame {f}, no fourth player");
    }
    // a client's from the frame the host gave it (the first few: its
    // pad then, as it was behind that frame when it got the seat)
    for (k, p) in peers.iter().enumerate().skip(1) {
        let seat = p.s.seat().expect("a seat");
        assert!(
            seat.from > 0 && seat.from < frames / 2,
            "seated at {}",
            seat.from
        );
        for f in seat.from + AHEAD + DELAY..=frames {
            assert_eq!(
                got[k][f as usize - 1][seat.slot],
                pressed(seat.slot, f - DELAY),
                "frame {f}, seat {}",
                seat.slot
            );
        }
    }
    assert!(
        peers.iter().any(|p| p.rollbacks > 0),
        "with a delay on the line, some guesses are wrong"
    );
}

#[test]
fn a_late_peer_catches_up() {
    let hub = Hub::new().with_latency(Duration::from_millis(2));
    let mut peers = vec![host(&hub, 2)];
    play(&mut peers, 300);
    peers.push(join(&hub, ROM));
    play(&mut peers, 600);
    let got = pads(&mut peers, 600);
    assert_eq!(
        got[1], got[0],
        "the latecomer has the same pads, those before it came too"
    );
    let seat = peers[1].s.seat().expect("a seat");
    assert!(seat.from > 300, "seated once caught up: {}", seat.from);
    for f in 1..seat.from {
        assert_eq!(
            got[0][f as usize - 1][1],
            0,
            "frame {f}: the seat was empty"
        );
    }
}

#[test]
fn a_seat_is_free_again_when_its_player_leaves() {
    let hub = Hub::new();
    let mut peers = vec![host(&hub, 2), join(&hub, ROM)];
    play(&mut peers, 200);
    peers.pop();
    play(&mut peers, 400);
    // the host plays on alone; someone else can have the seat
    peers.push(join(&hub, ROM));
    play(&mut peers, 700);
    assert_eq!(peers[1].s.seat().map(|s| s.slot), Some(1));
}

#[test]
fn another_rom_is_refused() {
    let hub = Hub::new();
    let mut h = host(&hub, 2);
    let mut c = join(&hub, ROM + 1);
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        assert!(Instant::now() < until, "no answer");
        h.tick();
        match c.s.poll() {
            Err(net::Error::Refused(_)) => break,
            Err(e) => panic!("{e}"),
            Ok(()) => {}
        }
    }
}
