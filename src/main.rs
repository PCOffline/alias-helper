use std::{env, io::{self, BufRead}};

use alias_helper::find_alias;

fn main() {
    let stdin = io::stdin();
    let aliases: Vec<String> = stdin.lock().lines().filter_map(|s| s.ok()).collect();
    let needle: Vec<String> = env::args().skip(1).collect();
    let needle: String = needle.join(" ");

    let result = find_alias(&aliases, &needle);
    println!("{:?}", result);
}
