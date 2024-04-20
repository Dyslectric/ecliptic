use super::events::*;

use web_sys::HtmlCanvasElement;
use winit::platform::web::WindowBuilderExtWebSys;
use wasm_bindgen::JsCast;

pub struct Window {
    pub subsystem_window: winit::window::Window
}

impl Window {
    pub fn create(event_loop: &EventLoop) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas: HtmlCanvasElement = document.get_element_by_id("wgpuCanvas").unwrap().dyn_into().unwrap();

        let subsystem_window = winit::window::WindowBuilder::new()
            .with_title("A fantastic window!")
            .with_canvas(Some(canvas))
            .build(&event_loop.subsystem).unwrap();

        Self {
            subsystem_window
        }
    }
}