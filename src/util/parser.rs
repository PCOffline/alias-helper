use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while1},
    character::complete::{char, none_of},
    combinator::{map, map_res, recognize},
    error::ParseError,
    multi::{many0, many1},
    sequence::{delimited, pair, terminated},
    IResult,
};

use crate::{debug, debug_value, function_name, AliasError};

#[derive(Debug, PartialEq)]
pub struct Alias {
    name: String,
    command: String,
}

#[derive(Debug, PartialEq)]
pub enum ParserError<'a> {
    AliasError(AliasError),
    NomError(nom::Err<nom::error::Error<&'a str>>),
}

impl<'a> ParseError<&'a str> for ParserError<'a> {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        ParserError::NomError(nom::Err::Error(nom::error::Error::new(input, kind)))
    }

    fn append(input: &'a str, kind: nom::error::ErrorKind, other: Self) -> Self {
        match other {
            ParserError::AliasError(e) => ParserError::AliasError(e),
            ParserError::NomError(_) => {
                ParserError::NomError(nom::Err::Error(nom::error::Error::new(input, kind)))
            }
        }
    }
}

impl<'a> From<AliasError> for ParserError<'a> {
    fn from(error: AliasError) -> Self {
        ParserError::AliasError(error)
    }
}

fn parse_escaped_quote(input: &str) -> IResult<&str, &str> {
    alt((tag("\\'"), tag("'\\''")))(input)
}

fn parse_name_part(input: &str) -> IResult<&str, String> {
    alt((
        map(is_not("'\\"), |s: &str| s.to_string()),
        map(tag("\\'"), |_| "'".to_string()),
        map(tag("'\\''"), |_| "'".to_string()),
        map(recognize(pair(char('\\'), none_of("'"))), |s: &str| s.to_string()),
    ))(input)
}

fn parse_quoted_content(input: &str) -> IResult<&str, String> {
    map(many0(parse_name_part), |parts| parts.concat())(input)
}

fn parse_quoted_name(input: &str) -> IResult<&str, String> {
    delimited(char('\''), parse_quoted_content, char('\''))(input)
}

fn parse_unquoted_name(input: &str) -> IResult<&str, &str> {
    debug_value!(input);
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

fn parse_name<'a>(input: &'a str) -> IResult<&str, String, ParserError<'a>> {
    map_res(
        many1(alt((
            parse_quoted_name,
            map(parse_unquoted_name, |s: &str| s.to_string()),
            map(parse_escaped_quote, |s: &str| s.to_string()),
        ))),
        |parts| {
            let name = parts.concat();
            if name.contains('=') || name == "-" || name == "+" || name.is_empty() {
                Err(AliasError::InvalidName(name))
            } else {
                Ok(name)
            }
        },
    )(input)
    .map_err(|e| match e {
        nom::Err::Error(e) | nom::Err::Failure(e) => {
            nom::Err::Error(ParserError::AliasError(AliasError::ParseError(e.input.to_string())))
        }
        nom::Err::Incomplete(_) => nom::Err::Error(ParserError::AliasError(AliasError::ParseError(input.to_string()))),
    })
}

fn parse_ansi_c_quoted(input: &str) -> IResult<&str, String> {
    delimited(
        tag("$'"),
        map(
            many0(alt((
                map(is_not("'\\"), |s: &str| s.to_string()),
                map(tag("\\n"), |_| "\\n".to_string()),
                map(tag("\\t"), |_| "\\t".to_string()),
                map(recognize(pair(char('\\'), none_of("nt"))), |s: &str| {
                    s.to_string()
                }),
            ))),
            |parts| parts.concat(),
        ),
        char('\''),
    )(input)
}

fn parse_command_content(input: &str) -> IResult<&str, String> {
    map(
        many1(alt((
            map(is_not("'"), |s: &str| s.to_string()),
            map(parse_escaped_quote, |_| "'".to_string()),
        ))),
        |parts| parts.concat(),
    )(input)
}

fn parse_unquoted_command(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace())(input)
}

fn parse_command<'a>(input: &'a str) -> IResult<&str, String, ParserError<'a>> {
    alt((
        map_res(parse_ansi_c_quoted, |s| {
            Ok::<String, AliasError>(s.to_string())
        }),
        map_res(
            delimited(char('\''), parse_command_content, char('\'')),
            |s| Ok::<String, AliasError>(s.to_string()),
        ),
        map_res(parse_unquoted_command, |s| {
            Ok::<String, AliasError>(s.to_string())
        }),
    ))(input)
    .map_err(|_| AliasError::InvalidCommand(input.to_string()))
    .map_err(|e| nom::Err::Error(ParserError::from(e)))
}

fn parse_alias(input: &str) -> Result<Alias, AliasError> {
    map(
        pair(terminated(parse_name, char('=')), parse_command),
        |(name, command)| Alias { name, command },
    )(input)
    .map(|(_, alias)| alias)
    .map_err(|e| match e {
        nom::Err::Error(ParserError::AliasError(e)) => e,
        _ => AliasError::ParseError(input.to_string()),
    })
}

pub fn extract_alias(input: &str) -> Result<Alias, AliasError> {
    parse_alias(input)
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
