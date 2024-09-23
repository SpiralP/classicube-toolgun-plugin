use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    os::raw::c_int,
    slice,
    time::{Duration, Instant},
};

use classicube_helpers::{entities::ENTITY_SELF_ID, tick::TickEventHandler};
use classicube_sys::{
    cc_uint8, BlockID, IVec3, Lighting, Net_Handler, Protocol, OPCODE__OPCODE_BULK_BLOCK_UPDATE,
    OPCODE__OPCODE_SET_BLOCK,
};
use tracing::debug;

use crate::plugin::networking::packet::{handle_packet, Packet};

thread_local!(
    static SET_BLOCK_ORIGINAL: Cell<Net_Handler> = Default::default();
);

thread_local!(
    static BULK_BLOCK_UPDATE_ORIGINAL: Cell<Net_Handler> = Default::default();
);

thread_local!(
    static TICK_HANDLER: RefCell<Option<TickEventHandler>> = Default::default();
);

thread_local!(
    static QUEUE: RefCell<VecDeque<(Net_Handler, Vec<u8>)>> = Default::default();
);

thread_local!(
    static NEXT_TIME: Cell<Option<Instant>> = Default::default();
);

thread_local!(
    static LIGHTING_ON_BLOCK_CHANGED_ORIGINAL: Cell<
        Option<
            unsafe extern "C" fn(
                x: c_int,
                y: c_int,
                z: c_int,
                oldBlock: BlockID,
                newBlock: BlockID,
            ),
        >,
    > = Default::default();
);

pub fn initialize() {
    let set_block_original = unsafe { Protocol.Handlers[OPCODE__OPCODE_SET_BLOCK as usize] };
    unsafe {
        Protocol.Handlers[OPCODE__OPCODE_SET_BLOCK as usize] = Some(set_block_hook);
    }

    let bulk_block_update_original =
        unsafe { Protocol.Handlers[OPCODE__OPCODE_BULK_BLOCK_UPDATE as usize] };
    unsafe {
        Protocol.Handlers[OPCODE__OPCODE_BULK_BLOCK_UPDATE as usize] = Some(bulk_block_update_hook);
    }

    let mut tick_handler = TickEventHandler::new();
    tick_handler.on(move |_event| {
        if let Some(next_time) = NEXT_TIME.get() {
            let now = Instant::now();
            if now < next_time {
                return;
            }
        }

        // make every grouping take X seconds
        QUEUE.with_borrow_mut(|queue| {
            if let Some((callback, mut data)) = queue.pop_front() {
                let now = Instant::now();
                NEXT_TIME.set(Some(
                    now + Duration::from_millis((50.0 - (queue.len() as f32)).max(10.0) as u64),
                ));

                debug!(?callback, ?data, "real");
                if let Some(callback) = callback {
                    unsafe {
                        callback(data.as_mut_ptr());
                    }
                }
            } else {
                NEXT_TIME.set(None);
            }
        });
    });

    let lighting_on_block_changed_original = unsafe { Lighting.OnBlockChanged };
    unsafe {
        Lighting.OnBlockChanged = Some(lighting_on_block_changed_hook);
    }

    LIGHTING_ON_BLOCK_CHANGED_ORIGINAL.set(lighting_on_block_changed_original);
    SET_BLOCK_ORIGINAL.set(set_block_original);
    BULK_BLOCK_UPDATE_ORIGINAL.set(bulk_block_update_original);
    TICK_HANDLER.with_borrow_mut(move |option| {
        *option = Some(tick_handler);
    });
}

extern "C" fn set_block_hook(data: *mut cc_uint8) {
    let data = unsafe {
        slice::from_raw_parts(
            data,
            Protocol.Sizes[OPCODE__OPCODE_SET_BLOCK as usize] as usize,
        )
    };
    let data = data.to_vec();
    debug!(?data, "set_block_hook");
    QUEUE.with_borrow_mut(|queue| {
        queue.push_back((SET_BLOCK_ORIGINAL.get(), data));
    });
}

extern "C" fn bulk_block_update_hook(data: *mut cc_uint8) {
    let data = unsafe {
        slice::from_raw_parts(
            data,
            Protocol.Sizes[OPCODE__OPCODE_BULK_BLOCK_UPDATE as usize] as usize,
        )
    };
    let data = data.to_vec();
    debug!(?data, "bulk_block_update_hook");
    QUEUE.with_borrow_mut(|queue| {
        queue.push_back((BULK_BLOCK_UPDATE_ORIGINAL.get(), data));
    });
}

unsafe extern "C" fn lighting_on_block_changed_hook(
    x: c_int,
    y: c_int,
    z: c_int,
    old_block: BlockID,
    new_block: BlockID,
) {
    if let Some(prev) = LIGHTING_ON_BLOCK_CHANGED_ORIGINAL.get() {
        prev(x, y, z, old_block, new_block)
    }

    if old_block == 0 && new_block != 0 {
        debug!(?x, ?y, ?z, ?old_block, ?new_block);
        handle_packet(Packet {
            player_id: ENTITY_SELF_ID,
            block_pos: IVec3 { X: x, Y: y, Z: z },
        })
    }
}

pub fn free() {
    TICK_HANDLER.with_borrow_mut(|option| {
        drop(option.take());
    });
    unsafe {
        Protocol.Handlers[OPCODE__OPCODE_BULK_BLOCK_UPDATE as usize] =
            BULK_BLOCK_UPDATE_ORIGINAL.take();
    }
    unsafe {
        Protocol.Handlers[OPCODE__OPCODE_SET_BLOCK as usize] = SET_BLOCK_ORIGINAL.take();
    }
    unsafe {
        Lighting.OnBlockChanged = LIGHTING_ON_BLOCK_CHANGED_ORIGINAL.take();
    }
}
