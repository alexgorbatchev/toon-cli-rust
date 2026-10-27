---
created_on: 2026-10-14 15:05
last_modified: 2026-10-14 15:05
status: current
---

# toon

Independent stdin-only Rust JSON, JSONC, and NDJSON to TOON converter. Inherit [../AGENTS.md](../AGENTS.md).

## Commands

- Build: `just build`, producing `bin/toon`.
- Run: `printf '{"name":"Ada","id":1}\n' | just run`.
- Agent mode: `printf '{"id":1}\n' | just run-ai`.
- Test: `just test`; lint: `just lint`; full gate: `just check`.
- Coverage: `just coverage`, at least 90% lines.

## Contracts

- Use native jsonc-parser ASTs to retain numeric lexemes and detect duplicate keys before insertion. Use ordered serde_json maps and native serde_toon encoding.
- Keep JSONC single-document; count JSON documents independently of their value type. Arrays are valid single documents.
- Preserve exact integer digits outside ±9007199254740991 as strings before native TOON encoding. Decimal and exponent values follow binary64 semantics; overflow remains a string.
- Configure JSONC explicitly: allow comments and trailing commas, and reject the additional JSON5 defaults of jsonc-parser.
- Keep conversion stdin-only. Native clap handles all arguments and shared cli-command-kit handles help, skill pointers, and agent mode. Report errors once at `entry` and return output-write failures.
- Update `skills/toon/SKILL.md` and its `last_modified` whenever commands, arguments, flags, defaults, accepted formats, environment variables, output, or side effects change. Verify coverage against live command metadata.
- Runtime-result changes require behavioral test-file changes and at least 90% line coverage; scripts are excluded. Prove tests with disabled behavior, restore it, and rerun the gate.
- Keep temporary evidence under `.tmp/`; scope formatters to this project. Keep progress outside agent instructions.
- Never publish releases, tags, packages, or deployments without explicit authorization. Existing Go release URLs do not distribute this Rust project.
