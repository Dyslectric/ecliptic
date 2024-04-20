use super::*;
pub struct BindGroupLayouts {
    pub texture: Rc<wgpu::BindGroupLayout>,
    pub sprite_uniforms: Rc<wgpu::BindGroupLayout>,
}

impl BindGroupLayouts {
    pub fn texture(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
            label: Some("Texture bind group layout"),
        })
    }
    pub fn sprite_uniforms(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("SpriteUniforms bind group layout"),
        })
    }
    pub fn create(device: &wgpu::Device) -> Self {
        let texture_bind_group_layout = BindGroupLayouts::texture(device);
        let sprite_uniforms_bind_group_layout = BindGroupLayouts::sprite_uniforms(device);
        Self {
            texture: Rc::new(texture_bind_group_layout),
            sprite_uniforms: Rc::new(sprite_uniforms_bind_group_layout),
        }
    }
}
