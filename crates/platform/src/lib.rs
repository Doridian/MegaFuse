//! MegaFuse's platform: a window that shows the picture an [`App`] makes
//! (wgpu, scaled by a whole number), the keys, and the sound (`audio`,
//! cpal), natively and in the browser (wasm32: the page's canvas, WebGPU
//! or WebGL2, WebAudio).  The window calls [`App::update`] as often as it
//! can; the app keeps its own time and says what to show.

pub mod audio;
mod display;
#[cfg(target_arch = "wasm32")]
pub mod web;

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Fullscreen, Window, WindowId};

pub use display::Error;
pub use web_time::{Duration, Instant};
pub use winit::keyboard::KeyCode;

/// A picture: RGBA, `width` x `height`.
pub struct Picture<'a> {
    pub rgba: &'a [u8],
    pub width: u32,
    pub height: u32,
}

/// What runs in the window.
pub trait App {
    /// Time passes (the app looks at the clock itself); returns when it
    /// next wants to be called.
    fn update(&mut self) -> Instant;
    /// What to show now, and a number that changes when it does.
    fn picture(&self) -> Option<(Picture<'_>, u64)>;
    /// A key went down or up.
    fn key(&mut self, code: KeyCode, down: bool);
    /// The window lost the focus (keys held then count as let go).
    fn unfocused(&mut self) {}
    /// Done: the window closes.
    fn finished(&self) -> bool {
        false
    }
}

struct Runner<A: App> {
    app: A,
    title: String,
    /// The window's size natively (the page sizes the browser's).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    size: (u32, u32),
    window: Option<Arc<Window>>,
    display: Option<display::Display>,
    /// In the browser, the display while the GPU is being set up.
    #[cfg(target_arch = "wasm32")]
    pending: Option<display::Pending>,
    error: Option<Error>,
    shown: Option<u64>,
}

impl<A: App> Runner<A> {
    fn fail(&mut self, event_loop: &ActiveEventLoop, e: Error) {
        #[cfg(target_arch = "wasm32")]
        web::show_error(&e.to_string());
        self.error = Some(e);
        event_loop.exit();
    }

    /// In the browser: the display, once the GPU is there.
    #[cfg(target_arch = "wasm32")]
    fn take_display(&mut self, event_loop: &ActiveEventLoop) {
        let Some(slot) = &self.pending else { return };
        let Some(result) = slot.borrow_mut().take() else {
            return;
        };
        self.pending = None;
        match result {
            Ok(d) => self.display = Some(d),
            Err(e) => self.fail(event_loop, e),
        }
    }
}

impl<A: App> ApplicationHandler for Runner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title(self.title.clone());
        // natively a window of this size; in the browser the canvas is as
        // big as the page makes it
        #[cfg(not(target_arch = "wasm32"))]
        let attrs = attrs.with_inner_size(winit::dpi::LogicalSize::new(self.size.0, self.size.1));
        #[cfg(target_arch = "wasm32")]
        let attrs = {
            use winit::platform::web::WindowAttributesExtWebSys;
            let canvas = web::canvas();
            let append = canvas.is_none();
            attrs
                .with_canvas(canvas)
                .with_append(append)
                .with_prevent_default(true)
                .with_focusable(true)
        };
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => return self.fail(event_loop, Error::Window(e.to_string())),
        };
        #[cfg(target_arch = "wasm32")]
        {
            window.focus_window();
            self.pending = Some(display::Display::start(event_loop, Arc::clone(&window)));
        }
        #[cfg(not(target_arch = "wasm32"))]
        match display::Display::new(event_loop, Arc::clone(&window)) {
            Ok(d) => self.display = Some(d),
            Err(e) => return self.fail(event_loop, e),
        }
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        #[cfg(target_arch = "wasm32")]
        self.take_display(event_loop);
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(d) = &mut self.display {
                    d.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // a key press lets the browser's sound start
                audio::resume();
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };
                let down = event.state == ElementState::Pressed;
                if down
                    && !event.repeat
                    && code == KeyCode::F11
                    && let Some(w) = &self.window
                {
                    let full = w.fullscreen().is_some();
                    w.set_fullscreen(if full {
                        None
                    } else {
                        Some(Fullscreen::Borderless(None))
                    });
                }
                if !event.repeat {
                    self.app.key(code, down);
                }
            }
            WindowEvent::Focused(false) => self.app.unfocused(),
            WindowEvent::RedrawRequested => {
                let Some(d) = &mut self.display else { return };
                let Some((p, serial)) = self.app.picture() else {
                    return;
                };
                self.shown = Some(serial);
                if let Err(e) = d.draw(p.rgba, p.width, p.height) {
                    self.fail(event_loop, e);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_arch = "wasm32")]
        self.take_display(event_loop);
        if self.app.finished() {
            event_loop.exit();
            return;
        }
        let next = self.app.update();
        let serial = self.app.picture().map(|(_, s)| s);
        if let Some(w) = &self.window
            && serial != self.shown
        {
            w.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(next));
    }
}

/// The window, until it is closed or the app is done.  In the browser it
/// returns at once and the page's events run it.
///
/// # Errors
/// The window or the GPU couldn't be had.
pub fn run<A: App + 'static>(title: &str, size: (u32, u32), app: A) -> Result<(), Error> {
    let event_loop = EventLoop::new().map_err(|e| Error::Window(e.to_string()))?;
    let runner = Runner {
        app,
        title: title.to_string(),
        size,
        window: None,
        display: None,
        #[cfg(target_arch = "wasm32")]
        pending: None,
        error: None,
        shown: None,
    };
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(runner);
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut runner = runner;
        event_loop
            .run_app(&mut runner)
            .map_err(|e| Error::Window(e.to_string()))?;
        runner.error.map_or(Ok(()), Err)
    }
}
