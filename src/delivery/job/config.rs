use rustc_serialize::json::{Json};
use errors::{DeliveryError};
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;

pub fn load_config(file: &PathBuf) -> Result<Json, DeliveryError> {
    let mut config_file = try!(File::open(file));
    let mut config_json = String::new();
    try!(config_file.read_to_string(&mut config_json));
    let data = try!(Json::from_str(&config_json));
    Ok(data)
}
