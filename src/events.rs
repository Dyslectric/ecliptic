pub struct EventLoop {
    pub subsystem: winit::event_loop::EventLoop<()>
}

impl EventLoop {
    pub fn create() -> Self {
        Self {
            subsystem: winit::event_loop::EventLoop::new().unwrap()
        }
    }
}