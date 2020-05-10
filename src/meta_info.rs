use crate::info::Info;
use bendy::{
    decoding::{self, FromBencode, Object},
    encoding::{self, SingleItemEncoder, ToBencode},
};

#[derive(Debug, PartialEq, Eq)]
pub struct MetaInfo {
    announce: String,
    info: Info,
}

impl MetaInfo {
    /// `announce`: The URL of the tracker.
    ///
    /// `info`: Metadata for the download.
    pub fn new(announce: String, info: Info) -> Self {
        Self { announce, info }
    }
}

impl ToBencode for MetaInfo {
    const MAX_DEPTH: usize = 5;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), encoding::Error> {
        encoder.emit_dict(|mut encoder| {
            encoder.emit_pair(b"announce", &self.announce)?;
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
        let mut dict = object.try_into_dictionary()?;

        while let Some(pair) = dict.next_pair()? {
            match pair {
                (b"announce", val) => announce = Some(String::decode_bencode_object(val)?),
                (b"info", val) => info = Some(Info::decode_bencode_object(val)?),
                (other, _) => {
                    return Err(decoding::Error::unexpected_field(String::from_utf8_lossy(
                        other,
                    )));
                }
            }
        }

        let announce = announce.ok_or_else(|| decoding::Error::missing_field("announce"))?;
        let info = info.ok_or_else(|| decoding::Error::missing_field("info"))?;

        Ok(Self::new(announce, info))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta_info() -> MetaInfo {
        MetaInfo::new(
            String::from("http://someurl.com"),
            crate::info::tests::info(),
        )
    }

    #[test]
    fn encoding_test() {
        assert_eq!(
            "d8:announce18:http://someurl.com4:infod6:lengthi321e4:name9:some name12:piece_lengthi1234e6:pieces16:blahblahblahblahee",
            &String::from_utf8_lossy(&meta_info().to_bencode().unwrap())
        );
    }

    #[test]
    fn decoding_test() {
        assert_eq!(
            meta_info(),
            MetaInfo::from_bencode(
            b"d8:announce18:http://someurl.com4:infod6:lengthi321e4:name9:some name12:piece_lengthi1234e6:pieces16:blahblahblahblahee")
                .unwrap()
        );
    }
}
