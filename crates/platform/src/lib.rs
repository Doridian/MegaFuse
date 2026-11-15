//! MegaFuse's platform: a window that shows the picture the game makes
//! (wgpu, scaled by a whole number), the keys held down, and the sound
//! (`audio`, cpal).  The game runs on a thread of its own (it never
//! returns: frames pass as it runs) and meets the window through
//! [`Shared`]: it puts each frame's picture there and reads the keys.

pub mod audio;
mod display;

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Fullscreen, Window, WindowId};

pub use display::Error;
pub use winit::keyboard::KeyCode;

/// A frame's picture: RGBA, `width` x `height`; `serial` counts frames.
#[derive(Default)]
pub struct Picture {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub serial: u64,
}

/// What the game thread and the window share.
#[derive(Default)]
pub struct Shared {
    pub picture: Mutex<Picture>,
    /// The keys held down.
    pub keys: Mutex<HashSet<KeyCode>>,
    /// The window was closed (the game stops), or the game ended.
    pub quit: AtomicBool,
}

impl Shared {
    #[must_use]
    pub fn held(&self, key: KeyCode) -> bool {
        self.keys.lock().unwrap_or_else(PoisonError::into_inner).contains(&key)
    }
}

struct App {
    title: String,
    size: (u32, u32),
    shared: Arc<Shared>,
    window: Option<Arc<Window>>,
    display: Option<display::Display>,
    error: Option<Error>,
    shown: u64,
}

impl App {
    fn fail(&mut self, event_loop: &ActiveEventLoop, e: Error) {
        self.error = Some(e);
        self.shared.quit.store(true, Ordering::Relaxed);
        event_loop.exit();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title(self.title.clone())
            .with_inner_size(LogicalSize::new(self.size.0, self.size.1));
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => return self.fail(event_loop, Error::Window(e.to_string())),
        };
        match display::Display::new(event_loop, Arc::clone(&window)) {
            Ok(d) => self.display = Some(d),
            Err(e) => return self.fail(event_loop, e),
        }
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.shared.quit.store(true, Ordering::Relaxed);
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(d) = &mut self.display {
                    d.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(code) = event.physical_key else { return };
                let down = event.state == ElementState::Pressed;
                if down
                    && !event.repeat
                    && code == KeyCode::F11
                    && let Some(w) = &self.window
                {
                    let full = w.fullscreen().is_some();
                    w.set_fullscreen(if full { None } else { Some(Fullscreen::Borderless(None)) });
                }
                let mut keys = self.shared.keys.lock().unwrap_or_else(PoisonError::into_inner);
                if down {
                    keys.insert(code);
                } else {
                    keys.remove(&code);
                }
            }
            WindowEvent::Focused(false) => {
                // keys let go while the window was away don't stay held
                self.shared.keys.lock().unwrap_or_else(PoisonError::into_inner).clear();
            }
            WindowEvent::RedrawRequested => {
                let Some(d) = &mut self.display else { return };
                let p = self.shared.picture.lock().unwrap_or_else(PoisonError::into_inner);
                if p.width == 0 {
                    return;
                }
                self.shown = p.serial;
                if let Err(e) = d.draw(&p.rgba, p.width, p.height) {
                    drop(p);
                    self.fail(event_loop, e);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.shared.quit.load(Ordering::Relaxed) {
            event_loop.exit();
            return;
        }
        // draw each new frame (the present waits for the screen)
        let serial = self.shared.picture.lock().unwrap_or_else(PoisonError::into_inner).serial;
        if let Some(w) = &self.window
            && serial != self.shown
        {
            w.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(4),
        ));
    }
}

/// The window, until it is closed or the game ends (`shared.quit`).
///
/// # Errors
/// The window or the GPU couldn't be had.
pub fn run(title: &str, size: (u32, u32), shared: Arc<Shared>) -> Result<(), Error> {
    let event_loop = EventLoop::new().map_err(|e| Error::Window(e.to_string()))?;
    let mut app = App {
        title: title.to_string(),
        size,
        shared,
        window: None,
        display: None,
        error: None,
        shown: 0,
    };
    event_loop.run_app(&mut app).map_err(|e| Error::Window(e.to_string()))?;
    app.error.map_or(Ok(()), Err)
}
