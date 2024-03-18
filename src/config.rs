use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct GitConfig {
    pub(crate) core: Option<CoreConfig>,
    pub(crate) extensions: Option<ExtensionsConfig>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct CoreConfig {
    pub(crate) repositoryformatversion: Option<u32>,
    pub(crate) filemode: Option<bool>,
    pub(crate) bare: Option<bool>,
    pub(crate) legalrefupdates: Option<bool>,
    pub(crate) ignorecase: Option<bool>,
    pub(crate) precomposeunicode: Option<bool>,
    pub(crate) symlinks: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct ExtensionsConfig {
    pub(crate) objectformat: Option<String>,
}
