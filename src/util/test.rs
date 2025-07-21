use crate::{
    debug, debug_value, function_name, unwrap_or_panic, unwrap_or_panic_err, Alias, Command,
    ErrorCode, Name, NewType,
};
use fancy_regex::Regex;
use rand::{seq::IteratorRandom, Rng};
use std::iter;

use super::validation;

pub fn get_valid_aliases(amount: Option<usize>) -> Vec<Alias> {
    get_valid_alias_strings(amount)
        .iter()
        .map(|alias| {
            unwrap_or_panic!(
                Alias::from(alias),
                ErrorCode::InvalidAlias(alias.to_string())
            )
        })
        .collect()
}

fn generate_random_words(
    charset: Vec<u8>,
    min_word_length: usize,
    max_words_length: usize,
) -> impl Iterator<Item = String> {
    let gen_word = move || {
        let mut rng = rand::thread_rng();
        let one_char = || charset[rng.gen_range(0..charset.len())] as char;
        let mut rng = rand::thread_rng();
        iter::repeat_with(one_char)
            .take(rng.gen_range(min_word_length..=max_words_length))
            .collect::<String>()
    };

    iter::repeat_with(gen_word)
}

fn get_valid_word_iterator() -> impl Iterator<Item = String> {
    const SKIPPED_CHARS: &[u8] = b"+\n \t\xa0";

    let v: Vec<u8> = (0..128)
        // Filter out space, tab and line feed
        .filter(|i| !SKIPPED_CHARS.contains(i))
        .map(|i| i as u8)
        .collect();
    generate_random_words(v, 1, 20)
}

fn get_invalid_word_iterator() -> impl Iterator<Item = String> {
    generate_random_words((0..128).map(|i| i as u8).collect(), 1, 20).filter(|word| {
        b"+\n \t\xa0".to_vec().iter().any(|c| {
            if word.contains(*c as char) {
                debug_value!(c, word);
                return true;
            }

            false
        })
    })
}

pub fn get_valid_alias_strings(amount: Option<usize>) -> Vec<String> {
    let commands = get_valid_command_strings(amount);
    get_valid_name_strings(amount)
        .into_iter()
        .enumerate()
        .map(move |(index, name)| format!("{}='{}'", name, commands[index]))
        .collect()
}

// TODO: Make it so commands get variations
pub fn get_invalid_alias_strings(amount: Option<usize>) -> Vec<String> {
    let commands = get_valid_command_strings(amount);
    get_invalid_name_strings(amount)
        .into_iter()
        .enumerate()
        .map(move |(index, name)| format!("{}='{}'", name, commands[index]))
        .collect()
}

pub fn get_valid_names(amount: Option<usize>) -> Vec<Name> {
    get_valid_name_strings(amount)
        .iter()
        .map(|name| Name::new(name).unwrap())
        .collect()
}

trait Formatter<I, O> {
    fn map(&self, input: I) -> O;
    fn validate(&self, input: &I) -> bool;
}

#[derive(Clone)]
struct RegexFormatter<F> {
    regex: Regex,
    func: Box<F>,
}

impl<F> RegexFormatter<F>
where
    F: Fn(&str) -> String,
{
    fn new(regex: &str, func: F) -> RegexFormatter<F> {
        let regex = unwrap_or_panic_err!(Regex::new(regex), ErrorCode::RegexParse, regex);
        RegexFormatter {
            regex,
            func: Box::new(func),
        }
    }
}

impl<F> Formatter<String, String> for RegexFormatter<F>
where
    F: Fn(&str) -> String,
{
    fn validate(&self, input: &String) -> bool {
        unwrap_or_panic_err!(
            self.regex.is_match(input),
            ErrorCode::RegexValidationMatch,
            &self.regex,
            input
        )
    }

    fn map(&self, input: String) -> String {
        if self.validate(&input) {
            (self.func)(&input)
        } else {
            input.to_string()
        }
    }
}

#[derive(Clone)]
struct PredicateFormatter<F, P> {
    predicate: Box<P>,
    func: Box<F>,
}

impl<F, P> PredicateFormatter<F, P>
where
    F: Fn(&str) -> String,
    P: Fn(&str) -> bool,
{
    fn new(predicate: P, func: F) -> PredicateFormatter<F, P> {
        PredicateFormatter {
            predicate: Box::new(predicate),
            func: Box::new(func),
        }
    }
}

impl<F, V> Formatter<String, String> for PredicateFormatter<F, V>
where
    F: Fn(&str) -> String,
    V: Fn(&str) -> bool,
{
    fn map(&self, input: String) -> String {
        if self.validate(&input) {
            (self.func)(&input)
        } else {
            input.to_string()
        }
    }

    fn validate(&self, input: &String) -> bool {
        (self.predicate)(input)
    }
}

pub fn get_valid_name_strings(amount: Option<usize>) -> Vec<String> {
    let filtered_single_chars: Vec<&str> = vec!["", "+", "-", "*"];
    let validations: Vec<Box<dyn Formatter<String, String>>> = vec![
        Box::new(RegexFormatter::new(
            "\\$[\\d\\w]",
            |input: &str| -> String {
                if input.starts_with("\\'") && input.ends_with("\\'") {
                    input.to_string()
                } else {
                    format!("''")
                }
            },
        )),
        Box::new(PredicateFormatter::new(
            |input: &str| -> bool {
                input.starts_with("'") || input.ends_with("'") || input.contains("=")
            },
            |input: &str| -> String { format!("\\'{input}\\'") },
        )),
        Box::new(RegexFormatter::new("'", |input: &str| -> String {
            Regex::new("(?<!\\\\)'")
                .unwrap()
                .replace_all(input, "\\'")
                .to_string()
        })),
    ];

    let amount = amount.unwrap_or(20);
    get_valid_word_iterator()
        .filter(|word| !filtered_single_chars.contains(&word.as_str()))
        .map(|word| {
            let mut result = word;
            for validation in validations.iter() {
                result = validation.map(result);
            }
            result
        })
        .take(amount)
        .collect()
}

pub fn get_invalid_name_strings(amount: Option<usize>) -> Vec<String> {
    const DEFAULT_NAMES_AMOUNT: usize = 20;
    get_invalid_word_iterator()
        .take(amount.unwrap_or(DEFAULT_NAMES_AMOUNT))
        .collect()
}

pub fn get_valid_commands(amount: Option<usize>) -> Vec<Command> {
    get_valid_command_strings(amount)
        .iter()
        .map(|command| Command::new(command).unwrap())
        .collect()
}

pub fn get_valid_command_strings(amount: Option<usize>) -> Vec<String> {
    const DEFAULT_COMMANDS_AMOUNT: usize = 20;
    const COMMAND_MAX_LENGTH: usize = 10;

    let formatters: Vec<Box<dyn Formatter<String, String>>> = vec![
        Box::new(RegexFormatter::new("\\s", |input: &str| -> String {
            format!("'{input}'")
        })),
        Box::new(RegexFormatter::new("'", |input: &str| -> String {
            Regex::new("(?<!\\\\)'")
                .unwrap()
                .replace_all(input, "\\'")
                .to_string()
        })),
    ];

    let amount = amount.unwrap_or(DEFAULT_COMMANDS_AMOUNT);
    (0..amount)
        .map(|_| {
            let mut rng = rand::thread_rng();
            get_valid_word_iterator()
                .filter(|word| !word.is_empty())
                .map(|word| {
                    let mut result = word;
                    for validation in formatters.iter() {
                        result = validation.map(result);
                    }
                    result
                })
                .take(rng.gen_range(0..=COMMAND_MAX_LENGTH))
                .collect::<Vec<String>>()
                .join(" ")
        })
        .collect()
}

fn get_command_suffixes() -> Vec<String> {
    vec![
        "--some_flag".to_string(),
        "--some-flag".to_string(),
        "-f".to_string(),
        "-l".to_string(),
        "abc".to_string(),
        "0".to_string(),
        "some very long thing".to_string(),
        "-a -b -c".to_string(),
        "-0 -b -#".to_string(),
        "\"abc\"".to_string(),
        "'abc'".to_string(),
        "'\\\"'".to_string(),
        "<some_string>".to_string(),
        "$123".to_string(),
        "-1".to_string(),
    ]
}

pub fn get_aliases_from_name(name: Name) -> Vec<Alias> {
    get_valid_commands(None)
        .into_iter()
        .map(|command| Alias::new(name.clone(), command))
        .collect()
}

pub fn get_aliases_from_command(command: Command) -> Vec<Alias> {
    get_valid_names(None)
        .into_iter()
        .map(|name| Alias::new(name, command.clone()))
        .collect()
}

pub fn get_variations_from_alias(alias: Alias) -> Vec<Alias> {
    get_variations_from_command(alias.command)
        .into_iter()
        .map(|command| Alias::new(alias.name.clone(), command))
        .collect()
}

pub fn get_variations_from_alias_string(alias: &str) -> Vec<Alias> {
    get_variations_from_alias(Alias::from(alias).unwrap())
}

pub fn get_variations_from_command(command: Command) -> Vec<Command> {
    get_variations_from_command_string(command.get())
}

pub fn get_variations_from_command_string(command: &str) -> Vec<Command> {
    get_command_suffixes()
        .iter()
        .map(|suffix| Command::new(&format!("{} {}", command, suffix)).unwrap())
        .collect()
}

/// Returns a list of circular aliases (aliases that reference each other directly or indirectly)
/// The returned vector includes multiple lists of circular aliases, each with various variations in names and commands
/// * `steps` — how many steps are in between the first and final alias; Default: 3
///
/// # Example
/// ```ignore
///
/// let aliases = test::get_circular_aliases(Some(3));
/// println!("{:?}", aliases); // ["e='f'" ,"f='g'", "g='e'"]
/// ```
pub fn get_circular_aliases(steps: Option<u8>) -> Vec<Alias> {
    const DEFAULT_STEPS_COUNT: u8 = 3;
    let steps = steps.unwrap_or(DEFAULT_STEPS_COUNT);

    let names = get_valid_names(Some(steps as usize));
    let commands = get_valid_commands(Some(steps as usize));
    let mut rng = rand::thread_rng();

    // ! FIXME: This limits the amount of steps to the amount of names
    // Map over random names,

    names
        .iter()
        .choose_multiple(&mut rng, steps as usize)
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let command: Command = if index == 0 {
                commands.iter().choose(&mut rng).unwrap().to_owned()
            } else if index == names.len() - 1 {
                get_variations_from_command_string(names[0].get())
                    .iter()
                    .choose(&mut rng)
                    .unwrap()
                    .to_owned()
            } else {
                get_variations_from_command_string(names[index - 1].get())
                    .iter()
                    .choose(&mut rng)
                    .unwrap()
                    .to_owned()
            };

            Alias::new((*name).clone(), command)
        })
        .collect()
}
