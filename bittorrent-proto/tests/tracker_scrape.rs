use bendy::{decoding::FromBencode, encoding::ToBencode};
use bittorrent_proto::{tracker::scrape::Response, MetaInfo};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use reqwest::{StatusCode, Url};
use sha1::Sha1;

#[tokio::test]
async fn tracker_scrape() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();
    let file_contents = std::fs::read("tests/fixtures/test.torrent").unwrap();
    let meta_info = MetaInfo::from_bencode(&file_contents).unwrap();
    let info = meta_info.info();
    let info_hash = Sha1::from(&info.to_bencode().unwrap()).digest();
    assert_eq!(
        &info_hash.to_string(),
        "80bbb5c4986d3dd4c52f8dab517451203c4fab1d"
    );
    let mut announce_url = Url::parse(meta_info.announce()).unwrap();
    announce_url
        .path_segments_mut()
        .unwrap()
        .pop_if_empty()
        .pop()
        .push("scrape");
    let mut scrape_url = announce_url;
    scrape_url.set_query(Some(&format!(
        "info_hash={}",
        percent_encode(&info_hash.bytes(), NON_ALPHANUMERIC).to_string()
    )));

    let response = reqwest::get(scrape_url).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = Response::from_bencode(&response.bytes().await.unwrap()).unwrap();
    assert_eq!(response.files().len(), 1);
    assert!(response.files().get(&info_hash).is_some());
}
