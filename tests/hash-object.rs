#[cfg(test)]
mod integration_tests {
    use std::{fs::File, io::Write};

    use assert_fs::{assert::PathAssert, fixture::PathChild, TempDir};
    use predicates::prelude::*;
    use assert_cmd::{Command, prelude::OutputAssertExt};

    fn create_test_file(temp_dir: &TempDir, file_name: &str, contents: &[u8]) {
        let test_file_path = temp_dir.child(file_name);
        let mut test_file = File::create(test_file_path).unwrap();
        test_file.write_all(contents).unwrap();
    }

    #[test]
    fn should_return_expected_hash_from_stdin() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("init")
        .current_dir(temp_dir.path())
        .unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("hash-object")
            .arg("--stdin")
            .current_dir(temp_dir.path())
            .write_stdin("test")
            .unwrap();

        cmd.assert()
            .success()
            .stdout("30d74d258442c7c65512eafab474568dd706c430\n");
    }

    #[test]
    fn should_write_object_file() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("init")
        .current_dir(temp_dir.path())
        .unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("hash-object")
            .arg("--stdin")
            .arg("-w")
            .current_dir(temp_dir.path())
            .write_stdin("test")
            .unwrap();

        cmd.assert()
            .success()
            .stdout("30d74d258442c7c65512eafab474568dd706c430\n");

        let objects_dir = temp_dir.child(".git/objects");
        let obj_file_folder = objects_dir.child("30");

        obj_file_folder.assert(predicate::path::exists());

        let obj_file = obj_file_folder.child("d74d258442c7c65512eafab474568dd706c430");
        obj_file.assert(predicate::path::exists());
    }

    #[test]
    fn should_return_expected_hash_from_files() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let test_file_name1 = "test-file-1.txt";
        create_test_file(&temp_dir, test_file_name1, b"test1");
        let test_file_name2 = "test-file-2.txt";
        create_test_file(&temp_dir, test_file_name2, b"test2");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("init")
        .current_dir(temp_dir.path())
        .unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("hash-object")
            .arg(test_file_name1)
            .arg(test_file_name2)
            .current_dir(temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("f079749c42ffdcc5f52ed2d3a6f15b09307e975e\nd606037cb232bfda7788a8322492312d55b2ae9d\n");
    }

    #[test]
    fn should_return_expected_hash_from_stdin_paths() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let test_file_name1 = "test-file-1.txt";
        create_test_file(&temp_dir, test_file_name1, b"test1");
        let test_file_name2 = "test-file-2.txt";
        create_test_file(&temp_dir, test_file_name2, b"test2");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("init")
        .current_dir(temp_dir.path())
        .unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("hash-object")
            .arg("--stdin-paths")
            .write_stdin(format!("{test_file_name1}\n{test_file_name2}"))
            .current_dir(temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("f079749c42ffdcc5f52ed2d3a6f15b09307e975e\nd606037cb232bfda7788a8322492312d55b2ae9d\n");
    }
 }
