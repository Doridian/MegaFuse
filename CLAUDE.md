# MegaFuse

MegaFuse is a decompilation of Mega Bomberman (Hudson, Mega Drive, 1994):
a matching disassembly, aimed at a Rust port (Linux, Windows, macOS, WebAssembly) with online multiplayer
over WebRTC. The models are `../DozerEncore` and `../Fuse64` (same
goals, same rules; look at their tools before writing a new one) and, for
the Rust port and its netplay, `../VacationDestination` (winit, wgpu, cpal,
`crates/net` with matchbox). docs/ROADMAP.md has the phases: 0 and 1 are
done (both ROMs rebuild from `asm/`, sha1-checked); in phase 2 the runtime
trace, pointer symbols and the SHIFT build with its check are in place
(docs/LAYOUT.md, "Runtime traces", "Pointers", "SHIFT").

The game is handwritten 68000 asm, not compiled C. Nothing gets compiled to
match: the asm is the source, and the port is a translation of it into
Rust (ROADMAP, phase 4). The port is Rust, not C (decided at the start).

## Build

```
make setup                   # binutils (m68k-elf), Genesis Plus GX, tools/mdtrace
make extract                 # tools/disasm.py: baserom.us.bin, baserom.eu.bin -> asm/
make VERSION=us              # assemble, link, sha1-checked
make test                    # tools/test_m68k.py: the decoder against gas
tools/trace_all.sh us        # headless runs of tools/inputs/, regenerates code/data/longs lists
make VERSION=us shift-check  # SHIFT=1 build run against the matching one (tools/shift_check.py)
tools/find_abs.py us         # what may still be a ROM address written as a number
tools/vermap.py eu           # where each us address is in eu (eu takes us's names)
tools/find_exits.py [STAGE]  # plans that take stages to their exit (tools/exits.json)
tools/unrun.py               # docs/UNRUN.md: what no script runs, by where it starts
```

Screenshots of a run (`tools/mdtrace --shot`) tiled with `tools/sheet.py`
are the quickest way to see where a script got to. Run emulator batches
under `nice`.

`VERSION` is `us` or `eu`; `asm/` is one source for both (`.if VERSION_EU`
where they differ, names from us through `tools/vermap.py`), built into
`build/<v>`. `baserom.<v>.bin` are copies of the no-intro files in this
directory.

## Rules

- **The sha1 is the arbiter.** Every change keeps `make` passing for us
  and eu.
- **asm/ is generated.** `tools/disasm.py` writes it; change the tool (or
  `code.<v>.txt`, addresses known to be code), not the asm. Hand-written
  asm comes later, once the disassembly is complete and named.
- **The decoder only emits what gas reassembles exactly.** Any change to
  `tools/m68k.py` passes `make test` (every encoding it accepts, through
  gas and back). An encoding gas spells differently is `raw` (written as
  `.short`) or refused (data). gas needs `.opt noquick` and `(0.w,an)`
  (docs/LAYOUT.md, "gas details").
- **Code found statically can be wrong.** Speculation (docs/LAYOUT.md,
  "Finding code") is a heuristic; the runtime trace of phase 2 is the
  evidence. Matching doesn't depend on it, shifting will.
- **Don't guess names.** Labels are `func_`/`loc_`/`D_`/`jtbl_` plus the
  address (us's addresses in both versions, once there is a us↔eu map).
- **Traces are evidence, speculation is a guess.** `code.<v>.txt`,
  `data.<v>.txt` and `longs.<v>.txt` come from `tools/trace_all.sh <v>` (never hand-edited);
  rerun it after changing `tools/inputs/` (`tools/gen_inputs.py`) and
  commit the lists. Genesis Plus GX (`tools/get_gpgx.sh`) is a dev tool
  only: nothing of it goes into the repo or the port.
- **shift-check passes** for us and eu (it does now): a change to the
  disassembly keeps the shifted build running like the original. The
  port doesn't reproduce the game's bugs, but the shifted build must
  (READS_PAST, BIASED in tools/disasm.py).
- New dependencies (downloaded code, crates, libraries) are the user's
  call: ask first.
- Drafts worth keeping go in `scratch/` (git-ignored), not `/tmp`.
