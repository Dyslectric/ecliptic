use std::{any::Any, rc::Rc};

use pixel_surface::{Sprite, SpriteTextureArea};
use wgpu::util::DeviceExt;

use crate::window::*;

pub mod pixel_surface;
use pixel_surface::PixelSurface;

mod pipelines;
use pipelines::*;

mod shaders;

mod bind_group_layouts;

use self::{bind_group_layouts::BindGroupLayouts, pixel_surface::SpriteRotation, shaders::Shaders};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PixelCoordinates {
    pub x: i32,
    pub y: i32,
}

impl PixelCoordinates {
    pub fn top_left() -> Self {
        PixelCoordinates { x: 0, y: 0 }
    }
    pub fn to_render_plane_coordinates(
        &self,
        plane_pixel_dimensions: &PixelDimensions,
    ) -> RenderPlaneCoordinates {
        let x = -1.0 + (self.x as f32 / plane_pixel_dimensions.width as f32) * 2.0;
        let y = 1.0 - (self.y as f32 / plane_pixel_dimensions.height as f32) * 2.0;

        RenderPlaneCoordinates { x, y }
    }
}

#[derive(Clone, Copy)]
pub struct PixelDimensions {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderPlaneCoordinates {
    pub x: f32,
    pub y: f32,
}

impl RenderPlaneCoordinates {
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    pub fn top_left() -> Self {
        Self { x: -1.0, y: 1.0 }
    }
    pub fn top_right() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
    pub fn bottom_left() -> Self {
        Self { x: -1.0, y: -1.0 }
    }
    pub fn bottom_right() -> Self {
        Self { x: 1.0, y: -1.0 }
    }
    pub fn to_vertex(&self) -> Vertex {
        Vertex {
            x: self.x,
            y: self.y,
            z: 0.0,
        }
    }
}

pub struct RenderPlaneDimensions {
    pub width: f32,
    pub height: f32,
}

impl RenderPlaneDimensions {
    pub fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

struct TextureManager {
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    bind_group_layout: Rc<wgpu::BindGroupLayout>,
}

impl TextureManager {
    pub fn create(
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        bind_group_layout: Rc<wgpu::BindGroupLayout>,
    ) -> Self {
        Self {
            device,
            queue,
            bind_group_layout,
        }
    }
    pub fn load_texture(&self, path: &str) -> Rc<Texture> {
        let texture_image = image::open(path).unwrap();
        let texture_rgba = texture_image.to_rgba8();
        Rc::new(self.create_texture(texture_rgba))
    }
    pub fn create_texture(
        &self,
        texture_rgba: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    ) -> Texture {
        Texture::create(
            &self.device,
            &self.queue,
            texture_rgba,
            &self.bind_group_layout,
        )
    }
}

pub struct Texture {
    pub wgpu_texture: wgpu::Texture,
    pub wgpu_texture_view: wgpu::TextureView,
    pub wgpu_sampler: wgpu::Sampler,
    pub wgpu_bind_group: wgpu::BindGroup,
    pub dimensions: PixelDimensions,
}

impl Texture {
    pub fn create(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_rgba: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Texture {
        let dimensions = {
            let (width, height) = texture_rgba.dimensions();
            PixelDimensions { width, height }
        };

        let texture_size = wgpu::Extent3d {
            width: dimensions.width,
            height: dimensions.height,
            depth_or_array_layers: 1,
        };
        let wgpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &wgpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.width),
                rows_per_image: Some(dimensions.height),
            },
            texture_size,
        );

        let wgpu_texture_view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor {
            ..Default::default()
        });
        let wgpu_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let wgpu_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&wgpu_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&wgpu_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        Self {
            wgpu_texture,
            wgpu_texture_view,
            wgpu_sampler,
            wgpu_bind_group,
            dimensions,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextureCoordinates {
    u: f32,
    v: f32,
}

impl TextureCoordinates {
    pub fn top_left() -> Self {
        Self { u: 0.0, v: 0.0 }
    }
    pub fn top_right() -> Self {
        Self { u: 1.0, v: 0.0 }
    }
    pub fn bottom_left() -> Self {
        Self { u: 0.0, v: 1.0 }
    }
    pub fn bottom_right() -> Self {
        Self { u: 1.0, v: 1.0 }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct WindowRefreshVertex {
    output_coords: RenderPlaneCoordinates,
    texture_coords: TextureCoordinates,
}

impl WindowRefreshVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<WindowRefreshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ],
        }
    }
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    bind_group_layouts: BindGroupLayouts,
    texture_manager: TextureManager,
    pipelines: Pipelines,
    output_surface: wgpu::Surface,
    swap_surface: PixelSurface,
}

impl Renderer {
    pub async fn new(window: &Window) -> Renderer {
        let window: winit::window::Window = window.winit_window();
        let (width, height) = {
            let winit::dpi::PhysicalSize { width, height } = window.inner_size();
            (width, height)
        };
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: wgpu::InstanceFlags::empty(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let output_surface = unsafe { instance.create_surface(&window) }
            .expect("Couldn't create surface from window.");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("Could not obtain an appropriate adapter.");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .expect("Could not obtain an appropriate device and queue.");

        let surface_caps = output_surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        output_surface.configure(&device, &config);

        let shaders = Shaders::create(&device);
        let bind_group_layouts = Rc::new(bind_group_layouts::BindGroupLayouts::create(&device));

        let pipelines = Pipelines::create(&device, &shaders, &bind_group_layouts, &config);

        let swap_surface = PixelSurface::new(
            &device,
            &queue,
            &bind_group_layouts,
            &pipelines.draw_sprite,
            width,
            height,
        );

        Self {
            device,
            queue,
            output_surface,
            swap_surface,
            bind_group_layouts,
            pipelines,
            //shaders,
            //config,
        }
    }
    pub fn clear(&self) {
        self.swap_surface.clear();
    }
    pub fn present(&self) {
        let output = self.output_surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            ..Default::default()
        });

        let device = &self.device;

        let vertices = [
            WindowRefreshVertex {
                output_coords: RenderPlaneCoordinates::top_left(),
                texture_coords: TextureCoordinates::top_left(),
            },
            WindowRefreshVertex {
                output_coords: RenderPlaneCoordinates::top_right(),
                texture_coords: TextureCoordinates::top_right(),
            },
            WindowRefreshVertex {
                output_coords: RenderPlaneCoordinates::bottom_left(),
                texture_coords: TextureCoordinates::bottom_left(),
            },
            WindowRefreshVertex {
                output_coords: RenderPlaneCoordinates::bottom_right(),
                texture_coords: TextureCoordinates::bottom_right(),
            },
        ];

        let indices: [u16; 6] = [0, 1, 2, 1, 3, 2];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        // NEW!
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = indices.len() as u32;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("deez nuts"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipelines.window_surface_refresh);
            render_pass.set_bind_group(0, &self.swap_surface.surface_texture.wgpu_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
    pub fn create_subsurface(&self, width: u32, height: u32) -> PixelSurface {
        PixelSurface::new(
            Rc::clone(&self.device),
            Rc::clone(&self.queue),
            Rc::clone(&self.bind_group_layouts),
            Rc::clone(&self.pipelines.draw_sprite),
            self.texture_manager.clone(),
            width,
            height,
        )
    }
    pub fn draw_sprite(
        &self,
        sprite: &Sprite,
        position: Option<PixelCoordinates>,
        dimensions: Option<PixelDimensions>,
        rotation: Option<SpriteRotation>,
        //blend_mode: Option<BlendMode>
    ) {
        self.swap_surface
            .draw_sprite(sprite, position, dimensions, rotation)
    }
    pub fn draw_subsurface(
        &self,
        subsurface: &PixelSurface,
        position: Option<PixelCoordinates>,
        dimensions: Option<PixelDimensions>,
        rotation: Option<SpriteRotation>,
        //blend_mode: Option<BlendMode>
    ) {
        self.swap_surface.draw_subsurface(subsurface, position, dimensions, rotation)
    }
    pub fn load_texture(&self, path: &str) -> Rc<Texture> {
        self.texture_manager.load_texture(path)
    }
    pub fn create_sprite(
        &self,
        texture: Rc<Texture>,
        texture_area: Option<SpriteTextureArea>,
    ) -> Sprite {
        self.swap_surface.create_sprite(texture, texture_area)
    }
}
