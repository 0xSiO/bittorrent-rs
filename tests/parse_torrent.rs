use bendy::decoding::FromBencode;
use bittorrent::meta_info::MetaInfo;

#[test]
fn parse_torrent() {
    let file_contents = std::fs::read("tests/fixtures/test.torrent").unwrap();
    let meta_info = MetaInfo::from_bencode(&file_contents).unwrap();
    assert_eq!(
        "http://torrent.fedoraproject.org:6969/announce",
        meta_info.announce()
    );
    assert!(meta_info.announce_list().is_none());
    assert_eq!(1587996219, meta_info.creation_date().unwrap().timestamp());
    assert_eq!(None, meta_info.comment());
    assert_eq!(None, meta_info.created_by());

    let info = meta_info.info();
    assert_eq!("Fedora-SoaS-Live-x86_64-32", info.name());
    assert_eq!(2_u64.pow(18), info.piece_length());
    assert_eq!(84_680, info.pieces().len());
    assert_eq!(None, info.length());
    assert!(info.files().is_some());

    let files = info.files().unwrap();
    assert_eq!(2, files.len());
    assert_eq!(1_109_803_008, files[0].length());
    assert_eq!(
        &[String::from("Fedora-SoaS-Live-x86_64-32-1.6.iso")],
        files[0].path()
    );
    assert_eq!(2032, files[1].length());
    assert_eq!(
        &[String::from("Fedora-Spins-32-1.6-x86_64-CHECKSUM")],
        files[1].path()
    );
}
