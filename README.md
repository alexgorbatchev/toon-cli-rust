`toon` converts piped JSON, JSONC, and NDJSON into TOON for people preparing compact structured text. It preserves object order and exact large integer digits.

# What It Does

- **Convert structured text:** Read stdin and print TOON with two-space indentation and comma delimiters.
- **Read commented JSON:** Accept one JSONC document, including arrays and trailing commas.
- **Combine streams:** Turn consecutive JSON values into one top-level array.

# How It Works

- Pipe JSON into `toon`, or redirect a JSON file into stdin.
- Read the converted document from stdout or redirect it to a file.
- Read failures once from stderr; a failed conversion returns exit status 1.

# How it Really Works

- Input is UTF-8, with one optional leading BOM. The whole document is held in memory.
- Object keys retain their input order. Duplicate keys return an error, including escaped spellings of the same key.
- Integers outside ±9007199254740991 become quoted decimal strings. Decimal and exponent values use binary64 precision; numeric overflow becomes the original quoted numeric string.
- JSONC accepts comments and trailing commas in a single document. Other JSON5 syntax returns an error.
- Conversion makes no network requests and changes no files. Successful output ends in a newline.
- `AGENT=1` selects compact help and error formatting; converted data is the same in both modes. Interactive stdin returns an error directing you to pipe input.

# Installation

Rust release binaries are unpublished. Contributor commands are in [AGENTS.md](AGENTS.md). The canonical [source repository](https://github.com/alexgorbatchev/toon-cli) and skill pointer do not establish publication of this Rust implementation.

# Quick Start

```sh
printf '{"name":"Ada","id":1}\n' | toon
printf '{"id":1}\n{"id":2}\n' | toon
toon < input.json > output.toon
```

Sample Output for the first command:

```text
name: Ada
id: 1
```

# Options & Flags

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--help` | `-h` | `false` | Print help |
| `--version` | `-V` | `false` | Print the raw version |

`toon skill` prints the installation command and URL for the operational reference at [skills/toon/SKILL.md](skills/toon/SKILL.md).

# License

[MIT License](LICENSE), copyright 2026 Alex Gorbatchev.
