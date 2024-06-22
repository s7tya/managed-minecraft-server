use std::net::TcpListener;

use super::{
    packet::{read_packet, Handshake, Packet, Ping, StatusRequest, WritePacketExt},
    raw_json_text::RawJsonText,
    status::{self, Players, Version},
};

#[derive(Default)]
pub struct Server {}

impl Server {
    pub fn serve() -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:25565")?;

        for stream in listener.incoming() {
            let mut stream = stream?;
            let _handshake: Handshake = read_packet(&mut stream)?;
            let _status_request: StatusRequest = read_packet(&mut stream)?;

            let status_response = status::StatusResponse {
                version: Version {
                    name: "Motd Only Server".to_string(),
                    protocol: 765,
                },
                players: Players { max: 0, online: 0 },
                description: RawJsonText::String("Hello from Rust!".to_string()),
                modinfo: None,
                favicon: None,
            };
            stream.write_packet(Packet::StatusResponse(status_response))?;

            let ping: Ping = read_packet(&mut stream)?;
            stream.write_packet(Packet::Ping(ping))?;
        }

        Ok(())
    }
}
