mod util;
pub use util::alias::*;
pub use util::log;
pub use util::log::init as init_logger;
use util::{log::*, macros::*, validation};

fn expand_command(aliases: &Vec<Alias>, command: &Command) -> Result<Command, AliasError> {
    debug_value!(aliases, command);
    let needle = command.get().split_whitespace().collect::<Vec<&str>>();
    debug_value!(needle);

    if needle.len() == 0 {
        trace!(
            "[{}] command is empty; returning empty string",
            function_name!()
        );
        return Command::new("");
    }

    let needle = needle[0];

    match aliases.iter().find(|alias| alias.name.get() == needle) {
        Some(candidate) => {
            trace!("[{}] found candidate, expanding", function_name!());
            let expanded = match expand_command(aliases, &candidate.command) {
                Ok(command) => command,
                Err(err) => {
                    ErrorCode::InvalidCommand(candidate.command.get().to_string())
                        .log_debug(function_name!());
                    return Err(err);
                }
            };

            debug_value!(expanded);
            let output = command.get().replace(needle, &expanded.get());
            debug!("[{}] returning {:?}", function_name!(), output);
            Command::new(&output)
        }
        None => {
            trace!("[{}] nothing to expand, exiting", function_name!());
            debug_value!(command);
            Ok(command.to_owned())
        }
    }
}

/// Takes a list of aliases and returns the most matching one
pub fn find_alias<'a>(haystack: &'a Vec<Alias>, needle: &str) -> Result<Vec<Alias>, AliasError> {
    debug_value!(haystack, needle);

    if haystack.len() == 0 {
        trace!("[{}] haystack is empty, leaving", function_name!());
        return Ok(vec![]);
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
            name: Name::from(alias),
            command: unwrap_or_panic_err!(
                expand_command(&haystack, &alias.command),
                ErrorCode::ExpandAlias,
                &alias.command
            ),
        })
        .collect();
    debug_value!(aliases);

    let command = Command::new(&command.join(" "))?;
    let mut command = match expand_command(&aliases, &command) {
        Ok(command) => command,
        Err(err) => {
            let new_err = err.clone();
            ErrorCode::ExpandAlias(&command, err).log_debug(function_name!());
            return Err(new_err);
        }
    };

    debug_value!(command);

    loop {
        let matches: Vec<Alias> = aliases
            .iter()
            .filter(|candidate| candidate.command == command)
            .map(|candidate| candidate.to_owned())
            .collect();

        if command.get().len() == 0 {
            trace!(
                "[{}] command is empty, breaking out of loop",
                function_name!()
            );
            break Ok(vec![]);
        } else if matches.len() == 0 {
            trace!("[{}] no matches, trying substring", function_name!());
            debug_value!(command, matches);
            let mut temp: Vec<&str> = command.get().split_whitespace().collect();
            temp.pop();
            command = Command::new(&temp.join(" "))?;
            debug_value!(command);
        } else {
            trace!("[{}] found match, breaking out of loop", function_name!());
            debug_value!(command, matches);
            break Ok(matches);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{expand_command, find_alias, Alias, Command, NewType};

    #[test]
    fn it_matches_only_the_exact_alias() {
        let aliases: Vec<Alias> = vec![
            Alias::from("g='git'").unwrap(),
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='git branch --all'").unwrap(),
        ];

        assert_eq!(
            find_alias(&aliases, "git"),
            Ok(vec![Alias::from("g='git'").unwrap()])
        );
        assert_eq!(
            find_alias(&aliases, "git branch"),
            Ok(vec![Alias::from("gb='git branch'").unwrap()])
        );
        assert_eq!(
            find_alias(&aliases, "git branch --all"),
            Ok(vec![Alias::from("gba='git branch --all'").unwrap()])
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
            Ok(vec![Alias::from("gb='git branch'").unwrap()])
        );
        assert_eq!(
            find_alias(&aliases, "git checkout -b"),
            Ok(vec![Alias::from("gc='git checkout'").unwrap()])
        );
    }

    #[test]
    fn it_handles_empty_command() {
        let aliases: Vec<Alias> = vec![
            Alias::from("g='git'").unwrap(),
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='git branch --all'").unwrap(),
        ];
        assert_eq!(find_alias(&aliases, ""), Ok(vec![]));

        let aliases: Vec<Alias> = vec![];
        assert_eq!(find_alias(&aliases, ""), Ok(vec![]));

        let aliases: Vec<Alias> = vec![Alias::from("a=''").unwrap()];
        assert_eq!(find_alias(&aliases, ""), Ok(vec![]));
    }

    #[test]
    fn it_matches_nothing_if_no_alias_is_found() {
        let aliases: Vec<Alias> = vec![];

        assert_eq!(
            find_alias(&aliases, "git checkout -b"),
            Ok(vec![] as Vec<Alias>)
        );

        let aliases: Vec<Alias> = vec![Alias::from("a=''").unwrap()];
        assert_eq!(
            find_alias(&aliases, "git checkout -b"),
            Ok(vec![] as Vec<Alias>)
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
            Ok(vec![
                Alias::from("gb='git branch'").unwrap(),
                Alias::from("gitb='git branch'").unwrap()
            ])
        );
    }

    #[test]
    fn it_expands_alias() {
        let aliases: Vec<Alias> = vec![
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gc='git checkout'").unwrap(),
        ];

        assert_eq!(
            expand_command(&aliases, &Command::new("gb").unwrap()),
            Ok(Command::new("git branch").unwrap())
        );
        assert_eq!(
            expand_command(&aliases, &Command::new("gb --all").unwrap()),
            Ok(Command::new("git branch --all").unwrap())
        );
        assert_eq!(
            expand_command(&aliases, &Command::new("gc -b").unwrap()),
            Ok(Command::new("git checkout -b").unwrap())
        );
    }

    #[test]
    fn it_fully_expands() {
        let aliases: Vec<Alias> = vec![
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='git branch --all'").unwrap(),
        ];

        assert_eq!(
            expand_command(&aliases, &Command::new("gba").unwrap()),
            Ok(Command::new("git branch --all").unwrap())
        );
        assert_eq!(
            expand_command(&aliases, &Command::new("gba --some_flag").unwrap()),
            Ok(Command::new("git branch --all --some_flag").unwrap())
        );
        assert_eq!(
            expand_command(&aliases, &Command::new("gb --all").unwrap()),
            Ok(Command::new("git branch --all").unwrap())
        );
        assert_eq!(
            expand_command(&aliases, &Command::new("gb --all --some_flag").unwrap()),
            Ok(Command::new("git branch --all --some_flag").unwrap())
        );
        assert_eq!(
            expand_command(&aliases, &Command::new("git branch --all").unwrap()),
            Ok(Command::new("git branch --all").unwrap())
        );
        assert_eq!(
            expand_command(
                &aliases,
                &Command::new("git branch --all --some_flag").unwrap()
            ),
            Ok(Command::new("git branch --all --some_flag").unwrap())
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

        assert_eq!(
            expand_command(&aliases, &Command::new("gba").unwrap()),
            Ok(Command::new("git branch --all").unwrap())
        );
        assert_eq!(
            expand_command(&aliases, &Command::new("gba --some_flag").unwrap()),
            Ok(Command::new("git branch --all --some_flag").unwrap())
        );
        assert_eq!(
            expand_command(&aliases, &Command::new("gcb").unwrap()),
            Ok(Command::new("git checkout -b").unwrap())
        );
        assert_eq!(
            expand_command(&aliases, &Command::new("gcb branch_name").unwrap()),
            Ok(Command::new("git checkout -b branch_name").unwrap())
        );
    }

    #[test]
    fn it_leaves_alias_as_is_when_nothing_to_expand() {
        let aliases: Vec<Alias> = vec![
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='gb --all'").unwrap(),
        ];

        assert_eq!(
            expand_command(&aliases, &Command::new("git branch --all").unwrap()),
            Ok(Command::new("git branch --all").unwrap())
        );
        assert_eq!(
            expand_command(&aliases, &Command::new("git checkout").unwrap()),
            Ok(Command::new("git checkout").unwrap())
        );
    }

    #[test]
    fn it_matches_alias_using_another_alias_and_expands_it() {
        let aliases: Vec<Alias> = vec![
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='gb --all'").unwrap(),
        ];

        assert_eq!(
            find_alias(&aliases, "git branch --all"),
            Ok(vec![Alias::from("gba='git branch --all'").unwrap()])
        );
    }
}
