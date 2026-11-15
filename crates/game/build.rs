//! The translated game (`src/translated/`, `make port`) is generated from
//! the ROM and isn't in the repository: without it (CI, a fresh clone)
//! the crate builds against `src/stub.rs`, which panics if run.

fn main() {
    println!("cargo::rustc-check-cfg=cfg(translated)");
    println!("cargo::rerun-if-changed=src/translated/mod.rs");
    if std::path::Path::new("src/translated/mod.rs").exists() {
        println!("cargo::rustc-cfg=translated");
    }
}
