# ROM layout

Both ROMs are 1 MiB (`0x100000`), big-endian, no mapper, no SRAM. The
header (`0x100`) reads `SEGA MEGA DRIVE`, `(C)SEGA 1994.DEC`,
`MEGA BOMBER MAN`, `GM MK-1573-00`, I/O `J4` (3-button pad and Team
Player), region `JUE` in both. The checksum at `0x18E` is the word sum from
`0x200` to the end (`tools/checksum.py`); the startup code checks it and
shows a red screen when it is wrong.

| range | what |
| --- | --- |
| `0x000000`-`0x0000FF` | 68000 vectors: SP `0xFFFF00`, reset `0x7AAE`, level 6 (VBlank) `0x7C5A` (eu `0x7C54`), level 4 (HBlank) and external `0x260`, the errors `0x200`-`0x248` |
| `0x000100`-`0x0001FF` | cartridge header |
| `0x000200`-`~0x01E800` | code, with tables and small data between functions |
| `~0x01E800`-`0x01FFFF` | zeros |
| `0x020000`-`0x03FFFF` | zeros |
| `0x040000`-`0x0FF91D` | data: graphics, maps, sound (not yet identified) |
| `0x0FF91E`-`0x0FFFFF` | zeros |

The game is handwritten 68000 assembly (Hudson Soft): no `link`/`unlk`
frames, arguments in registers, RAM addressed with absolute short
addressing (`(0x80C3).w` is `0xFF80C3`), and work RAM from `0xFF8000` up.
The startup code from `0x7AAE` is Sega's standard `ICD_BLK4` init.

## US and EU

The two ROMs are the same program. The data from `0x20000` on is
identical, at the same offsets. The code differs in two places only:

- `0x7C4A`: us has `movem.l (0).w,sp` (reload the stack pointer from the
  vector) before enabling interrupts; eu doesn't, so everything after it
  is 6 bytes earlier in eu.
- After the code (`0x1E756` in us), us has 14 more bytes, a copy of the
  last 14 (`9F06 023C 00FE 4E75 003C 0001 4E75`, the tail of the
  functions before): left over in the build. eu has zeros there.

The other ~83,000 differing bytes are absolute addresses moved by the
first (and the checksum). The header doesn't name a region, so whether eu
behaves differently on a 50 Hz console is something to find in the code.

**The version map** (`tools/vermap.py eu` prints it) is found from the
two disassemblies: their instructions, numbers masked, aligned with
difflib, and runs at a fixed distance made ranges. It comes out as
`0-0x7C4A +0`, `0x7C50-0x1E756 -6`, and the data `+0`. Through it:

- **eu takes us's names**: a label is named after the address of the
  same thing in us (`Disasm.tag`; one with no counterpart in us would be
  `D_XXXXXX_eu`; there are none), and so are the files;
- **numbers**: a long immediate that stays the same in both versions
  while what it would point at moves is a number
  (`vermap.numbers`): `move.l #0xFE00,(0x8A5E).w` sets the counter at
  `0x8A60` to 0xFE, it doesn't point at 0xFE00.

**One source.** `tools/disasm.py` analyses both versions (a second
each), writes each with us's names, and merges the two texts file by
file: what they share as it is, the rest under `.if VERSION_EU` (make
passes `--defsym VERSION_EU=0/1`). Three places differ: the header's
checksum, the `movem.l`, and us's stray bytes (`01E756.s`, empty in eu).
`shift_pad`s go where they fit in both versions. `tools/shift_check.py`
pairs the two builds' symbols by name, so it doesn't read addresses out
of names.

## The matching build

`tools/disasm.py` writes `asm/`, one source for us and eu ("US and EU"):

- `asm/header.s`: the vectors (as symbols) and the header bytes.
- `asm/NNNNNN.s`: the ROM from `0xNNNNNN` on. A file is cut where 256 or
  more bytes of data start, and where code starts again after that, but
  never inside the span of a PC-relative reference (an 8-bit displacement
  can't cross objects, and `.L` labels are file-local).
- `asm/link.ld`: the objects in order, which the Makefile puts into
  `mbm.ld`, and `. = 0x040000;` before the data: the zeros between the
  code and the data aren't in any file, so the data stays at its address
  in a SHIFT build; `asm/symbols.<v>.txt`: every global label and its
  address in that version.

Code is written as instructions with labels, and everything else as
`.byte` (with `.space` for long zero runs). The build matches however much
is found to be code: a word that is not found stays data, and its bytes are
the same either way. What finding code decides is how much is readable and
symbolic, which the SHIFT build (below) needs.

### gas details

The output assembles with `m68k-elf-as -m68000 --register-prefix-optional
-no-pad-sections`, linked with `SUBALIGN(2)` (objects start at any even
address).

- `.opt noquick` (in `include/macros.inc`): without it gas makes
  `move.l #0,d0` a `moveq` and `add.w #1,d0` an `addq`.
- `(0.w,a0)`: gas makes `(0,a0)` into `(a0)`; the size keeps the word.
- A few real encodings have no gas spelling: `add`/`sub`/`cmp`/`and`/`or`
  with an immediate source in the `<ea>,Dn` form (gas picks `addi` and the
  like). The decoder marks them `raw`, and they are written as `.short`
  with the instruction in a comment.
- The decoder refuses what gas would write differently and the 68000
  ignores: index scale bits, a byte immediate's high byte, an empty
  `movem` list, a PC index to its own extension word. Those stay data.

`tools/test_m68k.py` (`make test`, run by CI) decodes every first word
with five sets of extension words, assembles everything the decoder
accepts, and checks it comes back byte for byte: 209,388 instructions.

## Finding code

Tracing from the vectors alone finds 5 KB: the main loop calls a scene
routine through a pointer in RAM (`(0x80E0).w`), and most of the game hangs
off pointers like that. `tools/disasm.py` adds, in this order:

1. **Fixed indirect flow** while tracing: `jmp`/`jsr (d8,pc,Xn)` into a
   table of 4-byte entries (`bra.w`, or `nop; rts`); an address loaded
   into an address register (`lea`, `movea.l #`) and called in the same
   block; a long stored (`move.l #x,(slot)`) into a RAM slot that some
   block loads and jumps through.
2. **Speculation** (`Disasm.speculate`): a candidate is traced in a
   journal, and kept only if everything it reaches decodes, stays below the
   end of the code area (`0x20000`, the first 16 KB block with no `rts`),
   overlaps no known code, holds no `trap`/`illegal`/`stop`/`reset`/raw
   instruction or `0000 0000`, and branches only to even addresses in the
   code area. Otherwise it is rolled back. Candidates, strongest first:
   - addresses the code names as addresses: absolute and PC-relative
     operands, `movea.l #x`, `move.l #x` to memory or the stack;
   - the entries of a table the code calls through
     (`movea.l (tbl,pc,dn.w),an` then `jsr`/`jmp (an)`,
     `Disasm.call_table`), up to the first target after the table or the
     first entry that isn't code; also `lea (x).l,a0; movea.l
     (0,a0,dn.w),a0; jsr (a0)`. The base may be biased: `func_015FAE`'s
     opcodes start at 1, so its base is the `bra.s` before the table, and
     up to 2 leading slots that can't be a code address are skipped. A
     biased base inside an instruction is written `table-off` (0x22B8's
     table, based at 0x22B4). Its
     handlers are 3 or 4 instructions, too short for the next kind;
   - other long immediates and longs in the data between code (pointer
     tables), which must reach at least 6 instructions. Short closures
     are rejected because the tail of a real instruction often decodes as
     code: `4CDF 7070` (`movem.l (sp)+,...`) holds `moveq #0x70,d0; rts`;
   - addresses right after a `rts`/`jmp`/`bra` that nothing claims,
     with the same minimum;
   - last, the same gaps however short, if they decode into closures
     that fill them exactly: each ends in a jump or a return, the last
     right where the known code starts, nothing reaches outside the gap
     and the code names nothing in it as data (`Disasm.fill_gap`). These
     are dead functions nothing calls (0x868E and 0x86A4, siblings of
     `func_00867E`), the `rte` after each exception handler's loop, and
     a stray `rts` here and there. Gaps that run into the code after
     without a return (`subq.l #8,d0; sf d0; ...` before `func_00CB2C`)
     stay data.

3. **Evidence from runtime traces**, which outranks both:
   `code.<v>.txt` (entry points a trace ran that nothing reaches
   directly: indirect targets) are traced like the vectors, before any
   speculation, and `data.<v>.txt` (ranges of the code area a trace read as
   data) are off limits to speculation. Both are regenerated from the
   traces (below), not edited.

The result in us: 1,634 functions (counting entries only pointers name),
110 KB of code, of which the traces ran 22,138 instructions (82%); every instruction that ran is in the
disassembly, and nothing the disassembly calls code was read as data
except the block below. What is left of the code area is mostly tables:
pointer tables, palettes, sine and step tables, text.

Open: `func_00C152` is likely an animation table (it is stored with
`move.l #x,(0xC,a6)`, like the three `D_` tables that store holds; the
`ending` trace turned `D_015DD0` from code into data), but no trace has
read it. From `0x1906E` on, code that runs is also read word by word (the
bosses' palette cycle reads its colours there, "SHIFT"), and `0x81C8` stores code
addresses at `0xFFF000`, which may be code run from RAM.

## Pointers

What `tools/disasm.py` writes as a symbol rather than a number
(`Disasm.collect_refs`):

- absolute and PC-relative operands in the ROM, as before;
- long immediates used as addresses: `movea.l #x`, `move.l #x` to memory
  or the stack, `addi.l`/`adda.l #x` (a table's base plus an index), and
  any long immediate that is a function's entry (`move.l #func,d0` hands
  a callback over). `move.l #x,dn`, `cmpi.l`, `andi.l` stay numbers;
- **pointer longs in data** (`.long label`): a long a trace read as a long
  (`longs.<v>.txt`) whose value can be pointed at (an instruction start or
  data in the code area, even there; or the data from 0x40000; not the
  zeros between), and the rest of its table: its neighbours 4 bytes on, or
  at the distance to another such long within 0x20 (records: 0x3704 is 10
  bytes each, two pointers), until something the code or a pointer names
  (a table usually ends where what it points at starts). A neighbour no
  trace read must point into the data or at something already named:
  palette scripts hold words like `0000 6D00`;
- a long the code reads off a `lea` base in the same block
  (`lea (D_00DA50).l,a0` then `move.l (0x4,a0),...`) whose value is a
  function's entry: a static read, so a value that is only a number
  stays one (`Disasm.read_longs`);
- **longs that moved between the versions** (`vermap.moved_longs`):
  a long whose value in eu is where the version map puts its value in
  us, and differs. A number doesn't move with the code, so this is
  evidence like a trace, at any alignment (`0x1B7BA`'s records, the
  10-byte records at `0x3704` with pointers at odd offsets, operands of
  dead code still written as bytes). One such long stands alone: it
  doesn't extend to its neighbours. Every byte of the code area that
  differs between the versions is now an instruction or a pointer; what
  points below `0x7C4A` is where it is in both, so this can't see it;
- a pointer's target that starts a table of pointers itself: where the
  pointer's neighbours point at such tables (`D_00B534`'s seventh,
  `D_00B728`), or where two pointers run up to it (a record points
  into the middle of `D_00FEB0`);
- a displacement off an address register that lands on data a trace read
  (`move.b (0x17A6,a0),d0` with an index in `a0`);
- `BIASED`: a table's address minus its smallest index, which only a
  trace shows (`lea (D_00EFB8-0x86).l,a0`);
- **longs in bytecode scripts** (`SCRIPTS`, `Disasm.walk_scripts`): a
  script is read a byte at a time through a pointer in RAM, so its longs
  sit wherever the ops put them. The walk starts at the long immediates
  stored into that pointer and decodes op by op (`SCRIPT_OPS`, checked
  against what each handler in the interpreter's table reads). A call
  op's callee may read on: what it reads is found from its code
  (`Disasm.script_reads`: every path to its return reads the same sizes,
  calls followed), so `func_0174BA` takes a palette address after it. A
  script's call targets are speculated as code (`0x16D8A`). One
  interpreter so far, `func_015FAE` (`0xAD5E`): its one script,
  `D_016FFA`, holds 51 pointers; the `ending` trace runs all of it. Its
  pointer ends one past the script, so no object starts there (no pad in
  a SHIFT build);
- **tables of records** (`RECORDS`, `Disasm.record_fields`): a table of
  record pointers (it ends where the first record starts) and what the
  code does with each long of a record: a pointer, a call table (its
  entries speculated as code, like any table the code calls through) or
  a function. `D_00A04A`, by `(0x953A).w`, is copied to
  `0x9E78`-`0x9E87` by `func_001030`: `func_005336` calls through
  `0x9E80`, `func_000E98` jumps to `0x9E84`. This reaches what no trace
  has run (the tenth record, `D_00A41E`, and its table `D_00A442`).

What a target is called: `D_` in data, `func_` for an instruction start
nothing branches to. Not handled yet: the lists a `D_00A04A` record's
first long points at (`D_00A0AA`: a count, `FF`, then `00 0D 8F EC 5A FF`
entries, a long at an odd offset into the data; nothing reads them by
name, so their format is a guess), word pointers, and the data's own pointers. `tools/find_abs.py` lists what may still be an address written
as a number.

The tracer didn't see PC-relative reads at first (Musashi reads them like
instruction words, past the hook); `tools/get_gpgx.sh` routes them through
it, so `move.l (tbl,pc,d0.w),a0` tables are evidence like any other.

## SHIFT

`make SHIFT=1` builds into `build/<v>-shift/` with 0x10 zero bytes after
each object in the code and 2 before most functions in it (`shift_pad`,
include/macros.inc), so everything from 0x200 on moves, by up to a few
KB. Pads go only where nothing falls through (after a `rts`, `jmp` or
`bra`), never inside a jump table, and only where every displacement
still fits (`Writer.plan_pads`): none under an 8-bit one, and a 16-bit one
(`bra.w`, `(d16,pc)`, a ROM address written `(xxx).w`, which must stay
below 0x8000) takes as many as its slack allows. The data from 0x40000
doesn't move (link.ld); its pointers into the code are symbols, but its
own aren't all known yet, and the Z80 driver addresses samples by 32 KB
bank.

`make shift-check` (`tools/shift_check.py`) runs the shifted and the
matching ROM headless over every script in tools/inputs/ for 9000 frames
and compares, every 300 frames, the screen (exactly) and work RAM, with
ROM addresses translated: a long or word that points at `label + off` in
the original must point at the label's new address `+ off`. The map is
the shifted ELF's symbol table, since labels are named after their
original address. On a failure, `--first` finds the first frame and the
bytes, and `--pcs NAME:FRAME` the first instruction or ROM read that
differs (mdtrace's `--pcs`); `mdtrace --watch ADDR` shows who reads and
writes a byte. us and eu pass. What it tolerates, and why:

- the stack (down to the lowest the stack pointer has been): return
  addresses and saved registers, and pieces of old ones;
- the stale half of an address whose other half was overwritten
  (`0xFF80C8` holds a pointer, then another variable's word);
- `PAST_TABLE`: the game reads `D_000B7E[0xFF]` (slot 0xFF is "none"),
  which is code, and keeps it in `a5`;
- the bosses' palette cycle (`func_00AA4C`) takes its colours from code
  (`0x19068` in eu, `jsr (0x19144).l` among them), so the boss scripts are
  compared only up to the fight.

The SHIFT build keeps the game's bugs where a pad would change them:
`READS_PAST` in tools/disasm.py (`0xFE` from `D_003FA2` read as +254
instead of -2, into an opcode byte that moves an enemy). The port fixes
such bugs instead (docs/ROADMAP.md, phase 4).

## Runtime traces

`tools/mdtrace` (`tools/src/mdtrace.c`) runs a ROM headless in Genesis
Plus GX's libretro core, built with its CPU hooks by `tools/get_gpgx.sh`
(`make setup`; a pinned commit, a development tool only, never in the
port). It plays a pad script, records for every ROM byte whether an
instruction started there and whether it was read as data (and at what
width), and takes screenshots and RAM dumps. The boot checksum's
`add.w (a0)+,d0` (`0x7BE2`) reads the whole ROM, so its reads are ignored
(`--ignore-reads`).

`tools/gen_inputs.py` writes the scripts in `tools/inputs/` (seeded, so
runs repeat): `normal` (random play from stage 1-1), `battle` (two pads)
and `battle4` (four, on the Team Player), cycling through the setup screens
and random matches, `password`, `idle`, and `stage-*`: one per password,
every stage, boss and the final area (Sega Retro's list, all checked in
the emulator), and `ending`. A script line `FRAME =ADDR VALUE` writes a
byte, word or long of work RAM before FRAME runs; `ending` uses that for
what random play can't do: from the final area's password it loads room 5
(`0x953E`/`0x953F`), zeroes the energy (`+0x11` in the 0x40-byte object
slots from `0xFFA432`) of the room's targets and enemies, puts the player
under the open gate, gives the player 0x7F hits, and takes the final boss
through its three forms (the orbs, a hit on slot 7, energy 0 in
`func_012CBC`), to the escape, the ending and the credits (`func_015FAE`'s
script). Only numbers are written, never ROM addresses, so the SHIFT build
runs it too. The
continue and game-over screens come up in random play already.

`exit-*`: every normal stage (not the bosses, not the final area) from
its password to its exit, the bonus stage after it and the next stage's
card. `tools/find_exits.py` worked the plans out by running the game a
step at a time and reading work RAM between runs (`tools/exits.json`,
which `tools/gen_inputs.py` turns into the scripts): Start on the screens
that wait (the stage card takes it some 100 frames after it shows), then
in each room energy 0 for object slots 1-19, and once the gate is open
(`0x9540` bit 5) the player put next to it, on a cell made floor, walking
in. The gate is either a task running `func_00C1F6` (its position at
`+0xE`/`+0x10`; it checks the player with `func_0018C4` and starts the
exit) or a door that `func_00C822` writes into the grid as 4 (into the
next room; most stages have two to four). The grid: a position `(x, y)`
is in the cell at `0xFF989C + row[y >> 4] + (x >> 4)`, the rows' offsets
being the longs at `0xFFA31A` (`func_001652`), 16 or 32 bytes a row. The
bonus stage sets `0x9540` bit 4. A script's `# frames N` line sets how
long it runs, `# compare N` how far tools/shift_check.py compares it: an
area's last stage goes on into the boss stage, whose palette cycle reads
its colours from code (below).
`tools/trace_all.sh <v>` runs them all (about half a
minute) into `build/<v>/trace/` and regenerates `code.<v>.txt`,
`data.<v>.txt` and `longs.<v>.txt` with `tools/coverage.py --write`; `tools/coverage.py`
alone reports how a trace and the disassembly disagree.
`tools/sheet.py` tiles screenshots into one picture.

There is no TAS of Mega Bomberman (TASVideos lists the us ROM, with no
movies, submissions or user files), so the passwords stand in for one. A
2011 run on YouTube (20 fps, no input display, another emulator) shows
the whole game but can't give its inputs back.

`tools/coverage.py --write` lists in `code.<v>.txt` what tracing from the
vectors doesn't reach, and also what ran that nothing that ran reaches
from there (code entered only from code that is still bytes, or a loop
into itself: `0xEEE8`); with that, every instruction that ran is in the
disassembly.
