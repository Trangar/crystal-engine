mod deg;
mod euler;
mod matrix4;
mod rad;
mod vector2;
mod vector3;

pub use self::{
    deg::Deg, euler::Euler, matrix4::Matrix4, rad::Rad, vector2::Vector2, vector3::Vector3,
};

pub fn perspective(fovy: Rad, aspect: f32, near: f32, far: f32) -> Matrix4 {
    assert!(
        fovy > Rad::zero(),
        "The vertical field of view cannot be below zero, found: {:?}",
        fovy
    );
    assert!(
        fovy < Rad::turn_div_2(),
        "The vertical field of view cannot be greater than a half turn, found: {:?}",
        fovy
    );
    assert!(
        aspect > 0.0,
        "The aspect ratio cannot be below zero, found: {:?}",
        aspect
    );
    assert!(
        near > 0.0,
        "The near plane distance cannot be below zero, found: {:?}",
        near
    );
    assert!(
        far > 0.0,
        "The far plane distance cannot be below zero, found: {:?}",
        far
    );
    assert!(
        far > near,
        "The far plane cannot be closer than the near plane, found: far: {:?}, near: {:?}",
        far,
        near
    );

    let two = 2.0;
    let f = Rad::cot(fovy / two);

    let c0r0 = f / aspect;
    let c0r1 = 0.0;
    let c0r2 = 0.0;
    let c0r3 = 0.0;

    let c1r0 = 0.0;
    let c1r1 = f;
    let c1r2 = 0.0;
    let c1r3 = 0.0;

    let c2r0 = 0.0;
    let c2r1 = 0.0;
    let c2r2 = (far + near) / (near - far);
    let c2r3 = -1.0;

    let c3r0 = 0.0;
    let c3r1 = 0.0;
    let c3r2 = (two * far * near) / (near - far);
    let c3r3 = 0.0;

    #[cfg_attr(rustfmt, rustfmt_skip)]
    Matrix4::new(
        c0r0, c0r1, c0r2, c0r3,
        c1r0, c1r1, c1r2, c1r3,
        c2r0, c2r1, c2r2, c2r3,
        c3r0, c3r1, c3r2, c3r3,
    )
}
