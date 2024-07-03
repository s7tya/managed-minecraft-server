use agent::minecraft::{
    self, client,
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
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_ec2::Client;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    time, try_join,
};

const USAGE_CHECK_INTERVAL: time::Duration = time::Duration::from_secs(5);

struct Server<'a> {
    is_proxy: Arc<Mutex<bool>>,
    client_address: &'a str,
    server_address: &'a str,
    instance_id: &'a str,
}

impl<'a> Server<'a> {
    pub fn new(client_address: &'a str, server_address: &'a str, instance_id: &'a str) -> Self {
        Self {
            is_proxy: Arc::new(Mutex::new(false)),
            client_address,
            server_address,
            instance_id,
        }
    }

    async fn handle_request(&self, stream: TcpStream) -> anyhow::Result<()> {
        let is_proxy = *self.is_proxy.lock().unwrap();
        if is_proxy {
            self.handle_proxy(stream).await?;
        } else {
            let mut std_stream = stream.into_std()?;
            std_stream.set_nonblocking(false)?;
            self.handle_motd(&mut std_stream).await?;
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

    async fn handle_motd(&self, stream: &mut std::net::TcpStream) -> anyhow::Result<()> {
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
                if self.start_instance().await.is_ok() {
                    stream.write_packet(DisconnectLogin {
                        reason: RawJsonText::String(
                            "プロキシを開始しました。再度接続してください。".to_string(),
                        ),
                    })?;

                    let mut is_server_active = false;
                    while !is_server_active {
                        let address = &self.server_address.split(':').collect::<Vec<_>>();
                        let mut client =
                            minecraft::client::Client::new(address[0], address[1].parse()?)?;
                        is_server_active = client.status().is_ok();

                        thread::sleep(Duration::from_secs(20));
                    }

                    let mut is_proxy = self.is_proxy.lock().unwrap();
                    *is_proxy = true;
                } else {
                    stream.write_packet(DisconnectLogin {
                        reason: RawJsonText::String(
                            "サーバを起動できませんでした。後ほど試してください。".to_string(),
                        ),
                    })?;
                }
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

    async fn start_instance(&self) -> anyhow::Result<()> {
        let region_provider = RegionProviderChain::default_provider().or_else("ap-northeast-1");
        let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .region(region_provider)
            .load()
            .await;
        let client = Client::new(&config);

        let _start_instances_response = client
            .start_instances()
            .instance_ids(self.instance_id)
            .send()
            .await?;

        Ok(())
    }

    async fn stop_instance(&self) -> anyhow::Result<()> {
        let region_provider = RegionProviderChain::default_provider().or_else("ap-northeast-1");
        let config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&config);

        let _stop_instances_response = client
            .stop_instances()
            .instance_ids(self.instance_id)
            .send()
            .await?;

        Ok(())
    }
}

impl Clone for Server<'_> {
    fn clone(&self) -> Self {
        Server {
            is_proxy: Arc::clone(&self.is_proxy),
            client_address: self.client_address,
            server_address: self.server_address,
            instance_id: self.instance_id,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let client_addr = "127.0.0.1:25564";
    let server_addr = "54.199.116.219:25565";
    let instance_id = "i-07ad2faad1eaed991";
    let server = Server::new(client_addr, server_addr, instance_id);
    let listener = TcpListener::bind(server.client_address).await?;

    let mut interval = time::interval(USAGE_CHECK_INTERVAL);
    let split = server_addr.split(':').collect::<Vec<_>>();
    let (host, port) = (split[0], split[1].parse()?);

    tokio::task::spawn({
        let is_proxy = server.clone().is_proxy.clone();
        let server = server.clone();
        async move {
            let mut inactive_count = 0;

            loop {
                interval.tick().await;

                {
                    let is_proxy = is_proxy.lock().unwrap();
                    if !*is_proxy {
                        continue;
                    }
                }

                let mut client = client::Client::new(host, port).unwrap();
                let status = client.status().unwrap();
                if status.players.online == 0 {
                    inactive_count += 1;
                } else {
                    inactive_count = 0;
                }

                if inactive_count >= 3 {
                    server.stop_instance().await.unwrap();

                    {
                        let mut is_proxy = is_proxy.lock().unwrap();
                        *is_proxy = false;
                    }
                }
            }
        }
    });

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn({
            let server = server.clone();
            async move {
                if let Err(e) = server.handle_request(stream).await {
                    eprintln!("Error handling request: {e}");
                }
            }
        });
    }
}
