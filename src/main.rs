use alias_helper::{self, find_alias, log::*, Alias};
use log::LevelFilter;
use std::{
    env,
    io::{self, BufRead},
    process,
};

extern crate exitcode;

fn main() {
    alias_helper::init_logger(LevelFilter::Info);

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

    if aliases.len() == 0 || needle.len() == 0 {
        if needle.len() == 0 {
            ErrorCode::NoCommandInput.log_and_panic("main")
        }

        ErrorCode::NoAliasesInput.log_and_panic("main");
    }

    let result = find_alias(&aliases, &needle)
        .unwrap_or_else(|err| ErrorCode::from(err).log_and_panic("main"));

    if result.len() > 0 {
        info!(
            "{}",
            result
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );
        process::exit(exitcode::OK);
    } else {
        ErrorCode::NoOutput.log_and_panic("main");
    }
}
