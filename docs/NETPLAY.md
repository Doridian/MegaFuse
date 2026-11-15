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
| session | lockstep first, then rollback (GGRS, or a session layer of our own as in VacationDestination's `crates/net`) |
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

Lockstep works (`crates/net`, `crates/server`, `megafuse --host` and
`--join`): the transport, the matchbox transport and the signalling
server are VacationDestination's; the session is this port's own. Each
peer sends its pad `delay` frames ahead (3 by default) to the host, the
star's middle, which sends each frame's four pads to everyone, itself
included; a frame runs on a peer only with them, so the games are the
same everywhere. Every 60 frames the clients' RAM hashes go to the host,
which compares them with its own. With three or four players every peer
has the Team Player plugged in.

    megafuse-server [--listen=0.0.0.0:3536]
    megafuse ROM --host ROOM [--players N] [--delay D] [--signal ws://HOST:3536]
    megafuse ROM --join ROOM [--signal ws://HOST:3536]

`megafuse ROM --loopback N --input SCRIPT --frames F` is the loopback
harness: N peers in one process over the in-memory transport, each
playing its pad of the script: 4 peers (Team Player) agree through 20000
frames of battle4, 2 through 30000 of battle. With `--frames` and
`--input`, `--host`/`--join` run headless too (two peers over WebRTC on
one machine agree).

Not yet: rollback (the snapshots are there, `Game::save`), joining a game
under way, the browser build.

## Phases

1. **Input plumbing.** Four pads from local devices, a replay file or the
   network, behind the Team Player as the game sees it.
2. **Loopback harness.** Two to four instances in one process over a
   simulated lossy, delayed link, comparing hashes every frame. A desync
   fails CI.
3. **Lockstep over WebRTC**, with a fixed input delay. It is always
   correct, so it stays as the fallback.
4. **Rollback.** Predict missing inputs (repeat the last), simulate ahead,
   restore and re-simulate on a misprediction.
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
