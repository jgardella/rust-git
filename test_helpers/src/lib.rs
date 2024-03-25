use std::{fs::File, io::{Read, Write}, str::from_utf8};

use assert_cmd::Command;
use assert_fs::{assert::PathAssert, fixture::{ChildPath, PathChild}, TempDir};
use predicates::{boolean::PredicateBooleanExt, prelude::predicate};
use flate2::read::ZlibDecoder;

pub trait TempDirExt {
    fn create_test_file(&self, file_name: &str, contents: &[u8]);
}

impl TempDirExt for TempDir {
    fn create_test_file(&self, file_name: &str, contents: &[u8]) {
        let test_file_path = self.child(file_name);
        let mut test_file = File::create(test_file_path).unwrap();
        test_file.write_all(contents).unwrap();
    }
}

pub struct TestGitRepo {
    pub temp_dir: TempDir
}

impl TestGitRepo {
    pub fn init() -> Self {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("init")
        .current_dir(temp_dir.path())
        .unwrap();

        TestGitRepo {
            temp_dir
        }
    }

    pub fn hash_object(&self, obj: &str) -> String {
        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("hash-object")
            .arg("--stdin")
            .arg("-w")
            .current_dir(self.temp_dir.path())
            .write_stdin(obj)
            .unwrap();

        String::from(from_utf8(&cmd.stdout).unwrap().trim())
    }

    fn git_dir(&self) -> ChildPath {
        self.temp_dir.child(".git")
    }

    pub fn objects_dir(&self) -> ChildPath {
        self.git_dir().child("objects")
    }

    pub fn assert_obj_file(&self, obj_id: &str, contents: &str) {
        let (folder_name, file_name) = obj_id.split_at(2);

        let obj_file_path = self.objects_dir().child(folder_name).child(file_name);
        obj_file_path.assert(predicate::path::exists());

        let mut obj_file = File::open(obj_file_path).unwrap();
        let obj_file_contents = Self::decompress_object_file(&mut obj_file);
        assert_eq!(contents, obj_file_contents);
    }

    pub fn assert_no_obj_file(&self, obj_id: &str) {
        let (folder_name, file_name) = obj_id.split_at(2);

        let obj_file_path = self.objects_dir().child(folder_name).child(file_name);
        obj_file_path.assert(predicate::path::exists().not());
    }

    pub fn assert_unsupported_option(&self, command: &str, args: Vec<&str>) {
        let option = args.first().unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg(command)
        .args(&args)
        .current_dir(self.temp_dir.path())
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(format!("{option} not supported"));
    }

    fn decompress_object_file(file: &mut File) -> String {
        let mut decoder = ZlibDecoder::new(file);
        let mut decoded = String::new();
        decoder.read_to_string(&mut decoded).unwrap();
        decoded
    }
}
