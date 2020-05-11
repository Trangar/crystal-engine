use super::Vertex;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Device,
};

// TODO: Make it so that developers can create their own models/vertices?
pub struct Model {
    pub(super) vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub(super) indices: Option<Arc<CpuAccessibleBuffer<[u32]>>>,
}

impl Model {
    pub fn from_obj_file(device: Arc<Device>, file: impl AsRef<std::path::Path>) -> Arc<Self> {
        use genmesh::EmitTriangles;

        let mut obj = obj::Obj::<genmesh::Polygon<obj::IndexTuple>>::load(file.as_ref())
            .expect("Could not load obj");
        obj.load_mtls().unwrap();
        let mut vertices = Vec::with_capacity(obj.position.len());
        for (index, position) in obj.position.into_iter().enumerate() {
            vertices.push(Vertex {
                position_in: position,
                tex_coord_in: obj.texture.get(index).cloned().unwrap_or([0.0, 0.0]),
                normal_in: obj.normal.get(index).cloned().unwrap_or([0.0, 0.0, 0.0]),
            });
        }
        let first_object = obj.objects.into_iter().next().unwrap();
        let first_group = first_object.groups.into_iter().next().unwrap();
        let mut indices: Vec<u32> = Vec::new();
        for poly in first_group.polys {
            poly.emit_triangles(|triangle| {
                indices.push(triangle.x.0 as u32);
                indices.push(triangle.y.0 as u32);
                indices.push(triangle.z.0 as u32);
            });
        }

        Self::from_vertices(device, vertices.into_iter(), indices.into_iter())
    }
    pub fn new_triangle(device: Arc<Device>) -> Arc<Self> {
        let vertex1 = Vertex {
            position_in: [-0.5, -0.5, 0.0],
            normal_in: [0.0, 0.0, 0.0],
            tex_coord_in: [0.0, 0.0],
        };
        let vertex2 = Vertex {
            position_in: [0.0, 0.5, 0.0],
            normal_in: [0.0, 0.0, 0.0],
            tex_coord_in: [1.0, 0.0],
        };
        let vertex3 = Vertex {
            position_in: [0.5, -0.25, 0.0],
            normal_in: [0.0, 0.0, 0.0],
            tex_coord_in: [1.0, 1.0],
        };
        Self::from_vertices(
            device,
            vec![vertex1, vertex2, vertex3].into_iter(),
            vec![].into_iter(),
        )
    }
    fn from_vertices(
        device: Arc<Device>,
        vertices: impl ExactSizeIterator<Item = Vertex>,
        indices: impl ExactSizeIterator<Item = u32>,
    ) -> Arc<Self> {
        let indices = if indices.len() == 0 {
            None
        } else {
            Some(
                CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, indices)
                    .unwrap(),
            )
        };

        let vertex_buffer =
            CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), false, vertices).unwrap();

        Arc::new(Self {
            vertex_buffer,
            indices,
        })
    }
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "#version 450

layout(location = 0) in vec3 position_in;
layout(location = 1) in vec2 tex_coord_in;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;
layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    gl_Position = uniforms.proj * worldview * vec4(position_in, 1.0);
}
"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "#version 450

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"
    }
}
