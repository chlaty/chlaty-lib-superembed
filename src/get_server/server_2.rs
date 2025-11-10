
use serde::{Deserialize, Serialize};
use visdom::Vis;
use regex::Regex;
use url::Url;
use urlencoding::{decode};


use crate::get_server::{TrackInfo, GetServerResult, DataResult, SourceInfo, ReturnConfig};

pub fn new(id: &str) -> Result<GetServerResult, Box<dyn std::error::Error>> {

    let client = reqwest::blocking::Client::new();
    
    let url = decode(&id)?.to_string();

    let res = client.get(url).send()?;

    if res.status().is_success() {
        let html = res.text().unwrap();
        let vis = Vis::load(html).map_err(|e| e.to_string())?;
        let iframe_node = vis.find("#player_iframe");
        let raw_src = iframe_node.attr("src").ok_or("rcp url not found")?.to_string();

        let forward_rcp_url = format!("{}{}", "https:", raw_src);
        
        let forward_prorcp_url = get_prorcp(&forward_rcp_url)?;
        
        
        let prorcp_hls_and_tracks = get_prorcp_hls_and_tracks(&forward_prorcp_url)?;
        let parse_prorcp_hls_url = Url::parse(&prorcp_hls_and_tracks.hls)?;
        let prorcp_hls_host = parse_prorcp_hls_url.host().ok_or("prorcp hls host not found")?;

        let result = GetServerResult {
            data: DataResult {
                intro: None,
                outro: None,
                tracks: prorcp_hls_and_tracks.tracks,
                sources: vec![SourceInfo {
                    file: prorcp_hls_and_tracks.hls,
                    _type: "hls".to_string(),
                }]
            },
            config: ReturnConfig { 
                host: prorcp_hls_host.to_string(),
                referer: "".to_string(),
                origin: "".to_string(),
                playlist_base_url: format!("https://{}", prorcp_hls_host),
                segment_base_url: "".to_string()
            }
        };

        return Ok(result);

    }
    


    return Err("[server_2] Request failed".into());
}


fn get_prorcp(rcp_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    
    let client = reqwest::blocking::Client::new();
    
    let res = client.get(rcp_url).send()?;

    if !res.status().is_success() {
        return Err("[get_prorcp] Request failed".into());
    }

    let html = res.text().unwrap();
    let vis = Vis::load(html).map_err(|e| e.to_string())?;

    let script_nodes = vis.find("script");
    for script in script_nodes{
        let script_node = Vis::dom(&script);
        let script_text = script_node.text();

        if let Some(start) = script_text.find("src: '") {
            let rest = &script_text[start + 6..];
            if let Some(end) = rest.find('\'') {
                let src = &rest[..end];
                if src.starts_with("/prorcp/") {
                    
                    let parse_rcp_url = Url::parse(rcp_url)?;
                    let rcp_host = parse_rcp_url.host().ok_or("rcp host not found")?;

                    let prorcp_url = format!("https://{}{}", rcp_host, src);
                    return Ok(prorcp_url);
                }
            }
        }
    }

    Err("[get_prorcp] prorcp url not found".into())
}



#[derive(Debug, Serialize, Deserialize)]
pub struct HlsAndTracks {
    pub hls: String,
    pub tracks: Vec<TrackInfo>
}

fn get_prorcp_hls_and_tracks(prorcp_url: &str) -> Result<HlsAndTracks, Box<dyn std::error::Error>> {
    let parse_prorcp_url = Url::parse(prorcp_url)?;
    let prorcp_host = parse_prorcp_url.host().ok_or("prorcp host not found")?;

    let client = reqwest::blocking::Client::new();
    
    let res = client.get(prorcp_url).send()?;

    if !res.status().is_success() {
        return Err("[get_prorcp] Request failed".into());
    }

    let html = res.text().unwrap();
    let vis = Vis::load(html).map_err(|e| e.to_string())?;
    let mut hls_url = "".to_string();

    let scripts = vis.find("script");
    let re = Regex::new(r#"file:\s*['"]([^'"]+?\.m3u8)['"]"#)?;
    for script in scripts {
        let script_text = script.text();
        if let Some(caps) = re.captures(&script_text) {
            if let Some(url) = caps.get(1) {
                hls_url = url.as_str().to_string();
            }
        }
    }
    if hls_url.is_empty() {
        return Err("[get_prorcp_hls] hls url not found".into());
    }

    /* Query to find tracks */
    let mut tracks = Vec::new();

    // Regex to extract the default_subtitles assignment
    let re = Regex::new(r#"default_subtitles\s*=\s*"([^"]+)""#).unwrap();

    for script in vis.find("script") {
        let content = script.text();

        if let Some(captures) = re.captures(&content) {
            if let Some(subtitles_str) = captures.get(1) {
                for entry in subtitles_str.as_str().split(',') {
                    if let Some((label, path)) = entry.split_once("]/") {
                        tracks.push(TrackInfo {
                            file: format!("https://{}/{}", prorcp_host, path),
                            label: Some(label.trim_start_matches('[').to_string()),
                            kind: "captions".to_string(),
                        });
                    }
                }
            }
        }
    }
    /* --- */

    return Ok(HlsAndTracks {
        hls: hls_url,
        tracks
    });

}