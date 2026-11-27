//! Routines rewritten by hand (docs/PORT.md, "Rewriting"): readable Rust
//! in place of tools/recomp's translation, without the console's timing
//! (no `insn`).  Each keeps the translation's signature, which the
//! generated callers use (tools/recomp writes the generated version to
//! `translated/rewritten.txt`), and is checked against the original call
//! by call (tools/routine_check.py).

mod a_cells;
mod a_motion;
mod a_objects;
mod a_random;
mod anim;
mod decode;
mod frame;
mod vdp;
mod z80;
mod lookup;
mod map;
mod math;
mod memory;
mod sprite;
mod task;
