mod integration_tests {
    use assert_cmd::{assert::OutputAssertExt, Command};
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_add_files_to_index() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test_in_dir.txt", b"test_in_dir");

        test_git_repo.init();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("add")
            .arg("test.txt")
            .arg("test2.txt")
            .arg("test_dir")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert().success();

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test.txt
test2.txt
test_dir/test_in_dir.txt")
    }

    #[test]
    fn should_add_file_to_index_from_nested_dir() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test_in_dir.txt", b"test_in_dir");

        test_git_repo.init();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("add")
            .arg("test_in_dir.txt")
            .current_dir(test_git_repo.temp_dir.join("test_dir"))
            .unwrap();

        cmd.assert().success();

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test_dir/test_in_dir.txt")
    }

    #[test]
    fn should_add_file_from_upper_directory() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test_dir/test_in_dir.txt", b"test_in_dir");

        test_git_repo.init();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("add")
            .arg("../test.txt")
            .current_dir(test_git_repo.temp_dir.join("test_dir"))
            .unwrap();

        cmd.assert().success();

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test.txt")
    }

    #[test]
    fn should_fail_to_add_file_from_outside_repo() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");

        test_git_repo.init_in_dir("my_proj");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("add")
        .arg("../test.txt")
        .current_dir(test_git_repo.temp_dir.join("my_proj"))
        .assert()
        .failure()
        .stderr("path \"../test.txt\" is outside of repo");
    }
}

mod compatibility_tests {

    use std::{fs::File, io::Read};

    use assert_fs::{fixture::PathChild, TempDir};
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_generate_same_index() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test_in_dir.txt", b"test_in_dir");

        let commands = vec![
            "init",
            "add test.txt test2.txt test_dir"
        ];

        let get_index_bytes = |temp_dir: &TempDir| {
            let mut file = File::open(temp_dir.child(".git/index")).unwrap();
            let mut b = Vec::new();
            file.read_to_end(&mut b).unwrap();
            b
        };

        test_git_repo.assert_compatibility(commands, get_index_bytes);
    }
}