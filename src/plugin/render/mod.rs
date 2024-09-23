pub mod context;
pub mod drawing;
pub mod laser;
pub mod render_hook;

use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use classicube_helpers::entities::Entities;
use classicube_sys::{IVec3, Vec3};
use tracing::debug;

use self::laser::Laser;
use crate::plugin::render::render_hook::renderable::StartStopRendering;

thread_local!(
    static ENTITIES: RefCell<Option<Entities>> = Default::default();
);

pub fn initialize() {
    context::initialize();
    render_hook::initialize();
    drawing::initialize();

    ENTITIES.with_borrow_mut(|option| {
        let entities = Entities::new();
        *option = Some(entities);
    });
}

pub fn free() {
    ENTITIES.with_borrow_mut(|option| {
        drop(option.take());
    });
    LASERS.with_borrow_mut(|lasers| {
        lasers.clear();
    });
    drawing::free();
    render_hook::free();
    context::free();
}

thread_local!(
    static LASERS: RefCell<Vec<Rc<RefCell<Laser>>>> = Default::default();
);

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
    LASERS.with_borrow_mut(|lasers| {
        let laser = Rc::new(RefCell::new(Laser::new(player_pos, block_pos.to_vec3())));
        laser.start_rendering();
        lasers.push(laser);
    })
}
