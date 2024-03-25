#[cfg(test)]
mod integration_tests {

    use assert_cmd::{Command, prelude::OutputAssertExt};
    use test_helpers::TestGitRepo;

    #[test]
    fn should_return_content_for_object_and_type() {
        let test_git_repo = TestGitRepo::init();

        let content = "test";
        let obj_id = test_git_repo.hash_object(content);

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("cat-file")
            .arg("blob")
            .arg(obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout(format!("{content}"));
    }

    #[test]
    fn should_return_content_for_object_with_print_flag() {
        let test_git_repo = TestGitRepo::init();

        let content = "test";
        let obj_id = test_git_repo.hash_object(content);

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("cat-file")
            .arg("-p")
            .arg(obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout(format!("{content}"));
    }

    #[test]
    fn should_return_type_for_object_with_type_flag() {
        let test_git_repo = TestGitRepo::init();

        let content = "test";
        let obj_id = test_git_repo.hash_object(content);

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("cat-file")
            .arg("-t")
            .arg(obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout(format!("blob"));
    }


    #[test]
    fn should_return_size_for_object_with_size_flag() {
        let test_git_repo = TestGitRepo::init();

        let content = "test";
        let obj_id = test_git_repo.hash_object(content);

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("cat-file")
            .arg("-s")
            .arg(obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout(format!("4"));
    }

    #[test]
    fn should_return_info_for_multiple_objects_in_batch_mode() {
        let test_git_repo = TestGitRepo::init();

        let content1 = "test1";
        let obj_id1 = test_git_repo.hash_object(content1);

        let content2 = "test2";
        let obj_id2 = test_git_repo.hash_object(content2);

        let content3 = "test3";
        let obj_id3 = test_git_repo.hash_object(content3);

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("cat-file")
            .arg("--batch")
            .write_stdin(format!("{obj_id1}\n{obj_id2}\n{obj_id3}"))
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success()
            .stdout(format!("{obj_id1} blob 5\n{content1}\n\n{obj_id2} blob 5\n{content2}\n\n{obj_id3} blob 5\n{content3}\n\n"));
    }

    #[test]
    fn should_return_success_exit_code_for_object_with_check_flag() {
        let test_git_repo = TestGitRepo::init();

        let content = "test";
        let obj_id = test_git_repo.hash_object(content);

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("cat-file")
            .arg("-e")
            .arg(obj_id)
            .current_dir(test_git_repo.temp_dir.path())
            .unwrap();

        cmd.assert()
            .success();
    }

    #[test]
    fn should_return_failure_status_code_with_check_flag_and_missing_object() {
        let test_git_repo = TestGitRepo::init();
        let obj_id = "not-an-object-id";

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("cat-file")
        .arg("-e")
        .arg(obj_id)
        .current_dir(test_git_repo.temp_dir.path())
        .assert()
        .failure()
        .stderr(format!("object {obj_id} not found"));
    }
 }
