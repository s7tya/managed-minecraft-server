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
use std::sync::{Arc, Mutex};
use tokio::try_join;

struct Server {
    is_proxy: Arc<Mutex<bool>>,
}

const CLIENT_ADDR: &str = "127.0.0.1:25564";
const SERVER_ADDR: &str = "127.0.0.1:25565";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server {
        is_proxy: Arc::new(Mutex::new(false)),
    };

    loop {
        let is_proxy = *server.is_proxy.lock().unwrap();

        if is_proxy {
            let listener = tokio::net::TcpListener::bind(CLIENT_ADDR).await?;
            let (client, _) = listener.accept().await?;
            if let Err(e) = handle_proxy(client).await {
                eprintln!("Error handling client connection: {:?}", e);
            }
        } else {
            let listener = std::net::TcpListener::bind(SERVER_ADDR)?;
            let (mut stream, _) = listener.accept()?;
            server.handle_request(&mut stream)?;
        }
    }
}

async fn handle_proxy(
    mut client_conn: tokio::net::TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut main_server_conn = tokio::net::TcpStream::connect(SERVER_ADDR).await?;
    let (mut client_recv, mut client_send) = client_conn.split();
    let (mut server_recv, mut server_send) = main_server_conn.split();

    let handle_one = async { tokio::io::copy(&mut server_recv, &mut client_send).await };
    let handle_two = async { tokio::io::copy(&mut client_recv, &mut server_send).await };

    try_join!(handle_one, handle_two)?;

    Ok(())
}

impl Server {
    fn handle_request(&self, stream: &mut std::net::TcpStream) -> anyhow::Result<()> {
        let handshake: Handshake = read_packet(stream)?;

        match handshake.next_status {
            0x01 => {
                let _status_request: StatusRequest = read_packet(stream)?;

                let status_response = status_response::StatusResponse {
                    version: Version {
                        name: "Not Proxying".to_string(),
                        protocol: 767,
                    },
                    players: Players {
                        max: 100,
                        online: 1,
                        sample: None,
                    },
                    description: RawJsonText::String("接続してプロキシを開始".to_string()),
                    modinfo: None,
                    favicon: None,
                };
                stream.write_packet(status_response)?;

                let ping: Ping = read_packet(stream)?;
                stream.write_packet(ping)?;
            }
            0x02 => {
                stream.write_packet(DisconnectLogin {
                    reason: RawJsonText::String(
                        "プロキシを開始しました。再度接続してください。".to_string(),
                    ),
                })?;

                let mut is_proxy = self.is_proxy.lock().unwrap();
                *is_proxy = true;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid next_state: {}",
                    handshake.next_status
                ));
            }
        }

        Ok(())
    }
}

// Clone implementation for Server
impl Clone for Server {
    fn clone(&self) -> Self {
        Server {
            is_proxy: Arc::clone(&self.is_proxy),
        }
    }
}
