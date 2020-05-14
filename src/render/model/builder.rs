use super::{Model, ModelHandle, Vertex};
use crate::GameState;
use cgmath::{Euler, Rad, Vector3, Zero};
use parking_lot::RwLock;
use std::{borrow::Cow, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBuffer, CommandBufferExecFuture},
    device::Queue,
    format::R8G8B8A8Srgb,
    image::{Dimensions, ImmutableImage},
    sync::NowFuture,
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
                    i.into_iter().map(|i| *i),
                )
                .unwrap()
            })
            .collect();

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            device,
            BufferUsage::all(),
            false,
            vertices.into_iter().map(|v| *v),
        )
        .unwrap();

        let mut model = Model {
            indices,
            vertex_buffer,
            texture: None,
            texture_future: RwLock::new(None),
        };

        if let Some(texture) = self.texture {
            let (tex, tex_future) = load_texture(self.game_state.queue.clone(), texture);
            model.texture = Some(tex);
            *model.texture_future.write() = Some(Box::new(tex_future) as _);
        }

        let (handle, id, data) =
            ModelHandle::from_model(Arc::new(model), self.game_state.model_handle_sender.clone());
        self.game_state.model_handles.insert(id, data);

        // TODO: Immediately set this on the handle
        handle.modify(|data| {
            data.position = position;
            data.rotation = rotation;
            data.scale = scale;
        });

        handle
    }
}

pub enum SourceOrShape<'a> {
    #[cfg(feature = "format-obj")]
    Obj(&'a str),
    #[cfg(feature = "format-fbx")]
    Fbx(&'a str),
    Triangle,
    Rectangle,

    // This dummy is needed to prevent compile issues when no formats are enabled
    // This should never be constructed
    #[allow(unused)]
    Dummy(std::marker::PhantomData<&'a ()>),
}

type CowVertex = Cow<'static, [Vertex]>;
type CowIndex = Cow<'static, [Cow<'static, [u32]>]>;

impl SourceOrShape<'_> {
    pub fn into_vertices_and_indices(self) -> (CowVertex, CowIndex) {
        match self {
            #[cfg(feature = "format-obj")]
            SourceOrShape::Obj(src) => load_obj_file(src),

            #[cfg(feature = "format-fbx")]
            SourceOrShape::Fbx(src) => load_fbx_file(src),
            SourceOrShape::Rectangle => load_rectangle_shape(),
            SourceOrShape::Triangle => load_triangle_shape(),
            SourceOrShape::Dummy(_) => unreachable!(),
        }
    }
}

#[cfg(feature = "format-obj")]
fn load_obj_file(src: &str) -> (CowVertex, CowIndex) {
    use genmesh::EmitTriangles;

    let mut obj = obj::Obj::<genmesh::Polygon<obj::IndexTuple>>::load(std::path::Path::new(src))
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

    let mut indices: Vec<_> = Vec::with_capacity(obj.objects.iter().map(|o| o.groups.len()).sum());
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
            indices.push(index_group.into());
        }
    }

    (vertices.into(), indices.into())
}
#[cfg(feature = "format-fbx")]
fn load_fbx_file(src: &str) -> (CowVertex, CowIndex) {
    use fbxcel_dom::any::AnyDocument;

    let file = std::fs::File::open(src).expect("Failed to open file");
    // You can also use raw `file`, but do buffering for better efficiency.
    let reader = std::io::BufReader::new(file);

    // Use `from_seekable_reader` for readers implementing `std::io::Seek`.
    // To use readers without `std::io::Seek` implementation, use `from_reader`
    // instead.
    match AnyDocument::from_seekable_reader(reader).expect("Failed to load document") {
        AnyDocument::V7400(_fbx_ver, doc) => {
            for object in doc.objects().map(|o| o.get_typed()) {
                println!("{:?}", object);
            }
            // You got a document. You can do what you want.
            unimplemented!()
        }
        // `AnyDocument` is nonexhaustive.
        // You should handle unknown document versions case.
        _ => panic!("Got FBX document of unsupported version"),
    }
}

fn load_rectangle_shape() -> (CowVertex, CowIndex) {
    static VERTICES: CowVertex = Cow::Borrowed(&[
        Vertex {
            position_in: [-0.5, -0.5, 0.0],
            normal_in: [0.0, 0.0, 1.0],
            tex_coord_in: [1.0, 1.0],
        },
        Vertex {
            position_in: [0.5, -0.5, 0.0],
            normal_in: [0.0, 0.0, 1.0],
            tex_coord_in: [0.0, 1.0],
        },
        Vertex {
            position_in: [0.5, 0.5, 0.0],
            normal_in: [0.0, 0.0, 1.0],
            tex_coord_in: [0.0, 0.0],
        },
        Vertex {
            position_in: [-0.5, 0.5, 0.0],
            normal_in: [0.0, 0.0, 1.0],
            tex_coord_in: [1.0, 0.0],
        },
    ]);
    static INDICES: CowIndex = Cow::Borrowed(&[Cow::Borrowed(&[0, 1, 2, 0, 2, 3])]);
    (VERTICES.clone(), INDICES.clone())
}

fn load_triangle_shape() -> (CowVertex, CowIndex) {
    static VERTICES: CowVertex = Cow::Borrowed(&[
        Vertex {
            position_in: [-0.5, -0.25, 0.0],
            normal_in: [0.0, 0.0, 0.0],
            tex_coord_in: [0.0, 0.0],
        },
        Vertex {
            position_in: [0.0, 0.5, 0.0],
            normal_in: [0.0, 0.0, 0.0],
            tex_coord_in: [1.0, 0.0],
        },
        Vertex {
            position_in: [0.25, -0.1, 0.0],
            normal_in: [0.0, 0.0, 0.0],
            tex_coord_in: [1.0, 1.0],
        },
    ]);
    (VERTICES.clone(), Cow::Borrowed(&[]))
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
