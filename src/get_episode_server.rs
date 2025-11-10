use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char};
use std::panic;
use std::ptr;

use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, to_string};
use reqwest::header::{HeaderMap, HeaderValue, REFERER, HOST, USER_AGENT, ORIGIN, CONTENT_TYPE};
use visdom::Vis;
use visdom::types::Elements;
use urlencoding::{encode, decode};
use regex::Regex;
use url::Url;
use html_escape::decode_html_entities;

use crate::{ SOURCE_HOST };

#[derive(Debug, Serialize, Deserialize)]
struct EpisodeInfo{
    imdb_id: String,
    s: Option<usize>,
    e: Option<usize>
}


#[derive(Debug, Serialize, Deserialize)]
struct EpisodeServerData{
    index: usize,
    id: String,
    title: String
}

#[derive(Debug, Serialize, Deserialize)]
struct ReturnResult {
    status: bool,
    message: String,
    data: HashMap<String, Vec<EpisodeServerData>>,
}

#[derive(Serialize, Deserialize)]
struct Arguments {
    episode_id: String
}

const SUPPORT_SERVER_INDEX: [usize; 1]  = [23];

#[unsafe(no_mangle)]
pub extern "C" fn get_episode_server(
    arguments_ptr : *const c_char,
) -> *const c_char {
    let result = panic::catch_unwind(|| {
        let mut return_result = ReturnResult {
            status: false,
            message: String::from(""),
            data: HashMap::new(),
            
        };

        // Check argument before processing
        if arguments_ptr.is_null() {
            panic!("Expected 1 argument.");
        }

        let args: Arguments = unsafe { 
            from_str(&CStr::from_ptr(arguments_ptr as *mut c_char).to_string_lossy().into_owned()).unwrap()
        };
        
        // ================================================

        

        let raw_episode_id = args.episode_id;

        let episode_info: EpisodeInfo = from_str(&decode(&raw_episode_id).unwrap()).unwrap();

        

        println!("{:?}", episode_info);
        

        let extract_token_url: String;
        
        if episode_info.s.is_some() && episode_info.e.is_some() {
            extract_token_url = format!("https://multiembed.mov/directstream.php?video_id={}&s={}&e={}", 
                episode_info.imdb_id,
                episode_info.s.unwrap(),
                episode_info.e.unwrap()
            );
        }else{
            extract_token_url = format!("https://multiembed.mov/directstream.php?video_id={}", episode_info.imdb_id);
        }
        

        let token = extract_token(&extract_token_url);

        println!("{:?}", token);

        let server_data = extract_server_data(&token);



        
        
        return serde_json::to_string(&return_result).unwrap();
    });
    
    match result {
        Ok(data) => {
            let result = CString::new(data).unwrap();
            return result.into_raw();
        },
        _ => ptr::null(),
    }
}




fn extract_token(url: &str) -> String {

    let client = reqwest::blocking::Client::new();

    let res = client.get(url).send().unwrap();
        
    if !res.status().is_success(){
        panic!("[get_episode_server] Extract Token Info Error: {}", res.status());
    }

    let redirect_url = res.url();

    let token = redirect_url.query_pairs().find(|(key, _)| key == "play").unwrap().1.to_string();

    return token;
}

fn extract_server_data(token: &str) {
    
    let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();

    headers.insert(USER_AGENT, HeaderValue::from_static(
        "Chrome/112.0.0.0"
    ));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));
    headers.insert(REFERER, HeaderValue::from_static("https://streamingnow.mov/"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://streamingnow.mov"));
    headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));
    headers.insert("Accept", HeaderValue::from_static("*/*"));
    headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));

    let res = client.post("https://streamingnow.mov/response.php")
        .headers(headers)
        .body(format!("token={}", token))
        .send().unwrap();
        
    if !res.status().is_success(){
        panic!("[get_episode_server] Extract Server Data Error: {}", res.status());
    }

    println!("{}", res.text().unwrap());
}