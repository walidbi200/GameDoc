use std::fs;
use std::path::Path;

use gamedoc::collect::command::CommandOutput;
use gamedoc::probe::FixtureSystemProbe;
use gamedoc::{engine, render};
use serde::Deserialize;

#[derive(Deserialize)]
struct CommandsFixture {
    commands: Vec<CommandFixture>,
    executables: Vec<ExecutableFixture>,
    environment: Vec<EnvironmentFixture>,
}

#[derive(Deserialize)]
struct CommandFixture {
    program: String,
    args: Vec<String>,
    stdout: String,
    stderr: String,
    exit_code: i32,
}

#[derive(Deserialize)]
struct ExecutableFixture {
    name: String,
    path: String,
}

#[derive(Deserialize)]
struct EnvironmentFixture {
    name: String,
    value: String,
}

#[derive(Deserialize)]
struct FilesFixture {
    files: Vec<FileFixture>,
    directories: Vec<DirectoryFixture>,
    unreadable_directories: Vec<String>,
}

#[derive(Deserialize)]
struct FileFixture {
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct DirectoryFixture {
    path: String,
    entries: Vec<String>,
}

#[test]
fn scenarios_match_all_goldens() {
    for scenario in [
        "arch",
        "steam_native",
        "steam_flatpak",
        "steam_snap",
        "steam_unreadable",
        "steam_tools_unreadable",
    ] {
        assert_scenario(scenario);
    }
}

fn assert_scenario(name: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name);
    let fixture = load_fixture(&root);
    let report = engine::build_report(&fixture).public_safe(false);

    assert_golden(
        &root.join("expected.json"),
        &render::json::render_report(&report).expect("JSON should render"),
    );
    assert_golden(
        &root.join("expected.md"),
        &render::markdown::render_report(&report),
    );
    assert_golden(
        &root.join("expected.txt"),
        &render::terminal::render_report(
            &report,
            &render::terminal::TerminalConfig {
                summary: false,
                verbose: false,
                ascii: true,
                duration_ms: 147,
            },
        ),
    );
}

fn load_fixture(root: &Path) -> FixtureSystemProbe {
    let commands: CommandsFixture = read_json(&root.join("commands.json"));
    let files: FilesFixture = read_json(&root.join("files.json"));
    let mut fixture = FixtureSystemProbe::new();

    for command in commands.commands {
        let args: Vec<&str> = command.args.iter().map(String::as_str).collect();
        fixture = fixture.with_command(
            &command.program,
            &args,
            CommandOutput {
                stdout: command.stdout,
                stderr: command.stderr,
                exit_code: Some(command.exit_code),
            },
        );
    }
    for executable in commands.executables {
        fixture = fixture.with_executable(&executable.name, executable.path);
    }
    for environment in commands.environment {
        fixture = fixture.with_env(&environment.name, environment.value);
    }
    for file in files.files {
        fixture = fixture.with_file(file.path, file.content);
    }
    for directory in files.directories {
        fixture = fixture.with_directory(directory.path, directory.entries);
    }
    for directory in files.unreadable_directories {
        fixture = fixture.with_unreadable_directory(directory);
    }

    fixture
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    let content = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    serde_json::from_str(&content).unwrap_or_else(|error| {
        panic!("failed to parse {}: {error}", path.display());
    })
}

fn assert_golden(path: &Path, actual: &str) {
    if std::env::var_os("GAMEDOC_UPDATE_GOLDENS").is_some() {
        std::fs::write(path, actual).unwrap();
        return;
    }

    let expected = std::fs::read_to_string(path).unwrap_or_else(|_| String::new());
    if expected != actual {
        println!("Golden file mismatch for {}", path.display());
        println!("Expected:\n{expected}");
        println!("Actual:\n{actual}");
        println!("Run with GAMEDOC_UPDATE_GOLDENS=1 to update");
        panic!("Golden file mismatch");
    }

    // Validate JSON output against schema
    if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
        let schema_path = Path::new("schema/gamedoc.v2.schema.json");
        let schema_str = std::fs::read_to_string(schema_path).unwrap();
        let schema_json: serde_json::Value = serde_json::from_str(&schema_str).unwrap();
        let compiled = jsonschema::JSONSchema::compile(&schema_json).expect("Invalid JSON schema");
        let actual_json: serde_json::Value = serde_json::from_str(actual).unwrap();
        let mut error_msgs = Vec::new();
        if let Err(errors) = compiled.validate(&actual_json) {
            for e in errors {
                error_msgs.push(e.to_string());
            }
        }
        if !error_msgs.is_empty() {
            for msg in &error_msgs {
                println!("Validation error: {msg}");
            }
            panic!("JSON does not validate against schema");
        }
    }
}
