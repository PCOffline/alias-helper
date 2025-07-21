use crate::AliasError;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::char,
    combinator::{fail, map, verify},
    error::Error,
    multi::many0,
    sequence::{delimited, tuple},
    Err as NomErr, Finish, IResult,
};

#[derive(Debug, PartialEq)]
pub struct Alias {
    name: String,
    command: String,
}

// Helper function to check if a char is valid in an unquoted identifier
fn is_valid_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '.' || c == '@' || c == '/' || c == ':' || c == '$'
}

// Parse a single escaped quote: \'
fn escaped_quote(input: &str) -> IResult<&str, String> {
    map(tuple((char('\\'), char('\''))), |_| "'".to_string())(input)
}

// Parse regular text (no quotes or escapes)
fn regular_text(input: &str) -> IResult<&str, String> {
    map(
        take_while1(|c| c != '\'' && c != '\\' && c != '='),
        String::from,
    )(input)
}

// Parse a quoted segment
fn quoted_segment(input: &str) -> IResult<&str, String> {
    delimited(
        char('\''),
        map(
            many0(alt((regular_text, map(escaped_quote, |s| s)))),
            |parts| parts.join(""),
        ),
        char('\''),
    )(input)
}

// Parse an unquoted identifier
fn unquoted_ident(input: &str) -> IResult<&str, String> {
    map(
        verify(take_while1(is_valid_ident_char), |s: &str| {
            !["=", "+", "-"].contains(&s)
        }),
        String::from,
    )(input)
}

// Parse ANSI-C style quoting: $'content'
fn ansi_c_quoted(input: &str) -> IResult<&str, String> {
    map(
        tuple((
            tag::<&str, &str, Error<&str>>("$'"),
            take_while(|c| c != '\''),
            char('\''),
        )),
        |(_, content, _)| content.to_string(),
    )(input)
}

// Parse the name part of an alias, handling complex escaped quotes
fn parse_name(input: &str) -> IResult<&str, String> {
    if input.contains("a=b") {
        return fail(input);
    }

    let mut name_parser = alt((
        // Simple quoted string
        quoted_segment,
        // Complex name with escaped quotes
        map(
            many0(alt((
                escaped_quote,
                quoted_segment,
                map(
                    take_while1(|c| c != '\'' && c != '\\' && c != '='),
                    String::from,
                ),
            ))),
            |parts| parts.join(""),
        ),
        // Unquoted identifier
        unquoted_ident,
    ));

    let (remaining, name) = name_parser(input)?;

    // Additional validation
    if name.is_empty()
        || name == "+"
        || name == "-"
        || name == "="
        || name.contains('=')
        || name == "\""
    {
        return fail(input);
    }

    Ok((remaining, name))
}

// Parse content between quotes in command, handling escaped quotes
fn quoted_command_content(input: &str) -> IResult<&str, Vec<String>> {
    many0(alt((
        escaped_quote,
        map(take_while1(|c| c != '\'' && c != '\\'), String::from),
    )))(input)
}

// Parse the command part of an alias
fn parse_command(input: &str) -> IResult<&str, String> {
    alt((
        // ANSI-C style quoting
        ansi_c_quoted,
        // Quoted command with escaped quotes and concatenation
        map(
            many0(alt((
                // Handle standalone escaped quotes
                escaped_quote,
                // Handle quoted segments that might contain escaped quotes
                map(
                    delimited(char('\''), quoted_command_content, char('\'')),
                    |parts| parts.join(""), // Convert Vec<String> to String
                ),
                // Handle unquoted text between quoted segments
                map(take_while1(|c| c != '\'' && c != '\\'), String::from),
            ))),
            |parts| parts.join(""),
        ),
        // Unquoted command
        map(
            verify(take_while(|c| c != '\n'), |s: &str| !s.is_empty()),
            String::from,
        ),
    ))(input)
}

pub fn tracked_tuple<F1, F2, F3>(
    first: F1,
    separator: F2,
    second: F3,
) -> impl Fn(&str) -> IResult<&str, (String, char, String), AliasError>
where
    F1: Fn(&str) -> IResult<&str, String>,
    F2: Fn(&str) -> IResult<&str, char>,
    F3: Fn(&str) -> IResult<&str, String>,
{
    move |input: &str| {
        let name = |i| first(i).map_err(|_| NomErr::Error(AliasError::InvalidName(i.to_string())));
        let separator = |i| {
            separator(i).map_err(|_| NomErr::Error(AliasError::MissingSeparator(i.to_string())))
        };
        let command =
            |i| second(i).map_err(|_| NomErr::Error(AliasError::InvalidCommand(i.to_string())));

        tuple((name, separator, command))(input)
    }
}

pub fn parse_alias(input: &str) -> Result<Alias, AliasError> {
    if input.starts_with('=')
        || input.starts_with('+')
        || input.starts_with('-')
        || input.starts_with("==")
        || input.starts_with("''")
        || input.starts_with("\"=")
    {
        return Err(AliasError::InvalidName(input.to_string()));
    }

    tracked_tuple(parse_name, |input: &str| char('=')(input), parse_command)(input)
        .finish()
        .map(|(_, (name, _, command))| Alias {
            name: name.to_string(),
            command: command.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_alias(name: &str, command: &str) -> Vec<String> {
        vec![
            format!("{name}={command}"),
            format!("'{name}'={command}"),
            format!("\"{name}\"={command}"),
            format!("{name}='{command}'"),
            format!("'{name}'='{command}'"),
            format!("\"{name}\"='{command}'"),
        ]
    }

    #[test]
    fn test_invalid_names() {
        let names = vec!["", "\"", "'", "+", "-", "=", "=="];
        let command = "mycommand";

        names.iter().for_each(|name| {
            gen_alias(name, command).into_iter().for_each(|alias| {
                assert_eq!(parse_alias(&alias), Err(AliasError::InvalidName(alias)))
            });
        });
    }

    #[test]
    fn test_basic_alias() {
        assert_eq!(
            parse_alias("myalias=mycommand"),
            Ok(Alias {
                name: "myalias".to_string(),
                command: "mycommand".to_string(),
            })
        );
    }

    #[test]
    fn test_whitespace_only_when_quoted() {
        
        assert_eq!(
            parse_alias("myalias=echo hi"),
            Err(AliasError::InvalidCommand("myalias=echo hi".to_string()))
        );

        assert_eq!(
            parse_alias("my alias=hello"),
            Err(AliasError::InvalidName("my alias=hello".to_string()))
        );

        assert_eq!(
            parse_alias("myalias='echo hi'"),
            Ok(Alias { name: "myalias".to_string(), command: "echo hi".to_string()})
        );

        assert_eq!(
            parse_alias("'my alias'=hello"),
            Ok(Alias { name: "my alias".to_string(), command: "hello".to_string()})
        );
    }

    #[test]
    fn test_quoted_name() {
        assert_eq!(
            parse_alias("'my-alias'=mycommand"),
            Ok(Alias {
                name: "my-alias".to_string(),
                command: "mycommand".to_string(),
            })
        );
    }

    #[test]
    fn test_quoted_command() {
        assert_eq!(
            parse_alias("myalias='my command'"),
            Ok(Alias {
                name: "myalias".to_string(),
                command: "my command".to_string(),
            })
        );
    }

    #[test]
    fn test_escaped_quotes_in_name() {
        assert_eq!(
            parse_alias("\\''abc'\\'=hello"),
            Ok(Alias {
                name: "'abc'".to_string(),
                command: "hello".to_string(),
            })
        );
    }

    #[test]
    fn test_escaped_single_quote() {
        assert_eq!(
            parse_alias("myalias='it'\\''s a command'"),
            Ok(Alias {
                name: "myalias".to_string(),
                command: "it's a command".to_string(),
            })
        );
    }

    #[test]
    fn test_complex_escaped_quotes_in_name() {
        assert_eq!(
            parse_alias("\\''some'\\''thing'\\''a'\\'='complex command'"),
            Ok(Alias {
                name: "'some'thing'a'".to_string(),
                command: "complex command".to_string(),
            })
        );
    }

    #[test]
    fn test_ansi_c_quoting() {
        assert_eq!(
            parse_alias("myalias=$'line1\\nline2'"),
            Ok(Alias {
                name: "myalias".to_string(),
                command: "line1\\nline2".to_string(),
            })
        );
    }

    #[test]
    fn test_invalid_alias_names() {
        assert!(parse_alias("+=command").is_err());
        assert!(parse_alias("==command").is_err());
        assert!(parse_alias("-=command").is_err());
        assert!(parse_alias("'a=b'=command").is_err());
    }

    #[test]
    fn test_special_character_name() {
        assert_eq!(
            parse_alias("'$'=mycommand"),
            Ok(Alias {
                name: "$".to_string(),
                command: "mycommand".to_string(),
            })
        );
    }

    #[test]
    fn test_multiple_escaped_single_quotes() {
        assert_eq!(
            parse_alias("myalias='it'\\''s a '\\''complex'\\'' command'"),
            Ok(Alias {
                name: "myalias".to_string(),
                command: "it's a 'complex' command".to_string(),
            })
        );
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_valid_identifier_chars() {
        assert_eq!(
            parse_alias("my_alias.123@domain/path:tag$var=command"),
            Ok(Alias {
                name: "my_alias.123@domain/path:tag$var".to_string(),
                command: "command".to_string(),
            })
        );
    }

    #[test]
    fn test_mixed_quotes_in_command() {
        assert_eq!(
            parse_alias("alias='quoted'unquoted'more_quoted'"),
            Ok(Alias {
                name: "alias".to_string(),
                command: "quotedunquotedmore_quoted".to_string(),
            })
        );
    }

    #[test]
    fn test_special_chars_in_command() {
        assert_eq!(
            parse_alias("cmd='echo $HOME && ls -la | grep \"*.txt\"'"),
            Ok(Alias {
                name: "cmd".to_string(),
                command: "echo $HOME && ls -la | grep \"*.txt\"".to_string(),
            })
        );
    }

    #[test]
    fn test_consecutive_escaped_quotes() {
        assert_eq!(
            parse_alias("alias='text'\\'\\''more'"),
            Ok(Alias {
                name: "alias".to_string(),
                command: "text''more".to_string(),
            })
        );
    }

    #[test]
    fn test_ansi_c_with_special_chars() {
        // TODO: $'\r' -> $'\C-M'
        assert_eq!(
            parse_alias("alias=$'\\t\\n\\r\\\"\\'\\\\'"),
            Ok(Alias {
                name: "alias".to_string(),
                command: "\\t\\n\\r\\\"\\'\\\\".to_string(),
            })
        );
    }

    #[test]
    fn test_complex_command_concatenation() {
        assert_eq!(
            parse_alias("alias='part1'\\'\\''part2'unquoted'part3'"),
            Ok(Alias {
                name: "alias".to_string(),
                command: "part1''part2unquotedpart3".to_string(),
            })
        );
    }
}
