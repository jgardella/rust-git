use std::{fmt::Display, path::PathBuf};

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};


#[derive(Args, Debug)]
#[command(about = "")]
#[command(long_about = "
")]
pub(crate) struct LsFilesArgs {

}