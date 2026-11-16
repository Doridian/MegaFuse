//! The browser build's start (docs/web.md): the page (`web/index.html`)
//! asks for the ROM and calls [`start`] with its bytes; the game then runs
//! in the page's canvas as natively in a window.

use wasm_bindgen::prelude::*;

/// The module's start: a panic shows on the page.
pub fn main() {
    std::panic::set_hook(Box::new(|info| {
        platform::web::show_error(&info.to_string())
    }));
}

/// Plays `rom` in the page's canvas in a network game: `room` on the
/// signalling server `signal` (`ws://` or `wss://`), hosted (for
/// `players`, 2-4) or joined; `stun`, a STUN server's URL or empty.
#[wasm_bindgen]
pub fn start_net(
    rom: Vec<u8>,
    signal: String,
    room: String,
    host: bool,
    players: u8,
    stun: String,
) {
    if let Err(e) = crate::rom::check(&rom) {
        platform::web::show_error(&e);
        return;
    }
    let ice = net::matchbox::IceServers {
        stun: (!stun.is_empty()).then_some(stun).into_iter().collect(),
        ..Default::default()
    };
    let join = crate::netplay::Join {
        room,
        host,
        players: usize::from(players),
        delay: crate::netplay::DELAY,
        signal,
        ice,
    };
    let session = match crate::netplay::connect(&join, &rom) {
        Ok(s) => s,
        Err(e) => {
            platform::web::show_error(&e);
            return;
        }
    };
    let app = crate::window::GameApp::net(rom, session);
    if let Err(e) = platform::run("MegaFuse", (320 * 3, 224 * 3), app) {
        platform::web::show_error(&e.to_string());
    }
}

/// Plays `rom` (the cartridge's bytes) in the page's canvas.
#[wasm_bindgen]
pub fn start(rom: Vec<u8>) {
    if let Err(e) = crate::rom::check(&rom) {
        platform::web::show_error(&e);
        return;
    }
    let app = crate::window::GameApp::new(rom);
    if let Err(e) = platform::run("MegaFuse", (320 * 3, 224 * 3), app) {
        platform::web::show_error(&e.to_string());
    }
}
