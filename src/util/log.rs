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
    RegexValidationMatch(&'a Regex, &'a str, fancy_regex::Error) = 3,
}

impl<'a> ErrorCode<'a> {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }

    fn default_err(&self) -> String {
        format!("Error {}: An unexpected error occured", self.discriminant())
    }

    pub fn log(&self, function_name: &str) -> ! {
        let message = match self {
            ErrorCode::RegexParse(regex, error) => {
                debug!(
                    "[{function_name}] Could not parse regex {:?}. Error details: {error}",
                    regex
                );
                self.default_err()
            }
            ErrorCode::RegexValidationMatch(regex, input, error) => {
                debug!(
                    "[{function_name}] Could not match regex {:?} for input {:?}. Error details: {error}",
                    regex,
                    input
                );
                self.default_err()
            }
            _ => self.default_err(),
        };

        panic!("{message}");
    }
}
