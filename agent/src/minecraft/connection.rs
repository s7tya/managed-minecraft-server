use core::panic;
use std::{
    io::{Cursor, Read},
    net::TcpStream,
};

use integer_encoding::VarIntReader;

use super::packet::{PacketEncoder, WritePacketExt};

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(host: &str, port: u16) -> anyhow::Result<Self> {
        let stream = TcpStream::connect((host.to_string(), port))?;
        stream.set_nodelay(true)?;

        Ok(Connection { stream })
    }

    pub fn send_packet<P: PacketEncoder>(&mut self, packet: P) -> anyhow::Result<()> {
        self.stream.write_packet(packet)?;
        Ok(())
    }

    pub fn read_handshake_resp_packet(&mut self) -> anyhow::Result<String> {
        let _: u32 = self.stream.read_varint()?;
        let packet_id: u32 = self.stream.read_varint()?;
        if packet_id != 0x00 {
            panic!("Unsupported protocol: packet_id={}", packet_id);
        }

        let len: u32 = self.stream.read_varint()?;

        let mut buf = vec![0; len as usize];
        self.stream.read_exact(&mut buf)?;

        let mut cur = Cursor::new(buf);
        let mut s = String::new();
        cur.read_to_string(&mut s)?;

        Ok(s)
    }
}
