pub struct FileInfo {
    length: u64,
    path: Vec<String>,
}

impl FileInfo {
    /// `length`: The length of the file, in bytes.
    ///
    /// `path`: A `Vec` of UTF-8 encoded strings corresponding to subdirectory
    /// names, the last of which is the actual file name (a zero length list
    /// is an error case).
    pub fn new(length: u64, path: Vec<String>) -> Self {
        Self { length, path }
    }
}
