use std::{
    collections::HashMap,
    fmt::{self, Display},
    net::IpAddr,
};

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
    pub fn new(
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

impl From<Request> for HashMap<&str, String> {
    fn from(request: Request) -> Self {
        let mut params = HashMap::with_capacity(13);
        params.insert("info_hash", request.info_hash);
        params.insert("peer_id", request.peer_id);
        if let Some(ip) = request.ip {
            params.insert("ip", ip.to_string());
        }
        params.insert("port", request.port.to_string());
        params.insert("uploaded", request.uploaded.to_string());
        params.insert("downloaded", request.downloaded.to_string());
        params.insert("left", request.left.to_string());
        if let Some(event) = request.event {
            params.insert("event", event.to_string());
        }
        if request.compact {
            params.insert("compact", String::from("1"));
        } else {
            params.insert("compact", String::from("0"));
        }
        if let Some(no_peer_id) = request.no_peer_id {
            if no_peer_id {
                params.insert("no_peer_id", String::from("1"));
            } else {
                params.insert("no_peer_id", String::from("0"));
            }
        }
        if let Some(numwant) = request.numwant {
            params.insert("numwant", numwant.to_string());
        }
        if let Some(key) = request.key {
            params.insert("key", key);
        }
        if let Some(trackerid) = request.trackerid {
            params.insert("trackerid", trackerid);
        }
        params
    }
}
