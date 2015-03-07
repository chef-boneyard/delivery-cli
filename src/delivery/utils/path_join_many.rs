use std::path::{Path, PathBuf};

/// You were too useful to die, join_many. This implements
/// what used to be the join_many method on old_path. Feed
/// it an array of &str's, and it will push them on to a
/// PathBuf, then return the final PathBuf.
pub trait PathJoinMany {
    fn join_many(&self, paths: &[&str]) -> PathBuf;
}

impl PathJoinMany for PathBuf {
    fn join_many(&self, paths: &[&str]) -> PathBuf {
        let mut buf = self.clone();
        for p in paths {
            buf = buf.join(p);
        }
        buf
    }
}

impl PathJoinMany for Path {
    fn join_many(&self, paths: &[&str]) -> PathBuf {
        let mut buf = self.to_path_buf();
        for p in paths {
            buf = buf.join(p);
        }
        buf
    }
}

