/*!
 * A crate for macros related to the wgpu framework.
 */

pub trait Vertex {
    fn desc<'desc>() -> wgpu::VertexBufferDescriptor<'desc>;
}

pub use wgpux_derive::Vertex;