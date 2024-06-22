use std::{
    io::{Read, Write},
    net::TcpStream,
};

use integer_encoding::{VarIntReader, VarIntWriter};

pub mod disconnect_login;
pub mod handshake;
pub mod ping;
pub mod status_request;
pub mod status_response;

pub trait WritePacketExt {
    fn write_packet<P: PacketEncoder>(&mut self, packet: P) -> anyhow::Result<()>;
}

impl WritePacketExt for TcpStream {
    fn write_packet<P: PacketEncoder>(&mut self, packet: P) -> anyhow::Result<()> {
        let mut buf: Vec<u8> = vec![];
        packet.encode(&mut buf)?;

        let mut packet_buf = vec![];
        packet_buf.write_varint(buf.len() as u32)?;
        packet_buf.write_all(&buf)?;

        self.write_all(&packet_buf)?;

        Ok(())
    }
}

pub fn read_packet<P: PacketDecoder>(stream: &mut TcpStream) -> anyhow::Result<P> {
    let _packet_len: u32 = stream.read_varint()?;
    let packet = P::decode(stream)?;

    Ok(*packet)
}

pub trait PacketEncoder {
    fn encode<W: Write>(&self, stream: &mut W) -> anyhow::Result<()>;
}

pub trait PacketDecoder {
    fn decode<R: Read>(stream: &mut R) -> anyhow::Result<Box<Self>>;
}
