use std::{cell::RefCell, collections::VecDeque, io::Cursor};

use classicube_helpers::{entities::ENTITY_SELF_ID, tick::TickEventHandler};
use classicube_sys::Entities;
use rodio::{Decoder, OutputStream, OutputStreamHandle, SpatialSink};

use super::networking::packet::Packet;

const MAX_SINKS: usize = 100;

const TOOLGUN_BYTES: &[u8] = include_bytes!("../../../sounds/toolgun.wav");

thread_local!(
    static RODIO_STREAM: RefCell<
        Option<(OutputStream, OutputStreamHandle, VecDeque<SpatialSink>)>,
    > = Default::default();
);

thread_local!(
    static TICK_HANDLER: RefCell<Option<TickEventHandler>> = Default::default();
);

pub fn handle_packet(packet: Packet) {
    let (left_ear_pos, right_ear_pos) = get_sink_ear_positions();
    let emitter_pos = [
        (packet.block_pos.X as f32) + 0.5,
        (packet.block_pos.Y as f32) + 0.5,
        (packet.block_pos.Z as f32) + 0.5,
    ];

    RODIO_STREAM.with_borrow_mut(|option| {
        if let Some((_, output_stream_handle, sinks)) = option.as_mut() {
            let source = Decoder::new(Cursor::new(TOOLGUN_BYTES)).unwrap();
            let sink = SpatialSink::try_new(
                output_stream_handle,
                emitter_pos,
                left_ear_pos,
                right_ear_pos,
            )
            .unwrap();
            sink.append(source);
            sinks.push_back(sink);
            if sinks.len() == MAX_SINKS {
                sinks.pop_front();
            }
        }
    });
}

pub fn initialize() {
    let (stream, stream_handle) = OutputStream::try_default().unwrap();

    let mut tick_handler = TickEventHandler::new();
    tick_handler.on(move |_event| {
        RODIO_STREAM.with_borrow_mut(move |option| {
            if let Some((_, _, sinks)) = option.as_mut() {
                let (left_ear_pos, right_ear_pos) = get_sink_ear_positions();
                for sink in sinks {
                    sink.set_left_ear_position(right_ear_pos);
                    sink.set_right_ear_position(left_ear_pos);
                }
            }
        });
    });

    RODIO_STREAM.with_borrow_mut(move |option| {
        *option = Some((stream, stream_handle, VecDeque::new()));
    });
    TICK_HANDLER.with_borrow_mut(move |option| {
        *option = Some(tick_handler);
    });
}

pub fn free() {
    TICK_HANDLER.with_borrow_mut(|option| {
        drop(option.take());
    });
    RODIO_STREAM.with_borrow_mut(|option| {
        drop(option.take());
    });
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

    // print(format!(
    //   "{:?} {:?}",
    //   &[left_cos, left_sin],
    //   &[right_cos, right_sin]
    // ));

    let mut left_ear_pos = self_pos;
    left_ear_pos.X += HEAD_SIZE * left_cos; // X
    left_ear_pos.Z += HEAD_SIZE * left_sin; // Z

    let mut right_ear_pos = self_pos;
    right_ear_pos.X += HEAD_SIZE * right_cos; // X
    right_ear_pos.Z += HEAD_SIZE * right_sin; // Z

    (
        [left_ear_pos.X, left_ear_pos.Y, left_ear_pos.Z],
        [right_ear_pos.X, right_ear_pos.Y, right_ear_pos.Z],
    )
}
