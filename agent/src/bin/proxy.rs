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
use tokio::{
    net::{TcpListener, TcpStream},
    try_join,
};

struct Server<'a> {
    is_proxy: Arc<Mutex<bool>>,
    client_address: &'a str,
    server_address: &'a str,
}

impl<'a> Server<'a> {
    pub fn new(client_address: &'a str, server_address: &'a str) -> Self {
        Self {
            is_proxy: Arc::new(Mutex::new(false)),
            client_address,
            server_address,
        }
    }

    async fn handle_request(&self, stream: TcpStream) -> anyhow::Result<()> {
        let is_proxy = *self.is_proxy.lock().unwrap();
        if is_proxy {
            self.handle_proxy(stream).await?;
        } else {
            let mut std_stream = stream.into_std()?;
            std_stream.set_nonblocking(false)?;
            self.handle_motd(&mut std_stream)?;
        }

        Ok(())
    }

    async fn handle_proxy(&self, mut client_conn: TcpStream) -> anyhow::Result<()> {
        let mut main_server_conn = TcpStream::connect(&self.server_address).await?;
        let (mut client_recv, mut client_send) = client_conn.split();
        let (mut server_recv, mut server_send) = main_server_conn.split();

        let handle_one = async { tokio::io::copy(&mut server_recv, &mut client_send).await };
        let handle_two = async { tokio::io::copy(&mut client_recv, &mut server_send).await };

        try_join!(handle_one, handle_two)?;

        Ok(())
    }

    fn handle_motd(&self, stream: &mut std::net::TcpStream) -> anyhow::Result<()> {
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

impl Clone for Server<'_> {
    fn clone(&self) -> Self {
        Server {
            is_proxy: Arc::clone(&self.is_proxy),
            client_address: self.client_address,
            server_address: self.server_address,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new("127.0.0.1:25564", "127.0.0.1:25565");
    let listener = TcpListener::bind(server.client_address).await?;

    loop {
        let server = server.clone();
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = server.handle_request(stream).await {
                eprintln!("Error handling request: {e}");
            }
        })
        .await?;
    }
}
