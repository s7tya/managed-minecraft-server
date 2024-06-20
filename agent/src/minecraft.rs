use std::{
    io::{Cursor, Read, Write},
    net::TcpStream,
};

use integer_encoding::{VarIntReader, VarIntWriter};

const MINECRAFT_PROTOCOL_VERSION: usize = 765;

enum Packet {
    Handshake {
        version: usize,
        host: String,
        port: u16,
        next_status: usize,
    },
}

impl Packet {
    fn send_packet(self) -> anyhow::Result<()> {
        match self {
            Packet::Handshake {
                version,
                host,
                port,
                next_status,
            } => {
                let mut buf: Vec<u8> = Vec::new();
                buf.write_varint(version)?;
                buf.write_varint(host.len())?;
                buf.write_all(host.as_bytes())?;
                buf.write_varint(port)?;
                buf.write_varint(next_status)?;

                let mut stream = TcpStream::connect((host, port))?;
                stream.set_nodelay(true)?;

                stream.write_varint(buf.len())?;
                stream.write_all(&buf)?;

                stream.write_all(&[0x01, 0x00])?;

                let len = stream.read_varint()?;
                let mut buf = vec![0; len];
                stream.read_exact(&mut buf)?;
                let mut cur = Cursor::new(buf);

                match cur.read_varint().unwrap() {
                    0x00 => {
                        let mut s = String::new();
                        cur.read_to_string(&mut s)?;

                        println!("resp: {}", s);
                    }
                    _ => {
                        panic!("Invalid resp");
                    }
                }

                Ok(())
            }
        }
    }
}

pub fn get_online_players_count(host: &str, port: u16) -> anyhow::Result<()> {
    let packet = Packet::Handshake {
        version: MINECRAFT_PROTOCOL_VERSION,
        host: host.to_string(),
        port,
        next_status: 1,
    };

    packet.send_packet()?;

    Ok(())
}
