use std::{
    env,
    io::{self, BufRead},
};

use alias_helper::{find_alias, Alias};

fn main() {
    let stdin = io::stdin();
    let aliases: Vec<Alias> = stdin
        .lock()
        .lines()
        .filter_map(|s| s.ok())
        .map(|s| Alias::from(&s))
        .filter_map(|s| s.ok())
        .collect();
    let needle: Vec<String> = env::args().skip(1).collect();
    let needle: String = needle.join(" ");

    let result = find_alias(&aliases, &needle);
    println!("{:?}", result);
}
