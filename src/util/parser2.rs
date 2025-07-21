use crate::{debug, debug_value, function_name, AliasError};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_till1},
    character::complete::{alphanumeric1, digit0},
    error::{Error, ErrorKind, ParseError},
    sequence::{delimited, separated_pair},
    Err as NomErr, Finish, IResult,
};

type ParserResult<'a, E = Error<&'a str>> = IResult<&'a str, &'a str, E>;

impl ParseError<&str> for AliasError {
    fn from_error_kind(input: &str, _kind: ErrorKind) -> Self {
        // We don't use the input or kind since we're tracking parser position instead
        AliasError::ParseError(input.to_string()) // Default case, though this shouldn't be reached
    }

    fn append(_input: &str, _kind: ErrorKind, other: Self) -> Self {
        // Keep the existing error variant
        other
    }
}

#[derive(Debug, PartialEq)]
pub struct Alias {
    name: String,
    command: String,
}

fn parse_quoted_content(
    parse_unquoted: impl Fn(&str) -> ParserResult,
    prase_quoted: impl Fn(&str) -> ParserResult,
) -> impl FnMut(&str) -> ParserResult {
    move |input: &str| {
        alt((
            delimited(tag("'"), &prase_quoted, tag("'")),
            delimited(tag("\""), &prase_quoted, tag("\"")),
            &parse_unquoted,
        ))(input)
    }
}

fn parse_name_unquoted(input: &str) -> ParserResult {
    take_till(|c: char| {
        c.is_ascii_whitespace()
            || matches!(
                c,
                '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | '`' | '=' | '+' | '-' | '$'
            )
    })(input)
}

fn any1(input: &str) -> ParserResult {
    if input.len() == 0 {
        return Err(NomErr::Error(Error::new(input, ErrorKind::NonEmpty)));
    }

    take_till1(|c: char| c == '\'' || c == '=')(input)
}

fn parse_name_quoted(input: &str) -> ParserResult {
    any1(input)
}

fn parse_name(input: &str) -> ParserResult {
    debug_value!(input);
    let result = delimited(tag("'"), parse_name_quoted, tag("'"))(input);

    match result {
        Ok((remaining, name)) => {
            debug_value!(remaining);
            debug_value!(name);
        }
        Err(int_err) => {
            debug_value!(int_err);
        }
    };

    parse_quoted_content(parse_name_unquoted, parse_name_quoted)(input)
}

fn parse_command_unquoted(input: &str) -> ParserResult {
    alt((alphanumeric1, digit0))(input)
}

fn parse_command_quoted(input: &str) -> ParserResult {
    any1(input)
}

fn parse_command(input: &str) -> ParserResult {
    parse_quoted_content(parse_command_unquoted, parse_command_quoted)(input)
}

pub fn tracked_separated_pair<F1, F2, F3>(
    first: F1,
    separator: F2,
    second: F3,
) -> impl Fn(&str) -> IResult<&str, (&str, &str), AliasError>
where
    F1: Fn(&str) -> IResult<&str, &str, Error<&str>>,
    F2: Fn(&str) -> IResult<&str, &str, Error<&str>>,
    F3: Fn(&str) -> IResult<&str, &str, Error<&str>>,
{
    move |input: &str| {
        let name = |i| first(i).map_err(|_| NomErr::Error(AliasError::InvalidName(i.to_string())));
        let separator = |i| {
            separator(i).map_err(|_| NomErr::Error(AliasError::MissingSeparator(i.to_string())))
        };
        let command =
            |i| second(i).map_err(|_| NomErr::Error(AliasError::InvalidCommand(i.to_string())));

        separated_pair(name, separator, command)(input)
    }
}

pub fn extract_alias(input: &str) -> Result<Alias, AliasError> {
    tracked_separated_pair(parse_name, |input: &str| tag("=")(input), parse_command)(input)
        .finish()
        .map(|(_, (name, command))| Alias {
            name: name.to_string(),
            command: command.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failing_alias() {
        extract_alias("'+'='mycommand'").unwrap_err();
        extract_alias("'+'=mycommand").unwrap_err();
        extract_alias("+='mycommand'").unwrap_err();
        extract_alias("-='mycommand'").unwrap_err();
        extract_alias("'-'='mycommand'").unwrap_err();
        extract_alias("'='='mycommand'").unwrap_err();
        extract_alias("'=='mycommand").unwrap_err();
        extract_alias("=='mycommand'").unwrap_err();
        extract_alias("==mycommand").unwrap_err();
        extract_alias("'='mycommand'").unwrap_err();
        extract_alias("'=mycommand").unwrap_err();
        extract_alias("''=mycommand").unwrap_err();
        extract_alias("''='mycommand'").unwrap_err();
        extract_alias("=mycommand").unwrap_err();
        extract_alias("='mycommand'").unwrap_err();
        extract_alias("\"='mycommand'").unwrap_err();
    }

    #[test]
    fn test_basic_alias() {
        assert_eq!(
            extract_alias("myalias=mycommand"),
            Ok(Alias {
                name: "myalias".to_string(),
                command: "mycommand".to_string(),
            })
        );
    }

    #[test]
    fn test_quoted_name() {
        assert_eq!(
            extract_alias("'my-alias'=mycommand"),
            Ok(Alias {
                name: "my-alias".to_string(),
                command: "mycommand".to_string(),
            })
        );
    }

    #[test]
    fn test_quoted_command() {
        assert_eq!(
            extract_alias("myalias='my command'"),
            Ok(Alias {
                name: "myalias".to_string(),
                command: "my command".to_string(),
            })
        );
    }

    #[test]
    fn test_escaped_quotes_in_name() {
        assert_eq!(
            extract_alias("\\''abc'\\'=hello"),
            Ok(Alias {
                name: "'abc'".to_string(),
                command: "hello".to_string(),
            })
        );
    }

    #[test]
    fn test_escaped_single_quote() {
        assert_eq!(
            extract_alias("myalias='it'\\''s a command'"),
            Ok(Alias {
                name: "myalias".to_string(),
                command: "it's a command".to_string(),
            })
        );
    }

    #[test]
    fn test_complex_escaped_quotes_in_name() {
        assert_eq!(
            extract_alias("\\''some'\\''thing'\\''a'\\'='complex command'"),
            Ok(Alias {
                name: "'some'thing'a'".to_string(),
                command: "complex command".to_string(),
            })
        );
    }

    #[test]
    fn test_ansi_c_quoting() {
        assert_eq!(
            extract_alias("myalias=$'line1\\nline2'"),
            Ok(Alias {
                name: "myalias".to_string(),
                command: "line1\\nline2".to_string(),
            })
        );
    }

    #[test]
    fn test_invalid_alias_names() {
        assert!(extract_alias("+=command").is_err());
        assert!(extract_alias("==command").is_err());
        assert!(extract_alias("-=command").is_err());
        assert!(extract_alias("'a=b'=command").is_err());
    }

    #[test]
    fn test_special_character_name() {
        assert_eq!(
            extract_alias("'$'=mycommand"),
            Ok(Alias {
                name: "$".to_string(),
                command: "mycommand".to_string(),
            })
        );
    }

    #[test]
    fn test_multiple_escaped_single_quotes() {
        assert_eq!(
            extract_alias("myalias='it'\\''s a '\\''complex'\\'' command'"),
            Ok(Alias {
                name: "myalias".to_string(),
                command: "it's a 'complex' command".to_string(),
            })
        );
    }
}
