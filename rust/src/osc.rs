use std::net::{SocketAddr, UdpSocket};

use rosc::{encoder, OscMessage, OscPacket, OscType};

pub struct OscOutput {
    socket: UdpSocket,
    target: SocketAddr,
    address: String,
}

impl OscOutput {
    pub fn new(host: &str, port: u16, address: &str) -> Result<Self, String> {
        let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
        let target: SocketAddr = format!("{host}:{port}")
            .parse()
            .map_err(|e: std::net::AddrParseError| e.to_string())?;
        Ok(Self {
            socket,
            target,
            address: address.to_string(),
        })
    }

    pub fn send(&self, value: f32) {
        let packet = OscPacket::Message(OscMessage {
            addr: self.address.clone(),
            args: vec![OscType::Float(value)],
        });
        // Sent at up to 50Hz during a fade - deliberately not logged per-message to avoid
        // spamming stdout; only failures are surfaced.
        match encoder::encode(&packet) {
            Ok(bytes) => {
                if let Err(e) = self.socket.send_to(&bytes, self.target) {
                    eprintln!("[OSC] failed to send to {}: {e}", self.target);
                }
            }
            Err(e) => eprintln!("[OSC] failed to encode message: {e:?}"),
        }
    }
}
