mod integration_tests {
    use assert_cmd::Command;
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_create_read_update_and_delete_lightweight_tag() {
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

        // Create lightweight tag.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .arg("v1.0")
            .arg(&tree_obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("refs/tags/v1.0", &tree_obj_id);

        // Update tag fails without force flag.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .arg("v1.0")
            .arg(&commit_obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .failure()
            .stderr("cannot overwrite existing tag v1.0");

        // No change to tag file.
        test_git_repo.assert_ref_file("refs/tags/v1.0", &tree_obj_id);

        // Update tag works with force flag.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .arg("-f")
            .arg("v1.0")
            .arg(&commit_obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        // Tag file updated.
        test_git_repo.assert_ref_file("refs/tags/v1.0", &commit_obj_id);

        // List tags.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success()
            .stdout("v1.0\n");

        // Delete tag.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .arg("--delete")
            .arg("v1.0")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        // Tag file deleted.
        test_git_repo.assert_no_ref_file("refs/tags/v1.0");

        // List tags.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success()
            .stdout("");
    }

    #[test]
    fn should_create_read_update_and_delete_annotated_tag() {
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

        // Create annotated tag.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .arg("v1.0")
            .arg(&tree_obj_id)
            .arg("-m")
            .arg("Test tag")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        let tag_obj_id = test_git_repo.assert_ref_file_read("refs/tags/v1.0");
        let cat_file_contents = test_git_repo.cat_file("-p", &tag_obj_id);
        let file_content_lines: Vec<&str> = cat_file_contents.split("\n").collect();

        let line1 = file_content_lines[0];
        let line2 = file_content_lines[1];
        let line3 = file_content_lines[2];
        let line4 = file_content_lines[3];
        let line5 = file_content_lines[4];
        let line6 = file_content_lines[5];

        assert_eq!(line1, &format!("object {tree_obj_id}"));
        assert_eq!(line2, "type tree");
        assert_eq!(line3, "tag v1.0");

        // TODO: check timestamp (somehow mock it)
        assert!(line4.starts_with("tagger Test User <test@user.com>"));
        assert_eq!(line5, "");
        assert_eq!(line6, "Test tag");

        // Update tag fails without force flag.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .arg("v1.0")
            .arg(&commit_obj_id)
            .arg("-m")
            .arg("Test tag")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .failure()
            .stderr("cannot overwrite existing tag v1.0");

        // No change to tag file.
        test_git_repo.assert_ref_file("refs/tags/v1.0", &tag_obj_id);
        let cat_file_contents = test_git_repo.cat_file("-p", &tag_obj_id);
        let file_content_lines: Vec<&str> = cat_file_contents.split("\n").collect();

        let line1 = file_content_lines[0];
        let line2 = file_content_lines[1];
        let line3 = file_content_lines[2];
        let line4 = file_content_lines[3];
        let line5 = file_content_lines[4];
        let line6 = file_content_lines[5];

        assert_eq!(line1, &format!("object {tree_obj_id}"));
        assert_eq!(line2, "type tree");
        assert_eq!(line3, "tag v1.0");

        // TODO: check timestamp (somehow mock it)
        assert!(line4.starts_with("tagger Test User <test@user.com>"));
        assert_eq!(line5, "");
        assert_eq!(line6, "Test tag");

        // Update tag works with force flag.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .arg("-f")
            .arg("v1.0")
            .arg(&commit_obj_id)
            .arg("-m")
            .arg("Updated test tag")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        // Tag object updated.
        let tag_obj_id = test_git_repo.assert_ref_file_read("refs/tags/v1.0");
        let cat_file_contents = test_git_repo.cat_file("-p", &tag_obj_id);
        let file_content_lines: Vec<&str> = cat_file_contents.split("\n").collect();

        let line1 = file_content_lines[0];
        let line2 = file_content_lines[1];
        let line3 = file_content_lines[2];
        let line4 = file_content_lines[3];
        let line5 = file_content_lines[4];
        let line6 = file_content_lines[5];

        assert_eq!(line1, &format!("object {commit_obj_id}"));
        assert_eq!(line2, "type commit");
        assert_eq!(line3, "tag v1.0");

        // TODO: check timestamp (somehow mock it)
        assert!(line4.starts_with("tagger Test User <test@user.com>"));
        assert_eq!(line5, "");
        assert_eq!(line6, "Updated test tag");

        // List tags.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success()
            .stdout("v1.0\n");

        // Delete tag.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .arg("--delete")
            .arg("v1.0")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        // Tag file deleted.
        test_git_repo.assert_no_ref_file("refs/tags/v1.0");

        // List tags.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success()
            .stdout("");
    }

    #[test]
    fn should_create_tag_for_head_if_no_sha_provided() {
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
        // TODO: use commit command which automatically updates HEAD ref
        let commit_obj_id = test_git_repo.commit_tree(&tree_obj_id, "Test commit");
        test_git_repo.update_ref("refs/heads/main", &commit_obj_id);
        test_git_repo.symbolic_ref("HEAD", "refs/heads/main");

        // Create tag from HEAD.
        Command::cargo_bin("rust-git")
            .unwrap()
            .arg("tag")
            .arg("v1.0")
            .current_dir(test_git_repo.temp_dir.path())
            .assert()
            .success();

        test_git_repo.assert_ref_file("refs/tags/v1.0", &commit_obj_id);
    }
}
