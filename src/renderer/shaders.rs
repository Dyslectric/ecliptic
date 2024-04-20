pub struct Shaders {
    pub diffuse2d: wgpu::ShaderModule,
    pub window_surface_refresh: wgpu::ShaderModule,
}

impl Shaders {
    pub fn create(device: &wgpu::Device) -> Self {
        let draw_sprite = device.create_shader_module(wgpu::include_wgsl!("diffuse2d.wgsl"));

        let window_surface_refresh =
            device.create_shader_module(wgpu::include_wgsl!("window_refresh.wgsl"));

        Self {
            diffuse2d: draw_sprite,
            window_surface_refresh,
        }
    }
}
