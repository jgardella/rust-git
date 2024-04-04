mod integration_tests {
    use std::{fs::File, os::unix::fs::MetadataExt};

    use assert_cmd::{assert::OutputAssertExt, Command};
    use test_helpers::{TempDirExt, TestGitRepo};

    #[test]
    fn should_show_basic_details_for_staged_files() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test_in_dir.txt", b"test_in_dir");

        test_git_repo.init();
        test_git_repo.add("test.txt test2.txt test_dir");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("ls-files")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
           .success()
           .stdout("test.txt
test2.txt
test_dir/test_in_dir.txt
");
    }

    #[test]
    fn should_show_additional_details_with_stage_flag() {
        let test_git_repo = TestGitRepo::new();
        test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");
        test_git_repo.temp_dir.create_test_dir("test_dir");
        test_git_repo.temp_dir.create_test_file("test_dir/test_in_dir.txt", b"test_in_dir");

        test_git_repo.init();
        test_git_repo.add("test.txt test2.txt test_dir");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("ls-files")
            .arg("-s")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
           .success()
           .stdout("100644 30d74d258442c7c65512eafab474568dd706c430 0\ttest.txt
100644 d606037cb232bfda7788a8322492312d55b2ae9d 0\ttest2.txt
100644 27961cd8c28d3fd780672e10e8c7c25d6eee10ee 0\ttest_dir/test_in_dir.txt
");
    }

    fn get_expected_debug_output(file: &File) -> String {
        let metadata = file.metadata().unwrap();
        return format!("  ctime: {}:{}
  mtime: {}:{}
  dev: {}\tino: {}
  uid: {}\tgid: {}
  size: {}\tflags: 0",
  metadata.ctime(), metadata.ctime_nsec(),
  metadata.mtime(), metadata.mtime_nsec(),
  metadata.dev(), metadata.ino(),
  metadata.uid(), metadata.gid(),
  metadata.size())
}

    #[test]
    fn should_show_additional_details_with_debug_flag() {
        let test_git_repo = TestGitRepo::new();
        let test_file1 = test_git_repo.temp_dir.create_test_file("test.txt", b"test");
        let test_file2 = test_git_repo.temp_dir.create_test_file("test2.txt", b"test2");
        test_git_repo.temp_dir.create_test_dir("test_dir");
        let test_file3 = test_git_repo.temp_dir.create_test_file("test_dir/test_in_dir.txt", b"test_in_dir");

        test_git_repo.init();
        test_git_repo.add("test.txt test2.txt test_dir");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("ls-files")
            .arg("--debug")
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
           .success()
           .stdout(format!("test.txt
{}
test2.txt
{}
test_dir/test_in_dir.txt
{}
", get_expected_debug_output(&test_file1),
get_expected_debug_output(&test_file2),
get_expected_debug_output(&test_file3)));
    }


}