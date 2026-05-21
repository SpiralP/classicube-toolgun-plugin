pub mod vertex_buffer;

use classicube_helpers::events::gfx::{ContextLostEventHandler, ContextRecreatedEventHandler};

use crate::plugin::module::Module;

pub struct ContextModule {
    _context_recreated_handler: ContextRecreatedEventHandler,
    _context_lost_handler: ContextLostEventHandler,
}

impl ContextModule {
    pub fn init() -> Self {
        let mut context_recreated_handler = ContextRecreatedEventHandler::new();
        context_recreated_handler.on(|_| {
            vertex_buffer::context_recreated();
        });

        let mut context_lost_handler = ContextLostEventHandler::new();
        context_lost_handler.on(|_| {
            vertex_buffer::context_lost();
        });

        // start with context created
        vertex_buffer::context_recreated();

        Self {
            _context_recreated_handler: context_recreated_handler,
            _context_lost_handler: context_lost_handler,
        }
    }
}

impl Module for ContextModule {
    fn free(&mut self) {
        vertex_buffer::context_lost();
    }
}
