use bytemuck::{Pod, Zeroable};
use std::mem;
use wgpu::util::DeviceExt;

use crate::dimensions::Dimensions;

// ---------------------------------------------------------------------------
// Structures that are shared between Rust and the compute/fragment shaders.

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    width: u32,
    //_padding0: [u32; 3],
    height: u32,
    //_padding1: [u32; 3],
    threshold: f32,
    //_padding2: [u32; 3],
}

// ---------------------------------------------------------------------------

pub struct LifeParams {
    param_buf: wgpu::Buffer,
}

impl LifeParams {
    pub fn new(device: &wgpu::Device, dimensions: Dimensions, threshold: f32) -> Self {
        let params = Params {
            width: dimensions.width(),
            height: dimensions.height(),
            threshold,
            /*
            _padding0: [0, 0, 0],
            _padding1: [0, 0, 0],
            _padding2: [0, 0, 0],
            */
        };
        let param_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("parameters buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        LifeParams { param_buf }
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.param_buf.as_entire_binding()
    }

    pub fn binding_type(&self) -> wgpu::BindingType {
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(mem::size_of::<Params>() as _),
        }
    }
}
