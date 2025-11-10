
use reqwest::header::{ HeaderMap, HeaderValue, HeaderName, HOST, REFERER };
use serde::{Deserialize, Serialize};
use visdom::{types::Elements, Vis};
use regex::Regex;
use url::Url;
use urlencoding::{decode};


use crate::get_server::{TrackInfo, GetServerResult, DataResult, SourceInfo, ReturnConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct HlsAndTracks {
    pub hls: String,
    pub tracks: Vec<TrackInfo>,
}

pub fn new(id: &str) -> Result<GetServerResult, Box<dyn std::error::Error>> {

    let client = reqwest::blocking::Client::new();
    
    let url = decode(&id)?.to_string();
    let parsed_url = Url::parse(&url)?;
    let host:String = parsed_url.host().ok_or("host not found")?.to_string();
    let referer: String = format!("https://{}/", &host);
    let origin: String = format!("https://{}", &host);

    let mut headers = HeaderMap::new();
    headers.insert(HOST, HeaderValue::from_str(&host)?);
    headers.insert(REFERER, HeaderValue::from_str(&referer)?);
    headers.insert(HeaderName::from_static("sec-fetch-site"), HeaderValue::from_static("same-origin"));
    

    let res = client.get(url).headers(headers).send()?;

    if res.status().is_success() {
        let html = res.text()?;
        
        let vis = Vis::load(html).map_err(|e| e.to_string())?;
        let hls_and_tracks = extract_hls_and_tracks(&vis)?;

        
        let parse_hls_url = Url::parse(&hls_and_tracks.hls)?;
        let hls_host = parse_hls_url.host().ok_or("hls host not found")?.to_string();


        let new_server_result = GetServerResult {
            data: DataResult { 
                intro: None,
                outro: None,
                sources: vec![
                    SourceInfo {
                        file: hls_and_tracks.hls,
                        _type: String::from("hls"),
                    }
                ],
                tracks: hls_and_tracks.tracks,
            },
            config: ReturnConfig {
                host: "".to_string(),
                referer: referer,
                origin: origin,
                playlist_base_url: format!("https://{}", &hls_host),
                segment_base_url: format!("https://{}", &hls_host),
            }
        };

        return Ok(new_server_result);
    }else{
        return Err(format!("[server_23] Request failed: {}", &res.status()).into());
    }
    
}


pub fn extract_hls_and_tracks(node: &Elements) -> Result<HlsAndTracks, Box<dyn std::error::Error>> {
    let script = node.find("script").text();

    // Regex for HLS URL
    let hls_re = Regex::new(r#"const quality = "(.*?)";"#)?;
    let hls = hls_re
        .captures(&script)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().replace("\\/", "/"))
        .ok_or("HLS URL not found")?;

    // Regex for subtitle tracks
    let track_re = Regex::new(r#"\{\s*"file":\s*"(.*?)",\s*"label":\s*"(.*?)"\s*\}"#)?;
    let mut tracks = Vec::new();

    for cap in track_re.captures_iter(&script) {
        let file = cap[1].replace("\\/", "/");
        let label = Some(cap[2].to_string());

        tracks.push(TrackInfo {
            file,
            label,
            kind: "subtitles".to_string(),
        });
    }

    Ok(HlsAndTracks { hls, tracks })
}


