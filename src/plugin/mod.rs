pub mod events;
pub mod networking;
pub mod render;
pub mod sound;

use classicube_helpers::async_manager;
use classicube_sys::Server;
use tracing::debug;

pub fn initialize() {
    debug!("plugin initialize");

    async_manager::initialize();
    render::initialize();
    sound::initialize();
    events::initialize();

    if unsafe { Server.IsSinglePlayer } == 0 {
        networking::initialize();
    }
}

pub fn on_new_map() {
    debug!("plugin on_new_map");
}

pub fn on_new_map_loaded() {
    debug!("plugin on_new_map_loaded");

    if unsafe { Server.IsSinglePlayer } == 0 {
        networking::on_new_map_loaded();
    }
}

pub fn reset() {
    debug!("plugin reset");
}

pub fn free() {
    debug!("plugin free");

    if unsafe { Server.IsSinglePlayer } == 0 {
        networking::free();
    }

    events::free();
    sound::free();
    render::free();

    // this will stop all tasks immediately
    async_manager::shutdown();
}
