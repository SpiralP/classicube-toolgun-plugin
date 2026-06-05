use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    os::raw::c_int,
    slice,
    time::{Duration, Instant},
};

use classicube_helpers::{entities::ENTITY_SELF_ID, tick::TickEventHandler};
use classicube_sys::{
    BlockID, IVec3, Lighting, Net_Handler, OPCODE__OPCODE_BULK_BLOCK_UPDATE,
    OPCODE__OPCODE_SET_BLOCK, Protocol, cc_uint8,
};
use tracing::debug;

use crate::plugin::{
    is_plugin_active,
    module::Module,
    networking::packet::{Packet, handle_packet},
};

thread_local!(
    static SET_BLOCK_ORIGINAL: Cell<Net_Handler> = Default::default();
);

thread_local!(
    static BULK_BLOCK_UPDATE_ORIGINAL: Cell<Net_Handler> = Default::default();
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
                old_block: BlockID,
                new_block: BlockID,
            ),
        >,
    > = Default::default();
);

type LightingHandler = unsafe extern "C" fn(c_int, c_int, c_int, BlockID, BlockID);

fn net_handlers_eq(a: Net_Handler, b: Net_Handler) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => core::ptr::fn_addr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

fn lighting_handlers_eq(a: Option<LightingHandler>, b: Option<LightingHandler>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => core::ptr::fn_addr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

// Idempotent + wipe-aware install: if our hook is already on top, no-op.
// If a foreign hook is stacked above our saved original, leave both alone.
// Otherwise (first install or re-install after a slot wipe), write our hook and save current.
macro_rules! install_hook {
    ($slot:expr, $old:expr, $ours:expr, $eq:expr) => {{
        let current = unsafe { $slot };
        if !$eq(current, Some($ours)) {
            let old = $old.get();
            if old.is_none() || $eq(current, old) {
                unsafe {
                    $slot = Some($ours);
                }
                $old.set(current);
            }
        }
    }};
}

// On-top-only uninstall: only restore if our hook is still on top.
macro_rules! uninstall_hook {
    ($slot:expr, $old:expr, $ours:expr, $eq:expr) => {{
        let current = unsafe { $slot };
        if $eq(current, Some($ours)) {
            unsafe {
                $slot = $old.take();
            }
        }
    }};
}

fn install_all() {
    install_hook!(
        Protocol.Handlers[OPCODE__OPCODE_SET_BLOCK as usize],
        SET_BLOCK_ORIGINAL,
        set_block_hook,
        net_handlers_eq
    );
    install_hook!(
        Protocol.Handlers[OPCODE__OPCODE_BULK_BLOCK_UPDATE as usize],
        BULK_BLOCK_UPDATE_ORIGINAL,
        bulk_block_update_hook,
        net_handlers_eq
    );
    install_hook!(
        Lighting.OnBlockChanged,
        LIGHTING_ON_BLOCK_CHANGED_ORIGINAL,
        lighting_on_block_changed_hook,
        lighting_handlers_eq
    );
}

fn uninstall_all() {
    uninstall_hook!(
        Protocol.Handlers[OPCODE__OPCODE_BULK_BLOCK_UPDATE as usize],
        BULK_BLOCK_UPDATE_ORIGINAL,
        bulk_block_update_hook,
        net_handlers_eq
    );
    uninstall_hook!(
        Protocol.Handlers[OPCODE__OPCODE_SET_BLOCK as usize],
        SET_BLOCK_ORIGINAL,
        set_block_hook,
        net_handlers_eq
    );
    uninstall_hook!(
        Lighting.OnBlockChanged,
        LIGHTING_ON_BLOCK_CHANGED_ORIGINAL,
        lighting_on_block_changed_hook,
        lighting_handlers_eq
    );
}

pub struct OtherBlocksModule {
    _tick_handler: TickEventHandler,
}

impl OtherBlocksModule {
    pub fn init() -> Self {
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

        install_all();

        Self {
            _tick_handler: tick_handler,
        }
    }
}

impl Module for OtherBlocksModule {
    fn reset(&mut self) {
        // Protocol.OnReset wiped SET_BLOCK / BULK_BLOCK_UPDATE back to defaults before this
        // callback; re-install on top of whatever is there now.
        install_all();
    }

    fn on_new_map_loaded(&mut self) {
        // ClassicLighting_SetActive() reassigned Lighting.OnBlockChanged on map load or
        // lighting-mode change; re-install. Already-on-top Protocol slots are no-ops.
        install_all();
    }

    fn free(&mut self) {
        QUEUE.with_borrow_mut(|queue| queue.clear());
        NEXT_TIME.set(None);
        uninstall_all();
    }
}

extern "C" fn set_block_hook(data: *mut cc_uint8) {
    if !is_plugin_active() {
        if let Some(f) = SET_BLOCK_ORIGINAL.get() {
            unsafe { f(data) }
        }
        return;
    }
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
    if !is_plugin_active() {
        if let Some(f) = BULK_BLOCK_UPDATE_ORIGINAL.get() {
            unsafe { f(data) }
        }
        return;
    }
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
        unsafe { prev(x, y, z, old_block, new_block) }
    }

    if !is_plugin_active() {
        return;
    }

    if old_block == 0 && new_block != 0 {
        debug!(?x, ?y, ?z, ?old_block, ?new_block);
        handle_packet(Packet {
            player_id: ENTITY_SELF_ID,
            block_pos: IVec3 { x, y, z },
        })
    }
}
