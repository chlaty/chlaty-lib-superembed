use std::ffi::{CString, CStr};
use std::os::raw::{c_char};
use std::panic;
use std::ptr;

use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string, json};
use reqwest::header::{HeaderMap, USER_AGENT, HeaderValue, REFERER};
use visdom::Vis;
use urlencoding::{encode, decode};
use html_escape::decode_html_entities;

use crate::{ SOURCE_HOST, SOURCE_REFERER, SOURCE_ORIGIN };




#[derive(Debug, Serialize, Deserialize)]
struct EpisodeData{
    index: usize,
    id: String,
    title: String
}

#[derive(Debug, Serialize, Deserialize)]
struct ReturnResult {
    status: bool,
    message: String,
    data: Vec<Vec<Vec<EpisodeData>>>,
}

#[derive(Serialize, Deserialize)]
struct Arguments {
    id: String
}

#[unsafe(no_mangle)]
pub extern "C" fn get_episode_list(
    arguments_ptr : *const c_char,
) -> *const c_char {
    let result = panic::catch_unwind(|| {

        let mut return_result = ReturnResult {
            status: false,
            message: String::from(""),
            data: Vec::new(),
        };

        // Check argument before processing
        if arguments_ptr.is_null() {
            panic!("Expected 1 argument.");
        }

        let args: Arguments = unsafe { 
            from_str(&CStr::from_ptr(arguments_ptr as *mut c_char).to_string_lossy().into_owned()).unwrap()
        };
        
        // ================================================


        let format_id = decode(&args.id).unwrap().to_string();
        let id_type = format_id.split("/").nth(1).unwrap();
        

        let client = reqwest::blocking::Client::new();
        let mut headers = HeaderMap::new();
        
        headers.insert(USER_AGENT, HeaderValue::from_static(
            "Chrome/126.0.0.0"
        ));
        
        let url = format!("https://{}{}/episodes", 
                SOURCE_HOST, format_id
            );

        

        let res = client.get(&url).headers(headers).send().unwrap();
        
        if !res.status().is_success(){
            panic!("Error: {}", res.status());
        }

        let html = res.text().unwrap();

        let vis = Vis::load(html).unwrap();

        /* Find IMDB ID */
        let imdb_ele = vis.find(".imdb-rating").find("a[rel=\"nofollow\"]");
        let imdb_url = imdb_ele.attr("href").unwrap().to_string();
        let imdb_id = imdb_url.split("/").nth(imdb_url.split("/").count() - 2).unwrap();
        /* --- */
        


        if id_type == "tv" {
            let ep_tab_ele = vis.find("#InfoTabsEpisodes");

            if ep_tab_ele.length() == 0 {
                return_result.data = Vec::new();
                
                return_result.message = String::from("Episode list not found.");
            }else{
                
                let details_ele = vis.find(".SimklTVEpisodesBlock").find(".SimklTVAboutTabsDetails");
                
                if details_ele.length() > 0 {
                    
                    let tr_ele = vis.find("tr");
                    
                    let mut season_index = 0;
                    for tr in tr_ele {
                        let tr_ele = Vis::dom(&tr);

                        let is_not_ep_ele = tr_ele.find(".SimklTVAboutTabsDetailsSeasonHead");
                        if is_not_ep_ele.length() > 0 {
                            continue;
                        }

                        let ep_ele = tr_ele.find("td").find("div.goEpisode");
                        if ep_ele.length() > 0 {
                            let mut new_ep_data: Vec<EpisodeData> = Vec::new();

                            let mut episode_index = 0;
                            for ep in ep_ele.into_iter() {
                                let ep_ele = Vis::dom(&ep);
                                
                                let raw_title = ep_ele.find(".SimklTVEpisodesEpTitle").text();
                                let title = decode_html_entities(&raw_title).to_string();
                                let id = encode(&to_string(&json!({
                                        "type": id_type,
                                        "imdb_id": imdb_id,
                                        "s": season_index,
                                        "e": episode_index,
                                    })).unwrap()).to_string();
                                new_ep_data.push(EpisodeData {
                                    index: episode_index,
                                    id: id,
                                    title: title,
                                });

                                episode_index += 1;
                            }

                            season_index += 1;
                            return_result.data.push(vec![new_ep_data]);
                        }
                        
                    }
                }

            }
        }else{
            return_result.data = vec![vec![vec![EpisodeData { 
                index: 0, 
                id: encode(&to_string(&json!({
                    "type": id_type,
                    "imdb_id": imdb_id,
                })).unwrap()).to_string(), 
                title: String::from("Full") 
            }]]];
            return_result.message = String::from("Episode list not found.");
        }
        
        
        return_result.status = true;
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