use super::log::*;
use super::macros::*;
use fancy_regex::Regex;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Name(String);

#[derive(Debug, Clone)]
pub struct Command(String);

pub trait NewType<T, U>: Sized {
    fn new<'a>(value: U) -> Result<Self, AliasError>;
    fn get(&self) -> &T;
}

impl NewType<String, &str> for Name {
    fn get(&self) -> &String {
        &self.0
    }

    fn new<'a>(name: &str) -> Result<Self, AliasError> {
        if name.is_empty() {
            ErrorCode::InvalidName(name.to_string()).log_debug(function_name!());
            return Err(AliasError::InvalidName(name.to_string()));
        }

        let regex_pattern = "^[\\w\\d]+$";
        let regex = unwrap_or_panic_err!(
            Regex::new(&regex_pattern),
            ErrorCode::RegexParse,
            &regex_pattern
        );

        let regex_passes = regex.is_match(name).unwrap_or_else(|err| {
            ErrorCode::RegexValidationMatch(&regex, &name, err).log_debug(function_name!());
            false
        });

        if !regex_passes {
            ErrorCode::InvalidName(name.to_string()).log_debug(function_name!());
            return Err(AliasError::InvalidName(name.to_string()));
        }

        Ok(Name(name.to_string()))
    }
}

impl NewType<String, &str> for Command {
    fn get(&self) -> &String {
        &self.0
    }

    fn new<'a>(command: &'a str) -> Result<Self, AliasError> {
        Ok(Command(command.to_string()))
    }
}

impl From<&Alias> for Name {
    fn from(value: &Alias) -> Self {
        value.name.to_owned()
    }
}

impl From<&Alias> for Command {
    fn from(value: &Alias) -> Self {
        value.command.to_owned()
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Name {}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Command {}

#[derive(Clone, Debug, PartialEq)]
pub struct Alias {
    pub name: Name,
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasError {
    ParseError(String),
    InvalidName(String),
    InvalidCommand(String),
}

impl Alias {
    fn is_valid(maybe_alias: &str) -> bool {
        let regex_pattern = "^\\w+=('|\").*\\1$";
        let regex = unwrap_or_panic_err!(
            Regex::new(&regex_pattern),
            ErrorCode::RegexParse,
            &regex_pattern
        );

        regex.is_match(maybe_alias).unwrap_or_else(|err| {
            ErrorCode::RegexValidationMatch(&regex, maybe_alias, err).log_debug(function_name!());
            false
        })
    }

    pub fn from(maybe_alias: &str) -> Result<Alias, AliasError> {
        debug_value!(maybe_alias);
        if Alias::is_valid(maybe_alias) {
            let split: Vec<&str> = maybe_alias.split("=").collect();
            let name = Name::new(split[0])?;
            let command = Command::new(&split[1][1..split[1].len() - 1])?;

            return Ok(Alias { name, command });
        }

        return Err(AliasError::ParseError(maybe_alias.to_string()));
    }
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}='{}'", self.name, self.command)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::format;

    use crate::{Alias, Command, Name};

    use super::NewType;

    #[test]
    fn it_parses_valid_names() {
        Name::new("abc").unwrap();
        Name::new("123").unwrap();
        Name::new("a32").unwrap();
        Name::new("AcEf32").unwrap();
        Name::new("_a32").unwrap();
        Name::new("_a_32_").unwrap();
        Name::new("_Acse_32").unwrap();
    }

    #[test]
    fn it_fails_to_parse_invalid_names() {
        Name::new("").unwrap_err();
        Name::new(" ").unwrap_err();
        Name::new("  ").unwrap_err();
        Name::new("$abc").unwrap_err();
        Name::new("hello world").unwrap_err();
        Name::new("(hey)").unwrap_err();
        Name::new("#duh").unwrap_err();
        Name::new("_Abc\\123").unwrap_err();
    }

    #[test]
    fn it_parses_valid_commands() {
        Command::new("").unwrap();
        Command::new(" ").unwrap();
        Command::new("git").unwrap();
        Command::new("git branch").unwrap();
        Command::new("git branch -d").unwrap();
        Command::new("13$git 32jdasbranch _$jdasu").unwrap();
    }

    #[test]
    fn it_parses_valid_aliases() {
        Alias::from("g='git'").unwrap();
        Alias::from("gb='git branch'").unwrap();
        Alias::from("gba='git branch --all'").unwrap();
        Alias::from("gc='git checkout'").unwrap();
        Alias::from("a=''").unwrap();
    }

    #[test]
    fn it_fails_to_parse_invalid_aliases() {
        Alias::from("").unwrap_err();
        Alias::from("git branch --all").unwrap_err();
        Alias::from("a=").unwrap_err();
        Alias::from("bla bla='some blah'").unwrap_err();
    }

    #[test]
    fn it_gets_name_from_alias() {
        fn test_get_name_from_alias(name: &str) {
            let alias = Alias::from(format!("{}='echo hello'", name).as_str()).unwrap();
            assert_eq!(Name::from(&alias), Name::new(name).unwrap());
        }

        test_get_name_from_alias("abc");
        test_get_name_from_alias("123");
        test_get_name_from_alias("a32");
        test_get_name_from_alias("AcEf32");
        test_get_name_from_alias("_a32");
        test_get_name_from_alias("_a_32_");
        test_get_name_from_alias("_Acse_32");
    }

    #[test]
    fn it_gets_command_from_alias() {
        fn test_get_command_from_alias(command: &str) {
            let alias = Alias::from(format!("a='{}'", command).as_str()).unwrap();
            assert_eq!(Command::from(&alias), Command::new(command).unwrap());
        }

        test_get_command_from_alias("abc");
        test_get_command_from_alias("123");
        test_get_command_from_alias("a32");
        test_get_command_from_alias("AcEf32");
        test_get_command_from_alias("_a32");
        test_get_command_from_alias("_a_32_");
        test_get_command_from_alias("_Acse_32");
    }
}
