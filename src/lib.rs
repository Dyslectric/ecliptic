use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::web::{WindowBuilderExtWebSys, WindowExtWebSys},
    window::{WindowAttributes, WindowBuilder},
};

mod window;
mod ctx;
mod gl;
mod events;

use ctx::*;

#[wasm_bindgen(start)]
pub async fn run() {
    console_log::init().unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let context = Context::create().await;

    let _ = wasm_bindgen_futures::spawn_local(run_event_loop(event_loop, context));
}

async fn run_event_loop(event_loop: EventLoop<()>, mut context: Context) {
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run(move |event, elwt | {

        match event {
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                context.render().unwrap();
            },
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                // Update GPU rendering to new size
                log::info!("Window resized to {:?}", size);
                context.resize(size);
            },
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                elwt.exit();
                log::info!("Exiting.");
                //*control_flow = ControlFlow::Exit;
            },
            Event::AboutToWait => {
                context.window().request_redraw();
                //log::info!("Window size: {} x {}", window.request_inner_size().unwrap().width, window.inner_size().height);
                // Application update code.
    
                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                //window.request_redraw()
                
            },
            _ => ()
        }
    }).unwrap();
}
