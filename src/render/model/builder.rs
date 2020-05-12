use super::{Model, ModelHandle, Vertex};
use crate::GameState;
use cgmath::{Euler, Rad, Vector3, Zero};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBuffer, CommandBufferExecFuture},
    device::Queue,
    format::R8G8B8A8Srgb,
    image::{Dimensions, ImmutableImage},
    sync::{GpuFuture, NowFuture},
};

pub struct ModelBuilder<'a> {
    game_state: &'a mut GameState,
    source_or_shape: SourceOrShape<'a>,
    fallback_color: Option<Vector3<f32>>,
    texture: Option<&'a str>,
    position: Vector3<f32>,
    rotation: Euler<Rad<f32>>,
    scale: f32,
}

impl<'a> ModelBuilder<'a> {
    pub(crate) fn new(game_state: &'a mut GameState, source_or_shape: SourceOrShape<'a>) -> Self {
        Self {
            game_state,
            source_or_shape,
            fallback_color: None,
            texture: None,
            position: Vector3::zero(),
            rotation: Euler::new(Rad(0.0), Rad(0.0), Rad(0.0)),
            scale: 1.0,
        }
    }

    pub fn with_fallback_color(mut self, color: impl Into<Vector3<f32>>) -> Self {
        self.fallback_color = Some(color.into());
        self
    }
    pub fn with_texture_from_file(mut self, texture_src: &'a str) -> Self {
        self.texture = Some(texture_src);
        self
    }
    pub fn with_position(mut self, position: impl Into<Vector3<f32>>) -> Self {
        self.position = position.into();
        self
    }
    pub fn with_rotation(mut self, rotation: Euler<Rad<f32>>) -> Self {
        self.rotation = rotation;
        self
    }
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn build(self) -> ModelHandle {
        let position = self.position;
        let rotation = self.rotation;
        let scale = self.scale;

        let (vertices, indices) = self.source_or_shape.into_vertices_and_indices();
        let device = self.game_state.device.clone();
        let indices = indices
            .into_iter()
            .map(|i| {
                CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::all(),
                    false,
                    i.into_iter(),
                )
                .unwrap()
            })
            .collect();

        let vertex_buffer =
            CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), false, vertices.into_iter())
                .unwrap();

        let mut model = Model {
            indices,
            vertex_buffer,
            texture: None,
            texture_future: None,
        };

        if let Some(texture) = self.texture {
            let (tex, tex_future) = load_texture(self.game_state.queue.clone(), texture);
            model.texture = Some(tex);
            // TODO: This is a hack
            // Properly make the pipeline checks all texture futures before rendering
            // Then wait until all buffers are completed, then start the actual render
            //
            // model.texture_future = Some(Box::new(tex_future) as _);
            tex_future
                .then_signal_fence_and_flush()
                .unwrap()
                .wait(None)
                .unwrap();
        }

        // TODO: Immediately set this on the handle
        // TODO: Move this logic away from game_state
        let handle = self.game_state.add_model(Arc::new(model));

        handle.modify(|data| {
            data.position = position;
            data.rotation = rotation;
            data.scale = scale;
        });

        handle
    }
}

pub enum SourceOrShape<'a> {
    Source(&'a str),
    Triangle,
    Rectangle,
}

impl SourceOrShape<'_> {
    pub fn into_vertices_and_indices(self) -> (Vec<Vertex>, Vec<Vec<u32>>) {
        match self {
            SourceOrShape::Source(src) => {
                use genmesh::EmitTriangles;

                let mut obj =
                    obj::Obj::<genmesh::Polygon<obj::IndexTuple>>::load(std::path::Path::new(src))
                        .expect("Could not load obj");
                obj.load_mtls().unwrap();

                let mut vertices = Vec::with_capacity(obj.position.len());
                for (index, position) in obj.position.into_iter().enumerate() {
                    vertices.push(Vertex {
                        position_in: position,
                        tex_coord_in: obj.texture.get(index).cloned().unwrap_or([-1.0, -1.0]),
                        normal_in: obj.normal.get(index).cloned().unwrap_or([0.0, 0.0, 0.0]),
                    });
                }

                let mut indices: Vec<Vec<u32>> =
                    Vec::with_capacity(obj.objects.iter().map(|o| o.groups.len()).sum());
                for object in obj.objects {
                    for group in object.groups {
                        let mut index_group = Vec::new();
                        for poly in group.polys {
                            poly.emit_triangles(|triangle| {
                                index_group.push(triangle.x.0 as u32);
                                index_group.push(triangle.y.0 as u32);
                                index_group.push(triangle.z.0 as u32);
                            });
                        }
                        indices.push(index_group);
                    }
                }

                (vertices, indices)
            }
            SourceOrShape::Rectangle => {
                let mut vertices = Vec::new();
                vertices.push(Vertex {
                    position_in: [-0.5, -0.5, 0.0],
                    normal_in: [0.0, 0.0, 1.0],
                    tex_coord_in: [1.0, 1.0],
                });
                vertices.push(Vertex {
                    position_in: [0.5, -0.5, 0.0],
                    normal_in: [0.0, 0.0, 1.0],
                    tex_coord_in: [0.0, 1.0],
                });
                vertices.push(Vertex {
                    position_in: [0.5, 0.5, 0.0],
                    normal_in: [0.0, 0.0, 1.0],
                    tex_coord_in: [0.0, 0.0],
                });
                vertices.push(Vertex {
                    position_in: [-0.5, 0.5, 0.0],
                    normal_in: [0.0, 0.0, 1.0],
                    tex_coord_in: [1.0, 0.0],
                });
                let indices = vec![0, 1, 2, 0, 2, 3];
                (vertices, vec![indices])
            }
            SourceOrShape::Triangle => {
                let vertex1 = Vertex {
                    position_in: [-0.5, -0.25, 0.0],
                    normal_in: [0.0, 0.0, 0.0],
                    tex_coord_in: [0.0, 0.0],
                };
                let vertex2 = Vertex {
                    position_in: [0.0, 0.5, 0.0],
                    normal_in: [0.0, 0.0, 0.0],
                    tex_coord_in: [1.0, 0.0],
                };
                let vertex3 = Vertex {
                    position_in: [0.25, -0.1, 0.0],
                    normal_in: [0.0, 0.0, 0.0],
                    tex_coord_in: [1.0, 1.0],
                };
                (vec![vertex1, vertex2, vertex3], vec![])
            }
        }
    }
}

fn load_texture(
    queue: Arc<Queue>,
    path: &str,
) -> (
    Arc<ImmutableImage<R8G8B8A8Srgb>>,
    CommandBufferExecFuture<NowFuture, AutoCommandBuffer>,
) {
    use std::fs::File;
    let file = File::open(path).unwrap();
    let decoder = png::Decoder::new(file);
    let (info, mut reader) = decoder.read_info().unwrap();
    let dimensions = Dimensions::Dim2d {
        width: info.width,
        height: info.height,
    };
    let mut image_data = Vec::new();
    image_data.resize((info.width * info.height * 4) as usize, 0);
    reader.next_frame(&mut image_data).unwrap();

    ImmutableImage::from_iter(image_data.iter().cloned(), dimensions, R8G8B8A8Srgb, queue).unwrap()
}
