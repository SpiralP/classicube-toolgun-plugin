pub mod context;
pub mod drawing;
pub mod laser;
pub mod render_hook;

use std::{cell::RefCell, rc::Rc};

use classicube_sys::{IVec3, Vec3};
use tracing::debug;

use self::laser::Laser;
use crate::plugin::render::render_hook::renderable::StartStopRendering;

pub fn initialize() {
    context::initialize();
    render_hook::initialize();
    drawing::initialize();
}

pub fn free() {
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
pub fn create_laser(todo: (), block_pos: IVec3) {
    debug!("");

    LASERS.with_borrow_mut(|lasers| {
        let laser = Rc::new(RefCell::new(Laser::new(
            block_pos.to_vec3(),
            Vec3 {
                X: 0.0,
                Y: 0.0,
                Z: 0.0,
            },
        )));
        laser.start_rendering();
        lasers.push(laser);
    })
}
