mod integration_tests {
    use std::io::Write;

    use assert_cmd::{assert::OutputAssertExt, Command};
    use assert_fs::{assert::PathAssert, fixture::PathChild};
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_restore_file_from_index() {
        let test_git_repo = TestGitRepo::new();
        let mut test_file = test_git_repo.temp_dir.create_test_file("test.txt", b"original contents");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        test_file.write(b"updated file").unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("restore")
            .arg("test.txt")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert().success();

        test_git_repo.temp_dir.child("test.txt").assert("original contents");
    }

    #[test]
    fn should_restore_directory_from_index() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        let mut test_file1 = test_git_repo.temp_dir.create_test_file("test_dir/test.txt", b"original contents");
        let mut test_file2 = test_git_repo.temp_dir.create_test_file("test_dir/test2.txt", b"original contents 2");

        test_git_repo.init();
        test_git_repo.add("test_dir");

        test_file1.write(b"updated file 1").unwrap();
        test_file2.write(b"updated file 2").unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("restore")
            .arg("test_dir")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert().success();

        test_git_repo.temp_dir.child("test_dir/test.txt").assert("original contents");
        test_git_repo.temp_dir.child("test_dir/test2.txt").assert("original contents 2");
    }

    #[test]
    fn should_fail_to_restore_file_outside_repo() {
        let test_git_repo = TestGitRepo::new();

        test_git_repo.init_in_dir("my_proj");
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("restore")
        .arg("../test.txt")
        .current_dir(test_git_repo.temp_dir.join("my_proj"))
        .assert()
        .failure()
        .stderr("path \"../test.txt\" is outside of repo");
    }

    #[test]
    fn should_restore_file_from_upper_directory() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        let mut test_file = test_git_repo.temp_dir.create_test_file("test.txt", b"original contents");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        test_file.write(b"updated file 1").unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("restore")
            .arg("../test.txt")
            .current_dir(test_git_repo.temp_dir.join("test_dir"))
            .unwrap();

        cmd.assert().success();

        test_git_repo.temp_dir.child("test.txt").assert("original contents");
    }
}