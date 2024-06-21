use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use core::panic;
use std::{
    io::{Cursor, Read, Write},
    net::TcpStream,
};

use integer_encoding::{VarIntReader, VarIntWriter};

use super::status::StatusResponse;

pub trait WritePacketExt {
    fn write_packet(&mut self, packet: Packet) -> anyhow::Result<()>;
}

impl WritePacketExt for TcpStream {
    fn write_packet(&mut self, packet: Packet) -> anyhow::Result<()> {
        let mut buf: Vec<u8> = vec![];

        match packet {
            Packet::Handshake(handshake) => {
                handshake.encode(&mut buf)?;
            }
            Packet::StatusRequest(status_request) => {
                status_request.encode(&mut buf)?;
            }
            Packet::StatusResponse(status_response) => {
                status_response.encode(&mut buf)?;
            }
        }

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

pub enum Packet {
    Handshake(Handshake),
    StatusRequest(StatusRequest),
    StatusResponse(StatusResponse),
}

#[derive(Debug)]
pub struct Handshake {
    pub version: i32,
    pub host: String,
    pub port: u16,
    pub next_status: i32,
}

impl PacketEncoder for Handshake {
    fn encode<W: Write>(&self, stream: &mut W) -> anyhow::Result<()> {
        stream.write_varint(0x00_u32)?;

        stream.write_varint(self.version as u32)?;

        let host_bytes = self.host.as_bytes();
        stream.write_varint(host_bytes.len() as u32)?;
        stream.write_all(host_bytes)?;

        stream.write_u16::<BigEndian>(self.port)?;

        stream.write_varint(self.next_status as u32)?;

        Ok(())
    }
}

impl PacketDecoder for Handshake {
    fn decode<R: Read>(stream: &mut R) -> anyhow::Result<Box<Self>> {
        let packet_id: u32 = stream.read_varint()?;
        if packet_id != 0x00 {
            panic!("Unsupported protocol: packet_id={:02X}", packet_id);
        }

        let version: u32 = stream.read_varint()?;

        let host_len: u32 = stream.read_varint()?;
        let mut host_buf = vec![0_u8; host_len as usize];
        stream.read_exact(&mut host_buf)?;

        let mut host_buf_cur = Cursor::new(host_buf);
        let mut host = String::new();
        host_buf_cur.read_to_string(&mut host)?;

        let port = stream.read_u16::<BigEndian>()?;

        let next_status: u32 = stream.read_varint()?;

        Ok(Box::new(Handshake {
            version: version as i32,
            host,
            port,
            next_status: next_status as i32,
        }))
    }
}

#[derive(Debug)]
pub struct StatusRequest {}

impl PacketEncoder for StatusRequest {
    fn encode<W: Write>(&self, stream: &mut W) -> anyhow::Result<()> {
        stream.write_varint(0x00_u32)?;

        Ok(())
    }
}

impl PacketDecoder for StatusRequest {
    fn decode<R: Read>(stream: &mut R) -> anyhow::Result<Box<Self>> {
        let packet_id: u32 = stream.read_varint()?;
        if packet_id != 0x00 {
            return Err(anyhow::anyhow!(
                "Unsupported protocol: packet_id={:02X?}",
                packet_id,
            ));
        }

        Ok(Box::new(StatusRequest {}))
    }
}
