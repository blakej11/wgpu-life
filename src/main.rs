// #[path = "../framework.rs"]
mod framework;

use rand::{
    distributions::{Distribution, Uniform},
    SeedableRng,
};

mod debug_buffer;
mod dimensions;
mod directions;
mod life;
mod life_params;
mod renderer;
mod texture;

use crate::{
    debug_buffer::DebugBuffer, dimensions::Dimensions, life::Life, life_params::LifeParams,
    renderer::Renderer, texture::Texture,
};

// ---------------------------------------------------------------------------

/// LifeProg struct holds all of the state used by the program.
struct LifeProg {
    life: Life,
    renderer: Renderer,
    debug_buffer: DebugBuffer<f32>,
}

impl framework::Example for LifeProg {
    /// Construct the initial instance of the LifeProg struct.
    fn init(
        sc_desc: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let dim = Dimensions::new(sc_desc.width, sc_desc.height);
        let ncells = dim.area();

        // Parameters for the game, shared between compute and fragment shaders.
        let params = LifeParams::new(device, dim, 0.70);

        // Create the texture that's shared between compute and fragment shaders.
        let texture = Texture::new(device, dim, wgpu::TextureFormat::R32Float);

        // Initialize the life algorithm.
        let life = Life::new(device, dim, &params, &texture);

        // Set the initial state for all cells in the life grid.
        life.import(device, queue, {
            let mut cell_data: Vec<f32> = Vec::new();
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);
            let unif = Uniform::new_inclusive(0.0, 1.0);
            for _ in 0..ncells {
                cell_data.push(unif.sample(&mut rng));
            }
            cell_data
        });

        // Step the algorithm a few times, so the initial image looks Life-like.
        let command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        queue.submit(Some(command_encoder.finish()));

        // Initialize the vertex and fragment shaders.
        let renderer = Renderer::new(sc_desc, device, &params, &texture);

        // Create a buffer to allow snooping on the generated data.
        let debug_buffer = DebugBuffer::new(device, ncells);

        LifeProg {
            life,
            renderer,
            debug_buffer,
        }
    }

    /// update is called for any WindowEvent not handled by the framework
    fn update(&mut self, _event: winit::event::WindowEvent) {
        // empty
    }

    /// resize is called on WindowEvent::Resized events
    fn resize(
        &mut self,
        _sc_desc: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        // empty
    }

    /// render is called to generate each new frame
    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &framework::Spawner,
    ) {
        let debug = false;

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        if debug {
            self.debug_buffer
                .enqueue_copyin(&mut command_encoder, self.life.src_buf());
        }

        // Run the life algorithm one step.
        self.life.step(&mut command_encoder);

        // Render the life cells into actual pixels, and display them.
        self.renderer.render(&mut command_encoder, view);

        queue.submit(Some(command_encoder.finish()));

        if debug {
            println!("Life data at step {}:", self.life.frame_num());
            self.debug_buffer.display(device);
            println!();
        }
    }
}

/// run example
fn main() {
    framework::run::<LifeProg>("life");
}

#[test]
fn life() {
    framework::test::<LifeProg>(framework::FrameworkRefTest {
        image_path: "/examples/life/screenshot.png",
        width: 1024,
        height: 768,
        optional_features: wgpu::Features::default(),
        base_test_parameters: framework::test_common::TestParameters::default()
            .downlevel_flags(wgpu::DownlevelFlags::COMPUTE_SHADERS),
        tolerance: 0,
        max_outliers: 0,
    });
}
