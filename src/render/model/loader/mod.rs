use crate::render::{Material, Vertex};
use std::borrow::Cow;

#[cfg(feature = "format-fbx")]
mod fbx;
#[cfg(feature = "format-obj")]
mod obj;

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

pub type CowVertex = Cow<'static, [Vertex]>;
pub type CowIndex = Cow<'static, [Cow<'static, [u32]>]>;

impl SourceOrShape<'_> {
    pub fn parse(self) -> ParsedModel {
        match self {
            #[cfg(feature = "format-obj")]
            SourceOrShape::Obj(src) => obj::load(src),

            #[cfg(feature = "format-fbx")]
            SourceOrShape::Fbx(src) => fbx::load(src),
            SourceOrShape::Rectangle => RECTANGLE.clone().into(),
            SourceOrShape::Triangle => TRIANGLE.clone().into(),
            SourceOrShape::Dummy(_) => unreachable!(),
        }
    }
}

pub struct ParsedModel {
    pub vertices: CowVertex,
    pub parts: Vec<ParsedModelPart>,
}

impl From<CowVertex> for ParsedModel {
    fn from(vertex: CowVertex) -> Self {
        Self {
            vertices: vertex,
            parts: Vec::new(),
        }
    }
}

impl From<(CowVertex, CowIndex)> for ParsedModel {
    fn from((vertex, indices): (CowVertex, CowIndex)) -> Self {
        Self {
            vertices: vertex,
            parts: indices
                .iter()
                .map(|index| ParsedModelPart {
                    index: index.clone(),
                    material: None,
                })
                .collect(),
        }
    }
}

pub struct ParsedModelPart {
    pub index: Cow<'static, [u32]>,
    pub material: Option<Material>,
}

static RECTANGLE: (CowVertex, CowIndex) = (
    Cow::Borrowed(&[
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
    ]),
    Cow::Borrowed(&[Cow::Borrowed(&[0, 1, 2, 0, 2, 3])]),
);

static TRIANGLE: CowVertex = Cow::Borrowed(&[
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