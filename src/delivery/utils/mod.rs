use std::old_io::process::Command;
use errors::{DeliveryError, Kind};
use libc::funcs::posix88::unistd;

pub mod say;

// This will need a windows implementation
pub fn copy_recursive(from: &Path, to: &Path) -> Result<(), DeliveryError> {
    let result = try!(Command::new("cp")
         .arg("-R")
         .arg("-a")
         .arg(from.as_str().unwrap())
         .arg(to.as_str().unwrap())
         .output());
    if !result.status.success() {
        return Err(DeliveryError{kind: Kind::CopyFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}", String::from_utf8_lossy(&result.output), String::from_utf8_lossy(&result.error)))});
    }
    Ok(())
}

// This too will need a windows implementation
pub fn remove_recursive(path: &Path) -> Result<(), DeliveryError> {
    try!(Command::new("rm")
         .arg("-rf")
         .arg(path.as_str().unwrap())
         .output());
    Ok(())
}

// This will need an, um, windows implementation
pub fn mkdir_recursive(path: &Path) -> Result<(), DeliveryError> {
    try!(Command::new("mkdir")
         .arg("-p")
         .arg(path.as_str().unwrap())
         .output());
    Ok(())
}

// This will need a windows implementation
pub fn chmod(path: &Path, setting: &str) -> Result<(), DeliveryError> {
    let result = try!(Command::new("chmod")
         .arg(setting)
         .arg(path.as_str().unwrap())
         .output());
    if !result.status.success() {
        return Err(DeliveryError{kind: Kind::ChmodFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}", String::from_utf8_lossy(&result.output), String::from_utf8_lossy(&result.error)))});
    }
    Ok(())
}

pub fn privileged_process() -> bool {
    match unsafe { unistd::getuid() } {
        0 => true,
        _ => false
    }
}
