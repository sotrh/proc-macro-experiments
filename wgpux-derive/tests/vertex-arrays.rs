use wgpux::Vertex;

#[repr(C)]
#[derive(Vertex)]
struct TexturedVertex {
    position: [f32; 3],
    texcoord: [f32; 2],
}

fn main() {
    let desc: wgpu::VertexBufferDescriptor<'_> = TexturedVertex::desc();
    assert_eq!(desc.attributes.len(), 2);

    let pos_attr = &desc.attributes[0];
    assert_eq!(pos_attr.offset, 0);
    assert_eq!(pos_attr.shader_location, 0);
    assert_eq!(pos_attr.format, wgpu::VertexFormat::Float3);

    let tex_attr = &desc.attributes[1];
    assert_eq!(tex_attr.offset, std::mem::size_of::<[f32; 3]>() as _);
    assert_eq!(tex_attr.shader_location, 1);
    assert_eq!(tex_attr.format, wgpu::VertexFormat::Float2);
}