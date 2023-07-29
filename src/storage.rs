use encase::{
    private::WriteInto, ArrayLength, ShaderSize, ShaderType, StorageBuffer, UniformBuffer,
};
use glam::f32;

pub trait Storable {
    fn into_bytes(&self) -> Vec<u8>;
}

pub struct Uniform<'a, T>(pub &'a T)
where
    T: ShaderType + ShaderSize + WriteInto;

impl<T> Storable for Uniform<'_, T>
where
    T: ShaderType + ShaderSize + WriteInto,
{
    fn into_bytes(&self) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(self.0).expect("Unable to write uniform");
        buffer.into_inner()
    }
}

pub struct Buffer<'a, T>(pub &'a [T])
where
    T: ShaderSize;

#[derive(ShaderType)]
struct SizedBuffer<'a, T: ShaderSize + 'a> {
    length: ArrayLength,

    #[size(runtime)]
    buffer: &'a [T],
}

impl<'a, T> SizedBuffer<'a, T>
where
    T: ShaderSize + 'a,
{
    fn new(buffer: &'a [T]) -> Self {
        Self {
            length: ArrayLength,
            buffer,
        }
    }
}

impl<T> Storable for Buffer<'_, T>
where
    T: ShaderSize + WriteInto,
{
    fn into_bytes(&self) -> Vec<u8> {
        let data = SizedBuffer::new(self.0);
        let mut buffer = StorageBuffer::new(Vec::new());
        buffer.write(&data).expect("Unable to write buffer");

        buffer.into_inner()
    }
}

/*
    Types
*/

#[derive(ShaderType)]
pub struct Globals {
    pub time: f32,
    pub scale: f32,
    pub radius: f32,
    pub center: f32::Vec2,
    pub orbit_offset: f32::Vec2,
    pub coefficients: [f32::Vec4; 4],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: f32::Vec3,
    pub uvs: f32::Vec2,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
