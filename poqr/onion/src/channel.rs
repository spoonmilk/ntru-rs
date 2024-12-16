use crate::{Message, OnionPacket};
use ntru::ntru_key::NtruPublicKey;
use rsa_ext::RsaPublicKey;
use std::io::{Read, Write};
use std::net::TcpStream;

/// A channel between two nodes in the network through which messages can be sent.
pub struct Channel {
    /// The ID of the relay node that the channel is connected to.
    pub id_key: NtruPublicKey,
    /// The public keys of the remote nodes used to encrypt messages sent through the connection.
    pub onion_keys: Vec<RsaPublicKey>,
    /// A TCP connection to the remote node.
    pub connection: TcpStream,
}

impl Channel {
    pub fn send(&mut self, packet: OnionPacket) {
        let bytes = packet.to_be_bytes(self.id_key.clone(), self.onion_keys.clone());
        self.connection.write(&bytes).unwrap();
    }

    pub fn recv(&mut self) -> Message {
        let mut buf: Vec<u8> = Vec::new();
        match self.connection.read(&mut buf) {
            Ok(_) => Message::from_be_bytes(buf),
            Err(_) => panic!("Failed to read from connection"),
        }
    }
}
