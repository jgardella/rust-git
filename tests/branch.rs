mod integration_tests {
    use assert_cmd::Command;
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_create_read_update_and_delete_branch() {
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
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("branch")
            .arg("my-new-branch")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("refs/heads/my-new-branch", &commit_obj_id);

        // TODO: test rename branch

        // List branches.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("branch")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success()
            .stdout("my-new-branch\nmain\n");

        // Delete branch.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("branch")
            .arg("--delete")
            .arg("my-new-branch")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        // Ref file deleted.
        test_git_repo.assert_no_ref_file("refs/heads/my-new-branch");

        // List branches.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("branch")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success()
            .stdout("main\n");
    }
}
