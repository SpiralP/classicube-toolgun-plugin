pub mod async_manager;
pub mod events;
pub mod logger;
pub mod module;
pub mod networking;
pub mod render;
pub mod sound;

use std::cell::{Cell, RefCell};

use classicube_sys::Server;

use crate::plugin::{
    async_manager::AsyncManagerModule, events::EventsModule, logger::LoggerModule, module::Module,
    networking::NetworkingModule, render::RenderModule, sound::SoundModule,
};

thread_local!(
    static MAIN_MODULE: RefCell<Option<MainModule>> = const { RefCell::new(None) };
);

thread_local!(
    static PLUGIN_ACTIVE: Cell<bool> = const { Cell::new(false) };
);

pub fn is_plugin_active() -> bool {
    PLUGIN_ACTIVE.get()
}

struct MainModule {
    logger: LoggerModule,
    async_manager: AsyncManagerModule,
    render: RenderModule,
    sound: SoundModule,
    events: EventsModule,
    networking: Option<NetworkingModule>,
}

impl MainModule {
    fn init() -> Self {
        let logger = LoggerModule::init();
        let async_manager = AsyncManagerModule::init();
        let render = RenderModule::init();
        let sound = SoundModule::init();
        let events = EventsModule::init();
        let networking = (unsafe { Server.IsSinglePlayer } == 0).then(NetworkingModule::init);

        Self {
            logger,
            async_manager,
            render,
            sound,
            events,
            networking,
        }
    }
}

impl Module for MainModule {
    fn children(&mut self) -> Vec<&mut dyn Module> {
        let mut children: Vec<&mut dyn Module> = vec![
            &mut self.logger,
            &mut self.async_manager,
            &mut self.render,
            &mut self.sound,
            &mut self.events,
        ];
        if let Some(networking) = self.networking.as_mut() {
            children.push(networking);
        }
        children
    }
}

pub fn initialize() {
    MAIN_MODULE.with_borrow_mut(|main_module| {
        if main_module.is_none() {
            *main_module = Some(MainModule::init());
        }
    });
    PLUGIN_ACTIVE.set(true);
}

pub fn free() {
    PLUGIN_ACTIVE.set(false);
    MAIN_MODULE.with_borrow_mut(|main_module| {
        if let Some(mut main_module) = main_module.take() {
            main_module.handle_free();
        }
    });
}

pub fn reset() {
    MAIN_MODULE.with_borrow_mut(|main_module| {
        if let Some(main_module) = main_module.as_mut() {
            main_module.handle_reset();
        }
    });
}

pub fn on_new_map() {
    MAIN_MODULE.with_borrow_mut(|main_module| {
        if let Some(main_module) = main_module.as_mut() {
            main_module.handle_on_new_map();
        }
    });
}

pub fn on_new_map_loaded() {
    MAIN_MODULE.with_borrow_mut(|main_module| {
        if let Some(main_module) = main_module.as_mut() {
            main_module.handle_on_new_map_loaded();
        }
    });
}
