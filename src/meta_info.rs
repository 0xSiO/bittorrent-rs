use crate::info::Info;

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
