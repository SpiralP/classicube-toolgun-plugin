pub mod local_blocks;
pub mod other_blocks;

use self::{local_blocks::LocalBlocksModule, other_blocks::OtherBlocksModule};
use crate::plugin::module::Module;

pub struct EventsModule {
    local_blocks_module: LocalBlocksModule,
    other_blocks_module: OtherBlocksModule,
}

impl EventsModule {
    pub fn init() -> Self {
        Self {
            local_blocks_module: LocalBlocksModule::init(),
            other_blocks_module: OtherBlocksModule::init(),
        }
    }
}

impl Module for EventsModule {
    fn children(&mut self) -> Vec<&mut dyn Module> {
        vec![&mut self.local_blocks_module, &mut self.other_blocks_module]
    }
}
