mod integration_tests {
    use assert_cmd::Command;
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_switch_to_branch() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.init();
        test_git_repo.write_config(
            b"
[user]
name = \"Test User\"
email = \"test@user.com\"",
        );

        test_git_repo.add("test.txt");
        let commit_obj_id = test_git_repo.commit("Test commit");

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

        // TODO: check HEAD is updated
        // Check working directory.
        // Make changes and commit to branch.
        // Switch back to main and check HEAD and working directory.
    }
}
