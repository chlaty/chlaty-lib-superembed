
use reqwest::header::{ HeaderMap, HeaderValue, HOST, REFERER, ORIGIN, USER_AGENT };
use serde::{Deserialize, Serialize};
use visdom::{Vis};
use regex::Regex;
use url::Url;
use serde_json::{from_str, json};
use urlencoding::{decode};
use rand::rng;
use rand::prelude::SliceRandom;
use std::collections::HashMap;


use crate::get_server::{TrackInfo, GetServerResult, Data, SourceInfo, Config};
use crate::get_episode_server::extract_token;
use crate::DUMMY_VERIFY_TOKENS;

#[derive(Debug, Serialize, Deserialize)]
pub struct HlsAndTracks {
    pub hls: String,
    pub tracks: Vec<TrackInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FormatServerArguments {
    imdb_id: String,
    s: Option<usize>,
    e: Option<usize>,
    server_id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractServerData {
    video_id: String,
    server_id: String,
    token: String,
}




pub fn new(id: &str) -> GetServerResult {

    let format_args: FormatServerArguments = from_str(&decode(&id).unwrap()).unwrap();
        
    

    let imdb_id = format_args.imdb_id;
    let server_id = format_args.server_id;
    let season_index = format_args.s;
    let episode_index = format_args.e;
    
    let extract_token_url: String;

    if season_index.is_some() && episode_index.is_some() {
        extract_token_url = format!("https://multiembed.mov/directstream.php?video_id={}&s={}&e={}", 
            imdb_id,
            season_index.unwrap()+1,
            episode_index.unwrap()+1
        );
    }else{
        extract_token_url = format!("https://multiembed.mov/directstream.php?video_id={}",imdb_id);
        
    }


    let token = extract_token(&extract_token_url);

    let extract_server_data = extract_server_data(&server_id, &token);
    

    let video_id = extract_server_data.video_id;
    let server_id = extract_server_data.server_id;
    let token = extract_server_data.token;

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
    ); // this url can be use to verify robot

    

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
    
    let res = client.get(next_url).headers(headers).send().unwrap();

    if !res.status().is_success() {
        panic!("[server_type_1] Request failed: {}", &res.status());
    }

    let html = res.text().unwrap();


    let hls_and_tracks = extract_hls_and_tracks(&html);

    let hls_url = hls_and_tracks.hls;

    let parse_url = Url::parse(&hls_url).unwrap();
    let url_host = parse_url.host_str().unwrap();


    

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


fn extract_server_data(server_id: &str, token: &str) -> ExtractServerData {
    
    let client = reqwest::blocking::Client::new();

    /* Generate Custom Header */
    let mut headers = HeaderMap::new();

    headers.insert(USER_AGENT, HeaderValue::from_static(
        "Chrome/112.0.0.0"
    ));
    headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));
    headers.insert(REFERER, HeaderValue::from_str("https://streamingnow.mov/").unwrap());
    headers.insert(ORIGIN, HeaderValue::from_static("https://streamingnow.mov"));
    headers.insert(HOST, HeaderValue::from_static("streamingnow.mov"));
    
    /* --- */

    /* Extract Second Token */

    let mut dummy_tokens = DUMMY_VERIFY_TOKENS.to_vec();
    dummy_tokens.shuffle(&mut rng());

    
    let mut second_token: String = String::new();

    for dummy_token in dummy_tokens {

        let url = format!("https://streamingnow.mov/?play={}", token);

        let res = client.post(url)
            .form(&json!({
                "button-click": dummy_token
            }))
            .send().unwrap();
            
        if !res.status().is_success(){
            eprintln!("[get_episode_server] Extract Dummy Token Error: {}", res.status());
            continue;
        }

        let html =  res.text().unwrap();
        let re = Regex::new(r#"load_sources\("([A-Za-z0-9+/=]+)"\)"#).unwrap();
        second_token = re.captures(&html).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string())).expect("unable to find second token.");
        
        break;
    
    }
    
    /* --- */
    
    let mut form:HashMap<&str, &str> = HashMap::new();
    
    form.insert("token", token);

    let res = client.post("https://streamingnow.mov/response.php")
        .headers(headers)
        .form(&form)
        .send().unwrap();
        
    if !res.status().is_success(){
        panic!("[get_episode_server] Extract Server Data Error: {}", res.status());
    }

    let html =  res.text().unwrap();

    let vis = Vis::load(html).unwrap();

    let source_list_ele = vis.find(".sources-list").find("li");

    for source in source_list_ele{
        let source_ele = Vis::dom(&source);

        let extract_server_id = source_ele.attr("data-server").unwrap().to_string();

        if extract_server_id != server_id {
            continue;
        }

        let video_id = source_ele.attr("data-id").unwrap().to_string();

        let new_server_data = ExtractServerData{
            video_id, 
            server_id: extract_server_id, 
            token: second_token
        };

        return new_server_data;
        
    }

    panic!("[get_episode_server] Extract Server Data Error: Server not found.");

}