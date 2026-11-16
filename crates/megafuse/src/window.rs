//! The game in a window (crates/platform), natively and in the browser:
//! the game runs ahead into a queue of finished frames (pictures and
//! sound), which the window shows at the console's pace.  When the queue
//! is empty the next step runs ([`game::Game::step`]; first the boot,
//! some 155 frames at once): nothing waits, so the same code runs in the
//! browser, which can't.  In a network game the steps come from the
//! netplay driver (netplay.rs), which may run frames again silently.
//! The keys are the pads (arrows, Z X C, Enter;
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
    [
        KeyCode::KeyW,
        KeyCode::KeyS,
        KeyCode::KeyA,
        KeyCode::KeyD,
        KeyCode::KeyF,
        KeyCode::KeyG,
        KeyCode::KeyH,
        KeyCode::Tab,
    ],
];
const BUTTONS: [u16; 8] = [
    button::UP,
    button::DOWN,
    button::LEFT,
    button::RIGHT,
    button::A,
    button::B,
    button::C,
    button::START,
];

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
    /// No sprite limit (F2).
    unlimited: bool,
}

/// The game's frontend: each frame into the queue, the pads from the
/// keys (or, in a network game, from the session).
struct Recorder {
    shared: Arc<Mutex<Shared>>,
    samples: Vec<f32>,
    last: Vec<u8>,
    net: Option<Arc<Mutex<crate::netplay::Net>>>,
}

impl Recorder {
    /// Frames run again (a rollback) or caught up with: not shown.
    fn silent(&self) -> bool {
        self.net
            .as_ref()
            .is_some_and(|n| n.lock().unwrap_or_else(PoisonError::into_inner).silent)
    }
}

impl Frontend for Recorder {
    fn unlimited_sprites(&self) -> bool {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner).unlimited
    }

    fn frame(
        &mut self,
        _frame: u64,
        picture: &[u32],
        width: usize,
        height: usize,
        _ram: &mut [u8; 0x10000],
        pads: &mut [u16; 4],
    ) {
        if self.silent() {
            self.samples.clear();
            if let Some(n) = &self.net {
                *pads = n
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .pads(_frame, _ram);
            }
            return;
        }
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
            *pad = keys
                .iter()
                .zip(BUTTONS)
                .filter(|&(k, _)| s.keys.contains(k))
                .fold(0, |m, (_, b)| m | b);
        }
        drop(s);
        if let Some(n) = &self.net {
            let mut n = n.lock().unwrap_or_else(PoisonError::into_inner);
            n.local = pads[0];
            *pads = n.pads(_frame, _ram);
        }
    }

    fn audio(&mut self, samples: &[f32]) {
        if !self.silent() {
            self.samples.extend_from_slice(samples);
        }
    }
}

/// What runs the game: the window itself, or the netplay driver.
enum Run {
    Local { game: Box<game::Game>, booted: bool },
    Net(Box<crate::netplay::Driver>),
}

/// The window's app: the game, the queue, the frame shown.
pub struct GameApp {
    run: Run,
    shared: Arc<Mutex<Shared>>,
    audio: Audio,
    /// The picture shown and its number.
    shown: Vec<u8>,
    size: (u32, u32),
    serial: u64,
    /// When the next frame is due.
    due: Option<Instant>,
    /// The network game's status line, as last shown.
    status: String,
    /// The network game ended (in the browser, where the page stays):
    /// the last picture stays.
    ended: bool,
    /// A note on the page's status line (a switch flipped), till this
    /// frame.
    note_until: Option<u64>,
}

impl GameApp {
    /// The game on `rom`, played here.
    pub fn new(rom: Vec<u8>) -> Self {
        let shared = Arc::new(Mutex::new(Shared::default()));
        let game = Box::new(game_for(rom, &shared, false, None));
        Self::with(
            Run::Local {
                game,
                booted: false,
            },
            shared,
        )
    }

    /// The game on `rom` in a network game: `session`'s.
    pub fn net(rom: Vec<u8>, session: crate::netplay::Link) -> Self {
        let shared = Arc::new(Mutex::new(Shared::default()));
        let s2 = Arc::clone(&shared);
        let make: crate::netplay::Make =
            Box::new(move |players, net| game_for(rom.clone(), &s2, players > 2, Some(net)));
        Self::with(
            Run::Net(Box::new(crate::netplay::Driver::new(session, make))),
            shared,
        )
    }

    fn with(run: Run, shared: Arc<Mutex<Shared>>) -> Self {
        GameApp {
            run,
            shared,
            audio: Audio::new(md::sound::SAMPLE_RATE),
            shown: Vec::new(),
            size: (0, 0),
            serial: 0,
            due: None,
            status: String::new(),
            ended: false,
            note_until: None,
        }
    }

    /// The next frame: from the queue, the game run on when it's empty
    /// (in a network game, none when the pads aren't in: the picture
    /// stays).
    fn next_frame(&mut self) -> Frame {
        let idle = Frame {
            rgba: None,
            width: self.size.0,
            height: self.size.1,
            samples: Vec::new(),
        };
        if self.ended {
            return idle;
        }
        loop {
            if let Some(f) = self
                .shared
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .queue
                .pop_front()
            {
                return f;
            }
            match &mut self.run {
                Run::Local { game, booted } => {
                    if *booted {
                        game.step();
                    } else {
                        game.boot();
                        *booted = true;
                    }
                }
                Run::Net(d) => match d.tick(Duration::from_millis(8)) {
                    crate::netplay::Tick::Stepped => {}
                    crate::netplay::Tick::Waiting => return idle,
                    crate::netplay::Tick::Ended(e) => {
                        self.ended = true;
                        #[cfg(target_arch = "wasm32")]
                        {
                            platform::web::show_error(&e);
                            return idle;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            eprintln!("{e}");
                            std::process::exit(1);
                        }
                    }
                },
            }
        }
    }
}

impl GameApp {
    /// The network game's status, where the player sees it, when it
    /// changes.
    fn show_status(&mut self) {
        let Run::Net(d) = &self.run else { return };
        if self.ended {
            return;
        }
        let status = d.status();
        // (catching up counts frames: not every one of them)
        let kind = |s: &str| s.split(':').next().unwrap_or_default().to_owned();
        if kind(&status) == kind(&self.status)
            && (status.is_empty() || !self.serial.is_multiple_of(30))
        {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        platform::web::show_status(&status, false);
        #[cfg(not(target_arch = "wasm32"))]
        if !status.is_empty() {
            eprintln!("{status}");
        }
        self.status = status;
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
            if self.note_until.is_some_and(|n| self.serial >= n) {
                self.note_until = None;
                #[cfg(target_arch = "wasm32")]
                platform::web::show_status("", false);
                self.status.clear();
            }
            self.show_status();
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
        (width > 0 && self.shown.len() >= (width * height * 4) as usize).then(|| {
            (
                Picture {
                    rgba: &self.shown,
                    width,
                    height,
                },
                self.serial,
            )
        })
    }

    fn key(&mut self, code: KeyCode, down: bool) {
        let mut s = self.shared.lock().unwrap_or_else(PoisonError::into_inner);
        if down && code == KeyCode::F2 {
            s.unlimited = !s.unlimited;
            let note = if s.unlimited { "No sprite limit (F2)" } else { "The console's sprite limit (F2)" };
            #[cfg(target_arch = "wasm32")]
            platform::web::show_status(note, false);
            #[cfg(not(target_arch = "wasm32"))]
            eprintln!("{note}");
            self.note_until = Some(self.serial + 120);
            return;
        }
        if down {
            s.keys.insert(code);
        } else {
            s.keys.remove(&code);
        }
    }

    fn unfocused(&mut self) {
        self.shared
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .keys
            .clear();
    }
}

/// A game on `rom` whose frontend is a [`Recorder`] into `shared`;
/// `multitap`: the Team Player in port A.
fn game_for(
    rom: Vec<u8>,
    shared: &Arc<Mutex<Shared>>,
    multitap: bool,
    net: Option<Arc<Mutex<crate::netplay::Net>>>,
) -> game::Game {
    let recorder = Recorder {
        shared: Arc::clone(shared),
        samples: Vec::new(),
        last: Vec::new(),
        net,
    };
    let mut md = md::Md::new(rom.clone(), Box::new(recorder));
    if multitap {
        md.plug_teamplayer();
    }
    game::Game::new(rom, Box::new(md))
}

/// Plays `rom` in a window; natively, in a network game.
#[cfg(not(target_arch = "wasm32"))]
/// `unlimited`: no sprite limit to start with (F2 switches it).
pub fn play(rom: Vec<u8>, join: Option<crate::netplay::Join>, unlimited: bool) -> ! {
    let app = match &join {
        Some(j) => match crate::netplay::connect(j, &rom) {
            Ok(session) => GameApp::net(rom, session),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
        None => GameApp::new(rom),
    };
    app.shared.lock().unwrap_or_else(PoisonError::into_inner).unlimited = unlimited;
    if let Err(e) = platform::run("MegaFuse", (320 * 3, 224 * 3), app) {
        eprintln!("{e}");
        std::process::exit(1);
    }
    std::process::exit(0)
}
