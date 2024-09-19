pub mod packet;

use std::{cell::RefCell, io::Cursor};

use anyhow::Error;
use classicube_helpers::{async_manager, events::net::PluginMessageReceivedEventHandler};
use classicube_sys::{CPE_SendPluginMessage, Server};
use tracing::{debug, error};

use self::packet::Packet;

thread_local!(
    static PLUGIN_MESSAGE_HANDLER: RefCell<Option<PluginMessageReceivedEventHandler>> =
        Default::default();
);

pub const CHANNEL: u8 = 71;
pub const PLUGIN_MESSAGE_LENGTH: usize = 64;

pub fn initialize() {
    if unsafe { Server.IsSinglePlayer } != 0 {
        // TODO singleplayer
        return;
    }

    let mut plugin_message_handler = PluginMessageReceivedEventHandler::new();

    plugin_message_handler.on(move |event| {
        if event.channel != CHANNEL {
            return;
        }

        match Packet::decode(&mut Cursor::new(&event.data)) {
            Ok(packet) => {
                debug!("packet {:?}", packet);
                // if let Err(e) = store.process_packet(packet) {
                //     error!("processing packet: {:#?}", e);
                // }
            }

            Err(e) => {
                error!("decoding packet: {:#?}", e);
            }
        }
    });

    PLUGIN_MESSAGE_HANDLER.with_borrow_mut(move |option| {
        *option = Some(plugin_message_handler);
    });
}

pub fn on_new_map_loaded() {
    if unsafe { Server.IsSinglePlayer } == 0 {
        async_manager::spawn_local_on_main_thread(async move {
            if let Err(e) = async move {
                // send empty packet to tell server we have this plugin
                let mut data = Vec::with_capacity(PLUGIN_MESSAGE_LENGTH);
                unsafe {
                    CPE_SendPluginMessage(CHANNEL, data.as_mut_ptr());
                }
                Ok::<_, Error>(())
            }
            .await
            {
                error!("{:?}", e);
            }
        });
    }
}

pub fn free() {
    PLUGIN_MESSAGE_HANDLER.with_borrow_mut(|option| {
        drop(option.take());
    });
}
