//! The session (docs/NETPLAY.md): every peer runs the whole game, only
//! the pads cross the network, and a peer doesn't wait for them.
//!
//! The host is the star's middle. Each peer sends its pad for a frame
//! `delay` frames ahead; the host passes every pad on to everyone (its
//! own, and the empty seats': 0). A peer asks [`Session::pads`] for a
//! frame's four pads whether they are all in or not: the ones not in yet
//! are guessed (the seat's last known pad), and when one comes that
//! differs from the guess, [`Session::take_rollback`] says from which
//! frame the game has to run again. [`Session::confirmed`] is how far
//! every pad is in: the game can't go more than a few frames past it.
//!
//! Frames count from 1 (the frames ended so far, as the game counts
//! them); the first `quiet` have every pad 0 (the boot, which runs at
//! once). The host starts alone; a client coming in gets a seat kept for
//! it and every pad so far, runs the game up to now, and then the seat
//! is its own from a frame the host picks ([`Session::ready`]). A
//! client that leaves frees its seat. Now and then each peer's state
//! hash (after a frame no guess went into) goes to the host, which
//! checks they agree. Nothing here blocks.

use std::collections::BTreeMap;

use crate::transport::{PeerId, Role, Transport, TransportEvent};
use crate::wire::{Message, RUNS, VERSION};

/// How often (frames) the states are compared.
pub const HASH_EVERY: u64 = 60;

/// The pads: four seats (the Team Player's), each a pad's buttons.
pub type Pads = [u16; 4];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The transport went away, with why.
    Closed(String),
    /// The host said no, with why.
    Refused(String),
    /// The host left.
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
            Error::Left => write!(f, "the host left"),
            Error::Desync(frame) => write!(f, "the games parted at frame {frame}"),
        }
    }
}

impl std::error::Error for Error {}

/// This peer's place in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seat {
    /// Its pad (0-3).
    pub slot: usize,
    /// How many seats (2-4; more than 2 is the Team Player).
    pub players: usize,
    /// The input delay (frames).
    pub delay: u64,
    /// The frames at the start whose pads are all 0.
    pub quiet: u64,
    /// The first frame whose pad is this peer's (0: not yet, it is still
    /// catching up).
    pub from: u64,
}

/// Who has a seat, as the host sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Held {
    /// Nobody: the host plays it, with no buttons.
    Empty,
    /// Kept for a client catching up: the host still plays it.
    Kept(PeerId),
    /// A client's.
    Played(PeerId),
}

/// What only the host keeps.
struct Host {
    rom: u32,
    seats: [Held; 4],
    /// Hashes: the host's own, and the clients' waiting for it.
    own: BTreeMap<u64, u32>,
    theirs: Vec<(u64, u32)>,
}

pub struct Session<T: Transport> {
    t: T,
    host: Option<Host>,
    /// A client's host.
    server: Option<PeerId>,
    rom: u32,
    seat: Option<Seat>,
    /// Every frame's pads so far as they are known (frame 0: all 0).
    known: Vec<[Option<u16>; 4]>,
    /// Every frame up to this one has all its pads.
    confirmed: u64,
    /// This peer's pad has gone out up to this frame.
    sent: u64,
    /// The pads handed out for frames not all in then.
    guessed: BTreeMap<u64, Pads>,
    /// The first frame a guess went wrong in, since the last
    /// [`Session::take_rollback`].
    rollback: Option<u64>,
    /// The host left or the transport closed.
    ended: Option<Error>,
    /// A client has asked for its seat ([`Session::ready`]).
    asked: bool,
}

impl<T: Transport> Session<T> {
    fn new(t: T, rom: u32) -> Self {
        Session {
            t,
            host: None,
            server: None,
            rom,
            seat: None,
            known: vec![[Some(0); 4]],
            confirmed: 0,
            sent: 0,
            guessed: BTreeMap::new(),
            rollback: None,
            ended: None,
            asked: false,
        }
    }

    /// Hosts `room` with `players` seats (2-4), this peer's the first,
    /// an input delay of `delay` frames and `quiet` frames with no pads
    /// at the start; `rom`, the ROM's CRC-32 (clients with another are
    /// refused). The game starts at once; the others come when they come.
    ///
    /// # Errors
    /// The transport failed.
    pub fn host(
        mut t: T,
        room: &str,
        players: usize,
        delay: u64,
        quiet: u64,
        rom: u32,
    ) -> Result<Self, Error> {
        t.connect(room, Role::Host).map_err(Error::Closed)?;
        let players = players.clamp(2, 4);
        let mut s = Session::new(t, rom);
        s.host = Some(Host {
            rom,
            seats: [Held::Empty; 4],
            own: BTreeMap::new(),
            theirs: Vec::new(),
        });
        s.welcome(Seat {
            slot: 0,
            players,
            delay,
            quiet,
            from: quiet + 1,
        });
        Ok(s)
    }

    /// Joins `room`'s host; [`Session::seat`] says when it has let this
    /// peer in.
    ///
    /// # Errors
    /// The transport failed.
    pub fn join(mut t: T, room: &str, rom: u32) -> Result<Self, Error> {
        t.connect(room, Role::Client).map_err(Error::Closed)?;
        Ok(Session::new(t, rom))
    }

    /// The seat, its pads all 0 through the quiet frames.
    fn welcome(&mut self, seat: Seat) {
        self.seat = Some(seat);
        self.sent = seat.from.saturating_sub(1);
        for f in 1..=seat.quiet {
            for slot in 0..4 {
                self.set(f, slot, 0);
            }
        }
    }

    /// This peer's seat, once it has one.
    #[must_use]
    pub fn seat(&self) -> Option<Seat> {
        self.seat
    }

    /// Whether this peer is the host.
    #[must_use]
    pub fn is_host(&self) -> bool {
        self.host.is_some()
    }

    /// Every frame up to this one has all its pads: the game is final
    /// there.
    #[must_use]
    pub fn confirmed(&self) -> u64 {
        self.confirmed
    }

    /// How many peers the transport is connected to.
    #[must_use]
    pub fn peers(&self) -> usize {
        self.t.peers().len()
    }

    /// Which seats' pads the first frame not all in still waits for
    /// (debugging).
    #[must_use]
    pub fn missing(&self) -> [bool; 4] {
        let row = self
            .known
            .get(self.confirmed as usize + 1)
            .copied()
            .unwrap_or([None; 4]);
        row.map(|p| p.is_none())
    }

    /// `pad`, this peer's for `frame` (and any frame before it not sent
    /// yet: the delay may have grown, or this peer has just got its
    /// seat). Frames already sent are left alone.
    pub fn local(&mut self, frame: u64, pad: u16) {
        let Some(seat) = self.seat else { return };
        if seat.from == 0 {
            return;
        }
        // the seats nobody plays yet, the host's to play
        let free: Vec<usize> = self.host.as_ref().map_or_else(Vec::new, |h| {
            (1..seat.players)
                .filter(|&k| matches!(h.seats[k], Held::Empty | Held::Kept(_)))
                .collect()
        });
        for f in self.sent + 1..=frame {
            self.set(f, seat.slot, pad);
            let out = Message::Input {
                frame: f,
                slot: seat.slot as u8,
                pad,
            }
            .encode();
            if self.host.is_some() {
                self.t.broadcast(&out);
                for &k in &free {
                    self.set(f, k, 0);
                    self.t.broadcast(
                        &Message::Input {
                            frame: f,
                            slot: k as u8,
                            pad: 0,
                        }
                        .encode(),
                    );
                }
            } else if let Some(server) = self.server {
                self.t.send(server, &out);
            }
        }
        self.sent = self.sent.max(frame);
    }

    /// The four pads for `frame`, and whether they are all in (else the
    /// missing ones are guessed: the seat's last known pad).
    pub fn pads(&mut self, frame: u64) -> (Pads, bool) {
        let row = self.known.get(frame as usize).copied().unwrap_or([None; 4]);
        let all = row.iter().all(Option::is_some);
        let mut pads = [0; 4];
        for (slot, p) in pads.iter_mut().enumerate() {
            *p = row[slot].unwrap_or_else(|| self.guess(frame, slot));
        }
        if !all {
            self.guessed.insert(frame, pads);
        }
        (pads, all)
    }

    /// A seat's last known pad before `frame`.
    fn guess(&self, frame: u64, slot: usize) -> u16 {
        let top = (frame as usize).min(self.known.len());
        self.known[..top]
            .iter()
            .rev()
            .find_map(|r| r[slot])
            .unwrap_or(0)
    }

    /// The first frame whose pads were guessed wrong since the last
    /// call: the game has to go back to before it.
    pub fn take_rollback(&mut self) -> Option<u64> {
        self.rollback.take()
    }

    /// This peer has caught up: its seat can be its own.
    pub fn ready(&mut self) {
        if let (Some(server), Some(seat)) = (self.server, self.seat)
            && seat.from == 0
            && !self.asked
        {
            // (asked once: the answer is a Seat)
            self.t.send(server, &Message::Ready.encode());
            self.asked = true;
        }
    }

    /// This peer's state hash after `frame` (one no guess went into),
    /// for the check.
    pub fn hash(&mut self, frame: u64, hash: u32) {
        if let Some(h) = &mut self.host {
            h.own.insert(frame, hash);
        } else if let Some(server) = self.server {
            self.t.send(server, &Message::Hash { frame, hash }.encode());
        }
    }

    /// A pad in: if it was guessed otherwise, the game goes back.
    fn set(&mut self, frame: u64, slot: usize, pad: u16) -> bool {
        let f = frame as usize;
        if self.known.len() <= f {
            let players = self.seat.map_or(4, |s| s.players);
            let row = [0, 1, 2, 3].map(|k| (k >= players).then_some(0));
            self.known.resize(f + 1, row);
        }
        if self.known[f][slot].is_some() {
            return false;
        }
        self.known[f][slot] = Some(pad);
        if let Some(g) = self.guessed.get(&frame)
            && g[slot] != pad
        {
            self.rollback = Some(self.rollback.map_or(frame, |r| r.min(frame)));
        }
        while self
            .known
            .get(self.confirmed as usize + 1)
            .is_some_and(|r| r.iter().all(Option::is_some))
        {
            self.confirmed += 1;
        }
        let c = self.confirmed;
        self.guessed.retain(|&f, _| f > c);
        true
    }

    /// What came: the host lets clients in, passes their pads on and
    /// checks their hashes; a client takes its seat and the pads.
    ///
    /// # Errors
    /// The host left, the transport closed, or the games parted.
    pub fn poll(&mut self) -> Result<(), Error> {
        if let Some(e) = &self.ended {
            return Err(e.clone());
        }
        for e in self.t.poll() {
            match e {
                TransportEvent::Connected(peer) => {
                    if self.host.is_none() {
                        self.server = Some(peer);
                        self.t.send(
                            peer,
                            &Message::Hello {
                                version: VERSION,
                                rom: self.rom,
                            }
                            .encode(),
                        );
                    }
                }
                TransportEvent::Disconnected(peer) => {
                    if self.host.is_some() {
                        self.leave(peer);
                    } else {
                        self.ended = Some(Error::Left);
                    }
                }
                TransportEvent::Closed(why) => self.ended = Some(Error::Closed(why)),
                TransportEvent::Message(peer, data) => {
                    if let Some(m) = Message::decode(&data) {
                        self.message(peer, m)?;
                    }
                }
            }
        }
        if let Some(h) = &mut self.host {
            let own = &h.own;
            if let Some(&(frame, _)) = h
                .theirs
                .iter()
                .find(|&&(f, x)| own.get(&f).is_some_and(|&y| y != x))
            {
                self.t.broadcast(&Message::Desync { frame }.encode());
                self.ended = Some(Error::Desync(frame));
            } else {
                h.theirs.retain(|&(f, _)| !own.contains_key(&f));
                if let Some(&last) = h.own.keys().next_back() {
                    h.own.retain(|&f, _| f + 4 * HASH_EVERY > last);
                }
            }
        }
        match &self.ended {
            Some(e) => Err(e.clone()),
            None => Ok(()),
        }
    }

    /// The host: a client went; its seat is nobody's from the frame after
    /// its last pad.
    fn leave(&mut self, peer: PeerId) {
        let Some(h) = self.host.as_mut() else { return };
        let Some(slot) = h
            .seats
            .iter()
            .position(|&s| s == Held::Kept(peer) || s == Held::Played(peer))
        else {
            return;
        };
        let played = h.seats[slot] == Held::Played(peer);
        h.seats[slot] = Held::Empty;
        if played {
            for f in 1..=self.sent {
                if self.set(f, slot, 0) {
                    self.t.broadcast(
                        &Message::Input {
                            frame: f,
                            slot: slot as u8,
                            pad: 0,
                        }
                        .encode(),
                    );
                }
            }
        }
    }

    fn message(&mut self, peer: PeerId, m: Message) -> Result<(), Error> {
        if self.host.is_some() {
            self.host_message(peer, m);
            return Ok(());
        }
        match m {
            Message::Welcome {
                slot,
                players,
                delay,
                quiet,
            } => self.welcome(Seat {
                slot: usize::from(slot),
                players: usize::from(players),
                delay: u64::from(delay),
                quiet,
                from: 0,
            }),
            Message::Refuse(why) => return Err(Error::Refused(why)),
            // (what came before the Welcome is in the history after it)
            _ if self.seat.is_none() => {}
            Message::Input { frame, slot, pad } => {
                self.set(frame, usize::from(slot) & 3, pad);
            }
            Message::History { from, runs } => {
                let mut f = from;
                for (n, pads) in runs {
                    for _ in 0..n {
                        for (slot, &p) in pads.iter().enumerate() {
                            self.set(f, slot, p);
                        }
                        f += 1;
                    }
                }
            }
            Message::Seat { slot, from } => {
                if let Some(seat) = &mut self.seat
                    && seat.slot == usize::from(slot)
                {
                    seat.from = from;
                    self.sent = from - 1;
                }
            }
            Message::Desync { frame } => return Err(Error::Desync(frame)),
            _ => {}
        }
        Ok(())
    }

    fn host_message(&mut self, peer: PeerId, m: Message) {
        let Some(seat) = self.seat else { return };
        let Some(h) = self.host.as_mut() else { return };
        let mine = h.seats.iter().position(|&s| s == Held::Played(peer));
        match m {
            Message::Hello { version, rom } => {
                let free = (1..seat.players).find(|&k| h.seats[k] == Held::Empty);
                let why = if version != VERSION {
                    Some(format!("version {version}, the host's is {VERSION}"))
                } else if rom != h.rom {
                    Some("another ROM".to_owned())
                } else if free.is_none() {
                    Some("the game is full".to_owned())
                } else {
                    None
                };
                match (why, free) {
                    (None, Some(slot)) => {
                        h.seats[slot] = Held::Kept(peer);
                        let welcome = Message::Welcome {
                            slot: slot as u8,
                            players: seat.players as u8,
                            delay: seat.delay as u8,
                            quiet: seat.quiet,
                        };
                        self.t.send(peer, &welcome.encode());
                        self.history(peer);
                    }
                    (why, _) => self
                        .t
                        .send(peer, &Message::Refuse(why.unwrap_or_default()).encode()),
                }
            }
            Message::Input { frame, pad, .. } => {
                if let Some(slot) = mine
                    && self.set(frame, slot, pad)
                {
                    let out = Message::Input {
                        frame,
                        slot: slot as u8,
                        pad,
                    }
                    .encode();
                    for other in self.t.peers() {
                        if other != peer {
                            self.t.send(other, &out);
                        }
                    }
                }
            }
            Message::Hash { frame, hash } => h.theirs.push((frame, hash)),
            Message::Ready => {
                if let Some(slot) = h.seats.iter().position(|&s| s == Held::Kept(peer)) {
                    h.seats[slot] = Held::Played(peer);
                    // the host has played the seat up to here
                    let from = self.sent + 1;
                    self.t.broadcast(
                        &Message::Seat {
                            slot: slot as u8,
                            from,
                        }
                        .encode(),
                    );
                }
            }
            _ => {}
        }
    }

    /// Every pad so far, to a client coming in: the frames all in as
    /// runs, then the pads of the frames after one by one.
    fn history(&mut self, peer: PeerId) {
        let mut runs: Vec<(u16, Pads)> = Vec::new();
        let mut from = 1;
        for f in 1..=self.confirmed as usize {
            let pads = self.known[f].map(|p| p.unwrap_or(0));
            match runs.last_mut() {
                Some((n, p)) if *p == pads && *n < u16::MAX => *n += 1,
                _ => runs.push((1, pads)),
            }
            if runs.len() == RUNS {
                let n: u64 = runs.iter().map(|&(n, _)| u64::from(n)).sum();
                self.t.send(
                    peer,
                    &Message::History {
                        from,
                        runs: std::mem::take(&mut runs),
                    }
                    .encode(),
                );
                from += n;
            }
        }
        if !runs.is_empty() {
            self.t.send(peer, &Message::History { from, runs }.encode());
        }
        for f in self.confirmed as usize + 1..self.known.len() {
            for slot in 0..4 {
                if let Some(pad) = self.known[f][slot] {
                    self.t.send(
                        peer,
                        &Message::Input {
                            frame: f as u64,
                            slot: slot as u8,
                            pad,
                        }
                        .encode(),
                    );
                }
            }
        }
    }
}
