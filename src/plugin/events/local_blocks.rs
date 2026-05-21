use std::{cell::Cell, rc::Rc};

use classicube_helpers::events::user::BlockChangedEventHandler;
use classicube_sys::{Blocks, SoundType_SOUND_NONE};

use crate::plugin::module::Module;

pub struct LocalBlocksModule {
    _pre_block_changed_handler: BlockChangedEventHandler,
    _post_block_changed_handler: BlockChangedEventHandler,
}

impl LocalBlocksModule {
    pub fn init() -> Self {
        let last_sound: Rc<Cell<Option<u8>>> = Default::default();

        let mut pre_block_changed_handler = BlockChangedEventHandler::new();
        pre_block_changed_handler.on({
            let last_sound = last_sound.clone();

            move |event| {
                if event.block == 0 {
                    return;
                }

                unsafe {
                    last_sound.set(Some(Blocks.StepSounds[event.block as usize]));
                    Blocks.StepSounds[event.block as usize] = SoundType_SOUND_NONE as _;
                }
            }
        });
        pre_block_changed_handler.reorder(0).unwrap();

        let mut post_block_changed_handler = BlockChangedEventHandler::new();
        post_block_changed_handler.on({
            let last_sound = last_sound.clone();

            move |event| {
                if event.block == 0 {
                    return;
                }

                if let Some(sound) = last_sound.take() {
                    unsafe {
                        Blocks.StepSounds[event.block as usize] = sound;
                    }
                }
            }
        });

        Self {
            _pre_block_changed_handler: pre_block_changed_handler,
            _post_block_changed_handler: post_block_changed_handler,
        }
    }
}

impl Module for LocalBlocksModule {}
