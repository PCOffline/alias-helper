use core::fmt;
use fancy_regex::Regex;

mod util;
pub use util::log::init as init_logger;
use util::{
    log::*,
    macros::{self, debug_value, function_name},
    validation,
};
pub use util::log;

#[derive(Clone, Debug, PartialEq)]
pub struct Alias {
    pub name: String,
    pub command: String,
}

#[derive(Debug)]
pub enum AliasError {
    ParseError,
}

impl Alias {
    pub fn is_valid(maybe_alias: &str) -> bool {
        let regex_pattern = "^\\w+=('|\").*\\1$";
        let regex = macros::unwrap_or_log_with_err!(
            Regex::new(&regex_pattern),
            ErrorCode::RegexParse,
            &regex_pattern
        );
        regex.is_match(maybe_alias).unwrap_or_else(|err| {
            ErrorCode::RegexValidationMatch(&regex, maybe_alias, err).log(function_name!())
        })
    }

    pub fn from(maybe_alias: &str) -> Result<Alias, AliasError> {
        if Alias::is_valid(maybe_alias) {
            let split: Vec<&str> = maybe_alias.split("=").collect();
            return Ok(Alias {
                name: split[0].to_string(),
                command: split[1][1..split[1].len() - 1].to_string(),
            });
        }

        return Err(AliasError::ParseError);
    }
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}='{}'", self.name, self.command)
    }
}

fn expand_command(aliases: &Vec<Alias>, command: &str) -> String {
    debug_value!(aliases, command);
    let needle = command.split_whitespace().collect::<Vec<&str>>();
    debug_value!(needle);

    if needle.len() == 0 {
        trace!(
            "[{}] command is empty; returning empty string",
            function_name!()
        );
        return "".to_string();
    }

    let needle = needle[0];

    match aliases.iter().find(|alias| alias.name == needle) {
        Some(candidate) => {
            trace!("[{}] found candidate, expanding", function_name!());
            let expanded = expand_command(aliases, &candidate.command);
            debug_value!(expanded);
            let output = command.replace(needle, &expanded);
            debug!("[{}] returning {:?}", function_name!(), output);
            output
        }
        None => {
            trace!("[{}] nothing to expand, exiting", function_name!());
            debug_value!(command);
            command.to_string()
        }
    }
}

/// Takes a list of aliases and returns the most matching one
pub fn find_alias<'a>(haystack: &'a Vec<Alias>, needle: &str) -> Vec<Alias> {
    debug_value!(haystack, needle);

    if haystack.len() == 0 {
        trace!("[{}] haystack is empty, leaving", function_name!());
        return vec![];
    }

    let command: Vec<&str> = needle.split_whitespace().collect();
    debug!("[{}] split command", function_name!());
    debug_value!(command);

    let haystack = validation::filter_invalid_aliases(&haystack);
    debug!("[{}] filtered successfully", function_name!());
    debug_value!(haystack);

    let aliases = haystack
        .iter()
        .map(|alias| Alias {
            name: alias.name.to_string(),
            command: expand_command(&haystack, &alias.command),
        })
        .collect();
    debug_value!(aliases);

    let mut command = expand_command(&aliases, &command.join(" "));
    debug_value!(command);

    loop {
        let matches: Vec<Alias> = aliases
            .iter()
            .filter(|candidate| candidate.command == command)
            .map(|candidate| candidate.to_owned())
            .collect();

        if command.len() == 0 {
            trace!(
                "[{}] command is empty, breaking out of loop",
                function_name!()
            );
            break vec![];
        } else if matches.len() == 0 {
            trace!("[{}] no matches, trying substring", function_name!());
            debug_value!(command, matches);
            let mut temp: Vec<&str> = command.split_whitespace().collect();
            temp.pop();
            command = temp.join(" ");
            debug_value!(command);
        } else {
            trace!("[{}] found match, breaking out of loop", function_name!());
            debug_value!(command, matches);
            break matches;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{expand_command, find_alias, Alias};

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
    fn it_matches_only_the_exact_alias() {
        let aliases: Vec<Alias> = vec![
            Alias::from("g='git'").unwrap(),
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='git branch --all'").unwrap(),
        ];

        assert_eq!(
            find_alias(&aliases, "git"),
            vec![Alias::from("g='git'").unwrap()]
        );
        assert_eq!(
            find_alias(&aliases, "git branch"),
            vec![Alias::from("gb='git branch'").unwrap()]
        );
        assert_eq!(
            find_alias(&aliases, "git branch --all"),
            vec![Alias::from("gba='git branch --all'").unwrap()]
        );
    }

    #[test]
    fn it_matches_substring_when_no_exact_alias_is_found() {
        let aliases: Vec<Alias> = vec![
            Alias::from("g='git'").unwrap(),
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='git branch --all'").unwrap(),
            Alias::from("gc='git checkout'").unwrap(),
        ];

        assert_eq!(
            find_alias(&aliases, "git branch -M branch_name"),
            vec![Alias::from("gb='git branch'").unwrap()]
        );
        assert_eq!(
            find_alias(&aliases, "git checkout -b"),
            vec![Alias::from("gc='git checkout'").unwrap()]
        );
    }

    #[test]
    fn it_handles_empty_command() {
        let aliases: Vec<Alias> = vec![
            Alias::from("g='git'").unwrap(),
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='git branch --all'").unwrap(),
        ];
        assert_eq!(find_alias(&aliases, ""), vec![]);

        let aliases: Vec<Alias> = vec![];
        assert_eq!(find_alias(&aliases, ""), vec![]);

        let aliases: Vec<Alias> = vec![Alias::from("a=''").unwrap()];
        assert_eq!(find_alias(&aliases, ""), vec![]);
    }

    #[test]
    fn it_matches_nothing_if_no_alias_is_found() {
        let aliases: Vec<Alias> = vec![];

        assert_eq!(
            find_alias(&aliases, "git checkout -b"),
            vec![] as Vec<Alias>
        );

        let aliases: Vec<Alias> = vec![Alias::from("a=''").unwrap()];
        assert_eq!(
            find_alias(&aliases, "git checkout -b"),
            vec![] as Vec<Alias>
        );
    }

    #[test]
    fn it_matches_multiple_aliases() {
        let aliases: Vec<Alias> = vec![
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gitb='git branch'").unwrap(),
        ];

        assert_eq!(
            find_alias(&aliases, "git branch"),
            vec![
                Alias::from("gb='git branch'").unwrap(),
                Alias::from("gitb='git branch'").unwrap()
            ]
        );
    }

    #[test]
    fn it_expands_alias() {
        let aliases: Vec<Alias> = vec![
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gc='git checkout'").unwrap(),
        ];

        assert_eq!(expand_command(&aliases, "gb"), "git branch");
        assert_eq!(expand_command(&aliases, "gb --all"), "git branch --all");
        assert_eq!(expand_command(&aliases, "gc -b"), "git checkout -b");
    }

    #[test]
    fn it_fully_expands() {
        let aliases: Vec<Alias> = vec![
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='git branch --all'").unwrap(),
        ];

        assert_eq!(expand_command(&aliases, "gba"), "git branch --all");
        assert_eq!(
            expand_command(&aliases, "gba --some_flag"),
            "git branch --all --some_flag"
        );
        assert_eq!(expand_command(&aliases, "gb --all"), "git branch --all");
        assert_eq!(
            expand_command(&aliases, "gb --all --some_flag"),
            "git branch --all --some_flag"
        );
        assert_eq!(
            expand_command(&aliases, "git branch --all"),
            "git branch --all"
        );
        assert_eq!(
            expand_command(&aliases, "git branch --all --some_flag"),
            "git branch --all --some_flag"
        );
    }

    #[test]
    fn it_recursively_expands_alias() {
        let aliases: Vec<Alias> = vec![
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='gb --all'").unwrap(),
            Alias::from("gc='git checkout'").unwrap(),
            Alias::from("gcb='gc -b'").unwrap(),
        ];

        assert_eq!(expand_command(&aliases, "gba"), "git branch --all");
        assert_eq!(
            expand_command(&aliases, "gba --some_flag"),
            "git branch --all --some_flag"
        );
        assert_eq!(expand_command(&aliases, "gcb"), "git checkout -b");
        assert_eq!(
            expand_command(&aliases, "gcb branch_name"),
            "git checkout -b branch_name"
        );
    }

    #[test]
    fn it_leaves_alias_as_is_when_nothing_to_expand() {
        let aliases: Vec<Alias> = vec![
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='gb --all'").unwrap(),
        ];

        assert_eq!(
            expand_command(&aliases, "git branch --all"),
            "git branch --all"
        );
        assert_eq!(expand_command(&aliases, "git checkout"), "git checkout");
    }

    #[test]
    fn it_matches_alias_using_another_alias_and_expands_it() {
        let aliases: Vec<Alias> = vec![
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='gb --all'").unwrap(),
        ];

        assert_eq!(
            find_alias(&aliases, "git branch --all"),
            vec![Alias::from("gba='git branch --all'").unwrap()]
        );
    }
}
