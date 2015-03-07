use std::process::Command;
use errors::{DeliveryError, Kind};
use libc::funcs::posix88::unistd;
use std::path::AsPath;
use std::fs;

pub mod say;
pub mod path_join_many;

// This will need a windows implementation
pub fn copy_recursive<P: ?Sized>(f: &P, t: &P) -> Result<(), DeliveryError> where P: AsPath {
    let from = f.as_path();
    let to = t.as_path();
    let result = try!(Command::new("cp")
         .arg("-R")
         .arg("-a")
         .arg(from.to_str().unwrap())
         .arg(to.to_str().unwrap())
         .output());
    if !result.status.success() {
        return Err(DeliveryError{kind: Kind::CopyFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}", String::from_utf8_lossy(&result.stdout), String::from_utf8_lossy(&result.stderr)))});
    }
    Ok(())
}

pub fn remove_recursive<P: ?Sized>(path: &P) -> Result<(), DeliveryError> where P: AsPath {
    try!(Command::new("rm")
         .arg("-rf")
         .arg(path.as_path().to_str().unwrap())
         .output());
    Ok(())
}

pub fn mkdir_recursive<P: ?Sized>(path: &P) -> Result<(), DeliveryError> where P: AsPath {
    try!(fs::create_dir_all(path.as_path()));
    Ok(())
}

// This will need a windows implementation
pub fn chmod<P: ?Sized>(path: &P, setting: &str) -> Result<(), DeliveryError> where P: AsPath {
    let result = try!(Command::new("chmod")
         .arg(setting)
         .arg(path.as_path().to_str().unwrap())
         .output());
    if !result.status.success() {
        return Err(DeliveryError{kind: Kind::ChmodFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}", String::from_utf8_lossy(&result.stdout), String::from_utf8_lossy(&result.stderr)))});
    }
    Ok(())
}

pub fn privileged_process() -> bool {
    match unsafe { unistd::getuid() } {
        0 => true,
        _ => false
    }
}
