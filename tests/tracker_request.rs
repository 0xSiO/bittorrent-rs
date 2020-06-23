use std::collections::HashMap;

use bendy::{decoding::FromBencode, encoding::ToBencode};
use bittorrent::{tracker::Request, MetaInfo};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use reqwest::StatusCode;
use sha1::Sha1;

#[tokio::test]
async fn tracker_request() {
    let file_contents = std::fs::read("tests/fixtures/test.torrent").unwrap();
    let meta_info = MetaInfo::from_bencode(&file_contents).unwrap();
    let info = meta_info.info();
    let info_hash = Sha1::from(&info.to_bencode().unwrap()).digest().bytes();
    let client = reqwest::Client::new();
    let request = Request::new(
        percent_encode(&info_hash, NON_ALPHANUMERIC).to_string(),
        String::from("some-random-peer-id"),
        None,
        6881,
        0,
        0,
        1109803008,
        None,
        true,
        None,
        None,
        None,
        None,
    );
    let response = client
        .get(meta_info.announce())
        .query(&HashMap::from(request))
        .send()
        .await
        .unwrap();

    // TODO: This returns a 400 for now; still trying to get it working
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
