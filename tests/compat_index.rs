mod compatibility_tests {

    use std::{fs::File, io::Read};

    use assert_fs::{fixture::PathChild, TempDir};
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_generate_same_index() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");

        let commands = vec![
            "init",
            "add test.txt test2.txt"
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