use std::{cell::RefCell, collections::VecDeque, io::Cursor};

use classicube_helpers::{entities::ENTITY_SELF_ID, tick::TickEventHandler};
use classicube_sys::{Entities, IVec3};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, SpatialPlayer};

use crate::plugin::module::Module;

const MAX_SINKS: usize = 100;

const TOOLGUN_BYTES: &[u8] = include_bytes!("../../../sounds/toolgun.wav");

thread_local!(
    static RODIO_STREAM: RefCell<Option<(MixerDeviceSink, VecDeque<SpatialPlayer>)>> =
        Default::default();
);

pub fn play_sound(block_pos: IVec3) {
    let (left_ear_pos, right_ear_pos) = get_sink_ear_positions();
    let emitter_pos = [
        (block_pos.x as f32) + 0.5,
        (block_pos.y as f32) + 0.5,
        (block_pos.z as f32) + 0.5,
    ];

    RODIO_STREAM.with_borrow_mut(|option| {
        if let Some((device_sink, sinks)) = option.as_mut() {
            let source = Decoder::new(Cursor::new(TOOLGUN_BYTES)).unwrap();
            let sink = SpatialPlayer::connect_new(
                device_sink.mixer(),
                emitter_pos,
                left_ear_pos,
                right_ear_pos,
            );
            sink.append(source);
            sinks.push_back(sink);
            if sinks.len() == MAX_SINKS {
                sinks.pop_front();
            }
        }
    });
}

pub struct SoundModule {
    _tick_handler: TickEventHandler,
}

impl SoundModule {
    pub fn init() -> Self {
        let device_sink = DeviceSinkBuilder::open_default_sink().unwrap();

        let mut tick_handler = TickEventHandler::new();
        tick_handler.on(move |_event| {
            RODIO_STREAM.with_borrow_mut(move |option| {
                if let Some((_, sinks)) = option.as_mut() {
                    let (left_ear_pos, right_ear_pos) = get_sink_ear_positions();
                    for sink in sinks {
                        sink.set_left_ear_position(right_ear_pos);
                        sink.set_right_ear_position(left_ear_pos);
                    }
                }
            });
        });

        RODIO_STREAM.with_borrow_mut(move |option| {
            *option = Some((device_sink, VecDeque::new()));
        });

        Self {
            _tick_handler: tick_handler,
        }
    }
}

impl Module for SoundModule {
    fn free(&mut self) {
        RODIO_STREAM.with_borrow_mut(|option| {
            drop(option.take());
        });
    }
}

fn get_sink_ear_positions() -> ([f32; 3], [f32; 3]) {
    use std::f32::consts::PI;

    let self_entity = unsafe { &*Entities.List[ENTITY_SELF_ID as usize] };

    let self_pos = self_entity.Position;
    let self_rot_yaw = self_entity.RotY;

    let (left_sin, left_cos) = {
        let ratio = self_rot_yaw / 360.0;
        let rot = ratio * (2.0 * PI) - PI;
        rot.sin_cos()
    };

    let (right_sin, right_cos) = {
        let ratio = self_rot_yaw / 360.0;
        let rot = ratio * (2.0 * PI);
        rot.sin_cos()
    };

    const HEAD_SIZE: f32 = 0.2;

    // Z is negative going forward

    let mut left_ear_pos = self_pos;
    left_ear_pos.x += HEAD_SIZE * left_cos; // x
    left_ear_pos.z += HEAD_SIZE * left_sin; // z

    let mut right_ear_pos = self_pos;
    right_ear_pos.x += HEAD_SIZE * right_cos; // x
    right_ear_pos.z += HEAD_SIZE * right_sin; // z

    (
        [left_ear_pos.x, left_ear_pos.y, left_ear_pos.z],
        [right_ear_pos.x, right_ear_pos.y, right_ear_pos.z],
    )
}
