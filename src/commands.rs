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
//! - `build muffin [flags...]`  (Configuration phase; emits Muffinconfig.mff)
//! - `resolve [flags...]`       (alias of `build muffin`)
//! - `check [flags...]`         (best-effort validate; emits then deletes unless strict)
//! - `print [flags...]`         (emits + prints the resolved mcfg)
//! - `graph`                    (stub; reserved for DOT/text graph export)
//! - `fmt`                      (stub; reserved for formatting Muffinfiles)

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::build_muf;
use crate::build_muf::BuildMufError;
use crate::run_muf;

pub const CLI_NAME: &str = "muffin";

#[derive(Debug)]
pub enum CommandError {
    Usage(String),
    Failure { code: i32, msg: String },
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::Usage(s) => write!(f, "{s}"),
            CommandError::Failure { msg, .. } => write!(f, "{msg}"),
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
    Run(run_muf::RunOptions),

    Doctor(DoctorOptions),
    Cache(CacheOptions),

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

#[derive(Debug, Clone, Default)]
pub struct DoctorOptions {
    pub root_dir: Option<PathBuf>,
    pub verbose: bool,
    pub json: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum CacheAction {
    Status,
    Clear,
}

#[derive(Debug, Clone)]
pub struct CacheOptions {
    pub root_dir: Option<PathBuf>,
    pub action: CacheAction,
    pub verbose: bool,
    pub json: bool,
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
        Err(CommandError::Failure { code, msg }) => {
            eprintln!("{msg}");
            if code <= 0 { 1 } else { code }
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
        return Err(CommandError::Usage(err_msg(
            "U001",
            "missing command. Run `muffin -help` for the list of commands.",
        )));
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
            let mut o = parse_build_args(&args[2..])?;
            // `resolve` is expected to emit; ensure print=false by default
            o.print = false;
            Ok(Cmd::Resolve(o))
        }
        // `check ...` (validate; best-effort)
        "check" => {
            let mut o = parse_build_args(&args[2..])?;
            o.print = false;
            Ok(Cmd::Check(o))
        }
        // `print ...` (emit + print)
        "print" => {
            let mut o = parse_build_args(&args[2..])?;
            o.print = true;
            Ok(Cmd::Print(o))
        }
        // `run ...` (execute tools)
        "run" => {
            let o = parse_run_args(&args[2..])?;
            Ok(Cmd::Run(o))
        }
        "doctor" => parse_doctor(&args[2..]),
        "cache" => parse_cache(&args[2..]),
        // stubs reserved for future
        "graph" => parse_graph(&args[2..]),
        "fmt" => parse_fmt(&args[2..]),
        other => Err(CommandError::Usage(usage_unknown(other))),
    }
}

fn parse_build(rest: &[String]) -> Result<Cmd> {
    if matches!(rest.first().map(String::as_str), Some("-h" | "--help" | "help")) {
        return Err(CommandError::Usage(usage_text().to_string()));
    }
    if rest.is_empty() {
        // `build` alone => help
        return Err(CommandError::Usage(usage_text().to_string()));
    }

    let tool = rest[0].as_str();
    match tool {
        "muffin" => {
            let o = parse_build_args(&rest[1..])?;
            Ok(Cmd::BuildMuffin(o))
        }
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

fn parse_doctor(rest: &[String]) -> Result<Cmd> {
    let mut o = DoctorOptions::default();

    let mut it = rest.iter().peekable();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--root" => {
                let v = it.next().ok_or_else(|| {
                    CommandError::Usage("--root expects a path\n\n".to_string() + doctor_help())
                })?;
                o.root_dir = Some(PathBuf::from(v));
            }
            "--json" => o.json = true,
            "-v" | "--verbose" => o.verbose = true,
            "-h" | "--help" => return Err(CommandError::Usage(doctor_help().to_string())),
            x if x.starts_with('-') => {
                return Err(CommandError::Usage(format!(
                    "unknown flag: {x}\n\n{}",
                    doctor_help()
                )))
            }
            other => {
                return Err(CommandError::Usage(format!(
                    "unexpected argument: {other}\n\n{}",
                    doctor_help()
                )))
            }
        }
    }

    Ok(Cmd::Doctor(o))
}

fn parse_cache(rest: &[String]) -> Result<Cmd> {
    if rest.is_empty() || matches!(rest.first().map(String::as_str), Some("-h" | "--help" | "help")) {
        return Err(CommandError::Usage(cache_help().to_string()));
    }

    let action = match rest[0].as_str() {
        "status" => CacheAction::Status,
        "clear" => CacheAction::Clear,
        other => {
            return Err(CommandError::Usage(format!(
                "unknown cache action: {other}\n\n{}",
                cache_help()
            )))
        }
    };

    let mut o = CacheOptions {
        root_dir: None,
        action,
        verbose: false,
        json: false,
    };

    let mut it = rest[1..].iter().peekable();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--root" => {
                let v = it.next().ok_or_else(|| {
                    CommandError::Usage("--root expects a path\n\n".to_string() + cache_help())
                })?;
                o.root_dir = Some(PathBuf::from(v));
            }
            "--json" => o.json = true,
            "-v" | "--verbose" => o.verbose = true,
            "-h" | "--help" => return Err(CommandError::Usage(cache_help().to_string())),
            x if x.starts_with('-') => {
                return Err(CommandError::Usage(format!(
                    "unknown flag: {x}\n\n{}",
                    cache_help()
                )))
            }
            other => {
                return Err(CommandError::Usage(format!(
                    "unexpected argument: {other}\n\n{}",
                    cache_help()
                )))
            }
        }
    }

    Ok(Cmd::Cache(o))
}

fn parse_build_args(rest: &[String]) -> Result<build_muf::BuildMufOptions> {
    build_muf::parse_args(rest).map_err(map_build_error)
}

fn parse_run_args(rest: &[String]) -> Result<run_muf::RunOptions> {
    let mut o = run_muf::RunOptions::default();

    let mut positional_root_set = false;
    let mut it = rest.iter().peekable();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--root" => {
                let v = it.next().ok_or_else(|| {
                    CommandError::Usage("--root expects a path\n\n".to_string() + run_help())
                })?;
                o.root_dir = PathBuf::from(v);
                positional_root_set = true;
            }
            "--file" => {
                let v = it.next().ok_or_else(|| {
                    CommandError::Usage("--file expects a path\n\n".to_string() + run_help())
                })?;
                o.muffin_file = Some(PathBuf::from(v));
            }
            "--profile" => {
                let v = it.next().ok_or_else(|| {
                    CommandError::Usage("--profile expects a name\n\n".to_string() + run_help())
                })?;
                o.profile = Some(v.to_string());
            }
            "--toolchain" => {
                let v = it.next().ok_or_else(|| {
                    CommandError::Usage("--toolchain expects a path\n\n".to_string() + run_help())
                })?;
                o.toolchain_dir = Some(PathBuf::from(v));
            }
            "--bake" => {
                let v = it.next().ok_or_else(|| {
                    CommandError::Usage("--bake expects a name\n\n".to_string() + run_help())
                })?;
                o.bakes.push(v.to_string());
            }
            "--all" => o.run_all = true,
            "--no-cache" => o.no_cache = true,
            "--print" => o.dry_run = true,
            "-v" | "--verbose" => o.verbose = true,
            "--log" => {
                let v = it.next().ok_or_else(|| {
                    CommandError::Usage("--log expects a path\n\n".to_string() + run_help())
                })?;
                o.log_path = Some(PathBuf::from(v));
            }
            "--log-mode" => {
                let v = it.next().ok_or_else(|| {
                    CommandError::Usage("--log-mode expects append|truncate\n\n".to_string() + run_help())
                })?;
                o.log_mode = match v.as_str() {
                    "append" => run_muf::LogMode::Append,
                    "truncate" => run_muf::LogMode::Truncate,
                    _ => {
                        return Err(CommandError::Usage(
                            "invalid --log-mode (use append|truncate)\n\n".to_string() + run_help(),
                        ))
                    }
                };
            }
            "-h" | "--help" => return Err(CommandError::Usage(run_help().to_string())),
            x if x.starts_with('-') => {
                return Err(CommandError::Usage(format!(
                    "unknown flag: {x}\n\n{}",
                    run_help()
                )))
            }
            other => {
                if positional_root_set {
                    return Err(CommandError::Usage(format!(
                        "unexpected argument: {other}\n\n{}",
                        run_help()
                    )));
                }
                o.root_dir = PathBuf::from(other);
                positional_root_set = true;
            }
        }
    }

    Ok(o)
}

fn map_build_error(err: BuildMufError) -> CommandError {
    match err {
        BuildMufError::Arg { msg } => {
            if msg.contains("build muffin —") {
                CommandError::Usage(msg)
            } else {
                CommandError::Usage(format!(
                    "{}\n\n{}",
                    err_msg("U002", msg),
                    build_muf::help_text()
                ))
            }
        }
        other => CommandError::Failure {
            code: 1,
            msg: err_msg("E001", other.to_string()),
        },
    }
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
        Cmd::Run(o) => exec_run_muf(o),
        Cmd::Doctor(o) => exec_doctor(o),
        Cmd::Cache(o) => exec_cache(o),

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

            let _cfg = build_muf::run(&o).map_err(|e| CommandError::Failure {
                code: 1,
                msg: err_msg("E001", e.to_string()),
            })?;

            match fs::remove_file(&check_emit) {
                Ok(_) => Ok(()),
                Err(err) => {
                    if o.strict {
                        Err(CommandError::Failure {
                            code: 1,
                            msg: err_msg(
                                "E001",
                                format!(
                                    "check succeeded but could not remove {}: {err}",
                                    check_emit.display()
                                ),
                            ),
                        })
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
    build_muf::run(&o).map_err(|e| CommandError::Failure {
        code: 1,
        msg: err_msg("E001", e.to_string()),
    })?;
    Ok(())
}

fn exec_run_muf(o: run_muf::RunOptions) -> Result<()> {
    run_muf::run(&o).map_err(|e| map_run_error(e))?;
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
    err_msg(
        "U001",
        format!("unknown command: {cmd}. Run `muffin -help` for the list of commands."),
    )
}

pub fn usage_text() -> &'static str {
    "muffin — Declarative configuration layer for the Muffin pipeline

USAGE:
  muffin <command> [args...]

COMMANDS:
  help, -h, --help
  version, -V, --version

  build muffin [--root <path>] [--file <path>] [--profile <name>] [--target <triple>] [--emit <path>]
              [--offline] [--strict] [--no-tool-fingerprint] [--include-hidden] [--follow-symlinks]
              [--max-depth <n>] [--print] [-v]

  resolve      Alias of: build muffin (emits Muffinconfig.mff)
  check        Validate best-effort (emits then deletes Muffinconfig.mff under .muffin-cache/check/)
  print        Emit + print Muffinconfig.mff to stdout
  run          Execute tool steps from MuffinConfig.muf (runner)
  doctor       Diagnostics for PATH, tools, and config
  cache        Cache status/clear

  graph        (stub) Export graph (text|dot)
  fmt          (stub) Format Muffinfile

NOTES:
  - `build muffin` performs the Configuration phase and emits Muffinconfig.mff.
  - Running `muffin` with no args shows this help."
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

fn run_help() -> &'static str {
    "muffin run — Execute tool steps from MuffinConfig.muf

USAGE:
  muffin run [--root <path>] [--file <path>] [--profile <name>] [--toolchain <path>] [--bake <name>] [--all] [--print] [--no-cache] [--log <path>] [--log-mode <m>] [-v]

FLAGS:
  --root <path>     Workspace root (default: cwd)
  --file <path>     Explicit Muffinfile path (default: MuffinConfig.muf under root)
  --profile <name>  Select profile (default: workspace.profile or debug)
  --toolchain <p>   Toolchain directory (overrides PATH lookup for tools)
  --bake <name>     Run a specific bake (repeatable)
  --all             Run all bakes in file order (with deps)
  --print           Dry-run: print commands only
  --no-cache        Disable incremental skip
  --log <path>      Write run log to a specific .mff path
  --log-mode <m>    Log write mode: append (default) or truncate
  -v, --verbose     Verbose output"
}

fn doctor_help() -> &'static str {
    "muffin doctor

USAGE:
  muffin doctor [--root <path>] [--json] [-v]

FLAGS:
  --root <path>   Workspace root (default: cwd)
  --json          JSON output (machine-friendly)
  -v, --verbose   Verbose output"
}

fn cache_help() -> &'static str {
    "muffin cache

USAGE:
  muffin cache <status|clear> [--root <path>] [--json] [-v]

FLAGS:
  --root <path>   Workspace root (default: cwd)
  --json          JSON output (machine-friendly)
  -v, --verbose   Verbose output"
}

fn err_msg(code: &str, msg: impl Into<String>) -> String {
    format!("error[{code}]: {}", msg.into())
}

fn map_run_error(err: run_muf::RunError) -> CommandError {
    match err {
        run_muf::RunError::Config { msg, help } => {
            let msg = if let Some(h) = help {
                format!("{}\nhelp: {h}", err_msg("C001", msg))
            } else {
                err_msg("C001", msg)
            };
            CommandError::Failure { code: 2, msg }
        }
        run_muf::RunError::Parse { path, msg } => CommandError::Failure {
            code: 2,
            msg: err_msg("P001", format!("parse {}: {msg}", path.display())),
        },
        run_muf::RunError::Io { op, path, err } => CommandError::Failure {
            code: 4,
            msg: err_msg("IO01", format!("{op} {}: {err}", path.display())),
        },
        run_muf::RunError::Exec { cmd, status, stderr } => {
            let mut s = match status {
                Some(code) => format!("command failed ({code}): {cmd}"),
                None => format!("command failed: {cmd}"),
            };
            if let Some(err) = stderr {
                if !err.trim().is_empty() {
                    s.push('\n');
                    s.push_str(err.trim_end());
                }
            }
            CommandError::Failure {
                code: 3,
                msg: err_msg("X001", s),
            }
        }
    }
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

fn exec_doctor(o: DoctorOptions) -> Result<()> {
    let root = o
        .root_dir
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let config_path = root.join("MuffinConfig.muf");

    let tools = ["muffin", "gcc", "ar"];
    if o.json {
        let config_exists = config_path.exists();
        let mut out = String::new();
        out.push_str("{\"root\":\"");
        out.push_str(&json_escape(&root.display().to_string()));
        out.push_str("\",\"config\":{\"path\":\"");
        out.push_str(&json_escape(&config_path.display().to_string()));
        out.push_str("\",\"exists\":");
        out.push_str(if config_exists { "true" } else { "false" });
        out.push_str("},\"tools\":[");
        let mut first = true;
        for tool in tools {
            if !first {
                out.push(',');
            }
            first = false;
            match find_in_path(tool) {
                Some(path) => {
                    out.push_str("{\"name\":\"");
                    out.push_str(tool);
                    out.push_str("\",\"ok\":true,\"path\":\"");
                    out.push_str(&json_escape(&path.display().to_string()));
                    out.push_str("\"}");
                }
                None => {
                    out.push_str("{\"name\":\"");
                    out.push_str(tool);
                    out.push_str("\",\"ok\":false}");
                }
            }
        }
        out.push_str("]}");
        println!("{out}");
        return Ok(());
    }

    println!("muffin doctor");
    println!("root: {}", root.display());
    if config_path.exists() {
        println!("config: ok ({})", config_path.display());
    } else {
        println!("config: missing ({})", config_path.display());
    }

    for tool in tools {
        match find_in_path(tool) {
            Some(path) => {
                if o.verbose {
                    println!("tool: {tool} => {}", path.display());
                } else {
                    println!("tool: {tool} ok");
                }
            }
            None => println!("tool: {tool} missing"),
        }
    }

    Ok(())
}

fn exec_cache(o: CacheOptions) -> Result<()> {
    let root = o
        .root_dir
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let cache_dir = root.join(".muffin-cache");

    match o.action {
        CacheAction::Status => {
            if o.json {
                let exists = cache_dir.exists();
                let (files, bytes) = if exists {
                    dir_stats(&cache_dir)?
                } else {
                    (0, 0)
                };
                println!(
                    "{{\"cache_dir\":\"{}\",\"exists\":{},\"files\":{},\"bytes\":{}}}",
                    json_escape(&cache_dir.display().to_string()),
                    if exists { "true" } else { "false" },
                    files,
                    bytes
                );
                return Ok(());
            }
            if !cache_dir.exists() {
                println!("cache: empty ({})", cache_dir.display());
                return Ok(());
            }
            let (files, bytes) = dir_stats(&cache_dir)?;
            println!("cache: {}", cache_dir.display());
            println!("files: {files}");
            println!("bytes: {bytes}");
        }
        CacheAction::Clear => {
            let existed = cache_dir.exists();
            if !existed {
                if o.json {
                    println!(
                        "{{\"cache_dir\":\"{}\",\"cleared\":false,\"exists\":false}}",
                        json_escape(&cache_dir.display().to_string())
                    );
                } else {
                    println!("cache: empty ({})", cache_dir.display());
                }
                return Ok(());
            }
            fs::remove_dir_all(&cache_dir).map_err(|e| CommandError::Failure {
                code: 4,
                msg: err_msg("IO01", format!("remove {}: {e}", cache_dir.display())),
            })?;
            if o.json {
                println!(
                    "{{\"cache_dir\":\"{}\",\"cleared\":true,\"exists\":true}}",
                    json_escape(&cache_dir.display().to_string())
                );
            } else {
                println!("cache cleared: {}", cache_dir.display());
            }
        }
    }

    Ok(())
}

fn dir_stats(path: &Path) -> Result<(u64, u64)> {
    let mut files = 0u64;
    let mut bytes = 0u64;
    let mut stack = vec![path.to_path_buf()];

    while let Some(p) = stack.pop() {
        let entries = fs::read_dir(&p).map_err(|e| CommandError::Failure {
            code: 4,
            msg: err_msg("IO01", format!("read {}: {e}", p.display())),
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| CommandError::Failure {
                code: 4,
                msg: err_msg("IO01", format!("read {}: {e}", p.display())),
            })?;
            let meta = entry.metadata().map_err(|e| CommandError::Failure {
                code: 4,
                msg: err_msg("IO01", format!("stat {}: {e}", entry.path().display())),
            })?;
            if meta.is_dir() {
                stack.push(entry.path());
            } else {
                files += 1;
                bytes += meta.len();
            }
        }
    }

    Ok((files, bytes))
}

fn find_in_path(tool: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(tool);
        if candidate.exists() {
            return Some(candidate);
        }
        if cfg!(windows) && !tool.to_ascii_lowercase().ends_with(".exe") {
            let candidate = dir.join(format!("{tool}.exe"));
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

fn json_escape(s: &str) -> String {
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
