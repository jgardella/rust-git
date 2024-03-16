#[cfg(test)]
mod integration_tests {
    use std::fs;

    use assert_fs::{prelude::*, fixture::ChildPath};
    use predicates::prelude::*;
    use assert_cmd::{Command, prelude::OutputAssertExt};

    fn assert_git_init_files(git_dir: &ChildPath) {
        git_dir.assert(predicate::path::exists());

        let objects_dir = git_dir.child("objects");
        objects_dir.assert(predicate::path::exists());

        objects_dir
            .child("info")
            .assert(predicate::path::exists());

        objects_dir
            .child("pack")
            .assert(predicate::path::exists());
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

        assert_git_init_files(&git_dir);
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
        assert_git_init_files(&git_dir);
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
        assert_git_init_files(&git_dir);
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
        assert_git_init_files(&git_dir);
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

        assert_git_init_files(&temp_dir.child("."));
    }

    #[test]
    fn should_fail_if_already_exists() {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        Command::cargo_bin("rust-git")
        .unwrap()
        .arg("init")
        .assert()
        .failure();
    }
}
