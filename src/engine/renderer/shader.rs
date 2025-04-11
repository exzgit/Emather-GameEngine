use std::borrow::Cow;
use wgpu;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

/// Shader manager for handling shader modules
pub struct ShaderManager {
    shaders: HashMap<String, wgpu::ShaderModule>,
}

#[allow(dead_code)]
impl ShaderManager {
    /// Create a new shader manager
    pub fn new(device: &wgpu::Device) -> Self {
        let mut manager = Self {
            shaders: HashMap::new(),
        };
        
        // Initialize with default shaders
        manager.create_basic_3d_shader(device);
        manager.create_pbr_shader(device);
        
        manager
    }
    
    /// Load a shader from a WGSL string
    pub fn load_from_string(&mut self, device: &wgpu::Device, name: &str, source: &str) -> &wgpu::ShaderModule {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
        });
        
        self.shaders.insert(name.to_string(), shader);
        self.shaders.get(name).unwrap()
    }
    
    /// Load a shader from a file
    pub fn load_from_file(&mut self, device: &wgpu::Device, name: &str, path: &Path) -> Result<&wgpu::ShaderModule> {
        let source = fs::read_to_string(path)?;
        Ok(self.load_from_string(device, name, &source))
    }
    
    /// Get a shader by name
    pub fn get(&self, name: &str) -> Option<&wgpu::ShaderModule> {
        self.shaders.get(name)
    }
    
    /// Create a basic 3D shader
    pub fn create_basic_3d_shader(&mut self, device: &wgpu::Device) -> &wgpu::ShaderModule {
        let source = r#"
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
            @location(0) normal: vec3<f32>,
            @location(1) tex_coords: vec2<f32>,
        };

        @vertex
        fn vs_main(
            in: VertexInput,
        ) -> VertexOutput {
            var out: VertexOutput;
            out.clip_position = uniforms.view_proj * uniforms.model * vec4<f32>(in.position, 1.0);
            
            // Transform the normal to world space
            out.normal = (uniforms.model * vec4<f32>(in.normal, 0.0)).xyz;
            out.tex_coords = in.tex_coords;
            
            return out;
        }

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.5));
            let normal = normalize(in.normal);
            
            // Calculate diffuse lighting
            let diffuse = max(dot(normal, light_dir), 0.0);
            let ambient = 0.3;
            let lighting = ambient + diffuse * 0.7;
            
            let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
            return vec4<f32>(color.rgb * lighting, color.a);
        }
        "#;
        
        self.load_from_string(device, "basic_3d", source)
    }
    
    /// Create a PBR shader
    pub fn create_pbr_shader(&mut self, device: &wgpu::Device) -> &wgpu::ShaderModule {
        let source = r#"
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

        struct SunLight {
            direction: vec3<f32>,
            _padding1: u32,
            color: vec3<f32>,
            intensity: f32,
            use_pbr: u32,
            metallic_factor: f32,
            roughness_factor: f32,
            _padding2: f32,
        };

        @group(0) @binding(3)
        var<uniform> sun_light: SunLight;

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
            @location(3) view_dir: vec3<f32>,
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
            out.normal = normalize((model_matrix * vec4<f32>(in.normal, 0.0)).xyz);
            
            // Calculate view direction (from position to camera)
            out.view_dir = normalize(-out.position);
            out.tex_coords = in.tex_coords;
            
            return out;
        }

        // PBR helper functions
        fn distributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
            let a = roughness * roughness;
            let a2 = a * a;
            let NdotH = max(dot(N, H), 0.0);
            let NdotH2 = NdotH * NdotH;
            
            let num = a2;
            let denom = (NdotH2 * (a2 - 1.0) + 1.0);
            return num / (3.14159265359 * denom * denom);
        }
        
        fn geometrySchlickGGX(NdotV: f32, roughness: f32) -> f32 {
            let r = (roughness + 1.0);
            let k = (r * r) / 8.0;
            
            return NdotV / (NdotV * (1.0 - k) + k);
        }
        
        fn geometrySmith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
            let NdotV = max(dot(N, V), 0.0);
            let NdotL = max(dot(N, L), 0.0);
            let ggx2 = geometrySchlickGGX(NdotV, roughness);
            let ggx1 = geometrySchlickGGX(NdotL, roughness);
            
            return ggx1 * ggx2;
        }
        
        fn fresnelSchlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
            return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
        }

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            // Sample the texture
            let albedo = textureSample(t_diffuse, s_diffuse, in.tex_coords).rgb;
            
            // Normal and view vectors
            let N = normalize(in.normal);
            let V = normalize(in.view_dir);
            
            // Material properties
            let metallic = sun_light.metallic_factor;
            let roughness = sun_light.roughness_factor;
            let ao = 1.0; // Ambient occlusion default
            
            // Reflectance at normal incidence (Fresnel F0)
            let F0 = mix(vec3<f32>(0.04), albedo, metallic);
            
            // Direct lighting calculation
            let light_dir = normalize(-sun_light.direction);
            let light_color = sun_light.color * sun_light.intensity;
            
            // Calculate the light parameters for PBR
            let H = normalize(V + light_dir);
            let NdotL = max(dot(N, light_dir), 0.0);
            
            // Cook-Torrance BRDF
            let NDF = distributionGGX(N, H, roughness);
            let G = geometrySmith(N, V, light_dir, roughness);
            let F = fresnelSchlick(max(dot(H, V), 0.0), F0);
            
            let kS = F; // Specular contribution
            let kD = (vec3<f32>(1.0) - kS) * (1.0 - metallic); // Diffuse contribution
            
            // Specular component
            let numerator = NDF * G * F;
            let denominator = 4.0 * max(dot(N, V), 0.0) * NdotL + 0.0001;
            let specular = numerator / denominator;
            
            // Combine diffuse and specular
            var Lo = vec3<f32>(0.0);
            if (NdotL > 0.0) {
                Lo += (kD * albedo / 3.14159265359 + specular) * light_color * NdotL;
            }
            
            // Ambient lighting
            let ambient = vec3<f32>(0.2) * albedo * ao;
            
            // Final color
            var final_color = vec3<f32>(0.0);
            
            // Debug visualization modes (based on material properties)
            let debug_mode = 0;  // 0 = normal rendering, 1 = metallic visualization, 2 = roughness visualization
            
            if (debug_mode == 1) {
                final_color = vec3<f32>(metallic);
            } else if (debug_mode == 2) {
                final_color = vec3<f32>(roughness);
            } else if (sun_light.use_pbr != 0u) {
                final_color = ambient + Lo;
            } else {
                // Use simple diffuse/ambient lighting
                let diffuse = max(dot(N, light_dir), 0.0);
                let ambient = 0.4;
                final_color = albedo * (ambient + diffuse * sun_light.intensity * sun_light.color);
            }
            
            // Apply tone mapping (HDR -> LDR)
            final_color = final_color / (final_color + vec3<f32>(1.0));
            
            // Apply gamma correction
            final_color = pow(final_color, vec3<f32>(1.0/2.2));
            
            return vec4<f32>(final_color, 1.0);
        }
        "#;
        
        self.load_from_string(device, "pbr", source)
    }
} 