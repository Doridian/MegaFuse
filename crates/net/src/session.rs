//! The lockstep session (docs/NETPLAY.md): every peer runs the whole game,
//! only the pads cross the network.  Each peer sends its pad for a frame
//! `delay` frames ahead; the host, the star's middle, gathers the
//! players' pads and sends every frame's four to everyone, itself
//! included: a frame runs, on every peer, only with them.  Frames count
//! from 1 (the frames ended so far, as the game counts them); the first
//! `delay` have no pads pressed.  Now and then each peer's state
//! hash goes to the host, which checks they agree.

use std::collections::BTreeMap;
use std::time::Duration;

use crate::transport::{PeerId, Role, Transport, TransportEvent};
use crate::wire::{Message, VERSION};

/// How often (frames) the states are compared.
pub const HASH_EVERY: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The transport went away, with why.
    Closed(String),
    /// The host said no, with why.
    Refused(String),
    /// A peer (the host, for a client) left.
    Left,
    /// The states parted at a frame: the game isn't the same on every
    /// peer any more.
    Desync(u64),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Closed(why) => write!(f, "the connection closed: {why}"),
            Error::Refused(why) => write!(f, "the host refused: {why}"),
            Error::Left => write!(f, "a player left"),
            Error::Desync(frame) => write!(f, "the games parted at frame {frame}"),
        }
    }
}

impl std::error::Error for Error {}

/// What the host knows of the players.
struct Host {
    players: usize,
    delay: u64,
    rom: u32,
    /// Each client's pad (the host is 0).
    slots: BTreeMap<PeerId, usize>,
    /// The pads in, frame by frame, until the frame goes out.
    inputs: BTreeMap<u64, [Option<u16>; 4]>,
    /// The next frame to send.
    next: u64,
    /// Hashes: the host's own, and the clients' waiting for it.
    own: BTreeMap<u64, u32>,
    theirs: Vec<(u64, u32)>,
}

pub struct Session<T: Transport> {
    t: T,
    host: Option<Host>,
    /// A client's host.
    server: Option<PeerId>,
    /// This peer's pad, the players, the delay (once started).
    slot: usize,
    players: usize,
    delay: u64,
    /// The frames' pads, as the host sent them.
    frames: BTreeMap<u64, [u16; 4]>,
    /// A peer left or the transport closed: the frames already in still
    /// run, then it ends.
    ended: Option<Error>,
}

impl<T: Transport> Session<T> {
    /// Hosts `room` for `players` (2-4, this one included) with an input
    /// delay of `delay` frames; `rom`, the ROM's CRC-32 (clients with
    /// another are refused).  Waits until the players are there.
    ///
    /// # Errors
    /// The transport failed.
    pub fn host(mut t: T, room: &str, players: usize, delay: u64, rom: u32) -> Result<Self, Error> {
        t.connect(room, Role::Host).map_err(Error::Closed)?;
        let mut s = Session {
            t,
            host: Some(Host {
                players: players.clamp(1, 4),
                delay,
                rom,
                slots: BTreeMap::new(),
                inputs: BTreeMap::new(),
                next: delay + 1,
                own: BTreeMap::new(),
                theirs: Vec::new(),
            }),
            server: None,
            slot: 0,
            players: players.clamp(1, 4),
            delay,
            frames: BTreeMap::new(),
            ended: None,
        };
        while s.host.as_ref().is_some_and(|h| h.slots.len() + 1 < h.players) {
            s.pump()?;
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(s)
    }

    /// Joins `room`'s host; waits to be let in.
    ///
    /// # Errors
    /// The transport failed or the host refused.
    pub fn join(mut t: T, room: &str, rom: u32) -> Result<Self, Error> {
        t.connect(room, Role::Client).map_err(Error::Closed)?;
        let mut s = Session {
            t,
            host: None,
            server: None,
            slot: usize::MAX,
            players: 0,
            delay: 0,
            frames: BTreeMap::new(),
            ended: None,
        };
        while s.slot == usize::MAX {
            for e in s.t.poll() {
                match e {
                    TransportEvent::Connected(peer) => {
                        s.server = Some(peer);
                        s.t.send(peer, &Message::Hello { version: VERSION, rom }.encode());
                    }
                    TransportEvent::Message(_, data) => match Message::decode(&data) {
                        Some(Message::Welcome { slot, players, delay }) => {
                            s.slot = usize::from(slot);
                            s.players = usize::from(players);
                            s.delay = u64::from(delay);
                        }
                        Some(Message::Refuse(why)) => return Err(Error::Refused(why)),
                        _ => {}
                    },
                    TransportEvent::Disconnected(_) => return Err(Error::Left),
                    TransportEvent::Closed(why) => return Err(Error::Closed(why)),
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(s)
    }

    /// This peer's pad (0-3) and how many players there are.
    #[must_use]
    pub fn seat(&self) -> (usize, usize) {
        (self.slot, self.players)
    }

    /// The pads for `frame`: this peer's `local` pad goes out for the
    /// frame `delay` on, then it waits for the host's pads of `frame`.
    /// `hash`, this peer's state hash before `frame` (every
    /// [`HASH_EVERY`]), for the check.
    ///
    /// # Errors
    /// A peer left, the transport closed, or the games parted.
    pub fn pads(&mut self, frame: u64, local: u16, hash: Option<u32>) -> Result<[u16; 4], Error> {
        let ahead = frame + self.delay;
        if let Some(h) = &mut self.host {
            h.inputs.entry(ahead).or_default()[0] = Some(local);
            if let Some(x) = hash {
                h.own.insert(frame, x);
            }
        } else if let Some(server) = self.server {
            self.t.send(server, &Message::Input { frame: ahead, pad: local }.encode());
            if let Some(x) = hash {
                self.t.send(server, &Message::Hash { frame, hash: x }.encode());
            }
        }
        if frame <= self.delay {
            return Ok([0; 4]);
        }
        loop {
            self.pump()?;
            if let Some(p) = self.frames.remove(&frame) {
                return Ok(p);
            }
            if let Some(e) = &self.ended {
                return Err(e.clone());
            }
            std::thread::sleep(Duration::from_micros(500));
        }
    }

    /// What came: the host lets players in, takes their pads and hashes,
    /// sends the frames it has all the pads of; a client takes frames.
    fn pump(&mut self) -> Result<(), Error> {
        for e in self.t.poll() {
            match e {
                TransportEvent::Connected(_) => {}
                TransportEvent::Disconnected(_) => self.ended = Some(Error::Left),
                TransportEvent::Closed(why) => self.ended = Some(Error::Closed(why)),
                TransportEvent::Message(peer, data) => {
                    let Some(m) = Message::decode(&data) else { continue };
                    self.message(peer, m)?;
                }
            }
        }
        if let Some(h) = &mut self.host {
            // the frames whose pads are all in
            while let Some(p) = h.inputs.get(&h.next) {
                if p.iter().take(h.players).any(Option::is_none) {
                    break;
                }
                let pads = [0, 1, 2, 3].map(|k| p[k].unwrap_or(0));
                h.inputs.remove(&h.next);
                let out = Message::Frame { frame: h.next, pads }.encode();
                for &peer in h.slots.keys() {
                    self.t.send(peer, &out);
                }
                self.frames.insert(h.next, pads);
                h.next += 1;
            }
            // the clients' hashes against the host's
            let own = &h.own;
            if let Some(&(frame, _)) = h.theirs.iter().find(|&&(f, x)| own.get(&f).is_some_and(|&y| y != x)) {
                let out = Message::Desync { frame }.encode();
                for &peer in h.slots.keys() {
                    self.t.send(peer, &out);
                }
                return Err(Error::Desync(frame));
            }
            h.theirs.retain(|&(f, _)| !own.contains_key(&f));
            if let Some(&last) = h.own.keys().next_back() {
                h.own.retain(|&f, _| f + 4 * HASH_EVERY > last);
            }
        }
        Ok(())
    }

    fn message(&mut self, peer: PeerId, m: Message) -> Result<(), Error> {
        if let Some(h) = &mut self.host {
            match m {
                Message::Hello { version, rom } => {
                    let why = if version != VERSION {
                        Some(format!("version {version}, the host's is {VERSION}"))
                    } else if rom != h.rom {
                        Some("another ROM".to_owned())
                    } else if h.slots.len() + 1 >= h.players {
                        Some("the game is full".to_owned())
                    } else {
                        None
                    };
                    if let Some(why) = why {
                        self.t.send(peer, &Message::Refuse(why).encode());
                    } else {
                        let slot = h.slots.len() + 1;
                        h.slots.insert(peer, slot);
                        let welcome = Message::Welcome {
                            slot: slot as u8,
                            players: h.players as u8,
                            delay: h.delay as u8,
                        };
                        self.t.send(peer, &welcome.encode());
                    }
                }
                Message::Input { frame, pad } => {
                    if let Some(&slot) = h.slots.get(&peer)
                        && frame >= h.next
                    {
                        h.inputs.entry(frame).or_default()[slot] = Some(pad);
                    }
                }
                Message::Hash { frame, hash } => h.theirs.push((frame, hash)),
                _ => {}
            }
        } else {
            match m {
                Message::Frame { frame, pads } => {
                    self.frames.insert(frame, pads);
                }
                Message::Desync { frame } => return Err(Error::Desync(frame)),
                _ => {}
            }
        }
        Ok(())
    }
}
