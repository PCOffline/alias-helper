fn split_alias(alias: &str) -> (String, String) {
    let alias: Vec<&str> = alias.split("=").collect();
    (alias[0].to_string(), alias[1][1..alias[1].len() - 1].to_string())
}

fn format_alias(split_alias: &(String, String)) -> String {
    format!("{}='{}'", split_alias.0, split_alias.1)
}

/// Takes a list of aliases and returns the most matching one
pub fn find_alias(haystack: &Vec<String>, needle: &str) -> Vec<String> {
    if haystack.len() == 0 {
        return vec![];
    }

    let mut command: Vec<&str> = needle.split_whitespace().collect();

    let aliases: Vec<(String, String)> = haystack
        .iter()
        .map(|alias| split_alias(&alias))
        .collect();

    loop {
        if command.len() == 1 {
            break aliases.iter().map(|alias| format_alias(&alias)).collect();
        }

        let matches: Vec<String> = aliases
            .iter()
            .filter(|alias| {
                alias.1 == command.join(" ")
            })
            .map(|alias| format_alias(&alias))
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
    use crate::find_alias;

    #[test]
    fn it_matches_only_the_exact_alias() {
        let aliases: Vec<String> = vec![
            "gb='git branch'".to_string(),
            "gba='git branch --all'".to_string(),
        ];

        assert_eq!(find_alias(&aliases, "git branch"), vec!["gb='git branch'"]);
        assert_eq!(find_alias(&aliases, "git branch --all"), vec!["gba='git branch --all'"]);
    }

    #[test]
    fn it_matches_substring_when_no_exact_alias_is_found() {
        let aliases: Vec<String> = vec![
            "gb='git branch'".to_string(),
            "gba='git branch --all'".to_string(),
            "gc='git checkout'".to_string(),
        ];

        assert_eq!(find_alias(&aliases, "git checkout -b"), vec!["gc='git checkout'"]);
    }

    #[test]
    fn it_matches_nothing_if_no_alias_is_found() {
        let aliases: Vec<String> = vec![];

        assert_eq!(find_alias(&aliases, "git checkout -b"), vec![] as Vec<&str>);
    }
}
