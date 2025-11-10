use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::panic;
use std::ptr;


use serde::{Deserialize, Serialize};
use serde_json::{from_str};





mod server_2;
mod server_23;

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
pub struct DataResult { 
    pub intro: Option<Timeline>,
    pub outro: Option<Timeline>,
    pub sources: Vec<SourceInfo>,
    pub tracks: Vec<TrackInfo>,
    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReturnConfig {
    pub host: String,
    pub referer: String,
    pub origin: String,
    pub playlist_base_url: String,
    pub segment_base_url: String
}

#[derive(Serialize, Deserialize)]
struct ReturnResult {
    status: bool,
    message: String,
    data: Option<DataResult>,
    config: Option<ReturnConfig>,
}

#[derive(Serialize, Deserialize)]
struct Arguments {
    index: usize,
    id: String
}

#[unsafe(no_mangle)]
pub extern "C" fn get_server(
    arguments_ptr: *const c_char,
) -> *const c_char {
    let result = panic::catch_unwind(|| {
        let mut return_result = ReturnResult {
            status: false,
            message: String::from(""),
            data: None,
            config: None,
        };

        // Check argument before processing
        let mut valid_arguments: bool = true;
        if arguments_ptr.is_null() {
            return_result.message = String::from("Expected 1 argument.");
            valid_arguments = false;
        }
        
        let mut args: Arguments = Arguments { index:0, id: String::from("") };
        if valid_arguments {
            unsafe { 
                match from_str::<Arguments>(&CStr::from_ptr(arguments_ptr as *mut c_char).to_string_lossy().into_owned()) {
                    Ok(result) => {
                        args.id = result.id;
                        args.index = result.index
                    },
                    Err(e) => {
                        return_result.message = String::from(e.to_string());
                        valid_arguments = false;
                    }
                }
            };
        }
        // ================================================

        if valid_arguments {
            let id = args.id;
            let index = args.index;

            let get_server_fn: Option<fn(&str) -> Result<GetServerResult, Box<dyn std::error::Error>>> = match index {
                2 => Some(server_2::new),
                23 => Some(server_23::new),
                _ => None
            };

            if let Some(get_server) = get_server_fn {
                let result = get_server(&id).unwrap();
                
                return_result.data = Some(result.data);
                return_result.config = Some(result.config);
                return_result.status = true;
            }else{
                return_result.message = String::from("[get_server] Unknown server index.");
            }
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


#[derive(Debug, Serialize, Deserialize)]
pub struct GetServerResult{
    pub data: DataResult,
    pub config: ReturnConfig
}