// This renders a rectangular wgpu::Texture using a pair of triangles.
// based on https://github.com/gfx-rs/wgpu-rs/blob/master/examples/cube/main.rs

use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, mem};
use wgpu::util::DeviceExt;

use crate::{
    life_params::LifeParams,
    texture::Texture,
};

pub struct Renderer {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

impl Renderer {
    fn vertex(pos: [i8; 2], tc: [i8; 2]) -> Vertex {
        Vertex {
            _pos: [pos[0] as f32, pos[1] as f32, 1.0, 1.0],
            _tex_coord: [tc[0] as f32, tc[1] as f32],
        }
    }

    fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
        let vertex_data = [
            Renderer::vertex([-1, -1], [0, 0]),
            Renderer::vertex([ 1, -1], [1, 0]),
            Renderer::vertex([ 1,  1], [1, 1]),
            Renderer::vertex([-1,  1], [0, 1]),
        ];

        let index_data: &[u16] = &[
            0, 1, 2, 2, 3, 0,
        ];

        (vertex_data.to_vec(), index_data.to_vec())
    }

    pub fn new(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        params: &LifeParams,
        texture: &Texture,
    ) -> Self {
        // Load and compile the shaders.
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("render.wgsl"))),
        });

        // Create the vertex and index buffers.
        let vertex_size = mem::size_of::<Vertex>();
        let (vertex_data, index_data) = Renderer::create_vertices();
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffers"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffers"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Bind the texture and params using a bind group.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: texture.binding_type(wgpu::StorageTextureAccess::ReadOnly),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: params.binding_type(),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: texture.binding_resource(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params.binding_resource(),
                },
            ],
            label: None,
        });

        // Create the render pipeline.
        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[sc_desc.format.into()],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        // Done.
        Renderer {
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            bind_group,
            pipeline,
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.push_debug_group("Prepare data for draw.");
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        rpass.pop_debug_group();
        rpass.insert_debug_marker("Draw!");
        rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
