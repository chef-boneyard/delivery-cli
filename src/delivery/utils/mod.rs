//
// Copyright:: Copyright (c) 2015 Chef Software, Inc.
// License:: Apache License, Version 2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use errors::{DeliveryError, Kind};
use std::convert::AsRef;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Output as CmdOutput;
use types::DeliveryResult;
use utils::path_join_many::PathJoinMany;

pub mod open;
pub mod path_ext;
pub mod path_join_many;
pub mod say;
#[cfg(test)]
pub mod test_paths;

use crypto::digest::Digest;
use crypto::md5::Md5;

#[cfg(not(target_os = "windows"))]
pub use self::unix::*;

#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(not(target_os = "windows"))]
mod unix;

#[cfg(target_os = "windows")]
mod windows;

// Extract the Environment Variable of the provided `key`
pub fn env_variable(key: &str) -> Option<String> {
    env::var(key).ok()
}

pub fn mkdir_recursive<P: ?Sized>(path: &P) -> Result<(), DeliveryError>
where
    P: AsRef<Path>,
{
    try!(fs::create_dir_all(path.as_ref()));
    Ok(())
}

pub fn home_dir(to_append: &[&str]) -> Result<PathBuf, DeliveryError> {
    match env::home_dir() {
        Some(home) => Ok(home.join_many(to_append)),
        None => {
            let msg = "unable to find home dir".to_string();
            Err(DeliveryError {
                kind: Kind::NoHomedir,
                detail: Some(msg),
            })
        }
    }
}

// Read from STDIN
//
// Useful helper method to ask questions to the end-user
//
// Example:
// ```
// say("yellow", "How cool is the delivery-cli? [1-10] ");
// let coolness = utils::read_from_terminal()?;
// assert_eq!(coolness, 10);
// ```
pub fn read_from_terminal() -> DeliveryResult<String> {
    let mut buff = String::new();
    try!(io::stdin().read_line(&mut buff));
    Ok(buff.trim().to_string())
}

/// Walk up a file hierarchy searching for `dir/target`.
pub fn walk_tree_for_path<P>(dir: P, target: &str) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    let mut current = dir.as_ref();
    loop {
        let candidate = current.join(target);
        if fs::metadata(&candidate).is_ok() {
            let ans = PathBuf::from(candidate);
            return Some(ans);
        }
        match current.parent() {
            Some(p) => current = p,
            None => return None,
        }
    }
}

// Convert a path into a String. Panic if the path contains
// non-unicode sequences.
pub fn path_to_string<P: AsRef<Path>>(p: P) -> String {
    let path = p.as_ref();
    match path.to_str() {
        Some(s) => s.to_string(),
        None => {
            let s = format!("invalid path (non-unicode): {}", path.to_string_lossy());
            panic!(s)
        }
    }
}

/// Return the content of the provided file
///
/// An easy way to read a file
///
/// # Examples
///
/// ```
/// use std::fs::{File, remove_file};
/// use std::io::prelude::*;
/// use std::path::PathBuf;
/// use delivery::utils::read_file;
///
/// let mut f = File::create("foo.txt").unwrap();
/// f.write_all(b"Cool beans!");
///
/// let f = PathBuf::from("foo.txt");
/// assert_eq!("Cool beans!", read_file(&f).unwrap());
///
/// remove_file("foo.txt");
/// ```
pub fn read_file<P>(path: P) -> DeliveryResult<String>
where
    P: AsRef<Path>,
{
    let mut buffer = String::new();
    let mut f = try!(File::open(path));
    try!(f.read_to_string(&mut buffer));
    Ok(buffer)
}

// Return the current directory path
pub fn cwd() -> PathBuf {
    env::current_dir().unwrap()
}

// Returns true if dest_f doesn't exist or has content different from source_f,
// returns false if dest_f exist but contains the exact content as source_f.
pub fn file_needs_updated<A, B>(source_f: A, dest_f: B) -> DeliveryResult<bool>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    if dest_f.as_ref().exists() {
        let mut md5_source = Md5::new();
        let mut source_f = try!(File::open(&source_f));
        let mut source_str = String::new();
        try!(source_f.read_to_string(&mut source_str));
        md5_source.input_str(&source_str);

        let mut md5_dest = Md5::new();
        let mut dest_f = try!(File::open(&dest_f));
        let mut dest_str = String::new();
        try!(dest_f.read_to_string(&mut dest_str));
        md5_dest.input_str(&dest_str);

        // If the md5 sun matches, return None to signify that
        // the file was not copied because they match exactly.
        if md5_source.result_str() == md5_dest.result_str() {
            return Ok(false);
        }
    }
    Ok(true)
}

// Verify if the `process::Output` of a `process::Command`
// was executed successfully, otherwise return the provided
// error and printing the STDOUT & STDERR
pub fn cmd_success_or_err(out: &CmdOutput, e_kind: Kind) -> DeliveryResult<()> {
    if !out.status.success() {
        return Err(DeliveryError {
            kind: e_kind,
            detail: Some(format!(
                "STDOUT: {}\nSTDERR: {}\n",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )),
        });
    }
    Ok(())
}

pub fn generate_command_from_string(cmd_str: &str) -> process::Command {
    let mut cmd_vec = cmd_str.split(" ").collect::<Vec<_>>();
    let mut cmd = make_command(&cmd_vec.remove(0));
    if cmd_vec.len() > 0 {
        cmd.args(&cmd_vec);
    }
    cmd
}

pub fn kill_child_processes(child_processes: Vec<process::Child>) -> DeliveryResult<()> {
    for mut child in child_processes {
        try!(child.kill());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::path::PathBuf;
    use utils::test_paths::fixture_file;

    #[test]
    fn traverse_up_for_dot_delivery_found() {
        let p = fixture_file("test_repo");
        let result = walk_tree_for_path(&p, ".delivery");
        assert!(result.is_some());
        assert_eq!(Some(OsStr::new(".delivery")), result.unwrap().file_name());
    }

    #[test]
    fn traverse_up_for_dot_delivery_not_found() {
        // starting from / we don't expect to find .delivery
        let result = walk_tree_for_path(&PathBuf::from("/"), ".delivery-123");
        assert!(result.is_none());
    }

    #[test]
    fn cmd_success_or_err() {
        let ls = make_command("ls").output().unwrap();
        assert!(super::cmd_success_or_err(&ls, Kind::FailedToExecute).is_ok());

        let no_zero_exitcode = make_command("ls").arg("-").output().unwrap();
        assert!(super::cmd_success_or_err(&no_zero_exitcode, Kind::FailedToExecute).is_err());
    }
}

#[cfg(test)]
mod test_find_command {
    pub use super::find_command;
    use std::env;
    use std::fs;

    fn setup_empty_pathext() {
        if env::var("PATHEXT").is_ok() {
            env::remove_var("PATHEXT")
        }
    }

    fn setup_path() {
        let path_env = env::var_os("PATH").unwrap();
        let mut path_vec = env::split_paths(&path_env).collect::<Vec<_>>();
        let second_path = fs::canonicalize("./tests/fixtures/bin").unwrap();
        path_vec.push(second_path);
        let test_paths = env::join_paths(path_vec).unwrap();
        env::set_var("PATH", &test_paths);
    }

    mod without_pathext_set {
        pub use super::find_command;
        use super::{setup_empty_pathext, setup_path};

        fn setup_environment() {
            setup_path();
            setup_empty_pathext();
        }

        mod argument_without_extension {
            use super::{find_command, setup_environment};

            #[test]
            fn command_exists() {
                setup_environment();
                let result = find_command("bin_with_no_extension");
                assert_eq!(result.is_some(), true);
            }

            #[test]
            fn command_does_not_exist() {
                setup_environment();
                let result = find_command("missing");
                assert_eq!(result.is_some(), false);
            }

            #[test]
            fn command_exists_with_extension() {
                setup_environment();
                let result = find_command("win95_dominator");
                assert_eq!(result.is_some(), false);
            }
        }

        mod argument_with_extension {
            use super::{find_command, setup_environment};
            use std::fs::canonicalize;

            #[test]
            fn command_exists() {
                setup_environment();
                let result = find_command("bin_with_extension.exe");
                assert_eq!(result.is_some(), true);
            }

            #[test]
            fn command_does_not_exist() {
                setup_environment();
                let result = find_command("missing.com");
                assert_eq!(result.is_some(), false);
            }

            #[test]
            fn command_different_extension_does_exist() {
                setup_environment();
                let result = find_command("bin_with_extension.com");
                assert_eq!(result.is_some(), false);
            }

            #[test]
            fn first_command_on_path_found() {
                setup_environment();
                let target_path = canonicalize("./tests/fixtures/bin/plan.sh").unwrap();
                let result = find_command("plan.sh");
                let found_path = result.unwrap();
                assert_eq!(found_path, target_path);
            }
        }
    }

    #[cfg(target_os = "windows")]
    mod with_pathext_set {
        pub use super::find_command;
        use super::setup_path;
        use std::env;
        use std::path::PathBuf;

        fn setup_pathext() {
            let path_bufs = vec![PathBuf::from(".COM"), PathBuf::from(".EXE")];
            let new_path = env::join_paths(path_bufs).unwrap();
            env::set_var("PATHEXT", &new_path);
        }

        fn setup_environment() {
            setup_path();
            setup_pathext();
        }

        mod argument_without_extension {
            use super::{find_command, setup_environment};

            #[test]
            fn command_exists() {
                setup_environment();
                let result = find_command("bin_with_no_extension");
                assert_eq!(result.is_some(), true);
            }

            #[test]
            fn command_does_not_exist() {
                setup_environment();
                let result = find_command("missing");
                assert_eq!(result.is_some(), false);
            }

            #[test]
            fn command_exists_with_extension_in_pathext() {
                setup_environment();
                let result = find_command("bin_with_extension");
                assert_eq!(result.is_some(), true);
            }

            #[test]
            fn command_exists_with_extension_not_in_pathext() {
                setup_environment();
                let result = find_command("win95_dominator");
                assert_eq!(result.is_some(), false);
            }
        }

        mod argument_with_extension {
            use super::{find_command, setup_environment};
            use std::fs::canonicalize;

            #[test]
            fn command_exists() {
                setup_environment();
                let result = find_command("bin_with_extension.exe");
                assert_eq!(result.is_some(), true);
            }

            #[test]
            fn command_does_not_exist() {
                setup_environment();
                let result = find_command("missing.com");
                assert_eq!(result.is_some(), false);
            }

            #[test]
            fn command_different_extension_does_exist() {
                setup_environment();
                let result = find_command("bin_with_extension.com");
                assert_eq!(result.is_some(), false);
            }

            #[test]
            fn first_command_on_path_found() {
                setup_environment();
                let target_path = canonicalize("./tests/fixtures/bin/plan.sh").unwrap();
                let result = find_command("plan.sh");
                let found_path = result.unwrap();
                assert_eq!(found_path, target_path);
            }
        }
    }
}
