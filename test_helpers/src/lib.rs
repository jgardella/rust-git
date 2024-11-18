use std::{
    fmt::Debug,
    fs::{self, File},
    io::{Read, Write},
    str::from_utf8,
};

use assert_cmd::Command;
use assert_fs::{
    assert::PathAssert,
    fixture::{ChildPath, PathChild},
    TempDir,
};
use flate2::read::ZlibDecoder;
use predicates::{boolean::PredicateBooleanExt, prelude::predicate};

pub trait TempDirExt {
    fn create_test_dir(&self, dir_name: &str);
    fn create_test_file(&self, file_name: &str, contents: &[u8]) -> File;
}

impl TempDirExt for TempDir {
    fn create_test_dir(&self, dir_name: &str) {
        let test_dir_path = self.child(dir_name);
        fs::create_dir_all(test_dir_path).unwrap();
    }

    fn create_test_file(&self, file_name: &str, contents: &[u8]) -> File {
        let test_file_path = self.child(file_name);
        let mut test_file = File::create(test_file_path).unwrap();
        test_file.write_all(contents).unwrap();
        test_file
    }
}

pub struct TestGitRepo {
    pub temp_dir: TempDir,
}

impl TestGitRepo {
    pub fn new() -> Self {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        TestGitRepo { temp_dir }
    }

    pub fn init(&self) {
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("init")
            .current_dir(self.temp_dir.path())
            .unwrap();
    }

    pub fn init_in_dir(&self, dir_name: &str) {
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("init")
            .arg(&dir_name)
            .current_dir(self.temp_dir.path())
            .unwrap();
    }

    pub fn hash_object(&self, obj: &str) -> String {
        let cmd = Command::cargo_bin("rust-git")
            .unwrap()
            .arg("hash-object")
            .arg("--stdin")
            .arg("-w")
            .current_dir(self.temp_dir.path())
            .write_stdin(obj)
            .unwrap();

        String::from(from_utf8(&cmd.stdout).unwrap().trim())
    }

    pub fn add(&self, files: &str) {
        let mut add_args = String::from("add ");
        add_args.push_str(files);

        Command::cargo_bin("rust-git")
            .unwrap()
            .args(add_args.split(' '))
            .current_dir(self.temp_dir.path())
            .unwrap();
    }

    pub fn cat_file(&self, flag: &str, file: &str) -> String {
        let cmd = Command::cargo_bin("rust-git")
            .unwrap()
            .arg("cat-file")
            .arg(flag)
            .arg(file)
            .current_dir(self.temp_dir.path())
            .unwrap();

        String::from(from_utf8(&cmd.stdout).unwrap().trim())
    }

    pub fn ls_files(&self) -> String {
        let cmd = Command::cargo_bin("rust-git")
            .unwrap()
            .arg("ls-files")
            .current_dir(self.temp_dir.path())
            .unwrap();

        String::from(from_utf8(&cmd.stdout).unwrap().trim())
    }

    pub fn write_tree(&self) -> String {
        let cmd = Command::cargo_bin("rust-git")
            .unwrap()
            .arg("write-tree")
            .current_dir(self.temp_dir.path())
            .unwrap();

        String::from(from_utf8(&cmd.stdout).unwrap().trim())
    }

    pub fn git_dir(&self) -> ChildPath {
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

    /// Runs the same commands through Rust Git and C Git and asserts that some state is the same.
    /// Perform any non-git related setup before calling this function (e.g. test file creation).
    pub fn assert_compatibility<T: Debug + PartialEq>(
        &self,
        commands: Vec<&str>,
        state_getter: impl Fn(&TempDir) -> T,
    ) {
        let split_commands: Vec<Vec<&str>> = commands
            .iter()
            .map(|command| command.split(' ').collect())
            .collect();

        // Run commands with Rust git.
        for command in &split_commands {
            Command::cargo_bin("rust-git")
                .unwrap()
                .args(command)
                .current_dir(self.temp_dir.path())
                .unwrap();
        }

        let rust_git_state = state_getter(&self.temp_dir);

        // Cleanup git folder.
        fs::remove_dir_all(self.git_dir()).unwrap();

        // Run commands with C git.
        // TODO: this will currently use whatever git is installed on the machine. Update to test against a specific version.
        for command in &split_commands {
            Command::new("git")
                .args(command)
                .current_dir(self.temp_dir.path())
                .unwrap();
        }

        let c_git_state = state_getter(&self.temp_dir);

        assert_eq!(rust_git_state, c_git_state);
    }
}
