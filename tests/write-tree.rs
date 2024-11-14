mod integration_tests {
    use assert_cmd::{assert::OutputAssertExt, Command};
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_write_tree_object_to_repo() {
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
        test_git_repo.add("test.txt test2.txt test_dir");

        let cmd = Command::cargo_bin("rust-git")
            .unwrap()
            .arg("write-tree")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("03f9b0b16745d6529b86f7e7cf12bb0a254b6b8e\n");

        let cat_file_type =
            test_git_repo.cat_file("-t", "03f9b0b16745d6529b86f7e7cf12bb0a254b6b8e");

        assert_eq!(cat_file_type, "tree");

        let cat_file_content =
            test_git_repo.cat_file("-p", "03f9b0b16745d6529b86f7e7cf12bb0a254b6b8e");

        assert_eq!(
            cat_file_content,
            "040000 tree c7c1cd98552375307d7d3ac561793842c3a47abd\ttest_dir
100644 blob 30d74d258442c7c65512eafab474568dd706c430\ttest.txt
100644 blob d606037cb232bfda7788a8322492312d55b2ae9d\ttest2.txt"
        );

        let cat_sub_tree_type =
            test_git_repo.cat_file("-t", "c7c1cd98552375307d7d3ac561793842c3a47abd");

        assert_eq!(cat_sub_tree_type, "tree");

        let cat_sub_tree_content =
            test_git_repo.cat_file("-p", "c7c1cd98552375307d7d3ac561793842c3a47abd");

        assert_eq!(
            cat_sub_tree_content,
            "100644 blob 27961cd8c28d3fd780672e10e8c7c25d6eee10ee\ttest_in_dir.txt"
        );
    }
}
