//! The game in a window (crates/platform): the picture shown, the sound
//! played, the keys as the pads, at the console's pace.

use std::sync::atomic::Ordering;
use std::sync::{Arc, PoisonError};
use std::time::{Duration, Instant};

use md::{Frontend, button};
use platform::{KeyCode, Shared, audio::Audio};

/// A frame: 262 lines of 3420 master clocks at 53.693175 MHz.
const FRAME: Duration = Duration::from_nanos(16_688_154);

/// The keys of each pad: up, down, left, right, A, B, C, Start.
const KEYS: [[KeyCode; 8]; 2] = [
    [
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::KeyZ,
        KeyCode::KeyX,
        KeyCode::KeyC,
        KeyCode::Enter,
    ],
    [KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD, KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH, KeyCode::Tab],
];
const BUTTONS: [u16; 8] =
    [button::UP, button::DOWN, button::LEFT, button::RIGHT, button::A, button::B, button::C, button::START];

pub struct Windowed {
    shared: Arc<Shared>,
    audio: Audio,
    /// When the next frame is due.
    due: Option<Instant>,
    /// In a network game: the pads come from the session (this keyboard's
    /// first pad is this peer's).
    net: Option<crate::netplay::Pads>,
}

impl Windowed {
    pub fn new(shared: Arc<Shared>) -> Self {
        Windowed { shared, audio: Audio::new(md::sound::SAMPLE_RATE), due: None, net: None }
    }
}

impl Frontend for Windowed {
    fn frame(&mut self, frame: u64, picture: &[u32], width: usize, height: usize, ram: &mut [u8; 0x10000], pads: &mut [u16; 4]) {
        if self.shared.quit.load(Ordering::Relaxed) {
            std::process::exit(0);
        }
        {
            let mut p = self.shared.picture.lock().unwrap_or_else(PoisonError::into_inner);
            p.rgba.clear();
            for y in 0..height {
                for &c in &picture[y * md::vdp::WIDTH..y * md::vdp::WIDTH + width] {
                    p.rgba.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8, 255]);
                }
            }
            p.width = width as u32;
            p.height = height as u32;
            p.serial += 1;
        }
        for (pad, keys) in pads.iter_mut().zip(KEYS) {
            *pad = keys.iter().zip(BUTTONS).filter(|&(&k, _)| self.shared.held(k)).fold(0, |m, (_, b)| m | b);
        }
        if let Some(net) = &mut self.net {
            match net(frame, pads[0], ram) {
                Ok(p) => *pads = p,
                Err(e) => {
                    eprintln!("{e}");
                    self.shared.quit.store(true, Ordering::Relaxed);
                    std::process::exit(1);
                }
            }
        }
        // the console's pace: a frame every 16.69 ms (a late frame isn't
        // made up for long)
        let now = Instant::now();
        let due = self.due.map_or(now, |d| d + FRAME);
        if due > now {
            std::thread::sleep(due - now);
        }
        self.due = Some(if now > due + FRAME * 4 { now } else { due });
    }

    fn audio(&mut self, samples: &[f32]) {
        self.audio.push(samples);
    }
}

/// The game on a thread of its own, the window on this one; in a network
/// game, once the players are there.
pub fn play(rom: Vec<u8>, join: Option<crate::netplay::Join>) -> ! {
    let shared = Arc::new(Shared::default());
    let mut frontend = Windowed::new(Arc::clone(&shared));
    let quit = Arc::clone(&shared);
    std::thread::spawn(move || {
        let mut multitap = false;
        if let Some(j) = &join {
            match crate::netplay::connect(j, &rom) {
                Ok((pads, _, players)) => {
                    frontend.net = Some(pads);
                    multitap = players > 2;
                }
                Err(e) => {
                    eprintln!("{e}");
                    quit.quit.store(true, Ordering::Relaxed);
                    return;
                }
            }
        }
        let mut md = md::Md::new(rom.clone(), Box::new(frontend));
        if multitap {
            md.plug_teamplayer();
        }
        let mut game = game::Game::new(rom, Box::new(md));
        game.run();
    });
    let result = platform::run("MegaFuse", (320 * 3, 224 * 3), shared);
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
    std::process::exit(0)
}
