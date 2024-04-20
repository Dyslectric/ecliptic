use super::gl::*;
use super::window::*;
use super::events::*;

pub struct Context {
    event_loop: EventLoop,
    window: Window,
    gl: GpuState,
}

impl<'a> Context {
    pub async fn create() -> Self {
        let event_loop = EventLoop::create();
        let window = Window::create(&event_loop);
        let gl = GpuState::create(&window).await;
        return Context {
            event_loop,
            window,
            gl,
        };
    }
    pub fn render_pass(&self) -> RenderPass {
        RenderPass::new(self)
    }
    pub fn load_sprite(&self, img: image::Rgba<u8>, width: usize, height: usize) -> Sprite {
        return self.gl.load_sprite(img, width, height);
    }
    //pub fn load_sprite_from_url(url: String) -> Sprite {
    //}
}
