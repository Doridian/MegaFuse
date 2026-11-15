//! The browser build's start (docs/web.md): the page (`web/index.html`)
//! asks for the ROM and calls [`start`] with its bytes; the game then runs
//! in the page's canvas as natively in a window.

use wasm_bindgen::prelude::*;

/// The module's start: a panic shows on the page.
pub fn main() {
    std::panic::set_hook(Box::new(|info| platform::web::show_error(&info.to_string())));
}

/// Plays `rom` (the cartridge's bytes) in the page's canvas.
#[wasm_bindgen]
pub fn start(rom: Vec<u8>) {
    let app = crate::window::GameApp::new(rom, false);
    if let Err(e) = platform::run("MegaFuse", (320 * 3, 224 * 3), app) {
        platform::web::show_error(&e.to_string());
    }
}
