use super::{Euler, Vector3};

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Matrix4(pub [[f32; 4]; 4]);

impl Matrix4 {
    pub fn new(
        m00: f32,
        m01: f32,
        m02: f32,
        m03: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m20: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m30: f32,
        m31: f32,
        m32: f32,
        m33: f32,
    ) -> Self {
        Self([
            [m00, m01, m02, m03],
            [m10, m11, m12, m13],
            [m20, m21, m22, m23],
            [m30, m31, m32, m33],
        ])
    }

    pub fn identity() -> Self {
        use cgmath::SquareMatrix;
        Self(cgmath::Matrix4::identity().into())
    }
    pub fn from_translation(position: Vector3) -> Self {
        Self(
            cgmath::Matrix4::from_translation(cgmath::Vector3::new(
                position.x, position.y, position.z,
            ))
            .into(),
        )
    }
    pub fn from_scale(scale: f32) -> Self {
        Self(cgmath::Matrix4::from_scale(scale).into())
    }
    pub fn zero() -> Self {
        use cgmath::Zero;
        Self(cgmath::Matrix4::zero().into())
    }
    pub fn position(self) -> Vector3 {
        Vector3::new(self.0[0][0], self.0[0][1], self.0[0][2])
    }
    pub fn look_at(eye: Vector3, center: Vector3, up: Vector3) -> Self {
        Self(
            cgmath::Matrix4::look_at(
                eye.into(),
                center.into(),
                cgmath::Vector3::new(up.x, up.y, up.z),
            )
            .into(),
        )
    }
}

impl From<Euler> for Matrix4 {
    fn from(src: Euler) -> Self {
        use cgmath::Angle;

        let (sx, cx) = cgmath::Rad::sin_cos(cgmath::Rad(src.x.0));
        let (sy, cy) = cgmath::Rad::sin_cos(cgmath::Rad(src.y.0));
        let (sz, cz) = cgmath::Rad::sin_cos(cgmath::Rad(src.z.0));

        #[cfg_attr(rustfmt, rustfmt_skip)]
        Matrix4::new(
            cy * cz, cx * sz + sx * sy * cz, sx * sz - cx * sy * cz, 0.0,
            -cy * sz, cx * cz - sx * sy * sz, sx * cz + cx * sy * sz, 0.0,
            sy, -sx * cy, cx * cy, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    }
}

impl std::ops::Mul for Matrix4 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Matrix4((cgmath::Matrix4::from(self.0) * cgmath::Matrix4::from(rhs.0)).into())
    }
}

impl Into<[[f32; 4]; 4]> for Matrix4 {
    fn into(self) -> [[f32; 4]; 4] {
        self.0
    }
}
