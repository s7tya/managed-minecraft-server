use crate::minecraft::{packet::Packet, status};

use super::{
    connection::Connection,
    packet::{Handshake, StatusRequest},
};

pub struct Client {
    status: ClientStatus,
    conn: Connection,
    host: String,
    port: u16,
}

enum ClientStatus {
    BeforeHandshake,
    AfterHandshake,
}

impl Client {
    pub fn new(host: &str, port: u16) -> anyhow::Result<Self> {
        let conn = Connection::new(host, port)?;

        Ok(Client {
            status: ClientStatus::BeforeHandshake,
            conn,
            host: host.to_string(),
            port,
        })
    }

    pub fn handshake(&mut self) -> anyhow::Result<()> {
        let packet = Packet::Handshake(Handshake {
            version: 765,
            host: self.host.clone(),
            port: self.port,
            next_status: 1,
        });

        self.conn.send_packet(packet)?;

        self.status = ClientStatus::AfterHandshake;

        Ok(())
    }

    pub fn status(&mut self) -> anyhow::Result<status::StatusResponse> {
        if let ClientStatus::BeforeHandshake = self.status {
            self.handshake()?;
        }

        self.conn
            .send_packet(Packet::StatusRequest(StatusRequest {}))?;
        let res = self.conn.read_handshake_resp_packet()?;

        let value: status::StatusResponse = serde_json::from_str(&res)?;

        Ok(value)
    }

    pub fn get_online_players_count(&mut self) -> anyhow::Result<usize> {
        let status = self.status()?;
        Ok(status.players.online)
    }
}
