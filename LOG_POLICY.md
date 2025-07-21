# Log Policy
### Always use the `log` module and provide full coverage

## Configuring Log Level
You are able to configure the desired log level by passing '--diagnostic', running `diagnose_last_alias`, passing '--log-level' or '-l' with the desired level or setting the `ALIAS_LOG_LEVEL` environment variable.

If you are using the library, the logger is configured to use `println` during tests and is disabled  otherwise by default. You can initialize the logger by calling the `init_logger` method publicly exposed with the desired log level.

The available log levels are (case-insensitive):
- None (or 0)
- Error (or 1)
- Warn (or 2)
- Info (or 3, default)
- Debug (or 4)
- Trace (or 5)

## Diagnostic Logs
Diagnostic logs are logs that are useful for debugging purposes. The user doesn't see them by default. 
These logs should provide debug details to be able to reproduce and resolve problems.

Each function should have enough diagnostic logs to, at the very least, be able to reproduce any state. For instance, we should know the values of all of its parameters, which conditions passed and what is the output.

### Format
Each debug log should contain the function name in square brackets, followed by a detailed description of the current state, a few examples are:
```
[CONTEXT] <name> is <value> # Print a value
[CONTEXT] Got an error when trying to <action>, falling back to <fallback> # Got an error
[CONTEXT] Entered/Left # Enter a clause, function, condition, loop, etc.
```

## Levels
- `DEBUG` - conditions passed or failed, output of functions, state values or changes, parameters and inputs
- `TRACE` - enter and exit of clause, condition or context

## User Logs
User logs are messages that the average user sees when using this program. These logs should be minimal to avoid cluttering the stdout and stderr.

Each log should use the `Display` trait.

### Errors
Each error should contain actionable steps to resolve (if possible) and report the issue.
It should be readable to the average user.

In the code, each type of error should be documented, and a have a message, actionable steps and a corresponding numeric code.

#### Format
```
alias-helper Error! <message>
You can resolve this issue by <steps>
Submit a bug report by creating an issue at https://github.com/PCOffline/alias-helper/issues/new
For more information, pass '--diagnostic' or run `diagnose_last_alias` in your terminal.
```

The `alias-helper` text should be blue, and the `Error!` text should be red.
The `message`, `steps` and the URL should be gold, and the rest of the text should be white.

### Warnings
Alerts should be rare and be displayed only when one of the following conditions is true:
1. The program is about to perform or take an action that is probably unwanted or unintentional (large performance hit, destructive behavior, etc.)
2. The program encountered an error that it could recover from, but is probably not the intended behavior (partially-invalid input, combination of flags that doesn't make sense, etc.)

In both cases, the program should ask the user whether the program should continue, **but not block the stdout or stderr threads**.

#### Format
```
alias-helper Alert! <message>
<suggestion>
If you believe this is a bug, please open an issue at https://github.com/PCOffline/alias-helper/issues/new
For more information, pass '--diagnostic' or run `diagnose_last_alias` in your terminal.
```

The `alias-helper` text should be blue, and the `Alert!` text should be yellow.
The `message`, `suggestion` and the URL should be gold, and the rest of the text should be white.
The `suggestion` should contain text that offers ways to resolve the alert, for example: "Consider passing the following flag".

### Info
The success output should be minimal and concise, to not overwhelm the user.
It can be either the output of the alias-helper algorithm, or an output of a flag.

#### Format
There is no template for this output, as the restrictions above should suffice, and the variety of flag outputs restricts us from deciding on a specific template.
Either way, where output may be implicit (as in, when the command may be called implicitly by a third-party), it is important to provide the name of the program for context.
In contrast, when the program is called explicitly, especially with flags such as `--help`, `--diagnostic` or `--version`, you may omit the name of the program.

If the name of the program is passed, it should be blue, plain text should be white and any important parts that you want to emphasize should be gold.
For example:
```
$ alias-helper --version
v0.2.1 # this text is fully white
```

```
$ alias-helper --help
--help # this text is blue
    shows the list of commands # this text is white
```