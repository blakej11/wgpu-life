use bytemuck::Pod;
use std::convert::TryInto;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;

pub struct DebugBuffer<T> {
    buf: wgpu::Buffer,
    size: u64,
    _phantom: PhantomData<T>,
}

impl<T> DebugBuffer<T>
where
    T: Debug + Pod,
{
    pub fn new(device: &wgpu::Device, nentries: usize) -> Self {
        let size = (nentries * mem::size_of::<T>()).try_into().unwrap();

        DebugBuffer {
            buf: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("debug buffer"),
                size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            size,
            _phantom: PhantomData,
        }
    }

    // Enqueue a copyin action into this debug buffer.
    //
    // Note that display()'ing this debug buffer after an enqueue_copyin()
    // will not show anything useful until the command encoder has been finished
    // and submitted to a command queue.
    pub fn enqueue_copyin(&self, command_encoder: &mut wgpu::CommandEncoder, buf: &wgpu::Buffer) {
        command_encoder.push_debug_group("filling debug buffer");
        command_encoder.copy_buffer_to_buffer(buf, 0, &self.buf, 0, self.size as u64);
        command_encoder.pop_debug_group();
    }

    // Note the above caveat about using enqueue_copyin() with display().
    pub fn display(&self, device: &wgpu::Device) {
        // Start a request to map the debug buffer, and wait for it.
        let buffer_slice = self.buf.slice(..);
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        device.poll(wgpu::Maintain::Wait);
        pollster::block_on(rx.receive()).unwrap().unwrap();
        let data: Vec<u8> = buffer_slice.get_mapped_range().to_vec();
        let result: Vec<T> = bytemuck::cast_slice(&data).to_vec();
        println!("{:?}", result);

        // Current API requires dropping the data before unmapping.
        drop(data);
        self.buf.unmap();
    }

    // Copy the given buffer into this debug buffer and display it immediately.
    // This avoids the caveat mentioned above.
    #[allow(dead_code)]
    pub fn copyin_and_display(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buf: &wgpu::Buffer,
    ) {
        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.enqueue_copyin(&mut command_encoder, buf);
        queue.submit(Some(command_encoder.finish()));

        self.display(device);
    }
}
