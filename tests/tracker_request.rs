use std::collections::HashMap;

use bendy::{decoding::FromBencode, encoding::ToBencode};
use bittorrent::{
    tracker::{Event, Request},
    MetaInfo,
};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use reqwest::{StatusCode, Url};
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
        percent_encode(b"abcdefghijklmnopqrst", NON_ALPHANUMERIC).to_string(),
        None,
        6881,
        0,
        0,
        info.files().unwrap()[0].length(),
        Some(Event::Started),
        true,
        None,
        None,
        None,
        None,
    );

    let mut params = HashMap::from(request);
    let info_hash = params.remove("info_hash").unwrap();
    let mut url = Url::parse_with_params(meta_info.announce(), params).unwrap();
    // The hash is already percent encoded, and we don't want it to be encoded again, so we
    // need to add it to the URL directly.
    url.set_query(Some(&format!(
        "info_hash={}&{}",
        info_hash,
        url.query().unwrap()
    )));
    let response = client.get(url).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
