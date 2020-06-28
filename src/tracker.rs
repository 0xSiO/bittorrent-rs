use std::{
    fmt::{self, Display},
    net::IpAddr,
};

use reqwest::Url;

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Started,
    Stopped,
    Completed,
    Empty,
}

impl Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Started => f.write_str("started"),
            Event::Stopped => f.write_str("stopped"),
            Event::Completed => f.write_str("completed"),
            Event::Empty => f.write_str("empty"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Request {
    announce_url: Url,
    info_hash: String,
    peer_id: String,
    ip: Option<IpAddr>,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    event: Option<Event>,
    compact: bool,
    no_peer_id: Option<bool>,
    numwant: Option<u64>,
    key: Option<String>,
    trackerid: Option<String>,
}

impl Request {
    /// Creates a `Request` from a URL and parameters. Existing query parameters in the URL
    /// will be overwritten when this `Request` is passed to `reqwest::Url::from`.
    pub fn new(
        announce_url: Url,
        info_hash: String,
        peer_id: String,
        ip: Option<IpAddr>,
        port: u16,
        uploaded: u64,
        downloaded: u64,
        left: u64,
        event: Option<Event>,
        compact: bool,
        no_peer_id: Option<bool>,
        numwant: Option<u64>,
        key: Option<String>,
        trackerid: Option<String>,
    ) -> Self {
        Self {
            announce_url,
            info_hash,
            peer_id,
            ip,
            port,
            uploaded,
            downloaded,
            left,
            event,
            compact,
            no_peer_id,
            numwant,
            key,
            trackerid,
        }
    }
}

impl From<Request> for Url {
    fn from(mut request: Request) -> Self {
        // We don't want the info hash to be double-encoded, so set it directly first
        request
            .announce_url
            .set_query(Some(&format!("info_hash={}", request.info_hash)));

        // Now add the rest of the params
        let mut query_pairs = request.announce_url.query_pairs_mut();
        query_pairs.append_pair("peer_id", &request.peer_id);
        if let Some(ip) = request.ip {
            query_pairs.append_pair("ip", &ip.to_string());
        }
        query_pairs.append_pair("port", &request.port.to_string());
        query_pairs.append_pair("uploaded", &request.uploaded.to_string());
        query_pairs.append_pair("downloaded", &request.downloaded.to_string());
        query_pairs.append_pair("left", &request.left.to_string());
        if let Some(event) = request.event {
            query_pairs.append_pair("event", &event.to_string());
        }
        query_pairs.append_pair("compact", &(request.compact as u8).to_string());
        if let Some(no_peer_id) = request.no_peer_id {
            query_pairs.append_pair("no_peer_id", &(no_peer_id as u8).to_string());
        }
        if let Some(numwant) = request.numwant {
            query_pairs.append_pair("numwant", &numwant.to_string());
        }
        if let Some(key) = request.key {
            query_pairs.append_pair("key", &key);
        }
        if let Some(trackerid) = request.trackerid {
            query_pairs.append_pair("trackerid", &trackerid);
        }

        // Drop mutable reference to announce_url so we can move it
        drop(query_pairs);
        request.announce_url
    }
}
