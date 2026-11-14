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
- `0x1A444`: us has a `subi.w`/`add.w` pair eu doesn't.

The other ~83,000 differing bytes are absolute addresses moved by those
shifts (and the checksum). The header doesn't name a region, so whether eu
behaves differently on a 50 Hz console is something to find in the code.
A us↔eu address map (B64's `vermap`) will be simple: two insertions.

## The matching build

`tools/disasm.py VERSION` writes `asm/`:

- `asm/header.s`: the vectors (as symbols) and the header bytes.
- `asm/NNNNNN.s`: the ROM from `0xNNNNNN` on. A file is cut where 256 or
  more bytes of data start, and where code starts again after that, but
  never inside the span of a PC-relative reference (an 8-bit displacement
  can't cross objects, and `.L` labels are file-local).
- `asm/link.ld`: the objects in order, which the Makefile puts into
  `mbm.ld`; `asm/symbols.txt`: every global label.

Code is written as instructions with labels, and everything else as
`.byte` (with `.space` for long zero runs). The build matches however much
is found to be code: a word that is not found stays data, and its bytes are
the same either way. What finding code decides is how much is readable and
symbolic, which the SHIFT build (ROADMAP, phase 2) will need.

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
   - other long immediates and longs in the data between code (pointer
     tables), which must reach at least 6 instructions. Short closures
     are rejected because the tail of a real instruction often decodes as
     code: `4CDF 7070` (`movem.l (sp)+,...`) holds `moveq #0x70,d0; rts`;
   - addresses right after a `rts`/`jmp`/`bra` that nothing claims,
     with the same minimum.

The result in us: 1,399 functions, 109 KB of code, about 21 KB of the code
area left as data. What is left is mostly tables: pointer tables (to data
records), palettes, sine and step tables, text, and a few blocks that look
like code copied into RAM (`0x81C8` stores code addresses at
`0xFFF000`). Speculation can be wrong in both directions. A runtime trace
in an emulator (ROADMAP, phase 2) will settle it.
