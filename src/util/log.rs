use colog::format::CologStyle;
use colored::Colorize;
use env_logger::{fmt::Formatter, Builder};
use fancy_regex::Regex;

#[cfg(not(test))]
#[allow(unused_imports)]
pub use log::{debug, error, info, trace, warn};
use log::{Level, LevelFilter, Record};
use std::io::{Error, Write};

// Workaround to use prinltn! for logs in test mode.
#[cfg(test)]
#[allow(unused_imports)]
pub use std::{
    println as trace, println as debug, println as info, println as warn, println as error,
};

use crate::{AliasError, Command};

trait CustomLog: CologStyle {
    fn suffix_message(&self, level: &Level) -> String;

    fn format(&self, buf: &mut Formatter, record: &Record<'_>) -> Result<(), Error> {
        let sep = self.line_separator();
        let mut prefix = self.prefix_token(&record.level());
        let suffix = self.suffix_message(&record.level());

        if !self.level_token(&record.level()).is_empty() {
            prefix += " ";
        }

        writeln!(
            buf,
            "{}{} {}",
            prefix,
            record.args().to_string().replace('\n', &sep),
            suffix,
        )
    }
}

struct CustomLogStyle;

impl CologStyle for CustomLogStyle {
    fn level_token(&self, level: &Level) -> &str {
        match level {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }

    fn level_color(&self, level: &log::Level, msg: &str) -> String {
        match level {
            Level::Error => msg.red(),
            Level::Warn => msg.yellow(),
            Level::Info => msg.white(),
            Level::Debug => msg.green(),
            Level::Trace => msg.purple(),
        }
        .to_string()
    }

    fn prefix_token(&self, level: &Level) -> String {
        format!(
            "{} {}",
            "alias-helper".cyan(),
            self.level_color(level, self.level_token(level))
        )
    }

    fn format(&self, buf: &mut Formatter, record: &Record<'_>) -> Result<(), Error> {
        CustomLog::format(self, buf, record)
    }
}

impl CustomLog for CustomLogStyle {
    fn suffix_message(&self, level: &Level) -> String {
        let url = "https://github.com/PCOffline/alias-helper/issues/new";
        let diagnostic = "For more information, pass '--diagnostic' or run 'diagnose_last_alias' in your terminal.";
        let bug_report =
            format!("\nIf you believe this is a bug, please open an issue at {url}\n{diagnostic}");

        match level {
            Level::Error => bug_report,
            Level::Warn => bug_report,
            _ => "".to_string(),
        }
    }
}

/// Initalizes the logger and enables log messages at
pub fn init(level: LevelFilter) {
    let mut builder = Builder::new();
    builder.format(colog::formatter(CustomLogStyle));
    builder.filter(None, level);
    builder.init();
}

#[repr(u8)]
pub enum ErrorCode<'a> {
    RegexParse(&'a str, fancy_regex::Error) = 2,
    RegexValidationMatch(&'a Regex, &'a str, fancy_regex::Error),
    NoCommandInput,
    NoAliasesInput,
    ExpandCommand(&'a Command, AliasError),
    InvalidName(String),
    InvalidCommand(String),
    InvalidAlias(String),
    NoOutput,
}

impl<'a> ErrorCode<'a> {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }

    fn default_err(&self) -> String {
        format!("Error {}: An unexpected error occured", self.discriminant())
    }

    pub fn log_debug(&self, function_name: &str) -> () {
        match self {
            ErrorCode::RegexParse(regex, error) => {
                debug!(
                    "[{function_name}] Could not parse regex {:?}. Error details: {error}",
                    regex
                );
            }
            ErrorCode::RegexValidationMatch(regex, input, error) => {
                debug!(
                    "[{function_name}] Could not match regex {:?} for input {:?}. Error details: {error}",
                    regex,
                    input
                );
            }
            ErrorCode::InvalidAlias(alias) => {
                debug!("[{function_name}] Received invalid alias {:?}", alias);
            }
            ErrorCode::ExpandCommand(command, error) => {
                debug!(
                    "[{function_name}] Could not expand command {:?}. Received the following error: {:?}",
                    command, error
                );
            }
            ErrorCode::InvalidCommand(command) => {
                debug!(
                    "[{function_name}] Received an invalid command {:?}",
                    command
                );
            }
            ErrorCode::InvalidName(name) => {
                debug!("[{function_name}] Received an invalid name {:?}", name);
            }
            ErrorCode::NoAliasesInput => {
                debug!("[{function_name}] No aliases were passed to the program")
            }
            ErrorCode::NoCommandInput => {
                debug!("[{function_name}] No command was passed to the program");
            }
            ErrorCode::NoOutput => {
                debug!("[{function_name}] Couldn't find any matching aliases");
            }
        };
    }

    pub fn log_err(&self) -> () {
        match self {
            ErrorCode::NoCommandInput => {
                error!("No command provided. Please specify a command to execute. For help, use the '-h' or '--help' flag.");
            }
            ErrorCode::NoAliasesInput => {
                error!("No aliases provided. Please pipe the list of aliases into the command. For help, use the '-h' or '--help' flag.")
            }
            ErrorCode::InvalidAlias(alias) => {
                // Technically, this error should never result in a panic,
                // since we filter out invalid aliases.
                error!(
                    "The program encountered an invalid alias: {:?}.
                Please remove or modify it to continue running.",
                    alias
                );
            }
            ErrorCode::NoOutput => {
                error!("There is no alias matching your command.");
            }
            _ => error!("{}", self.default_err()),
        }
    }

    pub fn log_and_panic(&self, function_name: &str) -> ! {
        self.log_debug(function_name);
        self.log_err();
        std::process::exit(self.discriminant() as i32);
    }
}

impl<'a> Into<i32> for ErrorCode<'a> {
    fn into(self) -> i32 {
        self.discriminant() as i32
    }
}

impl<'a> From<AliasError> for ErrorCode<'a> {
    fn from(value: AliasError) -> Self {
        match value {
            AliasError::InvalidCommand(command) => ErrorCode::InvalidCommand(command),
            AliasError::InvalidName(name) => ErrorCode::InvalidName(name),
            AliasError::ParseError(alias) => ErrorCode::InvalidAlias(alias),
        }
    }
}
