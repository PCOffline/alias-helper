use crate::{
    util::macros::{debug_value, function_name},
    NewType,
};

use super::super::Alias;
use log::*;
use std::collections::HashSet;

pub fn remove_cycles(aliases: &Vec<Alias>) -> Vec<Alias> {
    debug_value!(aliases);
    let mut visited = HashSet::new();
    let mut stack = Vec::new(); // Use Vec to maintain order of traversal
    let mut in_cycle = HashSet::new(); // Nodes found in cycles

    fn dfs(
        node: &str,
        aliases: &Vec<Alias>,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
        in_cycle: &mut HashSet<String>,
    ) {
        debug_value!(node, aliases, visited, stack, in_cycle);
        if stack.contains(&node.to_string()) {
            trace!("[{}] stack contains node", function_name!());
            let cycle_start_index = stack.iter().position(|n| n == node).unwrap();
            for i in cycle_start_index..stack.len() {
                in_cycle.insert(stack[i].clone());
            }
            return;
        }
        if !visited.insert(node.to_string()) {
            trace!("[{}] already visited this node", function_name!());
            return;
        }
        debug!("[{}] pushing {:?} to stack", function_name!(), node);
        stack.push(node.to_string());

        let Some(node) = &aliases.into_iter().find(|alias| alias.name.get() == node) else {
            debug!(
                "[{}] Cannot find {:?} under commands: {:?}",
                function_name!(),
                node,
                aliases
            );
            stack.pop();
            return;
        };

        if node.command.get().trim().len() == 0 {
            trace!("[{}] node has no command", function_name!());
            debug_value!(node);
            stack.pop();
            return;
        }

        let command = &node.command.get().split_whitespace().nth(0).unwrap();

        trace!("[{}] calling dfs again", function_name!());
        debug_value!(command, aliases, visited, stack, in_cycle);

        dfs(command, aliases, visited, stack, in_cycle);

        stack.pop();
    }

    for alias in aliases {
        dfs(
            &alias.name.get(),
            &aliases,
            &mut visited,
            &mut stack,
            &mut in_cycle,
        );
    }

    // Remove nodes found in cycles from the original map
    let mut result: Vec<Alias> = vec![];
    for alias in aliases.into_iter() {
        if !in_cycle.contains(alias.name.get()) {
            result.push(alias.clone());
        }
    }

    debug_value!(result);

    result
}

pub fn filter_invalid_aliases(aliases: &Vec<Alias>) -> Vec<Alias> {
    let aliases: Vec<Alias> = aliases
        .iter()
        .filter(|alias| alias.command.get().trim().len() > 0)
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
