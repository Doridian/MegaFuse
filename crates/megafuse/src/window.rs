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
}

impl Windowed {
    pub fn new(shared: Arc<Shared>) -> Self {
        Windowed { shared, audio: Audio::new(md::sound::SAMPLE_RATE), due: None }
    }
}

impl Frontend for Windowed {
    fn frame(&mut self, picture: &[u32], width: usize, height: usize, _: &mut [u8; 0x10000], pads: &mut [u16; 4]) {
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

/// The game on a thread of its own, the window on this one.
pub fn play(rom: Vec<u8>) -> ! {
    let shared = Arc::new(Shared::default());
    let frontend = Windowed::new(Arc::clone(&shared));
    std::thread::spawn(move || {
        let md = md::Md::new(rom.clone(), Box::new(frontend));
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
