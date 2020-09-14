use anyhow::*;
use futures::executor::block_on;
use std::io::Write;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpux::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, Vertex)]
struct ColoredVertex {
    position: [f32; 2],
    color: [f32; 4],
}

unsafe impl bytemuck::Pod for ColoredVertex {}
unsafe impl bytemuck::Zeroable for ColoredVertex {}

fn main() -> Result<()> {
    env_logger::init();
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let adapter: wgpu::Adapter = block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = block_on(adapter.request_device(&Default::default(), None))?;

    let output_size = wgpu::Extent3d { 
        width: 512, 
        height: 512, 
        depth: 1 
    };
    let output_desc = wgpu::TextureDescriptor {
        label: Some("Output"),
        size: output_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
    };
    let output = device.create_texture(&output_desc);
    let output_view = output.create_view(&Default::default());
    let bytes_per_pixel = std::mem::size_of::<u32>();
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
    let unpadded_bytes_per_row = output_size.width as usize * bytes_per_pixel;
    let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
    let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
    let output_buffer_desc = wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: (padded_bytes_per_row * output_size.height as usize) as _,
        usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    };
    let output_buffer = device.create_buffer(&output_buffer_desc);

    let layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        }
    );
    let vs_module = device.create_shader_module(wgpu::include_spirv!("vertex_colored.vert.spv"));
    let fs_module = device.create_shader_module(wgpu::include_spirv!("vertex_colored.frag.spv"));
    let pipeline = device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: None,
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: output_desc.format,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }
            ],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[ColoredVertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        }
    );
    let vertices = [
        ColoredVertex { 
            position: [ -0.5, -0.5 ],
            color: [ 1.0, 0.0, 0.0, 1.0 ],
        },
        ColoredVertex { 
            position: [ 0.5, -0.5 ],
            color: [ 0.0, 1.0, 0.0, 1.0 ],
        },
        ColoredVertex { 
            position: [ 0.0, 0.5 ],
            color: [ 0.0, 0.0, 1.0, 1.0 ],
        },
    ];
    let vertex_buffer = device.create_buffer_init(
        &BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        }
    );

    let mut encoder = device.create_command_encoder(&Default::default());
    let mut pass: wgpu::RenderPass = encoder.begin_render_pass(
        &wgpu::RenderPassDescriptor {
            color_attachments: &[
                wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations::default(),
                }
            ],
            depth_stencil_attachment: None,
        }
    );
    pass.set_pipeline(&pipeline);
    pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    pass.draw(0..vertices.len() as _, 0..1);
    drop(pass);
    assert_eq!(padded_bytes_per_row as u32 % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT, 0);
    encoder.copy_texture_to_buffer(
        wgpu::TextureCopyView {
            texture: &output,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::BufferCopyView {
            buffer: &output_buffer,
            layout: wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: padded_bytes_per_row as _,
                rows_per_image: output_size.height,
            },
        },
        output_size,
    );
    queue.submit(Some(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
    device.poll(wgpu::Maintain::Wait);
    
    if let Ok(()) = block_on(buffer_future) {
        let padded_buffer = buffer_slice.get_mapped_range();
        let mut png_encoder = png::Encoder::new(
            std::fs::File::create("output.png").unwrap(),
            output_size.width,
            output_size.height,
        );
        png_encoder.set_depth(png::BitDepth::Eight);
        png_encoder.set_color(png::ColorType::RGBA);
        let mut png_writer = png_encoder
            .write_header()
            .unwrap()
            .into_stream_writer_with_size(unpadded_bytes_per_row);
        
        for chunk in padded_buffer.chunks(padded_bytes_per_row) {
            png_writer
                .write(&chunk[..unpadded_bytes_per_row])
                .unwrap();
        }
        png_writer.finish().unwrap();
        drop(padded_buffer);
        output_buffer.unmap();
    }

    Ok(())
}