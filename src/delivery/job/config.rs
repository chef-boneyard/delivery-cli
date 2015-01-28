#![allow(unstable)]
use rustc_serialize::json::{Json};
use errors::{DeliveryError};
use std::io::{File};

pub fn load_config(file: &Path) -> Result<Json, DeliveryError> {
    let config_json = try!(File::open(file).read_to_string());
    let data = try!(Json::from_str(config_json.as_slice()));
    Ok(data)
}
