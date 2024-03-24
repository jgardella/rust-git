use std::path::Path;

use crate::{config::GitConfig, error::RustGitError};

pub(crate) struct GitRepo {
    pub(crate) config: GitConfig,
}

impl GitRepo {
    pub(crate) fn new(dir: &Path) -> Result<GitRepo, RustGitError>
    {
        let config = GitConfig::new(dir)?;

        Ok(GitRepo {
            config
        })
    }
}
