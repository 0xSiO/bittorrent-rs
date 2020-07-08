use bendy::{decoding::FromBencode, encoding::ToBencode};
use bittorrent_proto::{
    tracker::announce::{Event, Request, Response},
    MetaInfo,
};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use reqwest::{StatusCode, Url};
use sha1::Sha1;

#[tokio::test]
async fn tracker_announce() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    let file_contents = std::fs::read("tests/fixtures/test.torrent").unwrap();
    let meta_info = MetaInfo::from_bencode(&file_contents).unwrap();
    let info = meta_info.info();
    let info_hash = Sha1::from(&info.to_bencode().unwrap()).digest();
    assert_eq!(
        &info_hash.to_string(),
        "80bbb5c4986d3dd4c52f8dab517451203c4fab1d"
    );
    let request = Request::new(
        Url::parse(meta_info.announce()).unwrap(),
        percent_encode(&info_hash.bytes(), NON_ALPHANUMERIC).to_string(),
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
    let response = reqwest::get(Url::from(request)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = Response::from_bencode(&response.bytes().await.unwrap());
    assert!(response.is_ok());
}
