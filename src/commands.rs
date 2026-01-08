// /Users/vincent/Documents/Github/muffin/src/commands.rs
//! commands — CLI command implementations (std-only)
//!
//! This module provides a thin, deterministic CLI dispatcher for Muffin.
//! It is designed to work without external crates and to keep the CLI contract
//! stable while internal modules evolve.
//!
//! Supported commands (current contract):
//! - `help` / `--help`
//! - `version`
//! - `build muffin [flags...]`  (Configuration phase; emits Muffinconfig.mcfg)
//! - `resolve [flags...]`       (alias of `build muffin`)
//! - `check [flags...]`         (best-effort validate; emits then deletes unless strict)
//! - `print [flags...]`         (emits + prints the resolved mcfg)
//! - `graph`                    (stub; reserved for DOT/text graph export)
//! - `fmt`                      (stub; reserved for formatting Muffinfiles)

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::build_muf;

pub const CLI_NAME: &str = "muffin";

#[derive(Debug)]
pub enum CommandError {
    Usage(String),
    Failure(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::Usage(s) => write!(f, "{s}"),
            CommandError::Failure(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for CommandError {}

pub type Result<T> = std::result::Result<T, CommandError>;

#[derive(Debug, Clone)]
pub enum Cmd {
    Help,
    Version,

    BuildMuffin(build_muf::BuildMufOptions),
    Resolve(build_muf::BuildMufOptions),
    Check(build_muf::BuildMufOptions),
    Print(build_muf::BuildMufOptions),

    Graph(GraphOptions),
    Fmt(FmtOptions),
}

#[derive(Debug, Clone, Default)]
pub struct GraphOptions {
    pub root_dir: Option<PathBuf>,
    pub format: GraphFormat,
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum GraphFormat {
    #[default]
    Text,
    Dot,
}

#[derive(Debug, Clone, Default)]
pub struct FmtOptions {
    pub file: Option<PathBuf>,
    pub check_only: bool,
    pub verbose: bool,
}

/// Entry point used by main binaries.
/// Returns a process exit code (0=OK, 2=usage, 1=runtime failure).
pub fn run_cli(args: &[String]) -> i32 {
    match dispatch(args) {
        Ok(()) => 0,
        Err(CommandError::Usage(msg)) => {
            eprintln!("{msg}");
            2
        }
        Err(CommandError::Failure(msg)) => {
            eprintln!("{msg}");
            1
        }
    }
}

/// Parse + execute command.
pub fn dispatch(args: &[String]) -> Result<()> {
    let cmd = parse_command(args)?;
    execute(cmd)
}

/// Parse argv into a command.
/// `args` is expected to be the full argv (including program name at index 0).
pub fn parse_command(args: &[String]) -> Result<Cmd> {
    if args.len() <= 1 {
        return Ok(Cmd::Help);
    }

    let sub = args[1].as_str();

    // Global help/version shortcuts
    if sub == "--help" || sub == "-h" || sub == "help" {
        return Ok(Cmd::Help);
    }
    if sub == "--version" || sub == "-V" || sub == "version" {
        return Ok(Cmd::Version);
    }

    match sub {
        // `build muffin ...`
        "build" => parse_build(&args[2..]),
        // `resolve ...` (alias)
        "resolve" => {
            let mut o = build_muf::parse_args(&args[2..].to_vec())
                .map_err(|e| CommandError::Usage(e.to_string()))?;
            // `resolve` is expected to emit; ensure print=false by default
            o.print = false;
            Ok(Cmd::Resolve(o))
        }
        // `check ...` (validate; best-effort)
        "check" => {
            let mut o = build_muf::parse_args(&args[2..].to_vec())
                .map_err(|e| CommandError::Usage(e.to_string()))?;
            o.print = false;
            Ok(Cmd::Check(o))
        }
        // `print ...` (emit + print)
        "print" => {
            let mut o = build_muf::parse_args(&args[2..].to_vec())
                .map_err(|e| CommandError::Usage(e.to_string()))?;
            o.print = true;
            Ok(Cmd::Print(o))
        }
        // stubs reserved for future
        "graph" => parse_graph(&args[2..]),
        "fmt" => parse_fmt(&args[2..]),
        other => Err(CommandError::Usage(format!(
            "{}",
            usage_unknown(other)
        ))),
    }
}

fn parse_build(rest: &[String]) -> Result<Cmd> {
    if rest.is_empty() {
        // `build` alone => help
        return Err(CommandError::Usage(usage_text().to_string()));
    }

    let tool = rest[0].as_str();
    match tool {
        "muffin" => {
            let o = build_muf::parse_args(&rest[1..].to_vec())
                .map_err(|e| CommandError::Usage(e.to_string()))?;
            Ok(Cmd::BuildMuffin(o))
        }
        // reserved for pipeline `build steel` (implemented elsewhere later)
        "steel" => Err(CommandError::Usage(
            "build steel: not implemented in muffin (Steel owns this command)".to_string(),
        )),
        other => Err(CommandError::Usage(format!(
            "unknown build target: {other}\n\n{}",
            usage_text()
        ))),
    }
}

fn parse_graph(rest: &[String]) -> Result<Cmd> {
    let mut o = GraphOptions::default();

    let mut it = rest.iter().peekable();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--root" => {
                let v = it.next().ok_or_else(|| {
                    CommandError::Usage("--root expects a path\n\n".to_string() + usage_text())
                })?;
                o.root_dir = Some(PathBuf::from(v));
            }
            "--dot" => o.format = GraphFormat::Dot,
            "--text" => o.format = GraphFormat::Text,
            "-v" | "--verbose" => o.verbose = true,
            "-h" | "--help" => return Err(CommandError::Usage(graph_help().to_string())),
            x if x.starts_with('-') => {
                return Err(CommandError::Usage(format!(
                    "unknown flag: {x}\n\n{}",
                    graph_help()
                )))
            }
            other => {
                // positional root (single)
                if o.root_dir.is_some() {
                    return Err(CommandError::Usage(format!(
                        "unexpected argument: {other}\n\n{}",
                        graph_help()
                    )));
                }
                o.root_dir = Some(PathBuf::from(other));
            }
        }
    }

    Ok(Cmd::Graph(o))
}

fn parse_fmt(rest: &[String]) -> Result<Cmd> {
    let mut o = FmtOptions::default();

    let mut it = rest.iter().peekable();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--file" => {
                let v = it.next().ok_or_else(|| {
                    CommandError::Usage("--file expects a path\n\n".to_string() + fmt_help())
                })?;
                o.file = Some(PathBuf::from(v));
            }
            "--check" => o.check_only = true,
            "-v" | "--verbose" => o.verbose = true,
            "-h" | "--help" => return Err(CommandError::Usage(fmt_help().to_string())),
            x if x.starts_with('-') => {
                return Err(CommandError::Usage(format!("unknown flag: {x}\n\n{}", fmt_help())))
            }
            other => {
                if o.file.is_some() {
                    return Err(CommandError::Usage(format!(
                        "unexpected argument: {other}\n\n{}",
                        fmt_help()
                    )));
                }
                o.file = Some(PathBuf::from(other));
            }
        }
    }

    Ok(Cmd::Fmt(o))
}

pub fn execute(cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::Help => {
            println!("{}", usage_text());
            Ok(())
        }
        Cmd::Version => {
            println!("{}", version_string());
            Ok(())
        }

        Cmd::BuildMuffin(o) => exec_build_muffin(o),
        Cmd::Resolve(o) => exec_build_muffin(o),

        Cmd::Print(mut o) => {
            o.print = true;
            exec_build_muffin(o)
        }

        Cmd::Check(mut o) => {
            // Best-effort semantics:
            // - run the resolver
            // - emit to a temp-ish path under `.muffin-cache/check/`
            // - remove after success (unless strict=false removal failures are ignored)
            let root = if o.root_dir.as_os_str().is_empty() {
                env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            } else {
                o.root_dir.clone()
            };

            let check_emit = root
                .join(".muffin-cache")
                .join("check")
                .join(build_muf::DEFAULT_EMIT_NAME);

            o.emit_path = Some(check_emit.clone());
            o.print = false;

            let _cfg = build_muf::run(&o).map_err(|e| CommandError::Failure(e.to_string()))?;

            match fs::remove_file(&check_emit) {
                Ok(_) => Ok(()),
                Err(err) => {
                    if o.strict {
                        Err(CommandError::Failure(format!(
                            "check succeeded but could not remove {}: {err}",
                            check_emit.display()
                        )))
                    } else {
                        // best-effort: ignore cleanup failure
                        Ok(())
                    }
                }
            }
        }

        Cmd::Graph(o) => exec_graph(o),
        Cmd::Fmt(o) => exec_fmt(o),
    }
}

fn exec_build_muffin(o: build_muf::BuildMufOptions) -> Result<()> {
    build_muf::run(&o).map_err(|e| CommandError::Failure(e.to_string()))?;
    Ok(())
}

fn exec_graph(o: GraphOptions) -> Result<()> {
    // Reserved: graph export of workspace/rules
    // Current behavior: deterministic placeholder.
    let root = o
        .root_dir
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    match o.format {
        GraphFormat::Text => {
            println!("graph (text): not implemented");
            println!("root: {}", root.display());
        }
        GraphFormat::Dot => {
            println!("digraph muffin {{");
            println!("  // graph export not implemented");
            println!("  root [label=\"{}\"];", escape_dot(&root.display().to_string()));
            println!("}}");
        }
    }
    Ok(())
}

fn exec_fmt(o: FmtOptions) -> Result<()> {
    // Reserved: Muffinfile formatter.
    // Current behavior: deterministic placeholder.
    let file = o.file.unwrap_or_else(|| PathBuf::from("Muffinfile"));
    if o.check_only {
        println!("fmt --check: not implemented (file={})", file.display());
    } else {
        println!("fmt: not implemented (file={})", file.display());
    }
    Ok(())
}

fn version_string() -> String {
    // Prefer build-time Cargo env if available; fallback is stable.
    let v = option_env!("CARGO_PKG_VERSION").unwrap_or("0.0.0-dev");
    let name = option_env!("CARGO_PKG_NAME").unwrap_or(CLI_NAME);
    format!("{name} {v}")
}

fn usage_unknown(cmd: &str) -> String {
    format!("unknown command: {cmd}\n\n{}", usage_text())
}

pub fn usage_text() -> &'static str {
    "muffin — Declarative configuration layer for the Muffin/Steel pipeline

USAGE:
  muffin <command> [args...]

COMMANDS:
  help
  version

  build muffin [--root <path>] [--file <path>] [--profile <name>] [--target <triple>] [--emit <path>]
              [--offline] [--strict] [--no-tool-fingerprint] [--include-hidden] [--follow-symlinks]
              [--max-depth <n>] [--print] [-v]

  resolve      Alias of: build muffin (emits Muffinconfig.mcfg)
  check        Validate best-effort (emits then deletes Muffinconfig.mcfg under .muffin-cache/check/)
  print        Emit + print Muffinconfig.mcfg to stdout

  graph        (stub) Export graph (text|dot)
  fmt          (stub) Format Muffinfile

NOTES:
  - `build muffin` performs the Configuration phase and emits Muffinconfig.mcfg.
  - Steel is responsible for the Construction phase (`build steel`)."
}

fn graph_help() -> &'static str {
    "muffin graph (stub)

USAGE:
  muffin graph [--root <path>] [--text|--dot] [-v]

FLAGS:
  --root <path>   Workspace root
  --text          Text format (default)
  --dot           DOT format
  -v, --verbose   Verbose output"
}

fn fmt_help() -> &'static str {
    "muffin fmt (stub)

USAGE:
  muffin fmt [--file <path>] [--check] [-v]

FLAGS:
  --file <path>   Muffinfile path (default: Muffinfile)
  --check         Check-only mode
  -v, --verbose   Verbose output"
}

fn escape_dot(s: &str) -> String {
    // Minimal DOT string escape.
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

/// Helper: resolve a path relative to a root if `p` is not absolute.
pub fn root_join_if_relative(root: &Path, p: impl AsRef<Path>) -> PathBuf {
    let p = p.as_ref();
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    }
}