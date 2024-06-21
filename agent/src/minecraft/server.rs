use std::{io::Write, net::TcpListener};

use integer_encoding::{VarIntReader, VarIntWriter};

use crate::minecraft::packet::read_handshake_packet;

use super::{
    raw_json_text::RawJsonText,
    status::{self, Players, Version},
};

#[derive(Default)]
pub struct Server {}

impl Server {
    pub fn serve() -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:25565")?;

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let _ = read_handshake_packet(&mut stream)?;

                    let packet_len: u32 = stream.read_varint()?;
                    let packet_id: u32 = stream.read_varint()?;
                    if packet_id != 0x00 {
                        panic!(
                            "Unsupported protocol: packet_id={:02X?}, packet_len={}",
                            packet_id, packet_len
                        );
                    }

                    let resp = status::Response {
                        version: Version {
                            name: "Motd Only Server".to_string(),
                            protocol: 765,
                        },
                        players: Players { max: 0, online: 0 },
                        description: RawJsonText::String("Hello from Rust!".to_string()),
                        modinfo: None,
                        favicon: None,
                    };

                    let mut buf: Vec<u8> = vec![];
                    buf.write_varint(0x00_u32)?;

                    let s = serde_json::to_string(&resp)?.into_bytes();
                    buf.write_varint(s.len() as u32)?;
                    buf.write_all(&s)?;

                    stream.write_varint(buf.len() as u32)?;
                    stream.write_all(&buf)?;
                }
                Err(e) => {
                    println!("Err: {}", e);
                }
            }
        }

        Ok(())
    }
}
