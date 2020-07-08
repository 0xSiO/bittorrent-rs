use std::collections::HashMap;

use bendy::decoding::{self, FromBencode, Object};
use serde::{Deserialize, Serialize};
use sha1::Digest;

#[derive(Debug, PartialEq, Eq)]
pub struct Response {
    files: HashMap<Digest, TorrentStats>,
}

impl Response {
    pub fn new(files: HashMap<Digest, TorrentStats>) -> Self {
        Self { files }
    }

    pub fn files(&self) -> &HashMap<Digest, TorrentStats> {
        &self.files
    }
}

impl FromBencode for Response {
    fn decode_bencode_object(object: Object) -> Result<Self, decoding::Error>
    where
        Self: Sized,
    {
        let mut files = None;
        let mut dict = object.try_into_dictionary()?;

        while let Some(pair) = dict.next_pair()? {
            match pair {
                (b"files", val) => {
                    let mut files_map = HashMap::new();
                    let mut files_dict = val.try_into_dictionary()?;
                    while let Some(pair) = files_dict.next_pair()? {
                        match pair {
                            (bytes, stats_obj) => {
                                let digest = hex::encode(bytes).parse().unwrap();
                                let _ = files_map.insert(
                                    digest,
                                    bendy::serde::from_bytes(
                                        stats_obj.try_into_dictionary()?.into_raw()?,
                                    )?,
                                );
                            }
                        }
                    }
                    files = Some(files_map);
                }
                // TODO: Add unofficial extension fields
                (other, _) => {
                    return Err(decoding::Error::unexpected_field(String::from_utf8_lossy(
                        other,
                    )));
                }
            }
        }

        let files = files.ok_or_else(|| decoding::Error::missing_field("files"))?;
        Ok(Self::new(files))
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TorrentStats {
    complete: u64,
    downloaded: u64,
    incomplete: u64,
    name: Option<String>,
}

impl TorrentStats {
    pub fn new(complete: u64, downloaded: u64, incomplete: u64, name: Option<String>) -> Self {
        Self {
            complete,
            downloaded,
            incomplete,
            name,
        }
    }
}
