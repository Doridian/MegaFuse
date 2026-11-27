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
mod b_control;
mod b_state;
mod b_walk;
mod c_chase;
mod c_object;
mod c_spawn;
mod d_angle;
mod d_object;
mod d_pair;
mod d_random;
mod decode;
mod e_cell;
mod e_cell_task;
mod e_follow;
mod e_placed;
mod e_placed_task;
mod f_cell_timers;
mod f_kinds;
mod f_objects;
mod f_records;
mod f_spread;
mod frame;
mod g_objects;
mod g_pads;
mod h_pad;
mod h_scroll;
mod h_search;
mod h_vdp;
mod vdp;
mod z80;
mod lookup;
mod map;
mod math;
mod memory;
mod sprite;
mod task;
mod i_pads;
mod i_palette;
mod i_queue;
mod i_load;
mod i_intro;
mod i_stage;
mod i_main;
