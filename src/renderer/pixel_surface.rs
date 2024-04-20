use super::*;

use bind_group_layouts::*;

use std::{f32::consts::PI, rc::Rc};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Rotation2d {
    pub rotation_matrix: [[f32; 2]; 2],
    pub rotation_center: RenderPlaneCoordinates,
}

impl Rotation2d {
    pub fn by_fraction(fraction: f32, center: Option<RenderPlaneCoordinates>) -> Self {
        let rotation_center = if let Some(center) = center {
            center
        } else {
            RenderPlaneCoordinates { x: 0.0, y: 0.0 }
        };

        let rotation_matrix = cgmath::Matrix2::from_angle(cgmath::Rad(fraction * 2.0 * PI)).into();

        Self {
            rotation_center,
            rotation_matrix,
        }
    }

    pub fn from_matrix(
        rotation_matrix: [[f32; 2]; 2],
        center: Option<RenderPlaneCoordinates>,
    ) -> Self {
        let rotation_center = if let Some(center) = center {
            center
        } else {
            RenderPlaneCoordinates { x: 0.0, y: 0.0 }
        };

        Self {
            rotation_center,
            rotation_matrix,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteUniforms {
    render_target_dimensions: [f32; 2],
    position: [f32; 2],
    dimensions: [f32; 2],
    rotation_center: [f32; 2],
    rotation: [[f32; 2]; 2],
}

pub struct SpriteRotation {
    pub rotation_matrix: [[f32; 2]; 2],
    pub rotation_center: Option<PixelCoordinates>, // top-left by default
}

impl SpriteRotation {
    pub fn by_fraction(fraction: f32, rotation_center: Option<PixelCoordinates>) -> Self {
        let rotation_matrix = cgmath::Matrix2::from_angle(cgmath::Rad(fraction * 2.0 * PI)).into();
        Self {
            rotation_matrix,
            rotation_center,
        }
    }
}

pub struct Texture2dDimensions {
    pub width: f32,
    pub height: f32,
}

pub struct Texture2dRect {
    pub position: TextureCoordinates,
    pub dimensions: Texture2dDimensions,
}

#[derive(Clone, Copy)]
pub struct SpriteTextureArea {
    pub coordinates: PixelCoordinates,
    pub dimensions: PixelDimensions,
}

pub const RECT_INDICES: [u16; 6] = [0, 1, 2, 1, 3, 2];

pub struct Sprite {
    pub texture: Rc<Texture>,
    pub vertices: [TextureCoordinates; 4],
    pub texture_area: Option<SpriteTextureArea>,
    pub dimensions: PixelDimensions,
}

impl Sprite {
    pub fn create(texture: Rc<Texture>, texture_area: Option<SpriteTextureArea>) -> Sprite {
        if let Some(sprite_area) = texture_area {
            use nalgebra::Vector2;

            let sprite_coordinates = Vector2::new(
                sprite_area.coordinates.x as f32,
                sprite_area.coordinates.y as f32,
            );

            let sprite_dimensions = sprite_area.dimensions;

            let texture_dimensions = Vector2::new(
                texture.dimensions.width as f32,
                texture.dimensions.height as f32,
            );

            let sprite_width_vec = Vector2::new(sprite_dimensions.width as f32, 0.0);
            let sprite_height_vec = Vector2::new(0.0, sprite_dimensions.height as f32);

            let top_left = sprite_coordinates.component_div(&texture_dimensions);
            let top_right =
                (sprite_coordinates + sprite_width_vec).component_div(&texture_dimensions);
            let bottom_left =
                (sprite_coordinates + sprite_height_vec).component_div(&texture_dimensions);
            let bottom_right = (sprite_coordinates + sprite_width_vec + sprite_height_vec)
                .component_div(&texture_dimensions);

            let vertices = [
                TextureCoordinates {
                    u: top_left.x,
                    v: top_left.y,
                },
                TextureCoordinates {
                    u: top_right.x,
                    v: top_right.y,
                },
                TextureCoordinates {
                    u: bottom_left.x,
                    v: bottom_left.y,
                },
                TextureCoordinates {
                    u: bottom_right.x,
                    v: bottom_right.y,
                },
            ];

            let dimensions = sprite_area.dimensions;

            Self {
                texture,
                texture_area,
                vertices,
                dimensions,
            }
        } else {
            let dimensions = texture.dimensions;
            Self {
                texture,
                vertices: [
                    TextureCoordinates::top_left(),
                    TextureCoordinates::top_right(),
                    TextureCoordinates::bottom_left(),
                    TextureCoordinates::bottom_right(),
                ],
                texture_area: None,
                dimensions,
            }
        }
    }
    pub fn from_texture(texture: Rc<Texture>) -> Self {
        Sprite::create(texture.clone(), None)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2d {
    pub plane_coords: RenderPlaneCoordinates,
    pub tex_coords: TextureCoordinates,
}

impl Vertex2d {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex2d>() as wgpu::BufferAddress,
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

pub struct PixelSurface {
    pub surface_texture: Texture,
    //pub surface_texture_bind_group: wgpu::BindGroup,
    pub dimensions: PixelDimensions,
}

impl PixelSurface {
    pub fn new_pixel_surface(
        width: u32,
        height: u32,
    ) -> Self {
        let mut surface_rgba = image::RgbaImage::new(width, height);

        for mut _pixel in surface_rgba.pixels_mut() {
            _pixel = &mut image::Rgba::<u8>::from([0, 0, 0, 0]);
        }

        let surface_texture = texture_manager.create_texture(surface_rgba);
        let dimensions = PixelDimensions { width, height };

        Self {
            surface_texture,
            dimensions,
        }
    }
    pub fn clear(&self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("rendersurface2d encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.surface_texture.wgpu_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn draw_subsurface(
        &self,
        subsurface: &PixelSurface,
        position: Option<PixelCoordinates>,
        dimensions: Option<PixelDimensions>,
        rotation: Option<SpriteRotation>,
        //blend_mode: Option<BlendMode>
    ) {
        let device = &self.device;

        let position = position.unwrap_or(PixelCoordinates { x: 0, y: 0 });

        let dimensions = dimensions.unwrap_or(PixelDimensions {
            width: subsurface.dimensions.width,
            height: subsurface.dimensions.height,
        });

        let (rotation_matrix, rotation_center) = {
            let rotation = rotation.unwrap_or(SpriteRotation {
                rotation_matrix: cgmath::Matrix2::from_angle(cgmath::Rad(0.0)).into(),
                rotation_center: None,
            });

            let rotation_center = rotation
                .rotation_center
                .unwrap_or(PixelCoordinates { x: 0, y: 0 });

            (rotation.rotation_matrix, rotation_center)
        };

        let sprite_uniforms = SpriteUniforms {
            render_target_dimensions: [self.dimensions.width as f32, self.dimensions.height as f32],
            position: [position.x as f32, position.y as f32],
            dimensions: [dimensions.width as f32, dimensions.height as f32],
            rotation: rotation_matrix,
            rotation_center: [rotation_center.x as f32, rotation_center.y as f32],
        };

        let sprite_uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite uniform buffer"),
            contents: bytemuck::cast_slice(&[sprite_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let sprite_uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("transform 2d bind group"),
            layout: &self.bind_group_layouts.sprite_uniforms,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sprite_uniforms_buffer.as_entire_binding(),
            }],
        });

        let vertices: [Vertex2d; 4] = [
            Vertex2d {
                // top-left
                tex_coords: TextureCoordinates::top_left(),
                plane_coords: RenderPlaneCoordinates { x: 0.0, y: 0.0 },
            },
            Vertex2d {
                // top-right
                tex_coords: TextureCoordinates::top_right(),
                plane_coords: RenderPlaneCoordinates { x: 1.0, y: 0.0 },
            },
            Vertex2d {
                // bottom-left
                tex_coords: TextureCoordinates::bottom_left(),
                plane_coords: RenderPlaneCoordinates { x: 0.0, y: 1.0 },
            },
            Vertex2d {
                // bottom-right
                tex_coords: TextureCoordinates::bottom_right(),
                plane_coords: RenderPlaneCoordinates { x: 1.0, y: 1.0 },
            },
        ];

        let indices: [u16; 6] = [0, 1, 2, 3, 2, 1];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = indices.len() as u32;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("surface_2d draw sprite encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.surface_texture.wgpu_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &subsurface.surface_texture.wgpu_bind_group, &[]);
            render_pass.set_bind_group(1, &sprite_uniforms_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    // copy a given texture to the surface buffer with parameters
    pub fn draw_sprite(
        &self,
        sprite: &Sprite,
        position: Option<PixelCoordinates>,
        dimensions: Option<PixelDimensions>,
        rotation: Option<SpriteRotation>,
        //blend_mode: Option<BlendMode>
    ) {
        let device = &self.device;

        let position = position.unwrap_or(PixelCoordinates { x: 0, y: 0 });

        let dimensions = dimensions.unwrap_or(PixelDimensions {
            width: sprite.dimensions.width,
            height: sprite.dimensions.height,
        });

        let (rotation_matrix, rotation_center) = {
            let rotation = rotation.unwrap_or(SpriteRotation {
                rotation_matrix: cgmath::Matrix2::from_angle(cgmath::Rad(0.0)).into(),
                rotation_center: None,
            });

            let rotation_center = rotation
                .rotation_center
                .unwrap_or(PixelCoordinates { x: 0, y: 0 });

            (rotation.rotation_matrix, rotation_center)
        };

        let sprite_uniforms = SpriteUniforms {
            render_target_dimensions: [self.dimensions.width as f32, self.dimensions.height as f32],
            position: [position.x as f32, position.y as f32],
            dimensions: [dimensions.width as f32, dimensions.height as f32],
            rotation: rotation_matrix,
            rotation_center: [rotation_center.x as f32, rotation_center.y as f32],
        };

        let sprite_uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite uniform buffer"),
            contents: bytemuck::cast_slice(&[sprite_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let sprite_uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("transform 2d bind group"),
            layout: &self.bind_group_layouts.sprite_uniforms,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sprite_uniforms_buffer.as_entire_binding(),
            }],
        });

        let vertices: [Vertex2d; 4] = [
            Vertex2d {
                // top-left
                tex_coords: sprite.vertices[0],
                plane_coords: RenderPlaneCoordinates { x: 0.0, y: 0.0 },
            },
            Vertex2d {
                // top-right
                tex_coords: sprite.vertices[1],
                plane_coords: RenderPlaneCoordinates { x: 1.0, y: 0.0 },
            },
            Vertex2d {
                // bottom-left
                tex_coords: sprite.vertices[2],
                plane_coords: RenderPlaneCoordinates { x: 0.0, y: 1.0 },
            },
            Vertex2d {
                // bottom-right
                tex_coords: sprite.vertices[3],
                plane_coords: RenderPlaneCoordinates { x: 1.0, y: 1.0 },
            },
        ];

        let indices: [u16; 6] = [0, 1, 2, 3, 2, 1];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = indices.len() as u32;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("surface_2d draw sprite encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.surface_texture.wgpu_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &sprite.texture.wgpu_bind_group, &[]);
            render_pass.set_bind_group(1, &sprite_uniforms_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
    pub fn load_texture(&self, path: &str) -> Rc<Texture> {
        self.texture_manager.load_texture(path)
    }
    pub fn create_sprite(
        &self,
        texture: Rc<Texture>,
        texture_area: Option<SpriteTextureArea>,
    ) -> Sprite {
        Sprite::create(texture, texture_area)
    }
}
