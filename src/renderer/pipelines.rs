use super::*;
use bind_group_layouts::*;
use shaders::*;

pub struct Pipelines {
    pub draw_sprite: Rc<wgpu::RenderPipeline>,
    //pub swap_draw_surface: Rc<wgpu::RenderPipeline>,
    pub window_surface_refresh: Rc<wgpu::RenderPipeline>,
}

impl Pipelines {
    pub fn create(
        device: &wgpu::Device,
        shaders: &Shaders,
        bind_group_layouts: &BindGroupLayouts,
        window_surface_config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let draw_sprite = {
            let shader = &shaders.diffuse2d;

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("draw_sprite pipeline layout"),
                bind_group_layouts: &[
                    &bind_group_layouts.texture,
                    &bind_group_layouts.sprite_uniforms,
                ],
                push_constant_ranges: &[],
            });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("draw_sprite render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[pixel_surface::Vertex2d::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
        };

        let window_surface_refresh = {
            let shader = &shaders.window_surface_refresh;

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("window surface refresh pipeline layout"),
                bind_group_layouts: &[&bind_group_layouts.texture],
                push_constant_ranges: &[],
            });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("window surface refresh render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[super::WindowRefreshVertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: window_surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
        };

        let draw_sprite = Rc::new(draw_sprite);
        let window_surface_refresh = Rc::new(window_surface_refresh);

        Self {
            draw_sprite,
            window_surface_refresh,
        }
    }
}
