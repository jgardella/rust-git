use std::{fmt::Display, str::FromStr};

use crate::error::RustGitError;
use clap::ValueEnum;
use itertools::Itertools;

#[derive(Clone, PartialEq, ValueEnum)]
pub(crate) enum GitIdentityType {
    Author,
    Committer,
    Tagger,
}

impl FromStr for GitIdentityType {
    type Err = RustGitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "author" => Ok(Self::Author),
            "committer" => Ok(Self::Committer),
            "tagger" => Ok(Self::Tagger),
            other => Err(RustGitError::new(format!(
                "invalid identity type '{other}'"
            ))),
        }
    }
}

impl Display for GitIdentityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Author => "author",
            Self::Committer => "committer",
            Self::Tagger => "tagger",
        };

        write!(f, "{}", s)
    }
}

pub(crate) struct GitIdentity {
    pub(crate) identity_type: GitIdentityType,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) timestamp: u128,
}

impl FromStr for GitIdentity {
    type Err = RustGitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut c = s.chars();

        let identity_type = c
            .peeking_take_while(|c| *c != ' ')
            .collect::<String>()
            .trim_end()
            .parse::<GitIdentityType>()?;

        let name = c
            .peeking_take_while(|c| *c != '<')
            .collect::<String>()
            .trim()
            .to_string();

        let email = c
            .peeking_take_while(|c| *c != ' ')
            .collect::<String>()
            .trim_start_matches('<')
            .trim_end_matches('>')
            .to_string();

        let remaining = c.collect::<String>().trim_start().to_string();
        let timestamp = u128::from_str(&remaining)?;

        Ok(GitIdentity {
            identity_type,
            name,
            email,
            timestamp,
        })
    }
}
