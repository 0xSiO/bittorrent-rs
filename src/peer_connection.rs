use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufStream},
    net::TcpStream,
};

use crate::{error::*, Peer};

pub struct PeerConnection {
    peer: Peer,
    stream: BufStream<TcpStream>,
}

impl PeerConnection {
    // TODO: Peer protocol over TCP is rarely used nowadays
    pub async fn new(peer: Peer) -> Result<Self> {
        let stream = BufStream::new(TcpStream::connect(peer.address()).await?);
        Ok(Self { peer, stream })
    }

    pub async fn send_handshake(&mut self, info_hash: sha1::Digest, peer_id: &[u8]) -> Result<()> {
        self.stream.write_u8(19).await?;
        self.stream.write_all(b"BitTorrent protocol").await?;
        self.stream.write_u64(0b00000000);
        self.stream.write_all(&info_hash.bytes()).await?;
        self.stream.write_all(peer_id).await?;
        self.stream.flush().await?;
        log::debug!("Handshake sent");
        Ok(())
    }

    pub async fn recv_handshake(&mut self) -> Result<()> {
        let proto_str_len = self.stream.read_u8().await? as usize;
        let mut buf = vec![0; proto_str_len];
        self.stream.read_exact(&mut buf).await?;
        log::debug!("Protocol is '{}'", String::from_utf8_lossy(&buf));
        let reserved_bytes = self.stream.read_u64().await?;
        log::debug!("Reserved bytes: {:#b}", reserved_bytes);
        buf = vec![0; 20];
        self.stream.read_exact(&mut buf).await?;
        log::debug!("Info hash: {:?}", buf);
        Ok(())
    }
}
