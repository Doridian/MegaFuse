# Online multiplayer (plan)

Mega Bomberman's battle mode is up to four players on one console, with
Sega's Team Player multitap (the header's I/O field is `J4`). The port
turns that into online play the way Bomberman 64's plan does
(`../Fuse64/docs/NETPLAY.md`): each player runs the game, only
controller inputs cross the network, and every peer simulates the same
frames. The transport is WebRTC data channels from both the native and the
browser build, so browser and native players can share a match.

Here the port is Rust, so the model isn't imitated, it is used:

| layer | here |
| --- | --- |
| signaling | matchbox's protocol; `matchbox_server` as the lobby server |
| transport | `matchbox_socket` (native and wasm), unreliable and unordered channels |
| session | rollback, a session layer of our own (`crates/net/src/session.rs`) |
| game | the translated game, one frame per step, with the four pads as input |

VacationDestination's `crates/net` (a transport trait, the matchbox transport, an
in-memory transport for tests, a lockstep session, the wire format) is
the starting point.

## Why this is easier than on the N64

The whole state of a Mega Drive game is small and has no pointers into
the host: 64 KB of work RAM, the VDP's 64 KB of VRAM, CRAM, VSRAM and its
registers, the Z80's 8 KB and the sound chips' registers. A snapshot is
about 140 KB, a copy of fixed arrays, the same on every host. Rollback
can afford to save every frame, and a state hash means the same thing on
a 64-bit native peer and a wasm32 one. In the translated game, typed state
replaces the raw RAM bit by bit, but it stays plain data without
pointers, so the same holds.

## Requirements the port has to meet first

Where the port is (docs/PORT.md, "Time" and "A step"): the port keeps
the console's time, so the game advances by its inputs alone (1; the HV
counter and the like are the console's, from the state); `Game::step`
is one pass of the main loop, lag frames from the state (2, 3); the game
is integers (4); `Game::save` and `Game::restore` (5). Still to do: the
hash across hosts (6) and a step that doesn't draw.

From B64's list, adapted:

1. **The game advances by inputs alone.** A step is a pure function of
   (state, four pads). Where the game reads a counter as a timer or RNG
   seed (the HV counter, the VBlank count), the port feeds it a value
   derived from the frame number. Whatever the RNG is seeded from is found
   in phase 2.
2. **One step on demand.** `step(pads[4])` runs one VBlank's worth of the
   game, drawing optional. Rollback re-simulates several frames per host
   frame without drawing them.
3. **Lag frames are frames.** If the game's main loop misses a VBlank
   under load, the port reproduces that from the state, not from host
   timing, so every peer lags the same.
4. **Integer only.** The 68000 has no floating point, and the translated
   code keeps the game's fixed-point arithmetic. Host floats are only for
   presentation (scaling, interpolation), never in the game state.
5. **Save and restore.** The snapshot is the state above. Sound output
   isn't part of it: re-simulated frames don't play their sound again.
6. **A state hash across hosts.** The hash covers the snapshot's bytes.
   A replay test runs native and wasm and compares hashes every frame.

## Where it is

Rollback over WebRTC works, natively and in the browser, and they play
together (`crates/net`, `crates/server`, `megafuse --host`/`--join`, the
page's "Online" box). The transport, the matchbox transport and the
signalling server are VacationDestination's; the session
(`crates/net/src/session.rs`) and the driver (`crates/megafuse/src/
netplay.rs`) are this port's own.

- **The session.** The host is the star's middle. Each peer sends its
  pad `delay` frames ahead (2 by default) to the host, which passes
  every pad on to everyone at once, its own and the empty seats' (0)
  included. Nothing waits: a frame's pads not in yet are guessed (the
  seat's last pad), and a pad that comes in unlike its guess means a
  rollback from that frame. The frames whose pads are all in are
  *confirmed*; the game runs at most `AHEAD` (8) frames past them, and
  waits there. The first `QUIET` (240) frames have every pad 0: the
  boot, which runs at once, and a little more.
- **The driver.** It owns the game: a snapshot (`Game::save`) after every
  step, the frames' pads from the session in the frontend's callback,
  and on a wrong guess the snapshot before it restored and the frames
  since run again silently (the window has shown them). Snapshots older
  than the last confirmed one go.
- **Coming and going.** The host starts at once, its other seats empty.
  A client coming in, at any time, is kept a seat and sent every pad so
  far (`History`, as runs); it runs the game from the boot on them,
  silently and as fast as it can ("catching up"), then says `Ready`, and
  the seat is its own from the next frame the host hasn't played yet
  (`Seat`). A client that leaves frees its seat (the host plays it, as
  0, from the frame after its last pad); someone else can take it. If
  the host leaves, the game ends.
- **The check.** Every 60 frames each client sends the host its RAM's
  hash after a frame no guess went into; the host compares it with its
  own, and a difference ends the game for everyone (a desync). A native
  and a browser peer agree: a page joined a native host 23,000 frames in,
  caught up in some 35 s and played on.

    megafuse-server [--listen=0.0.0.0:3536]
    megafuse ROM --host ROOM [--players N] [--delay D] [--signal ws://HOST:3536]
    megafuse ROM --join ROOM [--signal ws://HOST:3536]
        [--stun stun:HOST:PORT]... [--turn turn:HOST:PORT --turn-user U --turn-password P]

The page takes the same: the room, the players, the signalling server
and a STUN server, and `?room=...&signal=...&stun=...` fills them in.

**STUN.** Without a STUN (or TURN) server the peers only offer their own
addresses: fine for native peers on one network, but the browser hides
its own behind mDNS names (`....local`), which the native peers can't
resolve, so a page and a native peer need a STUN server even on one
machine (any: `scratch/stun.py`-like test servers do). Across the
internet every peer needs one (and TURN behind a strict NAT). None is
contacted unless named.

`megafuse ROM --loopback N --input SCRIPT --frames F [--latency MS]
[--late MS] [--speed X]` is the harness: N peers in one process over the
in-memory transport, each message `--latency` late, each peer playing
its pad of the script at X times the console's pace (8 by default), the
last one coming in `--late` ms after the start. Every peer has to end
with the same RAM: 4 peers through 20000 frames of battle4 with 30 ms
(some 1,200 rollbacks each, 15,000 frames run again), 3 with the third
coming in 5 s late (seated at frame 4148). `cargo test -p net` runs the
session without a game (guesses, a latecomer, a seat freed and taken
again, a refused ROM). With `--frames` and `--input`, `--host`/`--join`
run headless too. `MEGAFUSE_NETDEBUG=1` prints each headless peer's
frame, confirmed frame and seat every second; `MEGAFUSE_NETSHOT=N`
writes each peer's picture at frame N.

Not yet: spectators, a lobby (the menus are played through the
session), rollback of the sound (rerun frames are silent; the frames
shown keep their sound).

## Phases

1. **Input plumbing.** Four pads from local devices, a replay file or the
   network, behind the Team Player as the game sees it.
2. **Loopback harness.** Two to four instances in one process over a
   simulated lossy, delayed link, comparing hashes every frame. A desync
   fails CI.
3. **Lockstep over WebRTC**, with a fixed input delay (done; replaced by
   4, which is lockstep when the guesses are never needed).
4. **Rollback.** Predict missing inputs (repeat the last), simulate ahead,
   restore and re-simulate on a misprediction (done).
5. **Lobby.** Room codes over the signaling server, peers mapped to
   battle-mode player numbers, spectators, and a dropped player handed to
   the game's own CPU player if battle mode allows it.

## Open questions

- Which of the game's modes take four pads: battle mode certainly. Does
  normal game have a two-player mode worth netplay?
- Whether the game also supports EA's 4 Way Play adapter. The code that
  reads the pads (`0x7E08`, `0x9086`) says which adapters it detects.
- us is 60 Hz; whether eu runs differently at 50 Hz. A version and ROM
  hash handshake refuses mismatched peers either way.
- Menus before a match: driven by all pads through the session, or
  replaced by a lobby that starts the match with agreed settings.
