#[cfg(test)]
mod integration_tests {
    use std::fs;

    use assert_fs::{prelude::*, fixture::ChildPath};
    use predicates::prelude::*;
    use assert_cmd::{Command, prelude::OutputAssertExt};

    fn assert_git_init_files(git_dir: &ChildPath, expected_head: Option<&str>) {
        let expected_head = format!("ref: refs/heads/{}", expected_head.unwrap_or("main"));
        git_dir.assert(predicate::path::exists());

        let objects_dir = git_dir.child("objects");
        objects_dir.assert(predicate::path::exists());

        objects_dir
            .child("info")
            .assert(predicate::path::exists());

        objects_dir
            .child("pack")
            .assert(predicate::path::exists());

        let info_dir = git_dir.child("info");
        info_dir.assert(predicate::path::exists());

        let hooks_dir = git_dir.child("hooks");
        hooks_dir.assert(predicate::path::exists());

        let refs_dir = git_dir.child("refs");
        refs_dir.assert(predicate::path::exists());

        refs_dir
            .child("heads")
            .assert(predicate::path::exists());

        refs_dir
            .child("tags")
            .assert(predicate::path::exists());

        let head_file = git_dir.child("HEAD");
        head_file.assert(expected_head);

        let config_file = git_dir.child("config");
        // TODO: can we test more here?
        // Some of the contents of the config file may vary between systems,
        // but some should be consistent.
        config_file.assert(predicate::path::exists());

    }

    #[test]
    fn should_create_expected_dirs() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("init")
            .current_dir(temp_dir.path())
            .unwrap();

        let git_dir = temp_dir.child(".git");
        let canonical_git_dir = fs::canonicalize(&git_dir).unwrap();
        let git_dir_display = canonical_git_dir.display();

        cmd.assert()
            .success()
            .stdout(format!("Initialized empty Git repository in {git_dir_display}\n"));

        assert_git_init_files(&git_dir, None);
    }

    #[test]
    fn should_create_expected_files_with_custom_initial_branch() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("init")
            .arg("-b=my-branch")
            .current_dir(temp_dir.path())
            .unwrap();

        let git_dir = temp_dir.child(".git");
        let canonical_git_dir = fs::canonicalize(&git_dir).unwrap();
        let git_dir_display = canonical_git_dir.display();

        cmd.assert()
            .success()
            .stdout(format!("Initialized empty Git repository in {git_dir_display}\n"));

        assert_git_init_files(&git_dir, Some("my-branch"));
    }

    #[test]
    fn should_create_expected_dirs_with_directory_specified() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("init")
            .arg("my_project")
            .current_dir(temp_dir.path())
            .unwrap();

        let project_dir = temp_dir.child("my_project");
        let git_dir = project_dir.child(".git");
        let canonical_git_dir = fs::canonicalize(&git_dir).unwrap();
        let git_dir_display = canonical_git_dir.display();

        cmd.assert()
            .success()
            .stdout(format!("Initialized empty Git repository in {git_dir_display}\n"));

        project_dir.assert(predicate::path::exists());
        assert_git_init_files(&git_dir, None);
    }

    #[test]
    fn should_create_expected_dirs_with_directory_specified_nested() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("init")
            .arg("my/great/project")
            .current_dir(temp_dir.path())
            .unwrap();

        let project_dir = temp_dir.child("my/great/project");
        let git_dir = project_dir.child(".git");
        let canonical_git_dir = fs::canonicalize(&git_dir).unwrap();
        let git_dir_display = canonical_git_dir.display();

        cmd.assert()
            .success()
            .stdout(format!("Initialized empty Git repository in {git_dir_display}\n"));

        project_dir.assert(predicate::path::exists());
        assert_git_init_files(&git_dir, None);
    }

    #[test]
    fn should_create_expected_dirs_when_already_exists() {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("my_project")).unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("init")
            .arg("my_project")
            .current_dir(temp_dir.path())
            .unwrap();

        let project_dir = temp_dir.child("my_project");
        let git_dir = project_dir.child(".git");
        let canonical_git_dir = fs::canonicalize(&git_dir).unwrap();
        let git_dir_display = canonical_git_dir.display();

        cmd.assert()
            .success()
            .stdout(format!("Initialized empty Git repository in {git_dir_display}\n"));

        project_dir.assert(predicate::path::exists());
        assert_git_init_files(&git_dir, None);
    }

    #[test]
    fn should_create_expected_dirs_with_bare() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("init")
            .arg("--bare")
            .current_dir(temp_dir.path())
            .unwrap();

        let canonical_git_dir = fs::canonicalize(&temp_dir).unwrap();
        let git_dir_display = canonical_git_dir.display();

        cmd.assert()
            .success()
            .stdout(format!("Initialized empty Git repository in {git_dir_display}\n"));

        assert_git_init_files(&temp_dir.child("."), None);
    }

    #[test]
    fn should_create_expected_dirs_with_custom_git_dir() {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let custom_git_dir = temp_dir.child(".my-custom-git-dir");

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("--git-dir=.my-custom-git-dir")
            .arg("init")
            .current_dir(temp_dir.path())
            .unwrap();

        let canonical_git_dir = fs::canonicalize(&custom_git_dir).unwrap();
        let git_dir_display = canonical_git_dir.display();

        cmd.assert()
            .success()
            .stdout(format!("Initialized empty Git repository in {git_dir_display}\n"));

        assert_git_init_files(&custom_git_dir, None);
    }

    #[test]
    fn should_fail_if_already_exists() {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .failure();
    }

    #[test]
    fn should_fail_if_separate_git_repository_provided() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("--separate-git-repository=../test")
        .assert()
        .failure();
    }

    #[test]
    fn should_fail_if_template_provided() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("--template=test")
        .assert()
        .failure();
    }

    #[test]
    fn should_fail_if_shared_provided_with_non_default() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("--shared=umask")
        .assert()
        .failure();
    }

    #[test]
    fn should_fail_if_work_tree_set_without_git_dir() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .current_dir(temp_dir.path())
        .arg("--work-tree ./test")
        .arg("init")
        .assert()
        .failure();
    }

    #[test]
    fn should_fail_if_bare_and_work_tree_set() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .current_dir(temp_dir.path())
        .arg("--work-tree test")
        .arg("init")
        .arg("--bare")
        .assert()
        .failure();
    }
}
