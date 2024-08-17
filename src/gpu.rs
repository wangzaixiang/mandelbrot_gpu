use std::ops::Bound;
use num::Complex;
use wgpu::{BindGroup, Device};
use wgpu::core::resource::StagingBuffer;
use wgpu::util::DeviceExt;

pub fn render(pixes: &mut [u8],
              bounds: (usize, usize),
              upper_left: Complex<f32>,
              lower_right: Complex<f32>)  {

    pollster::block_on( render_gpu(pixes, bounds, upper_left, lower_right) );
}

async fn render_gpu(pixes: &mut [u8],
                    bounds: (usize, usize),
                    upper_left: Complex<f32>,
                    lower_right: Complex<f32>)  {

    let instance = wgpu::Instance::default();
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();

    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor::default(), None)
        .await.unwrap();

    fn create_shader_module(device: &wgpu::Device, source: &str) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        })
    }

    fn create_compute_pipeline(device: &wgpu::Device, module: &wgpu::ShaderModule) -> wgpu::ComputePipeline {
        device.create_compute_pipeline( &wgpu::ComputePipelineDescriptor {
            label: Some("mandelbrot pipeline"),
            layout: None,
            module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None
        })
    }


    fn build_bind_group(device: &wgpu::Device, pipeline: &wgpu::ComputePipeline,
                        pixes_buffer: &wgpu::Buffer, bounds_buffer: &wgpu::Buffer,
                        upper_left_buffer: &wgpu::Buffer, lower_right_buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        let bind_group_layout = pipeline.get_bind_group_layout(0);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pixes_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: bounds_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: upper_left_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: lower_right_buffer.as_entire_binding()
                }
            ],
            label: Some("bind group")
        });

        bind_group
    }

    fn build_encoder(device: &wgpu::Device, pipeline: &wgpu::ComputePipeline, bind_group: BindGroup,
                     bounds: (usize, usize), // 0: width, 1: height
                     size: usize, storage_buffer: &wgpu::Buffer, staging_buffer: &wgpu::Buffer) -> wgpu::CommandEncoder {

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("mandelbrot encoder")
        });

        let mut cpass = encoder.begin_compute_pass( &wgpu::ComputePassDescriptor {
            label: Some("mandelbrot compute pass"),
            timestamp_writes: None
        }); ;
        cpass.set_pipeline(pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.insert_debug_marker("compute pass");
        cpass.dispatch_workgroups( size as u32 / 256, 1, 1);

        println!("dispatch workgroups: {}", size/256);
        // cpass.dispatch_workgroups( size as u32, 1, 1);
        drop(cpass); // pass_end

        encoder.copy_buffer_to_buffer(storage_buffer, 0, staging_buffer, 0, size as u64 * std::mem::size_of::<u32>() as u64);

        encoder
    }

    let module = create_shader_module(&device, include_str!("mandelbrot.wgsl"));

    let pipeline = create_compute_pipeline(&device, &module);

    // TODO change to BufferUsage::WRITE to avoid copying
    let mut pixes_u32: Vec<u32> = pixes.iter().map(|&x| x as u32).collect();
    let pixes_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("pixes buffer"),
        contents: bytemuck::cast_slice(&pixes_u32),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
    });

    // using a staging buffer to read the result
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging buffer"),
        size: pixes_u32.len() as u64 * std::mem::size_of::<u32>() as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false
    });

    let bounds_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("bounds buffer"),
        contents: bytemuck::cast_slice(&[bounds.0 as u32, bounds.1 as u32]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    });
    let upper_left_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("upper_left buffer"),
        contents: bytemuck::cast_slice(&[upper_left.re as f32, upper_left.im as f32]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    });
    let lower_right_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("lower_right buffer"),
        contents: bytemuck::cast_slice(&[lower_right.re as f32, lower_right.im as f32]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    });

    let bind_group = build_bind_group(&device, &pipeline,
        &pixes_buffer, &bounds_buffer, &upper_left_buffer, &lower_right_buffer);

    let mut encoder = build_encoder(&device, &pipeline, bind_group,
                                    bounds, bounds.0 * bounds.1,
                                    &pixes_buffer, &staging_buffer);

    queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| { sender.send(v).unwrap(); });

    device.poll(wgpu::Maintain::Wait).panic_on_timeout();

    if let Ok(Ok(())) = receiver.recv_async().await {
        let data = buffer_slice.get_mapped_range();
        let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();

        for (i, &x) in result.iter().enumerate() {
            pixes[i] = 255 - (x as u8);
        }

    }
    else {
        panic!("Failed to receive data from GPU");
    }

}