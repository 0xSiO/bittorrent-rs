use bendy::{
    decoding::{self, FromBencode, Object},
    encoding::{self, SingleItemEncoder, ToBencode},
};

#[derive(Debug, PartialEq, Eq)]
pub struct FileInfo {
    length: u64,
    path: Vec<String>,
    md5sum: Option<String>,
}

impl FileInfo {
    /// `length`: The length of the file, in bytes.
    ///
    /// `path`: A `Vec` of UTF-8 encoded strings corresponding to subdirectory
    /// names, the last of which is the actual file name (a zero length list
    /// is an error case).
    pub fn new(length: u64, path: Vec<String>, md5sum: Option<String>) -> Self {
        Self {
            length,
            path,
            md5sum,
        }
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn path(&self) -> &[String] {
        &self.path
    }

    pub fn md5sum(&self) -> Option<&str> {
        self.md5sum.as_deref()
    }
}

impl ToBencode for FileInfo {
    const MAX_DEPTH: usize = 2;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), encoding::Error> {
        encoder.emit_dict(|mut encoder| {
            encoder.emit_pair(b"length", self.length())?;
            if let Some(md5sum) = self.md5sum() {
                encoder.emit_pair(b"md5sum", md5sum)?;
            }
            encoder.emit_pair(b"path", self.path())?;
            Ok(())
        })
    }
}

impl FromBencode for FileInfo {
    const EXPECTED_RECURSION_DEPTH: usize = 2;

    fn decode_bencode_object(object: Object) -> Result<Self, decoding::Error>
    where
        Self: Sized,
    {
        let mut length = None;
        let mut path = None;
        let mut md5sum = None;
        let mut dict = object.try_into_dictionary()?;

        while let Some(pair) = dict.next_pair()? {
            match pair {
                (b"length", val) => length = Some(u64::decode_bencode_object(val)?),
                (b"path", val) => path = Some(Vec::decode_bencode_object(val)?),
                (b"md5sum", val) => md5sum = Some(String::decode_bencode_object(val)?),
                (other, _) => {
                    return Err(decoding::Error::unexpected_field(String::from_utf8_lossy(
                        other,
                    )));
                }
            }
        }

        let length = length.ok_or_else(|| decoding::Error::missing_field("length"))?;
        let path = path.ok_or_else(|| decoding::Error::missing_field("path"))?;

        Ok(Self::new(length, path, md5sum))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file_info() -> FileInfo {
        FileInfo::new(
            123456,
            vec![
                String::from("testing"),
                String::from("another"),
                String::from("final.txt"),
            ],
            None,
        )
    }

    #[test]
    fn encoding_test() {
        assert_eq!(
            "d6:lengthi123456e4:pathl7:testing7:another9:final.txtee",
            &String::from_utf8_lossy(&file_info().to_bencode().unwrap())
        );
    }

    #[test]
    fn decoding_test() {
        assert_eq!(
            file_info(),
            FileInfo::from_bencode(b"d6:lengthi123456e4:pathl7:testing7:another9:final.txtee")
                .unwrap()
        );
        // missing 'length' field
        assert!(FileInfo::from_bencode(b"d4:pathl7:testing7:another9:final.txtee").is_err());
    }
}
