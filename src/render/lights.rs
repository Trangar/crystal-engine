use crate::model::vs as model_vs;
use cgmath::{Vector3, Zero};

/// A direction lightsource in the world.
///
/// Note: lights coming from the sky are going down, so their direction would be `Vector3::new(0.0,
/// -1.0, 0.0)`
///
/// For more information, see the amazing tutorial at [https://learnopengl.com/Lighting/Colors](https://learnopengl.com/Lighting/Colors)
pub struct DirectionalLight {
    /// The direction of the light source
    pub direction: Vector3<f32>,
    /// The color of the light source.
    pub color: LightColor,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vector3::zero(),
            color: LightColor::default(),
        }
    }
}

/// A pointlight in the world.
///
/// Note: Not implemented yet
///
/// For more information, see the amazing tutorial at [https://learnopengl.com/Lighting/Colors](https://learnopengl.com/Lighting/Colors)
pub struct PointLight {
    /// The position of the light in the world.
    pub position: Vector3<f32>,
    /// The color of the light in the world.
    pub color: LightColor,

    /// The attenuation of the light, or how much the light decays over a distance.
    /// `PointLightAttenuation` implements `Default` so you can take a good initial value, or you
    /// can tune this until the end of time.
    pub attenuation: PointLightAttenuation,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            color: LightColor::default(),
            attenuation: PointLightAttenuation::default(),
        }
    }
}

/// The color of the light. This is divided in 3 fields: ambient, diffuse and specular. See each field for the definition.
///
/// For more information, see the amazing tutorial at [https://learnopengl.com/Lighting/Colors](https://learnopengl.com/Lighting/Colors)
pub struct LightColor {
    /// Even when it is dark there is usually still some light somewhere in the world (the moon, a distant light) so objects are almost never completely dark.
    /// To simulate this we use an ambient lighting constant that always gives the object some color.
    ///
    /// This will be merged with the ambient factor of the material of your model.
    pub ambient: Vector3<f32>,

    /// Diffuse light simulates the directional impact a light object has on an object.
    /// This is the most visually significant component of the lighting model.
    /// The more a part of an object faces the light source, the brighter it becomes.
    ///
    /// This will be merged with the diffuse factor of the material of your model.
    pub diffuse: Vector3<f32>,

    /// Specular light simulates the bright spot of a light that appears on shiny objects.
    /// Specular highlights are more inclined to the color of the light than the color of the object.
    ///
    /// This will be merged with the specular factor of the material of your model.
    pub specular: Vector3<f32>,
}

impl Default for LightColor {
    fn default() -> Self {
        LightColor {
            ambient: Vector3::zero(),
            diffuse: Vector3::zero(),
            specular: Vector3::zero(),
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
///
/// This should mirror most functions that exist on [Vec]. If you're missing a function, feel free to open an issue or PR!
pub struct FixedVec<T> {
    pub(crate) data: [T; LIGHT_COUNT],
    len: usize,
}

impl FixedVec<DirectionalLight> {
    pub(crate) fn to_shader_value(&self) -> (i32, [model_vs::ty::DirectionalLight; LIGHT_COUNT]) {
        let result = array_init::array_init(|i| {
            let light = &self.data[i];
            model_vs::ty::DirectionalLight {
                direction_x: light.direction.x,
                direction_y: light.direction.y,
                direction_z: light.direction.z,
                color_ambient_r: light.color.ambient.x,
                color_ambient_g: light.color.ambient.y,
                color_ambient_b: light.color.ambient.z,
                color_diffuse_r: light.color.diffuse.x,
                color_diffuse_g: light.color.diffuse.x,
                color_diffuse_b: light.color.diffuse.z,
                color_specular_r: light.color.specular.x,
                color_specular_g: light.color.specular.y,
                color_specular_b: light.color.specular.z,
            }
        });
        (self.len() as i32, result)
    }
}

impl<T: Default> FixedVec<T> {
    pub(crate) fn new() -> Self {
        Self {
            // safe because this is a `DirectionalLight` which has a size and is not a reference
            data: array_init::array_init(|_| T::default()),
            len: 0,
        }
    }
}

// Implementation of relevant std::vec::Vec functions
impl<T> FixedVec<T> {
    /// Extracts a slice containing the entire fixed vec.
    ///
    /// Equivalent to `&s[..]`.
    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.len]
    }

    /// Extracts a mutable slice of the entire fixed vec.
    ///
    /// Equivalent to `&mut s[..]`.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data[..self.len]
    }

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
