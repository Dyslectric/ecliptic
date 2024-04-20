use image::GenericImageView;

use super::ctx::*;
use super::window::*;

pub struct GpuState {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    diffuse2d_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    diffuse2d_texture_bind_group: wgpu::BindGroup,
    diffuse2d_uniform_bind_group: wgpu::BindGroup,
}

impl GpuState {
    pub async fn create(window: &Window) -> GpuState {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window.subsystem_window) }
            .expect("Something went wrong with your surface thang dude.");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let (width, height) = {
            let width = window.subsystem_window.inner_size().width;
            let height = window.subsystem_window.inner_size().height;

            if  width > 0 &&
                height > 0
            {
                (width, height)
            } else {
                (300, 150)
            }
        };

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            //desired_maximum_frame_latency: 2
        };
        surface.configure(&device, &config);

        GpuState {
            surface,
            device,
            config,
            queue,
        }
    }
    pub fn load_sprite(&self, img: image::Rgba<u8>, width: usize, height: usize) -> Sprite {
    }
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self,image::ImageError> {
        let img = image::load_from_memory(bytes)?;
        Ok(Self::from_image(device, queue, &img, Some(label)))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Self {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

pub struct Sprite {
    texture: Texture,
    pub width: usize,
    pub height: usize
}

pub struct RenderPass<'a> {
    ctx: &'a Context
}

impl<'a> RenderPass<'a> {
    pub fn new(ctx: &'a Context) -> Self {
        return Self {
            ctx
        }
    }
    pub fn draw_sprite(
        sprite: &Sprite,
        x: usize, y: usize,
        w: Option<usize>, h: Option<usize>,
        // rotation in radians
        rot: f32, rot_x: usize, rot_y: usize
    ) {
    }
    pub fn present_frame() {
    }
}
