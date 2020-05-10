use crate::file_info::FileInfo;

pub struct Info {
    name: String,
    piece_length: u64,
    pieces: String,
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
    /// `pieces`: A string whose length is a multiple of 20. It is to be
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
        pieces: String,
        length: Option<u64>,
        files: Option<Vec<FileInfo>>,
    ) -> Self {
        Self {
            name,
            piece_length,
            pieces,
            length,
            files,
        }
    }
}

