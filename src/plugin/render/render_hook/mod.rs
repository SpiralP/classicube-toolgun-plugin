pub mod renderable;

use std::{cell::Cell, pin::Pin};

use classicube_sys::{ENTITIES_SELF_ID, Entities, Entity, EntityVTABLE};

use crate::plugin::module::Module;

thread_local!(
    static ORIGINAL_FN: Cell<Option<unsafe extern "C" fn(*mut Entity, f32, f32)>> =
        Default::default();
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

pub struct RenderHookModule {
    _new_vtable: Pin<Box<EntityVTABLE>>,
    original_vtable: *const EntityVTABLE,
}

impl RenderHookModule {
    pub fn init() -> Self {
        let me = unsafe { &mut *Entities.List[ENTITIES_SELF_ID as usize] };
        let original_vtable = me.VTABLE;
        let v_table = unsafe { &*original_vtable };

        ORIGINAL_FN.set(v_table.RenderModel);

        let new_vtable = Box::pin(EntityVTABLE {
            Tick: v_table.Tick,
            Despawn: v_table.Despawn,
            SetLocation: v_table.SetLocation,
            GetCol: v_table.GetCol,
            RenderModel: Some(hook),
            ShouldRenderName: v_table.ShouldRenderName,
        });
        me.VTABLE = new_vtable.as_ref().get_ref();

        Self {
            _new_vtable: new_vtable,
            original_vtable,
        }
    }
}

impl Module for RenderHookModule {
    fn free(&mut self) {
        // Restore the entity's VTABLE pointer FIRST so it no longer references the
        // boxed hooked table; the box drops with the struct.
        let entity_ptr = unsafe { Entities.List[ENTITIES_SELF_ID as usize] };
        if !entity_ptr.is_null() {
            let me = unsafe { &mut *entity_ptr };
            me.VTABLE = self.original_vtable;
        }

        ORIGINAL_FN.set(None);
        renderable::clear();
    }
}
