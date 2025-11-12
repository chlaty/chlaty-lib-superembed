use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::panic;
use std::ptr;


use serde::{Deserialize, Serialize};
use serde_json::{from_str};
use urlencoding::{decode};




mod server_type_1;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Timeline {
    pub start: usize,
    pub end: usize
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceInfo {
    pub file: String,

    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackInfo {
    pub file: String,
    pub label: Option<String>,
    pub kind: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data { 
    pub intro: Option<Timeline>,
    pub outro: Option<Timeline>,
    pub sources: Vec<SourceInfo>,
    pub tracks: Vec<TrackInfo>,
    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub referer: String,
    pub origin: String,
    pub playlist_base_url: String,
    pub segment_base_url: String
}

#[derive(Serialize, Deserialize)]
struct ServerInfo {
    status: bool,
    message: String,
    data: Option<Data>,
    config: Option<Config>,
}

#[derive(Serialize, Deserialize)]
struct Arguments {
    id: String,
    index: usize
}

#[derive(Debug, Serialize, Deserialize)]
struct FormatServerArguments {
    video_id: String,
    server_id: usize,
    token: String
}



#[derive(Debug, Serialize, Deserialize)]
pub struct GetServerResult{
    pub data: Data,
    pub config: Config
}

const SERVER_TYPES_1: [usize; 3]  = [89, 90, 88];

#[unsafe(no_mangle)]
pub extern "C" fn get_server(
    arguments_ptr: *const c_char,
) -> *const c_char {
    let result = panic::catch_unwind(|| {
        let mut return_result = ServerInfo {
            status: false,
            message: String::from(""),
            data: None,
            config: None,
        };

        // Check argument before processing
        if arguments_ptr.is_null() {
            panic!("Expected 1 argument.");
        }

        let args: Arguments = unsafe { 
            from_str(&CStr::from_ptr(arguments_ptr as *mut c_char).to_string_lossy().into_owned()).unwrap()
        };
        
        // ================================================

        
        let raw_id = args.id;
        let index = args.index;
        

        let format_args: FormatServerArguments = from_str(&decode(&raw_id).unwrap()).unwrap();
        

        let video_id = format_args.video_id;
        let server_id = format_args.server_id;
        let token = format_args.token;


        

        let get_server_fn: Option<fn(&str, &str, &str) -> GetServerResult> = 
            if SERVER_TYPES_1.contains(&index) {
                Some(server_type_1::new)
            } else {
                None
            };

        if let Some(get_server) = get_server_fn {
            let result = get_server(&video_id, &server_id.to_string(), &token);
            
            return_result.data = Some(result.data);
            return_result.config = Some(result.config);
            return_result.status = true;
        }else{
            return_result.message = String::from("[get_server] Unknown server index.");
        }
        
        
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


