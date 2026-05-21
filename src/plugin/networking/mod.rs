pub mod packet;

use std::io::Cursor;

use anyhow::Error;
use classicube_helpers::{async_manager, events::net::PluginMessageReceivedEventHandler};
use classicube_sys::CPE_SendPluginMessage;
use tracing::{debug, error};

use self::packet::{Packet, handle_packet};
use crate::plugin::module::Module;

pub const CHANNEL: u8 = 71;
pub const PLUGIN_MESSAGE_LENGTH: usize = 64;

pub struct NetworkingModule {
    _plugin_message_handler: PluginMessageReceivedEventHandler,
}

impl NetworkingModule {
    pub fn init() -> Self {
        let mut plugin_message_handler = PluginMessageReceivedEventHandler::new();
        plugin_message_handler.on(move |event| {
            if event.channel != CHANNEL {
                return;
            }

            match Packet::decode(&mut Cursor::new(&event.data)) {
                Ok(packet) => {
                    debug!("packet {:?}", packet);
                    handle_packet(packet);
                }

                Err(e) => {
                    error!("decoding packet: {:#?}", e);
                }
            }
        });

        Self {
            _plugin_message_handler: plugin_message_handler,
        }
    }
}

impl Module for NetworkingModule {
    fn on_new_map_loaded(&mut self) {
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
