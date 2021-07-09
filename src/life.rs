// Conway's Game of Life

use std::{borrow::Cow, mem};
use wgpu::util::DeviceExt;

use crate::{
    dimensions::Dimensions,
    directions::{RenderDir, RenderMotion, RenderSources},
    life_params::LifeParams,
    texture::Texture,
};

// ---------------------------------------------------------------------------
// Data that is shared between Rust and the compute pipeline in WGSL.

// Number of cells calculated in each gpu work group.
// This must match the value of the workgroup_size() annotation in life.wgsl
const WORKGROUP_SIZE: (u32, u32) = (8, 8);

// ---------------------------------------------------------------------------

pub struct Life {
    // Data for the compute shader.
    compute_pipeline: wgpu::ComputePipeline,
    bind_groups: RenderMotion<wgpu::BindGroup>,
    dimensions: Dimensions,
    cell_buffers: RenderSources<wgpu::Buffer>,
    frame_num: usize,
}

impl Life {
    pub fn new(
        device: &wgpu::Device,
        dimensions: Dimensions,
        params: &LifeParams,
        texture: &Texture,
    ) -> Self {
        // Load and compile the compute shader.
        let compute_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("life.wgsl"))),
        });

        // Allocate a pair of equal-sized GPU buffers to hold cell data.
        // COPY_SRC is used so they can be read from for debugging.
        let cell_bufsize = dimensions.area() * mem::size_of::<f32>();
        let cell_buffers = RenderSources::new(|dir|
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Source for {:?}", dir)),
                usage: wgpu::BufferUsages::VERTEX
                     | wgpu::BufferUsages::STORAGE
                     | wgpu::BufferUsages::COPY_SRC
                     | wgpu::BufferUsages::COPY_DST,
                size: cell_bufsize as _,
                mapped_at_creation: false,
            }));

        // Create the bind group layout and compute pipeline for the life algorithm.
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Binding for the global variable "params".
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: params.binding_type(),
                        count: None,
                    },

                    // Binding for the global variable "cellSrc".
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size:
                                wgpu::BufferSize::new(cell_bufsize as _),
                        },
                        count: None,
                    },

                    // Binding for the global variable "cellDst".
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size:
                                wgpu::BufferSize::new(cell_bufsize as _),
                        },
                        count: None,
                    },

                    // Binding for the global variable "texture".
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: texture.binding_type(wgpu::StorageTextureAccess::WriteOnly),
                        count: None,
                    },
                ],
                label: None,
            });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("life compute pipeline layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("life compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "life",
        });

        // Create a RenderMotion of bind groups to map the RenderSources of cell_buffers.
        let bind_groups = RenderMotion::new(|dir|
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params.binding_resource(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: cell_buffers.src(dir).as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: cell_buffers.dst(dir).as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: texture.binding_resource(),
                    },
                ],
                label: None,
            })
        );

        Life {
            compute_pipeline,
            bind_groups,
            dimensions,
            cell_buffers,
            frame_num: 0,
        }
    }

    /// update is called for any WindowEvent not handled by the framework
    fn _update(
        &mut self,
         _event: winit::event::WindowEvent,
    ) {
        // empty
    }

    /// resize is called on WindowEvent::Resized events
    fn _resize(
        &mut self,
        _sc_desc: &wgpu::SwapChainDescriptor,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        // empty
    }

    // Import some data into the Life grid.
    pub fn import(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cells: Vec<f32>,
    ) {
        assert_eq!(cells.len(), self.dimensions.area());

        let import_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cell data import buffer"),
            usage: wgpu::BufferUsages::VERTEX
                 | wgpu::BufferUsages::STORAGE
                 | wgpu::BufferUsages::COPY_SRC,
            contents: bytemuck::cast_slice(&cells),
            });

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("filling Life grid")
            });

        let next_src = &self.cell_buffers.src(RenderDir::dir(self.frame_num));

        command_encoder.copy_buffer_to_buffer(
            &import_buf, 0, next_src, 0,
            bytemuck::cast_slice::<_, u8>(&cells).len() as u64
        );

        queue.submit(Some(command_encoder.finish()));
    }

    pub fn step(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut cpass =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Life grid step")
            });

        let xdim = self.dimensions.width() + WORKGROUP_SIZE.0 - 1;
        let xgroups = xdim / WORKGROUP_SIZE.0;
        let ydim = self.dimensions.height() + WORKGROUP_SIZE.1 - 1;
        let ygroups = ydim / WORKGROUP_SIZE.1;
        let dir = RenderDir::dir(self.frame_num);

        cpass.set_pipeline(&self.compute_pipeline);
        cpass.set_bind_group(0, &self.bind_groups.get(dir), &[]);
        cpass.dispatch(xgroups, ygroups, 1);

        self.frame_num += 1;
    }

    #[allow(dead_code)]
    pub fn frame_num(&self) -> usize {
        self.frame_num
    }

    #[allow(dead_code)]
    pub fn src_buf(&self) -> &wgpu::Buffer {
        self.cell_buffers.src(RenderDir::dir(self.frame_num))
    }

    #[allow(dead_code)]
    pub fn dst_buf(&self) -> &wgpu::Buffer {
        self.cell_buffers.dst(RenderDir::dir(self.frame_num))
    }
}
