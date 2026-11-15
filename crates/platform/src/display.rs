//! The GPU side: the picture as a texture, scaled into the window by one
//! fragment shader (VacationDestination's, for a true-colour picture).

use std::fmt;
use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Window(String),
    Gpu(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Window(e) => write!(f, "window: {e}"),
            Error::Gpu(e) => write!(f, "GPU: {e}"),
        }
    }
}

impl std::error::Error for Error {}

/// Where the picture goes in the window. Matches `View` in the shader.
#[derive(Debug, Clone, Copy)]
struct View {
    offset: [f32; 2],
    scale: f32,
}

impl View {
    /// The largest whole scale that fits, centred; a fractional one if even
    /// 1:1 doesn't fit.
    #[allow(clippy::cast_precision_loss)]
    fn new(window: PhysicalSize<u32>, width: u32, height: u32) -> Self {
        let fit = (window.width as f32 / width.max(1) as f32).min(window.height as f32 / height.max(1) as f32);
        let scale = if fit >= 1.0 { fit.floor() } else { fit.max(f32::EPSILON) };
        let offset = [
            ((window.width as f32 - width as f32 * scale) / 2.0).floor(),
            ((window.height as f32 - height as f32 * scale) / 2.0).floor(),
        ];
        View { offset, scale }
    }

    fn bytes(self) -> [u8; 16] {
        let mut b = [0; 16];
        for (i, v) in [self.offset[0], self.offset[1], self.scale, 0.0].into_iter().enumerate() {
            b[4 * i..4 * i + 4].copy_from_slice(&v.to_le_bytes());
        }
        b
    }
}

struct Picture {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

pub(crate) struct Display {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    view: wgpu::Buffer,
    picture: Option<Picture>,
}

fn gpu(e: impl fmt::Display) -> Error {
    Error::Gpu(e.to_string())
}

/// Where the browser's display comes once its GPU is set up.
#[cfg(target_arch = "wasm32")]
pub(crate) type Pending = std::rc::Rc<std::cell::RefCell<Option<Result<Display, Error>>>>;

fn descriptor(event_loop: &ActiveEventLoop) -> wgpu::InstanceDescriptor {
    wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(event_loop.owned_display_handle()))
}

impl Display {
    /// The display for `window`, waiting for the GPU.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(event_loop: &ActiveEventLoop, window: Arc<Window>) -> Result<Self, Error> {
        let instance = wgpu::Instance::new(descriptor(event_loop));
        pollster::block_on(Display::with_instance(&instance, window))
    }

    /// In the browser, which can't wait: starts setting the GPU up for
    /// `window` (WebGPU where the browser has it, else WebGL2) and gives
    /// the slot the display comes to.
    #[cfg(target_arch = "wasm32")]
    pub fn start(event_loop: &ActiveEventLoop, window: Arc<Window>) -> Pending {
        let slot = Pending::default();
        let to = std::rc::Rc::clone(&slot);
        let desc = descriptor(event_loop);
        wasm_bindgen_futures::spawn_local(async move {
            let instance = wgpu::util::new_instance_with_webgpu_detection(desc).await;
            let display = Display::with_instance(&instance, window).await;
            *to.borrow_mut() = Some(display);
        });
        slot
    }

    async fn with_instance(instance: &wgpu::Instance, window: Arc<Window>) -> Result<Self, Error> {
        let surface = instance.create_surface(window.clone()).map_err(gpu)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(gpu)?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("megafuse"),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await
            .map_err(gpu)?;
        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or_else(|| gpu("the surface doesn't fit the adapter"))?;
        // The shader writes the picture's sRGB values as they are.
        config.format = config.format.remove_srgb_suffix();
        config.view_formats = vec![config.format];
        surface.configure(&device, &config);

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(config.format.into())],
            }),
            multiview_mask: None,
            cache: None,
        });
        let view = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Display { window, surface, device, queue, config, pipeline, layout, view, picture: None })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn ensure_picture(&mut self, width: u32, height: u32) {
        if self.picture.as_ref().is_some_and(|p| (p.width, p.height) == (width, height)) {
            return;
        }
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry { binding: 1, resource: self.view.as_entire_binding() },
            ],
        });
        self.picture = Some(Picture { width, height, texture, bind_group });
    }

    /// Draws `rgba` (`width` x `height`).
    pub fn draw(&mut self, rgba: &[u8], width: u32, height: u32) -> Result<(), Error> {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return Ok(()),
            other @ wgpu::CurrentSurfaceTexture::Validation => return Err(gpu(format!("{other:?}"))),
        };
        let size = PhysicalSize::new(self.config.width, self.config.height);
        self.queue.write_buffer(&self.view, 0, &View::new(size, width, height).bytes());
        self.ensure_picture(width, height);
        let Some(picture) = &self.picture else {
            return Ok(());
        };
        if rgba.len() >= (width * height * 4) as usize {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &picture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            );
        }
        let target = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &picture.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(output);
        Ok(())
    }
}
