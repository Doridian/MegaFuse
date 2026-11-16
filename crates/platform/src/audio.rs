//! Sound output: the game pushes stereo samples at its rate (the
//! YM2612's), a `cpal` stream (the `audio-device` feature) pulls them at
//! the sound card's, resampled linearly.  The queue is kept short: what
//! runs over some 100 ms is dropped, and while it is empty the last sample
//! holds.  Without a device the samples are dropped.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, PoisonError};

/// The samples between the game and the sound card.
struct Queue {
    samples: VecDeque<[f32; 2]>,
    /// Source samples per output sample, and where between the first two
    /// queued the output is.
    step: f64,
    phase: f64,
    last: [f32; 2],
    /// The most to keep queued (source samples).
    limit: usize,
}

impl Queue {
    fn next(&mut self) -> [f32; 2] {
        let (Some(&a), Some(&b)) = (self.samples.front(), self.samples.get(1)) else {
            return self.last;
        };
        let t = self.phase as f32;
        let out = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        self.phase += self.step;
        while self.phase >= 1.0 && self.samples.len() > 1 {
            self.phase -= 1.0;
            self.samples.pop_front();
        }
        self.last = out;
        out
    }
}

pub struct Audio {
    queue: Option<Arc<Mutex<Queue>>>,
    #[cfg(feature = "audio-device")]
    _stream: Option<Arc<cpal::Stream>>,
}

// The browser's streams, for [`resume`].
#[cfg(all(feature = "audio-device", target_arch = "wasm32"))]
thread_local! {
    static STREAMS: std::cell::RefCell<Vec<std::sync::Weak<cpal::Stream>>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Starts the sound after the player pressed a key: a browser keeps a
/// page's `AudioContext` suspended until then, and the stream asks for
/// it to run when it plays.  Natively it does nothing.
pub fn resume() {
    #[cfg(all(feature = "audio-device", target_arch = "wasm32"))]
    STREAMS.with(|s| {
        use cpal::traits::StreamTrait;
        s.borrow_mut().retain(|w| {
            let Some(st) = w.upgrade() else { return false };
            let _ = st.play();
            true
        });
    });
}

impl Audio {
    /// Sound at `rate` samples a second to the default output device, if
    /// there is one; else silence.
    #[must_use]
    pub fn new(rate: u32) -> Self {
        #[cfg(feature = "audio-device")]
        match open(rate) {
            Ok((queue, stream)) => {
                let stream = Arc::new(stream);
                #[cfg(target_arch = "wasm32")]
                STREAMS.with(|s| s.borrow_mut().push(Arc::downgrade(&stream)));
                return Audio { queue: Some(queue), _stream: Some(stream) };
            }
            Err(e) => eprintln!("no sound: {e}"),
        }
        Audio {
            queue: None,
            #[cfg(feature = "audio-device")]
            _stream: None,
        }
    }

    /// Stereo samples, interleaved.
    pub fn push(&self, samples: &[f32]) {
        let Some(q) = &self.queue else { return };
        let mut q = q.lock().unwrap_or_else(PoisonError::into_inner);
        q.samples.extend(samples.as_chunks::<2>().0.iter().copied());
        let limit = q.limit;
        while q.samples.len() > limit {
            q.samples.pop_front();
        }
    }
}

#[cfg(feature = "audio-device")]
fn open(rate: u32) -> Result<(Arc<Mutex<Queue>>, cpal::Stream), String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("no audio output device")?;
    let supported = device.default_output_config().map_err(|e| e.to_string())?;
    let config = supported.config();
    let channels = usize::from(config.channels);
    let queue = Arc::new(Mutex::new(Queue {
        samples: VecDeque::new(),
        step: f64::from(rate) / f64::from(config.sample_rate),
        phase: 0.0,
        last: [0.0; 2],
        limit: rate as usize / 10,
    }));
    let err = |e: cpal::Error| eprintln!("sound: {e}");
    let fill = {
        let queue = Arc::clone(&queue);
        move |out: &mut [f32]| {
            let mut q = queue.lock().unwrap_or_else(PoisonError::into_inner);
            for frame in out.chunks_mut(channels) {
                let [l, r] = q.next();
                for (k, o) in frame.iter_mut().enumerate() {
                    *o = if k % 2 == 0 { l } else { r };
                }
            }
        }
    };
    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => device
            .build_output_stream(config, move |out: &mut [f32], _: &cpal::OutputCallbackInfo| fill(out), err, None)
            .map_err(|e| e.to_string())?,
        cpal::SampleFormat::I16 => {
            let mut buf = Vec::new();
            device
                .build_output_stream(
                    config,
                    move |out: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        buf.resize(out.len(), 0.0);
                        fill(&mut buf);
                        for (o, &v) in out.iter_mut().zip(&buf) {
                            *o = (v.clamp(-1.0, 1.0) * 32767.0) as i16;
                        }
                    },
                    err,
                    None,
                )
                .map_err(|e| e.to_string())?
        }
        f => return Err(format!("sample format {f:?}")),
    };
    stream.play().map_err(|e| e.to_string())?;
    Ok((queue, stream))
}
