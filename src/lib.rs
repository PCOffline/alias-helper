use core::fmt;

use fancy_regex::Regex;

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
        let regex = Regex::new("^\\w+=('|\").*\\1$").expect("Failed to parse regex");
        regex
            .is_match(maybe_alias)
            .expect("Failed to validate regex")
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
    let needle = command.split_whitespace().collect::<Vec<&str>>()[0];

    match aliases.iter().find(|alias| alias.name == needle) {
        Some(candidate) => command.replace(needle, &expand_command(aliases, &candidate.command)),
        None => command.to_string(),
    }
}

/// Takes a list of aliases and returns the most matching one
pub fn find_alias<'a>(haystack: &'a Vec<Alias>, needle: &str) -> Vec<Alias> {
    if haystack.len() == 0 {
        return vec![];
    }

    let command: Vec<&str> = needle.split_whitespace().collect();

    let aliases = haystack.iter().map(|alias| Alias {
        name: alias.name.to_string(),
        command: expand_command(&haystack, &alias.command),
    }).collect();

    let mut command = expand_command(&aliases, &command.join(" "));

    loop {
        let matches: Vec<Alias> = aliases
            .iter()
            .filter(|candidate| candidate.command == command)
            .map(|candidate| candidate.to_owned())
            .collect();

        if matches.len() == 0 {
            command.pop();
        } else {
            break matches;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{expand_command, find_alias, Alias};

    #[test]
    fn it_recognizes_valid_aliases() {
        Alias::from("g='git'").unwrap();
        Alias::from("gb='git branch'").unwrap();
        Alias::from("gba='git branch --all'").unwrap();
        Alias::from("gc='git checkout'").unwrap();
        Alias::from("a=''").unwrap();
    }

    #[test]
    fn it_recognizes_invalid_aliases() {
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
    fn it_matches_nothing_if_no_alias_is_found() {
        let aliases: Vec<Alias> = vec![];

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
