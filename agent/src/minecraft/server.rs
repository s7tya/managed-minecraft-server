use std::{
    io::{Cursor, Read, Write},
    net::TcpListener,
};

use byteorder::{BigEndian, ReadBytesExt};
use integer_encoding::{VarIntReader, VarIntWriter};

use super::{
    raw_json_text::RawJsonText,
    status::{self, Players, Version},
};

#[derive(Default)]
pub struct Server {}

impl Server {
    pub fn serve() -> anyhow::Result<()> {
        let listener = TcpListener::bind("localhost:25565")?;

        loop {
            let (mut stream, _) = listener.accept()?;

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

            let resp = status::Response {
                version: Version {
                    name: "Waterfall 0.0.1".to_string(),
                    protocol: 765,
                },
                players: Players { max: 0, online: 0 },
                description: RawJsonText::String("paco".to_string()),
                modinfo: None,
                favicon: None,
            };

            let resp_body = serde_json::to_string(&resp)?;

            stream.write_varint(0x00_u32)?;
            stream.write_varint(resp_body.as_bytes().len() as u32)?;
            stream.write_all(resp_body.as_bytes())?;

            let packet_len: u32 = stream.read_varint()?;
            let packet_id: u32 = stream.read_varint()?;
            if packet_id != 0x00 {
                panic!(
                    "Unsupported protocol: packet_id={:02X?}, packet_len={}",
                    packet_id, packet_len
                );
            }

            println!(
                "request to {host}:{port} (protocol_version={version}, next_status={next_status})"
            );
        }
    }
}
