//! The game in a window (crates/platform), natively and in the browser:
//! the game runs ahead into a queue of finished frames (pictures and
//! sound), which the window shows at the console's pace.  When the queue
//! is empty the next step runs ([`game::Game::step`]; first the boot,
//! some 155 frames at once): nothing waits, so the same code runs in the
//! browser, which can't.  The keys are the pads (arrows, Z X C, Enter;
//! W A S D, F G H, Tab).

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};

use md::{Frontend, button};
use platform::audio::Audio;
use platform::{App, Duration, Instant, KeyCode, Picture};

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

/// A finished frame: its picture (none: the same as the frame before's)
/// and its sound.
struct Frame {
    rgba: Option<Vec<u8>>,
    width: u32,
    height: u32,
    samples: Vec<f32>,
}

/// What the game's frontend and the window share.
#[derive(Default)]
struct Shared {
    queue: VecDeque<Frame>,
    keys: HashSet<KeyCode>,
}

/// The game's frontend: each frame into the queue, the pads from the
/// keys (or, in a network game, from the session).
struct Recorder {
    shared: Arc<Mutex<Shared>>,
    samples: Vec<f32>,
    last: Vec<u8>,
    #[cfg(not(target_arch = "wasm32"))]
    net: Option<crate::netplay::Pads>,
}

impl Frontend for Recorder {
    fn frame(&mut self, _frame: u64, picture: &[u32], width: usize, height: usize, _ram: &mut [u8; 0x10000], pads: &mut [u16; 4]) {
        let mut rgba = Vec::with_capacity(width * height * 4);
        for y in 0..height {
            for &c in &picture[y * md::vdp::WIDTH..y * md::vdp::WIDTH + width] {
                rgba.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8, 255]);
            }
        }
        let changed = rgba != self.last;
        if changed {
            self.last.clone_from(&rgba);
        }
        let mut s = self.shared.lock().unwrap_or_else(PoisonError::into_inner);
        s.queue.push_back(Frame {
            rgba: changed.then_some(rgba),
            width: width as u32,
            height: height as u32,
            samples: std::mem::take(&mut self.samples),
        });
        for (pad, keys) in pads.iter_mut().zip(KEYS) {
            *pad = keys.iter().zip(BUTTONS).filter(|&(k, _)| s.keys.contains(k)).fold(0, |m, (_, b)| m | b);
        }
        drop(s);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(net) = &mut self.net {
            match net(_frame, pads[0], _ram) {
                Ok(p) => *pads = p,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
    }

    fn audio(&mut self, samples: &[f32]) {
        self.samples.extend_from_slice(samples);
    }
}

/// The window's app: the game, the queue, the frame shown.
pub struct GameApp {
    game: game::Game,
    shared: Arc<Mutex<Shared>>,
    audio: Audio,
    booted: bool,
    /// The picture shown and its number.
    shown: Vec<u8>,
    size: (u32, u32),
    serial: u64,
    /// When the next frame is due.
    due: Option<Instant>,
}

impl GameApp {
    /// The game on `rom`; `multitap`: the Team Player in port A.
    pub fn new(rom: Vec<u8>, multitap: bool, #[cfg(not(target_arch = "wasm32"))] net: Option<crate::netplay::Pads>) -> Self {
        let shared = Arc::new(Mutex::new(Shared::default()));
        let recorder = Recorder {
            shared: Arc::clone(&shared),
            samples: Vec::new(),
            last: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            net,
        };
        let mut md = md::Md::new(rom.clone(), Box::new(recorder));
        if multitap {
            md.plug_teamplayer();
        }
        GameApp {
            game: game::Game::new(rom, Box::new(md)),
            shared,
            audio: Audio::new(md::sound::SAMPLE_RATE),
            booted: false,
            shown: Vec::new(),
            size: (0, 0),
            serial: 0,
            due: None,
        }
    }

    /// The next frame: from the queue, the game run on when it's empty.
    fn next_frame(&mut self) -> Frame {
        loop {
            if let Some(f) = self.shared.lock().unwrap_or_else(PoisonError::into_inner).queue.pop_front() {
                return f;
            }
            if self.booted {
                self.game.step();
            } else {
                self.game.boot();
                self.booted = true;
            }
        }
    }
}

impl App for GameApp {
    fn update(&mut self) -> Instant {
        let now = Instant::now();
        let mut due = self.due.unwrap_or(now);
        // far behind (a pause, a slow machine): start again from now
        if now > due + FRAME * 8 {
            due = now;
        }
        while now >= due {
            let f = self.next_frame();
            if let Some(rgba) = f.rgba {
                self.shown = rgba;
            }
            self.size = (f.width, f.height);
            self.audio.push(&f.samples);
            self.serial += 1;
            due += FRAME;
        }
        self.due = Some(due);
        due
    }

    fn picture(&self) -> Option<(Picture<'_>, u64)> {
        let (width, height) = self.size;
        (width > 0 && self.shown.len() >= (width * height * 4) as usize)
            .then(|| (Picture { rgba: &self.shown, width, height }, self.serial))
    }

    fn key(&mut self, code: KeyCode, down: bool) {
        let mut s = self.shared.lock().unwrap_or_else(PoisonError::into_inner);
        if down {
            s.keys.insert(code);
        } else {
            s.keys.remove(&code);
        }
    }

    fn unfocused(&mut self) {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner).keys.clear();
    }
}

/// Plays `rom` in a window; natively, in a network game once the players
/// are there.
#[cfg(not(target_arch = "wasm32"))]
pub fn play(rom: Vec<u8>, join: Option<crate::netplay::Join>) -> ! {
    let (net, multitap) = match &join {
        Some(j) => match crate::netplay::connect(j, &rom) {
            Ok((pads, _, players)) => (Some(pads), players > 2),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
        None => (None, false),
    };
    let app = GameApp::new(rom, multitap, net);
    if let Err(e) = platform::run("MegaFuse", (320 * 3, 224 * 3), app) {
        eprintln!("{e}");
        std::process::exit(1);
    }
    std::process::exit(0)
}
