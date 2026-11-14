# Mega Bomberman

A matching disassembly of Hudson Soft's *Mega Bomberman* (Sega Mega
Drive / Genesis, 1994; the port of *Bomberman '94* was made by Westone), aimed at a port to Rust for Linux, Windows, macOS
and the browser, with online multiplayer over WebRTC. You need your own
ROM: none of the game's code or data is in this repository.

Both ROMs rebuild byte for byte from disassembled code and data. The game
is handwritten 68000 assembly, so the plan is a complete, symbolic
disassembly first, then a translation of it into Rust, checked against the
original in an emulator. docs/ROADMAP.md has the phases, and
docs/NETPLAY.md the multiplayer design.

## ROMs

| version | file | no-intro | sha1 |
| --- | --- | --- | --- |
| `us` | `baserom.us.bin` | `Mega Bomberman (USA)` | `89495a8acbcb0ddcadfe5b7bada50f4d9efd0ddd` |
| `eu` | `baserom.eu.bin` | `Mega Bomberman (Europe, Korea) (En)` | `01f76e2f719bdae5f21ff0e5a1ac1262c2def279` |

Both are 1 MiB, plain big-endian (`.md`/`.bin`, not interleaved `.smd`).
`us` is the working version. eu is the same program with two small code
differences (docs/LAYOUT.md, "US and EU").

## Build

You need Python 3, a C compiler, `git`, `curl` and zlib.

```
make setup                 # once: GNU binutils for m68k-elf, Genesis Plus GX, tools/mdtrace
make VERSION=us extract    # baserom.us.bin (sha1-checked) -> asm/
make VERSION=us            # assemble, link, sha1-checked
make test                  # the decoder against gas, no ROM needed
tools/trace_all.sh us      # play tools/inputs/ headless, regenerate code.us.txt/data.us.txt
```

On Debian/Ubuntu, `apt install binutils-m68k-linux-gnu` and
`make CROSS=m68k-linux-gnu-` instead of `make setup` works too.

`VERSION` is `us` (the default) or `eu`. `asm/` holds one version at a
time, so run `make clean` before you switch.

## Layout

| path | what |
| --- | --- |
| `mbm.<v>.sha1` | the ROM's sha1, what the build has to reproduce |
| `mbm.ld` | the linker script; `asm/link.ld` fills in the objects |
| `include/macros.inc` | included by every asm file (`.opt noquick`) |
| `tools/m68k.py` | the 68000 decoder: what gas can spell, in Motorola syntax |
| `tools/test_m68k.py` | every encoding the decoder accepts, through gas and back (CI) |
| `tools/disasm.py` | the ROM to `asm/`: code found by tracing and speculation, the rest data |
| `code.<v>.txt`, `data.<v>.txt` | what runtime traces proved is code and data (generated) |
| `tools/checksum.py` | the header checksum and ROM end |
| `tools/src/mdtrace.c` | headless runs in Genesis Plus GX: coverage, screenshots, RAM dumps |
| `tools/gen_inputs.py`, `tools/inputs/` | the pad scripts: menus, battle, every stage by password |
| `tools/trace_all.sh`, `tools/coverage.py` | run them all; compare a trace with the disassembly |
| `tools/get_gpgx.sh` | Genesis Plus GX's libretro core with CPU hooks (a dev tool, not in the port) |
| `tools/get_binutils.sh` | builds GNU binutils 2.47 for m68k-elf (sha256-pinned) |
| `docs/` | LAYOUT (the ROM, the build, finding code), ROADMAP, NETPLAY |

## Credits

- The structure and the rules follow the author's Blast Corps and
  Bomberman 64 decompilations.
- [Genesis Plus GX](https://github.com/libretro/Genesis-Plus-GX) (Eke-Eke
  and others), fetched and built by `make setup` for tracing only. Its
  license forbids commercial use; nothing of it is in this repository or
  the port.
- The normal game's passwords are Sega Retro's list.

Licensed AGPL-3.0-or-later (LICENSE). That covers this project's own code,
not the game or anything extracted from it.
