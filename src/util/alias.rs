use super::log::*;
use super::macros::*;
use super::parser;
use super::parser2;
use fancy_regex::Regex;
use nom::error::ErrorKind;
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

        let regex_pattern = "^[^\t \u{00a0}\n]+$";
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
    MissingSeparator(String)
}

// impl ParseError<&str> for AliasError {
//     fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
//         AliasError::ParseError(input.to_string())
//     }
//     fn append(input: &str, kind: ErrorKind, other: Self) -> Self {
//         todo!();
//     }
//     fn from_char(input: &str, _: char) -> Self {
//         todo!();
//     }
//     fn or(self, other: Self) -> Self {
//         todo!();
//     }
// }

impl Alias {
    fn is_valid(maybe_alias: &str) -> bool {
        // let regex_pattern = "^(?:(?:[^\"'=]|\\\\.)*|(?:'(?:[^'\\\\]|\\\\.)*')|(?:\"(?:[^\"\\\\]|\\\\.)*\"))=(?:(?:[^\"'\\\\\\s]|\\\\.)*|(?:'(?:[^'\\\\]|\\\\.)*')|(?:\"(?:[^\"\\\\]|\\\\.)*\"))$";
        // let regex = unwrap_or_panic_err!(
        //     Regex::new(&regex_pattern),
        //     ErrorCode::RegexParse,
        //     &regex_pattern
        // );

        // regex.is_match(maybe_alias).unwrap_or_else(|err| {
        //     ErrorCode::RegexValidationMatch(&regex, maybe_alias, err).log_debug(function_name!());
        //     false
        // })

        false
    }

    pub fn from(maybe_alias: &str) -> Result<Alias, AliasError> {
        debug_value!(maybe_alias);
        if Alias::is_valid(maybe_alias) {
            let split: Vec<&str> = maybe_alias.split("=").collect();
            debug_value!(split);
            let name = Name::new(split[0])?;
            let command = Command::new(&split[1][1..split[1].len() - 1])?;

            return Ok(Alias { name, command });
        }

        return Err(AliasError::ParseError(maybe_alias.to_string()));
    }

    pub fn new(name: Name, command: Command) -> Alias {
        return Alias { name, command };
    }
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}='{}'", self.name, self.command)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        util::test::{
            get_invalid_alias_strings, get_invalid_name_strings, get_valid_alias_strings,
            get_valid_command_strings, get_valid_name_strings,
        },
        Alias, Command, Name,
    };

    use super::NewType;

    #[test]
    fn it_parses_valid_names() {
        get_valid_name_strings(None).iter().for_each(|name| {
            Name::new(&name).unwrap();
        });
    }

    #[test]
    fn it_fails_to_parse_invalid_names() {
        get_invalid_name_strings(None).iter().for_each(|name| {
            Name::new(&name).unwrap_err();
        });
    }

    #[test]
    fn it_parses_valid_commands() {
        get_valid_command_strings(None).iter().for_each(|command| {
            Command::new(&command).unwrap();
        });
    }

    #[test]
    fn it_parses_valid_aliases() {
        get_valid_alias_strings(None).iter().for_each(|alias| {
            Alias::from(&alias).unwrap();
        });
    }

    #[test]
    fn it_fails_to_parse_invalid_aliases() {
        get_invalid_alias_strings(None).iter().for_each(|alias| {
            Alias::from(&alias).unwrap_err();
        });
    }

    #[test]
    fn it_gets_name_from_alias() {
        fn test_get_name_from_alias(name: &str) {
            let alias = Alias::from(
                format!("{}='{}'", name, get_valid_command_strings(Some(1))[0]).as_str(),
            )
            .unwrap();
            assert_eq!(Name::from(&alias), Name::new(name).unwrap());
        }

        get_valid_name_strings(None)
            .iter()
            .for_each(|name| test_get_name_from_alias(name));
    }

    #[test]
    fn it_gets_command_from_alias() {
        fn test_get_command_from_alias(command: &str) {
            let alias = Alias::from(
                format!("{}='{}'", get_valid_name_strings(Some(1))[0], command).as_str(),
            )
            .unwrap();
            assert_eq!(Command::from(&alias), Command::new(command).unwrap());
        }

        get_valid_command_strings(None)
            .iter()
            .for_each(|command| test_get_command_from_alias(command));
    }
}
