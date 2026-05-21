pub mod context;
pub mod laser;
pub mod render_hook;

use std::{cell::RefCell, rc::Rc};

use classicube_helpers::entities::Entities;
use classicube_sys::{IVec3, Vec3};
use tracing::debug;

use self::{
    context::ContextModule,
    laser::Laser,
    render_hook::{RenderHookModule, renderable::StartStopRendering},
};
use crate::plugin::module::Module;

thread_local!(
    static ENTITIES: RefCell<Option<Entities>> = Default::default();
);

thread_local!(
    static LASERS: RefCell<Vec<Rc<RefCell<Laser>>>> = Default::default();
);

pub struct RenderModule {
    context_module: ContextModule,
    render_hook_module: RenderHookModule,
}

impl RenderModule {
    pub fn init() -> Self {
        let context_module = ContextModule::init();
        let render_hook_module = RenderHookModule::init();

        ENTITIES.with_borrow_mut(|option| {
            let entities = Entities::new();
            *option = Some(entities);
        });

        Self {
            context_module,
            render_hook_module,
        }
    }
}

impl Module for RenderModule {
    fn children(&mut self) -> Vec<&mut dyn Module> {
        vec![&mut self.context_module, &mut self.render_hook_module]
    }

    fn free(&mut self) {
        ENTITIES.with_borrow_mut(|option| {
            drop(option.take());
        });
        LASERS.with_borrow_mut(|lasers| {
            lasers.clear();
        });
    }
}

#[tracing::instrument]
pub fn create_laser(entity_id: u8, block_pos: IVec3) {
    debug!("");

    let player_pos = ENTITIES.with_borrow(|option| {
        option
            .as_ref()
            .unwrap()
            .get(entity_id)
            .unwrap()
            .upgrade()
            .unwrap()
            .get_eye_position()
    });
    let block_pos = Vec3 {
        x: block_pos.x as f32 + 0.5,
        y: block_pos.y as f32 + 0.5,
        z: block_pos.z as f32 + 0.5,
    };
    LASERS.with_borrow_mut(|lasers| {
        let laser = Rc::new(RefCell::new(Laser::new(player_pos, block_pos)));
        laser.start_rendering();
        lasers.push(laser);
    })
}
