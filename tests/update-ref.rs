mod integration_tests {
    use assert_cmd::Command;
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_create_and_update_ref() {
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

        // Create initial ref.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("update-ref")
            .arg("refs/heads/master")
            .arg(&commit_obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("refs/heads/master", &commit_obj_id);

        // Update initial ref.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("update-ref")
            .arg("refs/heads/master")
            .arg(&tree_obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("refs/heads/master", &tree_obj_id);
    }

    #[test]
    fn should_create_and_update_ref_with_old_value() {
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

        // Create initial ref (old_value as empty string).
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("update-ref")
            .arg("refs/heads/master")
            .arg(&commit_obj_id)
            .arg("")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("refs/heads/master", &commit_obj_id);

        // Update initial ref (old_value as commit id).
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("update-ref")
            .arg("refs/heads/master")
            .arg(&tree_obj_id)
            .arg(&commit_obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("refs/heads/master", &tree_obj_id);
    }

    #[test]
    fn should_fail_to_update_with_incorrect_old_value() {
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

        // Create initial ref (old_value as non-empty string).
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("update-ref")
            .arg("refs/heads/master")
            .arg(&commit_obj_id)
            .arg("non-empty")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .failure();

        // Ref file shouldn't be created.
        test_git_repo.assert_no_ref_file("refs/heads/master");

        // Create initial ref.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("update-ref")
            .arg("refs/heads/master")
            .arg(&commit_obj_id)
            .arg("")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("refs/heads/master", &commit_obj_id);

        // Update initial ref (old_value doesn't match).
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("update-ref")
            .arg("refs/heads/master")
            .arg(&tree_obj_id)
            .arg("non-matching-old-value")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .failure();

        // File should have same contents.
        test_git_repo.assert_ref_file("refs/heads/master", &commit_obj_id);
    }
}
