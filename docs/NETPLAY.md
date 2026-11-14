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
