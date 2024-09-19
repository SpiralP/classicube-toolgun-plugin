use std::io::Read;

use anyhow::Result;
use byteorder::{NetworkEndian, ReadBytesExt};

#[derive(Debug)]
pub struct Packet {
    pub player_id: u8,
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

impl Packet {
    pub fn decode(data_stream: &mut impl Read) -> Result<Self> {
        let player_id = data_stream.read_u8()?;
        let x = data_stream.read_u16::<NetworkEndian>()?;
        let y = data_stream.read_u16::<NetworkEndian>()?;
        let z = data_stream.read_u16::<NetworkEndian>()?;

        Ok(Self { player_id, x, y, z })
    }
}
