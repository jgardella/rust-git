mod integration_tests {
    use assert_cmd::Command;
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_create_read_update_and_delete_symbolic_ref() {
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
        let tree_obj_id = test_git_repo.write_tree();
        let commit_obj_id = test_git_repo.commit_tree(&tree_obj_id, "Test commit");
        test_git_repo.update_ref("refs/heads/main", &commit_obj_id);

        // Create symbolic ref.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("symbolic-ref")
            .arg("HEAD")
            .arg("refs/heads/main")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("HEAD", "ref: refs/heads/main");

        // Update symbolic ref.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("symbolic-ref")
            .arg("HEAD")
            .arg("refs/heads/test")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("HEAD", "ref: refs/heads/test");

        // Read symbolic ref.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("symbolic-ref")
            .arg("HEAD")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success()
            .stdout("refs/heads/test");

        // Delete symbolic ref.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("symbolic-ref")
            .arg("--delete")
            .arg("HEAD")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_no_ref_file("HEAD");
    }

    #[test]
    fn should_print_message_for_missing_ref_file() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.init();

        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("symbolic-ref")
            .arg("MISSING")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success()
            .stdout("no symbolic-ref 'MISSING'\n");
    }

    #[test]
    fn should_print_short_ref() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.init();

        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("symbolic-ref")
            .arg("--short")
            .arg("HEAD")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success()
            .stdout("main\n");
    }
}
