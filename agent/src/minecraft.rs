use byteorder::{BigEndian, WriteBytesExt};
use core::panic;
use serde::Deserialize;
use std::{
    io::{Cursor, Read, Write},
    net::TcpStream,
};

use integer_encoding::{VarIntReader, VarIntWriter};

const MINECRAFT_PROTOCOL_VERSION: i32 = -1;

struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(host: &str, port: u16) -> anyhow::Result<Self> {
        let stream = TcpStream::connect((host.to_string(), port))?;
        stream.set_nodelay(true)?;

        Ok(Connection { stream })
    }

    fn send_packet(&mut self, packet: Packet) -> anyhow::Result<()> {
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

                buf.write_varint(host.len() as u32)?;
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

    fn read_packet(&mut self) -> anyhow::Result<String> {
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

enum Packet {
    Handshake {
        version: i32,
        host: String,
        port: u16,
        next_status: i32,
    },
    StatusRequest,
}

#[derive(Deserialize, Debug)]
struct StatusResponse {
    pub version: StatusResponseVersion,
    pub players: StatusResponsePlayers,
}

#[derive(Deserialize, Debug)]
struct StatusResponseVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Deserialize, Debug)]
struct StatusResponsePlayers {
    pub max: usize,
    pub online: usize,
}

pub fn get_online_players_count(host: &str, port: u16) -> anyhow::Result<usize> {
    let packet = Packet::Handshake {
        version: MINECRAFT_PROTOCOL_VERSION,
        host: host.to_string(),
        port,
        next_status: 1,
    };

    let mut conn = Connection::new(host, port)?;

    conn.send_packet(packet)?;
    conn.send_packet(Packet::StatusRequest)?;

    let res = conn.read_packet()?;
    let value: StatusResponse = serde_json::from_str(&res)?;

    Ok(value.players.online)
}
