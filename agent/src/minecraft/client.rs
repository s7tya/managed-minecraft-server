use crate::minecraft::{
    packet::{self, Packet},
    status,
};

pub struct Client {
    status: ClientStatus,
    conn: packet::Connection,
    host: String,
    port: u16,
}

enum ClientStatus {
    BeforeHandshake,
    AfterHandshake,
}

impl Client {
    pub fn new(host: &str, port: u16) -> anyhow::Result<Self> {
        let conn = packet::Connection::new(host, port)?;

        Ok(Client {
            status: ClientStatus::BeforeHandshake,
            conn,
            host: host.to_string(),
            port,
        })
    }

    pub fn handshake(&mut self) -> anyhow::Result<()> {
        let packet = Packet::Handshake {
            version: 765,
            host: self.host.clone(),
            port: self.port,
            next_status: 1,
        };

        self.conn.send_packet(packet)?;

        self.status = ClientStatus::AfterHandshake;

        Ok(())
    }

    pub fn status(&mut self) -> anyhow::Result<status::Response> {
        if let ClientStatus::BeforeHandshake = self.status {
            self.handshake()?;
        }

        self.conn.send_packet(Packet::StatusRequest)?;
        let res = self.conn.read_packet()?;

        let value: status::Response = serde_json::from_str(&res)?;

        Ok(value)
    }

    pub fn get_online_players_count(&mut self) -> anyhow::Result<usize> {
        self.handshake()?;
        let status = self.status()?;
        Ok(status.players.online)
    }
}
