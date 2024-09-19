use std::{
    cell::RefCell,
    collections::VecDeque,
    io::{BufReader, Cursor},
};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use super::networking::packet::Packet;

const MAX_SINKS: usize = 10;

thread_local!(
    static RODIO_STREAM: RefCell<Option<(OutputStream, OutputStreamHandle, VecDeque<Sink>)>> =
        Default::default();
);

pub fn handle_packet(packet: Packet) {
    RODIO_STREAM.with_borrow_mut(|option| {
        if let Some((_, output_stream_handle, sinks)) = option.as_mut() {
            let reader = BufReader::new(Cursor::new(include_bytes!("../../../sounds/toolgun.wav")));
            let source = Decoder::new(reader).unwrap();

            let sink = Sink::try_new(output_stream_handle).unwrap();
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

    RODIO_STREAM.with_borrow_mut(move |option| {
        *option = Some((stream, stream_handle, VecDeque::new()));
    });
}

pub fn free() {
    RODIO_STREAM.with_borrow_mut(|option| {
        drop(option.take());
    });
}
