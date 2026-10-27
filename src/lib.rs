use anyhow::{Context, Result, bail};
use cli_command_kit::{
    HelpOptions, Mode,
    clap::{Arg, ArgAction, Command},
};
use jsonc_parser::{CollectOptions, ParseOptions, ast, parse_to_ast};
use serde_json::{Map, Number, Value, value::RawValue};
use std::{
    ffi::OsString,
    io::{Read, Write},
};

const SKILL: &str = "Install the toon skill using the Skills CLI:\n\n  npx skills add alexgorbatchev/toon-cli --skill toon\n\nSkill:\n\n  https://github.com/alexgorbatchev/toon-cli/tree/main/skills/toon\n";

pub fn command() -> Command {
    Command::new("toon")
        .about("Convert piped JSON, JSONC, or NDJSON into TOON")
        .arg(
            Arg::new("version")
                .long("version")
                .short('V')
                .action(ArgAction::SetTrue)
                .exclusive(true)
                .help("Print the raw version"),
        )
}

pub fn help_options() -> HelpOptions {
    HelpOptions {
        skill: Some(SKILL.into()),
        ..HelpOptions::default()
    }
}

fn number(raw: &str) -> Result<Value> {
    if raw.contains(['.', 'e', 'E']) {
        let value: f64 = raw.parse().context("invalid JSON number")?;
        return Ok(match Number::from_f64(value) {
            Some(value) => Value::Number(value),
            None => Value::String(raw.into()),
        });
    }
    match raw.parse::<i64>() {
        Ok(value) if value.unsigned_abs() <= 9_007_199_254_740_991 => Ok(value.into()),
        _ => Ok(Value::String(raw.into())),
    }
}

fn normalize(value: ast::Value<'_>) -> Result<Value> {
    Ok(match value {
        ast::Value::StringLit(value) => Value::String(value.value.into_owned()),
        ast::Value::NumberLit(value) => number(value.value)?,
        ast::Value::BooleanLit(value) => Value::Bool(value.value),
        ast::Value::NullKeyword(_) => Value::Null,
        ast::Value::Array(value) => Value::Array(
            value
                .elements
                .into_iter()
                .map(normalize)
                .collect::<Result<_>>()?,
        ),
        ast::Value::Object(value) => {
            let mut output = Map::new();
            for property in value.properties {
                let name = property.name.into_string();
                if output.contains_key(&name) {
                    bail!("duplicate object key {name:?}");
                }
                output.insert(name, normalize(property.value)?);
            }
            Value::Object(output)
        }
    })
}

fn document(text: &str, jsonc: bool) -> Result<Value> {
    let options = ParseOptions {
        allow_comments: jsonc,
        allow_trailing_commas: jsonc,
        allow_loose_object_property_names: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
        allow_bare_decimal_point_numbers: false,
        allow_non_finite_numbers: false,
        allow_extended_string_escapes: false,
    };
    let parsed = parse_to_ast(text, &CollectOptions::default(), &options).context("parse input")?;
    normalize(parsed.value.context("input is empty")?)
}

/// Convert ordered JSON values to native TOON; JSONC accepts one document.
pub fn convert(input: &str) -> Result<String> {
    let text = input.strip_prefix('\u{feff}').unwrap_or(input).trim();
    if text.is_empty() {
        bail!("input is empty");
    }
    let stream = serde_json::Deserializer::from_str(text)
        .into_iter::<Box<RawValue>>()
        .collect::<serde_json::Result<Vec<_>>>();
    let value = match stream {
        Ok(raw) => {
            let mut values = raw
                .iter()
                .map(|raw| document(raw.get(), false))
                .collect::<Result<Vec<_>>>()?;
            if values.len() == 1 {
                values.remove(0)
            } else {
                Value::Array(values)
            }
        }
        Err(_) => document(text, true)?,
    };
    serde_toon::to_string(&value).context("encode TOON")
}

fn perform(
    matches: &cli_command_kit::clap::ArgMatches,
    options: &HelpOptions,
    reader: &mut impl Read,
    terminal: bool,
    output: &mut impl Write,
) -> Result<()> {
    if cli_command_kit::write_skill(matches, options, output)? {
        return Ok(());
    }
    if matches.get_flag("version") {
        writeln!(output, "{}", env!("CARGO_PKG_VERSION"))?;
        return Ok(());
    }
    if terminal {
        bail!("no input detected on stdin; pipe JSON, JSONC, or NDJSON");
    }
    let mut input = String::new();
    reader.read_to_string(&mut input).context("read stdin")?;
    let encoded = convert(&input)?;
    output
        .write_all(encoded.as_bytes())
        .context("write output")?;
    if !encoded.ends_with('\n') {
        output.write_all(b"\n").context("write trailing newline")?;
    }
    Ok(())
}

/// Native clap boundary; each error is reported once, with exit status 1.
pub fn entry(
    args: impl IntoIterator<Item = OsString>,
    reader: &mut impl Read,
    terminal: bool,
    output: &mut impl Write,
    diagnostics: &mut impl Write,
    mode: Mode,
) -> u8 {
    let options = help_options();
    let root =
        cli_command_kit::setup(command(), mode, &options).expect("valid static command tree");
    let result = match root.try_get_matches_from(args) {
        Ok(matches) => perform(&matches, &options, reader, terminal, output),
        Err(error) if !error.use_stderr() => output
            .write_all(error.to_string().as_bytes())
            .map_err(Into::into),
        Err(error) => {
            let _ = write!(diagnostics, "{error}");
            return 1;
        }
    };
    match result {
        Ok(()) => 0,
        Err(error) => {
            let prefix = if mode == Mode::Agent {
                "ERR:"
            } else {
                "[ERROR]"
            };
            let _ = writeln!(diagnostics, "{prefix} {error:#}");
            1
        }
    }
}
