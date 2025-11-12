use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char};
use std::panic;
use std::ptr;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, to_string};
use reqwest::header::{HeaderMap, HeaderValue, REFERER, HOST, USER_AGENT, ORIGIN};
use visdom::Vis;
use urlencoding::{encode, decode};
use regex::Regex;

use html_escape::decode_html_entities;
use rand::rng;


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

const SUPPORT_SERVER_INDEX: [usize; 3]  = [89, 90, 88];

const DUMMY_VERIFY_TOKENS: [&str; 5] = [
    "ZEhKMVpTLVF0LVBTLVF0LVB6QS1RTFMtUXpPLTBJdEwtMC1WM05qSTROamMtUU9Ea3otUC0wYy01",
    "ZEhKMVpTLVF0LVBTLVF0TkRrMkxTLVF6Ty0wLVB0TC0wLVYzTmpJNE5qWTUtUERBMy1QalUtNQ==",
    "ZEhKMVpTLVF0LVBTLVF0Ti0wSXdMUy1Rek56LVZ0TC0wLVYzTmpJNE5qSTMtUHprMS1QRGstNQ==",
    "ZEhKMVpTLVF0LVBTLVF0TnpVeUxTLVF6Ty0wLVZ0TC0wLVYzTmpJNE56LVAtUS1QREkzTnpJLTU",
    "ZEhKMVpTLVF0LVBTLVF0TnpZeExTLVF6T0RndEwtMC1WM05qSTROei1QMU56LVAyLVAtMEktNQ==",
];

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


        let server_data = extract_server_data(&token);
        

        return_result.status = true;
        return_result.message = String::from("Success");
        return_result.data.insert(String::from("SERVER"), server_data);
        
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

fn extract_server_data(token: &str) -> Vec<EpisodeServerData> {
    
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

    let mut new_server_data: Vec<EpisodeServerData> = Vec::new();

    for source in source_list_ele{
        let source_ele = Vis::dom(&source);

        let raw_server_id = source_ele.attr("data-server").unwrap().to_string();
        let server_id = raw_server_id.parse::<usize>().unwrap();

        if !SUPPORT_SERVER_INDEX.contains(&server_id){
            continue;
        }

        let raw_title = source_ele.find(".server-name").text();
        let title = decode_html_entities(raw_title.trim()).to_string();

        let video_id = source_ele.attr("data-id").unwrap().to_string();

        let raw_id = json!({
            "video_id": &video_id,
            "server_id": &server_id,
            "token": &second_token
        });

        

        new_server_data.push(EpisodeServerData {
            index: server_id,
            id: encode(&to_string(&raw_id).unwrap()).to_string(),
            title,
        });
    }

    return new_server_data;


}