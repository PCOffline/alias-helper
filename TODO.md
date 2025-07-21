# Intro
In each phase, the definition of "matching alias" becomes more sophisticated.
With time, the script will be able to suggest using existing aliases in more cases, and even suggest to add new aliases based on usage.

## Actual shit that needs to be done
### Add flags
**Flags take priority over environment variables**
- --verbose or ALIAS_HELPER_LOG_LEVEL=trace env-var: output all diagnostics logs
- -l, --log-level [level] or ALIAS_HELPER_LOG_LEVEL env-var: set desired log level
- diagnose_last_alias: output all diagnostic logs for last input
- -x, --exclude [csv] or ALIAS_HELPER_EXCLUDE env-var: don't suggest aliases for the following substrings
- -q, --quiet or ALIAS_HELPER_LOG_LEVEL=none env-var: supress output and only return an exit code
- -v, --version: output version information
- -c, --config or ALIAS_HELPER_CONFIG_FILE env-var: configuration file
- -h, --help: output help information
- -o, --log-file: file to which logs will be written instead of displaying in stdout

## Configuration File
Create a TOML configuration file where you can specify all the flags above

# Phases
## Version 0.1.0
### Definition of Matching Alias
An alias that is a substring of the command:
```zsh
alias gb="git branch"
alias gba="git branch --all"
alias gc="git checkout"

git branch # Should suggest 'gb'
git branch --all # Should suggest 'gba'
git checkout dev # should suggest 'gc dev'
```

### Goals
- Create & Configure GitHub repo
- Public release 0.1.0

## Version 0.2.0
### Definition of Matching Alias
An alias that is a substring of the command, or an alias that uses an alias which is a substring of the command:
```zsh
alias gb="git branch"
alias gba="gb --all"
alias gc="git checkout"
alias gcb="gc -b"

git branch # Should suggest 'gb'
git branch --all # Should suggest 'gba'
git checkout dev # Should suggest 'gc dev'
git checkout -b feature # Should suggest 'gcb'
```

### Goals
- Add unit tests

## Version 0.3.0
- [x] Split to modules
- [x] Use the newtype pattern for Name & Command
- [x] Better logs and macros
- [x] Better error messages and handling
- [] More rigid and organised tests
- [x] Return Result for all functions
- [x] Better validation of commands and names
- [] Optimise performance
- [] Get rid of regex validation
- [x] Change panic macros to exit with status code according to the error code
- [x] Make the main.rs log an error and exit with status code when relevant

In Practice:
- Fuzz testing
- Dynamic testing with templates (e.g. make sure this charater exists in name)
- Actual parser instead of regexes

## Version 0.4.0
- Change list of Aliases to Map
- Hook onto Bash history file
- Begin collecting logs of commands and suggesting to create aliases for repeating commands
- Add disclaimer for sensitive information, make it a feature in the Cargo.toml and ENV variables
- Add support for multiple commands split by `&&`, `;`, `|`, `then`, etc. (?)
- Read $1 parameter as a wildcard, counting any replacements of this placeholder as valid candidates for suggesting the alias
- Add unit tests

## Version 0.5.0
- Group variations of commands under a base command alias, and make all variations use the base alias
  e.g.: an alias for `git checkout -b` and `git checkout --detach` should use the base alias of `git checkout`
- Give option to create suggested aliases with a convenient command
- Add support for Git Aliases from `/.gitconfig and local git configs
- Add unit tests

### Version 0.6.0
- Create an Oh My Zsh Plugin
- Add proper documentation
- Add unit tests
- Improve performance
- Polish code