use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use classicube_helpers::events::user::BlockChangedEventHandler;
use classicube_sys::{Blocks, SoundType_SOUND_NONE};

thread_local!(
    static PRE_BLOCK_CHANGED_HANDLER: RefCell<Option<BlockChangedEventHandler>> =
        Default::default();
);
thread_local!(
    static POST_BLOCK_CHANGED_HANDLER: RefCell<Option<BlockChangedEventHandler>> =
        Default::default();
);

pub fn initialize() {
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

            // debug!(?event);
            // sound::handle_packet(Packet {
            //     player_id: ENTITY_SELF_ID,
            //     block_pos: event.coords,
            // })
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

    PRE_BLOCK_CHANGED_HANDLER.with_borrow_mut(move |option| {
        *option = Some(pre_block_changed_handler);
    });
    POST_BLOCK_CHANGED_HANDLER.with_borrow_mut(move |option| {
        *option = Some(post_block_changed_handler);
    });
}

pub fn free() {
    POST_BLOCK_CHANGED_HANDLER.with_borrow_mut(|option| {
        drop(option.take());
    });
    PRE_BLOCK_CHANGED_HANDLER.with_borrow_mut(|option| {
        drop(option.take());
    });
}
