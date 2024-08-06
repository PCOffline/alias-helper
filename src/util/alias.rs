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
    use crate::Alias;

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
}
