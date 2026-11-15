# Roadmap

The end state follows Bomberman 64 (`../Fuse64`) and Blast Corps
(`../DozerEncore`): every ROM rebuilds byte for byte from one source tree,
the ROM's assets are editable files that rebuild to the same bytes, and a
port plays the game on Linux, Windows, macOS and in the browser, with
**online multiplayer** over WebRTC, natively and in the browser alike
(docs/NETPLAY.md).

The difference from the N64 projects: Mega Bomberman is handwritten 68000
assembly, not compiled C. There is no compiler output to match, so the
"decompilation" is two things:

1. a **matching disassembly** that is complete and symbolic (every
   pointer a label, so the code can move), with names and documentation;
2. a **translation into Rust**, function by function, checked against the
   original running in an emulator, which becomes the port's game code.

The port is Rust, as in the VacationDestination and GadgetGenerator ports (winit,
wgpu, cpal, matchbox), not C: no part of the port has to compile to
anything in particular, and the netplay stack (matchbox and GGRS-style
rollback) is native Rust. The matching build stays Python tools plus GNU
as.

Versions: **us** is the working version; **eu** builds from the same
source with `.if`s, using us's names (docs/LAYOUT.md, "US and EU").

## Phase 0: scaffolding (done)

- Repo, `make setup` (GNU binutils 2.47 for `m68k-elf`, sha256-pinned,
  `tools/get_binutils.sh`), `mbm.<v>.sha1`.
- `tools/m68k.py`, a 68000 decoder that knows what gas can spell, and
  `tools/test_m68k.py`, its ROM-free test against gas (CI).

## Phase 1: matching disassembly (done)

`make extract && make VERSION=us` rebuilds each ROM from
`asm/`, sha1-checked, for us and eu. Code is found statically
(docs/LAYOUT.md, "Finding code"); what isn't found stays data.

## Phase 2: complete and symbolic

- **A runtime trace (done).** `tools/mdtrace` over Genesis Plus GX
  marks what runs and what is read; scripted inputs cover the intro,
  menus, battle mode with two and four pads, and every normal-game stage
  through the passwords (docs/LAYOUT.md, "Runtime traces"). The evidence
  is `code.<v>.txt`/`data.<v>.txt`, and the trace wins over speculation.
  The ending is covered with RAM writes in the script (`ending`); the
  continue and game-over screens come up in random play, and with RAM
  writes too every stage's exit, bonus stage and next card, every boss's
  defeat and the next area's intro (`exit-*`, the plans found by
  `tools/find_exits.py`), every item kind and Louie ability (`items`),
  every battle arena (`arena-*`), the clock running out, the final
  boss's forms left to themselves and EA's 4-Way Play. What still never
  runs is in docs/UNRUN.md (`tools/unrun.py`), the list to hunt down. A human
  playthrough recorded as a pad script would do, at the end (BizHawk's
  Mega Drive core is Genesis Plus GX too, so its `.bk2` input logs should
  play back in mdtrace).
- **Data words become symbols (started).** Long immediates used as
  addresses, pointer longs the traces read and the rest of their tables,
  table addresses used as displacements (docs/LAYOUT.md, "Pointers").
  Bytecode scripts are walked op by op (`func_015FAE`'s, so far), and
  tables of records by what the code does with each field (`D_00A04A`).
  Word pointers: none. No `movea.w` loads a ROM address and there are
  no offset tables (`move.w (tbl,pc,dn.w)` then `jmp`); the words
  `tools/find_abs.py` lists are numbers (`0x200`, `0x7000`, colours) or
  operands in dead code that is still bytes (after `D_00A83A`,
  `D_013838`, `D_01974E`: nothing marks where it starts). Still to do:
  the lists those records point at (0xA0AA's entries hold a long at odd
  offsets) and the data's own pointers (0x40000 on).
  RAM gets a symbol file (`0xFF8000`-`0xFFFFFF`), still at fixed
  addresses.
- **Objects.** Files cut at module boundaries rather than at long data,
  with `.s` per function where that helps.
- **SHIFT=1 (in place for the code).** Pads after each object and before
  most functions, recomputes the checksum, and `make shift-check` runs the
  shifted and the original ROM headless over every script in
  tools/inputs/, comparing screens and work RAM (docs/LAYOUT.md,
  "SHIFT"): us and eu pass. The data (0x40000 on) stays at its address
  for now; moving it too needs its formats (phase 3) and the Z80's bank
  pointers. `tools/find_abs.py` lists what may still be an address left
  as a number.
- **eu from us (done).** `tools/vermap.py` maps us to eu, eu takes us's
  names, and `asm/` is one source for both (`.if VERSION_EU` in three
  places). Comparing the versions through the map also tells numbers
  from pointers both ways (docs/LAYOUT.md, "US and EU").

## Phase 3: assets

The data from `0x40000` on: compressed tiles (Hudson's compression,
to be identified), tilemaps and stage layouts, palettes, sprite mappings,
text, the Z80 sound driver and the music and sound effect data. Each gets
an extractor to editable files (PNG, YAML, ...) that must rebuild to the
same bytes, as `tools/assets.py` does in B64 and Blast Corps.

## Phase 4: the game in Rust

- `tools/recomp/`: a mechanical 68000 → Rust translator, one Rust
  function per routine over a CPU state (`d`/`a` registers, flags, the
  68000's memory map), the way Blast Corps' `tools/recomp` handled Rare's
  handwritten code. It is checked by running each function both ways from
  the same states (the emulator's), comparing registers and memory.
- Then the translated code is rewritten as readable Rust, function by
  function, with typed game state in place of the raw RAM. The game's
  bugs (reads past a table into code, docs/LAYOUT.md "SHIFT") are fixed
  there, not reproduced, each one noted and allowed for in the
  differential tests. The
  differential tests stay as the arbiter, and the TAS (if one syncs) is
  the end-to-end test.
- No asm in the port: the 68000 code is all translated, and so is the
  Z80 sound driver (or it runs on a small Z80 interpreter until it is).

## Phase 5: the port

A Cargo workspace like VacationDestination's: `crates/game` (the translated
game, no platform dependencies), `crates/md` (the Mega Drive hardware
the game drives: the VDP (planes A/B, window, sprites, per-line scroll and
HBlank effects, DMA), the YM2612 and SN76489, the pads and the Team Player
multitap), `crates/platform` (winit, wgpu, cpal), `crates/net`, and the
binary, `megafuse`. Linux, Windows, macOS and `wasm32-unknown-unknown`, built and
packaged by CI as in the other ports. The port is deterministic from
the start (docs/NETPLAY.md, "Requirements").

Enhancements stay switchable and off in compared runs: widescreen (more
of the arena), no sprite limits, and the like.

## Phase 6: online multiplayer

docs/NETPLAY.md.
