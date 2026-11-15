# The port

The game in Rust (docs/ROADMAP.md, phases 4-6). The 68000 code is
translated by `tools/recomp/` into **idiomatic, structured Rust**: one
function per routine, `if`/`loop`/`match` control flow, registers as
typed locals, arguments and return values, calls as Rust calls. It is not
an emulator with the decoding done ahead of time (no register file, no
program counter, no dispatch loop), and it is checked against the original
running in Genesis Plus GX, frame by frame (RAM and screens), as the SHIFT
build is.

The translated Rust is generated from the ROM, like `asm/`, so it is not
in the repository: `make port` makes it. The hand-written parts (the Mega
Drive hardware the game drives, the platform) are.

## Translation

`tools/recomp/` works on tools/disasm.py's analysis, in passes:

1. **Routines** (`routines.py`): an entry and what it reaches without
   passing another entry. Entries: the functions, the traces' code
   entries, jump table entries, what pointers and `move.l #code` name, and
   then, until no instruction belongs to two routines, each shared
   instruction one of them reaches from outside the other. Falling
   through or jumping into another routine is a tail call. us: 1,858
   routines, median 9 instructions, the largest 155.
2. **The stack** (`stack.py`): sp's depth through each routine, statically.
   Where it is the same wherever paths meet and 0 at every `rts`, pushes
   and pops (`movem` saves and restores) become locals and what a caller
   pushes before a call becomes the callee's arguments (`move.l #f,-(sp);
   move.b #x,-(sp); jsr func_0003E8` creates a task running `f`). 1,842
   routines are like that; the others are the patterns below.
3. **Control flow**: each routine's graph into nested Rust blocks and
   loops (the dominator-tree method WebAssembly compilers use, which maps
   onto Rust's labeled `loop`/`break`).
4. **Registers and flags**: which registers and condition flags each
   routine reads before writing and which of what it writes its callers
   use: its arguments and return values. A flag a routine returns (`ori
   #1,ccr; rts`, then `bcs` in the caller) becomes a `bool`.
5. **Emission**: Rust for each instruction on those locals; memory
   through accessors (work RAM, the ROM, the hardware), which become typed
   state as each part of the game is understood.

### Patterns that need more than a call

- **The task system.** Tasks are records with a routine called once a
  frame. `func_00053E(f)` sets the running task's next routine;
  `func_000522`, called in the middle of a routine, makes the rest of that
  routine (from its return address) the next one and returns past its
  caller to the scheduler: a yield. The translator makes each such return
  address a routine of its own and the call a `return` that sets the
  continuation.
- **Returning from the caller.** `addq.l #4,sp; rts` (`func_001BB4`,
  `func_017950`) or `movem.l (sp)+,d0; rts` (`func_00D554`) drops the
  caller's return address: the callee returns a flag and the caller
  returns when it is set.
- **Waiting.** Most of the game runs in the tasks, once a frame. Some code
  waits in a loop: for the hardware (the Z80's bus, the VDP's DMA: done
  at once in the port), for a raster line (the HV counter: a hook at that
  line) or for a RAM flag the vertical interrupt sets (`0x805A` in the main
  loop): there the game waits for the next frame.
