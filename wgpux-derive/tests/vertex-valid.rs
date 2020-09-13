use wgpux::Vertex;

#[repr(C)]
#[derive(Vertex)]
struct TexturedVertex {
    position: f32,
    texcoord: f32,
    instance: u32,
    dummy: i32,
}

fn main() {
    let desc: wgpu::VertexBufferDescriptor<'_> = TexturedVertex::desc();
    assert_eq!(desc.attributes.len(), 4);

    let mut offset: wgpu::BufferAddress = 0;
    let pos_attr = &desc.attributes[0];
    assert_eq!(pos_attr.offset, offset);
    assert_eq!(pos_attr.shader_location, 0);
    assert_eq!(pos_attr.format, wgpu::VertexFormat::Float);
    offset += std::mem::size_of::<f32>() as wgpu::BufferAddress;
    
    let tex_attr = &desc.attributes[1];
    assert_eq!(tex_attr.offset, offset);
    assert_eq!(tex_attr.shader_location, 1);
    assert_eq!(tex_attr.format, wgpu::VertexFormat::Float);
    offset += std::mem::size_of::<f32>() as wgpu::BufferAddress;
    
    let instance_attr = &desc.attributes[2];
    assert_eq!(instance_attr.offset, offset);
    assert_eq!(instance_attr.shader_location, 2);
    assert_eq!(instance_attr.format, wgpu::VertexFormat::Uint);
    offset += std::mem::size_of::<u32>() as wgpu::BufferAddress;

    let tex_attr = &desc.attributes[3];
    assert_eq!(tex_attr.offset, offset);
    assert_eq!(tex_attr.shader_location, 3);
    assert_eq!(tex_attr.format, wgpu::VertexFormat::Int);
}