use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use core::panic;
use serde::Serialize;
use std::{
    io::{Cursor, Read, Write},
    net::TcpStream,
    time::{SystemTime, UNIX_EPOCH},
};

use integer_encoding::{VarIntReader, VarIntWriter};

use super::{raw_json_text::RawJsonText, status::StatusResponse};

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
            Packet::Ping(ping) => {
                ping.encode(&mut buf)?;
            }
            Packet::DisconnectLogin(disconnect_login) => {
                disconnect_login.encode(&mut buf)?;
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
    Ping(Ping),
    DisconnectLogin(DisconnectLogin),
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
#[derive(Debug)]
pub struct Ping {
    payload: u64,
}

impl PacketEncoder for Ping {
    fn encode<W: Write>(&self, stream: &mut W) -> anyhow::Result<()> {
        stream.write_u64::<LittleEndian>(self.payload)?;

        Ok(())
    }
}

impl PacketDecoder for Ping {
    fn decode<R: Read>(_stream: &mut R) -> anyhow::Result<Box<Self>> {
        let start = SystemTime::now();
        let now = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let payload = now.as_secs();

        Ok(Box::new(Ping { payload }))
    }
}

#[derive(Debug, Serialize)]
pub struct DisconnectLogin {
    pub reason: RawJsonText,
}

impl PacketEncoder for DisconnectLogin {
    fn encode<W: Write>(&self, stream: &mut W) -> anyhow::Result<()> {
        let s = serde_json::to_string(&self.reason)?.into_bytes();
        stream.write_varint(0x00_u32)?;
        stream.write_varint(s.len() as u32)?;
        stream.write_all(&s)?;

        Ok(())
    }
}
