mod element;

pub use self::element::GuiElement;

/*#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub offset: [f32; 2],
    pub tex_coord: [f32; 2],
}
vulkano::impl_vertex!(Vertex, offset, tex_coord);*/

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub position_in: [f32; 3],
    pub normal_in: [f32; 3],
    pub tex_coord_in: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position_in, normal_in, tex_coord_in);

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "#version 450

layout(location = 0) in vec3 position_in;
layout(location = 1) in vec3 normal_in;
layout(location = 2) in vec2 tex_coord_in;

layout(location = 0) out vec2 fragment_tex_coord;
// layout(location = 1) out vec3 fragment_normal;

struct DirectionalLight {
    float direction_x;
    float direction_y;
    float direction_z;
    float color_ambient_r;
    float color_ambient_g;
    float color_ambient_b;
    float color_diffuse_r;
    float color_diffuse_g;
    float color_diffuse_b;
    float color_specular_r;
    float color_specular_g;
    float color_specular_b;
};

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    DirectionalLight[100] lights;
    int lightCount;

    float camera_x;
    float camera_y;
    float camera_z;

    float material_ambient_r;
    float material_ambient_g;
    float material_ambient_b;
    float material_diffuse_r;
    float material_diffuse_g;
    float material_diffuse_b;
    float material_specular_r;
    float material_specular_g;
    float material_specular_b;
    float material_shininess;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    gl_Position = uniforms.proj * worldview * vec4(position_in, 1.0);
    fragment_tex_coord = tex_coord_in;

    // fragment_normal = transpose(inverse(mat3(worldview))) * normal_in;
}
"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "#version 450

layout(location = 0) in vec2 fragment_tex_coord;
// layout(location = 1) in vec3 fragment_normal;

layout(location = 0) out vec4 f_color;

struct DirectionalLight {
    float direction_x;
    float direction_y;
    float direction_z;
    float color_ambient_r;
    float color_ambient_g;
    float color_ambient_b;
    float color_diffuse_r;
    float color_diffuse_g;
    float color_diffuse_b;
    float color_specular_r;
    float color_specular_g;
    float color_specular_b;
};

layout(set = 0, binding = 1) uniform sampler2D tex;
layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    DirectionalLight[100] lights;
    int lightCount;

    float camera_x;
    float camera_y;
    float camera_z;

    float material_ambient_r;
    float material_ambient_g;
    float material_ambient_b;
    float material_diffuse_r;
    float material_diffuse_g;
    float material_diffuse_b;
    float material_specular_r;
    float material_specular_g;
    float material_specular_b;
    float material_shininess;
} uniforms;

vec3 max_member(vec3 lhs, vec3 rhs) {
    return vec3(
        max(lhs.x, rhs.x),
        max(lhs.y, rhs.y),
        max(lhs.z, rhs.z)
    );
}

vec4 min_member(vec4 lhs, vec4 rhs) {
    return vec4(
        min(lhs.x, rhs.x),
        min(lhs.y, rhs.y),
        min(lhs.z, rhs.z),
        min(lhs.w, rhs.w)
    );
}

vec4 CalcDirLight(DirectionalLight light, vec4 tex_color, vec3 normal, vec3 viewDir)
{
    vec3 direction = vec3(light.direction_x, light.direction_y, light.direction_z);
    vec3 ambient = vec3(light.color_ambient_r, light.color_ambient_g, light.color_ambient_b);
    vec3 diffuse = vec3(light.color_diffuse_r, light.color_diffuse_g, light.color_diffuse_b);
    vec3 specular = vec3(light.color_specular_r, light.color_specular_g, light.color_specular_b);

    vec3 material_ambient = vec3(uniforms.material_ambient_r, uniforms.material_ambient_g, uniforms.material_ambient_b);
    vec3 material_diffuse = vec3(uniforms.material_diffuse_r, uniforms.material_diffuse_g, uniforms.material_diffuse_b);
    vec3 material_specular = vec3(uniforms.material_specular_r, uniforms.material_specular_g, uniforms.material_specular_b);

    vec3 lightDir = normalize(-direction);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), uniforms.material_shininess);
    // combine results
    ambient  = ambient  * material_ambient;
    diffuse  = diffuse  * diff * material_diffuse;
    specular = specular * spec * material_specular;
    return tex_color * min_member(vec4(ambient + diffuse + specular, 1.0), vec4(1.0, 1.0, 1.0, 1.0));
} 


void main() {
    if(fragment_tex_coord.x < 0.0 && fragment_tex_coord.y < 0.0) {
        f_color = vec4(uniforms.material_ambient_r, uniforms.material_ambient_g, uniforms.material_ambient_b, 1);
    } else {
        f_color = texture(tex, fragment_tex_coord);
    }

    /*vec3 camera_pos = vec3(uniforms.camera_x, uniforms.camera_y, uniforms.camera_z);
    
    for(int i = 0; i < uniforms.lightCount; i++) {
        f_color = CalcDirLight(
            uniforms.lights[i],
            f_color,
            fragment_normal,
            camera_pos
        );
    }*/
}
"
    }
}
/*
pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "#version 450

layout(location = 0) in vec2 offset;
layout(location = 1) in vec2 tex_coord;

layout(location = 0) out vec2 fragment_tex_coord;

layout(set = 0, binding = 0) uniform Data {
    vec2 screen_size;
    vec2 position;
    vec2 size;
} uniforms;

void main() {
    // vec2 pos = uniforms.position + offset * uniforms.size;

    // gl_Position = vec4(pos / (uniforms.screen_size * 2) - 1, 0.0, 0.0);
    gl_Position = vec4(offset, 0.0, 0.0);
    fragment_tex_coord = tex_coord;
}
"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "#version 450

layout(location = 0) in vec2 fragment_tex_coord;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D tex;
layout(set = 0, binding = 0) uniform Data {
    vec2 screen_size;
    vec2 position;
    vec2 size;
} uniforms;

void main() {
    f_color = vec4(1.0, 1.0, 1.0, 1.0); // texture(tex, fragment_tex_coord);
}
"
    }
}*/
