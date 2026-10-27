use cli_command_kit::Mode;
use std::{
    ffi::OsString,
    io::{self, Read, Write},
};
use toon_cli::{command, convert, entry, help_options};

fn invoke(args: &[&str], input: &[u8], tty: bool, mode: Mode) -> (u8, Vec<u8>, String) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = entry(
        args.iter().map(OsString::from),
        &mut &input[..],
        tty,
        &mut out,
        &mut err,
        mode,
    );
    (code, out, String::from_utf8(err).unwrap())
}

#[test]
fn preserves_go_conversion_contracts() {
    for (input, expected) in [
        (
            r#"{"name":"Ada","id":1,"active":true}"#,
            "name: Ada\nid: 1\nactive: true",
        ),
        ("[1,2,3]", "[3]: 1,2,3"),
        (
            "{ // comment\n \"name\":\"Ada\",\"languages\":[\"go\",\"toon\"],}",
            "name: Ada\nlanguages[2]: go,toon",
        ),
        (
            "{\"id\":1,\"name\":\"Ada\"}\n{\"id\":2,\"name\":\"Bob\"}\n",
            "[2]{id,name}:\n  1,Ada\n  2,Bob",
        ),
        (r#"{"id":9007199254740993}"#, "id: \"9007199254740993\""),
        (
            r#"{"id":1000000000000000000000000}"#,
            "id: \"1000000000000000000000000\"",
        ),
    ] {
        assert_eq!(convert(input).unwrap(), expected, "{input}");
        for mode in [Mode::Human, Mode::Agent] {
            assert_eq!(
                invoke(&["toon"], input.as_bytes(), false, mode),
                (0, format!("{expected}\n").into_bytes(), String::new())
            );
        }
    }
}

#[test]
fn jsonc_arrays_are_single_documents_and_duplicate_keys_are_errors() {
    assert_eq!(convert("/* comment */ [1,2,3,]").unwrap(), "[3]: 1,2,3");
    assert_eq!(
        convert("{\"a\": [/* comment */ {\"x\":true},],}").unwrap(),
        "a[1]{x}:\n  true"
    );
    for input in [
        r#"{"a":1,"a":2}"#,
        r#"{"nested":{"a":1,"\u0061":2}}"#,
        "{\"a\":1,/* c */\"a\":2}",
        "{\"a\":1}\n{\"a\":1,\"a\":2}",
    ] {
        assert!(
            convert(input)
                .unwrap_err()
                .to_string()
                .contains("duplicate"),
            "{input}"
        );
    }
    assert!(convert("/* c */ 1\n2").is_err());
}

#[test]
fn ordered_nested_values_and_numeric_normalization_round_trip() {
    for (input, expected) in [
        ("9007199254740991", "9007199254740991"),
        ("-9007199254740991", "-9007199254740991"),
        ("9007199254740992", "\"9007199254740992\""),
        ("-9007199254740992", "\"-9007199254740992\""),
        ("-0", "0"),
        ("-0.0", "0"),
        ("1e3", "1000"),
        ("1.250", "1.25"),
        ("1e400", "\"1e400\""),
        ("1e-400", "0"),
        ("\u{feff}  true  ", "true"),
        ("null", "null"),
        ("\"hello\"", "hello"),
    ] {
        assert_eq!(convert(input).unwrap(), expected, "{input}");
    }
    let original = serde_json::json!({"z": {"b": "世界\nline", "a": false}, "a": [null, true, "1", {"q":"x"}]});
    let encoded = convert(&original.to_string()).unwrap();
    let decoded: serde_json::Value = serde_toon::from_str(&encoded).unwrap();
    assert_eq!(decoded, original);
    assert!(encoded.starts_with("z:\n  b:"), "{encoded}");
    assert_eq!(convert("1 true \"a\"").unwrap(), "[3]: 1,true,a");
}

#[test]
fn rejects_empty_invalid_and_json5_extensions_without_partial_output() {
    for input in [
        "",
        " \n\t ",
        "// comment",
        "{\"name\":]",
        "{\"a\":1}garbage",
        "{x:1}",
        "{'x':1}",
        "{\"a\":1 \"b\":2}",
        "0xFF",
        "+1",
        ".5",
        "5.",
        "NaN",
        "Infinity",
        r#""\x41""#,
        "/* unterminated",
        "\u{feff}\u{feff}1",
    ] {
        let result = invoke(&["toon"], input.as_bytes(), false, Mode::Agent);
        assert_eq!(result.0, 1, "{input:?}: {result:?}");
        assert!(result.1.is_empty());
        assert_eq!(
            result
                .2
                .lines()
                .filter(|line| line.starts_with("ERR:"))
                .count(),
            1
        );
        assert!(!result.2.contains("Usage:"));
    }
    assert_eq!(invoke(&["toon"], b"\xff", false, Mode::Human).0, 1);
    assert_eq!(invoke(&["toon"], b"{}", true, Mode::Human).0, 1);
    for args in [
        vec!["toon", "file.json"],
        vec!["toon", "--wat"],
        vec!["toon", "skill", "extra"],
    ] {
        assert_eq!(invoke(&args, b"{}", false, Mode::Human).0, 1);
    }
}

#[test]
fn native_help_skill_and_raw_version_work_without_input() {
    for mode in [Mode::Human, Mode::Agent] {
        assert_eq!(
            invoke(&["toon", "--version"], b"", true, mode),
            (0, b"0.1.0\n".to_vec(), String::new())
        );
        assert_eq!(
            invoke(&["toon", "skill"], b"", true, mode).1,
            help_options().skill.unwrap().as_bytes()
        );
        for args in [
            vec!["toon", "--help"],
            vec!["toon", "skill", "--help"],
            vec!["toon", "help"],
            vec!["toon", "help", "skill"],
        ] {
            let result = invoke(&args, b"", true, mode);
            assert_eq!(result.0, 0, "{args:?}: {result:?}");
            assert_eq!(
                String::from_utf8(result.1).unwrap().starts_with("ALERT:"),
                mode == Mode::Agent
            );
        }
    }
    let doc = std::fs::read_to_string("skills/toon/SKILL.md").unwrap();
    assert!(doc.contains("name: toon"));
    assert!(
        cli_command_kit::skill_omissions(&command(), &help_options(), &doc).is_empty(),
        "{:?}",
        cli_command_kit::skill_omissions(&command(), &help_options(), &doc)
    );
}

struct Broken;
impl Read for Broken {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("reader failed"))
    }
}
impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("writer failed"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn native_read_write_and_diagnostic_failures_are_returned() {
    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(
        entry(
            [OsString::from("toon")],
            &mut Broken,
            false,
            &mut out,
            &mut err,
            Mode::Human
        ),
        1
    );
    assert!(String::from_utf8(err).unwrap().contains("reader failed"));
    for args in [
        vec!["toon"],
        vec!["toon", "skill"],
        vec!["toon", "--help"],
        vec!["toon", "--version"],
    ] {
        let mut err = Vec::new();
        assert_eq!(
            entry(
                args.iter().map(OsString::from),
                &mut &b"{}"[..],
                false,
                &mut Broken,
                &mut err,
                Mode::Agent
            ),
            1
        );
        assert!(String::from_utf8(err).unwrap().contains("writer failed"));
    }
    assert_eq!(
        entry(
            [OsString::from("toon"), OsString::from("--bad")],
            &mut &b""[..],
            false,
            &mut out,
            &mut Broken,
            Mode::Human
        ),
        1
    );
}

#[test]
fn native_executable_converts_pipes_and_reports_once() {
    use std::process::{Command, Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_toon"))
        .env("AGENT", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"{\"name\":\"Ada\",\"id\":1}")
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success());
    assert_eq!(result.stdout, b"name: Ada\nid: 1\n");
    assert!(result.stderr.is_empty());
    let result = Command::new(env!("CARGO_BIN_EXE_toon"))
        .env("AGENT", "1")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert_eq!(String::from_utf8(result.stderr).unwrap().lines().count(), 1);
}
