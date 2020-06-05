use crate::model::{Material, Vertex};
use std::borrow::Cow;

#[cfg(feature = "format-fbx")]
mod fbx;
#[cfg(feature = "format-obj")]
mod obj;

#[derive(Debug)]
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
    pub fn parse(&self) -> ParsedModel {
        match self {
            #[cfg(feature = "format-obj")]
            SourceOrShape::Obj(src) => obj::load(src),

            #[cfg(feature = "format-fbx")]
            SourceOrShape::Fbx(src) => fbx::load(src).expect("Could not load FBX").into(),
            SourceOrShape::Rectangle => RECTANGLE.clone().into(),
            SourceOrShape::Triangle => TRIANGLE.clone().into(),
            SourceOrShape::Dummy(_) => unreachable!(),
        }
    }
}

#[derive(Default)]
pub struct ParsedModel {
    pub vertices: Option<CowVertex>,
    pub parts: Vec<ParsedModelPart>,
}
#[derive(Default)]
pub struct ParsedModelPart {
    pub vertices: Option<CowVertex>,
    pub index: Cow<'static, [u32]>,
    pub material: Option<Material>,
    pub texture: Option<ParsedTexture>,
}

pub struct ParsedTexture {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,
}

impl From<CowVertex> for ParsedModel {
    fn from(vertex: CowVertex) -> Self {
        Self {
            vertices: Some(vertex),
            parts: Vec::new(),
        }
    }
}

impl From<(CowVertex, CowIndex)> for ParsedModel {
    fn from((vertex, indices): (CowVertex, CowIndex)) -> Self {
        Self {
            vertices: Some(vertex),
            parts: indices.iter().map(|index| index.into()).collect(),
        }
    }
}

impl<'a> From<&'a Cow<'static, [u32]>> for ParsedModelPart {
    fn from(index: &'a Cow<'static, [u32]>) -> Self {
        Self {
            index: index.clone(),
            ..Default::default()
        }
    }
}

impl From<Vec<u32>> for ParsedModelPart {
    fn from(indices: Vec<u32>) -> Self {
        Self {
            index: indices.into(),
            ..Default::default()
        }
    }
}

static RECTANGLE: (CowVertex, CowIndex) = (
    Cow::Borrowed(&[
        Vertex {
            position_in: [-0.5, -0.5, 0.0],
            normal_in: [0.0, 0.0, 1.0],
            tex_coord_in: [0.0, 1.0],
        },
        Vertex {
            position_in: [0.5, -0.5, 0.0],
            normal_in: [0.0, 0.0, 1.0],
            tex_coord_in: [1.0, 1.0],
        },
        Vertex {
            position_in: [0.5, 0.5, 0.0],
            normal_in: [0.0, 0.0, 1.0],
            tex_coord_in: [1.0, 0.0],
        },
        Vertex {
            position_in: [-0.5, 0.5, 0.0],
            normal_in: [0.0, 0.0, 1.0],
            tex_coord_in: [0.0, 0.0],
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
