use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

use crate::{
    gui, pipeline,
    storage::{self, Storable},
};

const QUAD_VERTICIES: &[storage::Vertex] = &[
    storage::Vertex {
        position: glam::f32::vec3(-1.0, 1.0, 0.0),
        uvs: glam::f32::vec2(0.0, 1.0),
    },
    storage::Vertex {
        position: glam::f32::vec3(-1.0, -1.0, 0.0),
        uvs: glam::f32::vec2(0.0, 0.0),
    },
    storage::Vertex {
        position: glam::f32::vec3(1.0, -1.0, 0.0),
        uvs: glam::f32::vec2(1.0, 0.0),
    },
    storage::Vertex {
        position: glam::f32::vec3(1.0, 1.0, 0.0),
        uvs: glam::f32::vec2(1.0, 1.0),
    },
];

const QUAD_INDICES: &[u32] = &[0, 1, 2, 2, 3, 0];

pub struct State {
    globals: Globals,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pipelines: Pipelines,
    compute_data: ComputeData,
    render_data: RenderData,
    gui_layer: GuiLayer,
}

pub struct Timing {
    pub time: f32,
    pub avs_fps: f32,
    pub reference_time: std::time::Instant,
    pub last_checkpoint: std::time::Instant,
    pub frames_since_last_checkpoint: usize,
}

pub struct Globals {
    pub timing: Timing,
}

pub struct Pipelines {
    compute: pipeline::compute::ComputePipeline,
    render: pipeline::render::RenderPipeline,
}

pub struct RenderData {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

pub struct ComputeData {
    globals_buffer: wgpu::Buffer,
    render_texture: wgpu::TextureView,
}

pub struct GuiLayer {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    interface: gui::Interface,
    enabled: bool,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let globals = Globals {
            timing: {
                let now = std::time::Instant::now();
                Timing {
                    time: 0.0,
                    avs_fps: 0.0,
                    reference_time: now,
                    last_checkpoint: now,
                    frames_since_last_checkpoint: 0,
                }
            },
        };

        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        // Shader code assumes an sRGB surface texture
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let pipelines = Pipelines {
            compute: pipeline::compute::ComputePipeline::new(&device),
            render: pipeline::render::RenderPipeline::new(&device, surface_format),
        };

        let render_data = RenderData {
            vertex_buffer: {
                let bytes = bytemuck::cast_slice(QUAD_VERTICIES);
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: &bytes,
                    usage: wgpu::BufferUsages::VERTEX,
                })
            },
            index_buffer: {
                let bytes = bytemuck::cast_slice(QUAD_INDICES);
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: &bytes,
                    usage: wgpu::BufferUsages::INDEX,
                })
            },
        };

        let compute_data = ComputeData {
            globals_buffer: {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Globals buffer"),
                    contents: {
                        let storage: storage::Globals = (&globals).into();
                        let uniform = storage::Uniform(&storage);
                        &uniform.into_bytes()
                    },
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
            },
            render_texture: {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Output texture"),
                    size: wgpu::Extent3d {
                        width: size.width,
                        height: size.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba32Float,
                    usage: wgpu::TextureUsages::STORAGE_BINDING,
                    view_formats: &[wgpu::TextureFormat::Rgba32Float],
                });

                texture.create_view(&wgpu::TextureViewDescriptor::default())
            },
        };

        let gui_layer = {
            let ctx = egui::Context::default();
            let state = egui_winit::State::new(window);
            let renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1);

            GuiLayer {
                ctx,
                state,
                renderer,
                interface: gui::Interface::new(),
                enabled: true,
            }
        };

        Self {
            globals,
            surface,
            device,
            queue,
            config,
            size,
            pipelines,
            render_data,
            compute_data,
            gui_layer,
        }
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0
            && new_size.height > 0
            && new_size.width < u32::MAX
            && new_size.height < u32::MAX
        {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let mut handled = self
            .gui_layer
            .state
            .on_event(&self.gui_layer.ctx, event)
            .consumed;

        match event {
            WindowEvent::KeyboardInput {
                input:
                    winit::event::KeyboardInput {
                        virtual_keycode: Some(winit::event::VirtualKeyCode::Space),
                        state,
                        ..
                    },
                ..
            } => {
                if *state == winit::event::ElementState::Released {
                    self.gui_layer.enabled = !self.gui_layer.enabled;
                    handled = true;
                }
            }
            WindowEvent::MouseWheel { delta, phase, .. } => match &phase {
                winit::event::TouchPhase::Moved => {
                    let (delta, multiplier) = match delta {
                        winit::event::MouseScrollDelta::PixelDelta(delta) => (
                            std::time::Duration::from_millis((delta.y * 10.0).abs() as u64),
                            delta.y.signum(),
                        ),
                        _ => (std::time::Duration::ZERO, 0.0),
                    };

                    if multiplier < 0.0 {
                        self.globals.timing.reference_time -= delta;
                    } else if multiplier > 0.0 {
                        self.globals.timing.reference_time += delta;
                    }

                    handled = true;
                }
                _ => {}
            },
            _ => {}
        }

        handled
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        self.globals.timing.time = self.globals.timing.reference_time.elapsed().as_secs_f32();
        self.globals.timing.frames_since_last_checkpoint += 1;
        if self.globals.timing.last_checkpoint.elapsed().as_secs_f32() >= 0.25 {
            self.globals.timing.avs_fps = {
                let frame_count = self.globals.timing.frames_since_last_checkpoint;
                (frame_count as f32) / self.globals.timing.last_checkpoint.elapsed().as_secs_f32()
            };

            self.globals.timing.frames_since_last_checkpoint = 0;
            self.globals.timing.last_checkpoint = std::time::Instant::now();
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut cmd_buffer = Vec::new();

        // Copy frame data to GPU
        {
            let globals_data = {
                let storage: storage::Globals = (&self.globals).into();
                storage::Uniform(&storage).into_bytes()
            };

            let globals_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: &globals_data,
                        usage: wgpu::BufferUsages::COPY_SRC,
                    });

            encoder.copy_buffer_to_buffer(
                &globals_buffer,
                0,
                &self.compute_data.globals_buffer,
                0,
                globals_data.len() as wgpu::BufferAddress,
            );
        }

        // Compute pass
        {
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Compute bind group"),
                layout: &self.pipelines.compute.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.compute_data.globals_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &self.compute_data.render_texture,
                        ),
                    },
                ],
            });

            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute pass"),
                });

                compute_pass.set_pipeline(&self.pipelines.compute.pipeline);
                compute_pass.set_bind_group(0, &bind_group, &[]);
                compute_pass.dispatch_workgroups(self.size.width, self.size.height, 1)
            }
        }

        // Render pass
        {
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Render bind group"),
                layout: &self.pipelines.render.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.compute_data.render_texture),
                }],
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        // This is what @location(0) in the fragment shader targets
                        Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 1.0,
                                    g: 0.0,
                                    b: 0.5,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }),
                    ],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.pipelines.render.pipeline);
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.render_data.vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.render_data.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }

        // GUI Pass
        if self.gui_layer.enabled {
            let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [self.size.width as u32, self.size.height as u32],
                pixels_per_point: self.gui_layer.state.pixels_per_point(),
            };

            let input = self.gui_layer.state.take_egui_input(window);
            let output = self.gui_layer.ctx.run(input, |ctx| {
                self.gui_layer.interface.ui(ctx, &mut self.globals);
            });

            self.gui_layer.state.handle_platform_output(
                window,
                &self.gui_layer.ctx,
                output.platform_output,
            );

            let texture_deltas = output.textures_delta;
            let paint_jobs = self.gui_layer.ctx.tessellate(output.shapes);

            for (id, image_delta) in &texture_deltas.set {
                self.gui_layer
                    .renderer
                    .update_texture(&self.device, &self.queue, *id, image_delta);
            }

            for id in &texture_deltas.free {
                self.gui_layer.renderer.free_texture(id);
            }

            let gui_commands = self.gui_layer.renderer.update_buffers(
                &self.device,
                &self.queue,
                &mut encoder,
                &paint_jobs,
                &screen_descriptor,
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GUI Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
            });

            self.gui_layer
                .renderer
                .render(&mut render_pass, &paint_jobs, &screen_descriptor);

            cmd_buffer.extend(gui_commands.into_iter());
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(
            cmd_buffer
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        output.present();

        Ok(())
    }
}

impl From<&Globals> for storage::Globals {
    fn from(globals: &Globals) -> Self {
        Self {
            time: globals.timing.time,
        }
    }
}
