use std::old_io::process::Command;
use errors::{DeliveryError};

pub mod say;

// This will need a windows implementation
pub fn copy_recursive(from: &Path, to: &Path) -> Result<(), DeliveryError> {
    try!(Command::new("cp")
         .arg("-R")
         .arg("-a")
         .arg(from.as_str().unwrap())
         .arg(to.as_str().unwrap())
         .output());
    Ok(())
}
