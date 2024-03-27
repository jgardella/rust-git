use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ChmodFlag {
    Executable,
    NonExecutable,
}

impl FromStr for ChmodFlag {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+x" => Ok(ChmodFlag::Executable),
            "-x" => Ok(ChmodFlag::NonExecutable),
            other => {
                Err(String::from(format!("chmod option '{other}' must be either -x or +x")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chmod_flag()
    {
        assert_eq!("+x".parse(), Ok(ChmodFlag::Executable));
        assert_eq!("-x".parse(), Ok(ChmodFlag::NonExecutable));
        assert_eq!("invalid".parse::<ChmodFlag>(), Err(String::from("chmod option 'invalid' must be either -x or +x")));
    }
}
