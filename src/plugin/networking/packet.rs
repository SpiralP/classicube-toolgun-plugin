use std::io::Read;

use anyhow::Result;
use byteorder::{NetworkEndian, ReadBytesExt};
use classicube_sys::IVec3;

#[derive(Debug)]
pub struct Packet {
    pub player_id: u8,
    pub block_pos: IVec3,
}

impl Packet {
    pub fn decode(data_stream: &mut impl Read) -> Result<Self> {
        let player_id = data_stream.read_u8()?;
        let x = data_stream.read_u16::<NetworkEndian>()?;
        let y = data_stream.read_u16::<NetworkEndian>()?;
        let z = data_stream.read_u16::<NetworkEndian>()?;

        Ok(Self {
            player_id,
            block_pos: IVec3 {
                X: x.into(),
                Y: y.into(),
                Z: z.into(),
            },
        })
    }
}
