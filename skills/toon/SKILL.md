---
name: toon
description: Convert JSON, JSONC, or NDJSON from stdin into ordered TOON output with the toon CLI.
metadata:
  created_on: 2026-10-14 15:08
  last_modified: 2026-10-14 15:08
  status: current
---

# Usage

`toon` reads UTF-8 stdin, accepts JSON, one JSONC document, or consecutive JSON values (including NDJSON), and prints TOON followed by a newline. Pipe input or redirect a file: `toon < input.json > output.toon`. Conversion has no positional arguments. Object field order is preserved; multiple JSON values become one top-level array. JSONC accepts comments and trailing commas. Duplicate object keys are errors. JSON5 extensions are rejected.

Integers beyond ±9007199254740991 become exact decimal strings. Decimal/exponent numbers use binary64 precision, zero becomes `0`, and overflow becomes the original numeric string. One leading UTF-8 BOM and surrounding whitespace are accepted. All input is read before conversion; this is an in-memory converter.

`toon --version` or `toon -V` prints the raw package version and newline. `--version` has type bool, defaults to false, and is exclusive with other options.

`toon --help` or `toon -h` prints help. `--help` has type bool and defaults to false. `toon help` prints root help; `toon help skill` prints skill-command help; `toon help help` prints generated-help help. `toon skill --help` prints skill-command help. These generated help commands perform no conversion. There are no completion commands.

`toon skill` accepts no positional arguments or command-specific flags. It prints the Skills CLI install command and canonical skill URL, without network or file access. The pointer addresses the original repository; it does not establish publication of this Rust implementation.

`AGENT=1`, `AGENT=true`, or `AGENT=yes` selects agent formatting: help begins with an alert to read `AGENT=1 toon skill`; runtime errors have an `ERR:` prefix. Human runtime errors have an `[ERROR]` prefix. Encoded data, raw version, and skill-command bytes are identical in both modes. Data goes to stdout; errors go once to stderr. Success exits 0; invalid input, arguments, interactive stdin, and read/write failures exit 1. Runtime failures print no usage screen. Conversion makes no network requests and writes no files beyond redirected stdout.

```sh
printf '{"name":"Ada","id":1}\n' | toon
printf '{"id":1}\n{"id":2}\n' | AGENT=1 toon
toon --version
```
