use std::{
    io::{Read, Write},
    net::TcpStream,
};

use disconnect_login::DisconnectLogin;
use handshake::Handshake;
use integer_encoding::{VarIntReader, VarIntWriter};
use ping::Ping;
use status_request::StatusRequest;
use status_response::StatusResponse;

pub mod disconnect_login;
pub mod handshake;
pub mod ping;
pub mod status_request;
pub mod status_response;

pub enum Packet {
    Handshake(Handshake),
    StatusRequest(StatusRequest),
    StatusResponse(StatusResponse),
    Ping(Ping),
    DisconnectLogin(DisconnectLogin),
}

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
