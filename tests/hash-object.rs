mod integration_tests {

    use assert_cmd::{Command, prelude::OutputAssertExt};
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_return_and_write_expected_hash_from_stdin() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.init();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("hash-object")
            .arg("--stdin")
            .arg("-w")
            .current_dir(test_git_repo.temp_dir.path())
            .write_stdin("test")
            .unwrap();

        cmd.assert()
            .success()
            .stdout("30d74d258442c7c65512eafab474568dd706c430\n");

        test_git_repo.assert_obj_file("30d74d258442c7c65512eafab474568dd706c430", "blob 4\0test");
    }

    #[test]
    fn should_not_write_if_flag_missing() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.init();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("hash-object")
            .arg("--stdin")
            .current_dir(test_git_repo.temp_dir.path())
            .write_stdin("test")
            .unwrap();

        cmd.assert()
            .success()
            .stdout("30d74d258442c7c65512eafab474568dd706c430\n");

        test_git_repo.assert_no_obj_file("30d74d258442c7c65512eafab474568dd706c430");
    }

    #[test]
    fn should_return_and_write_expected_hash_from_files() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.init();

        let test_file_name1 = "test-file-1.txt";
        test_git_repo.temp_dir.create_test_file(test_file_name1, b"test1");
        let test_file_name2 = "test-file-2.txt";
        test_git_repo.temp_dir.create_test_file(test_file_name2, b"test2");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("hash-object")
            .arg("-w")
            .arg(test_file_name1)
            .arg(test_file_name2)
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("f079749c42ffdcc5f52ed2d3a6f15b09307e975e\nd606037cb232bfda7788a8322492312d55b2ae9d\n");

        test_git_repo.assert_obj_file("f079749c42ffdcc5f52ed2d3a6f15b09307e975e", "blob 5\0test1");
        test_git_repo.assert_obj_file("d606037cb232bfda7788a8322492312d55b2ae9d", "blob 5\0test2");
    }

    #[test]
    fn should_return_and_write_expected_hash_from_stdin_paths() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.init();

        let test_file_name1 = "test-file-1.txt";
        test_git_repo.temp_dir.create_test_file(test_file_name1, b"test1");
        let test_file_name2 = "test-file-2.txt";
        test_git_repo.temp_dir.create_test_file(test_file_name2, b"test2");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("hash-object")
            .arg("-w")
            .arg("--stdin-paths")
            .write_stdin(format!("{test_file_name1}\n{test_file_name2}"))
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("f079749c42ffdcc5f52ed2d3a6f15b09307e975e\nd606037cb232bfda7788a8322492312d55b2ae9d\n");

        test_git_repo.assert_obj_file("f079749c42ffdcc5f52ed2d3a6f15b09307e975e", "blob 5\0test1");
        test_git_repo.assert_obj_file("d606037cb232bfda7788a8322492312d55b2ae9d", "blob 5\0test2");
    }

    #[test]
    fn should_fail_if_unsupported_options_provided() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.init();

        test_git_repo.assert_unsupported_option("hash-object", vec!["--no-filters"]);
        test_git_repo.assert_unsupported_option("hash-object", vec!["--path", "my-path"]);
        test_git_repo.assert_unsupported_option("hash-object", vec!["--literally"]);
    }

    // TODO: this is not quite correct, as hash-object should work outside of a git repo,
    // as long as the -w flag is not provided
    #[test]
    fn should_return_failure_if_no_git_repo_found() {
        let test_git_repo = TestGitRepo::new();

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("hash-object")
        .arg("--stdin")
        .current_dir(test_git_repo.temp_dir.path())
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(format!("not a git repository (or any of the parent directories): \"./.git\""));
    }
 }
