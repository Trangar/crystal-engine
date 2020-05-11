use super::model::vs as model_vs;
use cgmath::{Vector3, Zero};

/// A direction lightsource in the world.
///
/// Note: lights coming from the sky are going down, so their direction would be `Vector3::new(0.0,
/// -1.0, 0.0)`
pub struct DirectionalLight {
    /// The direction of the light source
    pub direction: Vector3<f32>,
    /// The color of the light source.
    pub color: Vector3<f32>,
    /// The brightness of the light source.
    pub brightness: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vector3::zero(),
            color: Vector3::zero(),
            brightness: 1.0,
        }
    }
}

/// A pointlight in the world.
///
/// Note: Not implemented yet
pub struct PointLight {
    /// The position of the light in the world.
    pub position: Vector3<f32>,
    /// The color of the light in the world.
    pub color: Vector3<f32>,

    /// The attenuation of the light, or how much the light decays over a distance.
    /// `PointLightAttenuation` implements `Default` so you can take a good initial value, or you
    /// can tune this until the end of time.
    pub attenuation: PointLightAttenuation,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            color: Vector3::zero(),
            attenuation: PointLightAttenuation::default(),
        }
    }
}

/// The attenuation of the pointlight, or how much the light impacts objects based on their
/// distance.
pub struct PointLightAttenuation {
    /// The constant or base attenuation. This will always reduce the effect of the light source,
    /// regardless on how far away the object is.
    ///
    /// This can also be seen as `brightness`.
    pub constant: f32,

    /// The linear attenuation of the light. This will reduce the effect of the light source if the
    /// model is far away
    pub linear: f32,

    /// The quadratic attenuation of the light. This will greatly reduce the effect of the light
    /// source if the model is far away.
    pub quadratic: f32,
}

impl Default for PointLightAttenuation {
    fn default() -> Self {
        // Values taken from https://learnopengl.com/Lighting/Multiple-lights
        Self {
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
        }
    }
}

/// The state of the lights in the game. Lights come in two flavors.
///
/// Directional lights: light sources that shine in a certain direction, e.g. the sun.
///
/// Point lights: lights that shine equally in all directions, e.g. a lightbulb.
///
/// Note: lights are limited to 100 of each type. Currently the shaders do not support more than
/// 100 light sources at a time. Please open an issue if you need more light sources.
pub struct LightState {
    /// A `FixedVec` of directional lights
    pub directional: FixedVec<DirectionalLight>,
    /// A `FixedVec` of point lights.
    ///
    /// Note: not implemented yet
    pub point: FixedVec<PointLight>,
}

impl LightState {
    pub(crate) fn new() -> Self {
        Self {
            directional: FixedVec::<DirectionalLight>::new(),
            point: FixedVec::<PointLight>::new(),
        }
    }
}

const LIGHT_COUNT: usize = 100;
/// A fixed vec of light sources. This is limited to 100 entries because of a limitation in the way
/// Crystal's shaders are implemented. Please open an issue if you need more light sources.
pub struct FixedVec<T> {
    pub(crate) data: [T; LIGHT_COUNT],
    len: usize,
}

impl FixedVec<DirectionalLight> {
    pub(crate) fn new() -> Self {
        use std::mem::{transmute, MaybeUninit};

        let mut data: [MaybeUninit<DirectionalLight>; LIGHT_COUNT] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for elem in &mut data[..] {
            *elem = MaybeUninit::new(DirectionalLight::default());
        }

        Self {
            data: unsafe { transmute(data) },
            len: 0,
        }
    }
    pub(crate) fn to_shader_value(&self) -> (i32, [model_vs::ty::DirectionalLight; LIGHT_COUNT]) {
        let mut result = [model_vs::ty::DirectionalLight {
            direction: [0.0, -1.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            _dummy0: [0, 0, 0, 0],
        }; 100];

        for (light, shader_light) in self.data.iter().take(self.len).zip(result.iter_mut()) {
            shader_light.direction = light.direction.into();
            shader_light.color = [
                light.color.x,
                light.color.y,
                light.color.z,
                light.brightness,
            ];
        }

        (self.len() as i32, result)
    }
}

impl FixedVec<PointLight> {
    pub(crate) fn new() -> Self {
        use std::mem::{transmute, MaybeUninit};

        let mut data: [MaybeUninit<PointLight>; LIGHT_COUNT] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for elem in &mut data[..] {
            *elem = MaybeUninit::new(PointLight::default());
        }

        Self {
            data: unsafe { transmute(data) },
            len: 0,
        }
    }
}

impl<T> FixedVec<T> {
    /// Get the amount of lights that are stored in this `FixedVec`.
    ///
    /// Note: this is always 100 or lower.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if this `FixedVec` is empty.
    ///
    /// This is an alias for `self.len() == 0`
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Add a new light to this `FixedVec`.
    ///
    /// This will panic if more than 100 lights are added.
    pub fn push(&mut self, t: T) {
        assert!(self.len() < LIGHT_COUNT);
        self.data[self.len] = t;
        self.len += 1;
    }

    /// Remove the last light source from this `FixedVec`.
    ///
    /// This will panic if the `FixedVec` is empty.
    pub fn pop(&mut self) {
        assert!(self.len > 0);
        self.len -= 1;
    }
}

impl<T> std::ops::Index<usize> for FixedVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        assert!(index < self.len());
        &self.data[index]
    }
}

impl<T> std::ops::IndexMut<usize> for FixedVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        assert!(index < self.len());
        &mut self.data[index]
    }
}
