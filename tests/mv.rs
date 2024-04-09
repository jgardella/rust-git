mod integration_tests {
    use std::fs;

    use assert_cmd::{assert::OutputAssertExt, Command};
    use assert_fs::{assert::PathAssert, fixture::PathChild};
    use predicates::prelude::predicate;
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_rename_file_in_index_and_working_directory() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("test.txt")
            .arg("renamed_test.txt")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "renamed_test.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("renamed_test.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_move_file_into_directory() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_dir("test_dir");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("test.txt")
            .arg("test_dir")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test_dir/test.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("test_dir/test.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_rename_directory() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test_dir/test.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("test_dir")
            .arg("renamed_test_dir")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "renamed_test_dir/test.txt");

        // TODO: cleanup empty source directories
        // test_git_repo.temp_dir.child("test_dir.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("renamed_test_dir/test.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_move_directory_into_directory() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test.txt", b"test");
        test_git_repo.temp_dir.create_test_dir("another_dir");

        test_git_repo.init();
        test_git_repo.add("test_dir/test.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("test_dir")
            .arg("another_dir")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "another_dir/test_dir/test.txt");

        test_git_repo.temp_dir.child("test_dir").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("another_dir/test_dir/test.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_move_multiple_inputs_into_directory() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");
        test_git_repo.temp_dir.create_test_file("test3.txt", b"test3");
        test_git_repo.temp_dir.create_test_dir("another_dir");

        test_git_repo.init();
        test_git_repo.add("test_dir/test.txt");
        test_git_repo.add("test2.txt");
        test_git_repo.add("test3.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("test_dir")
            .arg("test2.txt")
            .arg("test3.txt")
            .arg("another_dir")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "another_dir/test2.txt
another_dir/test3.txt
another_dir/test_dir/test.txt");

        test_git_repo.temp_dir.child("test_dir").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("test2.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("test3.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("another_dir/test_dir/test.txt").assert(predicate::path::exists());
        test_git_repo.temp_dir.child("another_dir/test2.txt").assert(predicate::path::exists());
        test_git_repo.temp_dir.child("another_dir/test3.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_fail_to_overwrite_file_without_force_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("renamed_test.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("mv")
        .arg("test.txt")
        .arg("renamed_test.txt")
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .failure()
        .stderr("cannot overwrite, source=\"test.txt\", destination=\"renamed_test.txt\"");
    }

    #[test]
    fn should_overwrite_file_with_force_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("renamed_test.txt", b"other_test");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("-f")
            .arg("test.txt")
            .arg("renamed_test.txt")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
           .success()
           .stdout("");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("renamed_test.txt").assert(predicate::path::exists());
        test_git_repo.temp_dir.child("renamed_test.txt").assert("test");
    }

    #[test]
    fn should_not_change_anything_with_dry_run_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("--dry-run")
            .arg("test.txt")
            .arg("renamed_test.txt")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("Checking rename of \"test.txt\" to \"renamed_test.txt\"
Renaming \"test.txt\" to \"renamed_test.txt\"
");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::exists());
        test_git_repo.temp_dir.child("renamed_test.txt").assert(predicate::path::missing());
    }

    #[test]
    fn should_include_details_with_verbose_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("--verbose")
            .arg("test.txt")
            .arg("renamed_test.txt")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("Renaming \"test.txt\" to \"renamed_test.txt\"
");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "renamed_test.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("renamed_test.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_fail_on_first_error_without_skip_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_dir("test_dir");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("mv")
        .arg("test2.txt") // File doesn't exist.
        .arg("test.txt")
        .arg("test_dir")
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .failure()
        .stderr("bad source, source=\"test2.txt\", destination=\"test_dir\"");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::exists());
        test_git_repo.temp_dir.child("test_dir/test.txt").assert(predicate::path::missing());
    }

    #[test]
    fn should_skip_errors_with_skip_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_dir("test_dir");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("-k")
            .arg("test2.txt") // File doesn't exist.
            .arg("test.txt")
            .arg("test_dir")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test_dir/test.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("test_dir/test.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_fail_moving_dir_into_self() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test_dir/test.txt");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("mv")
        .arg("test_dir")
        .arg("test_dir")
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .failure()
        .stderr("can not move directory into itself, source=\"test_dir\", destination=\"test_dir\"");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test_dir/test.txt");

        test_git_repo.temp_dir.child("test_dir/test.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_fail_moving_dir_over_file() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("other_file.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test_dir/test.txt");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("mv")
        .arg("test_dir")
        .arg("other_file.txt")
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .failure()
        .stderr("destination already exists, source=\"test_dir\", destination=\"other_file.txt\"");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test_dir/test.txt");

        test_git_repo.temp_dir.child("test_dir/test.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_fail_moving_dir_with_no_tracked_files() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test.txt", b"test");

        test_git_repo.init();

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("mv")
        .arg("test_dir")
        .arg("renamed_dir")
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .failure()
        .stderr("source directory is empty, source=\"test_dir\", destination=\"renamed_dir\"");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "");

        test_git_repo.temp_dir.child("test_dir/test.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_fail_moving_file_to_missing_dir() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("mv")
        .arg("test.txt")
        .arg("missing_dir/")
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .failure()
        .stderr("destination directory does not exist, source=\"test.txt\", destination=\"missing_dir/\"");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_fail_moving_untracked_file() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");

        test_git_repo.init();

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("mv")
        .arg("test.txt")
        .arg("test2.txt")
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .failure()
        .stderr("not under version control, source=\"test.txt\", destination=\"test2.txt\"");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_rename_file_in_index_but_removed_from_working_directory() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test.txt");

        fs::remove_file(test_git_repo.temp_dir.join("test.txt")).unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("test.txt")
            .arg("renamed_test.txt")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout("");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "renamed_test.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("renamed_test.txt").assert(predicate::path::missing());
    }

    #[test]
    fn should_fail_to_overwrite_file_in_index_without_force_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test.txt");
        test_git_repo.add("test2.txt");

        fs::remove_file(test_git_repo.temp_dir.join("test.txt")).unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("mv")
        .arg("test.txt")
        .arg("test2.txt")
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .failure()
        .stderr("destination exists, source=\"test.txt\", destination=\"test2.txt\"");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test.txt
test2.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("test2.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_overwrite_file_in_index_with_force_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test.txt");
        test_git_repo.add("test2.txt");

        fs::remove_file(test_git_repo.temp_dir.join("test.txt")).unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("-f")
            .arg("test.txt")
            .arg("test2.txt")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
           .success()
           .stdout("");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test2.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::missing());
        test_git_repo.temp_dir.child("test2.txt").assert(predicate::path::exists());
    }

    #[test]
    fn should_fail_to_move_missing_file() {
        let test_git_repo = TestGitRepo::new();

        test_git_repo.init();

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("mv")
        .arg("test.txt")
        .arg("test2.txt")
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .failure()
        .stderr("bad source, source=\"test.txt\", destination=\"test2.txt\"");

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "");
    }

    #[test]
    fn should_move_file_up_a_folder() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test_dir/test.txt");

        let cmd =
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("test.txt")
            .arg("..")
            .current_dir(test_git_repo.temp_dir.join("test_dir"))
            .unwrap();

        cmd.assert().success();

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test.txt");

        test_git_repo.temp_dir.child("test.txt").assert(predicate::path::exists());
        test_git_repo.temp_dir.child("test_dir/test.txt").assert(predicate::path::missing());
    }

    #[test]
    fn should_move_file_around_folders() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_dir("test_dir2");
        test_git_repo.temp_dir.create_test_file("test_dir/test.txt", b"test");

        test_git_repo.init();
        test_git_repo.add("test_dir/test.txt");

        let cmd =
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("mv")
            .arg("test.txt")
            .arg("../test_dir2")
            .current_dir(test_git_repo.temp_dir.join("test_dir"))
            .unwrap();

        cmd.assert().success();

        let ls_files_output = test_git_repo.ls_files();
        assert_eq!(ls_files_output, "test_dir2/test.txt");

        test_git_repo.temp_dir.child("test_dir2/test.txt").assert(predicate::path::exists());
        test_git_repo.temp_dir.child("test_dir/test.txt").assert(predicate::path::missing());
    }

    #[test]
    fn should_fail_to_mv_file_to_outside_repo() {
        let test_git_repo = TestGitRepo::new();

        test_git_repo.init_in_dir("my_proj");
        test_git_repo.temp_dir.create_test_file("my_proj/test.txt", b"test");

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("mv")
        .arg("test.txt")
        .arg("..")
        .current_dir(test_git_repo.temp_dir.join("my_proj"))
        .assert()
        .failure()
        .stderr("path \"..\" is outside of repo");
    }
}