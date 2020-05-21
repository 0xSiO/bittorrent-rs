use crate::{error::Error, info::Info};
use bendy::{
    decoding::{self, FromBencode, Object},
    encoding::{self, SingleItemEncoder, ToBencode},
};
use chrono::NaiveDateTime;

#[derive(Debug, PartialEq, Eq)]
pub struct MetaInfo {
    announce: String,
    info: Info,
    announce_list: Option<Vec<Vec<String>>>,
    creation_date: Option<NaiveDateTime>,
}

impl MetaInfo {
    /// `announce`: The URL of the tracker.
    ///
    /// `info`: Metadata for the download.
    pub fn new(
        announce: String,
        info: Info,
        announce_list: Option<Vec<Vec<String>>>,
        creation_date: Option<NaiveDateTime>,
    ) -> Self {
        Self {
            announce,
            info,
            announce_list,
            creation_date,
        }
    }

    pub fn announce(&self) -> &str {
        &self.announce
    }

    pub fn info(&self) -> &Info {
        &self.info
    }

    pub fn announce_list(&self) -> Option<&Vec<Vec<String>>> {
        self.announce_list.as_ref()
    }

    pub fn creation_date(&self) -> Option<NaiveDateTime> {
        self.creation_date
    }
}

impl ToBencode for MetaInfo {
    const MAX_DEPTH: usize = 5;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), encoding::Error> {
        encoder.emit_dict(|mut encoder| {
            encoder.emit_pair(b"announce", &self.announce)?;
            if let Some(announce_list) = &self.announce_list {
                encoder.emit_pair(b"announce-list", announce_list)?;
            }
            if let Some(date_time) = &self.creation_date {
                encoder.emit_pair(b"creation date", date_time.timestamp())?;
            }
            encoder.emit_pair(b"info", &self.info)?;
            Ok(())
        })
    }
}

impl FromBencode for MetaInfo {
    const EXPECTED_RECURSION_DEPTH: usize = 5;

    fn decode_bencode_object(object: Object) -> Result<Self, decoding::Error>
    where
        Self: Sized,
    {
        let mut announce = None;
        let mut info = None;
        let mut announce_list = None;
        let mut creation_date = None;
        let mut dict = object.try_into_dictionary()?;

        while let Some(pair) = dict.next_pair()? {
            match pair {
                (b"announce", val) => announce = Some(String::decode_bencode_object(val)?),
                (b"info", val) => info = Some(Info::decode_bencode_object(val)?),
                (b"announce-list", val) => announce_list = Some(Vec::decode_bencode_object(val)?),
                (b"creation date", val) => {
                    let seconds = i64::decode_bencode_object(val)?;
                    creation_date = Some(NaiveDateTime::from_timestamp_opt(seconds, 0).ok_or_else(
                        || {
                            decoding::Error::malformed_content(Error::InvalidMetadata(format!(
                                "invalid creation date timestamp: {}",
                                seconds
                            )))
                        },
                    )?)
                }
                // TODO: Add other metainfo fields
                (b"comment", _) => {}
                (b"created_by", _) => {}
                (b"encoding", _) => {}
                (other, _) => {
                    return Err(decoding::Error::unexpected_field(String::from_utf8_lossy(
                        other,
                    )));
                }
            }
        }

        let announce = announce.ok_or_else(|| decoding::Error::missing_field("announce"))?;
        let info = info.ok_or_else(|| decoding::Error::missing_field("info"))?;

        Ok(Self::new(announce, info, announce_list, creation_date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta_info() -> MetaInfo {
        MetaInfo::new(
            String::from("http://someurl.com"),
            crate::info::tests::info(),
            Some(vec![
                vec![
                    String::from("http://primary.url"),
                    String::from("http://second-primary.url"),
                ],
                vec![String::from("http://backup.url")],
            ]),
            Some(NaiveDateTime::from_timestamp(1234567890, 0)),
        )
    }

    #[test]
    fn encoding_test() {
        assert_eq!(
            "d8:announce18:http://someurl.com13:announce-listll18:http://primary.url25:http://second-primary.urlel17:http://backup.urlee13:creation datei1234567890e4:infod6:lengthi321e4:name9:some name12:piece lengthi1234e6:pieces16:blahblahblahblahee",
            &String::from_utf8_lossy(&meta_info().to_bencode().unwrap())
        );
    }

    #[test]
    fn decoding_test() {
        assert_eq!(
            meta_info(),
            MetaInfo::from_bencode(
                b"d8:announce18:http://someurl.com13:announce-listll18:http://primary.url25:http://second-primary.urlel17:http://backup.urlee13:creation datei1234567890e4:infod6:lengthi321e4:name9:some name12:piece lengthi1234e6:pieces16:blahblahblahblahee",
            ).unwrap()
        );
    }
}
