
use reqwest::header::{ HeaderMap, HeaderValue, HOST, REFERER, ORIGIN, USER_AGENT };
use serde::{Deserialize, Serialize};
use visdom::{Vis};
use regex::Regex;
use url::Url;


use crate::get_server::{TrackInfo, GetServerResult, Data, SourceInfo, Config};

#[derive(Debug, Serialize, Deserialize)]
pub struct HlsAndTracks {
    pub hls: String,
    pub tracks: Vec<TrackInfo>,
}

pub fn new(video_id: &str, server_id: &str, token: &str) -> GetServerResult {

    let client = reqwest::blocking::Client::new();

    
    
    /* Generate Cusom Header */
    let mut headers = HeaderMap::new();

    headers.insert(USER_AGENT, HeaderValue::from_static(
        "Chrome/112.0.0.0"
    ));
    headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));
    headers.insert(REFERER, HeaderValue::from_str("https://streamingnow.mov/").unwrap());
    headers.insert(ORIGIN, HeaderValue::from_static("https://streamingnow.mov"));
    headers.insert(HOST, HeaderValue::from_static("streamingnow.mov"));
    
    /* --- */

    let url = format!("https://streamingnow.mov/playvideo.php?video_id={}&server_id={}&token={}=&init=1",
        video_id,
        server_id,
        token
    );

    /* Extract Next URL */

    let res = client.get(url).headers(headers.clone()).send().unwrap();

    if !res.status().is_success() {
        panic!("[server_type_1] Request failed: {}", &res.status());
    }

    let html = res.text().unwrap();

    let vis = Vis::load(&html).unwrap();

    let iframe_ele = vis.find(".source-frame");
    let next_url = iframe_ele.attr("src").unwrap().to_string();

    /* --- */

    /* Extract HLS and Tracks */
    println!("next_url: {}", next_url);
    let res = client.get(next_url).headers(headers).send().unwrap();

    if !res.status().is_success() {
        panic!("[server_type_1] Request failed: {}", &res.status());
    }

    let html = res.text().unwrap();


    let hls_and_tracks = extract_hls_and_tracks(&html);

    let hls_url = hls_and_tracks.hls;

    let parse_url = Url::parse(&hls_url).unwrap();
    let url_host = parse_url.host_str().unwrap();


    println!("host: {}", url_host);

    let data = Data {
        intro: None,
        outro: None,
        tracks: hls_and_tracks.tracks,
        sources: vec![SourceInfo {
            file: hls_url,
            _type: String::from("hls"),
        }]
        
    };

    let config = Config {
        referer: String::from("https://streamingnow.mov/"),
        origin: String::from("https://streamingnow.mov"),
        host: url_host.to_string(),
        playlist_base_url: format!("https://{}", url_host),
        segment_base_url: format!("https://{}", url_host),
    };

    return GetServerResult { data, config };

    /* --- */
    
}


fn extract_hls_and_tracks(html: &str) -> HlsAndTracks {
    // Regex to extract the HLS URL
    let hls_re = Regex::new(r#"file:"(https?://[^"]+\.m3u8)""#).unwrap();
    let hls = hls_re.captures(html).unwrap().get(1).unwrap().as_str().to_string();

    // Regex to extract subtitle block
    let subtitle_re = Regex::new(r#"subtitle:"([^"]+)""#).unwrap();
    let subtitle_block = subtitle_re.captures(html).unwrap().get(1).unwrap().as_str();

    // Parse subtitle entries
    let tracks = subtitle_block
        .split(',')
        .map(|entry| {
            let parts: Vec<&str> = entry.splitn(2, ']').collect();
            let label = parts[0].strip_prefix('[').map(|s| s.to_string());
            let file = parts[1].to_string();
            TrackInfo {
                file,
                label,
                kind: "captions".to_string(),
            }
        })
        .collect();

    HlsAndTracks { hls, tracks }
}