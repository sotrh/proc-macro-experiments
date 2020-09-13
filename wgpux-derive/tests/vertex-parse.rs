use wgpux::Vertex;

#[allow(dead_code)]
#[repr(C)]
#[derive(Vertex)]
struct TexturedVertex {
    position: f32,
    texcoord: f32,
}

fn main() {
    let _ = TexturedVertex::desc();
}