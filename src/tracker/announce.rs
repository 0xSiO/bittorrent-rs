use std::{
    convert::TryFrom,
    fmt::{self, Display},
    net::IpAddr,
};

use bendy::decoding::{self, FromBencode, ListDecoder, Object};
use either::Either;
use reqwest::Url;

use crate::Peer;

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

// TODO: Look into request parameter named 'corrupt'
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

#[derive(Debug, PartialEq, Eq)]
pub struct Response {
    failure_reason: Option<String>,
    warning_message: Option<String>,
    interval: Option<u64>,
    min_interval: Option<u64>,
    tracker_id: Option<String>,
    complete: Option<u64>,
    incomplete: Option<u64>,
    downloaded: Option<u64>,
    peers: Option<Vec<Peer>>,
}

impl Response {
    pub fn new(
        failure_reason: Option<String>,
        warning_message: Option<String>,
        interval: Option<u64>,
        min_interval: Option<u64>,
        tracker_id: Option<String>,
        complete: Option<u64>,
        incomplete: Option<u64>,
        downloaded: Option<u64>,
        peers: Option<Vec<Peer>>,
    ) -> Self {
        Self {
            failure_reason,
            warning_message,
            interval,
            min_interval,
            tracker_id,
            complete,
            incomplete,
            downloaded,
            peers,
        }
    }
}

impl FromBencode for Response {
    const EXPECTED_RECURSION_DEPTH: usize = 2;

    fn decode_bencode_object(object: Object) -> Result<Self, decoding::Error>
    where
        Self: Sized,
    {
        let mut failure_reason = None;
        let mut warning_message = None;
        let mut interval = None;
        let mut min_interval = None;
        let mut tracker_id = None;
        let mut complete = None;
        let mut incomplete = None;
        let mut downloaded = None;
        let mut peers = None;
        let mut dict = object.try_into_dictionary()?;

        while let Some(pair) = dict.next_pair()? {
            match pair {
                (b"failure reason", val) => {
                    failure_reason = Some(String::decode_bencode_object(val)?)
                }
                (b"warning message", val) => {
                    warning_message = Some(String::decode_bencode_object(val)?)
                }
                (b"interval", val) => interval = Some(u64::decode_bencode_object(val)?),
                (b"min interval", val) => min_interval = Some(u64::decode_bencode_object(val)?),
                (b"tracker id", val) => tracker_id = Some(String::decode_bencode_object(val)?),
                (b"complete", val) => complete = Some(u64::decode_bencode_object(val)?),
                (b"incomplete", val) => incomplete = Some(u64::decode_bencode_object(val)?),
                (b"downloaded", val) => downloaded = Some(u64::decode_bencode_object(val)?),
                (b"peers", val) => {
                    // Peer list is either a list of dictionaries or a byte string
                    let peers_obj: Either<ListDecoder, &[u8]> =
                        Either::from(val.bytes_or_else(|obj| Err(obj.try_into_list().unwrap())));
                    let mut peer_list = Vec::new();
                    match peers_obj {
                        Either::Left(mut list) => {
                            while let Some(obj) = list.next_object()? {
                                peer_list.push(Peer::decode_bencode_object(obj)?)
                            }
                        }
                        Either::Right(bytes) => {
                            for chunk in bytes.chunks(6) {
                                peer_list.push(Peer::try_from(chunk)?);
                            }
                        }
                    }
                    peers = Some(peer_list);
                }
                (other, _) => {
                    return Err(decoding::Error::unexpected_field(String::from_utf8_lossy(
                        other,
                    )));
                }
            }
        }

        Ok(Self::new(
            failure_reason,
            warning_message,
            interval,
            min_interval,
            tracker_id,
            complete,
            incomplete,
            downloaded,
            peers,
        ))
    }
}
