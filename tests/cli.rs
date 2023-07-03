use std::{
    fs::{create_dir, read_to_string},
    time::{Duration, SystemTime},
};

use assert_cmd::Command;
use assert_fs::prelude::{FileWriteStr, PathChild};
use filetime::{set_file_mtime, FileTime};

const CONF: &'static str = r#"
[global]
min_age_seconds = 500

[paths.test1]
path = "foo"
dest = "bar"

[paths.test2]
path = "mydir/baz"
dest = "mydir/buz space"
min_age_seconds = 0
"#;

// TODO: this could be cleaner - MCL - 2023-07-02
#[test]
fn happy_path() {
    let temp = assert_fs::TempDir::new().unwrap();

    temp.child("shift.toml").write_str(CONF).unwrap();

    temp.child("foo/file one.txt")
        .write_str("file one")
        .unwrap();

    temp.child("foo/file2.txt").write_str("file2").unwrap();

    temp.child("foo/file3.txt").write_str("file3").unwrap();

    temp.child("foo/bar/file4.txt").write_str("file4").unwrap();

    create_dir(temp.child("bar").path()).unwrap();
    create_dir(temp.child("mydir").path()).unwrap();
    create_dir(temp.child("mydir/baz").path()).unwrap();
    create_dir(temp.child("mydir/buz space").path()).unwrap();

    temp.child("mydir/baz/file1.txt")
        .write_str("file1")
        .unwrap();

    temp.child("mydir/baz/file2.txt")
        .write_str("file2")
        .unwrap();

    let cur = SystemTime::now();
    let adjust = Duration::new(600, 0);
    let adjusted = cur - adjust;

    let old_enough = FileTime::from_system_time(adjusted);
    set_file_mtime(temp.child("foo/file one.txt"), old_enough).unwrap();
    set_file_mtime(temp.child("foo/file3.txt"), old_enough).unwrap();

    let mut cmd = Command::cargo_bin("shifters").unwrap();
    cmd.current_dir(temp.path());
    cmd.args([
        "--execute",
        "--config",
        &temp.child("shift.toml").to_string_lossy(),
    ]);
    cmd.assert().success();

    // for test1
    assert!(!temp.child("foo/file one.txt").exists());
    assert!(temp.child("bar/file one.txt").exists());
    // sanity check the contents of at least one file
    assert_eq!(
        read_to_string(temp.child("bar/file one.txt")).unwrap(),
        "file one"
    );

    // this should not have been moved because it's not old enough
    assert!(temp.child("foo/file2.txt").exists());

    assert!(!temp.child("foo/file3.txt").exists());
    assert!(temp.child("bar/file3.txt").exists());

    // we don't recurse
    assert!(temp.child("foo/bar/file4.txt").exists());

    // for test2, which has no min age
    assert!(!temp.child("mydir/baz/file1.txt").exists());
    assert!(temp.child("mydir/buz space/file1.txt").exists());

    assert!(!temp.child("mydir/baz/file2.txt").exists());
    assert!(temp.child("mydir/buz space/file2.txt").exists());

    temp.close().unwrap();
}
