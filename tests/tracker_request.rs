use std::collections::HashMap;

use bendy::{decoding::FromBencode, encoding::ToBencode};
use bittorrent::{
    tracker::{Event, Request},
    MetaInfo,
};
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

    // This is a hack. reqwest always percent-encodes query parameters, so our percent-encoded
    // info_hash will be encoded a second time, which is bad. I couldn't find a way to just put
    // encoded bytes in the query parameters, so I modify the URL directly.
    let mut params = HashMap::from(request);
    let hash = params.remove("info_hash").unwrap();
    let mut http_req = client
        .get(meta_info.announce())
        .query(&params)
        .build()
        .unwrap();
    let url = http_req.url_mut();
    url.set_query(Some(&format!(
        "info_hash={}&{}",
        hash,
        url.query().unwrap()
    )));
    let response = client.execute(http_req).await.unwrap();

    // TODO: This returns a 400 for now; still trying to get it working
    assert_eq!(response.status(), StatusCode::OK);
}
