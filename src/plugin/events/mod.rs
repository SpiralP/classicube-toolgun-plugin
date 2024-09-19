use std::cell::RefCell;

use classicube_helpers::{entities::ENTITY_SELF_ID, events::user::BlockChangedEventHandler};
use tracing::debug;

use crate::plugin::{
    networking::packet::{Packet, Vec3},
    sound,
};

thread_local!(
    static BLOCK_CHANGED_HANDLER: RefCell<Option<BlockChangedEventHandler>> = Default::default();
);

pub fn initialize() {
    let mut block_changed_handler = BlockChangedEventHandler::new();
    block_changed_handler.on(move |event| {
        if event.block == 0 {
            return;
        }

        debug!(?event);
        sound::handle_packet(Packet {
            player_id: ENTITY_SELF_ID,
            block_pos: Vec3 {
                x: event.coords.X.try_into().unwrap(),
                y: event.coords.Y.try_into().unwrap(),
                z: event.coords.Z.try_into().unwrap(),
            },
        })
    });

    BLOCK_CHANGED_HANDLER.with_borrow_mut(move |option| {
        *option = Some(block_changed_handler);
    });
}

pub fn free() {
    BLOCK_CHANGED_HANDLER.with_borrow_mut(|option| {
        drop(option.take());
    });
}
