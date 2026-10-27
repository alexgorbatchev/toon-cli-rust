---
created_on: 2026-10-14 15:05
last_modified: 2026-10-14 15:05
status: current
---

# Native dependency choices and behavioral differences

| Go dependency or responsibility | Rust implementation |
| :--- | :--- |
| tailscale/hujson comments, order, and literals | jsonc-parser 0.34.0 native AST; explicit JSONC options disable its JSON5 defaults |
| encoding/json consecutive documents | serde_json native RawValue stream retains exact source numbers until normalization |
| toon-format/toon-go ordered objects and encoder | serde_json preserve_order maps and serde_toon_format 0.2.0 native encoder |
| Custom argv switches and help | cli-command-kit and native clap |
| big.Int numeric normalization | Native i64 parsing inside the safe integer range, exact raw decimal strings outside it |

All direct dependency licenses are MIT or MIT/Apache-2.0; the source MIT attribution is retained. APIs were inspected in the pinned Cargo registry source. `serde_toon_format` names its library crate `serde_toon`; its default encoder uses two spaces and commas. Its documented target is TOON v4.1. The [official specification](https://github.com/toon-format/spec/blob/main/SPEC.md) currently contains a v4.4 working draft; this project does not claim that draft's full conformance.

The Go `parseSingleDocument` checks whether the parsed value is an array and rejects valid JSONC arrays. Rust counts documents independently of their types. The Go ordered-object converter retains duplicate keys; emitting those is ambiguous for strict TOON consumers. Rust rejects decoded duplicate keys before encoding. Additional JSON5 syntax remains rejected.

The Go CLI manually prints some errors before returning them to `main`, which prints them again. Rust has one error-reporting boundary. `--version` prints the raw package version, in accordance with the shared CLI contract. The generic `skill` command and native framework help are provided alongside the original stdin-only workflow.

Integer normalization preserves the Go encoder's safe-integer policy. Decimal and exponent values retain binary64 behavior; overflow retains the original quoted numeric string. No decimal exactness is claimed for fractional or exponent input. The native TOON engine receives the fully normalized value through its intended Serde API.

Runtime tests are local Linux x86_64 evidence. Other operating systems require native verification. No Rust release has been published.
