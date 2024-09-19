pub mod networking;

use classicube_helpers::async_manager;
use tracing::debug;

pub fn initialize() {
    debug!("plugin initialize");

    async_manager::initialize();

    networking::initialize();
}

pub fn on_new_map() {
    debug!("plugin on_new_map");
}

pub fn on_new_map_loaded() {
    debug!("plugin on_new_map_loaded");

    networking::on_new_map_loaded();
}

pub fn reset() {
    debug!("plugin reset");
}

pub fn free() {
    debug!("plugin free");

    networking::free();

    // this will stop all tasks immediately
    async_manager::shutdown();
}
