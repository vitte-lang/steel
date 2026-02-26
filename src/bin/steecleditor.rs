use std::fs;
use std::io::{self, stdout, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{execute, queue};
use regex::Regex;
use walkdir::WalkDir;

mod language_data;

const TAB_WIDTH: usize = 2;
const BLOCK_KEYWORDS: &[&str] = &[
    "[workspace]",
    "[profile]",
    "[tool]",
    "[bake]",
    "[run]",
    "[export]",
    "[cache]",
    "[env]",
    "[targets]",
    "[target]",
];
const DIRECTIVES: &[&str] = &[
    ".set",
    ".make",
    ".takes",
    ".emits",
    ".output",
    ".exec",
    ".ref",
    ".needs",
    ".env",
    ".cwd",
    ".shell",
    ".when",
    ".desc",
    ".arg",
    ".cache",
    ".path",
];

const STEELCONF_SNIPPETS: &[Snippet] = &[
    Snippet {
        trigger: "workspace",
        label: "workspace block",
        body: "[workspace]\n  .set name \"project_name\"\n  .set root \".\"\n  .set target_dir \"target\"\n  .set profile \"debug\"\n..\n",
    },
    Snippet {
        trigger: "profile",
        label: "profile block",
        body: "[profile debug]\n  .set opt 0\n  .set debug 1\n..\n",
    },
    Snippet {
        trigger: "tool",
        label: "tool block",
        body: "[tool name]\n  .exec \"tool_exec\"\n..\n",
    },
    Snippet {
        trigger: "bake",
        label: "bake block",
        body: "[bake name]\n  .make src glob \"src/*.c\"\n  [run tool]\n    .set \"-O2\" 1\n  ..\n  .output exe \"target/out/app\"\n..\n",
    },
    Snippet {
        trigger: "run",
        label: "run block",
        body: "[run tool]\n  .set \"flag\" value\n..\n",
    },
    Snippet {
        trigger: "export",
        label: "export block",
        body: "[export]\n  .ref target\n..\n",
    },
    Snippet {
        trigger: ".set",
        label: "directive .set",
        body: ".set key \"value\"",
    },
    Snippet {
        trigger: ".make",
        label: "directive .make",
        body: ".make name cglob \"src/**/*.c\"",
    },
    Snippet {
        trigger: ".takes",
        label: "directive .takes",
        body: ".takes src as \"@args\"",
    },
    Snippet {
        trigger: ".emits",
        label: "directive .emits",
        body: ".emits exe as \"-o\"",
    },
    Snippet {
        trigger: ".output",
        label: "directive .output",
        body: ".output exe \"target/out/app\"",
    },
    Snippet {
        trigger: ".exec",
        label: "directive .exec",
        body: ".exec \"tool_exec\"",
    },
    Snippet {
        trigger: ".ref",
        label: "directive .ref",
        body: ".ref target",
    },
];

const C_KEYWORDS: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else",
    "enum", "extern", "float", "for", "goto", "if", "inline", "int", "long", "register",
    "restrict", "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef",
    "union", "unsigned", "void", "volatile", "while",
];
const CPP_KEYWORDS: &[&str] = &[
    "alignas", "alignof", "and", "and_eq", "asm", "auto", "bitand", "bitor", "bool", "break",
    "case", "catch", "char", "char16_t", "char32_t", "class", "compl", "const", "consteval",
    "constexpr", "constinit", "const_cast", "continue", "decltype", "default", "delete", "do",
    "double", "dynamic_cast", "else", "enum", "explicit", "export", "extern", "false", "float",
    "for", "friend", "goto", "if", "inline", "int", "long", "mutable", "namespace", "new",
    "noexcept", "not", "not_eq", "nullptr", "operator", "or", "or_eq", "private", "protected",
    "public", "register", "reinterpret_cast", "return", "short", "signed", "sizeof", "static",
    "static_assert", "static_cast", "struct", "switch", "template", "this", "thread_local",
    "throw", "true", "try", "typedef", "typeid", "typename", "union", "unsigned", "using",
    "virtual", "void", "volatile", "wchar_t", "while", "xor", "xor_eq",
];
const PY_KEYWORDS: &[&str] = &[
    "and", "as", "assert", "break", "class", "continue", "def", "del", "elif", "else", "except",
    "False", "finally", "for", "from", "global", "if", "import", "in", "is", "lambda", "None",
    "nonlocal", "not", "or", "pass", "raise", "return", "True", "try", "while", "with", "yield",
];
const JAVA_KEYWORDS: &[&str] = &[
    "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class", "const",
    "continue", "default", "do", "double", "else", "enum", "extends", "final", "finally", "float",
    "for", "goto", "if", "implements", "import", "instanceof", "int", "interface", "long",
    "native", "new", "null", "package", "private", "protected", "public", "return", "short",
    "static", "strictfp", "super", "switch", "synchronized", "this", "throw", "throws",
    "transient", "try", "void", "volatile", "while",
];
const OCAML_KEYWORDS: &[&str] = &[
    "and", "as", "assert", "begin", "class", "constraint", "do", "done", "downto", "else",
    "end", "exception", "external", "false", "for", "fun", "function", "functor", "if", "in",
    "include", "inherit", "initializer", "lazy", "let", "match", "method", "module", "mutable",
    "new", "object", "of", "open", "or", "private", "rec", "sig", "struct", "then", "to",
    "true", "try", "type", "val", "virtual", "when", "while", "with",
];
const ZIG_KEYWORDS: &[&str] = &[
    "addrspace", "align", "allowzero", "and", "anyframe", "anytype", "asm", "async", "await",
    "break", "callconv", "catch", "comptime", "const", "continue", "defer", "else", "enum",
    "errdefer", "error", "export", "extern", "false", "for", "if", "inline", "linksection",
    "noalias", "noinline", "nosuspend", "null", "opaque", "or", "orelse", "packed", "pub",
    "resume", "return", "struct", "suspend", "switch", "test", "threadlocal", "true", "try",
    "union", "unreachable", "usingnamespace", "var", "volatile", "while",
];
const CSHARP_KEYWORDS: &[&str] = &[
    "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char", "checked",
    "class", "const", "continue", "decimal", "default", "delegate", "do", "double", "else",
    "enum", "event", "explicit", "extern", "false", "finally", "fixed", "float", "for",
    "foreach", "goto", "if", "implicit", "in", "int", "interface", "internal", "is", "lock",
    "long", "namespace", "new", "null", "object", "operator", "out", "override", "params",
    "private", "protected", "public", "readonly", "ref", "return", "sbyte", "sealed", "short",
    "sizeof", "stackalloc", "static", "string", "struct", "switch", "this", "throw", "true",
    "try", "typeof", "uint", "ulong", "unchecked", "unsafe", "ushort", "using", "virtual",
    "void", "volatile", "while",
];
const JS_KEYWORDS: &[&str] = &[
    "break", "case", "catch", "class", "const", "continue", "debugger", "default", "delete",
    "do", "else", "export", "extends", "finally", "for", "function", "if", "import", "in",
    "instanceof", "let", "new", "return", "super", "switch", "this", "throw", "try", "typeof",
    "var", "void", "while", "with", "yield", "async", "await",
];
const TS_KEYWORDS: &[&str] = &[
    "abstract", "as", "asserts", "break", "case", "catch", "class", "const", "continue",
    "debugger", "declare", "default", "delete", "do", "else", "enum", "export", "extends",
    "finally", "for", "from", "function", "if", "implements", "import", "in", "infer",
    "instanceof", "interface", "is", "keyof", "let", "module", "namespace", "new", "private",
    "protected", "public", "readonly", "return", "satisfies", "static", "super", "switch",
    "this", "throw", "try", "type", "typeof", "var", "void", "while", "with", "yield", "async",
    "await",
];
const GO_KEYWORDS: &[&str] = &[
    "break", "case", "chan", "const", "continue", "default", "defer", "else", "fallthrough",
    "for", "func", "go", "goto", "if", "import", "interface", "map", "package", "range",
    "return", "select", "struct", "switch", "type", "var",
];
const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
    "where", "while", "async", "await", "dyn",
];
const PHP_KEYWORDS: &[&str] = &[
    "abstract", "and", "array", "as", "break", "callable", "case", "catch", "class", "clone",
    "const", "continue", "declare", "default", "do", "echo", "else", "elseif", "empty", "endfor",
    "endforeach", "endif", "endswitch", "endwhile", "eval", "exit", "extends", "final", "for",
    "foreach", "function", "global", "if", "implements", "include", "include_once", "instanceof",
    "interface", "isset", "list", "match", "namespace", "new", "or", "print", "private",
    "protected", "public", "readonly", "require", "require_once", "return", "static", "switch",
    "throw", "trait", "try", "use", "var", "while", "xor", "yield",
];
const LUA_KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if",
    "in", "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];
const PERL_KEYWORDS: &[&str] = &[
    "if", "elsif", "else", "unless", "while", "for", "foreach", "continue", "last", "next",
    "redo", "goto", "my", "our", "state", "local", "sub", "package", "use", "require", "return",
    "given", "when", "default", "do",
];
const RUBY_KEYWORDS: &[&str] = &[
    "BEGIN", "END", "alias", "and", "begin", "break", "case", "class", "def", "defined?", "do",
    "else", "elsif", "end", "ensure", "false", "for", "if", "in", "module", "next", "nil", "not",
    "or", "redo", "rescue", "retry", "return", "self", "super", "then", "true", "undef", "unless",
    "until", "when", "while", "yield",
];
const HASKELL_KEYWORDS: &[&str] = &[
    "case", "class", "data", "default", "deriving", "do", "else", "if", "import", "in", "infix",
    "infixl", "infixr", "instance", "let", "module", "newtype", "of", "then", "type", "where",
];
const SHELL_KEYWORDS: &[&str] = &[
    "if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done", "case", "esac",
    "function", "in", "select", "time", "coproc",
];
const COFFEE_KEYWORDS: &[&str] = &[
    "if", "then", "else", "unless", "for", "while", "loop", "switch", "when", "try", "catch",
    "finally", "class", "extends", "super", "new", "return", "break", "continue", "throw", "in",
    "of", "by", "and", "or", "is", "isnt", "not", "yes", "no", "on", "off", "null", "undefined",
];
const PASCAL_KEYWORDS: &[&str] = &[
    "program", "unit", "interface", "implementation", "uses", "begin", "end", "var", "const",
    "type", "record", "class", "object", "set", "array", "of", "string", "integer", "real",
    "boolean", "char", "function", "procedure", "constructor", "destructor", "if", "then", "else",
    "case", "for", "to", "downto", "while", "repeat", "until", "try", "except", "finally", "with",
];
const ALGOL_KEYWORDS: &[&str] = &[
    "begin", "end", "if", "then", "else", "for", "while", "do", "go", "to", "procedure", "value",
    "own", "integer", "real", "boolean", "string", "array", "switch", "label",
];
const HOLYC_KEYWORDS: &[&str] = &[
    "if", "else", "for", "while", "do", "switch", "case", "default", "break", "continue", "goto",
    "return", "class", "union", "enum", "typedef", "static", "extern", "const", "volatile",
    "inline", "asm", "import", "try", "catch", "throw", "true", "false", "I8", "I16", "I32",
    "I64", "U8", "U16", "U32", "U64", "F64", "Bool",
];
const PY_BUILTINS: &[&str] = &[
    "abs", "all", "any", "bool", "bytes", "bytearray", "callable", "chr", "dict", "dir", "divmod",
    "enumerate", "eval", "exec", "filter", "float", "format", "frozenset", "getattr", "hasattr",
    "hash", "help", "hex", "id", "int", "isinstance", "issubclass", "iter", "len", "list", "map",
    "max", "min", "next", "object", "oct", "open", "ord", "pow", "print", "range", "repr", "reversed",
    "round", "set", "slice", "sorted", "str", "sum", "tuple", "type", "zip", "super", "property",
    "classmethod", "staticmethod", "globals", "locals", "vars",
];
const GO_BUILTINS: &[&str] = &[
    "append", "cap", "close", "complex", "copy", "delete", "imag", "len", "make", "new", "panic",
    "print", "println", "real", "recover", "error", "iota", "nil", "true", "false",
];
const ZIG_BUILTINS: &[&str] = &[
    "@import", "@cImport", "@sizeOf", "@TypeOf", "@intCast", "@ptrCast", "@as", "@compileError",
    "@panic", "@memcpy", "@memset",
];
const JAVA_BUILTINS: &[&str] = &[
    "System", "String", "Integer", "Long", "Double", "Boolean", "List", "Map", "Set", "Math",
];
const HASKELL_BUILTINS: &[&str] = &[
    "map", "filter", "foldl", "foldr", "head", "tail", "length", "null", "not", "fst", "snd",
    "putStrLn", "print", "show", "read", "Just", "Nothing",
];
const LUA_BUILTINS: &[&str] = &[
    "print", "pairs", "ipairs", "pcall", "xpcall", "tonumber", "tostring", "type", "assert",
    "require", "table", "string", "math", "coroutine", "io", "os",
];
const JS_BUILTINS: &[&str] = &[
    "console", "window", "document", "setTimeout", "setInterval", "Promise", "Array", "Object",
    "String", "Number", "Boolean", "Map", "Set", "Date", "JSON",
];
const TS_BUILTINS: &[&str] = &[
    "Promise", "Readonly", "Partial", "Pick", "Record", "Exclude", "Extract", "Omit", "unknown",
    "never", "any",
];
const COFFEE_BUILTINS: &[&str] = &["console", "require", "module", "exports", "__dirname", "__filename"];
const PHP_BUILTINS: &[&str] = &[
    "echo", "print_r", "var_dump", "isset", "empty", "count", "array_merge", "strlen", "strpos",
    "explode", "implode", "json_encode", "json_decode",
];
const PASCAL_BUILTINS: &[&str] = &["writeln", "readln", "length", "setlength", "high", "low", "ord", "chr"];
const ALGOL_BUILTINS: &[&str] = &["print", "read", "sqrt", "sin", "cos", "abs"];
const RUBY_BUILTINS: &[&str] = &[
    "puts", "print", "p", "require", "include", "extend", "attr_reader", "attr_writer",
    "attr_accessor", "describe", "context", "it", "expect", "let", "before", "after",
];
const PERL_BUILTINS: &[&str] = &[
    "say", "print", "warn", "die", "open", "close", "split", "join", "push", "pop", "shift",
    "unshift", "grep", "map", "keys", "values", "exists", "defined", "chomp", "substr", "length",
];
struct LangSnippet {
    #[allow(dead_code)]
    lang: Language,
    trigger: &'static str,
    label: &'static str,
    body: &'static str,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Language {
    Steelconf,
    C,
    Cpp,
    Python,
    Kotlin,
    Swift,
    Dart,
    Elixir,
    Erlang,
    Clojure,
    FSharp,
    RLang,
    Julia,
    MatlabOctave,
    Scala,
    Groovy,
    Nim,
    Crystal,
    Fortran,
    Cobol,
    Ada,
    Assembly,
    VLang,
    Solidity,
    Move,
    Vhdl,
    Verilog,
    Prolog,
    Scheme,
    Smalltalk,
    Tcl,
    PowerShell,
    Fish,
    Makefile,
    Wgsl,
    OpenClC,
    Hack,
    Apex,
    JavaScript,
    TypeScript,
    Go,
    Rust,
    Php,
    Lua,
    Perl,
    Ruby,
    Haskell,
    Shell,
    CoffeeScript,
    Pascal,
    Algol,
    HolyC,
    Java,
    Ocaml,
    Zig,
    CSharp,
    Other,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[allow(dead_code)]
enum ShellDialect {
    Posix,
    Bash,
    Zsh,
    Union,
}

#[derive(Clone)]
enum MultiLineState {
    String(String),
    Comment(String),
    HaskellBlock(usize),
    PascalBlock(usize),
}

#[derive(Copy, Clone)]
enum Theme {
    Dark,
    Light,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Palette {
    Default,
    Vivid,
    Soft,
}

struct ThemeColors {
    fg: Color,
    comment: Color,
    doc_comment: Color,
    keyword: Color,
    type_name: Color,
    builtin: Color,
    function: Color,
    string: Color,
    number: Color,
    operator: Color,
    todo: Color,
    directive: Color,
    header_ok: Color,
    header_bad: Color,
    status_ok: Color,
    status_warn: Color,
    tab_inactive: Color,
    lint: Color,
    minimap: Color,
    minimap_changed: Color,
    selection: Color,
}

impl Theme {
    fn colors(self) -> ThemeColors {
        match self {
            Theme::Dark => ThemeColors {
                fg: Color::White,
                comment: Color::DarkGrey,
                doc_comment: Color::DarkCyan,
                keyword: Color::Cyan,
                type_name: Color::Blue,
                builtin: Color::Green,
                function: Color::Yellow,
                string: Color::Green,
                number: Color::Yellow,
                operator: Color::Magenta,
                todo: Color::Yellow,
                directive: Color::Yellow,
                header_ok: Color::Green,
                header_bad: Color::Red,
                status_ok: Color::DarkGrey,
                status_warn: Color::Red,
                tab_inactive: Color::DarkGrey,
                lint: Color::Red,
                minimap: Color::DarkGrey,
                minimap_changed: Color::Yellow,
                selection: Color::Cyan,
            },
            Theme::Light => ThemeColors {
                fg: Color::Black,
                comment: Color::DarkGrey,
                doc_comment: Color::DarkBlue,
                keyword: Color::Blue,
                type_name: Color::DarkMagenta,
                builtin: Color::DarkGreen,
                function: Color::DarkCyan,
                string: Color::DarkGreen,
                number: Color::DarkYellow,
                operator: Color::DarkMagenta,
                todo: Color::DarkYellow,
                directive: Color::DarkCyan,
                header_ok: Color::DarkGreen,
                header_bad: Color::Red,
                status_ok: Color::DarkGrey,
                status_warn: Color::Red,
                tab_inactive: Color::DarkGrey,
                lint: Color::Red,
                minimap: Color::DarkGrey,
                minimap_changed: Color::DarkYellow,
                selection: Color::Blue,
            },
        }
    }
}

impl Palette {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "default" => Some(Self::Default),
            "vivid" => Some(Self::Vivid),
            "soft" => Some(Self::Soft),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Vivid => "vivid",
            Self::Soft => "soft",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Default => Self::Vivid,
            Self::Vivid => Self::Soft,
            Self::Soft => Self::Default,
        }
    }
}

struct EditorConfig {
    autosave_interval: Option<u64>,
    theme: Option<Theme>,
    palette_c: Option<Palette>,
    palette_cpp: Option<Palette>,
    palette_py: Option<Palette>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum LineEnding {
    Lf,
    CrLf,
}

struct RunError {
    path: PathBuf,
    line: usize,
    message: String,
}

struct Snippet {
    trigger: &'static str,
    label: &'static str,
    body: &'static str,
}

#[derive(Clone)]
struct CompletionItem {
    label: String,
    insert: String,
    is_snippet: bool,
}

fn main() -> io::Result<()> {
    let file = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("steelconf"));
    let mut editor = Editor::open(file)?;
    editor.run()
}

struct Editor {
    file: PathBuf,
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    scroll: usize,
    dirty: bool,
    status: String,
    confirm_quit: bool,
    clipboard: String,
    show_help: bool,
    validation: Option<String>,
    language: Language,
    last_search: String,
    history: Vec<PathBuf>,
    read_only: bool,
    autosave: bool,
    last_autosave: Instant,
    tabs: Vec<PathBuf>,
    current_tab: usize,
    show_history: bool,
    lint: Vec<String>,
    undo: Vec<Vec<String>>,
    redo: Vec<Vec<String>>,
    original_lines: Vec<String>,
    diff_mode: bool,
    autosave_interval: Duration,
    theme: Theme,
    colors: ThemeColors,
    palette_c: Palette,
    palette_cpp: Palette,
    palette_py: Palette,
    extra_cursors: Vec<(usize, usize)>,
    line_ending: LineEnding,
    encoding: String,
    show_run_panel: bool,
    run_output: Vec<String>,
    run_status: Option<i32>,
    show_terminal_panel: bool,
    terminal_output: Vec<String>,
    terminal_status: Option<i32>,
    last_terminal_cmd: String,
    session_paths: Vec<PathBuf>,
    pending_restore: bool,
    run_errors: Vec<RunError>,
    safe_mode: bool,
    soft_wrap: bool,
    show_glob_panel: bool,
    glob_preview: Vec<String>,
    last_glob_refresh: Instant,
    completion_active: bool,
    completion_items: Vec<CompletionItem>,
    completion_selected: usize,
    completion_start: usize,
    completion_prefix: String,
}

impl Editor {
    fn open(file: PathBuf) -> io::Result<Self> {
        let bytes = fs::read(&file).unwrap_or_default();
        let line_ending = detect_line_ending(&bytes);
        let encoding = detect_encoding(&bytes);
        let content = String::from_utf8_lossy(&bytes);
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let config = load_editor_config();
        let theme = config.theme.unwrap_or(Theme::Dark);
        let colors = theme.colors();
        let autosave_interval = Duration::from_secs(config.autosave_interval.unwrap_or(30).max(1));
        let session_paths = load_session_paths();
        let language = detect_language(&file);
        let palette_c = config.palette_c.unwrap_or(Palette::Default);
        let palette_cpp = config.palette_cpp.unwrap_or(Palette::Default);
        let palette_py = config.palette_py.unwrap_or(Palette::Default);
        Ok(Self {
            file: file.clone(),
            lines: lines.clone(),
            cursor_x: 0,
            cursor_y: 0,
            scroll: 0,
            dirty: false,
            status: "Ctrl+S save | Ctrl+Q quit".to_string(),
            confirm_quit: false,
            clipboard: String::new(),
            show_help: false,
            validation: None,
            language,
            last_search: String::new(),
            history: vec![file.clone()],
            read_only: false,
            autosave: config.autosave_interval.unwrap_or(0) > 0,
            last_autosave: Instant::now(),
            tabs: vec![file.clone()],
            current_tab: 0,
            show_history: false,
            lint: Vec::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            original_lines: lines,
            diff_mode: false,
            autosave_interval,
            theme,
            colors,
            palette_c,
            palette_cpp,
            palette_py,
            extra_cursors: Vec::new(),
            line_ending,
            encoding,
            show_run_panel: false,
            run_output: Vec::new(),
            run_status: None,
            show_terminal_panel: false,
            terminal_output: Vec::new(),
            terminal_status: None,
            last_terminal_cmd: String::new(),
            session_paths,
            pending_restore: true,
            run_errors: Vec::new(),
            safe_mode: false,
            soft_wrap: false,
            show_glob_panel: false,
            glob_preview: Vec::new(),
            last_glob_refresh: Instant::now(),
            completion_active: false,
            completion_items: Vec::new(),
            completion_selected: 0,
            completion_start: 0,
            completion_prefix: String::new(),
        })
    }

    fn run(&mut self) -> io::Result<()> {
        let mut out = stdout();
        terminal::enable_raw_mode()?;
        execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;

        if self.pending_restore {
            self.maybe_restore_session()?;
        }

        let result = loop {
            self.render(&mut out)?;
            if let Some(ev) = self.read_event()? {
                if self.handle_event(ev)? {
                    break Ok(());
                }
            }
            if self.autosave && self.dirty && self.last_autosave.elapsed() >= self.autosave_interval {
                let _ = self.save();
                self.last_autosave = Instant::now();
            }
        };

        self.save_session();
        execute!(out, terminal::LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;
        result
    }

    fn read_event(&self) -> io::Result<Option<Event>> {
        if event::poll(Duration::from_millis(50))? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }

    fn handle_event(&mut self, ev: Event) -> io::Result<bool> {
        match ev {
            Event::Key(key) => self.handle_key(key),
            _ => Ok(false),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> io::Result<bool> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.modifiers.contains(KeyModifiers::SHIFT) {
            if let KeyCode::Char('d') = key.code {
                self.diff_mode = !self.diff_mode;
                self.status = if self.diff_mode { "Diff on".to_string() } else { "Diff off".to_string() };
                return Ok(false);
            }
            if let KeyCode::Char('n') = key.code {
                self.goto_prev_match_word();
                return Ok(false);
            }
            if let KeyCode::Char('s') = key.code {
                self.safe_mode = !self.safe_mode;
                self.status = if self.safe_mode {
                    "Safe mode on".to_string()
                } else {
                    "Safe mode off".to_string()
                };
                return Ok(false);
            }
            if let KeyCode::Char('e') = key.code {
                self.jump_run_error()?;
                return Ok(false);
            }
            if let KeyCode::Char('g') = key.code {
                self.show_glob_panel = !self.show_glob_panel;
                if self.show_glob_panel {
                    self.refresh_glob_preview();
                    self.status = "Glob preview".to_string();
                } else {
                    self.status = "Glob preview off".to_string();
                }
                return Ok(false);
            }
            if let KeyCode::Char('i') = key.code {
                self.insert_snippet()?;
                return Ok(false);
            }
            if let KeyCode::Char('l') = key.code {
                let verbose = key.modifiers.contains(KeyModifiers::ALT);
                self.show_completion_debug(verbose);
                return Ok(false);
            }
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('g') => {
                    self.show_help = !self.show_help;
                    return Ok(false);
                }
                KeyCode::Char('o') => {
                    self.open_prompt()?;
                    return Ok(false);
                }
                KeyCode::Char('s') => {
                    self.save()?;
                    return Ok(false);
                }
                KeyCode::Char('f') => {
                    self.search_prompt()?;
                    return Ok(false);
                }
                KeyCode::Char('w') => {
                    self.soft_wrap = !self.soft_wrap;
                    self.status = if self.soft_wrap {
                        "Soft wrap on".to_string()
                    } else {
                        "Soft wrap off".to_string()
                    };
                    return Ok(false);
                }
                KeyCode::Char('l') => {
                    self.goto_line_prompt()?;
                    return Ok(false);
                }
                KeyCode::Char('r') => {
                    self.run_steel()?;
                    return Ok(false);
                }
                KeyCode::Char('p') => {
                    self.find_file_prompt()?;
                    return Ok(false);
                }
                KeyCode::Char('/') => {
                    self.toggle_comment();
                    return Ok(false);
                }
                KeyCode::Char('h') => {
                    self.replace_prompt()?;
                    return Ok(false);
                }
                KeyCode::Char('d') => {
                    self.add_next_match();
                    return Ok(false);
                }
                KeyCode::Char('t') => {
                    self.format_buffer();
                    return Ok(false);
                }
                KeyCode::Char('b') => {
                    self.jump_bake_block();
                    return Ok(false);
                }
                KeyCode::Char('j') => {
                    self.jump_symbol_prompt()?;
                    return Ok(false);
                }
                KeyCode::Char('z') => {
                    self.undo();
                    return Ok(false);
                }
                KeyCode::Char('y') => {
                    self.redo();
                    return Ok(false);
                }
                KeyCode::Tab => {
                    self.next_tab();
                    return Ok(false);
                }
                KeyCode::Char('a') => {
                    self.autosave = !self.autosave;
                    self.status = if self.autosave {
                        format!("Autosave on ({}s)", self.autosave_interval.as_secs())
                    } else {
                        "Autosave off".to_string()
                    };
                    return Ok(false);
                }
                KeyCode::Char('e') => {
                    self.read_only = !self.read_only;
                    self.status = if self.read_only {
                        "Read-only on".to_string()
                    } else {
                        "Read-only off".to_string()
                    };
                    return Ok(false);
                }
                KeyCode::Char('m') => {
                    self.toggle_theme();
                    return Ok(false);
                }
                KeyCode::Char(',') => {
                    self.open_settings_menu()?;
                    return Ok(false);
                }
                KeyCode::Char('`') => {
                    self.open_native_terminal()?;
                    return Ok(false);
                }
                KeyCode::Char('n') => {
                    self.goto_next_match_word();
                    return Ok(false);
                }
                KeyCode::Char('k') => {
                    self.cut_line();
                    return Ok(false);
                }
                KeyCode::Char('c') => {
                    self.copy_current_line_system();
                    return Ok(false);
                }
                KeyCode::Char('v') => {
                    self.paste_from_system();
                    return Ok(false);
                }
                KeyCode::Char('u') => {
                    self.paste_line();
                    return Ok(false);
                }
                KeyCode::Char('q') => {
                    if self.dirty && !self.confirm_quit {
                        self.status = "Unsaved changes. Press Ctrl+Q again to quit.".to_string();
                        self.confirm_quit = true;
                        return Ok(false);
                    }
                    return Ok(true);
                }
                _ => {}
            }
        }

        self.confirm_quit = false;
        match key.code {
            KeyCode::Esc => {
                if self.completion_active {
                    self.clear_completion();
                    return Ok(false);
                }
                if self.show_terminal_panel {
                    self.show_terminal_panel = false;
                    return Ok(false);
                }
                if self.show_run_panel {
                    self.show_run_panel = false;
                    return Ok(false);
                }
                if self.show_glob_panel {
                    self.show_glob_panel = false;
                    return Ok(false);
                }
            }
            KeyCode::Up => {
                if self.completion_active {
                    self.completion_move(-1);
                } else if key.modifiers.contains(KeyModifiers::ALT) && key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.add_block_cursor(-1);
                } else {
                    self.move_up();
                }
            }
            KeyCode::Down => {
                if self.completion_active {
                    self.completion_move(1);
                } else if key.modifiers.contains(KeyModifiers::ALT) && key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.add_block_cursor(1);
                } else {
                    self.move_down();
                }
            }
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Backspace => {
                if !self.read_only {
                    self.backspace();
                } else {
                    self.status = "Read-only: edit disabled.".to_string();
                }
            }
            KeyCode::F(3) => {
                let query = self.last_search.clone();
                if !query.is_empty() {
                    if !self.find_next(&query) {
                        self.status = format!("Not found: {}", self.last_search);
                    }
                }
            }
            KeyCode::F(2) => {
                self.open_recent_prompt()?;
            }
            KeyCode::Enter => self.insert_newline(),
            KeyCode::Tab => {
                if self.completion_active {
                    self.apply_completion();
                } else {
                    if !self.read_only {
                        if !self.complete_token()? {
                            self.insert_tab();
                        }
                    } else {
                        self.status = "Read-only: edit disabled.".to_string();
                    }
                }
            }
            KeyCode::Char('%') => {
                self.jump_match();
            }
            KeyCode::Char(ch) => self.insert_char(ch),
            _ => {}
        }
        Ok(false)
    }

    fn render(&mut self, out: &mut io::Stdout) -> io::Result<()> {
        let (cols, rows) = terminal::size()?;
        let panel_height = if self.show_terminal_panel {
            self.terminal_panel_height(rows)
        } else {
            0
        };
        let height = rows.saturating_sub(3 + panel_height) as usize;

        self.validation = self.validate_header();
        self.lint = self.lint_warnings();

        if self.soft_wrap {
            let text_cols = cols.saturating_sub(2) as usize;
            let cursor_row = self.visual_row_from(self.scroll, text_cols);
            if cursor_row == 0 && self.cursor_y < self.scroll {
                self.scroll = self.cursor_y;
            } else if cursor_row >= height {
                self.scroll = self.scroll.saturating_add(1);
            }
        } else {
            if self.cursor_y < self.scroll {
                self.scroll = self.cursor_y;
            }
            if self.cursor_y >= self.scroll + height {
                self.scroll = self.cursor_y.saturating_sub(height.saturating_sub(1));
            }
        }

        queue!(out, cursor::Show, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
        self.render_tabs(out, cols as usize)?;

        if self.show_help {
            self.render_help(out, cols as usize, rows as usize)?;
            out.flush()?;
            return Ok(());
        }
        if self.show_history {
            self.render_history(out, cols as usize, rows as usize)?;
            out.flush()?;
            return Ok(());
        }
        if self.show_run_panel {
            self.render_run_panel(out, cols as usize, rows as usize)?;
            out.flush()?;
            return Ok(());
        }
        if self.show_glob_panel {
            self.refresh_glob_preview();
            self.render_glob_panel(out, cols as usize, rows as usize)?;
            out.flush()?;
            return Ok(());
        }

        let text_cols = cols.saturating_sub(2) as usize;
        let mut raw_state: Option<MultiLineState> = None;
        let mut raw_carry = String::new();
        let mut py_state: Option<String> = None;
        let mut py_carry = String::new();
        let mut screen_row = 1usize;
        let mut line_idx = self.scroll;
        while screen_row <= height && line_idx < self.lines.len() {
            let line = &self.lines[line_idx];
            if self.soft_wrap && text_cols > 0 && line.len() > text_cols {
                let mut start = 0usize;
                while start < line.len() && screen_row <= height {
                    let end = (start + text_cols).min(line.len());
                    let chunk = &line[start..end];
                    let y = screen_row as u16;
                    queue!(out, cursor::MoveTo(0, y), Clear(ClearType::CurrentLine))?;
                    self.render_line(
                        out,
                        chunk,
                        text_cols,
                        &mut raw_state,
                        &mut raw_carry,
                        &mut py_state,
                        &mut py_carry,
                    )?;
                    screen_row += 1;
                    start += text_cols;
                }
            } else {
                let y = screen_row as u16;
                queue!(out, cursor::MoveTo(0, y), Clear(ClearType::CurrentLine))?;
                self.render_line(
                    out,
                    line,
                    text_cols,
                    &mut raw_state,
                    &mut raw_carry,
                    &mut py_state,
                    &mut py_carry,
                )?;
                if !self.soft_wrap {
                    self.render_minimap(out, y, cols)?;
                }
                screen_row += 1;
            }
            line_idx += 1;
        }

        if !self.completion_active && !self.show_terminal_panel {
            if let Some(msg) = self.lint.first() {
                let warn_line = truncate(&format!("warn: {msg}"), cols as usize);
                queue!(
                    out,
                    cursor::MoveTo(0, rows.saturating_sub(2)),
                    SetForegroundColor(self.colors.lint),
                    Print(warn_line),
                    ResetColor
                )?;
            }
        }

        if self.show_terminal_panel && panel_height > 0 {
            self.render_terminal_panel(out, cols as usize, rows as usize, panel_height)?;
        }

        let mut status = format!(
            "{} | {}{} | {}:{}",
            self.file.display(),
            if self.dirty { "*" } else { "" },
            self.status,
            self.cursor_y + 1,
            self.cursor_x + 1
        );
        status.push_str(&format!(" | {}/{}", self.current_tab + 1, self.tabs.len()));
        status.push_str(&format!(" | {} {}", self.encoding, line_ending_label(self.line_ending)));
        if !self.extra_cursors.is_empty() {
            status.push_str(&format!(" | MC:{}", self.extra_cursors.len() + 1));
        }
        let now = chrono::Local::now().format("%H:%M").to_string();
        status.push_str(&format!(" | {now}"));
        if self.autosave && self.dirty {
            let remaining = self.autosave_interval.saturating_sub(self.last_autosave.elapsed());
            status.push_str(&format!(" | AS {}s", remaining.as_secs()));
        }
        if self.read_only {
            status.push_str(" | RO");
        }
        if self.autosave {
            status.push_str(" | AS");
        }
        if let Some(shell_status) = shell_dialect_status_segment() {
            status.push_str(&shell_status);
        }
        if let Some(msg) = &self.validation {
            status.push_str(" | ");
            status.push_str(msg);
        }
        let status_line = truncate(&status, cols as usize);
        let status_color = if self.validation.is_some() {
            self.colors.status_warn
        } else {
            self.colors.status_ok
        };
        queue!(
            out,
            cursor::MoveTo(0, rows - 1),
            SetForegroundColor(status_color),
            Print(status_line),
            ResetColor
        )?;

        if self.completion_active {
            self.render_completion_popup(out, cols as usize, rows as usize)?;
        }

        let (cursor_x, cursor_y) = if self.soft_wrap && text_cols > 0 {
            let row = self.visual_row_from(self.scroll, text_cols);
            let col = self.cursor_x.min(self.current_line_len()) % text_cols;
            (col as u16, (row + 1) as u16)
        } else {
            let cursor_x = self.cursor_x.min(self.current_line_len()) as u16;
            let cursor_y = (self.cursor_y.saturating_sub(self.scroll)) as u16 + 1;
            (cursor_x, cursor_y)
        };
        queue!(out, cursor::MoveTo(cursor_x, cursor_y))?;

        out.flush()?;
        Ok(())
    }

    fn render_help(&self, out: &mut io::Stdout, cols: usize, rows: usize) -> io::Result<()> {
        let help = [
            "steecleditor (steelconf)",
            "",
            "Ctrl+O  Open",
            "Ctrl+X  Quit",
            "Ctrl+F  Search",
            "F3      Find next",
            "Ctrl+L  Go to line",
            "Ctrl+P  Find file",
            "Ctrl+/  Toggle comment",
            "Ctrl+H  Search/replace",
            "Ctrl+Shift+D  Diff toggle",
            "Ctrl+T  Format",
            "Ctrl+B  Jump bake block",
            "Ctrl+J  Jump to symbol",
            "Ctrl+K  Cut line",
            "Ctrl+C  Copy line",
            "Ctrl+V  Paste",
            "Ctrl+U  Paste line",
            "Ctrl+`  Open terminal",
            "Ctrl+G  Toggle help",
            "F2      Recent files",
            "Ctrl+A  Autosave toggle",
            "Ctrl+E  Read-only toggle",
            "Ctrl+M  Toggle theme",
            "Ctrl+,  Settings",
            "Ctrl+R  Run steel",
            "Ctrl+D  Add next match",
            "Ctrl+N  Next word match",
            "Ctrl+Shift+N  Prev word match",
            "Alt+Shift+Up/Down  Block cursors",
            "Ctrl+W  Soft wrap toggle",
            "Ctrl+Shift+S  Safe mode toggle",
            "Ctrl+Shift+E  Jump run error",
            "Ctrl+Shift+G  Glob preview",
            "Ctrl+Shift+I  Insert snippet",
            "Ctrl+Shift+L  Debug language/completion",
            "Ctrl+Shift+Alt+L  Debug --verbose",
            "",
            "Arrows  Move cursor",
            "Tab     Indent (2 spaces)",
            "Ctrl+S  Save",
        ];

        let start_y = rows.saturating_sub(help.len()) / 2;
        for (i, line) in help.iter().enumerate() {
            let y = (start_y + i) as u16;
            let text = truncate(line, cols);
            let x = cols.saturating_sub(text.len()) / 2;
            queue!(
                out,
                cursor::MoveTo(x as u16, y),
                SetForegroundColor(self.colors.keyword),
                Print(text),
                ResetColor
            )?;
        }
        Ok(())
    }

    fn render_line(
        &self,
        out: &mut io::Stdout,
        line: &str,
        cols: usize,
        raw_state: &mut Option<MultiLineState>,
        raw_carry: &mut String,
        py_state: &mut Option<String>,
        py_carry: &mut String,
    ) -> io::Result<()> {
        let trimmed = line.trim_start();
        if contains_todo(trimmed) {
            queue!(
                out,
                SetForegroundColor(self.colors.todo),
                Print(truncate(line, cols)),
                ResetColor
            )?;
            return Ok(());
        }
        if self.language != Language::Steelconf {
            if let Some(state) = raw_state.clone() {
                match state {
                    MultiLineState::String(term) => {
                        if let Some(marker) = term.strip_prefix("LINE:") {
                            queue!(
                                out,
                                SetForegroundColor(self.colors.string),
                                Print(truncate(line, cols)),
                                ResetColor
                            )?;
                            if (marker == "\"@" && is_powershell_here_string_terminator(line, '"'))
                                || (marker == "'@" && is_powershell_here_string_terminator(line, '\''))
                                || is_line_terminator_marker(line, marker)
                            {
                                *raw_state = None;
                                raw_carry.clear();
                            }
                            return Ok(());
                        }
                        let mut search = String::new();
                        if !raw_carry.is_empty() {
                            search.push_str(raw_carry);
                        }
                        search.push_str(line);
                        if let Some(pos) = search.find(&term) {
                            if pos < raw_carry.len() {
                                *raw_state = None;
                                raw_carry.clear();
                                return self
                                    .render_inline_with_keywords(out, line, cols, raw_state, py_state, py_carry);
                            }
                            let end = pos - raw_carry.len() + term.len();
                            let end = end.min(line.len());
                            let head = &line[..end];
                            queue!(
                                out,
                                SetForegroundColor(self.colors.string),
                                Print(truncate(head, cols)),
                                ResetColor
                            )?;
                            *raw_state = None;
                            raw_carry.clear();
                            let rest = &line[end..];
                            let remaining_cols = cols.saturating_sub(head.chars().count());
                            return self
                                .render_inline_with_keywords(out, rest, remaining_cols, raw_state, py_state, py_carry);
                        }
                        queue!(
                            out,
                            SetForegroundColor(self.colors.string),
                            Print(truncate(line, cols)),
                            ResetColor
                        )?;
                        let tail_len = term.len().saturating_sub(1);
                        if tail_len == 0 {
                            raw_carry.clear();
                        } else {
                            let combined = search;
                            let start = combined.len().saturating_sub(tail_len);
                            raw_carry.clear();
                            raw_carry.push_str(&combined[start..]);
                        }
                        return Ok(());
                    }
                    MultiLineState::Comment(term) => {
                        let mut search = String::new();
                        if !raw_carry.is_empty() {
                            search.push_str(raw_carry);
                        }
                        search.push_str(line);
                        if let Some(pos) = search.find(&term) {
                            if pos < raw_carry.len() {
                                *raw_state = None;
                                raw_carry.clear();
                                return self
                                    .render_inline_with_keywords(out, line, cols, raw_state, py_state, py_carry);
                            }
                            let end = pos - raw_carry.len() + term.len();
                            let end = end.min(line.len());
                            let head = &line[..end];
                            queue!(
                                out,
                                SetForegroundColor(self.colors.comment),
                                Print(truncate(head, cols)),
                                ResetColor
                            )?;
                            *raw_state = None;
                            raw_carry.clear();
                            let rest = &line[end..];
                            let remaining_cols = cols.saturating_sub(head.chars().count());
                            return self
                                .render_inline_with_keywords(out, rest, remaining_cols, raw_state, py_state, py_carry);
                        }
                        queue!(
                            out,
                            SetForegroundColor(self.colors.comment),
                            Print(truncate(line, cols)),
                            ResetColor
                        )?;
                        let tail_len = term.len().saturating_sub(1);
                        if tail_len == 0 {
                            raw_carry.clear();
                        } else {
                            let combined = search;
                            let start = combined.len().saturating_sub(tail_len);
                            raw_carry.clear();
                            raw_carry.push_str(&combined[start..]);
                        }
                        return Ok(());
                    }
                    MultiLineState::HaskellBlock(mut depth) => {
                        let mut idx = 0usize;
                        let mut end = line.len();
                        while idx + 1 < line.len() {
                            let two = &line[idx..idx + 2];
                            if two == "{-" {
                                depth += 1;
                                idx += 2;
                            } else if two == "-}" {
                                depth = depth.saturating_sub(1);
                                idx += 2;
                                if depth == 0 {
                                    end = idx;
                                    break;
                                }
                            } else {
                                idx += 1;
                            }
                        }
                        let head = &line[..end.min(line.len())];
                        queue!(
                            out,
                            SetForegroundColor(self.colors.comment),
                            Print(truncate(head, cols)),
                            ResetColor
                        )?;
                        if depth == 0 {
                            *raw_state = None;
                            raw_carry.clear();
                            let rest = &line[end.min(line.len())..];
                            if !rest.is_empty() {
                                let remaining_cols = cols.saturating_sub(head.chars().count());
                                return self
                                    .render_inline_with_keywords(out, rest, remaining_cols, raw_state, py_state, py_carry);
                            }
                        } else {
                            *raw_state = Some(MultiLineState::HaskellBlock(depth));
                        }
                    }
                    MultiLineState::PascalBlock(mut depth) => {
                        let mut idx = 0usize;
                        let mut end = line.len();
                        while idx + 1 < line.len() {
                            let two = &line[idx..idx + 2];
                            if two == "(*" {
                                depth += 1;
                                idx += 2;
                            } else if two == "*)" {
                                depth = depth.saturating_sub(1);
                                idx += 2;
                                if depth == 0 {
                                    end = idx;
                                    break;
                                }
                            } else {
                                idx += 1;
                            }
                        }
                        let head = &line[..end.min(line.len())];
                        queue!(
                            out,
                            SetForegroundColor(self.colors.comment),
                            Print(truncate(head, cols)),
                            ResetColor
                        )?;
                        if depth == 0 {
                            *raw_state = None;
                            raw_carry.clear();
                            let rest = &line[end.min(line.len())..];
                            if !rest.is_empty() {
                                let remaining_cols = cols.saturating_sub(head.chars().count());
                                return self
                                    .render_inline_with_keywords(out, rest, remaining_cols, raw_state, py_state, py_carry);
                            }
                        } else {
                            *raw_state = Some(MultiLineState::PascalBlock(depth));
                        }
                    }
                }
            }
            raw_carry.clear();

            if self.language == Language::Python {
                if let Some(term) = py_state.clone() {
                    let mut search = String::new();
                    if !py_carry.is_empty() {
                        search.push_str(py_carry);
                    }
                    search.push_str(line);
                    if let Some(pos) = search.find(&term) {
                        if pos < py_carry.len() {
                            *py_state = None;
                            py_carry.clear();
                            return self.render_inline_with_keywords(out, line, cols, raw_state, py_state, py_carry);
                        }
                        let end = pos - py_carry.len() + term.len();
                        let end = end.min(line.len());
                        let head = &line[..end];
                        queue!(
                            out,
                            SetForegroundColor(self.colors.string),
                            Print(truncate(head, cols)),
                            ResetColor
                        )?;
                        *py_state = None;
                        py_carry.clear();
                        let rest = &line[end..];
                        let remaining_cols = cols.saturating_sub(head.chars().count());
                        return self.render_inline_with_keywords(out, rest, remaining_cols, raw_state, py_state, py_carry);
                    }
                    queue!(
                        out,
                        SetForegroundColor(self.colors.string),
                        Print(truncate(line, cols)),
                        ResetColor
                    )?;
                    let tail_len = term.len().saturating_sub(1);
                    if tail_len == 0 {
                        py_carry.clear();
                    } else {
                        let combined = search;
                        let start = combined.len().saturating_sub(tail_len);
                        py_carry.clear();
                        py_carry.push_str(&combined[start..]);
                    }
                    return Ok(());
                }
                py_carry.clear();
            }

            if self.language == Language::Makefile && is_makefile_recipe_line(line) {
                queue!(
                    out,
                    SetForegroundColor(self.colors.directive),
                    Print(truncate(line, cols)),
                    ResetColor
                )?;
                return Ok(());
            }

            if self.language == Language::Makefile && line.starts_with('\t') {
                queue!(
                    out,
                    SetForegroundColor(self.colors.directive),
                    Print(truncate(line, cols)),
                    ResetColor
                )?;
                return Ok(());
            }

            return self.render_inline_with_keywords(out, line, cols, raw_state, py_state, py_carry);
        }
        if trimmed.starts_with(";;") {
            queue!(
                out,
                SetForegroundColor(self.colors.comment),
                Print(truncate(line, cols)),
                ResetColor
            )?;
            return Ok(());
        }

        if is_block_keyword(line) {
            queue!(
                out,
                SetForegroundColor(self.colors.keyword),
                Print(truncate(line, cols)),
                ResetColor
            )?;
            return Ok(());
        }

        if trimmed.starts_with("!muf") {
            let color = if trimmed == "!muf 4" {
                self.colors.header_ok
            } else {
                self.colors.header_bad
            };
            queue!(out, SetForegroundColor(color), Print(truncate(line, cols)), ResetColor)?;
            return Ok(());
        }

        if trimmed.starts_with('.') {
            queue!(
                out,
                SetForegroundColor(self.colors.directive),
                Print(truncate(line, cols)),
                ResetColor
            )?;
            return Ok(());
        }

        self.render_inline(out, line, cols)
    }

    fn current_line_len(&self) -> usize {
        self.lines.get(self.cursor_y).map(|l| l.len()).unwrap_or(0)
    }

    fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.current_line_len();
        }
        self.clear_completion();
    }

    fn move_right(&mut self) {
        let len = self.current_line_len();
        if self.cursor_x < len {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
        self.clear_completion();
    }

    fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.cursor_x.min(self.current_line_len());
        }
        self.clear_completion();
    }

    fn move_down(&mut self) {
        if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.cursor_x = self.cursor_x.min(self.current_line_len());
        }
        self.clear_completion();
    }

    fn insert_char(&mut self, ch: char) {
        if !self.can_edit() {
            return;
        }
        self.record_undo();
        if self.completion_active && ch.is_whitespace() {
            self.clear_completion();
        }
        if !self.extra_cursors.is_empty() {
            self.insert_char_multi(ch);
            return;
        }
        if ch == '[' {
            if let Some(line) = self.lines.get_mut(self.cursor_y) {
                line.insert(self.cursor_x, '[');
                line.insert(self.cursor_x + 1, ']');
                self.cursor_x += 1;
                self.dirty = true;
            }
            return;
        }
        if let Some(line) = self.lines.get_mut(self.cursor_y) {
            line.insert(self.cursor_x, ch);
            self.cursor_x += 1;
            self.dirty = true;
        }
        self.update_auto_completion();
    }

    fn insert_tab(&mut self) {
        for _ in 0..TAB_WIDTH {
            self.insert_char(' ');
        }
    }

    fn insert_newline(&mut self) {
        if !self.can_edit() {
            return;
        }
        if self.completion_active {
            self.apply_completion();
            return;
        }
        if !self.extra_cursors.is_empty() {
            self.extra_cursors.clear();
            self.status = "Multi-cursor cleared for newline.".to_string();
            return;
        }
        self.record_undo();
        let indent = self.current_indent();
        let current_line = self.lines.get(self.cursor_y).map(|s| s.clone()).unwrap_or_default();
        let current = self.lines.get_mut(self.cursor_y).unwrap();
        let tail = current.split_off(self.cursor_x);
        let extra = self.smart_indent_extra(&current_line);
        let new_line = format!("{}{}{}", indent, extra, tail);
        self.lines.insert(self.cursor_y + 1, new_line);
        self.cursor_y += 1;
        self.cursor_x = indent.len() + extra.len();
        self.dirty = true;
        self.clear_completion();
    }

    fn copy_current_line_system(&mut self) {
        let line = self.lines.get(self.cursor_y).cloned().unwrap_or_default();
        self.clipboard = line.clone();
        if system_clipboard_set(&line) {
            self.status = "Copied line to system clipboard.".to_string();
        } else {
            self.status = "Copied line (system clipboard unavailable).".to_string();
        }
    }

    fn paste_from_system(&mut self) {
        if !self.can_edit() {
            return;
        }
        let text = system_clipboard_get().unwrap_or_else(|| self.clipboard.clone());
        if text.is_empty() {
            self.status = "Clipboard empty.".to_string();
            return;
        }
        self.insert_text_at_cursor(&text);
        self.status = "Pasted.".to_string();
    }

    fn insert_text_at_cursor(&mut self, text: &str) {
        self.record_undo();
        let text = text.replace("\r\n", "\n");
        let parts: Vec<&str> = text.split('\n').collect();
        if parts.is_empty() {
            return;
        }
        let current = self.lines.get_mut(self.cursor_y).unwrap();
        let tail = current.split_off(self.cursor_x);
        current.push_str(parts[0]);
        if parts.len() == 1 {
            current.push_str(&tail);
            self.cursor_x += parts[0].len();
            self.dirty = true;
            return;
        }
        for part in &parts[1..] {
            self.cursor_y += 1;
            self.lines.insert(self.cursor_y, part.to_string());
        }
        if let Some(last) = self.lines.get_mut(self.cursor_y) {
            last.push_str(&tail);
        }
        self.cursor_x = parts.last().unwrap_or(&"").len();
        self.dirty = true;
    }

    fn current_indent(&self) -> String {
        let line = self.lines.get(self.cursor_y).map(|s| s.as_str()).unwrap_or("");
        line.chars().take_while(|c| *c == ' ').collect()
    }

    fn backspace(&mut self) {
        if !self.can_edit() {
            return;
        }
        self.record_undo();
        if !self.extra_cursors.is_empty() {
            self.backspace_multi();
            return;
        }
        if self.cursor_x > 0 {
            if let Some(line) = self.lines.get_mut(self.cursor_y) {
                line.remove(self.cursor_x - 1);
                self.cursor_x -= 1;
                self.dirty = true;
            }
        } else if self.cursor_y > 0 {
            let current = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            let prev_len = self.current_line_len();
            if let Some(prev) = self.lines.get_mut(self.cursor_y) {
                prev.push_str(&current);
                self.cursor_x = prev_len;
                self.dirty = true;
            }
        }
        self.update_auto_completion();
    }

    fn save(&mut self) -> io::Result<()> {
        if self.read_only {
            self.status = "Read-only: save disabled.".to_string();
            return Ok(());
        }
        let sep = match self.line_ending {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
        };
        let mut content = self.lines.join(sep);
        content.push_str(sep);
        fs::write(&self.file, content)?;
        self.dirty = false;
        self.original_lines = self.lines.clone();
        self.status = "Saved.".to_string();
        Ok(())
    }

    fn complete_token(&mut self) -> io::Result<bool> {
        let (prefix, start) = self.current_prefix();
        if prefix.is_empty() {
            return Ok(false);
        }

        if self.language == Language::Steelconf {
            if let Some(done) = self.complete_steelconf(&prefix, start)? {
                return Ok(done);
            }
        }

        if self.language != Language::Steelconf && self.language != Language::Other {
            let items = self.collect_language_completion_items(&prefix);
            if !items.is_empty() {
                let choice = if items.len() == 1 {
                    Some(items[0].clone())
                } else {
                    let labels = items.iter().map(|item| item.label.clone()).collect::<Vec<_>>();
                    let picked = self.pick_from_list("Complete", &labels)?;
                    picked.and_then(|label| items.into_iter().find(|item| item.label == label))
                };
                if let Some(item) = choice {
                    self.apply_completion_item(start, &prefix, &item);
                    return Ok(true);
                }
            }
        }

        let candidates = if prefix.starts_with('[') {
            BLOCK_KEYWORDS
        } else if prefix.starts_with('.') {
            DIRECTIVES
        } else {
            return Ok(false);
        };

        let matches = candidates
            .iter()
            .filter(|c| c.starts_with(&prefix))
            .copied()
            .collect::<Vec<_>>();
        if matches.is_empty() {
            return Ok(false);
        }
        let choice = if matches.len() == 1 {
            Some(matches[0].to_string())
        } else {
            let labels = matches.iter().map(|c| c.to_string()).collect::<Vec<_>>();
            self.pick_from_list("Complete", &labels)?
        };
        if let Some(choice) = choice {
            if let Some(line) = self.lines.get_mut(self.cursor_y) {
                line.replace_range(start..self.cursor_x, &choice);
                self.cursor_x = start + choice.len();
                self.dirty = true;
                if self.language == Language::Steelconf {
                    if let Some(snippet) = snippet_for_trigger(&choice) {
                        if line.trim() == choice {
                            self.insert_snippet_lines(snippet);
                        }
                    }
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn current_prefix(&self) -> (String, usize) {
        let line = match self.lines.get(self.cursor_y) {
            Some(l) => l,
            None => return (String::new(), 0),
        };
        let mut start = self.cursor_x;
        while start > 0 {
            let ch = line.chars().nth(start - 1).unwrap_or(' ');
            if ch.is_whitespace() {
                break;
            }
            start -= 1;
        }
        let prefix = line[start..self.cursor_x].to_string();
        (prefix, start)
    }

    fn validate_header(&self) -> Option<String> {
        if self.language != Language::Steelconf {
            return None;
        }
        for line in &self.lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with(";;") {
                continue;
            }
            if trimmed != "!muf 4" {
                return Some("warn: header must be !muf 4".to_string());
            }
            return None;
        }
        Some("warn: missing !muf 4 header".to_string())
    }

    fn render_inline(&self, out: &mut io::Stdout, line: &str, cols: usize) -> io::Result<()> {
        let mut current = String::new();
        let mut count = 0usize;

        let flush = |out: &mut io::Stdout, text: &str, color: Option<Color>| -> io::Result<()> {
            if text.is_empty() {
                return Ok(());
            }
            match color {
                Some(c) => queue!(out, SetForegroundColor(c), Print(text), ResetColor),
                None => queue!(out, SetForegroundColor(self.colors.fg), Print(text), ResetColor),
            }?;
            Ok(())
        };

        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if count >= cols {
                break;
            }
            if ch == '"' {
                flush(out, &current, None)?;
                current.clear();
                let mut string = String::from("\"");
                while let Some(n) = chars.next() {
                    string.push(n);
                    if n == '"' {
                        break;
                    }
                }
                count += string.len();
                flush(out, &string, Some(self.colors.string))?;
                continue;
            }

            if ch == ';' && chars.peek() == Some(&';') {
                flush(out, &current, None)?;
                current.clear();
                let mut comment = String::from(";;");
                chars.next();
                for n in chars {
                    comment.push(n);
                }
                flush(out, &comment, Some(self.colors.comment))?;
                return Ok(());
            }

            current.push(ch);
            count += 1;
        }

        flush(out, &current, None)?;
        Ok(())
    }

    fn cut_line(&mut self) {
        self.record_undo();
        if self.cursor_y >= self.lines.len() {
            return;
        }
        self.clipboard = self.lines.remove(self.cursor_y);
        if self.lines.is_empty() {
            self.lines.push(String::new());
            self.cursor_y = 0;
        } else if self.cursor_y >= self.lines.len() {
            self.cursor_y = self.lines.len() - 1;
        }
        self.cursor_x = self.cursor_x.min(self.current_line_len());
        self.dirty = true;
        self.status = "Line cut.".to_string();
    }

    fn paste_line(&mut self) {
        if self.clipboard.is_empty() {
            self.status = "Clipboard empty.".to_string();
            return;
        }
        self.record_undo();
        let insert_at = self.cursor_y + 1;
        self.lines.insert(insert_at, self.clipboard.clone());
        self.cursor_y = insert_at;
        self.cursor_x = 0;
        self.dirty = true;
        self.status = "Line pasted.".to_string();
    }

    fn search_prompt(&mut self) -> io::Result<()> {
        let query = self.prompt("Search")?;
        if query.is_empty() {
            return Ok(());
        }
        self.last_search = query.clone();
        if self.find_next(&query) {
            self.status = format!("Found: {query}");
        } else {
            self.status = format!("Not found: {query}");
        }
        Ok(())
    }

    fn goto_line_prompt(&mut self) -> io::Result<()> {
        let input = self.prompt("Go to line")?;
        if input.is_empty() {
            return Ok(());
        }
        if let Ok(line_no) = input.parse::<usize>() {
            let target = line_no.saturating_sub(1);
            if target < self.lines.len() {
                self.cursor_y = target;
                self.cursor_x = self.cursor_x.min(self.current_line_len());
                self.status = format!("Line {line_no}");
            } else {
                self.status = "Line out of range.".to_string();
            }
        } else {
            self.status = "Invalid line number.".to_string();
        }
        Ok(())
    }

    fn open_prompt(&mut self) -> io::Result<()> {
        let input = self.prompt("Open file")?;
        if input.is_empty() {
            return Ok(());
        }
        let file = PathBuf::from(input);
        self.open_file(file)?;
        Ok(())
    }

    fn prompt(&mut self, label: &str) -> io::Result<String> {
        let mut input = String::new();
        loop {
            self.status = format!("{label}: {input}");
            let mut out = stdout();
            self.render(&mut out)?;
            if let Some(Event::Key(key)) = self.read_event()? {
                match key.code {
                    KeyCode::Enter => break,
                    KeyCode::Esc => {
                        input.clear();
                        break;
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Char(ch) => {
                        if !key.modifiers.contains(KeyModifiers::CONTROL) {
                            input.push(ch);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(input)
    }

    fn find_next(&mut self, query: &str) -> bool {
        if query.is_empty() {
            return false;
        }
        let mut y = self.cursor_y;
        let mut x = self.cursor_x + 1;
        for _ in 0..self.lines.len() {
            if y >= self.lines.len() {
                y = 0;
                x = 0;
            }
            if let Some(pos) = self.lines[y][x..].find(query) {
                self.cursor_y = y;
                self.cursor_x = x + pos;
                return true;
            }
            y += 1;
            x = 0;
        }
        false
    }

    fn open_file(&mut self, file: PathBuf) -> io::Result<()> {
        let bytes = fs::read(&file).unwrap_or_default();
        let content = String::from_utf8_lossy(&bytes);
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        self.file = file.clone();
        self.lines = lines;
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.scroll = 0;
        self.dirty = false;
        self.validation = None;
        self.language = detect_language(&file);
        self.safe_mode = false;
        self.line_ending = detect_line_ending(&bytes);
        self.encoding = detect_encoding(&bytes);
        self.history.push(file.clone());
        if !self.tabs.iter().any(|p| p == &file) {
            self.tabs.push(file.clone());
            self.current_tab = self.tabs.len() - 1;
        }
        self.undo.clear();
        self.redo.clear();
        self.original_lines = self.lines.clone();
        self.extra_cursors.clear();
        self.clear_completion();
        self.status = "File opened.".to_string();
        Ok(())
    }

    fn lint_warnings(&self) -> Vec<String> {
        if self.language != Language::Steelconf {
            return Vec::new();
        }
        let mut warns = Vec::new();
        let text = self.lines.join("\n");
        if !text.contains("[workspace]") {
            warns.push("missing [workspace] block".to_string());
        }
        if !text.contains("[tool ") && !text.contains("[tool\t") && !text.contains("[tool]") {
            warns.push("missing [tool] block".to_string());
        }
        if !text.contains("[bake ") && !text.contains("[bake\t") && !text.contains("[bake]") {
            warns.push("missing [bake] block".to_string());
        }
        warns
    }

    fn format_buffer(&mut self) {
        if !self.can_edit() {
            return;
        }
        let indent = match self.language {
            Language::Python | Language::Java | Language::C | Language::Cpp | Language::CSharp | Language::Zig => 4,
            Language::Ocaml | Language::Steelconf => 2,
            _ => 2,
        };
        let tab = " ".repeat(indent);
        let mut out = Vec::new();
        for line in &self.lines {
            let replaced = line.replace('\t', &tab);
            out.push(replaced.trim_end().to_string());
        }
        self.lines = out;
        self.dirty = true;
        self.status = "Formatted.".to_string();
    }

    fn jump_bake_block(&mut self) {
        if self.language != Language::Steelconf {
            return;
        }
        let line = match self.lines.get(self.cursor_y) {
            Some(l) => l.trim(),
            None => return,
        };
        if line.starts_with("[bake") {
            for i in self.cursor_y + 1..self.lines.len() {
                if self.lines[i].trim() == ".." {
                    self.cursor_y = i;
                    self.cursor_x = 0;
                    self.status = "Jumped to bake end.".to_string();
                    return;
                }
            }
        } else if line == ".." {
            let mut i = self.cursor_y as i32 - 1;
            while i >= 0 {
                if self.lines[i as usize].trim().starts_with("[bake") {
                    self.cursor_y = i as usize;
                    self.cursor_x = 0;
                    self.status = "Jumped to bake start.".to_string();
                    return;
                }
                i -= 1;
            }
        }
    }

    fn render_minimap(&self, out: &mut io::Stdout, y: u16, cols: u16) -> io::Result<()> {
        if cols < 2 {
            return Ok(());
        }
        let line_idx = self.scroll + y as usize;
        let changed = self.diff_mode && line_idx < self.lines.len()
            && self.lines.get(line_idx) != self.original_lines.get(line_idx);
        let marker = if line_idx == self.cursor_y { '█' } else if changed { '!' } else { '│' };
        let color = if changed {
            self.colors.minimap_changed
        } else {
            self.colors.minimap
        };
        queue!(
            out,
            cursor::MoveTo(cols.saturating_sub(1), y),
            SetForegroundColor(color),
            Print(marker),
            ResetColor
        )?;
        Ok(())
    }

    fn render_tabs(&self, out: &mut io::Stdout, cols: usize) -> io::Result<()> {
        let current_name = self
            .tabs
            .get(self.current_tab)
            .and_then(|tab| tab.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("untitled");
        let mut line = format!(
            "Mitsou Editor 2026 — {current_name} ({})  ",
            self.language_label()
        );
        for (i, tab) in self.tabs.iter().enumerate() {
            let name = tab.file_name().and_then(|s| s.to_str()).unwrap_or("untitled");
            if i == self.current_tab {
                line.push_str(&format!("[{name}] "));
            } else {
                line.push_str(&format!(" {name}  "));
            }
        }
        queue!(
            out,
            cursor::MoveTo(0, 0),
            SetForegroundColor(self.colors.tab_inactive),
            Print(truncate(&line, cols)),
            ResetColor
        )?;
        Ok(())
    }

    fn language_label(&self) -> &'static str {
        language_label_for(self.language)
    }

    fn render_history(&self, out: &mut io::Stdout, cols: usize, rows: usize) -> io::Result<()> {
        let title = "Recent files (F2 to close)";
        queue!(
            out,
            cursor::MoveTo(0, 0),
            SetForegroundColor(self.colors.keyword),
            Print(truncate(title, cols)),
            ResetColor
        )?;
        for (i, path) in self.history.iter().rev().take(rows.saturating_sub(2)).enumerate() {
            let line = format!("{}: {}", i + 1, path.display());
            queue!(
                out,
                cursor::MoveTo(0, (i + 1) as u16),
                Print(truncate(&line, cols))
            )?;
        }
        Ok(())
    }

    fn render_run_panel(&self, out: &mut io::Stdout, cols: usize, rows: usize) -> io::Result<()> {
        let title = match self.run_status {
            Some(code) => format!("steel run (exit {code}) - Esc to close"),
            None => "steel run - Esc to close".to_string(),
        };
        queue!(
            out,
            cursor::MoveTo(0, 0),
            SetForegroundColor(self.colors.keyword),
            Print(truncate(&title, cols)),
            ResetColor
        )?;
        let max_lines = rows.saturating_sub(2);
        for (i, line) in self.run_output.iter().rev().take(max_lines as usize).enumerate() {
            let y = rows.saturating_sub(2).saturating_sub(i) as u16;
            queue!(out, cursor::MoveTo(0, y), Print(truncate(line, cols)))?;
        }
        Ok(())
    }

    fn render_glob_panel(&self, out: &mut io::Stdout, cols: usize, rows: usize) -> io::Result<()> {
        let title = "Glob preview (Esc to close)";
        queue!(
            out,
            cursor::MoveTo(0, 0),
            SetForegroundColor(self.colors.keyword),
            Print(truncate(title, cols)),
            ResetColor
        )?;
        let max_lines = rows.saturating_sub(2);
        for (i, line) in self.glob_preview.iter().take(max_lines as usize).enumerate() {
            let y = (i + 1) as u16;
            queue!(out, cursor::MoveTo(0, y), Print(truncate(line, cols)))?;
        }
        Ok(())
    }

    fn jump_match(&mut self) {
        let line = match self.lines.get(self.cursor_y) {
            Some(l) => l,
            None => return,
        };
        let bytes = line.as_bytes();
        if self.cursor_x >= bytes.len() {
            return;
        }
        let ch = bytes[self.cursor_x] as char;
        let (open, close, dir) = match ch {
            '(' => ('(', ')', 1),
            '[' => ('[', ']', 1),
            '{' => ('{', '}', 1),
            ')' => ('(', ')', -1),
            ']' => ('[', ']', -1),
            '}' => ('{', '}', -1),
            _ => return,
        };

        let mut depth = 0i32;
        if dir > 0 {
            for i in self.cursor_x..bytes.len() {
                let c = bytes[i] as char;
                if c == open {
                    depth += 1;
                } else if c == close {
                    depth -= 1;
                    if depth == 0 {
                        self.cursor_x = i;
                        return;
                    }
                }
            }
        } else {
            let mut i = self.cursor_x as i32;
            while i >= 0 {
                let c = bytes[i as usize] as char;
                if c == close {
                    depth += 1;
                } else if c == open {
                    depth -= 1;
                    if depth == 0 {
                        self.cursor_x = i as usize;
                        return;
                    }
                }
                i -= 1;
            }
        }
    }

    fn find_file_prompt(&mut self) -> io::Result<()> {
        let query = self.prompt("Find file")?;
        if query.is_empty() {
            return Ok(());
        }
        let mut matches = Vec::new();
        for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path().to_string_lossy().to_string();
                if path.contains(&query) {
                    matches.push(path);
                    if matches.len() >= 20 {
                        break;
                    }
                }
            }
        }
        if matches.is_empty() {
            self.status = "No match.".to_string();
            return Ok(());
        }
        let choice = self.pick_from_list("Open file", &matches)?;
        if let Some(path) = choice {
            self.open_file(PathBuf::from(path))?;
        } else {
            self.status = "Canceled.".to_string();
        }
        Ok(())
    }

    fn open_recent_prompt(&mut self) -> io::Result<()> {
        if self.history.is_empty() {
            self.status = "No recent files.".to_string();
            return Ok(());
        }
        let recent = self
            .history
            .iter()
            .rev()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        let choice = self.pick_from_list("Open recent", &recent)?;
        if let Some(path) = choice {
            self.open_file(PathBuf::from(path))?;
        }
        Ok(())
    }

    fn jump_symbol_prompt(&mut self) -> io::Result<()> {
        let symbols = self.collect_symbols();
        if symbols.is_empty() {
            self.status = "No symbols.".to_string();
            return Ok(());
        }
        let list = symbols
            .iter()
            .map(|(line, name)| format!("{}: {}", line + 1, name))
            .collect::<Vec<_>>();
        let choice = self.pick_from_list("Jump to", &list)?;
        if let Some(item) = choice {
            if let Some((line_str, _)) = item.split_once(':') {
                if let Ok(line_no) = line_str.trim().parse::<usize>() {
                    let target = line_no.saturating_sub(1);
                    if target < self.lines.len() {
                        self.cursor_y = target;
                        self.cursor_x = 0;
                        self.status = format!("Jumped to line {line_no}");
                    }
                }
            }
        }
        Ok(())
    }

    fn pick_from_list(&mut self, label: &str, items: &[String]) -> io::Result<Option<String>> {
        if items.is_empty() {
            return Ok(None);
        }
        let mut selected = 0usize;
        let mut offset = 0usize;
        loop {
            let (cols, rows) = terminal::size()?;
            let max_lines = rows.saturating_sub(3) as usize;
            if selected < offset {
                offset = selected;
            }
            if selected >= offset + max_lines {
                offset = selected.saturating_sub(max_lines.saturating_sub(1));
            }
            self.status = format!("{label}: ↑/↓ select, Enter open, Esc cancel");
            let mut out = stdout();
            self.render(&mut out)?;
            let title = format!("{label} ({}/{})", selected + 1, items.len());
            queue!(
                out,
                cursor::MoveTo(0, 1),
                SetForegroundColor(self.colors.keyword),
                Print(truncate(&title, cols as usize)),
                ResetColor
            )?;
            for (i, item) in items.iter().enumerate().skip(offset).take(max_lines) {
                let y = (i - offset + 2) as u16;
                let prefix = if i == selected { "> " } else { "  " };
                let line = format!("{prefix}{}", truncate(item, cols as usize - 2));
                if i == selected {
                    queue!(
                        out,
                        cursor::MoveTo(0, y),
                        SetForegroundColor(self.colors.selection),
                        Print(line),
                        ResetColor
                    )?;
                } else {
                    queue!(out, cursor::MoveTo(0, y), Print(line))?;
                }
            }
            out.flush()?;

            if let Some(Event::Key(key)) = self.read_event()? {
                match key.code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected + 1 < items.len() {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        return Ok(Some(items[selected].clone()));
                    }
                    KeyCode::Esc => return Ok(None),
                    _ => {}
                }
            }
        }
    }

    fn insert_char_multi(&mut self, ch: char) {
        let mut by_line: std::collections::BTreeMap<usize, Vec<usize>> = std::collections::BTreeMap::new();
        by_line.entry(self.cursor_y).or_default().push(self.cursor_x);
        for (x, y) in &self.extra_cursors {
            by_line.entry(*y).or_default().push(*x);
        }
        for (y, mut xs) in by_line {
            xs.sort_by(|a, b| b.cmp(a));
            if let Some(line) = self.lines.get_mut(y) {
                for x in xs {
                    if x <= line.len() {
                        line.insert(x, ch);
                    }
                }
            }
        }
        self.cursor_x += 1;
        for cursor in &mut self.extra_cursors {
            cursor.0 += 1;
        }
        self.dirty = true;
    }

    fn backspace_multi(&mut self) {
        let mut by_line: std::collections::BTreeMap<usize, Vec<usize>> = std::collections::BTreeMap::new();
        by_line.entry(self.cursor_y).or_default().push(self.cursor_x);
        for (x, y) in &self.extra_cursors {
            by_line.entry(*y).or_default().push(*x);
        }
        for (y, mut xs) in by_line {
            xs.sort_by(|a, b| b.cmp(a));
            if let Some(line) = self.lines.get_mut(y) {
                for x in xs {
                    if x > 0 && x <= line.len() {
                        line.remove(x - 1);
                    }
                }
            }
        }
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
        for cursor in &mut self.extra_cursors {
            if cursor.0 > 0 {
                cursor.0 -= 1;
            }
        }
        self.dirty = true;
    }

    fn add_block_cursor(&mut self, delta: i32) {
        let new_y = if delta < 0 {
            self.cursor_y.saturating_sub((-delta) as usize)
        } else {
            self.cursor_y.saturating_add(delta as usize)
        };
        if new_y >= self.lines.len() {
            return;
        }
        let pos = (self.cursor_x.min(self.lines[new_y].len()), new_y);
        if pos.1 != self.cursor_y || pos.0 != self.cursor_x {
            self.extra_cursors.push((self.cursor_x, self.cursor_y));
        }
        self.cursor_y = new_y;
        self.cursor_x = pos.0;
        self.dedupe_cursors();
    }

    fn add_next_match(&mut self) {
        if let Some(word) = self.current_word() {
            if let Some((y, x)) = self.find_next_occurrence(&word) {
                self.extra_cursors.push((self.cursor_x, self.cursor_y));
                self.cursor_y = y;
                self.cursor_x = x;
                self.last_search = word;
                self.dedupe_cursors();
                self.status = format!("Added match at {}:{}", y + 1, x + 1);
                return;
            }
            self.status = "No next match.".to_string();
        } else {
            self.status = "No word under cursor.".to_string();
        }
    }

    fn goto_next_match_word(&mut self) {
        if let Some(word) = self.current_or_last_word() {
            if let Some((y, x)) = self.find_next_occurrence(&word) {
                self.cursor_y = y;
                self.cursor_x = x;
                self.last_search = word;
                self.status = "Next match.".to_string();
            } else {
                self.status = "No next match.".to_string();
            }
        } else {
            self.status = "No word under cursor.".to_string();
        }
    }

    fn goto_prev_match_word(&mut self) {
        if let Some(word) = self.current_or_last_word() {
            if let Some((y, x)) = self.find_prev_occurrence(&word) {
                self.cursor_y = y;
                self.cursor_x = x;
                self.last_search = word;
                self.status = "Prev match.".to_string();
            } else {
                self.status = "No previous match.".to_string();
            }
        } else {
            self.status = "No word under cursor.".to_string();
        }
    }

    fn current_word(&self) -> Option<String> {
        let line = self.lines.get(self.cursor_y)?;
        if self.cursor_x > line.len() {
            return None;
        }
        let bytes = line.as_bytes();
        let mut start = self.cursor_x;
        while start > 0 {
            let ch = bytes[start - 1] as char;
            if ch.is_alphanumeric() || ch == '_' {
                start -= 1;
            } else {
                break;
            }
        }
        let mut end = self.cursor_x;
        while end < bytes.len() {
            let ch = bytes[end] as char;
            if ch.is_alphanumeric() || ch == '_' {
                end += 1;
            } else {
                break;
            }
        }
        if start == end {
            return None;
        }
        Some(line[start..end].to_string())
    }

    fn current_or_last_word(&self) -> Option<String> {
        if let Some(word) = self.current_word() {
            return Some(word);
        }
        if self.last_search.is_empty() {
            None
        } else {
            Some(self.last_search.clone())
        }
    }

    fn find_next_occurrence(&self, word: &str) -> Option<(usize, usize)> {
        let mut y = self.cursor_y;
        let mut x = self.cursor_x + 1;
        for _ in 0..self.lines.len() {
            if y >= self.lines.len() {
                y = 0;
                x = 0;
            }
            if let Some(pos) = self.lines[y][x..].find(word) {
                return Some((y, x + pos));
            }
            y += 1;
            x = 0;
        }
        None
    }

    fn find_prev_occurrence(&self, word: &str) -> Option<(usize, usize)> {
        if self.lines.is_empty() {
            return None;
        }
        let mut y = self.cursor_y;
        let mut x = self.cursor_x.saturating_sub(1);
        for _ in 0..self.lines.len() {
            if y >= self.lines.len() {
                y = self.lines.len() - 1;
                x = self.lines[y].len();
            }
            let line = &self.lines[y];
            let search_slice = if x <= line.len() { &line[..x] } else { line.as_str() };
            if let Some(pos) = search_slice.rfind(word) {
                return Some((y, pos));
            }
            if y == 0 {
                y = self.lines.len() - 1;
                x = self.lines[y].len();
            } else {
                y -= 1;
                x = self.lines[y].len();
            }
        }
        None
    }

    fn dedupe_cursors(&mut self) {
        let mut seen = std::collections::BTreeSet::new();
        self.extra_cursors.retain(|pos| seen.insert(*pos));
        self.extra_cursors
            .retain(|(x, y)| !(*x == self.cursor_x && *y == self.cursor_y));
    }

    fn collect_symbols(&self) -> Vec<(usize, String)> {
        collect_symbols_for(self.language, &self.lines)
    }

    fn record_undo(&mut self) {
        self.undo.push(self.lines.clone());
        if self.undo.len() > 100 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            self.redo.push(self.lines.clone());
            self.lines = prev;
            self.cursor_x = self.cursor_x.min(self.current_line_len());
            self.status = "Undo".to_string();
            self.dirty = true;
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(self.lines.clone());
            self.lines = next;
            self.cursor_x = self.cursor_x.min(self.current_line_len());
            self.status = "Redo".to_string();
            self.dirty = true;
        }
    }

    fn replace_prompt(&mut self) -> io::Result<()> {
        let needle = self.prompt("Search")?;
        if needle.is_empty() {
            return Ok(());
        }
        let replace = self.prompt("Replace")?;
        if !self.can_edit() {
            return Ok(());
        }
        let mut preview = None;
        let mut count = 0usize;
        self.record_undo();
        for (i, line) in self.lines.iter_mut().enumerate() {
            if line.contains(&needle) {
                if preview.is_none() {
                    preview = Some(i + 1);
                }
                let new_line = line.replace(&needle, &replace);
                if new_line != *line {
                    *line = new_line;
                    count += 1;
                }
            }
        }
        if let Some(line_no) = preview {
            self.status = format!("Replaced {count} line(s), preview line {line_no}");
        } else {
            self.status = "No matches.".to_string();
        }
        self.dirty = true;
        Ok(())
    }

    fn toggle_comment(&mut self) {
        if !self.can_edit() {
            return;
        }
        if let Some(line) = self.lines.get_mut(self.cursor_y) {
            let prefix = match self.language {
                Language::Python | Language::Perl | Language::Ruby | Language::Shell | Language::CoffeeScript => "# ",
                Language::Lua | Language::Haskell => "-- ",
                Language::Steelconf => ";; ",
                _ => "// ",
            };
            let trimmed = line.trim_start();
            let indent_len = line.len() - trimmed.len();
            if trimmed.starts_with(prefix) {
                line.replace_range(indent_len..indent_len + prefix.len(), "");
            } else {
                line.insert_str(indent_len, prefix);
            }
            self.dirty = true;
        }
    }

    fn next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.current_tab = (self.current_tab + 1) % self.tabs.len();
        if let Some(path) = self.tabs.get(self.current_tab).cloned() {
            let _ = self.open_file(path);
        }
    }

    fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        };
        self.colors = self.theme.colors();
        self.status = match self.theme {
            Theme::Dark => "Theme: dark".to_string(),
            Theme::Light => "Theme: light".to_string(),
        };
    }

    fn open_settings_menu(&mut self) -> io::Result<()> {
        let items = vec![
            format!("Theme: {}", self.theme_label()),
            format!("C palette: {}", self.palette_c.as_str()),
            format!("C++ palette: {}", self.palette_cpp.as_str()),
            format!("Python palette: {}", self.palette_py.as_str()),
        ];
        let choice = self.pick_from_list("Settings", &items)?;
        let Some(item) = choice else { return Ok(()); };
        if item.starts_with("Theme:") {
            self.toggle_theme();
        } else if item.starts_with("C palette:") {
            self.palette_c = self.palette_c.next();
            self.status = format!("C palette: {}", self.palette_c.as_str());
        } else if item.starts_with("C++ palette:") {
            self.palette_cpp = self.palette_cpp.next();
            self.status = format!("C++ palette: {}", self.palette_cpp.as_str());
        } else if item.starts_with("Python palette:") {
            self.palette_py = self.palette_py.next();
            self.status = format!("Python palette: {}", self.palette_py.as_str());
        }
        let _ = self.save_editor_config();
        Ok(())
    }

    fn theme_label(&self) -> &'static str {
        match self.theme {
            Theme::Dark => "dark",
            Theme::Light => "light",
        }
    }

    fn save_editor_config(&self) -> io::Result<()> {
        let base = config_root().join("steel");
        fs::create_dir_all(&base)?;
        let path = base.join("steecleditor.conf");
        let autosave = if self.autosave { self.autosave_interval.as_secs() } else { 0 };
        let content = format!(
            "theme={}\nautosave_interval={}\npalette_c={}\npalette_cpp={}\npalette_py={}\n",
            self.theme_label(),
            autosave,
            self.palette_c.as_str(),
            self.palette_cpp.as_str(),
            self.palette_py.as_str(),
        );
        fs::write(path, content)?;
        Ok(())
    }

    fn run_steel(&mut self) -> io::Result<()> {
        let root = find_workspace_root(&self.file).unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        self.status = "Running steel run...".to_string();
        let mut out = stdout();
        self.render(&mut out)?;
        let steel_bin = resolve_steel_bin().unwrap_or_else(|| PathBuf::from("steel"));
        let output = Command::new(steel_bin)
            .arg("run")
            .current_dir(root)
            .output();
        match output {
            Ok(result) => {
                let mut lines = Vec::new();
                lines.push(format!("$ steel run"));
                if !result.stdout.is_empty() {
                    lines.extend(String::from_utf8_lossy(&result.stdout).lines().map(|s| s.to_string()));
                }
                if !result.stderr.is_empty() {
                    lines.extend(String::from_utf8_lossy(&result.stderr).lines().map(|s| s.to_string()));
                }
                self.run_output = lines;
                self.run_errors = parse_run_errors(&self.run_output);
                self.run_status = result.status.code();
                self.show_run_panel = true;
                self.status = "steel run complete.".to_string();
            }
            Err(err) => {
                self.run_output = vec![format!("Failed to run steel: {err}")];
                self.run_errors = Vec::new();
                self.run_status = None;
                self.show_run_panel = true;
                self.status = "steel run failed.".to_string();
            }
        }
        Ok(())
    }

    fn open_native_terminal(&mut self) -> io::Result<()> {
        let root = find_workspace_root(&self.file)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let launched = if cfg!(windows) {
            self.launch_windows_terminal(&root)
        } else if cfg!(target_os = "macos") {
            self.launch_macos_terminal(&root)
        } else {
            self.launch_linux_terminal(&root)
        };
        if launched {
            self.status = "Opened native terminal.".to_string();
        } else {
            self.status = "Failed to open native terminal.".to_string();
        }
        self.show_terminal_panel = false;
        Ok(())
    }

    fn launch_macos_terminal(&self, root: &Path) -> bool {
        Command::new("open")
            .args(["-a", "Terminal"])
            .arg(root)
            .spawn()
            .is_ok()
    }

    fn launch_windows_terminal(&self, root: &Path) -> bool {
        let root = root.display().to_string();
        let wt = Command::new("cmd")
            .args(["/C", "start", "", "wt.exe", "-d"])
            .arg(&root)
            .spawn()
            .is_ok();
        if wt {
            return true;
        }
        Command::new("cmd")
            .args(["/C", "start", "", "cmd.exe", "/K"])
            .arg(format!("cd /d {root}"))
            .spawn()
            .is_ok()
    }

    fn launch_linux_terminal(&self, root: &Path) -> bool {
        if let Ok(term) = std::env::var("TERMINAL") {
            if self.spawn_terminal(&term, root) {
                return true;
            }
        }
        let candidates = [
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "kitty",
            "alacritty",
            "xterm",
        ];
        for term in candidates {
            if self.spawn_terminal(term, root) {
                return true;
            }
        }
        false
    }

    fn spawn_terminal(&self, term: &str, root: &Path) -> bool {
        let root_str = root.display().to_string();
        let root_quoted = root_str.replace('"', "\\\"");
        let mut cmd = Command::new(term);
        match term {
            "gnome-terminal" | "xfce4-terminal" | "alacritty" => {
                cmd.args(["--working-directory", &root_str]);
            }
            "konsole" => {
                cmd.args(["--workdir", &root_str]);
            }
            "kitty" => {
                cmd.args(["--directory", &root_str]);
            }
            "x-terminal-emulator" => {
                cmd.args([
                    "-e",
                    "sh",
                    "-c",
                    &format!("cd \"{root_quoted}\"; exec $SHELL"),
                ]);
            }
            "xterm" => {
                cmd.args([
                    "-e",
                    "sh",
                    "-c",
                    &format!("cd \"{root_quoted}\"; exec $SHELL"),
                ]);
            }
            _ => {}
        }
        cmd.spawn().is_ok()
    }

    fn terminal_panel_height(&self, rows: u16) -> u16 {
        let available = rows.saturating_sub(4);
        if available < 4 {
            return 0;
        }
        let desired = (rows / 3).max(6);
        desired.min(available)
    }

    fn render_terminal_panel(
        &self,
        out: &mut io::Stdout,
        cols: usize,
        rows: usize,
        panel_height: u16,
    ) -> io::Result<()> {
        if panel_height == 0 {
            return Ok(());
        }
        let start_row = rows.saturating_sub(1 + panel_height as usize) as u16;
        let cmd = if self.last_terminal_cmd.is_empty() {
            "<no command>"
        } else {
            self.last_terminal_cmd.as_str()
        };
        let header_status = if self.last_terminal_cmd.is_empty() {
            "idle".to_string()
        } else {
            match self.terminal_status {
                Some(code) => format!("exit {code}"),
                None => "error".to_string(),
            }
        };
        let header = truncate(&format!("Terminal | {cmd} | {header_status}"), cols);
        queue!(
            out,
            cursor::MoveTo(0, start_row),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(self.colors.status_ok),
            Print(header),
            ResetColor
        )?;
        let max_lines = panel_height.saturating_sub(1) as usize;
        for (i, line) in self.terminal_output.iter().rev().take(max_lines).enumerate() {
            let y = start_row + 1 + i as u16;
            queue!(out, cursor::MoveTo(0, y), Clear(ClearType::CurrentLine))?;
            let text = truncate(line, cols);
            queue!(out, Print(text))?;
        }
        Ok(())
    }

    fn insert_snippet(&mut self) -> io::Result<()> {
        if !self.can_edit() {
            return Ok(());
        }
        let labels = STEELCONF_SNIPPETS
            .iter()
            .map(|snippet| snippet.label.to_string())
            .collect::<Vec<_>>();
        let choice = self.pick_from_list("Insert snippet", &labels)?;
        let Some(label) = choice else { return Ok(()); };
        if let Some(snippet) = STEELCONF_SNIPPETS.iter().find(|snippet| snippet.label == label) {
            self.record_undo();
            self.insert_snippet_lines(snippet.body);
        }
        Ok(())
    }

    fn insert_snippet_lines(&mut self, snippet: &str) {
        let insert_at = self.cursor_y + 1;
        let snippet = expand_snippet_placeholders(snippet);
        let mut insert_lines = snippet.lines().map(|s| s.to_string()).collect::<Vec<_>>();
        if insert_lines.is_empty() {
            return;
        }
        for (offset, line) in insert_lines.drain(..).enumerate() {
            self.lines.insert(insert_at + offset, line);
        }
        self.cursor_y = insert_at;
        self.cursor_x = 0;
        self.dirty = true;
        self.status = "Snippet inserted.".to_string();
    }

    fn refresh_glob_preview(&mut self) {
        if self.last_glob_refresh.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.last_glob_refresh = Instant::now();
        if self.language != Language::Steelconf {
            self.glob_preview = vec!["Glob preview only for steelconf.".to_string()];
            return;
        }
        let root = find_workspace_root(&self.file).unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        self.glob_preview = collect_glob_preview(&root, &self.lines);
        if self.glob_preview.is_empty() {
            self.glob_preview = vec!["No cglob patterns found.".to_string()];
        }
    }

    fn complete_steelconf(&mut self, prefix: &str, start: usize) -> io::Result<Option<bool>> {
        let items = self.collect_completion_items(prefix);
        if items.is_empty() {
            return Ok(None);
        }
        let choice = if items.len() == 1 {
            Some(items[0].clone())
        } else {
            let labels = items.iter().map(|item| item.label.clone()).collect::<Vec<_>>();
            let picked = self.pick_from_list("Complete", &labels)?;
            picked.and_then(|label| items.into_iter().find(|item| item.label == label))
        };

        if let Some(item) = choice {
            self.apply_completion_item(start, prefix, &item);
            return Ok(Some(true));
        }
        Ok(Some(false))
    }

    fn collect_completion_items(&self, prefix: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        for snippet in STEELCONF_SNIPPETS {
            if snippet.trigger.starts_with(prefix) {
                items.push(CompletionItem {
                    label: format!("snippet: {}", snippet.label),
                    insert: snippet.body.to_string(),
                    is_snippet: true,
                });
            }
        }
        for block in BLOCK_KEYWORDS {
            if block.starts_with(prefix) {
                items.push(CompletionItem {
                    label: format!("block: {}", block),
                    insert: block.to_string(),
                    is_snippet: false,
                });
            }
        }
        for directive in DIRECTIVES {
            if directive.starts_with(prefix) {
                items.push(CompletionItem {
                    label: format!("directive: {}", directive),
                    insert: directive.to_string(),
                    is_snippet: false,
                });
            }
        }
        items
    }

    fn collect_language_completion_items(&self, prefix: &str) -> Vec<CompletionItem> {
        language_completion_items(self.language, prefix)
    }

    fn apply_completion_item(&mut self, start: usize, prefix: &str, item: &CompletionItem) {
        if item.is_snippet {
            self.apply_snippet(start, prefix, &item.insert);
            return;
        }
        if let Some(line) = self.lines.get_mut(self.cursor_y) {
            let end = start + prefix.len();
            line.replace_range(start..end, &item.insert);
            self.cursor_x = start + item.insert.len();
            self.dirty = true;
            if let Some(snippet) = snippet_for_trigger(&item.insert) {
                if line.trim() == item.insert {
                    self.insert_snippet_lines(snippet);
                }
            }
        }
    }

    fn apply_snippet(&mut self, start: usize, prefix: &str, snippet: &str) {
        self.record_undo();
        let snippet = expand_snippet_placeholders(snippet);
        let mut lines = snippet.lines();
        let Some(first_line) = lines.next() else {
            return;
        };
        if let Some(line) = self.lines.get_mut(self.cursor_y) {
            let end = start + prefix.len();
            line.replace_range(start..end, first_line);
            self.cursor_x = start + first_line.len();
        }
        let mut insert_at = self.cursor_y + 1;
        for line in lines {
            self.lines.insert(insert_at, line.to_string());
            insert_at += 1;
        }
        self.dirty = true;
        self.status = "Snippet inserted.".to_string();
    }

    fn update_auto_completion(&mut self) {
        if self.language == Language::Other {
            self.clear_completion();
            return;
        }
        let (prefix, start) = self.current_prefix();
        if prefix.len() < 2 {
            self.clear_completion();
            return;
        }
        let items = if self.language == Language::Steelconf {
            self.collect_completion_items(&prefix)
        } else {
            self.collect_language_completion_items(&prefix)
        };
        if items.is_empty() {
            self.clear_completion();
            return;
        }
        self.completion_active = true;
        self.completion_items = items;
        self.completion_selected = 0;
        self.completion_start = start;
        self.completion_prefix = prefix;
    }

    fn clear_completion(&mut self) {
        self.completion_active = false;
        self.completion_items.clear();
        self.completion_selected = 0;
        self.completion_start = 0;
        self.completion_prefix.clear();
    }

    fn show_completion_debug(&mut self, verbose: bool) {
        let (prefix, start) = self.current_prefix();
        let items = if self.language == Language::Steelconf {
            self.collect_completion_items(&prefix)
        } else {
            self.collect_language_completion_items(&prefix)
        };
        let lines = format_completion_debug_lines(
            self.language,
            &self.file,
            &prefix,
            start,
            &items,
            verbose,
        );
        self.terminal_output = lines;
        self.show_terminal_panel = true;
        self.status = if verbose {
            "Debug completion source --verbose".to_string()
        } else {
            "Debug completion source".to_string()
        };
    }

    fn completion_move(&mut self, delta: i32) {
        if self.completion_items.is_empty() {
            return;
        }
        let len = self.completion_items.len();
        let current = self.completion_selected as i32;
        let next = (current + delta).rem_euclid(len as i32) as usize;
        self.completion_selected = next;
    }

    fn apply_completion(&mut self) {
        if !self.completion_active || self.completion_items.is_empty() {
            return;
        }
        let item = self.completion_items[self.completion_selected].clone();
        let start = self.completion_start;
        let prefix = self.completion_prefix.clone();
        self.apply_completion_item(start, &prefix, &item);
        self.clear_completion();
    }

    fn render_completion_popup(&self, out: &mut io::Stdout, cols: usize, rows: usize) -> io::Result<()> {
        if self.completion_items.is_empty() {
            return Ok(());
        }
        let max_lines = 6usize.min(rows.saturating_sub(3) as usize);
        if max_lines == 0 {
            return Ok(());
        }
        let start_row = rows.saturating_sub(2 + max_lines) as u16;
        for (idx, item) in self.completion_items.iter().take(max_lines).enumerate() {
            let y = start_row + idx as u16;
            let line = truncate(&item.label, cols);
            if idx == self.completion_selected {
                queue!(
                    out,
                    cursor::MoveTo(0, y),
                    SetForegroundColor(self.colors.selection),
                    Print(line),
                    ResetColor
                )?;
            } else {
                queue!(out, cursor::MoveTo(0, y), Print(line))?;
            }
        }
        Ok(())
    }

    fn maybe_restore_session(&mut self) -> io::Result<()> {
        self.pending_restore = false;
        if self.session_paths.is_empty() {
            return Ok(());
        }
        let answer = self.prompt("Restore previous session? (y/n)")?;
        if answer.to_lowercase().starts_with('y') {
            let mut first = true;
            for path in self.session_paths.clone() {
                if first {
                    let _ = self.open_file(path);
                    first = false;
                } else if path.exists() && !self.tabs.iter().any(|p| p == &path) {
                    self.tabs.push(path);
                }
            }
            if self.tabs.is_empty() {
                self.tabs.push(self.file.clone());
                self.current_tab = 0;
            }
            self.status = "Session restored.".to_string();
        } else {
            self.status = "Session skipped.".to_string();
        }
        Ok(())
    }

    fn save_session(&self) {
        let dir = config_root().join("steel");
        let path = dir.join("steecleditor.session");
        let _ = fs::create_dir_all(&dir);
        let content = self
            .tabs
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");
        let _ = fs::write(path, content);
    }

    fn smart_indent_extra(&self, line: &str) -> String {
        let trimmed = line.trim_end();
        let indent = match self.language {
            Language::Python => 4,
            Language::C | Language::Cpp | Language::Java | Language::CSharp => 4,
            Language::Steelconf | Language::Ocaml | Language::Zig => 2,
            _ => 2,
        };
        let pad = " ".repeat(indent);
        if self.language == Language::Python && trimmed.ends_with(':') {
            return pad;
        }
        if matches!(self.language, Language::C | Language::Cpp | Language::Java | Language::CSharp) && trimmed.ends_with('{') {
            return pad;
        }
        if self.language == Language::Steelconf && trimmed.ends_with("..") {
            return pad;
        }
        String::new()
    }

    fn render_inline_with_keywords(
        &self,
        out: &mut io::Stdout,
        line: &str,
        cols: usize,
        raw_state: &mut Option<MultiLineState>,
        py_state: &mut Option<String>,
        py_carry: &mut String,
    ) -> io::Result<()> {
        let mut count = 0usize;
        let mut chars = line.chars().peekable();
        let comment_start = match self.language {
            Language::Python
            | Language::Perl
            | Language::Ruby
            | Language::Shell
            | Language::CoffeeScript
            | Language::PowerShell
            | Language::Makefile => "#",
            Language::Lua => "--",
            Language::Haskell => "--",
            Language::Pascal | Language::Algol => "//",
            _ => "//",
        };

        if matches!(self.language, Language::C | Language::Cpp) {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                queue!(
                    out,
                    SetForegroundColor(self.colors.directive),
                    Print(truncate(line, cols)),
                    ResetColor
                )?;
                return Ok(());
            }
        }
        if self.language == Language::Php {
            let trimmed = line.trim_start();
            if trimmed.starts_with("<?") || trimmed.starts_with("?>") {
                queue!(
                    out,
                    SetForegroundColor(self.colors.directive),
                    Print(truncate(line, cols)),
                    ResetColor
                )?;
                return Ok(());
            }
        }

        while let Some(ch) = chars.next() {
            if count >= cols {
                break;
            }
            if self.language == Language::Haskell && ch == '{' && chars.peek() == Some(&'-') {
                chars.next();
                let mut comment = String::from("{-");
                let mut depth = 1usize;
                while let Some(n) = chars.next() {
                    comment.push(n);
                    if n == '{' && chars.peek() == Some(&'-') {
                        chars.next();
                        comment.push('-');
                        depth += 1;
                    } else if n == '-' && chars.peek() == Some(&'}') {
                        chars.next();
                        comment.push('}');
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            break;
                        }
                    }
                }
                count += comment.len();
                queue!(out, SetForegroundColor(self.colors.comment), Print(&comment), ResetColor)?;
                if depth > 0 {
                    *raw_state = Some(MultiLineState::HaskellBlock(depth));
                    return Ok(());
                }
                continue;
            }
            if self.language == Language::Lua && ch == '-' && chars.peek() == Some(&'-') {
                let mut probe = chars.clone();
                let _ = probe.next();
                if probe.peek() == Some(&'[') {
                    let _ = probe.next();
                    let mut eq = String::new();
                    while probe.peek() == Some(&'=') {
                        eq.push('=');
                        let _ = probe.next();
                    }
                    if probe.peek() == Some(&'[') {
                        chars.next();
                        let mut comment = String::from("--[");
                        comment.push_str(&eq);
                        comment.push('[');
                        for _ in 0..eq.len() + 1 {
                            chars.next();
                        }
                        let mut end = String::from("]");
                        end.push_str(&eq);
                        end.push(']');
                        let mut found = false;
                        let mut window = String::new();
                        while let Some(n) = chars.next() {
                            comment.push(n);
                            window.push(n);
                            if window.len() > end.len() {
                                window.remove(0);
                            }
                            if window == end {
                                found = true;
                                break;
                            }
                        }
                        count += comment.len();
                        queue!(out, SetForegroundColor(self.colors.comment), Print(&comment), ResetColor)?;
                        if !found {
                            *raw_state = Some(MultiLineState::Comment(end));
                            return Ok(());
                        }
                        continue;
                    }
                }
            }
            if self.language == Language::Lua && ch == '[' {
                let mut probe = chars.clone();
                let mut eq = String::new();
                while probe.peek() == Some(&'=') {
                    eq.push('=');
                    let _ = probe.next();
                }
                if probe.peek() == Some(&'[') {
                    let mut literal = String::from("[");
                    literal.push_str(&eq);
                    literal.push('[');
                    for _ in 0..eq.len() + 1 {
                        chars.next();
                    }
                    let mut end = String::from("]");
                    end.push_str(&eq);
                    end.push(']');
                    let mut found = false;
                    let mut window = String::new();
                    while let Some(n) = chars.next() {
                        literal.push(n);
                        window.push(n);
                        if window.len() > end.len() {
                            window.remove(0);
                        }
                        if window == end {
                            found = true;
                            break;
                        }
                    }
                    count += literal.len();
                    queue!(out, SetForegroundColor(self.colors.string), Print(&literal), ResetColor)?;
                    if !found {
                        *raw_state = Some(MultiLineState::String(end));
                        return Ok(());
                    }
                    continue;
                }
            }
            if matches!(self.language, Language::JavaScript | Language::TypeScript) && ch == '`' {
                let mut literal = String::from("`");
                let mut segments: Vec<(String, bool)> = Vec::new();
                let mut segment = String::from("`");
                while let Some(n) = chars.next() {
                    literal.push(n);
                    segment.push(n);
                    if n == '\\' {
                        if let Some(esc) = chars.next() {
                            literal.push(esc);
                            segment.push(esc);
                        }
                        continue;
                    }
                    if n == '$' && chars.peek() == Some(&'{') {
                        let esc = chars.next().unwrap();
                        literal.push(esc);
                        segment.push(esc);
                        let mut brace_depth = 1usize;
                        while let Some(m) = chars.next() {
                            literal.push(m);
                            segment.push(m);
                            if m == '{' {
                                brace_depth += 1;
                            } else if m == '}' {
                                brace_depth = brace_depth.saturating_sub(1);
                                if brace_depth == 0 {
                                    break;
                                }
                            }
                        }
                        segments.push((segment.clone(), true));
                        segment.clear();
                        continue;
                    }
                    if n == '`' {
                        break;
                    }
                }
                count += literal.len();
                if segments.is_empty() {
                    queue!(out, SetForegroundColor(self.colors.string), Print(&literal), ResetColor)?;
                } else {
                    for (seg, is_expr) in segments {
                        let color = if is_expr { self.colors.keyword } else { self.colors.string };
                        queue!(out, SetForegroundColor(color), Print(&seg), ResetColor)?;
                    }
                    if !segment.is_empty() {
                        queue!(out, SetForegroundColor(self.colors.string), Print(&segment), ResetColor)?;
                    }
                }
                continue;
            }
            if self.language == Language::Go && ch == '`' {
                let mut literal = String::from("`");
                let mut found = false;
                while let Some(n) = chars.next() {
                    literal.push(n);
                    if n == '`' {
                        found = true;
                        break;
                    }
                }
                count += literal.len();
                queue!(out, SetForegroundColor(self.colors.string), Print(&literal), ResetColor)?;
                if !found {
                    *raw_state = Some(MultiLineState::String("`".to_string()));
                    return Ok(());
                }
                continue;
            }
            if self.language == Language::Rust && ch == 'r' {
                let mut probe = chars.clone();
                let mut hashes = String::new();
                while probe.peek() == Some(&'#') {
                    hashes.push('#');
                    let _ = probe.next();
                }
                if probe.peek() == Some(&'"') {
                    let mut literal = String::from("r");
                    literal.push_str(&hashes);
                    literal.push('"');
                    for _ in 0..hashes.len() {
                        chars.next();
                    }
                    chars.next();
                    let mut end = String::from("\"");
                    end.push_str(&hashes);
                    let mut found = false;
                    let mut window = String::new();
                    while let Some(n) = chars.next() {
                        literal.push(n);
                        window.push(n);
                        if window.len() > end.len() {
                            window.remove(0);
                        }
                        if window == end {
                            found = true;
                            break;
                        }
                    }
                    count += literal.len();
                    queue!(out, SetForegroundColor(self.colors.string), Print(&literal), ResetColor)?;
                    if !found {
                        *raw_state = Some(MultiLineState::String(end));
                        return Ok(());
                    }
                    continue;
                }
            }
            if self.language == Language::CoffeeScript && ch == '#' && chars.peek() == Some(&'#') {
                let mut probe = chars.clone();
                let _ = probe.next();
                if probe.peek() == Some(&'#') {
                    chars.next();
                    chars.next();
                    let mut comment = String::from("###");
                    let tail: String = chars.by_ref().collect();
                    comment.push_str(&tail);
                    let found = coffee_block_comment_closed(&tail);
                    count += comment.len();
                    queue!(out, SetForegroundColor(self.colors.comment), Print(&comment), ResetColor)?;
                    if !found {
                        *raw_state = Some(MultiLineState::Comment("###".to_string()));
                        return Ok(());
                    }
                    continue;
                }
            }
            if self.language == Language::CoffeeScript && (ch == '"' || ch == '\'') {
                if chars.peek() == Some(&ch) {
                    let mut probe = chars.clone();
                    let _ = probe.next();
                    if probe.peek() == Some(&ch) {
                        chars.next();
                        chars.next();
                        let mut literal = String::new();
                        literal.push(ch);
                        literal.push(ch);
                        literal.push(ch);
                        let delimiter = literal.clone();
                        let mut window = String::new();
                        let mut found = false;
                        while let Some(n) = chars.next() {
                            literal.push(n);
                            window.push(n);
                            if window.len() > delimiter.len() {
                                window.remove(0);
                            }
                            if window == delimiter {
                                found = true;
                                break;
                            }
                        }
                        count += literal.len();
                        queue!(out, SetForegroundColor(self.colors.string), Print(&literal), ResetColor)?;
                        if !found {
                            *raw_state = Some(MultiLineState::String(delimiter));
                            return Ok(());
                        }
                        continue;
                    }
                }
            }
            if matches!(self.language, Language::Pascal | Language::Algol) && ch == '{' {
                let mut comment = String::from("{");
                let mut found = false;
                while let Some(n) = chars.next() {
                    comment.push(n);
                    if n == '}' {
                        found = true;
                        break;
                    }
                }
                count += comment.len();
                queue!(out, SetForegroundColor(self.colors.comment), Print(&comment), ResetColor)?;
                if !found {
                    *raw_state = Some(MultiLineState::Comment("}".to_string()));
                    return Ok(());
                }
                continue;
            }
            if self.language == Language::Pascal && ch == '(' && chars.peek() == Some(&'*') {
                chars.next();
                let mut comment = String::from("(*");
                let tail: String = chars.by_ref().collect();
                comment.push_str(&tail);
                let depth = pascal_nested_depth_after(&tail, 1);
                count += comment.len();
                queue!(out, SetForegroundColor(self.colors.comment), Print(&comment), ResetColor)?;
                if depth > 0 {
                    *raw_state = Some(MultiLineState::PascalBlock(depth));
                    return Ok(());
                }
                continue;
            }
            if self.language == Language::Algol && ch == '(' && chars.peek() == Some(&'*') {
                chars.next();
                let mut comment = String::from("(*");
                let mut prev = '\0';
                let mut found = false;
                while let Some(n) = chars.next() {
                    comment.push(n);
                    if prev == '*' && n == ')' {
                        found = true;
                        break;
                    }
                    prev = n;
                }
                count += comment.len();
                queue!(out, SetForegroundColor(self.colors.comment), Print(&comment), ResetColor)?;
                if !found {
                    *raw_state = Some(MultiLineState::Comment("*)".to_string()));
                    return Ok(());
                }
                continue;
            }
            if self.language == Language::Python && matches!(ch, 'f' | 'F' | 'r' | 'R' | 'b' | 'B' | 'u' | 'U') {
                let mut probe = chars.clone();
                let mut prefix = String::new();
                prefix.push(ch);
                let mut next = probe.peek().copied();
                for _ in 0..2 {
                    if let Some(p) = next {
                        if matches!(p, 'f' | 'F' | 'r' | 'R' | 'b' | 'B' | 'u' | 'U') {
                            prefix.push(p);
                            probe.next();
                            next = probe.peek().copied();
                            continue;
                        }
                    }
                    break;
                }
                if let Some(quote) = next {
                    if quote == '"' || quote == '\'' {
                        let (valid_prefix, has_f) = validate_python_prefix(&prefix);
                        if valid_prefix && has_f {
                            let consumed = prefix.len().saturating_sub(1);
                            for _ in 0..consumed {
                                chars.next();
                            }
                            let _ = chars.next();
                            let mut probe_quote = chars.clone();
                            let mut is_triple = false;
                            if probe_quote.peek() == Some(&quote) {
                                let _ = probe_quote.next();
                                if probe_quote.peek() == Some(&quote) {
                                    is_triple = true;
                                }
                            }
                            let mut literal = String::new();
                            literal.push_str(&prefix);
                            literal.push(quote);
                            if is_triple {
                                literal.push(quote);
                                literal.push(quote);
                                chars.next();
                                chars.next();
                                let delimiter = format!("{quote}{quote}{quote}");
                                let mut window = String::new();
                                let mut found = false;
                                let mut segments: Vec<(String, bool)> = Vec::new();
                                let mut segment = String::new();
                                let mut brace_depth = 0usize;
                                while let Some(n) = chars.next() {
                                    literal.push(n);
                                    if brace_depth == 0 {
                                        if n == '{' && chars.peek() == Some(&'{') {
                                            let esc = chars.next().unwrap();
                                            literal.push(esc);
                                            segment.push('{');
                                        } else if n == '}' && chars.peek() == Some(&'}') {
                                            let esc = chars.next().unwrap();
                                            literal.push(esc);
                                            segment.push('}');
                                        } else if n == '{' {
                                            if !segment.is_empty() {
                                                segments.push((segment.clone(), false));
                                                segment.clear();
                                            }
                                            segment.push(n);
                                            brace_depth = 1;
                                        } else {
                                            segment.push(n);
                                        }
                                    } else {
                                        segment.push(n);
                                        if n == '\\' {
                                            if let Some(esc) = chars.next() {
                                                literal.push(esc);
                                                segment.push(esc);
                                            }
                                        } else if n == '{' {
                                            if chars.peek() == Some(&'{') {
                                                let esc = chars.next().unwrap();
                                                literal.push(esc);
                                                segment.push(esc);
                                            } else {
                                                brace_depth += 1;
                                            }
                                        } else if n == '}' && brace_depth > 0 {
                                            if chars.peek() == Some(&'}') {
                                                let esc = chars.next().unwrap();
                                                literal.push(esc);
                                                segment.push(esc);
                                            } else {
                                                brace_depth -= 1;
                                                if brace_depth == 0 {
                                                    segments.push((segment.clone(), true));
                                                    segment.clear();
                                                }
                                            }
                                        }
                                    }
                                    window.push(n);
                                    if window.len() > delimiter.len() {
                                        window.remove(0);
                                    }
                                    if window == delimiter {
                                        found = true;
                                        break;
                                    }
                                }
                                count += literal.len();
                                if !segments.is_empty() {
                                    for (seg, is_expr) in segments {
                                        let color = if is_expr { self.colors.keyword } else { self.colors.string };
                                        queue!(out, SetForegroundColor(color), Print(seg), ResetColor)?;
                                    }
                                    if !segment.is_empty() {
                                        let color = if brace_depth > 0 { self.colors.keyword } else { self.colors.string };
                                        queue!(out, SetForegroundColor(color), Print(segment), ResetColor)?;
                                    }
                                } else {
                                    queue!(out, SetForegroundColor(self.colors.string), Print(&literal), ResetColor)?;
                                }
                                if !found {
                                    *py_state = Some(delimiter);
                                    py_carry.clear();
                                    if literal.len() >= 2 {
                                        let start = literal.len().saturating_sub(2);
                                        py_carry.push_str(&literal[start..]);
                                    } else {
                                        py_carry.push_str(&literal);
                                    }
                                    return Ok(());
                                }
                                continue;
                            }
                            let mut segments: Vec<(String, bool)> = Vec::new();
                            let mut segment = String::new();
                            let mut brace_depth = 0usize;
                            while let Some(n) = chars.next() {
                                literal.push(n);
                                if brace_depth == 0 {
                                    if n == '{' && chars.peek() == Some(&'{') {
                                        let esc = chars.next().unwrap();
                                        literal.push(esc);
                                        segment.push('{');
                                    } else if n == '}' && chars.peek() == Some(&'}') {
                                        let esc = chars.next().unwrap();
                                        literal.push(esc);
                                        segment.push('}');
                                    } else if n == '{' {
                                        if !segment.is_empty() {
                                            segments.push((segment.clone(), false));
                                            segment.clear();
                                        }
                                        segment.push(n);
                                        brace_depth = 1;
                                    } else {
                                        segment.push(n);
                                    }
                                } else {
                                    segment.push(n);
                                    if n == '\\' {
                                        if let Some(esc) = chars.next() {
                                            literal.push(esc);
                                            segment.push(esc);
                                        }
                                    } else if n == '{' {
                                        if chars.peek() == Some(&'{') {
                                            let esc = chars.next().unwrap();
                                            literal.push(esc);
                                            segment.push(esc);
                                        } else {
                                            brace_depth += 1;
                                        }
                                    } else if n == '}' && brace_depth > 0 {
                                        if chars.peek() == Some(&'}') {
                                            let esc = chars.next().unwrap();
                                            literal.push(esc);
                                            segment.push(esc);
                                        } else {
                                            brace_depth -= 1;
                                            if brace_depth == 0 {
                                                segments.push((segment.clone(), true));
                                                segment.clear();
                                            }
                                        }
                                    }
                                }
                                if n == quote && brace_depth == 0 {
                                    break;
                                }
                            }
                            count += literal.len();
                            if !segments.is_empty() {
                                for (seg, is_expr) in segments {
                                    let color = if is_expr { self.colors.keyword } else { self.colors.string };
                                    queue!(out, SetForegroundColor(color), Print(seg), ResetColor)?;
                                }
                                if !segment.is_empty() {
                                    let color = if brace_depth > 0 { self.colors.keyword } else { self.colors.string };
                                    queue!(out, SetForegroundColor(color), Print(segment), ResetColor)?;
                                }
                            } else {
                                queue!(out, SetForegroundColor(self.colors.string), Print(&literal), ResetColor)?;
                            }
                            continue;
                        }
                    }
                }
            }
            if self.language == Language::Python && (ch == '"' || ch == '\'') {
                if chars.peek() == Some(&ch) {
                    let mut probe = chars.clone();
                    let _ = probe.next();
                    if probe.peek() == Some(&ch) {
                        let mut literal = String::new();
                        literal.push(ch);
                        literal.push(ch);
                        literal.push(ch);
                        chars.next();
                        chars.next();
                        let delimiter = literal.clone();
                        let mut window = String::new();
                        let mut found = false;
                        while let Some(n) = chars.next() {
                            literal.push(n);
                            window.push(n);
                            if window.len() > delimiter.len() {
                                window.remove(0);
                            }
                            if window == delimiter {
                                found = true;
                                break;
                            }
                        }
                        count += literal.len();
                        let is_doc = line.trim_start().starts_with(&delimiter);
                        let color = if is_doc { self.color_doc_comment() } else { self.colors.string };
                        queue!(out, SetForegroundColor(color), Print(&literal), ResetColor)?;
                        if !found {
                            *py_state = Some(delimiter);
                            py_carry.clear();
                            if literal.len() >= 2 {
                                let start = literal.len().saturating_sub(2);
                                py_carry.push_str(&literal[start..]);
                            } else {
                                py_carry.push_str(&literal);
                            }
                            return Ok(());
                        }
                        continue;
                    }
                }
            }
            if matches!(self.language, Language::Php | Language::Perl | Language::Ruby | Language::Shell)
                && ch == '<'
                && chars.peek() == Some(&'<')
            {
                let rest: String = chars.clone().collect();
                if let Some(marker) = parse_heredoc_marker(self.language == Language::Shell, &rest) {
                    let whole = format!("<{rest}");
                    queue!(out, SetForegroundColor(self.colors.directive), Print(&whole), ResetColor)?;
                    let term = format!("LINE:{marker}");
                    *raw_state = Some(MultiLineState::String(term));
                    return Ok(());
                }
            }
            if self.language == Language::PowerShell {
                if let Some(quote) = is_powershell_here_string_start(ch, chars.peek().copied()) {
                    chars.next();
                    let mut here = String::from("@");
                    here.push(quote);
                    for n in chars.by_ref() {
                        here.push(n);
                    }
                    queue!(out, SetForegroundColor(self.colors.string), Print(&here), ResetColor)?;
                    let term = format!("LINE:{quote}@");
                    *raw_state = Some(MultiLineState::String(term));
                    return Ok(());
                }
            }
            if self.language == Language::Makefile && is_makefile_var_start(ch, chars.peek().copied()) {
                chars.next();
                let mut var = String::from("$(");
                let mut depth = 1usize;
                while let Some(n) = chars.next() {
                    var.push(n);
                    if n == '(' {
                        depth += 1;
                    } else if n == ')' {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            break;
                        }
                    }
                }
                count += var.len();
                queue!(out, SetForegroundColor(self.color_builtin()), Print(&var), ResetColor)?;
                continue;
            }
            if self.language == Language::Ruby && ch == '%' {
                if let Some(kind) = chars.peek().copied() {
                    if matches!(kind, 'r' | 'q' | 'Q' | 'w' | 'x' | 'i' | 'I') {
                        let mut literal = String::from("%");
                        literal.push(kind);
                        chars.next();
                        if let Some(delim) = chars.next() {
                            literal.push(delim);
                            let end = match delim {
                                '(' => ')',
                                '[' => ']',
                                '{' => '}',
                                '<' => '>',
                                _ => delim,
                            };
                            let mut depth = 1usize;
                            while let Some(n) = chars.next() {
                                literal.push(n);
                                if n == '\\' {
                                    if let Some(esc) = chars.next() {
                                        literal.push(esc);
                                    }
                                    continue;
                                }
                                if n == delim && delim != end {
                                    depth += 1;
                                } else if n == end {
                                    depth = depth.saturating_sub(1);
                                    if depth == 0 {
                                        break;
                                    }
                                }
                            }
                            count += literal.len();
                            queue!(out, SetForegroundColor(self.colors.string), Print(&literal), ResetColor)?;
                            continue;
                        }
                    }
                }
            }
            if matches!(self.language, Language::Php | Language::Perl | Language::Ruby) && matches!(ch, '$' | '@' | '%') {
                let mut var = String::new();
                var.push(ch);
                if self.language == Language::Ruby && ch == '@' && chars.peek() == Some(&'@') {
                    var.push('@');
                    chars.next();
                }
                while let Some(n) = chars.peek() {
                    if n.is_alphanumeric() || *n == '_' {
                        var.push(*n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if var.len() > 1 {
                    count += var.len();
                    queue!(out, SetForegroundColor(self.color_builtin()), Print(&var), ResetColor)?;
                    continue;
                }
            }
            if self.language == Language::Ruby && ch == ':' {
                if let Some(n) = chars.peek() {
                    if n.is_alphabetic() || *n == '_' {
                        let mut sym = String::from(":");
                        while let Some(c) = chars.peek() {
                            if c.is_alphanumeric() || *c == '_' || *c == '!' || *c == '?' {
                                sym.push(*c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        count += sym.len();
                        queue!(out, SetForegroundColor(self.color_type()), Print(sym), ResetColor)?;
                        continue;
                    }
                }
            }
            if ch == '"' {
                let mut string = String::from("\"");
                while let Some(n) = chars.next() {
                    string.push(n);
                    if n == '"' {
                        break;
                    }
                }
                count += string.len();
                queue!(out, SetForegroundColor(self.colors.string), Print(string), ResetColor)?;
                continue;
            }
            if ch == '\'' {
                let mut literal = String::from("'");
                while let Some(n) = chars.next() {
                    literal.push(n);
                    if n == '\'' {
                        break;
                    }
                }
                count += literal.len();
                queue!(out, SetForegroundColor(self.colors.string), Print(&literal), ResetColor)?;
                continue;
            }
            if matches!(self.language, Language::JavaScript | Language::TypeScript | Language::CoffeeScript | Language::Perl | Language::Ruby)
                && ch == '/'
                && chars.peek() != Some(&'/')
                && chars.peek() != Some(&'*')
            {
                let mut regex = String::from("/");
                let mut escaped = false;
                while let Some(n) = chars.next() {
                    regex.push(n);
                    if escaped {
                        escaped = false;
                        continue;
                    }
                    if n == '\\' {
                        escaped = true;
                        continue;
                    }
                    if n == '/' {
                        while let Some(flag) = chars.peek() {
                            if flag.is_ascii_alphabetic() {
                                regex.push(*flag);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        break;
                    }
                }
                count += regex.len();
                queue!(out, SetForegroundColor(self.colors.string), Print(&regex), ResetColor)?;
                continue;
            }

            if comment_start == "#" && ch == '#' {
                let mut comment = String::from("#");
                for n in chars {
                    comment.push(n);
                }
                queue!(out, SetForegroundColor(self.colors.comment), Print(&comment), ResetColor)?;
                return Ok(());
            }
            if self.language == Language::Php && ch == '#' {
                let mut comment = String::from("#");
                for n in chars {
                    comment.push(n);
                }
                queue!(out, SetForegroundColor(self.colors.comment), Print(&comment), ResetColor)?;
                return Ok(());
            }
            if comment_start == "--" && ch == '-' && chars.peek() == Some(&'-') {
                chars.next();
                let mut comment = String::from("--");
                for n in chars {
                    comment.push(n);
                }
                queue!(out, SetForegroundColor(self.colors.comment), Print(&comment), ResetColor)?;
                return Ok(());
            }
            if comment_start == "//" && ch == '/' && chars.peek() == Some(&'*') {
                chars.next();
                let mut comment = String::from("/*");
                let is_doc = chars.peek() == Some(&'*');
                if is_doc {
                    comment.push('*');
                    chars.next();
                }
                let mut prev = '\0';
                let mut closed = false;
                while let Some(n) = chars.next() {
                    comment.push(n);
                    if prev == '*' && n == '/' {
                        closed = true;
                        break;
                    }
                    prev = n;
                }
                let color = if is_doc { self.color_doc_comment() } else { self.colors.comment };
                queue!(out, SetForegroundColor(color), Print(&comment), ResetColor)?;
                count += comment.len();
                if !closed {
                    *raw_state = Some(MultiLineState::Comment("*/".to_string()));
                    return Ok(());
                }
                continue;
            }
            if comment_start == "//" && ch == '/' && chars.peek() == Some(&'/') {
                chars.next();
                let mut comment = String::from("//");
                let is_doc = chars.peek() == Some(&'/');
                if is_doc {
                    comment.push('/');
                    chars.next();
                }
                for n in chars {
                    comment.push(n);
                }
                let color = if is_doc { self.color_doc_comment() } else { self.colors.comment };
                queue!(out, SetForegroundColor(color), Print(&comment), ResetColor)?;
                return Ok(());
            }

            if matches!(self.language, Language::C | Language::Cpp) && ch == 'R' && chars.peek() == Some(&'"') {
                let mut literal = String::from("R");
                chars.next();
                literal.push('"');
                let mut delim = String::new();
                while let Some(n) = chars.peek() {
                    if *n == '(' {
                        break;
                    }
                    if delim.len() > 16 {
                        break;
                    }
                    delim.push(*n);
                    chars.next();
                }
                if chars.peek() == Some(&'(') {
                    chars.next();
                    literal.push_str(&delim);
                    literal.push('(');
                    let mut tail = String::from(")");
                    tail.push_str(&delim);
                    tail.push('"');
                    let mut found_tail = false;
                    let mut window = String::new();
                    while let Some(n) = chars.next() {
                        literal.push(n);
                        window.push(n);
                        if window.len() > tail.len() {
                            window.remove(0);
                        }
                        if window == tail {
                            found_tail = true;
                            break;
                        }
                    }
                    count += literal.len();
                    queue!(out, SetForegroundColor(self.colors.string), Print(&literal), ResetColor)?;
                    if !found_tail {
                        *raw_state = Some(MultiLineState::String(tail));
                    }
                    continue;
                }
                count += literal.len();
                queue!(out, SetForegroundColor(self.colors.fg), Print(&literal), ResetColor)?;
                continue;
            }
            if ch.is_alphanumeric() || ch == '_' {
                let mut token = String::new();
                token.push(ch);
                while let Some(n) = chars.peek() {
                    if n.is_alphanumeric() || *n == '_' {
                        token.push(*n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                count += token.len();
                if is_solidity_mapping_token(self.language, &token) {
                    queue!(out, SetForegroundColor(self.color_type()), Print(token), ResetColor)?;
                } else if is_type_name(self.language, &token) {
                    queue!(out, SetForegroundColor(self.color_type()), Print(token), ResetColor)?;
                } else if is_builtin(self.language, &token) {
                    queue!(out, SetForegroundColor(self.color_builtin()), Print(token), ResetColor)?;
                } else if is_keyword(self.language, &token) {
                    queue!(out, SetForegroundColor(self.color_keyword()), Print(token), ResetColor)?;
                } else if is_function_token(&chars, &token) {
                    queue!(out, SetForegroundColor(self.color_function()), Print(token), ResetColor)?;
                } else {
                    queue!(out, SetForegroundColor(self.colors.fg), Print(token), ResetColor)?;
                }
                continue;
            }
            if ch.is_ascii_digit() {
                let mut number = String::new();
                number.push(ch);
                while let Some(n) = chars.peek() {
                    if n.is_ascii_digit() || *n == '.' {
                        number.push(*n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                count += number.len();
                queue!(out, SetForegroundColor(self.colors.number), Print(number), ResetColor)?;
                continue;
            }
            if "+-*/=%<>!&|".contains(ch) {
                count += 1;
                queue!(out, SetForegroundColor(self.colors.operator), Print(ch), ResetColor)?;
                continue;
            }

            count += 1;
            queue!(out, SetForegroundColor(self.colors.fg), Print(ch), ResetColor)?;
        }
        Ok(())
    }

    fn can_edit(&mut self) -> bool {
        if self.read_only {
            self.status = "Read-only: edit disabled.".to_string();
            return false;
        }
        if self.safe_mode && self.language == Language::Steelconf && !self.header_valid() {
            self.status = "Safe mode: add !muf 4 header to edit.".to_string();
            return false;
        }
        true
    }

    fn header_valid(&self) -> bool {
        for line in &self.lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with(";;") {
                continue;
            }
            return trimmed == "!muf 4";
        }
        false
    }

    fn visual_row_from(&self, start_line: usize, text_cols: usize) -> usize {
        if !self.soft_wrap || text_cols == 0 {
            return self.cursor_y.saturating_sub(start_line);
        }
        let mut row = 0usize;
        for (idx, line) in self.lines.iter().enumerate().skip(start_line) {
            if idx == self.cursor_y {
                row += self.cursor_x.min(line.len()) / text_cols;
                break;
            }
            row += wrapped_rows(line, text_cols);
        }
        row
    }

    fn jump_run_error(&mut self) -> io::Result<()> {
        if self.run_errors.is_empty() {
            self.status = "No run errors.".to_string();
            return Ok(());
        }
        let list = self
            .run_errors
            .iter()
            .map(|err| format!("{}:{}: {}", err.path.display(), err.line, err.message))
            .collect::<Vec<_>>();
        let choice = self.pick_from_list("Jump error", &list)?;
        if let Some(item) = choice {
            if let Some((path, rest)) = item.split_once(':') {
                if let Some((line_str, _)) = rest.trim().split_once(':') {
                    if let Ok(line_no) = line_str.trim().parse::<usize>() {
                        self.open_file(PathBuf::from(path))?;
                        let target = line_no.saturating_sub(1);
                        if target < self.lines.len() {
                            self.cursor_y = target;
                            self.cursor_x = 0;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn color_keyword(&self) -> Color {
        match self.language {
            Language::C => palette_color(self.palette_c, self.theme, ColorRole::Keyword),
            Language::Cpp => palette_color(self.palette_cpp, self.theme, ColorRole::Keyword),
            Language::Python => palette_color(self.palette_py, self.theme, ColorRole::Keyword),
            _ => self.colors.keyword,
        }
    }

    fn color_type(&self) -> Color {
        match self.language {
            Language::C => palette_color(self.palette_c, self.theme, ColorRole::TypeName),
            Language::Cpp => palette_color(self.palette_cpp, self.theme, ColorRole::TypeName),
            _ => self.colors.type_name,
        }
    }

    fn color_builtin(&self) -> Color {
        match self.language {
            Language::C => palette_color(self.palette_c, self.theme, ColorRole::Builtin),
            Language::Cpp => palette_color(self.palette_cpp, self.theme, ColorRole::Builtin),
            Language::Python => palette_color(self.palette_py, self.theme, ColorRole::Builtin),
            _ => self.colors.builtin,
        }
    }

    fn color_function(&self) -> Color {
        match self.language {
            Language::C => palette_color(self.palette_c, self.theme, ColorRole::Function),
            Language::Cpp => palette_color(self.palette_cpp, self.theme, ColorRole::Function),
            Language::Python => palette_color(self.palette_py, self.theme, ColorRole::Function),
            _ => self.colors.function,
        }
    }

    fn color_doc_comment(&self) -> Color {
        match self.language {
            Language::Python => palette_color(self.palette_py, self.theme, ColorRole::Docstring),
            _ => self.colors.doc_comment,
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
    }
}

fn collect_symbols_for(language: Language, lines: &[String]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if is_probably_non_code_symbol_line(language, trimmed) {
            continue;
        }
        match language {
            Language::Python => {
                if let Some(name) = trimmed.strip_prefix("def ").and_then(extract_ident) {
                    out.push((idx, format!("def {name}")));
                } else if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                    out.push((idx, format!("class {name}")));
                }
            }
            Language::Java => {
                if trimmed.ends_with('{') && trimmed.contains('(') && trimmed.contains(')') {
                    if let Some(name) = extract_func_name(trimmed) {
                        out.push((idx, name));
                    }
                } else if trimmed.starts_with("class ") {
                    if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                        out.push((idx, format!("class {name}")));
                    }
                }
            }
            Language::C | Language::Cpp => {
                if trimmed.ends_with('{') && trimmed.contains('(') && trimmed.contains(')') {
                    if let Some(name) = extract_func_name(trimmed) {
                        out.push((idx, name));
                    }
                } else if trimmed.starts_with("struct ") {
                    if let Some(name) = trimmed.strip_prefix("struct ").and_then(extract_ident) {
                        out.push((idx, format!("struct {name}")));
                    }
                } else if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                    out.push((idx, format!("class {name}")));
                } else if let Some(name) = trimmed.strip_prefix("namespace ").and_then(extract_ident) {
                    out.push((idx, format!("namespace {name}")));
                } else if let Some(name) = trimmed.strip_prefix("enum ").and_then(extract_ident) {
                    out.push((idx, format!("enum {name}")));
                } else if let Some(name) = trimmed.strip_prefix("typedef ").and_then(extract_ident) {
                    out.push((idx, format!("typedef {name}")));
                }
            }
            Language::Go => {
                if let Some(name) = trimmed.strip_prefix("func ").and_then(extract_ident) {
                    out.push((idx, format!("func {name}")));
                } else if let Some(name) = trimmed.strip_prefix("type ").and_then(extract_ident) {
                    out.push((idx, format!("type {name}")));
                }
            }
            Language::JavaScript | Language::TypeScript | Language::CoffeeScript => {
                if let Some(name) = trimmed.strip_prefix("function ").and_then(extract_ident) {
                    out.push((idx, format!("function {name}")));
                } else if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                    out.push((idx, format!("class {name}")));
                } else if trimmed.ends_with("=> {") && trimmed.contains('=') {
                    if let Some((head, _)) = trimmed.split_once('=') {
                        let name = head.trim();
                        if !name.is_empty() {
                            out.push((idx, format!("lambda {name}")));
                        }
                    }
                }
            }
            Language::Rust => {
                if let Some(name) = extract_rust_fn_symbol(trimmed) {
                    out.push((idx, format!("fn {name}")));
                } else if let Some(name) = trimmed.strip_prefix("struct ").and_then(extract_ident) {
                    out.push((idx, format!("struct {name}")));
                } else if let Some(name) = trimmed.strip_prefix("enum ").and_then(extract_ident) {
                    out.push((idx, format!("enum {name}")));
                } else if let Some(name) = trimmed.strip_prefix("trait ").and_then(extract_ident) {
                    out.push((idx, format!("trait {name}")));
                } else if let Some(name) = trimmed.strip_prefix("impl ").and_then(extract_ident) {
                    out.push((idx, format!("impl {name}")));
                } else if let Some(name) = extract_rust_macro_symbol(trimmed) {
                    out.push((idx, format!("macro {name}")));
                }
            }
            Language::Ruby => {
                if let Some(name) = trimmed.strip_prefix("def ").and_then(extract_ident) {
                    out.push((idx, format!("def {name}")));
                } else if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                    out.push((idx, format!("class {name}")));
                } else if let Some(name) = trimmed.strip_prefix("module ").and_then(extract_ident) {
                    out.push((idx, format!("module {name}")));
                }
            }
            Language::Perl => {
                if let Some(name) = trimmed.strip_prefix("sub ").and_then(extract_ident) {
                    out.push((idx, format!("sub {name}")));
                }
            }
            Language::Php => {
                if let Some(name) = trimmed.strip_prefix("function ").and_then(extract_ident) {
                    out.push((idx, format!("function {name}")));
                } else if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                    out.push((idx, format!("class {name}")));
                }
            }
            Language::Lua => {
                if let Some(name) = trimmed.strip_prefix("function ").and_then(extract_ident) {
                    out.push((idx, format!("function {name}")));
                } else if let Some(name) = trimmed.strip_prefix("local function ").and_then(extract_ident) {
                    out.push((idx, format!("function {name}")));
                }
            }
            Language::Haskell => {
                if let Some((name, _)) = trimmed.split_once("::") {
                    let name = name.trim();
                    if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        out.push((idx, format!("sig {name}")));
                    }
                } else if let Some((name, _)) = trimmed.split_once('=') {
                    let name = name.trim();
                    if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        out.push((idx, format!("def {name}")));
                    }
                }
            }
            Language::Pascal | Language::Algol => {
                if let Some(name) = trimmed.strip_prefix("program ").and_then(extract_ident) {
                    out.push((idx, format!("program {name}")));
                } else if let Some(name) = trimmed.strip_prefix("procedure ").and_then(extract_ident) {
                    out.push((idx, format!("procedure {name}")));
                } else if let Some(name) = trimmed.strip_prefix("function ").and_then(extract_ident) {
                    out.push((idx, format!("function {name}")));
                }
            }
            Language::Shell => {
                if let Some(name) = extract_shell_function_symbol(trimmed) {
                    out.push((idx, format!("function {name}")));
                }
            }
            Language::HolyC => {
                if trimmed.ends_with('{') && trimmed.contains('(') && trimmed.contains(')') {
                    if let Some(name) = extract_func_name(trimmed) {
                        out.push((idx, name));
                    }
                }
            }
            Language::Wgsl => {
                if let Some(name) = trimmed.strip_prefix("fn ").and_then(extract_ident) {
                    out.push((idx, format!("fn {name}")));
                } else if let Some(name) = trimmed.strip_prefix("struct ").and_then(extract_ident) {
                    out.push((idx, format!("struct {name}")));
                }
            }
            Language::OpenClC => {
                if trimmed.contains("__kernel")
                    && trimmed.contains('(')
                    && trimmed.contains(')')
                {
                    if let Some(name) = extract_func_name(trimmed) {
                        out.push((idx, format!("kernel {name}")));
                    }
                } else if trimmed.ends_with('{') && trimmed.contains('(') && trimmed.contains(')') {
                    if let Some(name) = extract_func_name(trimmed) {
                        out.push((idx, name));
                    }
                }
            }
            Language::Hack => {
                if let Some(name) = trimmed.strip_prefix("function ").and_then(extract_ident) {
                    out.push((idx, format!("function {name}")));
                } else if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                    out.push((idx, format!("class {name}")));
                }
            }
            Language::Apex => {
                if let Some(name) = trimmed.strip_prefix("trigger ").and_then(extract_ident) {
                    out.push((idx, format!("trigger {name}")));
                } else if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                    out.push((idx, format!("class {name}")));
                } else if trimmed.ends_with('{') && trimmed.contains('(') && trimmed.contains(')') {
                    if let Some(name) = extract_func_name(trimmed) {
                        out.push((idx, name));
                    }
                }
            }
            Language::Kotlin => {
                if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                    out.push((idx, format!("class {name}")));
                } else if let Some(name) = trimmed.strip_prefix("fun ").and_then(extract_ident) {
                    out.push((idx, format!("fun {name}")));
                }
            }
            Language::Swift => {
                if let Some(name) = trimmed.strip_prefix("struct ").and_then(extract_ident) {
                    out.push((idx, format!("struct {name}")));
                } else if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                    out.push((idx, format!("class {name}")));
                } else if let Some(name) = trimmed.strip_prefix("func ").and_then(extract_ident) {
                    out.push((idx, format!("func {name}")));
                }
            }
            Language::Dart => {
                if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                    out.push((idx, format!("class {name}")));
                } else if let Some(name) = trimmed.strip_prefix("extension ").and_then(extract_ident) {
                    out.push((idx, format!("extension {name}")));
                } else if let Some(name) = trimmed.strip_prefix("mixin ").and_then(extract_ident) {
                    out.push((idx, format!("mixin {name}")));
                } else if let Some(name) = trimmed.strip_prefix("void ").and_then(extract_ident) {
                    out.push((idx, format!("func {name}")));
                } else if trimmed.ends_with('{') && trimmed.contains('(') && trimmed.contains(')') {
                    if let Some(name) = extract_func_name(trimmed) {
                        out.push((idx, format!("func {name}")));
                    }
                }
            }
            Language::Solidity => {
                if let Some(name) = trimmed.strip_prefix("contract ").and_then(extract_ident) {
                    out.push((idx, format!("contract {name}")));
                } else if let Some(name) = trimmed.strip_prefix("event ").and_then(extract_ident) {
                    out.push((idx, format!("event {name}")));
                } else if let Some(name) = trimmed.strip_prefix("function ").and_then(extract_ident) {
                    out.push((idx, format!("function {name}")));
                }
            }
            Language::PowerShell => {
                if let Some(name) = trimmed.strip_prefix("function ").and_then(extract_ident) {
                    out.push((idx, format!("function {name}")));
                } else if let Some(name) = trimmed.strip_prefix("class ").and_then(extract_ident) {
                    out.push((idx, format!("class {name}")));
                } else if let Some(name) = trimmed.strip_prefix("enum ").and_then(extract_ident) {
                    out.push((idx, format!("enum {name}")));
                }
            }
            Language::Makefile => {
                if !trimmed.starts_with('\t') && !trimmed.starts_with('#') {
                    if let Some((head, rest)) = trimmed.split_once(':') {
                        if rest.starts_with('=') {
                            continue;
                        }
                        let target = head.trim();
                        if !target.is_empty() && !target.contains('=') && !target.contains('$') {
                            out.push((idx, format!("target {target}")));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn is_probably_non_code_symbol_line(language: Language, trimmed: &str) -> bool {
    if trimmed.is_empty() {
        return true;
    }
    if matches!(trimmed.chars().next(), Some('"') | Some('\'')) {
        return true;
    }
    match language {
        Language::Rust
        | Language::C
        | Language::Cpp
        | Language::Java
        | Language::JavaScript
        | Language::TypeScript
        | Language::Go
        | Language::HolyC
        | Language::Php
        | Language::Wgsl
        | Language::OpenClC
        | Language::Hack
        | Language::Apex
        | Language::Kotlin
        | Language::Swift
        | Language::Dart
        | Language::Solidity => {
            trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
        }
        Language::Shell
        | Language::Perl
        | Language::Ruby
        | Language::Python
        | Language::CoffeeScript
        | Language::PowerShell
        | Language::Makefile => trimmed.starts_with('#'),
        Language::Lua | Language::Haskell => trimmed.starts_with("--"),
        Language::Pascal | Language::Algol => {
            trimmed.starts_with('{') || trimmed.starts_with("(*") || trimmed.starts_with("//")
        }
        _ => false,
    }
}

fn is_block_keyword(line: &str) -> bool {
    let keywords = ["workspace", "profile", "tool", "bake", "run", "export"];
    let trimmed = line.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = trimmed.trim_matches(&['[', ']'][..]);
        let head = inner.split_whitespace().next().unwrap_or("");
        return keywords.contains(&head);
    }
    false
}

fn detect_language(file: &PathBuf) -> Language {
    if let Some(name) = file.file_name().and_then(|n| n.to_str()) {
        if name == "steelconf" || name.ends_with(".muf") {
            return Language::Steelconf;
        }
        if matches!(name, "Makefile" | "makefile" | "GNUmakefile") {
            return Language::Makefile;
        }
        if matches!(name, ".bashrc" | ".bash_profile" | ".zshrc" | ".zprofile") {
            return Language::Shell;
        }
    }
    match file.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "c" | "h" => Language::C,
        "cc" | "cpp" | "cxx" | "hpp" | "hh" => Language::Cpp,
        "py" => Language::Python,
        "kt" | "kts" => Language::Kotlin,
        "swift" => Language::Swift,
        "dart" => Language::Dart,
        "ex" | "exs" => Language::Elixir,
        "erl" | "hrl" => Language::Erlang,
        "clj" | "cljs" | "cljc" | "edn" => Language::Clojure,
        "fs" | "fsi" | "fsx" => Language::FSharp,
        "r" | "R" => Language::RLang,
        "jl" => Language::Julia,
        "m" | "oct" => Language::MatlabOctave,
        "scala" | "sc" => Language::Scala,
        "groovy" | "gvy" | "gradle" => Language::Groovy,
        "nim" | "nims" => Language::Nim,
        "cr" => Language::Crystal,
        "f90" | "f95" | "f03" | "f08" | "for" | "f" | "ftn" => Language::Fortran,
        "cob" | "cbl" | "cpy" => Language::Cobol,
        "adb" | "ads" => Language::Ada,
        "s" | "asm" => Language::Assembly,
        "vlang" => Language::VLang,
        "sol" => Language::Solidity,
        "move" => Language::Move,
        "vhd" | "vhdl" => Language::Vhdl,
        "v" | "sv" | "svh" | "vh" => Language::Verilog,
        "pro" | "prolog" => Language::Prolog,
        "scm" | "ss" | "rkt" => Language::Scheme,
        "st" => Language::Smalltalk,
        "tcl" => Language::Tcl,
        "ps1" | "psm1" | "psd1" => Language::PowerShell,
        "fish" => Language::Fish,
        "mk" => Language::Makefile,
        "wgsl" => Language::Wgsl,
        "cl" | "opencl" => Language::OpenClC,
        "hack" | "hhi" => Language::Hack,
        "apex" | "cls" | "trigger" => Language::Apex,
        "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
        "ts" | "mts" | "cts" | "tsx" => Language::TypeScript,
        "go" => Language::Go,
        "rs" => Language::Rust,
        "php" | "phtml" | "php5" | "php7" | "phps" => Language::Php,
        "lua" => Language::Lua,
        "sh" | "bash" | "zsh" => Language::Shell,
        "pl" | "pm" | "t" => Language::Perl,
        "rb" | "rake" | "gemspec" => Language::Ruby,
        "hs" | "lhs" => Language::Haskell,
        "coffee" | "cson" => Language::CoffeeScript,
        "pas" | "pp" | "lpr" => Language::Pascal,
        "alg" | "algol" | "a68" => Language::Algol,
        "hc" | "holyc" => Language::HolyC,
        "java" => Language::Java,
        "ml" | "mli" => Language::Ocaml,
        "zig" => Language::Zig,
        "cs" => Language::CSharp,
        _ => Language::Other,
    }
}

fn language_label_for(language: Language) -> &'static str {
    match language {
        Language::Steelconf => "steelconf",
        Language::C => "C",
        Language::Cpp => "C++",
        Language::Python => "Python",
        Language::Kotlin => "Kotlin",
        Language::Swift => "Swift",
        Language::Dart => "Dart",
        Language::Elixir => "Elixir",
        Language::Erlang => "Erlang",
        Language::Clojure => "Clojure",
        Language::FSharp => "F#",
        Language::RLang => "R",
        Language::Julia => "Julia",
        Language::MatlabOctave => "MATLAB/Octave",
        Language::Scala => "Scala",
        Language::Groovy => "Groovy",
        Language::Nim => "Nim",
        Language::Crystal => "Crystal",
        Language::Fortran => "Fortran",
        Language::Cobol => "COBOL",
        Language::Ada => "Ada",
        Language::Assembly => "Assembly",
        Language::VLang => "V",
        Language::Solidity => "Solidity",
        Language::Move => "Move",
        Language::Vhdl => "VHDL",
        Language::Verilog => "Verilog/SystemVerilog",
        Language::Prolog => "Prolog",
        Language::Scheme => "Scheme",
        Language::Smalltalk => "Smalltalk",
        Language::Tcl => "Tcl",
        Language::PowerShell => "PowerShell",
        Language::Fish => "Fish",
        Language::Makefile => "Makefile",
        Language::Wgsl => "WGSL",
        Language::OpenClC => "OpenCL C",
        Language::Hack => "Hack",
        Language::Apex => "Apex",
        Language::JavaScript => "JavaScript",
        Language::TypeScript => "TypeScript",
        Language::Go => "Go",
        Language::Rust => "Rust",
        Language::Php => "PHP",
        Language::Lua => "Lua",
        Language::Perl => "Perl",
        Language::Ruby => "Ruby",
        Language::Haskell => "Haskell",
        Language::Shell => "sh/zsh",
        Language::CoffeeScript => "CoffeeScript",
        Language::Pascal => "Pascal",
        Language::Algol => "Algol",
        Language::HolyC => "HolyC",
        Language::Java => "Java",
        Language::Ocaml => "OCaml",
        Language::Zig => "Zig",
        Language::CSharp => "C#",
        Language::Other => "text",
    }
}

fn is_keyword(lang: Language, token: &str) -> bool {
    language_keywords(lang).contains(&token)
}

fn language_keywords(lang: Language) -> &'static [&'static str] {
    language_data::keywords(lang)
}

fn language_builtins(lang: Language, shell_dialect: ShellDialect) -> &'static [&'static str] {
    language_data::builtins(lang, shell_dialect)
}

fn language_snippets(lang: Language) -> &'static [LangSnippet] {
    language_data::snippets(lang)
}

fn canonical_trigger<'a>(language: Language, trigger: &'a str) -> &'a str {
    language_data::canonical_trigger(language, trigger).unwrap_or(trigger)
}

fn shell_dialect_label(dialect: ShellDialect) -> &'static str {
    match dialect {
        ShellDialect::Posix => "posix",
        ShellDialect::Bash => "bash",
        ShellDialect::Zsh => "zsh",
        ShellDialect::Union => "union",
    }
}

fn parse_shell_dialect(value: &str) -> ShellDialect {
    match value.to_ascii_lowercase().as_str() {
        "posix" => ShellDialect::Posix,
        "bash" => ShellDialect::Bash,
        "zsh" => ShellDialect::Zsh,
        _ => ShellDialect::Union,
    }
}

fn active_shell_dialect() -> ShellDialect {
    configured_shell_dialect().unwrap_or(ShellDialect::Union)
}

fn configured_shell_dialect() -> Option<ShellDialect> {
    let value = std::env::var("STEECLEDITOR_SHELL_DIALECT").unwrap_or_default();
    configured_shell_dialect_from_raw(&value)
}

fn configured_shell_dialect_from_raw(value: &str) -> Option<ShellDialect> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(parse_shell_dialect(trimmed))
    }
}

fn shell_dialect_status_segment() -> Option<String> {
    let value = std::env::var("STEECLEDITOR_SHELL_DIALECT").unwrap_or_default();
    shell_dialect_status_segment_from_raw(&value)
}

fn shell_dialect_status_segment_from_raw(value: &str) -> Option<String> {
    configured_shell_dialect_from_raw(value)
        .map(|dialect| format!(" | sh:{}", shell_dialect_label(dialect)))
}

fn language_completion_items_with_dialect(
    language: Language,
    prefix: &str,
    shell_dialect: ShellDialect,
) -> Vec<CompletionItem> {
    if matches!(language, Language::Steelconf | Language::Other) {
        return Vec::new();
    }
    if !is_identifier_prefix(prefix) {
        return Vec::new();
    }
    let mut items = Vec::new();
    for kw in language_keywords(language).iter().filter(|kw| kw.starts_with(prefix)) {
        items.push(CompletionItem {
            label: format!("keyword: {}", kw),
            insert: (*kw).to_string(),
            is_snippet: false,
        });
    }
    for built in language_builtins(language, shell_dialect)
        .iter()
        .filter(|built| built.starts_with(prefix))
    {
        items.push(CompletionItem {
            label: format!("builtin: {}", built),
            insert: (*built).to_string(),
            is_snippet: false,
        });
    }
    for snippet in language_snippets(language)
        .iter()
        .filter(|s| s.trigger.starts_with(prefix))
    {
        let canonical = canonical_trigger(language, snippet.trigger);
        let insert_body = language_snippets(language)
            .iter()
            .find(|s| s.trigger == canonical)
            .map(|s| s.body)
            .unwrap_or(snippet.body);
        items.push(CompletionItem {
            label: format!("snippet: {}", snippet.label),
            insert: insert_body.to_string(),
            is_snippet: true,
        });
    }
    items
}

fn language_completion_items(language: Language, prefix: &str) -> Vec<CompletionItem> {
    language_completion_items_with_dialect(language, prefix, active_shell_dialect())
}

fn format_completion_debug_lines(
    language: Language,
    file: &Path,
    prefix: &str,
    start: usize,
    items: &[CompletionItem],
    verbose: bool,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("debug language: {}", language_label_for(language)));
    lines.push(format!("debug file: {}", file.display()));
    lines.push(format!("debug prefix: '{prefix}' (start={start})"));
    lines.push(format!("debug completions: {}", items.len()));
    if verbose {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("<none>");
        let is_prefix_ident = is_identifier_prefix(prefix);
        let shell_dialect = active_shell_dialect();
        let kw = language_keywords(language)
            .iter()
            .filter(|k| k.starts_with(prefix))
            .count();
        let bi = language_builtins(language, shell_dialect)
            .iter()
            .filter(|b| b.starts_with(prefix))
            .count();
        let sn = language_snippets(language)
            .iter()
            .filter(|s| s.trigger.starts_with(prefix))
            .count();
        lines.push("debug mode: --verbose".to_string());
        lines.push(format!("debug extension: {ext}"));
        if language == Language::Shell {
            lines.push(format!("debug shell dialect: {}", shell_dialect_label(shell_dialect)));
        }
        lines.push(format!("debug prefix scanner: is_identifier_prefix={is_prefix_ident}"));
        lines.push(format!("debug filters: keyword={kw} builtin={bi} snippet={sn}"));
    }
    for item in items.iter().take(16) {
        lines.push(format!("  - {}", item.label));
    }
    lines
}

fn parse_heredoc_marker(shell_mode: bool, rest_after_first_lt: &str) -> Option<String> {
    let mut chars = rest_after_first_lt.chars().peekable();
    if chars.next()? != '<' {
        return None;
    }
    if chars.peek() == Some(&'<') {
        chars.next();
    }
    if shell_mode && chars.peek() == Some(&'-') {
        chars.next();
    }
    while chars.peek().is_some_and(|c| c.is_whitespace()) {
        chars.next();
    }
    let mut marker = String::new();
    if matches!(chars.peek(), Some('\'') | Some('"')) {
        let quote = chars.next().unwrap();
        while let Some(c) = chars.peek() {
            if *c == quote {
                break;
            }
            marker.push(*c);
            chars.next();
        }
    } else {
        while let Some(c) = chars.peek() {
            if c.is_alphanumeric() || *c == '_' || *c == '-' {
                marker.push(*c);
                chars.next();
            } else {
                break;
            }
        }
    }
    if marker.is_empty() {
        None
    } else {
        Some(marker)
    }
}

fn is_powershell_here_string_start(ch: char, next: Option<char>) -> Option<char> {
    if ch == '@' {
        if next == Some('"') {
            return Some('"');
        }
        if next == Some('\'') {
            return Some('\'');
        }
    }
    None
}

fn is_powershell_here_string_terminator(line: &str, quote: char) -> bool {
    match quote {
        '"' => is_line_terminator_marker(line, "\"@"),
        '\'' => is_line_terminator_marker(line, "'@"),
        _ => false,
    }
}

fn is_line_terminator_marker(line: &str, marker: &str) -> bool {
    let trimmed = line.trim();
    if trimmed == marker {
        return true;
    }
    let tab_trimmed = line.trim_start_matches('\t').trim();
    tab_trimmed == marker
}

fn is_makefile_recipe_line(line: &str) -> bool {
    line.starts_with('\t')
}

fn is_makefile_var_start(ch: char, next: Option<char>) -> bool {
    ch == '$' && next == Some('(')
}

fn is_solidity_mapping_token(language: Language, token: &str) -> bool {
    language == Language::Solidity && token == "mapping"
}

fn coffee_block_comment_closed(s: &str) -> bool {
    s.contains("###")
}

fn pascal_nested_depth_after(input: &str, initial_depth: usize) -> usize {
    let mut depth = initial_depth;
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '(' && chars.peek() == Some(&'*') {
            chars.next();
            depth += 1;
        } else if ch == '*' && chars.peek() == Some(&')') {
            chars.next();
            depth = depth.saturating_sub(1);
        }
    }
    depth
}

fn is_type_name(lang: Language, token: &str) -> bool {
    language_data::types(lang).contains(&token)
}

fn is_builtin(lang: Language, token: &str) -> bool {
    match lang {
        Language::Php => language_builtins(lang, ShellDialect::Union).contains(&token) || token.starts_with('$'),
        Language::Ruby => language_builtins(lang, ShellDialect::Union).contains(&token) || token.starts_with('$') || token.starts_with('@'),
        Language::Perl if token.starts_with('$') || token.starts_with('@') || token.starts_with('%') => true,
        _ => language_builtins(lang, ShellDialect::Union).contains(&token),
    }
}

fn is_function_token(chars: &std::iter::Peekable<std::str::Chars<'_>>, token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let mut probe = chars.clone();
    while let Some(ch) = probe.peek() {
        if ch.is_whitespace() {
            probe.next();
        } else {
            break;
        }
    }
    matches!(probe.peek(), Some('('))
}

fn validate_python_prefix(prefix: &str) -> (bool, bool) {
    let mut has_f = false;
    let mut has_r = false;
    let mut has_b = false;
    let mut has_u = false;
    for ch in prefix.chars() {
        match ch {
            'f' | 'F' => {
                if has_f {
                    return (false, false);
                }
                has_f = true;
            }
            'r' | 'R' => {
                if has_r {
                    return (false, false);
                }
                has_r = true;
            }
            'b' | 'B' => {
                if has_b {
                    return (false, false);
                }
                has_b = true;
            }
            'u' | 'U' => {
                if has_u {
                    return (false, false);
                }
                has_u = true;
            }
            _ => return (false, false),
        }
    }
    (true, has_f)
}

enum ColorRole {
    Keyword,
    TypeName,
    Builtin,
    Function,
    Docstring,
}

fn palette_color(palette: Palette, theme: Theme, role: ColorRole) -> Color {
    match (palette, theme, role) {
        (Palette::Default, Theme::Dark, ColorRole::Keyword) => Color::Magenta,
        (Palette::Default, Theme::Dark, ColorRole::TypeName) => Color::Cyan,
        (Palette::Default, Theme::Dark, ColorRole::Builtin) => Color::Green,
        (Palette::Default, Theme::Dark, ColorRole::Function) => Color::Yellow,
        (Palette::Default, Theme::Dark, ColorRole::Docstring) => Color::DarkYellow,
        (Palette::Default, Theme::Light, ColorRole::Keyword) => Color::DarkRed,
        (Palette::Default, Theme::Light, ColorRole::TypeName) => Color::Blue,
        (Palette::Default, Theme::Light, ColorRole::Builtin) => Color::DarkGreen,
        (Palette::Default, Theme::Light, ColorRole::Function) => Color::DarkMagenta,
        (Palette::Default, Theme::Light, ColorRole::Docstring) => Color::DarkCyan,
        (Palette::Vivid, Theme::Dark, ColorRole::Keyword) => Color::Red,
        (Palette::Vivid, Theme::Dark, ColorRole::TypeName) => Color::Blue,
        (Palette::Vivid, Theme::Dark, ColorRole::Builtin) => Color::Cyan,
        (Palette::Vivid, Theme::Dark, ColorRole::Function) => Color::Yellow,
        (Palette::Vivid, Theme::Dark, ColorRole::Docstring) => Color::Magenta,
        (Palette::Vivid, Theme::Light, ColorRole::Keyword) => Color::DarkRed,
        (Palette::Vivid, Theme::Light, ColorRole::TypeName) => Color::DarkBlue,
        (Palette::Vivid, Theme::Light, ColorRole::Builtin) => Color::DarkCyan,
        (Palette::Vivid, Theme::Light, ColorRole::Function) => Color::DarkYellow,
        (Palette::Vivid, Theme::Light, ColorRole::Docstring) => Color::DarkMagenta,
        (Palette::Soft, Theme::Dark, ColorRole::Keyword) => Color::DarkGrey,
        (Palette::Soft, Theme::Dark, ColorRole::TypeName) => Color::DarkCyan,
        (Palette::Soft, Theme::Dark, ColorRole::Builtin) => Color::Green,
        (Palette::Soft, Theme::Dark, ColorRole::Function) => Color::DarkYellow,
        (Palette::Soft, Theme::Dark, ColorRole::Docstring) => Color::DarkGrey,
        (Palette::Soft, Theme::Light, ColorRole::Keyword) => Color::DarkGrey,
        (Palette::Soft, Theme::Light, ColorRole::TypeName) => Color::DarkBlue,
        (Palette::Soft, Theme::Light, ColorRole::Builtin) => Color::DarkGreen,
        (Palette::Soft, Theme::Light, ColorRole::Function) => Color::DarkMagenta,
        (Palette::Soft, Theme::Light, ColorRole::Docstring) => Color::DarkGrey,
    }
}

fn is_identifier_prefix(prefix: &str) -> bool {
    let mut chars = prefix.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '@' || first == '$' || first == '%') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '@' || c == '$' || c == '%')
}

fn contains_todo(line: &str) -> bool {
    line.contains("TODO") || line.contains("FIXME") || line.contains("NOTE")
}

fn wrapped_rows(line: &str, cols: usize) -> usize {
    if cols == 0 {
        return 1;
    }
    let len = line.len().max(1);
    (len + cols - 1) / cols
}

fn detect_line_ending(bytes: &[u8]) -> LineEnding {
    if bytes.windows(2).any(|w| w == b"\r\n") {
        LineEnding::CrLf
    } else {
        LineEnding::Lf
    }
}

fn detect_encoding(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        "UTF-8-BOM".to_string()
    } else {
        "UTF-8".to_string()
    }
}

fn line_ending_label(ending: LineEnding) -> &'static str {
    match ending {
        LineEnding::Lf => "LF",
        LineEnding::CrLf => "CRLF",
    }
}

fn parse_run_errors(lines: &[String]) -> Vec<RunError> {
    let mut out = Vec::new();
    for line in lines {
        if let Some(err) = parse_error_line(line) {
            if err.path.exists() {
                out.push(err);
            }
        }
    }
    out
}

fn parse_error_line(line: &str) -> Option<RunError> {
    let mut parts = line.splitn(3, ':');
    let path_part = parts.next()?.trim();
    let line_part = parts.next()?.trim();
    let rest = parts.next().unwrap_or("").trim();
    let line_no = line_part.parse::<usize>().ok()?;
    let message = rest.trim().trim_start_matches(':').trim();
    Some(RunError {
        path: PathBuf::from(path_part),
        line: line_no,
        message: message.to_string(),
    })
}

fn snippet_for_trigger(trigger: &str) -> Option<&'static str> {
    match trigger {
        "[workspace]" => find_snippet_body("workspace"),
        "[profile]" => find_snippet_body("profile"),
        "[tool]" => find_snippet_body("tool"),
        "[bake]" => find_snippet_body("bake"),
        "[run]" => find_snippet_body("run"),
        "[export]" => find_snippet_body("export"),
        ".set" => find_snippet_body(".set"),
        ".make" => find_snippet_body(".make"),
        ".takes" => find_snippet_body(".takes"),
        ".emits" => find_snippet_body(".emits"),
        ".output" => find_snippet_body(".output"),
        ".exec" => find_snippet_body(".exec"),
        ".ref" => find_snippet_body(".ref"),
        _ => None,
    }
}

fn find_snippet_body(trigger: &str) -> Option<&'static str> {
    STEELCONF_SNIPPETS
        .iter()
        .find(|snippet| snippet.trigger == trigger)
        .map(|snippet| snippet.body)
}

fn expand_snippet_placeholders(snippet: &str) -> String {
    let mut out = String::new();
    let mut chars = snippet.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next();
            let mut num = String::new();
            while let Some(n) = chars.peek() {
                if n.is_ascii_digit() {
                    num.push(*n);
                    chars.next();
                } else {
                    break;
                }
            }
            if chars.peek() == Some(&':') {
                chars.next();
                let mut value = String::new();
                while let Some(n) = chars.next() {
                    if n == '}' {
                        break;
                    }
                    value.push(n);
                }
                out.push_str(&value);
            } else {
                out.push('$');
                out.push('{');
                out.push_str(&num);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn collect_glob_preview(root: &PathBuf, lines: &[String]) -> Vec<String> {
    let patterns = extract_cglob_patterns(lines);
    if patterns.is_empty() {
        return Vec::new();
    }
    let files = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.path().strip_prefix(root).ok().map(|p| p.to_string_lossy().replace('\\', "/")))
        .collect::<Vec<_>>();
    let mut out = Vec::new();
    for pattern in patterns.into_iter().take(5) {
        let Some(regex) = glob_to_regex(&pattern) else {
            out.push(format!("cglob \"{pattern}\" (invalid)"));
            continue;
        };
        let mut matches = Vec::new();
        for path in &files {
            if regex.is_match(path) {
                matches.push(path.clone());
                if matches.len() >= 10 {
                    break;
                }
            }
        }
        let count = files.iter().filter(|p| regex.is_match(p)).count();
        out.push(format!("cglob \"{pattern}\" ({count})"));
        for m in matches {
            out.push(format!("  - {m}"));
        }
    }
    out
}

fn extract_cglob_patterns(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines {
        if let Some(idx) = line.find("cglob") {
            let after = &line[idx + 5..];
            let mut chars = after.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '"' {
                    let mut pattern = String::new();
                    while let Some(n) = chars.next() {
                        if n == '"' {
                            break;
                        }
                        pattern.push(n);
                    }
                    if !pattern.is_empty() {
                        out.push(pattern);
                    }
                    break;
                }
            }
        }
    }
    out
}

fn glob_to_regex(pattern: &str) -> Option<Regex> {
    let mut regex = String::from("^");
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    regex.push_str(".*");
                } else {
                    regex.push_str("[^/]*");
                }
            }
            '?' => regex.push_str("[^/]"),
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\' => {
                regex.push('\\');
                regex.push(ch);
            }
            _ => regex.push(ch),
        }
    }
    regex.push('$');
    Regex::new(&regex).ok()
}

fn config_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(dir);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config");
    }
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return PathBuf::from(appdata);
    }
    PathBuf::from(".")
}

fn resolve_steel_bin() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("STEEL_BIN") {
        let path = PathBuf::from(explicit);
        if path.exists() {
            return Some(path);
        }
    }
    for path in common_steel_paths() {
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn common_steel_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(&home).join(".local/bin/steel"));
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        paths.push(PathBuf::from(&profile).join(".local/bin/steel.exe"));
        paths.push(PathBuf::from(&profile).join("AppData/Local/Steel/steel.exe"));
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        paths.push(PathBuf::from(&local).join("Steel/steel.exe"));
    }
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        paths.push(PathBuf::from(&program_files).join("Steel/steel.exe"));
    }
    if let Ok(program_files) = std::env::var("ProgramFiles(x86)") {
        paths.push(PathBuf::from(&program_files).join("Steel/steel.exe"));
    } else {
        paths.push(PathBuf::from("C:/Program Files (x86)/Steel/steel.exe"));
    }
    paths.push(PathBuf::from("/usr/local/bin/steel"));
    paths.push(PathBuf::from("/opt/homebrew/bin/steel"));
    paths.push(PathBuf::from("/usr/bin/steel"));
    paths.push(PathBuf::from("/bin/steel"));
    paths
}

fn system_clipboard_set(text: &str) -> bool {
    if cfg!(target_os = "macos") {
        return Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(text.as_bytes());
                }
                child.wait()
            })
            .is_ok();
    }
    if cfg!(target_os = "windows") {
        return Command::new("powershell")
            .args(["-NoProfile", "-Command", "Set-Clipboard -Value ([Console]::In.ReadToEnd())"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(text.as_bytes());
                }
                child.wait()
            })
            .is_ok();
    }
    for cmd in [
        ("wl-copy", &[] as &[&str]),
        ("xclip", &["-selection", "clipboard"]),
        ("xsel", &["--clipboard", "--input"]),
    ] {
        let mut child = match Command::new(cmd.0)
            .args(cmd.1)
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => continue,
        };
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(text.as_bytes());
        }
        if child.wait().is_ok() {
            return true;
        }
    }
    false
}

fn system_clipboard_get() -> Option<String> {
    if cfg!(target_os = "macos") {
        if let Ok(out) = Command::new("pbpaste").output() {
            if out.status.success() {
                return Some(String::from_utf8_lossy(&out.stdout).to_string());
            }
        }
        return None;
    }
    if cfg!(target_os = "windows") {
        if let Ok(out) = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-Clipboard"])
            .output()
        {
            if out.status.success() {
                return Some(String::from_utf8_lossy(&out.stdout).to_string());
            }
        }
        return None;
    }
    for cmd in [
        ("wl-paste", &[] as &[&str]),
        ("xclip", &["-selection", "clipboard", "-o"]),
        ("xsel", &["--clipboard", "--output"]),
    ] {
        if let Ok(out) = Command::new(cmd.0).args(cmd.1).output() {
            if out.status.success() {
                return Some(String::from_utf8_lossy(&out.stdout).to_string());
            }
        }
    }
    None
}

fn load_session_paths() -> Vec<PathBuf> {
    let path = config_root().join("steel").join("steecleditor.session");
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(PathBuf::from)
        .collect()
}

fn find_workspace_root(file: &PathBuf) -> Option<PathBuf> {
    let mut dir = if file.is_dir() {
        file.clone()
    } else {
        file.parent()?.to_path_buf()
    };
    loop {
        if dir.join("steelconf").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn load_editor_config() -> EditorConfig {
    let mut config = EditorConfig {
        autosave_interval: None,
        theme: None,
        palette_c: None,
        palette_cpp: None,
        palette_py: None,
    };
    let path = config_root().join("steel").join("steecleditor.conf");
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return config,
    };
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = match trimmed.split_once('=') {
            Some(kv) => kv,
            None => continue,
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "autosave_interval" => {
                if let Ok(secs) = value.parse::<u64>() {
                    config.autosave_interval = Some(secs);
                }
            }
            "theme" => {
                let theme = match value {
                    "dark" => Some(Theme::Dark),
                    "light" => Some(Theme::Light),
                    _ => None,
                };
                if theme.is_some() {
                    config.theme = theme;
                }
            }
            "palette_c" => {
                if let Some(palette) = Palette::from_str(value) {
                    config.palette_c = Some(palette);
                }
            }
            "palette_cpp" => {
                if let Some(palette) = Palette::from_str(value) {
                    config.palette_cpp = Some(palette);
                }
            }
            "palette_py" => {
                if let Some(palette) = Palette::from_str(value) {
                    config.palette_py = Some(palette);
                }
            }
            _ => {}
        }
    }
    config
}

fn extract_ident(s: &str) -> Option<String> {
    let name = s
        .split(|c: char| c == '(' || c.is_whitespace() || c == '{' || c == ':')
        .next()
        .unwrap_or("")
        .trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn extract_shell_function_symbol(trimmed: &str) -> Option<String> {
    if let Some(name) = trimmed.strip_prefix("function ").and_then(extract_ident) {
        return Some(name);
    }
    if let Some((head, _)) = trimmed.split_once("()") {
        let name = head.trim();
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Some(name.to_string());
        }
    }
    if let Some((head, _)) = trimmed.split_once('{') {
        let name = head.trim();
        if !name.is_empty()
            && !name.contains(' ')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Some(name.to_string());
        }
    }
    None
}

fn extract_rust_fn_symbol(trimmed: &str) -> Option<String> {
    const PREFIXES: &[&str] = &["pub ", "async ", "const ", "unsafe ", "extern \"C\" ", "extern "];
    let mut rest = trimmed;
    let mut changed = true;
    while changed {
        changed = false;
        if let Some(t) = rest.strip_prefix("pub(").and_then(|r| r.split_once(')').map(|(_, tail)| tail.trim_start())) {
            rest = t;
            changed = true;
        }
        for prefix in PREFIXES {
            if let Some(t) = rest.strip_prefix(prefix) {
                rest = t;
                changed = true;
            }
        }
    }
    rest.strip_prefix("fn ").and_then(extract_ident)
}

fn extract_rust_macro_symbol(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("macro_rules!")?.trim();
    let token = rest
        .split(|c: char| c.is_whitespace() || c == '{' || c == '(')
        .next()
        .unwrap_or("")
        .trim_end_matches('!');
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn extract_func_name(line: &str) -> Option<String> {
    let before = line.split('(').next().unwrap_or("").trim();
    let name = before.split_whitespace().last().unwrap_or("").trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

#[cfg(test)]
fn classify_token_for_highlighting(language: Language, token: &str, next_non_space: Option<char>) -> &'static str {
    if is_solidity_mapping_token(language, token) || is_type_name(language, token) {
        "type"
    } else if is_builtin(language, token) {
        "builtin"
    } else if is_keyword(language, token) {
        "keyword"
    } else if next_non_space == Some('(') {
        "function"
    } else {
        "plain"
    }
}

#[cfg(test)]
fn token_classes_for_line(language: Language, line: &str) -> Vec<&'static str> {
    let mut out = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_ascii_alphabetic() || ch == '_' || ch == '@' || ch == '$' || ch == '%' {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '@' || chars[i] == '$' || chars[i] == '%')
            {
                i += 1;
            }
            let token: String = chars[start..i].iter().collect();
            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            out.push(classify_token_for_highlighting(language, &token, chars.get(j).copied()));
        } else {
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn has_label(items: &[CompletionItem], expected: &str) -> bool {
        items.iter().any(|i| i.label == expected)
    }

    fn assert_has_triggers(lang: Language, expected: &[&str]) {
        let snippets = language_snippets(lang);
        for trigger in expected {
            assert!(
                snippets.iter().any(|s| s.trigger == *trigger),
                "missing trigger '{}' for {:?}",
                trigger,
                lang
            );
        }
    }

    fn assert_has_symbol(lang: Language, lines: &[&str], expected: &str) {
        let lines = lines.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
        let syms = collect_symbols_for(lang, &lines);
        assert!(
            syms.iter().any(|(_, s)| s.contains(expected)),
            "missing symbol '{expected}' for {:?}",
            lang
        );
    }

    fn new_temp_editor_file(ext: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "steecleditor-test-{}-{ts}.{}",
            std::process::id(),
            ext
        ));
        fs::write(&path, "").expect("failed to create temporary test file");
        path
    }

    fn assert_completion_kind_order(items: &[CompletionItem]) {
        let mut seen_builtin = false;
        let mut seen_snippet = false;
        for item in items {
            if item.label.starts_with("keyword: ") {
                assert!(!seen_builtin && !seen_snippet, "keyword appears after builtin/snippet");
            } else if item.label.starts_with("builtin: ") {
                seen_builtin = true;
                assert!(!seen_snippet, "builtin appears after snippet");
            } else if item.label.starts_with("snippet: ") {
                seen_snippet = true;
            }
        }
    }

    #[test]
    fn detect_language_new_extensions() {
        let cases = [
            ("x.pl", Language::Perl),
            ("x.sh", Language::Shell),
            ("x.zsh", Language::Shell),
            ("x.kt", Language::Kotlin),
            ("x.swift", Language::Swift),
            ("x.dart", Language::Dart),
            ("x.ex", Language::Elixir),
            ("x.erl", Language::Erlang),
            ("x.clj", Language::Clojure),
            ("x.fs", Language::FSharp),
            ("x.r", Language::RLang),
            ("x.jl", Language::Julia),
            ("x.oct", Language::MatlabOctave),
            ("x.scala", Language::Scala),
            ("x.groovy", Language::Groovy),
            ("x.nim", Language::Nim),
            ("x.cr", Language::Crystal),
            ("x.f90", Language::Fortran),
            ("x.cob", Language::Cobol),
            ("x.adb", Language::Ada),
            ("x.asm", Language::Assembly),
            ("x.vlang", Language::VLang),
            ("x.sol", Language::Solidity),
            ("x.move", Language::Move),
            ("x.vhdl", Language::Vhdl),
            ("x.sv", Language::Verilog),
            ("x.pro", Language::Prolog),
            ("x.scm", Language::Scheme),
            ("x.st", Language::Smalltalk),
            ("x.tcl", Language::Tcl),
            ("x.ps1", Language::PowerShell),
            ("x.fish", Language::Fish),
            ("x.mk", Language::Makefile),
            ("Makefile", Language::Makefile),
            ("x.wgsl", Language::Wgsl),
            ("x.cl", Language::OpenClC),
            ("x.hack", Language::Hack),
            ("x.apex", Language::Apex),
            ("x.go", Language::Go),
            ("x.rs", Language::Rust),
            ("x.rb", Language::Ruby),
            ("x.pas", Language::Pascal),
            ("x.alg", Language::Algol),
            ("x.hc", Language::HolyC),
            ("x.hs", Language::Haskell),
            ("x.lua", Language::Lua),
            ("x.js", Language::JavaScript),
            ("x.ts", Language::TypeScript),
            ("x.coffee", Language::CoffeeScript),
            ("x.php", Language::Php),
            (".zshrc", Language::Shell),
        ];
        for (file, expected) in cases {
            let got = detect_language(&PathBuf::from(file));
            assert!(got == expected, "expected {:?} for {file}, got {:?}", expected, got);
        }
    }

    #[test]
    fn collect_symbols_for_new_languages() {
        let per_lang = [
            (Language::Perl, vec!["sub run_tests {".to_string()], "sub run_tests"),
            (Language::Shell, vec!["deploy() {".to_string()], "function deploy"),
            (Language::Kotlin, vec!["class Builder {".to_string()], "class Builder"),
            (Language::Swift, vec!["func build() {".to_string()], "func build"),
            (
                Language::Dart,
                vec!["extension StringX on String {".to_string()],
                "extension StringX",
            ),
            (Language::Solidity, vec!["contract Vault {".to_string()], "contract Vault"),
            (Language::PowerShell, vec!["enum Mode {".to_string()], "enum Mode"),
            (Language::Makefile, vec!["build:".to_string()], "target build"),
            (Language::Wgsl, vec!["fn shade() -> f32 {".to_string()], "fn shade"),
            (Language::OpenClC, vec!["__kernel void blur(__global float* img) {".to_string()], "kernel blur"),
            (Language::Hack, vec!["function build(): void {".to_string()], "function build"),
            (Language::Apex, vec!["trigger MyTrigger on Account (before insert) {".to_string()], "trigger MyTrigger"),
            (Language::Go, vec!["func BuildAll() {".to_string()], "func BuildAll"),
            (Language::Rust, vec!["fn parse_cfg() {".to_string()], "fn parse_cfg"),
            (Language::Ruby, vec!["def run_specs".to_string()], "def run_specs"),
            (Language::Pascal, vec!["procedure BuildAll;".to_string()], "procedure BuildAll"),
            (Language::Algol, vec!["procedure solve;".to_string()], "procedure solve"),
            (Language::HolyC, vec!["U0 Main() {".to_string()], "Main"),
            (Language::Haskell, vec!["main :: IO ()".to_string()], "sig main"),
            (Language::Lua, vec!["function build_all()".to_string()], "function build_all"),
            (Language::JavaScript, vec!["function buildAll() {".to_string()], "function buildAll"),
            (Language::TypeScript, vec!["class Builder {".to_string()], "class Builder"),
            (Language::CoffeeScript, vec!["function brew() {".to_string()], "function brew"),
            (Language::Php, vec!["function build_all() {".to_string()], "function build_all"),
        ];
        for (lang, lines, expected) in per_lang {
            let syms = collect_symbols_for(lang, &lines);
            assert!(syms.iter().any(|(_, s)| s.contains(expected)), "missing symbol '{expected}'");
        }
    }

    #[test]
    fn collect_symbols_phase2_extended_power_shell_dart() {
        assert_has_symbol(
            Language::PowerShell,
            &["class BuildConfig {"],
            "class BuildConfig",
        );
        assert_has_symbol(Language::PowerShell, &["enum BuildMode {"], "enum BuildMode");
        assert_has_symbol(
            Language::PowerShell,
            &["function Invoke-Build {"],
            "function Invoke-Build",
        );

        assert_has_symbol(Language::Dart, &["mixin JsonCodec {"], "mixin JsonCodec");
        assert_has_symbol(
            Language::Dart,
            &["extension StringX on String {"],
            "extension StringX",
        );
        assert_has_symbol(Language::Dart, &["class Builder {"], "class Builder");
    }

    #[test]
    fn completion_keywords_builtins_snippets_non_regression() {
        let ruby = language_completion_items(Language::Ruby, "rs");
        assert!(has_label(&ruby, "snippet: RSpec describe"));
        let ruby_main = language_completion_items(Language::Ruby, "ma");
        assert!(has_label(&ruby_main, "snippet: Ruby main guard"));

        let js_snip = language_completion_items(Language::JavaScript, "je");
        assert!(has_label(&js_snip, "snippet: Jest describe/it"));
        let ts_snip = language_completion_items(Language::TypeScript, "vi");
        assert!(has_label(&ts_snip, "snippet: TS Vitest test"));
        let rust_snip = language_completion_items(Language::Rust, "te");
        assert!(has_label(&rust_snip, "snippet: Rust #[test]"));

        let go = language_completion_items(Language::Go, "pr");
        assert!(has_label(&go, "builtin: print"));
        assert!(has_label(&go, "builtin: println"));
        let go_kw = language_completion_items(Language::Go, "fu");
        assert!(has_label(&go_kw, "keyword: func"));

        let zig = language_completion_items(Language::Zig, "@i");
        assert!(has_label(&zig, "builtin: @import"));

        let java = language_completion_items(Language::Java, "Sy");
        assert!(has_label(&java, "builtin: System"));

        let holyc = language_completion_items(Language::HolyC, "Pr");
        assert!(has_label(&holyc, "builtin: Print"));

        let pascal = language_completion_items(Language::Pascal, "wr");
        assert!(has_label(&pascal, "builtin: writeln"));
        let pas_kw = language_completion_items(Language::Pascal, "pro");
        assert!(has_label(&pas_kw, "keyword: program"));

        let algol = language_completion_items(Language::Algol, "pr");
        assert!(has_label(&algol, "builtin: print"));

        let haskell = language_completion_items(Language::Haskell, "ma");
        assert!(has_label(&haskell, "builtin: map"));

        let lua = language_completion_items(Language::Lua, "pr");
        assert!(has_label(&lua, "builtin: print"));

        let js = language_completion_items(Language::JavaScript, "con");
        assert!(has_label(&js, "builtin: console"));

        let ts = language_completion_items(Language::TypeScript, "Pro");
        assert!(has_label(&ts, "builtin: Promise"));

        let coffee = language_completion_items(Language::CoffeeScript, "con");
        assert!(has_label(&coffee, "builtin: console"));

        let php = language_completion_items(Language::Php, "js");
        assert!(has_label(&php, "builtin: json_encode"));

        let shell_kw = language_completion_items(Language::Shell, "if");
        assert!(has_label(&shell_kw, "keyword: if"));
        let shell_bi = language_completion_items(Language::Shell, "ec");
        assert!(has_label(&shell_bi, "builtin: echo"));
        let shell_cmd = language_completion_items(Language::Shell, "co");
        assert!(has_label(&shell_cmd, "builtin: command"));
        let perl_bi = language_completion_items(Language::Perl, "sa");
        assert!(has_label(&perl_bi, "builtin: say"));

        let wgsl = language_completion_items(Language::Wgsl, "ve");
        assert!(has_label(&wgsl, "builtin: vec2"));
        let wgsl_kw = language_completion_items(Language::Wgsl, "st");
        assert!(has_label(&wgsl_kw, "keyword: struct"));

        let opencl = language_completion_items(Language::OpenClC, "get_");
        assert!(has_label(&opencl, "builtin: get_global_id"));
        let opencl_kw = language_completion_items(Language::OpenClC, "__k");
        assert!(has_label(&opencl_kw, "keyword: __kernel"));

        let hack = language_completion_items(Language::Hack, "fu");
        assert!(has_label(&hack, "keyword: function"));
        let hack_bi = language_completion_items(Language::Hack, "ve");
        assert!(has_label(&hack_bi, "builtin: vec"));

        let apex = language_completion_items(Language::Apex, "tr");
        assert!(has_label(&apex, "keyword: trigger"));
        let apex_bi = language_completion_items(Language::Apex, "Sy");
        assert!(has_label(&apex_bi, "builtin: System"));

        let holyc_func = language_completion_items(Language::HolyC, "fu");
        assert!(has_label(&holyc_func, "snippet: HolyC function"));
        let holyc_const = language_completion_items(Language::HolyC, "co");
        assert!(has_label(&holyc_const, "snippet: HolyC const str"));
        let holyc_static = language_completion_items(Language::HolyC, "st");
        assert!(has_label(&holyc_static, "snippet: HolyC static arr"));
        let holyc_struct = language_completion_items(Language::HolyC, "str");
        assert!(has_label(&holyc_struct, "snippet: HolyC struct"));
        let holyc_assert = language_completion_items(Language::HolyC, "as");
        assert!(has_label(&holyc_assert, "snippet: HolyC assert"));
        let rust_main = language_completion_items(Language::Rust, "ma");
        assert!(has_label(&rust_main, "snippet: Rust main"));

        let kt_sn = language_completion_items(Language::Kotlin, "ma");
        assert!(has_label(&kt_sn, "snippet: Kotlin main"));
        let sw_sn = language_completion_items(Language::Swift, "fu");
        assert!(has_label(&sw_sn, "snippet: Swift function"));
        let dart_sn = language_completion_items(Language::Dart, "te");
        assert!(has_label(&dart_sn, "snippet: Dart group/test"));
        let sol_sn = language_completion_items(Language::Solidity, "fu");
        assert!(has_label(&sol_sn, "snippet: Solidity function"));
        let ps_sn = language_completion_items(Language::PowerShell, "te");
        assert!(has_label(&ps_sn, "snippet: PowerShell test it"));
        let mk_sn = language_completion_items(Language::Makefile, "te");
        assert!(has_label(&mk_sn, "snippet: Make test target"));
    }

    #[test]
    fn completion_new_30_languages_smoke() {
        let checks = [
            (Language::Kotlin, "cl", "keyword: class"),
            (Language::Swift, "pr", "builtin: print"),
            (Language::Dart, "Fu", "builtin: Future"),
            (Language::Elixir, "de", "keyword: def"),
            (Language::Erlang, "io", "builtin: io"),
            (Language::Clojure, "de", "keyword: def"),
            (Language::FSharp, "pr", "builtin: printfn"),
            (Language::RLang, "fu", "keyword: function"),
            (Language::Julia, "pr", "builtin: print"),
            (Language::MatlabOctave, "fu", "keyword: function"),
            (Language::Scala, "ob", "keyword: object"),
            (Language::Groovy, "de", "keyword: def"),
            (Language::Nim, "pr", "keyword: proc"),
            (Language::Crystal, "pu", "builtin: puts"),
            (Language::Fortran, "pr", "keyword: program"),
            (Language::Cobol, "DI", "keyword: DIVISION"),
            (Language::Ada, "pr", "keyword: procedure"),
            (Language::Assembly, "mo", "builtin: mov"),
            (Language::VLang, "mo", "keyword: module"),
            (Language::Solidity, "co", "keyword: contract"),
            (Language::Move, "mo", "keyword: module"),
            (Language::Vhdl, "en", "keyword: entity"),
            (Language::Verilog, "mo", "keyword: module"),
            (Language::Prolog, "wr", "builtin: write"),
            (Language::Scheme, "de", "keyword: define"),
            (Language::Smalltalk, "Tr", "builtin: Transcript"),
            (Language::Tcl, "pr", "keyword: proc"),
            (Language::PowerShell, "Wr", "builtin: Write-Host"),
            (Language::Fish, "ec", "builtin: echo"),
            (Language::Makefile, "in", "keyword: include"),
        ];
        for (lang, prefix, expected_label) in checks {
            let items = language_completion_items(lang, prefix);
            assert!(
                has_label(&items, expected_label),
                "missing '{expected_label}' for {:?} with prefix '{prefix}'",
                lang
            );
        }
    }

    #[test]
    fn completion_30_languages_stability() {
        let checks = [
            (Language::Kotlin, "cl", "keyword: class"),
            (Language::Swift, "pr", "builtin: print"),
            (Language::Dart, "Fu", "builtin: Future"),
            (Language::Elixir, "de", "keyword: def"),
            (Language::Erlang, "io", "builtin: io"),
            (Language::Clojure, "de", "keyword: def"),
            (Language::FSharp, "pr", "builtin: printfn"),
            (Language::RLang, "fu", "keyword: function"),
            (Language::Julia, "pr", "builtin: print"),
            (Language::MatlabOctave, "fu", "keyword: function"),
            (Language::Scala, "ob", "keyword: object"),
            (Language::Groovy, "de", "keyword: def"),
            (Language::Nim, "pr", "keyword: proc"),
            (Language::Crystal, "pu", "builtin: puts"),
            (Language::Fortran, "pr", "keyword: program"),
            (Language::Cobol, "DI", "keyword: DIVISION"),
            (Language::Ada, "pr", "keyword: procedure"),
            (Language::Assembly, "mo", "builtin: mov"),
            (Language::VLang, "mo", "keyword: module"),
            (Language::Solidity, "co", "keyword: contract"),
            (Language::Move, "mo", "keyword: module"),
            (Language::Vhdl, "en", "keyword: entity"),
            (Language::Verilog, "mo", "keyword: module"),
            (Language::Prolog, "wr", "builtin: write"),
            (Language::Scheme, "de", "keyword: define"),
            (Language::Smalltalk, "Tr", "builtin: Transcript"),
            (Language::Tcl, "pr", "keyword: proc"),
            (Language::PowerShell, "Wr", "builtin: Write-Host"),
            (Language::Fish, "ec", "builtin: echo"),
            (Language::Makefile, "in", "keyword: include"),
        ];
        for (lang, prefix, sentinel) in checks {
            let items = language_completion_items(lang, prefix);
            assert_completion_kind_order(&items);
            assert!(
                has_label(&items, sentinel),
                "missing sentinel '{sentinel}' for {:?} with prefix '{prefix}'",
                lang
            );
        }
    }

    #[test]
    fn advanced_highlighting_phase2_helpers() {
        assert_eq!(is_powershell_here_string_start('@', Some('"')), Some('"'));
        assert_eq!(is_powershell_here_string_start('@', Some('\'')), Some('\''));
        assert_eq!(is_powershell_here_string_start('@', Some('x')), None);

        assert!(is_makefile_recipe_line("\techo hi"));
        assert!(!is_makefile_recipe_line("echo hi"));
        assert!(is_makefile_var_start('$', Some('(')));
        assert!(!is_makefile_var_start('$', Some('{')));

        assert!(is_solidity_mapping_token(Language::Solidity, "mapping"));
        let classes = token_classes_for_line(Language::Solidity, "mapping(address => uint) balances;");
        assert!(classes.contains(&"type"), "solidity mapping should be typed");
    }

    #[test]
    fn phase2_highlighting_non_regression() {
        let powershell = token_classes_for_line(
            Language::PowerShell,
            "class BuildMode { enum Kind { One } }",
        );
        assert_eq!(powershell, vec!["keyword", "plain", "keyword", "plain", "plain"]);

        let makefile = token_classes_for_line(Language::Makefile, "include shared.mk");
        assert_eq!(makefile.first().copied(), Some("keyword"));
        assert!(makefile.iter().skip(1).all(|c| *c == "plain"));

        let solidity = token_classes_for_line(
            Language::Solidity,
            "event Updated(address indexed user); mapping(address => uint256) balances;",
        );
        assert_eq!(solidity.first().copied(), Some("keyword"));
        assert_eq!(solidity.get(1).copied(), Some("function"));
        assert!(solidity.contains(&"type"));

        let ps_here_start = token_classes_for_line(Language::PowerShell, "@'");
        let ps_here_end = token_classes_for_line(Language::PowerShell, "'@");
        assert!(ps_here_start.iter().all(|c| *c == "plain"));
        assert!(ps_here_end.iter().all(|c| *c == "plain"));
        assert!(is_powershell_here_string_start('@', Some('\'')) == Some('\''));
        assert!(is_powershell_here_string_terminator("'@", '\''));

        let make_recipe = token_classes_for_line(Language::Makefile, "\t$(CC) -o app main.o");
        assert!(is_makefile_recipe_line("\t$(CC) -o app main.o"));
        assert!(!make_recipe.is_empty());
    }

    #[test]
    fn holyc_snippets_non_regression() {
        let snippets = language_snippets(Language::HolyC);
        assert!(snippets.iter().any(|s| s.trigger == "func" && s.label == "HolyC function"));
        assert!(snippets.iter().any(|s| s.trigger == "test" && s.label == "HolyC test"));
        assert!(snippets.iter().any(|s| s.trigger == "assert" && s.label == "HolyC assert"));
        assert!(snippets.iter().any(|s| s.trigger == "const" && s.label == "HolyC const str"));
        assert!(snippets.iter().any(|s| s.trigger == "static" && s.label == "HolyC static arr"));
        assert!(snippets.iter().any(|s| s.trigger == "struct" && s.label == "HolyC struct"));
    }

    #[test]
    fn canonical_trigger_non_regression() {
        let cases = [
            (Language::Ruby, "rspec", "test"),
            (Language::Ruby, "spec", "test"),
            (Language::Ruby, "example", "test"),
            (Language::Ruby, "test", "test"),
            (Language::JavaScript, "jest", "jest"),
            (Language::HolyC, "test", "test"),
        ];
        for (lang, trigger, expected) in cases {
            assert_eq!(canonical_trigger(lang, trigger), expected);
        }
    }

    #[test]
    fn advanced_coloring_helpers() {
        let marker = parse_heredoc_marker(true, "<<-EOF");
        assert_eq!(marker.as_deref(), Some("EOF"));
        let marker_q = parse_heredoc_marker(true, "<<'SQL'");
        assert_eq!(marker_q.as_deref(), Some("SQL"));

        assert!(coffee_block_comment_closed("line\n###"));
        assert!(!coffee_block_comment_closed("line\n##"));

        let d1 = pascal_nested_depth_after("abc (* inner *) def *)", 1);
        assert_eq!(d1, 0);
        let d2 = pascal_nested_depth_after("abc (* inner", 1);
        assert_eq!(d2, 2);
    }

    #[test]
    fn golden_token_classes_by_language() {
        let cases = [
            (Language::Go, "func build() { print(value) }", vec!["keyword", "function", "builtin", "plain"]),
            (Language::Rust, "pub fn parse() { println!(x); }", vec!["keyword", "keyword", "function", "builtin", "plain"]),
            (Language::JavaScript, "async function run() { console.log(v) }", vec!["keyword", "keyword", "function", "builtin", "function", "plain"]),
            (Language::TypeScript, "interface X { readonly id: string }", vec!["keyword", "plain", "keyword", "plain", "plain"]),
            (Language::Php, "function run($value) { echo $value; }", vec!["keyword", "function", "builtin", "builtin", "builtin"]),
            (Language::Perl, "my $name = say($x);", vec!["keyword", "builtin", "builtin", "builtin"]),
            (Language::Shell, "if test \"$x\"; then echo ok; fi", vec!["keyword", "builtin", "plain", "keyword", "builtin", "plain", "keyword"]),
        ];
        for (lang, line, expected) in cases {
            let got = token_classes_for_line(lang, line);
            assert_eq!(got, expected, "golden mismatch for {:?} on line: {}", lang, line);
        }
    }

    #[test]
    fn debug_verbose_snapshot() {
        let prefix = "vi";
        let items = language_completion_items(Language::TypeScript, prefix);
        let lines = format_completion_debug_lines(
            Language::TypeScript,
            Path::new("samples/example.ts"),
            prefix,
            2,
            &items,
            true,
        );
        let expected = vec![
            "debug language: TypeScript".to_string(),
            "debug file: samples/example.ts".to_string(),
            "debug prefix: 'vi' (start=2)".to_string(),
            "debug completions: 1".to_string(),
            "debug mode: --verbose".to_string(),
            "debug extension: ts".to_string(),
            "debug prefix scanner: is_identifier_prefix=true".to_string(),
            "debug filters: keyword=0 builtin=0 snippet=1".to_string(),
            "  - snippet: TS Vitest test".to_string(),
        ];
        assert_eq!(lines, expected);
    }

    #[test]
    fn debug_non_verbose_snapshot() {
        let prefix = "ma";
        let items = language_completion_items(Language::Haskell, prefix);
        let lines = format_completion_debug_lines(
            Language::Haskell,
            Path::new("samples/example.hs"),
            prefix,
            3,
            &items,
            false,
        );
        let expected = vec![
            "debug language: Haskell".to_string(),
            "debug file: samples/example.hs".to_string(),
            "debug prefix: 'ma' (start=3)".to_string(),
            "debug completions: 2".to_string(),
            "  - builtin: map".to_string(),
            "  - snippet: Haskell main".to_string(),
        ];
        assert_eq!(lines, expected);
    }

    #[test]
    fn completion_order_is_stable() {
        let items = language_completion_items(Language::Haskell, "m");
        let mut saw_builtin = false;
        let mut saw_snippet = false;
        for item in items {
            let rank = if item.label.starts_with("keyword: ") {
                0
            } else if item.label.starts_with("builtin: ") {
                saw_builtin = true;
                1
            } else if item.label.starts_with("snippet: ") {
                saw_snippet = true;
                2
            } else {
                3
            };
            if rank == 0 {
                assert!(!saw_builtin && !saw_snippet, "keyword appears after builtin/snippet");
            } else if rank == 1 {
                assert!(!saw_snippet, "builtin appears after snippet");
            }
        }
        assert!(saw_builtin, "expected at least one builtin in completion set");
        assert!(saw_snippet, "expected at least one snippet in completion set");
    }

    #[test]
    fn collect_symbols_edge_cases() {
        let rust_lines = vec![
            "impl Worker {".to_string(),
            "  pub async fn nested_method() {}".to_string(),
            "}".to_string(),
            "macro_rules! build_vec { ($x:expr) => { vec![$x] } }".to_string(),
        ];
        let rust_syms = collect_symbols_for(Language::Rust, &rust_lines);
        assert!(rust_syms.iter().any(|(_, s)| s == "fn nested_method"));
        assert!(rust_syms.iter().any(|(_, s)| s == "macro build_vec"));

        let shell_lines = vec![
            "function deploy {".to_string(),
            "release() {".to_string(),
            "cleanup {".to_string(),
        ];
        let shell_syms = collect_symbols_for(Language::Shell, &shell_lines);
        assert!(shell_syms.iter().any(|(_, s)| s == "function deploy"));
        assert!(shell_syms.iter().any(|(_, s)| s == "function release"));
        assert!(shell_syms.iter().any(|(_, s)| s == "function cleanup"));

        let rust_false_positive = vec![
            "// fn commented_out() {}".to_string(),
            "\"fn string_literal() {}\"".to_string(),
        ];
        let rust_fp_syms = collect_symbols_for(Language::Rust, &rust_false_positive);
        assert!(rust_fp_syms.is_empty(), "rust false positives should be ignored");

        let shell_false_positive = vec![
            "# function hidden {}".to_string(),
            "\"deploy() {\"".to_string(),
        ];
        let shell_fp_syms = collect_symbols_for(Language::Shell, &shell_false_positive);
        assert!(shell_fp_syms.is_empty(), "shell false positives should be ignored");
    }

    #[test]
    fn symbols_false_positives_phase2() {
        let cases = [
            (
                Language::Kotlin,
                vec!["// class Hidden {}".to_string(), "\"fun fake() {}\"".to_string()],
            ),
            (
                Language::Swift,
                vec!["// func hidden() {}".to_string(), "\"class Ghost {}\"".to_string()],
            ),
            (
                Language::Dart,
                vec!["// class Hidden {}".to_string(), "\"void fake() {}\"".to_string()],
            ),
            (
                Language::Solidity,
                vec!["// contract Hidden {}".to_string(), "\"function fake() {}\"".to_string()],
            ),
            (
                Language::PowerShell,
                vec!["# function Hidden {}".to_string(), "\"function Ghost {}\"".to_string()],
            ),
            (
                Language::Makefile,
                vec!["# build:".to_string(), "\"test:\"".to_string()],
            ),
        ];
        for (lang, lines) in cases {
            let syms = collect_symbols_for(lang, &lines);
            assert!(syms.is_empty(), "false positives should be ignored for {:?}", lang);
        }
    }

    #[test]
    fn collect_symbols_phase2_false_positives() {
        let dart_false = vec![
            "// extension NotReal on String {".to_string(),
            "\"mixin NotReal {}\"".to_string(),
        ];
        let dart = collect_symbols_for(Language::Dart, &dart_false);
        assert!(dart.is_empty(), "dart false positives should be ignored");

        let powershell_false = vec![
            "# enum Hidden { A }".to_string(),
            "\"class Hidden {}\"".to_string(),
        ];
        let powershell = collect_symbols_for(Language::PowerShell, &powershell_false);
        assert!(powershell.is_empty(), "powershell false positives should be ignored");
    }

    #[test]
    #[ignore = "manual perf check"]
    fn benchmark_highlighting_large_files() {
        let js = include_str!("../../tests/fixtures/large.js");
        let rust = include_str!("../../tests/fixtures/large.rs");
        let php = include_str!("../../tests/fixtures/large.php");
        let iterations = 10usize;

        let start = Instant::now();
        for _ in 0..iterations {
            for line in js.lines() {
                let _ = token_classes_for_line(Language::JavaScript, line);
            }
            for line in rust.lines() {
                let _ = token_classes_for_line(Language::Rust, line);
            }
            for line in php.lines() {
                let _ = token_classes_for_line(Language::Php, line);
            }
        }
        let elapsed = start.elapsed();
        let total_lines = (js.lines().count() + rust.lines().count() + php.lines().count()) * iterations;
        eprintln!("highlight benchmark over {} fixture lines took {:?}", total_lines, elapsed);
        assert!(elapsed < Duration::from_secs(30), "benchmark too slow: {:?}", elapsed);
    }

    #[test]
    fn language_data_consistency() {
        let snippet_langs = [
            Language::Ruby,
            Language::Kotlin,
            Language::Swift,
            Language::Dart,
            Language::Solidity,
            Language::PowerShell,
            Language::Makefile,
            Language::Wgsl,
            Language::OpenClC,
            Language::Hack,
            Language::Apex,
            Language::Go,
            Language::Zig,
            Language::Java,
            Language::HolyC,
            Language::Pascal,
            Language::Algol,
            Language::Haskell,
            Language::Lua,
            Language::JavaScript,
            Language::TypeScript,
            Language::CoffeeScript,
            Language::Php,
            Language::Rust,
        ];
        for lang in snippet_langs {
            let snippets = language_snippets(lang);
            assert!(!snippets.is_empty(), "language {:?} must expose at least one snippet", lang);
            let mut uniq = HashSet::new();
            for snippet in snippets {
                assert!(uniq.insert(snippet.trigger), "duplicate trigger '{}' for {:?}", snippet.trigger, lang);
                let canonical = canonical_trigger(lang, snippet.trigger);
                assert!(
                    snippets.iter().any(|s| s.trigger == canonical),
                    "canonical trigger '{}' missing for {:?}",
                    canonical,
                    lang
                );
            }
            assert!(!uniq.is_empty(), "language {:?} must expose at least one unique trigger", lang);
        }
        assert_eq!(canonical_trigger(Language::Ruby, "rspec"), "test");
    }

    #[test]
    fn c_holyc_shared_consistency() {
        let c = language_builtins(Language::C, ShellDialect::Union);
        let holyc = language_builtins(Language::HolyC, ShellDialect::Union);
        assert!(c.contains(&"MemCpy"));
        assert!(c.contains(&"MemSet"));
        assert!(holyc.contains(&"MemCpy"));
        assert!(holyc.contains(&"MemSet"));
    }

    #[test]
    fn shell_builtins_union_sorted_unique() {
        assert!(!language_builtins(Language::Shell, ShellDialect::Posix).is_empty());
        assert!(!language_builtins(Language::Shell, ShellDialect::Bash).is_empty());
        assert!(!language_builtins(Language::Shell, ShellDialect::Zsh).is_empty());
        let union = language_builtins(Language::Shell, ShellDialect::Union);
        let mut sorted = union.to_vec();
        sorted.sort_unstable();
        assert_eq!(union, sorted.as_slice(), "shell union builtins must stay sorted for stable completion");
        let mut uniq = HashSet::new();
        for b in union {
            assert!(uniq.insert(*b), "duplicate shell builtin in union: {b}");
        }
    }

    #[test]
    fn shell_dialect_non_regression() {
        let posix = language_builtins(Language::Shell, ShellDialect::Posix);
        let bash = language_builtins(Language::Shell, ShellDialect::Bash);
        let zsh = language_builtins(Language::Shell, ShellDialect::Zsh);
        assert!(posix.contains(&"ulimit"), "posix sentinel missing");
        assert!(bash.contains(&"source"), "bash sentinel missing");
        assert!(zsh.contains(&"autoload"), "zsh sentinel missing");
    }

    #[test]
    fn snippet_alias_insert_equivalence() {
        let ruby_snippets = language_snippets(Language::Ruby);
        let rspec = ruby_snippets
            .iter()
            .find(|s| s.trigger == "rspec")
            .expect("missing rspec snippet");
        let example = ruby_snippets
            .iter()
            .find(|s| s.trigger == "example")
            .expect("missing example snippet");
        let spec = ruby_snippets
            .iter()
            .find(|s| s.trigger == "spec")
            .expect("missing spec snippet");
        let test = ruby_snippets
            .iter()
            .find(|s| s.trigger == "test")
            .expect("missing test snippet");
        assert_eq!(canonical_trigger(Language::Ruby, "rspec"), "test");
        assert_eq!(canonical_trigger(Language::Ruby, "spec"), "test");
        assert_eq!(canonical_trigger(Language::Ruby, "example"), "test");
        assert_eq!(rspec.body, test.body, "rspec alias body must match canonical test body");
        assert_eq!(example.body, test.body, "example alias body must match canonical test body");
        assert_eq!(spec.body, test.body, "spec alias body must match canonical test body");
    }

    #[test]
    fn completion_with_dialect_shell() {
        let posix = language_completion_items_with_dialect(Language::Shell, "au", ShellDialect::Posix);
        let bash = language_completion_items_with_dialect(Language::Shell, "au", ShellDialect::Bash);
        let zsh = language_completion_items_with_dialect(Language::Shell, "au", ShellDialect::Zsh);
        assert!(!has_label(&posix, "builtin: autoload"));
        assert!(!has_label(&bash, "builtin: autoload"));
        assert!(has_label(&zsh, "builtin: autoload"));

        let posix_src = language_completion_items_with_dialect(Language::Shell, "so", ShellDialect::Posix);
        let bash_src = language_completion_items_with_dialect(Language::Shell, "so", ShellDialect::Bash);
        assert!(!has_label(&posix_src, "builtin: source"));
        assert!(has_label(&bash_src, "builtin: source"));
    }

    #[test]
    fn language_alias_body_lint() {
        let snippet_langs = [
            Language::Ruby,
            Language::Go,
            Language::Zig,
            Language::Java,
            Language::HolyC,
            Language::Pascal,
            Language::Algol,
            Language::Haskell,
            Language::Lua,
            Language::JavaScript,
            Language::TypeScript,
            Language::CoffeeScript,
            Language::Php,
            Language::Rust,
        ];
        for lang in snippet_langs {
            let snippets = language_snippets(lang);
            for snippet in snippets {
                let canonical = canonical_trigger(lang, snippet.trigger);
                if canonical != snippet.trigger {
                    let Some(canonical_snippet) = snippets.iter().find(|s| s.trigger == canonical) else {
                        panic!("missing canonical trigger '{canonical}' for {:?}", lang);
                    };
                    assert_eq!(
                        snippet.body,
                        canonical_snippet.body,
                        "alias '{}' and canonical '{}' diverge for {:?}",
                        snippet.trigger,
                        canonical,
                        lang
                    );
                }
            }
        }
    }

    #[test]
    fn canonical_trigger_table_consistency() {
        for (lang, alias, canonical) in language_data::canonical_trigger_table() {
            assert_eq!(
                language_data::canonical_trigger(*lang, alias),
                Some(*canonical),
                "canonical table lookup mismatch for {:?}:{alias}",
                lang
            );
            let snippets = language_snippets(*lang);
            assert!(snippets.iter().any(|s| s.trigger == *alias), "missing alias trigger '{}'", alias);
            assert!(
                snippets.iter().any(|s| s.trigger == *canonical),
                "missing canonical trigger '{}' for {:?}",
                canonical,
                lang
            );
        }
    }

    #[test]
    fn snippet_templates_consistency() {
        let canonical = [
            Language::Ruby,
            Language::Kotlin,
            Language::Swift,
            Language::Dart,
            Language::Solidity,
            Language::PowerShell,
            Language::Makefile,
            Language::Wgsl,
            Language::OpenClC,
            Language::Hack,
            Language::Apex,
            Language::Go,
            Language::Java,
            Language::HolyC,
            Language::Pascal,
            Language::Lua,
            Language::Rust,
        ];
        for lang in canonical {
            assert_has_triggers(lang, &["main", "func", "test"]);
            let snippets = language_snippets(lang);
            for (trigger, allowed_tokens) in [
                ("main", &["main", "script", "contract", "target", "kernel", "entry", "class"][..]),
                ("func", &["func", "function", "method", "variable", "helper"][..]),
                ("test", &["test", "xctestcase", "rspec"][..]),
            ] {
                let sn = snippets
                    .iter()
                    .find(|s| s.trigger == trigger)
                    .unwrap_or_else(|| panic!("missing '{}' snippet for {:?}", trigger, lang));
                let label = sn.label.to_ascii_lowercase();
                assert!(
                    !label.trim().is_empty(),
                    "empty label for {:?} trigger '{}'",
                    lang,
                    trigger
                );
                assert!(
                    allowed_tokens.iter().any(|token| label.contains(token)),
                    "label '{}' for {:?} trigger '{}' should contain one of {:?}",
                    sn.label,
                    lang,
                    trigger,
                    allowed_tokens
                );
            }
        }

        let documented_exceptions = [
            (Language::Algol, "begin"),
            (Language::CoffeeScript, "func"),
            (Language::Php, "func"),
            (Language::Haskell, "main"),
            (Language::JavaScript, "jest"),
            (Language::TypeScript, "vitest"),
            (Language::Zig, "main"),
        ];
        for (lang, trigger) in documented_exceptions {
            assert_has_triggers(lang, &[trigger]);
        }
    }

    fn generated_snippet_doc_snapshot() -> String {
        let langs = [
            Language::Ruby,
            Language::Kotlin,
            Language::Swift,
            Language::Dart,
            Language::Solidity,
            Language::PowerShell,
            Language::Makefile,
            Language::Wgsl,
            Language::OpenClC,
            Language::Hack,
            Language::Apex,
            Language::Go,
            Language::Zig,
            Language::Java,
            Language::HolyC,
            Language::Pascal,
            Language::Algol,
            Language::Haskell,
            Language::Lua,
            Language::JavaScript,
            Language::TypeScript,
            Language::CoffeeScript,
            Language::Php,
            Language::Rust,
        ];
        let mut lines = Vec::new();
        for lang in langs {
            let mut triggers = language_snippets(lang)
                .iter()
                .map(|s| s.trigger)
                .collect::<Vec<_>>();
            triggers.sort_unstable();
            lines.push(format!("{}: {}", language_label_for(lang), triggers.join(", ")));
        }
        lines.join("\n")
    }

    fn update_snippet_snapshot_block(doc: &str, generated: &str) -> String {
        let start_marker = "<!-- SNIPPET-SNAPSHOT:START -->";
        let end_marker = "<!-- SNIPPET-SNAPSHOT:END -->";
        let start = doc.find(start_marker).expect("missing snippet snapshot start marker");
        let rest = &doc[start + start_marker.len()..];
        let end_rel = rest.find(end_marker).expect("missing snippet snapshot end marker");
        let after = &rest[end_rel..];
        let mut out = String::new();
        out.push_str(&doc[..start + start_marker.len()]);
        out.push('\n');
        out.push_str("```text\n");
        out.push_str(generated);
        out.push('\n');
        out.push_str("```\n");
        out.push_str(after);
        out
    }

    fn generated_new30_capability_snapshot() -> String {
        let mut lines = Vec::new();
        for lang in language_data::new30_languages() {
            let advanced = if language_data::is_advanced_new30(*lang) { "yes" } else { "no" };
            lines.push(format!(
                "{}: base=yes advanced={advanced}",
                language_label_for(*lang)
            ));
        }
        lines.join("\n")
    }

    fn generated_phase2_coverage_snapshot() -> String {
        [
            "| Language | Advanced highlighting | Symbols | Snippets | QA tests | Known gaps |",
            "|---|---|---|---|---|---|",
            "| Kotlin | yes (base C-like + phase2 non-regression) | class/fun | main/func/test | `collect_symbols_for_new_languages`, `symbols_false_positives_phase2`, `phase2_highlighting_non_regression` | no dedicated multiline literals edge-case test yet |",
            "| Swift | yes (base C-like + phase2 non-regression) | struct/class/func | main/func/test | `collect_symbols_for_new_languages`, `symbols_false_positives_phase2`, `phase2_highlighting_non_regression` | protocol/extension symbol nuances remain heuristic |",
            "| Dart | yes (base C-like + phase2 non-regression) | class/func + extension/mixin | main/func/test | `collect_symbols_phase2_extended_power_shell_dart`, `collect_symbols_phase2_false_positives`, `phase2_highlighting_non_regression` | generic extension edge cases still regex-based |",
            "| Solidity | yes (`mapping(...)`, phase2 non-regression) | contract/event/function | main/func/test | `collect_symbols_for_new_languages`, `symbols_false_positives_phase2`, `phase2_highlighting_non_regression` | modifier/library coverage still partial in symbols |",
            "| PowerShell | yes (here-strings + phase2 non-regression) | function + class/enum | main/func/test | `collect_symbols_phase2_extended_power_shell_dart`, `collect_symbols_phase2_false_positives`, `phase2_highlighting_non_regression`, `render_line_here_string_roundtrip`, `render_line_here_string_double_quote_roundtrip` | parser remains line-oriented for complex scriptblocks |",
            "| Makefile | yes (`$(...)` vars, tab recipes) | target | main/func/test | `collect_symbols_for_new_languages`, `collect_symbols_makefile_false_positives`, `collect_symbols_makefile_pattern_targets`, `symbols_false_positives_phase2`, `phase2_highlighting_non_regression` | pattern/static pattern targets are simplified |",
        ]
        .join("\n")
    }

    fn generated_phase2_non_goals_snapshot() -> String {
        [
            "- No AST parser per language in phase2; symbol detection remains regex/pattern based.",
            "- No full shell/PowerShell parser state machine for every nested quoting edge case.",
            "- No exhaustive Makefile semantics (pattern-specific variants, include graph resolution, eval/call expansion).",
            "- No strict framework-specific snippet style unification when idiomatic labels are clearer (example: XCTestCase, group/test, Make target).",
        ]
        .join("\n")
    }

    fn update_phase2_coverage_block(doc: &str, generated: &str) -> String {
        let start_marker = "<!-- PHASE2-COV:START -->";
        let end_marker = "<!-- PHASE2-COV:END -->";
        let start = doc.find(start_marker).expect("missing phase2 coverage start marker");
        let rest = &doc[start + start_marker.len()..];
        let end_rel = rest.find(end_marker).expect("missing phase2 coverage end marker");
        let after = &rest[end_rel..];
        let mut out = String::new();
        out.push_str(&doc[..start + start_marker.len()]);
        out.push('\n');
        out.push_str(generated);
        out.push('\n');
        out.push_str(after);
        out
    }

    fn update_phase2_non_goals_block(doc: &str, generated: &str) -> String {
        let start_marker = "<!-- PHASE2-NONGOALS:START -->";
        let end_marker = "<!-- PHASE2-NONGOALS:END -->";
        let start = doc.find(start_marker).expect("missing phase2 non-goals start marker");
        let rest = &doc[start + start_marker.len()..];
        let end_rel = rest.find(end_marker).expect("missing phase2 non-goals end marker");
        let after = &rest[end_rel..];
        let mut out = String::new();
        out.push_str(&doc[..start + start_marker.len()]);
        out.push('\n');
        out.push_str(generated);
        out.push('\n');
        out.push_str(after);
        out
    }

    fn update_new30_capability_block(doc: &str, generated: &str) -> String {
        let start_marker = "<!-- NEW30-CAP:START -->";
        let end_marker = "<!-- NEW30-CAP:END -->";
        let start = doc.find(start_marker).expect("missing new30 capability start marker");
        let rest = &doc[start + start_marker.len()..];
        let end_rel = rest.find(end_marker).expect("missing new30 capability end marker");
        let after = &rest[end_rel..];
        let mut out = String::new();
        out.push_str(&doc[..start + start_marker.len()]);
        out.push('\n');
        out.push_str("```text\n");
        out.push_str(generated);
        out.push('\n');
        out.push_str("```\n");
        out.push_str(after);
        out
    }

    #[test]
    fn snippets_doc_snapshot_matches_language_data() {
        let doc = include_str!("../../docs/editor-setup.md");
        let start_marker = "<!-- SNIPPET-SNAPSHOT:START -->";
        let end_marker = "<!-- SNIPPET-SNAPSHOT:END -->";
        let start = doc.find(start_marker).expect("missing snippet snapshot start marker");
        let rest = &doc[start + start_marker.len()..];
        let end_rel = rest.find(end_marker).expect("missing snippet snapshot end marker");
        let section = rest[..end_rel].trim();
        let section = section
            .strip_prefix("```text")
            .and_then(|s| s.strip_suffix("```"))
            .map(str::trim)
            .expect("snapshot block must be fenced as ```text");
        let generated = generated_snippet_doc_snapshot();
        assert_eq!(section, generated, "snippet snapshot in docs is out of sync with language_data");
    }

    #[test]
    fn new30_capability_snapshot_matches_language_data() {
        let doc = include_str!("../../docs/editor-setup.md");
        let start_marker = "<!-- NEW30-CAP:START -->";
        let end_marker = "<!-- NEW30-CAP:END -->";
        let start = doc.find(start_marker).expect("missing new30 capability start marker");
        let rest = &doc[start + start_marker.len()..];
        let end_rel = rest.find(end_marker).expect("missing new30 capability end marker");
        let section = rest[..end_rel].trim();
        let section = section
            .strip_prefix("```text")
            .and_then(|s| s.strip_suffix("```"))
            .map(str::trim)
            .expect("new30 capability block must be fenced as ```text");
        let generated = generated_new30_capability_snapshot();
        assert_eq!(
            section, generated,
            "new30 capability snapshot in docs is out of sync with language_data"
        );
    }

    #[test]
    fn support_matrix_doc_snapshot_matches_language_data() {
        new30_capability_snapshot_matches_language_data();
        phase2_coverage_snapshot_matches_docs();
        phase2_coverage_non_goals_snapshot();
    }

    #[test]
    fn readme_site_downloads_policy_present() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("## Site downloads policy"),
            "README must document site downloads policy section"
        );
        assert!(
            readme.contains("GitHub Releases"),
            "README site policy should mention GitHub Releases"
        );
        assert!(
            readme.contains("VS Code extension"),
            "README site policy should mention VS Code extension"
        );
    }

    #[test]
    fn readme_site_downloads_policy_content_non_regression() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("La page Downloads pointe vers un **lien unique GitHub Releases** pour toutes les plateformes."),
            "README site policy should keep the single Releases rule"
        );
        assert!(
            readme.contains("La page Downloads expose aussi le lien **VS Code extension** (Marketplace)."),
            "README site policy should keep the VS Code Marketplace rule"
        );
        assert!(
            readme.contains("gardez cette convention"),
            "README site policy should keep the anti-regression convention note"
        );
    }

    #[test]
    fn ci_site_quick_contract() {
        let workflow = include_str!("../../.github/workflows/steecleditor.yml");
        let start = workflow.find("  site-quick:").expect("workflow must define a site-quick job");
        let tail = &workflow[start..];
        let end = tail.find("\n  phase2-quick:").unwrap_or(tail.len());
        let site_quick = &tail[..end];
        assert!(
            site_quick.contains("run: npm ci"),
            "site-quick must install docs/angular dependencies with npm ci"
        );
        assert!(
            site_quick.contains("run: npm run build:verify"),
            "site-quick must run npm run build:verify"
        );
        assert!(
            site_quick.contains("run: ./scripts/verify-site-sync.sh"),
            "site-quick must verify docs/site sync via scripts/verify-site-sync.sh"
        );
    }

    #[test]
    fn site_diff_preview_contract() {
        let workflow = include_str!("../../.github/workflows/steecleditor.yml");
        let start = workflow
            .find("  site-diff-preview:")
            .expect("workflow must define a site-diff-preview job");
        let tail = &workflow[start..];
        let end = tail.find("\n  phase2-quick:").unwrap_or(tail.len());
        let site_preview = &tail[..end];
        assert!(
            site_preview.contains("continue-on-error: true"),
            "site-diff-preview must stay non-blocking with continue-on-error: true"
        );
        assert!(
            site_preview.contains("name: docs-site-diff-preview"),
            "site-diff-preview must upload docs-site-diff-preview artifact"
        );
    }

    #[test]
    fn site_urls_quick_contract() {
        let workflow = include_str!("../../.github/workflows/steecleditor.yml");
        let start = workflow
            .find("  site-urls-quick:")
            .expect("workflow must define a site-urls-quick job");
        let tail = &workflow[start..];
        let end = tail.find("\n  site-diff-preview:").unwrap_or(tail.len());
        let site_urls = &tail[..end];
        assert!(
            site_urls.contains("run: npm run check:urls"),
            "site-urls-quick must execute npm run check:urls"
        );
        assert!(
            !site_urls.contains("npm run build") && !site_urls.contains("build:verify"),
            "site-urls-quick must not run a build command"
        );
    }

    #[test]
    fn verify_site_sync_script_contract() {
        let script = include_str!("../../scripts/verify-site-sync.sh");
        assert!(
            script.contains("--allow-diff"),
            "verify-site-sync.sh must support --allow-diff"
        );
        assert!(
            script.contains("--diff-file"),
            "verify-site-sync.sh must support --diff-file"
        );
        assert!(
            script.contains("docs/site is out of sync with docs/angular sources"),
            "verify-site-sync.sh must keep stable out-of-sync error message"
        );
        assert!(
            script.contains("docs/site is in sync"),
            "verify-site-sync.sh must keep stable in-sync output message"
        );
    }

    #[test]
    fn qa_local_script_contract() {
        let package_json = include_str!("../../docs/angular/package.json");
        assert!(
            package_json.contains("\"qa:local\": \"npm ci && npm run build:verify\""),
            "docs/angular/package.json must keep qa:local script contract"
        );
    }

    #[test]
    fn qa_site_local_script_contract() {
        let script = include_str!("../../scripts/qa-site-local.sh");
        let pos_editor = script
            .find("./scripts/verify-editorconfig.sh")
            .expect("qa-site-local.sh must run verify-editorconfig first");
        let pos_npm = script
            .find("npm run qa:local")
            .expect("qa-site-local.sh must run npm run qa:local");
        let pos_sync = script
            .find("./scripts/verify-site-sync.sh")
            .expect("qa-site-local.sh must run verify-site-sync");
        assert!(
            pos_editor < pos_npm && pos_npm < pos_sync,
            "qa-site-local.sh order must be verify-editorconfig -> npm run qa:local -> verify-site-sync"
        );
        assert!(
            script.contains("--no-npm-ci"),
            "qa-site-local.sh must support --no-npm-ci"
        );
    }

    #[test]
    fn readme_local_qa_commands_non_regression() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("### Local QA"),
            "README should keep Local QA section for site workflow"
        );
        assert!(readme.contains("npm ci"), "Local QA must include npm ci");
        assert!(
            readme.contains("npm run build:verify"),
            "Local QA must include npm run build:verify"
        );
        assert!(
            readme.contains("git diff --quiet -- docs/site"),
            "Local QA must include docs/site sync check"
        );
    }

    #[test]
    fn readme_verify_editorconfig_note_non_regression() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("./scripts/verify-editorconfig.sh"),
            "README Local QA should keep verify-editorconfig.sh note"
        );
        assert!(
            readme.contains("steelconf") && readme.contains("*.muf"),
            "README note should mention steelconf and *.muf scope"
        );
    }

    #[test]
    fn ci_jobs_readme_alignment_non_regression() {
        let readme = include_str!("../../README.md");
        let workflow = include_str!("../../.github/workflows/steecleditor.yml");
        let section_start = readme
            .find("### CI quick jobs")
            .expect("README must include CI quick jobs section");
        let rest = &readme[section_start..];
        let section_end = rest
            .find("\n### ")
            .unwrap_or(rest.len());
        let section = &rest[..section_end];
        for line in section.lines() {
            let Some(tick_start) = line.find('`') else { continue };
            let after = &line[tick_start + 1..];
            let Some(tick_end) = after.find('`') else { continue };
            let job = &after[..tick_end];
            let needle = format!("  {job}:");
            assert!(
                workflow.contains(&needle),
                "README CI quick job '{job}' must exist in steecleditor.yml"
            );
        }
    }

    #[test]
    fn site_quick_contracts_bundle() {
        ci_site_quick_contract();
        site_diff_preview_contract();
        site_urls_quick_contract();
        qa_local_script_contract();
    }

    #[test]
    fn phase2_coverage_snapshot_matches_docs() {
        let doc = include_str!("../../docs/editor-setup.md");
        let start_marker = "<!-- PHASE2-COV:START -->";
        let end_marker = "<!-- PHASE2-COV:END -->";
        let start = doc.find(start_marker).expect("missing phase2 coverage start marker");
        let rest = &doc[start + start_marker.len()..];
        let end_rel = rest.find(end_marker).expect("missing phase2 coverage end marker");
        let section = rest[..end_rel].trim();
        let generated = generated_phase2_coverage_snapshot();
        assert_eq!(
            section, generated,
            "phase2 coverage table in docs is out of sync with tests/capabilities"
        );
    }

    #[test]
    fn phase2_coverage_non_goals_snapshot() {
        let doc = include_str!("../../docs/editor-setup.md");
        let start_marker = "<!-- PHASE2-NONGOALS:START -->";
        let end_marker = "<!-- PHASE2-NONGOALS:END -->";
        let start = doc.find(start_marker).expect("missing phase2 non-goals start marker");
        let rest = &doc[start + start_marker.len()..];
        let end_rel = rest.find(end_marker).expect("missing phase2 non-goals end marker");
        let section = rest[..end_rel].trim();
        let generated = generated_phase2_non_goals_snapshot();
        assert_eq!(
            section, generated,
            "phase2 non-goals snapshot in docs is out of sync with expected content"
        );
    }

    #[test]
    fn phase2_coverage_snapshot_format_non_regression() {
        let generated = generated_phase2_coverage_snapshot();
        let mut lines = generated.lines();
        let header = lines.next().expect("missing phase2 coverage header");
        assert_eq!(
            header,
            "| Language | Advanced highlighting | Symbols | Snippets | QA tests | Known gaps |"
        );
        let sep = lines.next().expect("missing phase2 coverage separator");
        assert_eq!(sep, "|---|---|---|---|---|---|");
        for line in lines {
            assert_eq!(
                line.matches('|').count(),
                7,
                "unexpected phase2 coverage column count in line: {line}"
            );
        }
    }

    #[test]
    fn status_bar_shell_dialect_visibility() {
        assert_eq!(configured_shell_dialect_from_raw(""), None);
        assert_eq!(shell_dialect_status_segment_from_raw(""), None);
        assert_eq!(
            shell_dialect_status_segment_from_raw("union").as_deref(),
            Some(" | sh:union")
        );
        let posix = configured_shell_dialect_from_raw("posix").expect("missing posix dialect");
        assert_eq!(shell_dialect_label(posix), "posix");
        assert_eq!(
            configured_shell_dialect_from_raw(" zsh ").map(shell_dialect_label),
            Some("zsh")
        );
        assert_eq!(
            shell_dialect_status_segment_from_raw("bash").as_deref(),
            Some(" | sh:bash")
        );
    }

    #[test]
    fn status_bar_shell_dialect_render_prefix() {
        assert_eq!(
            shell_dialect_status_segment_from_raw("posix").as_deref(),
            Some(" | sh:posix")
        );
    }

    #[test]
    fn shell_dialect_status_segment_trimmed_input() {
        assert_eq!(
            shell_dialect_status_segment_from_raw(" zsh ").as_deref(),
            Some(" | sh:zsh")
        );
    }

    #[test]
    fn render_line_here_string_roundtrip() {
        let path = new_temp_editor_file("ps1");
        let mut editor = Editor::open(path.clone()).expect("failed to open temp editor file");
        editor.language = Language::PowerShell;
        let mut out = stdout();
        let mut raw_state = None;
        let mut raw_carry = String::new();
        let mut py_state = None;
        let mut py_carry = String::new();

        editor
            .render_line(
                &mut out,
                "@'",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render start failed");
        match raw_state {
            Some(MultiLineState::String(ref term)) => assert_eq!(term, "LINE:'@"),
            _ => panic!("expected here-string multiline state after start"),
        }

        editor
            .render_line(
                &mut out,
                "content with $var and symbols",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render body failed");
        assert!(matches!(raw_state, Some(MultiLineState::String(_))));

        editor
            .render_line(
                &mut out,
                "'@",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render end failed");
        assert!(raw_state.is_none(), "expected multiline state to close on '@ marker");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn render_line_here_string_double_quote_roundtrip() {
        let path = new_temp_editor_file("ps1");
        let mut editor = Editor::open(path.clone()).expect("failed to open temp editor file");
        editor.language = Language::PowerShell;
        let mut out = stdout();
        let mut raw_state = None;
        let mut raw_carry = String::new();
        let mut py_state = None;
        let mut py_carry = String::new();

        editor
            .render_line(
                &mut out,
                "@\"",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render start failed");
        match raw_state {
            Some(MultiLineState::String(ref term)) => assert_eq!(term, "LINE:\"@"),
            _ => panic!("expected double-quote here-string multiline state after start"),
        }

        editor
            .render_line(
                &mut out,
                "content with interpolation-like $value",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render body failed");
        assert!(matches!(raw_state, Some(MultiLineState::String(_))));

        editor
            .render_line(
                &mut out,
                "\"@",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render end failed");
        assert!(raw_state.is_none(), "expected multiline state to close on \"@ marker");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn render_line_here_string_whitespace_terminator_roundtrip() {
        let path = new_temp_editor_file("ps1");
        let mut editor = Editor::open(path.clone()).expect("failed to open temp editor file");
        editor.language = Language::PowerShell;
        let mut out = stdout();
        let mut raw_state = None;
        let mut raw_carry = String::new();
        let mut py_state = None;
        let mut py_carry = String::new();

        editor
            .render_line(
                &mut out,
                "@\"",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render start failed");
        assert!(matches!(raw_state, Some(MultiLineState::String(_))));

        editor
            .render_line(
                &mut out,
                "\t\"@",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render end with tab failed");
        assert!(raw_state.is_none(), "expected multiline state to close on tab-indented \"@ marker");

        editor
            .render_line(
                &mut out,
                "@'",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render single-quote start failed");
        assert!(matches!(raw_state, Some(MultiLineState::String(_))));
        editor
            .render_line(
                &mut out,
                "\t'@",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render single-quote end with tab failed");
        assert!(raw_state.is_none(), "expected multiline state to close on tab-indented '@ marker");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn render_line_here_string_no_terminator_keeps_state() {
        let path = new_temp_editor_file("ps1");
        let mut editor = Editor::open(path.clone()).expect("failed to open temp editor file");
        editor.language = Language::PowerShell;
        let mut out = stdout();
        let mut raw_state = None;
        let mut raw_carry = String::new();
        let mut py_state = None;
        let mut py_carry = String::new();

        editor
            .render_line(
                &mut out,
                "@\"",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render start failed");
        assert!(matches!(raw_state, Some(MultiLineState::String(_))));
        editor
            .render_line(
                &mut out,
                "\"@x",
                120,
                &mut raw_state,
                &mut raw_carry,
                &mut py_state,
                &mut py_carry,
            )
            .expect("render non-terminator failed");
        assert!(
            matches!(raw_state, Some(MultiLineState::String(_))),
            "state must stay open for non-terminator marker"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn holyc_test_assert_complementary() {
        let snippets = language_snippets(Language::HolyC);
        let test = snippets
            .iter()
            .find(|s| s.trigger == "test")
            .expect("missing HolyC test snippet");
        assert!(
            test.body.contains("assert"),
            "HolyC test snippet should hint assert trigger usage"
        );
        assert!(snippets.iter().any(|s| s.trigger == "assert"), "missing HolyC assert snippet");
    }

    #[test]
    fn shell_env_dialect_override_non_regression() {
        let cases = [
            ("posix", ShellDialect::Posix),
            ("bash", ShellDialect::Bash),
            ("zsh", ShellDialect::Zsh),
            ("union", ShellDialect::Union),
            ("invalid", ShellDialect::Union),
            ("", ShellDialect::Union),
        ];
        for (raw, expected) in cases {
            assert_eq!(parse_shell_dialect(raw), expected, "parse failed for '{raw}'");
        }
    }

    #[test]
    fn collect_symbols_makefile_false_positives() {
        let lines = vec![
            "X := \"build:\"".to_string(),
            "VAR:=$(shell echo build:)".to_string(),
            "export X:=build".to_string(),
            "# test:".to_string(),
            "MESSAGE = release:ok".to_string(),
        ];
        let syms = collect_symbols_for(Language::Makefile, &lines);
        assert!(syms.is_empty(), "makefile assignments/comments should not create targets");
    }

    #[test]
    fn collect_symbols_makefile_pattern_targets() {
        let lines = vec![
            "%.o: %.c".to_string(),
            "%: %.in".to_string(),
        ];
        let syms = collect_symbols_for(Language::Makefile, &lines);
        assert!(syms.iter().any(|(_, s)| s == "target %.o"));
        assert!(syms.iter().any(|(_, s)| s == "target %"));
    }

    #[test]
    fn collect_symbols_makefile_static_pattern_targets() {
        assert_has_symbol(
            Language::Makefile,
            &["targets: %.o: %.c"],
            "target targets",
        );
    }

    #[test]
    fn collect_symbols_makefile_order_stability() {
        let lines = vec![
            "build:".to_string(),
            "test:".to_string(),
            "deploy:".to_string(),
        ];
        let syms = collect_symbols_for(Language::Makefile, &lines);
        let got = syms.into_iter().map(|(_, s)| s).collect::<Vec<_>>();
        assert_eq!(
            got,
            vec![
                "target build".to_string(),
                "target test".to_string(),
                "target deploy".to_string(),
            ]
        );
    }

    #[test]
    fn makefile_symbol_assignment_guard_non_regression() {
        let lines = vec!["X := \"build:\"".to_string()];
        let syms = collect_symbols_for(Language::Makefile, &lines);
        assert!(
            syms.is_empty(),
            "assignment with := must not be treated as a make target symbol"
        );
    }

    #[test]
    fn phase2_non_goals_doc_presence() {
        let doc = include_str!("../../docs/editor-setup.md");
        let marker = "### Phase2 non-goals";
        let start = doc.find(marker).expect("missing 'Phase2 non-goals' section");
        let tail = &doc[start + marker.len()..];
        let next_h2 = tail.find("\n## ").unwrap_or(tail.len());
        let section = &tail[..next_h2];
        let bullets = section
            .lines()
            .filter(|line| line.trim_start().starts_with("- "))
            .count();
        assert!(
            bullets >= 3,
            "phase2 non-goals section must keep at least 3 bullets (found {bullets})"
        );
    }

    #[test]
    #[ignore = "manual utility check; invokes cargo via script"]
    fn update_snippet_snapshot_script_idempotent() {
        let run = |label: &str| {
            let status = Command::new("bash")
                .arg("scripts/update-snippet-snapshot.sh")
                .status()
                .unwrap_or_else(|e| panic!("{label}: failed to run script: {e}"));
            assert!(status.success(), "{label}: script failed");
        };
        run("first");
        run("second");
        let status = Command::new("git")
            .args(["diff", "--quiet", "--", "docs/editor-setup.md"])
            .status()
            .expect("failed to run git diff");
        assert!(status.success(), "docs/editor-setup.md changed after two script runs");
    }

    #[test]
    #[ignore = "manual utility to rewrite docs snippet snapshot"]
    fn regenerate_snippet_snapshot_block() {
        let path = Path::new("docs/editor-setup.md");
        let doc = fs::read_to_string(path).expect("failed to read docs/editor-setup.md");
        let generated = generated_snippet_doc_snapshot();
        let generated_caps = generated_new30_capability_snapshot();
        let generated_phase2 = generated_phase2_coverage_snapshot();
        let generated_non_goals = generated_phase2_non_goals_snapshot();
        let updated_snippets = update_snippet_snapshot_block(&doc, &generated);
        let updated_caps = update_new30_capability_block(&updated_snippets, &generated_caps);
        let updated_phase2 = update_phase2_coverage_block(&updated_caps, &generated_phase2);
        let updated = update_phase2_non_goals_block(&updated_phase2, &generated_non_goals);
        fs::write(path, updated).expect("failed to write docs/editor-setup.md");
    }

    #[test]
    #[ignore = "manual utility to rewrite docs snapshots (snippets/support matrix/phase2 coverage)"]
    fn regenerate_doc_snapshot_blocks() {
        let path = Path::new("docs/editor-setup.md");
        let doc = fs::read_to_string(path).expect("failed to read docs/editor-setup.md");
        let generated = generated_snippet_doc_snapshot();
        let generated_caps = generated_new30_capability_snapshot();
        let generated_phase2 = generated_phase2_coverage_snapshot();
        let generated_non_goals = generated_phase2_non_goals_snapshot();
        let updated_snippets = update_snippet_snapshot_block(&doc, &generated);
        let updated_caps = update_new30_capability_block(&updated_snippets, &generated_caps);
        let updated_phase2 = update_phase2_coverage_block(&updated_caps, &generated_phase2);
        let updated = update_phase2_non_goals_block(&updated_phase2, &generated_non_goals);
        fs::write(path, updated).expect("failed to write docs/editor-setup.md");
    }
}
