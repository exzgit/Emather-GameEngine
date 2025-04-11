struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * uniforms.model * vec4<f32>(in.position, 1.0);
    
    // Transform position and normal to world space
    let model_matrix = uniforms.model;
    out.position = (model_matrix * vec4<f32>(in.position, 1.0)).xyz;
    
    // Transform normal to world space (ignoring scale)
    // For proper normal transformation we should use the inverse transpose of the model matrix
    // but for simplicity, we're just applying rotation
    out.normal = normalize((model_matrix * vec4<f32>(in.normal, 0.1)).xyz);
    
    out.tex_coords = in.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_direction = normalize(vec3<f32>(0.5, 1.0, 0.5));
    let ambient = 0.3;
    
    // Sample the texture
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // Calculate diffuse lighting
    let diffuse = max(dot(in.normal, light_direction), 0.0);
    
    // Combine ambient and diffuse lighting
    let lighting = ambient + diffuse * 1.0;
    
    // Check if we got a very dark texture sample - might indicate no texture
    // If so, we'll show the UV coordinates as a debug overlay
    let uv_debug = vec3<f32>(in.tex_coords.x, in.tex_coords.y, 0.5);
    
    // Apply lighting to texture color
    var final_color = tex_color.rgb * lighting;
    
    // If texture is very dark or uniform, blend in the UV visualization
    if all(final_color < vec3<f32>(0.1, 0.1, 0.1)) {
        final_color = mix(final_color, uv_debug, 0.0);
    }
    
    return vec4<f32>(final_color, 1.0);
} 