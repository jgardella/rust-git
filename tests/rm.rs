mod integration_tests {
    use assert_cmd::{assert::OutputAssertExt, Command};
    use assert_fs::{assert::PathAssert, fixture::PathChild};
    use predicates::prelude::predicate;
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_rm_files_from_index_and_working_directory() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");

        test_git_repo.init();
        test_git_repo.add("test.txt");
        test_git_repo.add("test2.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("rm")
            .arg("test.txt")
            .arg("test2.txt")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("rm 'test.txt'
rm 'test2.txt'
");

        // Use ls-files to check the files are removed from the index.
        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("test2.txt").assert(predicate::path::missing());
    }

    #[test]
    fn should_rm_files_only_from_index_with_cached_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");

        test_git_repo.init();
        test_git_repo.add("test.txt");
        test_git_repo.add("test2.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("rm")
            .arg("--cached")
            .arg("test.txt")
            .arg("test2.txt")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("rm 'test.txt'
rm 'test2.txt'
");

        // Use ls-files to check the files are removed from the index.
        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::exists());
        test_git_repo.temp_dir.child("test2.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_produce_no_output_with_quiet_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");

        test_git_repo.init();
        test_git_repo.add("test.txt");
        test_git_repo.add("test2.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("rm")
            .arg("--quiet")
            .arg("test.txt")
            .arg("test2.txt")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("");

        // Use ls-files to check the files are removed from the index.
        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("test2.txt").assert(predicate::path::missing());
    }

    #[test]
    fn should_just_write_output_with_dry_run_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");

        test_git_repo.init();
        test_git_repo.add("test.txt");
        test_git_repo.add("test2.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("rm")
            .arg("--dry-run")
            .arg("test.txt")
            .arg("test2.txt")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("rm 'test.txt'
rm 'test2.txt'
");

        // Use ls-files to check the files are not removed from the index.
        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test.txt
test2.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::exists());
        test_git_repo.temp_dir.child("test2.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_fail_if_no_matches_for_input() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");

        test_git_repo.init();
        test_git_repo.add("test.txt");
        test_git_repo.add("test2.txt");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("rm")
        .arg("test3.txt")
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr("No files matched for rm");
    }

    #[test]
    fn should_succeed_if_no_matches_with_ignore_unmatch_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");

        test_git_repo.init();
        test_git_repo.add("test.txt");
        test_git_repo.add("test2.txt");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("rm")
        .arg("--ignore-unmatch")
        .arg("test3.txt")
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .success();
    }
}