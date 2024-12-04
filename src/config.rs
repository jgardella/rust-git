use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{commands::porcelain::init::cli::HashAlgorithm, error::RustGitError};

const CONFIG_FILE_NAME: &str = "config";

// We are using the Default trait here, which means we'll include all the fields
// in the serialized config file, even if they're set to the default.
// TODO: look into skipping default values when serializing:
// https://stackoverflow.com/questions/53900612/how-do-i-avoid-generating-json-when-serializing-a-value-that-is-null-or-a-defaul
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub(crate) struct GitConfig {
    pub(crate) core: CoreConfig,
    pub(crate) user: UserConfig,
    pub(crate) extensions: ExtensionsConfig,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub(crate) struct CoreConfig {
    pub(crate) repositoryformatversion: u32,
    pub(crate) filemode: bool,
    pub(crate) bare: bool,
    pub(crate) logallrefupdates: bool,
    pub(crate) ignorecase: bool,
    pub(crate) precomposeunicode: bool,
    pub(crate) symlinks: bool,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            repositoryformatversion: 1,
            filemode: true,
            bare: false,
            logallrefupdates: true,
            ignorecase: false,
            precomposeunicode: true,
            symlinks: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub(crate) struct ExtensionsConfig {
    pub(crate) objectformat: HashAlgorithm,
}

impl GitConfig {
    pub(crate) fn new(dir: &Path) -> Result<GitConfig, RustGitError> {
        let file_path = dir.join(CONFIG_FILE_NAME);

        if !file_path.exists() {
            return Ok(GitConfig::default());
        }

        let mut repo_config_s = String::new();
        File::open(file_path)?.read_to_string(&mut repo_config_s)?;
        let config: GitConfig = toml::de::from_str(&repo_config_s)?;
        Ok(config)
    }

    pub(crate) fn write(self, dir: &Path) -> Result<(), RustGitError> {
        let config_file_path = dir.join(CONFIG_FILE_NAME);
        let mut config_file = File::create(config_file_path)?;
        // TODO: using TOML for ease of use, but the git config format isn't TOML
        // Might need to implement a custom serde for that format.
        config_file.write_all(toml::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub(crate) struct UserConfig {
    pub(crate) name: Option<String>,
    pub(crate) email: Option<String>,
}
