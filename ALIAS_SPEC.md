# Edge Cases
- Closing brackets 
- Single character commands
- Single character names

## Definitions
- By default, an alias has the following format: `<name>=<command>`
- A name consists of a single word, without any whitespace characters
- If the name contains special characters, it will be surrounded by single quotes
- If the command contains special characters or whitespace, it will be surrounded by single quotes
- In order to use a single-quote in an alias, you must exit the current context of quotes, type `\'` and then open a new context. For example: `alias name="it's a command"` will become `name='it'\''s a command'. If the single-quote is at the beginning or end of a name or command, the former single-quote is omitted. For example: `alias name="'command"` becomes `name=\''command'`.
- Aliases support the ANSI-C quoting syntax, see [the docs](https://typeoverflow.com/developer/docs/bash/ansi_002dc-quoting)
- Special characters are:
  - empty string
  - quotes (`'`,`"`,`\``)
  - whitespace characters (`\s`,`\t`,`\n`,`\r`)
  - `$`
  - `=`
  - `()`
  - `[]`
  - `{}`
- The names `+`, `=` and `-` are not allowed

### Bugs with current validation system
- Mocks can generate duplicates
- Current system treats any unescaped \ as invalid, regardless of its position
- You are able to escape whitespace without quotes

Examples
```zsh
 $ alias "a\$1b"="echo hello"
 $ alias "'a\$1b"="echo hello"
 $ alias "'a\$1b'"="echo hello"
 $ alias | grep "echo hello"   
```
outputs:
```zsh
\''a$1b'='echo hello'
\''a$1b'\'='echo hello'
'a$1b'='echo hello'
```