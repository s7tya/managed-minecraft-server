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

        buf.write_varint(packet.packet_id())?;
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
    let packet_id: u32 = stream.read_varint()?;

    let packet = *P::decode(stream)?;
    if packet.packet_id() != packet_id {
        return Err(anyhow::anyhow!("Invalid packet_id"));
    }

    Ok(packet)
}

pub trait PacketEncoder {
    fn encode<W: Write>(&self, stream: &mut W) -> anyhow::Result<()>;
    fn packet_id(&self) -> u32;
}

pub trait PacketDecoder {
    fn decode<R: Read>(stream: &mut R) -> anyhow::Result<Box<Self>>;
    fn packet_id(&self) -> u32;
}
