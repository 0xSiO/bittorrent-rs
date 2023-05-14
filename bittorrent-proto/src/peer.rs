use std::{
    convert::TryFrom,
    net::{SocketAddr, ToSocketAddrs},
};

use bendy::decoding::{self, FromBencode, Object};
use tokio::task;

mod message;

use crate::error::Error;

pub use message::Message;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Peer {
    peer_id: Option<String>,
    address: SocketAddr,
}

impl Peer {
    pub fn new(peer_id: Option<String>, address: SocketAddr) -> Self {
        Self { peer_id, address }
    }

    pub fn address(&self) -> SocketAddr {
        self.address
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
                    // TODO: This assumes the field can be parsed as a string
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
                decoding::Error::malformed_content(Error::InvalidSocketAddress(ip, port))
            })?;

        Ok(Self::new(peer_id, address))
    }
}

impl TryFrom<&[u8]> for Peer {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> crate::error::Result<Self> {
        if bytes.len() == 6 {
            let ip = [bytes[0], bytes[1], bytes[2], bytes[3]];
            let port = (bytes[4] as u16) * 100 + bytes[5] as u16;
            Ok(Self::new(None, SocketAddr::from((ip, port))))
        } else {
            Err(Error::InvalidCompactPeerLength(bytes.len()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use bendy::decoding::{Decoder, FromBencode};

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn old_conversion_test_ipv4() {
        let bencode_ipv4 = b"d2:ip9:127.0.0.17:peer id6:abcdef4:porti6080ee";
        let mut decoder = Decoder::new(bencode_ipv4);
        let dict = decoder.next_object().unwrap().unwrap();
        assert_eq!(
            Peer::decode_bencode_object(dict).unwrap(),
            Peer::new(
                Some(String::from("abcdef"),),
                "127.0.0.1:6080".parse().unwrap()
            )
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn old_conversion_test_ipv6() {
        let bencode_ipv6 = b"d2:ip24:fe80::202:b3ff:fe1e:83297:peer id6:abcdef4:porti6080ee";
        let mut decoder = Decoder::new(bencode_ipv6);
        let dict = decoder.next_object().unwrap().unwrap();
        assert_eq!(
            Peer::decode_bencode_object(dict).unwrap(),
            Peer::new(
                Some(String::from("abcdef")),
                SocketAddr::from(("fe80::202:b3ff:fe1e:8329".parse::<IpAddr>().unwrap(), 6080))
            )
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn old_conversion_test_dns() {
        let bencode_dns = b"d2:ip11:example.com7:peer id6:abcdef4:porti80ee";
        let mut decoder = Decoder::new(bencode_dns);
        let dict = decoder.next_object().unwrap().unwrap();
        assert_eq!(
            Peer::decode_bencode_object(dict).unwrap(),
            Peer::new(
                Some(String::from("abcdef")),
                ("example.com", 80)
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .unwrap()
            )
        );
    }

    #[test]
    fn conversion_test() {
        let bytes: &[u8] = &[127, 0, 0, 1, 60, 80];
        let peer = Peer::new(None, "127.0.0.1:6080".parse().unwrap());
        assert_eq!(Peer::try_from(bytes).unwrap(), peer);
    }
}
