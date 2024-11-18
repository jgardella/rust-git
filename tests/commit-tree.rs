mod integration_tests {
    use std::str::from_utf8;

    use assert_cmd::{assert::OutputAssertExt, Command};
    use predicates::str::starts_with;
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_return_error_emssage_when_user_config_unset() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.init();
        test_git_repo.add("test.txt");
        let tree_obj_id = test_git_repo.write_tree();

        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("commit-tree")
            .arg(&tree_obj_id)
            .arg("-m")
            .arg("Test commit")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .failure()
            .stderr(starts_with("*** Please tell me who you are."));
    }

    #[test]
    fn should_return_error_message_for_missing_commit_message() {
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

        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("commit-tree")
            .arg(&tree_obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .failure()
            .stderr("commit message cannot be empty");
    }

    #[test]
    fn should_commit_tree_object_to_repo() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo
            .temp_dir
            .create_test_file("test2.txt", b"test2");
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo
            .temp_dir
            .create_test_file("test_dir/test_in_dir.txt", b"test_in_dir");

        test_git_repo.init();
        test_git_repo.write_config(
            b"
[user]
name = \"Test User\"
email = \"test@user.com\"",
        );

        test_git_repo.add("test.txt test2.txt test_dir");
        let tree_obj_id = test_git_repo.write_tree();

        let cmd = Command::cargo_bin("rust-git")
            .unwrap()
            .arg("commit-tree")
            .arg(&tree_obj_id)
            .arg("-m")
            .arg("Test commit")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        let commit_id = String::from(from_utf8(&cmd.stdout).unwrap().trim());

        cmd.assert().success();

        let cat_file_type = test_git_repo.cat_file("-t", &commit_id);

        assert_eq!(cat_file_type, "commit");

        let cat_file_content = test_git_repo.cat_file("-p", &commit_id);
        let file_content_lines: Vec<&str> = cat_file_content.split("\n").collect();

        let line1: &str = file_content_lines[0];
        let line2: &str = file_content_lines[1];
        let line3: &str = file_content_lines[2];
        let line4: &str = file_content_lines[3];
        let line5: &str = file_content_lines[4];

        assert_eq!(line1, "tree 03f9b0b16745d6529b86f7e7cf12bb0a254b6b8e");

        // TODO: check timestamp (somehow mock it)
        assert!(line2.starts_with("author Test User <test@user.com>"));
        assert!(line3.starts_with("committer Test User <test@user.com>"));
        assert_eq!(line4, "");
        assert_eq!(line5, "Test commit");

        // TODO: create commit with parent and check file content
    }
}
