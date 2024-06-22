use std::net::TcpListener;

use agent::minecraft::{
    packet::{
        disconnect_login::DisconnectLogin,
        handshake::Handshake,
        ping::Ping,
        read_packet,
        status_request::StatusRequest,
        status_response::{self, Players, Version},
        WritePacketExt,
    },
    raw_json_text::RawJsonText,
};

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:25565")?;

    for stream in listener.incoming() {
        let mut stream = stream?;
        let handshake: Handshake = read_packet(&mut stream)?;

        match handshake.next_status {
            0x01 => {
                let _status_request: StatusRequest = read_packet(&mut stream)?;

                let status_response = status_response::StatusResponse {
                    version: Version {
                        name: "Motd Only Server".to_string(),
                        protocol: 765,
                    },
                    players: Players {
                        max: 100,
                        online: 1,
                        sample: None,
                    },
                    description: RawJsonText::String("Hello from Rust!".to_string()),
                    modinfo: None,
                    favicon: None,
                };
                stream.write_packet(status_response)?;

                let ping: Ping = read_packet(&mut stream)?;
                stream.write_packet(ping)?;
            }
            0x02 => {
                stream.write_packet(DisconnectLogin {
                    reason: RawJsonText::String("Hello!".to_string()),
                })?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid next_state: {}",
                    handshake.next_status
                ))
            }
        }
    }

    Ok(())
}
