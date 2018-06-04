extern crate delivery;
extern crate log;

use delivery::utils;
use delivery::utils::path_join_many::PathJoinMany;
use std::fs;
use std::path::Path;
use support::paths::fixture_file;
use tempdir::TempDir;

macro_rules! setup {
    () => {};
}

test!(copy_recursive {
    let source_dir = fixture_file("test_repo");
    let tmpdir = TempDir::new("utils-copy_recursive").unwrap();
    let dest_dir = tmpdir.path().to_path_buf();

    panic_on_error!(utils::copy_recursive(&source_dir, &dest_dir));

    let expected: &[&[&str]] = &[
        &["test_repo", "README.md"],
        &["test_repo", "cookbooks", "delivery_test", "metadata.rb"],
        &["test_repo", "cookbooks", "delivery_test", "recipes", "unit.rb"]];

    for e in expected {
        if !file_exists(&dest_dir.join_many(e)) {
            panic!(format!("copy_recursive failure: NOT FOUND '{:?}'",
                           &dest_dir.join_many(e)));
        }
    }
});

test!(remove_recursive {
    let source_dir = fixture_file("test_repo");
    let tmpdir = TempDir::new("utils-copy_recursive").unwrap();
    let dest_dir = tmpdir.path().to_path_buf();
    let cookbooks_dir = dest_dir.join_many(&["test_repo", "cookbooks"]);
    panic_on_error!(utils::copy_recursive(&source_dir, &dest_dir));
    assert!(file_exists(&cookbooks_dir));

    panic_on_error!(utils::remove_recursive(&cookbooks_dir));

    assert!(!file_exists(&cookbooks_dir));
});

fn file_exists<P: ?Sized>(f: &P) -> bool
where
    P: AsRef<Path>,
{
    fs::metadata(f).is_ok()
}
