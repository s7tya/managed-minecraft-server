use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use core::panic;
use std::{
    io::{Cursor, Read, Write},
    net::TcpStream,
};

use integer_encoding::{VarIntReader, VarIntWriter};

pub enum Packet {
    Handshake {
        version: i32,
        host: String,
        port: u16,
        next_status: i32,
    },
    StatusRequest,
}

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(host: &str, port: u16) -> anyhow::Result<Self> {
        let stream = TcpStream::connect((host.to_string(), port))?;
        stream.set_nodelay(true)?;

        Ok(Connection { stream })
    }

    pub fn send_packet(&mut self, packet: Packet) -> anyhow::Result<()> {
        let mut buf: Vec<u8> = vec![];

        match packet {
            Packet::Handshake {
                version,
                host,
                port,
                next_status,
            } => {
                buf.write_varint(0x00_u32)?;

                buf.write_varint(version as u32)?;

                buf.write_varint(host.as_bytes().len() as u32)?;
                buf.write_all(host.as_bytes())?;

                buf.write_u16::<BigEndian>(port)?;

                buf.write_varint(next_status as u32)?;
            }
            Packet::StatusRequest => {
                buf.write_varint(0x00_u32)?;
            }
        }

        let mut packet_buf = vec![];
        packet_buf.write_varint(buf.len() as u32)?;
        packet_buf.write_all(&buf)?;

        self.stream.write_all(&packet_buf)?;

        Ok(())
    }

    pub fn read_handshake_resp_packet(&mut self) -> anyhow::Result<String> {
        let _: u32 = self.stream.read_varint()?;
        let packet_id: u32 = self.stream.read_varint()?;
        if packet_id != 0x00 {
            panic!("Unsupported protocol: packet_id={}", packet_id);
        }

        let len: u32 = self.stream.read_varint()?;

        let mut buf = vec![0; len as usize];
        self.stream.read_exact(&mut buf)?;

        let mut cur = Cursor::new(buf);
        let mut s = String::new();
        cur.read_to_string(&mut s)?;

        Ok(s)
    }
}

pub fn read_handshake_packet(stream: &mut TcpStream) -> anyhow::Result<Packet> {
    let packet_len: u32 = stream.read_varint()?;

    let packet_id: u32 = stream.read_varint()?;
    if packet_id != 0x00 {
        panic!(
            "Unsupported protocol: packet_id={:02X}, packet_len={:02X}",
            packet_id, packet_len
        );
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

    Ok(Packet::Handshake {
        version: version as i32,
        host,
        port,
        next_status: next_status as i32,
    })
}
