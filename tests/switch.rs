mod integration_tests {
    use std::{fs, io::Write};

    use assert_cmd::Command;
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_switch_to_branch() {
        let test_git_repo = TestGitRepo::new();
        let mut test_file = test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_dir("test_dir");
        let mut nested_test_file = test_git_repo
            .temp_dir
            .create_test_file("test_dir/test_in_dir.txt", b"test_in_dir");

        test_git_repo.init();
        test_git_repo.write_config(
            b"
[user]
name = \"Test User\"
email = \"test@user.com\"",
        );

        test_git_repo.add("test.txt test_dir");
        test_git_repo.commit("Test commit");

        // Create branch.
        test_git_repo.branch("my-new-branch");

        // Switch to new branch
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("switch")
            .arg("my-new-branch")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("HEAD", "ref: refs/heads/my-new-branch");

        // Update test files and commit.
        test_file.write_all(b"updated test").unwrap();
        nested_test_file.write_all(b"updated nested test").unwrap();
        test_git_repo.add("test.txt test_dir");
        test_git_repo.commit("Test commit on new branch");

        // Switch back to main branch.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("switch")
            .arg("main")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("HEAD", "ref: refs/heads/main");
        let file_contents =
            fs::read_to_string(test_git_repo.temp_dir.path().join("test.txt")).unwrap();
        assert_eq!(file_contents, "test");
        let nested_file_contents = fs::read_to_string(
            test_git_repo
                .temp_dir
                .path()
                .join("test_dir/test_in_dir.txt"),
        )
        .unwrap();
        assert_eq!(nested_file_contents, "test_in_dir");
    }
}
