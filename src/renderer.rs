use std::sync::Arc;
use winit::window::Window as WinitWindow;

use crate::window::{Color32, PresentError, WindowError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterMode {
    #[default]
    Nearest,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AspectMode {
    #[default]
    Stretch,
    AspectFit,
    Center,
}

pub(crate) struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,

    buffer_width: u32,
    buffer_height: u32,
    pixel_texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    pipeline: wgpu::RenderPipeline,

    aspect_mode: AspectMode,
    background_color: Color32,
}

impl Renderer {
    pub(crate) fn new(
        window: Arc<WinitWindow>,
        filter_mode: FilterMode,
        buffer_width: u32,
        buffer_height: u32,
        aspect_mode: AspectMode,
    ) -> Result<Self, WindowError> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| WindowError::WindowCreationFailed(e.to_string()))?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            ..Default::default()
        }))
        .map_err(|e| WindowError::NoSuitableAdapter(e.to_string()))?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("nanofb device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))
        .map_err(|e| WindowError::DeviceCreationFailed(e.to_string()))?;

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &surface_config);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nanofb sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter_to_wgpu(filter_mode),
            min_filter: filter_to_wgpu(filter_mode),
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nanofb bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let buffer_width = buffer_width.max(1);
        let buffer_height = buffer_height.max(1);
        let (pixel_texture, bind_group) = create_pixel_texture(
            &device,
            &bind_group_layout,
            &sampler,
            buffer_width,
            buffer_height,
        );

        let shader_source = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

@group(0) @binding(0) var t_pixels: texture_2d<f32>;
@group(0) @binding(1) var s_pixels: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_pixels, s_pixels, in.uv);
}
"#;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nanofb blit shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nanofb pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nanofb blit pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            surface,
            device,
            queue,
            surface_config,
            buffer_width,
            buffer_height,
            pixel_texture,
            bind_group,
            bind_group_layout,
            sampler,
            pipeline,
            aspect_mode,
            background_color: Color32::BLACK,
        })
    }

    pub(crate) fn buffer_width(&self) -> u32 {
        self.buffer_width
    }

    pub(crate) fn buffer_height(&self) -> u32 {
        self.buffer_height
    }

    pub(crate) fn set_buffer_size(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        if (width, height) == (self.buffer_width, self.buffer_height) {
            return;
        }

        self.buffer_width = width;
        self.buffer_height = height;
        let (texture, bind_group) = create_pixel_texture(
            &self.device,
            &self.bind_group_layout,
            &self.sampler,
            width,
            height,
        );
        self.pixel_texture = texture;
        self.bind_group = bind_group;
    }

    pub(crate) fn set_aspect_mode(&mut self, mode: AspectMode) {
        self.aspect_mode = mode;
    }

    pub(crate) fn set_background_color(&mut self, color: Color32) {
        self.background_color = color;
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        if width == self.surface_config.width && height == self.surface_config.height {
            return;
        }

        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub(crate) fn present(&mut self, buffer: &[Color32]) -> Result<(), PresentError> {
        let (buf_width, buf_height) = (self.buffer_width, self.buffer_height);
        let expected = (buf_width * buf_height) as usize;
        if buffer.len() != expected {
            return Err(PresentError::BufferSizeMismatch {
                expected,
                got: buffer.len(),
            });
        }

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.pixel_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(buffer),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * buf_width),
                rows_per_image: Some(buf_height),
            },
            wgpu::Extent3d {
                width: buf_width,
                height: buf_height,
                depth_or_array_layers: 1,
            },
        );

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                self.surface.configure(&self.device, &self.surface_config);
                frame
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.surface_config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(PresentError::Fatal(
                    "wgpu surface validation error; see the registered error scope or uncaptured error handler for details".to_string(),
                ));
            }
        };

        let (vx, vy, vw, vh) = compute_viewport(
            self.aspect_mode,
            self.surface_config.width,
            self.surface_config.height,
            buf_width,
            buf_height,
        );

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("nanofb encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nanofb blit pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color32_to_wgpu(self.background_color)),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_viewport(vx, vy, vw, vh, 0.0, 1.0);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(frame);

        Ok(())
    }
}

fn filter_to_wgpu(mode: FilterMode) -> wgpu::FilterMode {
    match mode {
        FilterMode::Nearest => wgpu::FilterMode::Nearest,
        FilterMode::Linear => wgpu::FilterMode::Linear,
    }
}

fn color32_to_wgpu(color: Color32) -> wgpu::Color {
    wgpu::Color {
        r: color.r() as f64 / 255.0,
        g: color.g() as f64 / 255.0,
        b: color.b() as f64 / 255.0,
        a: 1.0,
    }
}

fn compute_viewport(
    mode: AspectMode,
    surface_width: u32,
    surface_height: u32,
    buf_width: u32,
    buf_height: u32,
) -> (f32, f32, f32, f32) {
    let (sw, sh) = (surface_width as f32, surface_height as f32);
    let (bw, bh) = (buf_width as f32, buf_height as f32);

    match mode {
        AspectMode::Stretch => (0.0, 0.0, sw, sh),
        AspectMode::AspectFit => {
            let scale = (sw / bw).min(sh / bh);
            let w = bw * scale;
            let h = bh * scale;
            ((sw - w) / 2.0, (sh - h) / 2.0, w, h)
        }
        AspectMode::Center => ((sw - bw) / 2.0, (sh - bh) / 2.0, bw, bh),
    }
}

fn create_pixel_texture(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::BindGroup) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("nanofb pixel texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("nanofb bind group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });

    (texture, bind_group)
}
