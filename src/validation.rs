use super::Alias;
use std::collections::HashSet;

pub fn remove_cycles(commands: &Vec<Alias>) -> Vec<Alias> {
    let mut visited = HashSet::new();
    let mut stack = Vec::new(); // Use Vec to maintain order of traversal
    let mut in_cycle = HashSet::new(); // Nodes found in cycles

    fn dfs(
        node: &str,
        commands: &Vec<Alias>,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
        in_cycle: &mut HashSet<String>,
    ) {
        if stack.contains(&node.to_string()) {
            let cycle_start_index = stack.iter().position(|n| n == node).unwrap();
            for i in cycle_start_index..stack.len() {
                in_cycle.insert(stack[i].clone());
            }
            return;
        }
        if !visited.insert(node.to_string()) {
            return;
        }
        stack.push(node.to_string());

        let Some(node) = &commands.into_iter().find(|c| c.name == node) else {
            println!("Cannot find {node} under commands: {:?}", commands);
            stack.pop();
            return;
        };

        if node.command.trim().len() == 0 {
            stack.pop();
            return;
        }

        dfs(
            &node.command.split_whitespace().nth(0).unwrap(),
            commands,
            visited,
            stack,
            in_cycle,
        );

        stack.pop();
    }

    for command in commands {
        dfs(
            &command.name,
            &commands,
            &mut visited,
            &mut stack,
            &mut in_cycle,
        );
    }

    // Remove nodes found in cycles from the original map
    let mut result: Vec<Alias> = vec![];
    for alias in commands.into_iter() {
        if !in_cycle.contains(&alias.name) {
            result.push(alias.clone());
        }
    }

    result
}

pub fn filter_invalid_aliases(aliases: &Vec<Alias>) -> Vec<Alias> {
    let aliases: Vec<Alias> = aliases
        .iter()
        .filter(|alias| alias.command.trim().len() > 0)
        .map(Clone::clone)
        .collect();

    remove_cycles(&aliases)
}

#[cfg(test)]
mod tests {
    use super::Alias;
    use crate::validation;

    #[test]
    fn it_filters_faulty_aliases() {
        let aliases: Vec<Alias> = vec![
            Alias::from("a=''").unwrap(),
            Alias::from("aa='       '").unwrap(),
            // Self-reference
            Alias::from("b='b'").unwrap(),
            // Bi-self-reference
            Alias::from("c='d'").unwrap(),
            Alias::from("d='c'").unwrap(),
            // Triangle of self-references
            Alias::from("e='f -i'").unwrap(),
            Alias::from("f='g -l'").unwrap(),
            Alias::from("g='e -y'").unwrap(),
        ];

        assert_eq!(
            validation::filter_invalid_aliases(&aliases),
            vec![] as Vec<Alias>
        );
    }

    #[test]
    fn it_doesnt_filter_valid_aliases() {
        let aliases: Vec<Alias> = vec![
            Alias::from("g='git'").unwrap(),
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='git branch --all'").unwrap(),
        ];

        assert_eq!(validation::filter_invalid_aliases(&aliases), aliases);

        let aliases: Vec<Alias> = vec![
            Alias::from("gb='git branch'").unwrap(),
            Alias::from("gba='gb --all'").unwrap(),
        ];

        assert_eq!(validation::filter_invalid_aliases(&aliases), aliases);
    }
}
