# The browser build

The port runs in a browser as WebAssembly (`wasm32-unknown-unknown`),
without Emscripten, as VacationDestination's does: winit draws into a
`<canvas>`, wgpu renders with WebGPU or WebGL2, cpal plays through
WebAudio, and wasm-bindgen glues it to the page. The game is the same
code as natively, and runs the same way (docs/PORT.md, "Playing it"):
the window's event loop runs the game ahead into a queue of finished
frames and shows them at the console's pace, so nothing waits.

## Building

Needs the `wasm32-unknown-unknown` target and the `wasm-bindgen` CLI in
the version the crates pin (`wasm-bindgen = "=0.2.130"` in
`crates/megafuse/Cargo.toml` and `crates/platform/Cargo.toml`; the CLI
refuses a module from another version),:

```sh
cargo install wasm-bindgen-cli --version 0.2.130
tools/build-web.sh            # release build into web/
tools/build-web.sh --debug    # quicker to build, slow to play
```

`web/` (ignored by git) gets `index.html` (from
`crates/megafuse/web/index.html`), `megafuse.js` and `megafuse_bg.wasm`.

## Playing

Any static HTTP server does; the module can't load from `file://`:

```sh
python3 -m http.server -d web 8000    # then open http://localhost:8000/
```

The page asks for the game's ROM (Mega Bomberman, USA), keeps it in the
browser's IndexedDB for the next time, and **Start** hands it to the
module (`start`, crates/megafuse/src/web.rs). The ROM never leaves the
browser. The keys are the native ones (docs/PORT.md); the sound starts
with the first key press (the browser's autoplay rules).

Network play isn't in the browser build yet (its session waits for the
network, which the browser can't).
