use std::net::{SocketAddr, ToSocketAddrs};

use bendy::decoding::{self, FromBencode, Object};
use tokio::task;

#[derive(Debug, PartialEq, Eq)]
pub struct Peer {
    peer_id: Option<String>,
    address: SocketAddr,
}

impl Peer {
    pub fn new(peer_id: Option<String>, address: SocketAddr) -> Self {
        Self { peer_id, address }
    }
}

impl FromBencode for Peer {
    fn decode_bencode_object(object: Object) -> Result<Self, decoding::Error>
    where
        Self: Sized,
    {
        let mut peer_id = None;
        let mut ip = None;
        let mut port = None;
        let mut dict = object.try_into_dictionary()?;

        while let Some(pair) = dict.next_pair()? {
            match pair {
                (b"peer id", val) => peer_id = Some(String::decode_bencode_object(val)?),
                (b"ip", val) => {
                    // IP can be v6, v4, or a DNS name, but we'll just treat it as a String
                    // TODO: This might fail if an ip is a bunch of random bytes
                    ip = Some(String::decode_bencode_object(val)?)
                }
                (b"port", val) => port = Some(u16::decode_bencode_object(val)?),
                (other, _) => {
                    return Err(decoding::Error::unexpected_field(String::from_utf8_lossy(
                        other,
                    )));
                }
            }
        }

        let ip: String = ip.ok_or_else(|| decoding::Error::missing_field("ip"))?;
        let port: u16 = port.ok_or_else(|| decoding::Error::missing_field("port"))?;
        let address = task::block_in_place(|| (ip.as_str(), port).to_socket_addrs())?
            .next()
            .ok_or_else(|| {
                // TODO: Eh, UnexpectedToken is not quite the right error, but trying to make
                // a MalformedContent error requires making a failure::Error which is too much
                // work
                decoding::Error::unexpected_token(
                    "an IP address",
                    format!("{}:{}", ip.as_str(), port),
                )
            })?;

        Ok(Self::new(peer_id, address))
    }
}
