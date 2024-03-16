#[cfg(test)]
mod integration_tests {
    use std::{io::{self, Write}, fs};

    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use assert_cmd::{Command, prelude::OutputAssertExt};

    #[test]
    fn should_create_expected_dirs() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let cmd = 
            Command::cargo_bin("rust-git")
            .unwrap()
            .arg("init")
            .current_dir(temp_dir.path())
            .unwrap();

        io::stdout().write_all(&cmd.stdout).unwrap();
        io::stderr().write_all(&cmd.stderr).unwrap();

        let git_dir = temp_dir.child(".git");
        let canonical_git_dir = fs::canonicalize(&git_dir).unwrap();
        let git_dir_display = canonical_git_dir.display();

        cmd.assert()
            .success()
            .stdout(format!("Initialized empty Git repository in {git_dir_display}\n"));

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
}
