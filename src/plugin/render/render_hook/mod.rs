pub mod renderable;

use std::{cell::Cell, pin::Pin};

use classicube_sys::{ENTITIES_SELF_ID, Entities, Entity, EntityVTABLE};

thread_local!(
    static ORIGINAL_FN: Cell<Option<unsafe extern "C" fn(*mut Entity, f32, f32)>> =
        Default::default();
);

thread_local!(
    static ORIGINAL_VTABLE: Cell<Option<*const EntityVTABLE>> = Default::default();
);

thread_local!(
    static VTABLE: Cell<Option<Pin<Box<EntityVTABLE>>>> = Default::default();
);

/// This is called when `LocalPlayer_RenderModel` is called.
extern "C" fn hook(local_player_entity: *mut Entity, delta: f32, t: f32) {
    ORIGINAL_FN.with(|cell| {
        if let Some(f) = cell.get() {
            unsafe {
                f(local_player_entity, delta, t);
            }
        }
    });

    renderable::render_all();
}

pub fn initialize() {
    let me = unsafe { &mut *Entities.List[ENTITIES_SELF_ID as usize] };
    let original_vtable_ptr = me.VTABLE;
    let v_table = unsafe { &*original_vtable_ptr };

    ORIGINAL_VTABLE.set(Some(original_vtable_ptr));
    ORIGINAL_FN.with(|cell| {
        cell.set(v_table.RenderModel);
    });

    let new_v_table = Box::pin(EntityVTABLE {
        Tick: v_table.Tick,
        Despawn: v_table.Despawn,
        SetLocation: v_table.SetLocation,
        GetCol: v_table.GetCol,
        RenderModel: Some(hook),
        ShouldRenderName: v_table.ShouldRenderName,
    });
    me.VTABLE = new_v_table.as_ref().get_ref();

    VTABLE.with(|cell| {
        cell.set(Some(new_v_table));
    });
}

pub fn free() {
    // Restore the entity's VTABLE pointer FIRST so it no longer references the
    // boxed hooked table, then drop the box.
    let entity_ptr = unsafe { Entities.List[ENTITIES_SELF_ID as usize] };
    if !entity_ptr.is_null() {
        if let Some(original) = ORIGINAL_VTABLE.take() {
            let me = unsafe { &mut *entity_ptr };
            me.VTABLE = original;
        }
    } else {
        ORIGINAL_VTABLE.set(None);
    }

    VTABLE.with(|cell| {
        cell.take();
    });
    ORIGINAL_FN.with(|cell| cell.set(None));
    renderable::clear();
}
