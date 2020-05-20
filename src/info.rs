use crate::{error::*, file_info::FileInfo};

use bendy::{
    decoding::{self, FromBencode, Object},
    encoding::{self, SingleItemEncoder, ToBencode},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Info {
    name: String,
    piece_length: u64,
    pieces: Vec<u8>,
    length: Option<u64>,
    files: Option<Vec<FileInfo>>,
}

impl Info {
    /// `name`: The suggested name to save the file (or directory) as. It is
    /// purely advisory.
    ///
    /// `piece_length`: The number of bytes in each piece the file is split
    /// into. For the purposes of transfer, files are split into fixed-size
    /// pieces which are all the same length except for possibly the last one,
    /// which may be truncated. piece length is almost always a power of two,
    /// most commonly 2^18 = 256 KB.
    ///
    /// `pieces`: A byte string whose length is a multiple of 20. It is to be
    /// subdivided into strings of length 20, each of which is the SHA1 hash of
    /// the piece at the corresponding index.
    ///
    /// `length`: If present, the download represents a single file, and this
    /// parameter maps to the length of the file in bytes. If not present, the
    /// download represents a set of files which go in a directory structure.
    ///
    /// `files`: If present, contains the information of all files for the
    /// download.
    pub fn new(
        name: String,
        piece_length: u64,
        pieces: Vec<u8>,
        length: Option<u64>,
        files: Option<Vec<FileInfo>>,
    ) -> Result<Self> {
        if length.is_some() && files.is_some() {
            Err(Error::InvalidMetadata(String::from(
                "'length' and 'files' cannot both be defined in info dictionary",
            )))
        } else if length.is_none() && files.is_none() {
            Err(Error::InvalidMetadata(String::from(
                "one of 'length' or 'files' must be defined in info dictionary",
            )))
        } else {
            Ok(Self {
                name,
                piece_length,
                pieces,
                length,
                files,
            })
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn piece_length(&self) -> u64 {
        self.piece_length
    }

    pub fn pieces(&self) -> &[u8] {
        &self.pieces
    }

    pub fn length(&self) -> Option<u64> {
        self.length
    }

    pub fn files(&self) -> Option<&[FileInfo]> {
        self.files.as_deref()
    }
}

impl ToBencode for Info {
    const MAX_DEPTH: usize = 4;

    fn encode(&self, encoder: SingleItemEncoder) -> std::result::Result<(), encoding::Error> {
        encoder.emit_dict(|mut encoder| {
            if let Some(files) = &self.files {
                encoder.emit_pair(b"files", files)?;
            }
            if let Some(length) = self.length {
                encoder.emit_pair(b"length", length)?;
            }
            encoder.emit_pair(b"name", &self.name)?;
            encoder.emit_pair(b"piece length", self.piece_length)?;
            encoder.emit_pair_with(b"pieces", |encoder| {
                encoder.emit_bytes(&self.pieces)?;
                Ok(())
            })?;
            Ok(())
        })
    }
}

impl FromBencode for Info {
    const EXPECTED_RECURSION_DEPTH: usize = 4;

    fn decode_bencode_object(object: Object) -> std::result::Result<Self, decoding::Error>
    where
        Self: Sized,
    {
        let mut name = None;
        let mut piece_length = None;
        let mut pieces = None;
        let mut length = None;
        let mut files = None;
        let mut dict = object.try_into_dictionary()?;

        while let Some(pair) = dict.next_pair()? {
            match pair {
                (b"name", val) => name = Some(String::decode_bencode_object(val)?),
                (b"piece length", val) => piece_length = Some(u64::decode_bencode_object(val)?),
                (b"pieces", val) => pieces = Some(val.try_into_bytes()?.to_vec()),
                (b"length", val) => length = Some(u64::decode_bencode_object(val)?),
                (b"files", val) => files = Some(Vec::decode_bencode_object(val)?),
                // TODO: Add other info fields
                (b"private", _) => {}
                (b"md5sum", _) => {}
                (other, _) => {
                    return Err(decoding::Error::unexpected_field(String::from_utf8_lossy(
                        other,
                    )));
                }
            }
        }

        let name = name.ok_or_else(|| decoding::Error::missing_field("name"))?;
        let piece_length =
            piece_length.ok_or_else(|| decoding::Error::missing_field("piece length"))?;
        let pieces = pieces.ok_or_else(|| decoding::Error::missing_field("pieces"))?;

        Ok(Self::new(name, piece_length, pieces, length, files)
            .map_err(|err| decoding::Error::malformed_content(err))?)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn info() -> Info {
        Info::new(
            String::from("some name"),
            1234,
            b"blahblahblahblah".to_vec(),
            Some(321),
            None,
        )
        .unwrap()
    }

    #[test]
    fn encoding_test() {
        assert_eq!(
            "d6:lengthi321e4:name9:some name12:piece lengthi1234e6:pieces16:blahblahblahblahe",
            &String::from_utf8_lossy(&info().to_bencode().unwrap())
        );
    }

    #[test]
    fn decoding_test() {
        assert_eq!(
            info(),
            Info::from_bencode(
                b"d6:lengthi321e4:name9:some name12:piece lengthi1234e6:pieces16:blahblahblahblahe"
            )
            .unwrap()
        );
    }
}
