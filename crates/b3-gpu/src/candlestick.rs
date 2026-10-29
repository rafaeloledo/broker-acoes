use b3_core::quote::GpuCandle;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ChartUniforms {
    pub view_width: f32,
    pub view_height: f32,
    pub min_price: f32,
    pub max_price: f32,
    pub start_idx: f32,
    pub end_idx: f32,
    pub candle_width: f32,
    pub wick_width: f32,
}

pub struct CandlestickPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_buffer: wgpu::Buffer,
    pub candle_buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub candle_count: u32,
    pub current_uniforms: ChartUniforms,
}

impl CandlestickPipeline {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Candlestick Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../shaders/candlestick.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Chart Uniform Buffer"),
            size: std::mem::size_of::<ChartUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Candlestick Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Candlestick Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Candlestick Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            uniform_buffer,
            candle_buffer: None,
            bind_group: None,
            candle_count: 0,
            current_uniforms: ChartUniforms {
                view_width: 800.0,
                view_height: 400.0,
                min_price: 0.0,
                max_price: 100.0,
                start_idx: 0.0,
                end_idx: 0.0,
                candle_width: 8.0,
                wick_width: 1.5,
            },
        }
    }

    pub fn update_data(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        candles: &[GpuCandle],
        uniforms: ChartUniforms,
    ) {
        self.current_uniforms = uniforms;
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        if candles.is_empty() {
            self.candle_count = 0;
            return;
        }

        self.candle_count = candles.len() as u32;

        let needed_size = (candles.len() * std::mem::size_of::<GpuCandle>()) as u64;
        let should_realloc = match &self.candle_buffer {
            Some(buf) => buf.size() < needed_size,
            None => true,
        };

        if should_realloc {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Candle Storage Buffer"),
                contents: bytemuck::cast_slice(candles),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Candlestick Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            });

            self.candle_buffer = Some(buffer);
            self.bind_group = Some(bind_group);
        } else if let Some(buf) = &self.candle_buffer {
            queue.write_buffer(buf, 0, bytemuck::cast_slice(candles));
        }
    }

    pub fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        if let Some(bind_group) = &self.bind_group {
            let visible_count = (self.current_uniforms.end_idx - self.current_uniforms.start_idx).max(0.0) as u32;
            if visible_count > 0 {
                rpass.set_pipeline(&self.pipeline);
                rpass.set_bind_group(0, bind_group, &[]);
                // 12 vertices per instance (6 wick + 6 body)
                rpass.draw(0..12, 0..visible_count);
            }
        }
    }
}
