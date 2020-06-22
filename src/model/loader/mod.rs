use crate::{
    model::{Material, Vertex},
    state::ModelError,
};

#[cfg(feature = "format-fbx")]
pub mod fbx;
#[cfg(feature = "format-obj")]
pub mod obj;

pub enum SourceOrShape<'a> {
    #[cfg(feature = "format-obj")]
    Obj(&'a str),
    #[cfg(feature = "format-fbx")]
    Fbx(&'a str),
    Triangle,
    Rectangle,
    Custom(ParsedModel),
}

impl SourceOrShape<'_> {
    pub fn parse(self) -> Result<ParsedModel, ModelError> {
        match self {
            #[cfg(feature = "format-obj")]
            SourceOrShape::Obj(src) => obj::load(src).map_err(ModelError::Obj),

            #[cfg(feature = "format-fbx")]
            SourceOrShape::Fbx(src) => fbx::load(src).map(Into::into),
            SourceOrShape::Rectangle => Ok(RECTANGLE.into()),
            SourceOrShape::Triangle => Ok(TRIANGLE.into()),
            SourceOrShape::Custom(model) => Ok(model),
        }
    }
}

/// A parsed model, ready to be imported into the engine.
pub struct ParsedModel {
    /// The vertices of the parsed model. Either this field or one of the parts' vertices must be filled in.
    pub vertices: Option<Vec<Vertex>>,
    /// The parts of this model. Each part is a sub-model, e.g. the wheels of a car that can rotate independently, but still belong to the car model.
    pub parts: Vec<ParsedModelPart>,
}

/// A part of the parsed model. Each part is a sub-model, e.g. the wheels of a car that can rotate independently, but still belong to the car model.
#[derive(Default)]
pub struct ParsedModelPart {
    /// The vertices of this part. Either this field or the parsed model vertices must be filled in.
    pub vertices: Option<Vec<Vertex>>,
    /// The indices of this part.
    pub index: Vec<u32>,
    /// The material of this part
    pub material: Option<Material>,
    /// The texture of this part
    pub texture: Option<ParsedTexture>,
}

/// The texture of a parsed model part
pub struct ParsedTexture {
    /// The width of the parsed texture
    pub width: u32,
    /// The height of the parsed texture
    pub height: u32,
    /// The RGBA data of the parsed texture. This is in the format `[r, g, b, a, r, g, b, a, ...]`. This vec should have exactly `4 * width * height` entries.
    pub rgba_data: Vec<u8>,
}

impl From<Vec<Vertex>> for ParsedModel {
    fn from(vertex: Vec<Vertex>) -> Self {
        Self {
            vertices: Some(vertex),
            parts: Vec::new(),
        }
    }
}

impl<'a> From<&'a [Vertex]> for ParsedModel {
    fn from(vertex: &'a [Vertex]) -> Self {
        Self {
            vertices: Some(vertex.iter().copied().collect()),
            parts: Vec::new(),
        }
    }
}

impl<'a> From<(&'a [Vertex], &'a [u32])> for ParsedModel {
    fn from((vertex, index): (&'a [Vertex], &'a [u32])) -> Self {
        Self {
            vertices: Some(vertex.iter().copied().collect()),
            parts: vec![index.into()],
        }
    }
}

impl<'a> From<&'a [u32]> for ParsedModelPart {
    fn from(index: &'a [u32]) -> Self {
        Self {
            index: index.iter().copied().collect(),
            ..Default::default()
        }
    }
}

impl From<Vec<u32>> for ParsedModelPart {
    fn from(index: Vec<u32>) -> Self {
        Self {
            index,
            ..Default::default()
        }
    }
}

static RECTANGLE: (&[Vertex], &[u32]) = (
    &[
        Vertex {
            position: [-0.5, -0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coord: [0.0, 1.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coord: [1.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coord: [1.0, 0.0],
        },
        Vertex {
            position: [-0.5, 0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coord: [0.0, 0.0],
        },
    ],
    &[0, 1, 2, 0, 2, 3],
);

static TRIANGLE: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.25, 0.0],
        normal: [0.0, 0.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, 0.5, 0.0],
        normal: [0.0, 0.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [0.25, -0.1, 0.0],
        normal: [0.0, 0.0, 0.0],
        tex_coord: [1.0, 1.0],
    },
];
