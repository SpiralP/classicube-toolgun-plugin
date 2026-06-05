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
        let entity_ptr = unsafe { Entities.List[ENTITIES_SELF_ID as usize] };
        if entity_ptr.is_null() {
            tracing::warn!("local player entity is null, skipping VTABLE swap");
            return Self {
                _new_vtable: Box::pin(EntityVTABLE {
                    Tick: None,
                    Despawn: None,
                    SetLocation: None,
                    GetCol: None,
                    RenderModel: None,
                    ShouldRenderName: None,
                }),
                original_vtable: core::ptr::null(),
            };
        }
        let me = unsafe { &mut *entity_ptr };
        let original_vtable = me.VTABLE;
        let v_table = unsafe { &*original_vtable };

        if v_table.RenderModel.is_some_and(|f| {
            core::ptr::fn_addr_eq(f, hook as unsafe extern "C" fn(*mut Entity, f32, f32))
        }) {
            tracing::warn!("VTABLE RenderModel already set to our hook, skipping");
            return Self {
                _new_vtable: Box::pin(EntityVTABLE {
                    Tick: None,
                    Despawn: None,
                    SetLocation: None,
                    GetCol: None,
                    RenderModel: None,
                    ShouldRenderName: None,
                }),
                original_vtable: core::ptr::null(),
            };
        }

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
