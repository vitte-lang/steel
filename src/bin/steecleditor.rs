use std::fs;
use std::io::{self, stdout, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{execute, queue};
use regex::Regex;
use walkdir::WalkDir;

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

#[derive(Copy, Clone, PartialEq)]
enum Language {
    Steelconf,
    C,
    Cpp,
    Python,
    Java,
    Ocaml,
    Zig,
    CSharp,
    Other,
}

#[derive(Copy, Clone)]
enum Theme {
    Dark,
    Light,
}

struct ThemeColors {
    fg: Color,
    comment: Color,
    keyword: Color,
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
                keyword: Color::Cyan,
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
                keyword: Color::Blue,
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

struct EditorConfig {
    autosave_interval: Option<u64>,
    theme: Option<Theme>,
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
    extra_cursors: Vec<(usize, usize)>,
    line_ending: LineEnding,
    encoding: String,
    show_run_panel: bool,
    run_output: Vec<String>,
    run_status: Option<i32>,
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
            extra_cursors: Vec::new(),
            line_ending,
            encoding,
            show_run_panel: false,
            run_output: Vec::new(),
            run_status: None,
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
                KeyCode::Char('n') => {
                    self.goto_next_match_word();
                    return Ok(false);
                }
                KeyCode::Char('k') => {
                    self.cut_line();
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
                KeyCode::Char('x') => {
                    if self.dirty && !self.confirm_quit {
                        self.status = "Unsaved changes. Press Ctrl+X again to quit.".to_string();
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
        let height = rows.saturating_sub(3) as usize;

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
                    self.render_line(out, chunk, text_cols)?;
                    screen_row += 1;
                    start += text_cols;
                }
            } else {
                let y = screen_row as u16;
                queue!(out, cursor::MoveTo(0, y), Clear(ClearType::CurrentLine))?;
                self.render_line(out, line, text_cols)?;
                if !self.soft_wrap {
                    self.render_minimap(out, y, cols)?;
                }
                screen_row += 1;
            }
            line_idx += 1;
        }

        if !self.completion_active {
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
            "Ctrl+U  Paste line",
            "Ctrl+G  Toggle help",
            "F2      Recent files",
            "Ctrl+A  Autosave toggle",
            "Ctrl+E  Read-only toggle",
            "Ctrl+M  Toggle theme",
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

    fn render_line(&self, out: &mut io::Stdout, line: &str, cols: usize) -> io::Result<()> {
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
            return self.render_inline_with_keywords(out, line, cols);
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
        let mut line = String::new();
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
        let mut out = Vec::new();
        for (idx, line) in self.lines.iter().enumerate() {
            let trimmed = line.trim();
            match self.language {
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
                    }
                }
                _ => {}
            }
        }
        out
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
                Language::Python => "# ",
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

    fn run_steel(&mut self) -> io::Result<()> {
        let root = find_workspace_root(&self.file).unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        self.status = "Running steel run...".to_string();
        let mut out = stdout();
        self.render(&mut out)?;
        let output = Command::new("steel")
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
        if self.language != Language::Steelconf {
            self.clear_completion();
            return;
        }
        let (prefix, start) = self.current_prefix();
        if prefix.len() < 2 {
            self.clear_completion();
            return;
        }
        let items = self.collect_completion_items(&prefix);
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

    fn render_inline_with_keywords(&self, out: &mut io::Stdout, line: &str, cols: usize) -> io::Result<()> {
        let mut count = 0usize;
        let mut chars = line.chars().peekable();
        let comment_start = match self.language {
            Language::Python => "#",
            _ => "//",
        };

        while let Some(ch) = chars.next() {
            if count >= cols {
                break;
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

            if comment_start == "#" && ch == '#' {
                let mut comment = String::from("#");
                for n in chars {
                    comment.push(n);
                }
                queue!(out, SetForegroundColor(self.colors.comment), Print(comment), ResetColor)?;
                return Ok(());
            }
            if comment_start == "//" && ch == '/' && chars.peek() == Some(&'/') {
                chars.next();
                let mut comment = String::from("//");
                for n in chars {
                    comment.push(n);
                }
                queue!(out, SetForegroundColor(self.colors.comment), Print(comment), ResetColor)?;
                return Ok(());
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
                if is_keyword(self.language, &token) {
                    queue!(out, SetForegroundColor(self.colors.keyword), Print(token), ResetColor)?;
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
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
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
    }
    match file.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "c" | "h" => Language::C,
        "cc" | "cpp" | "cxx" | "hpp" | "hh" => Language::Cpp,
        "py" => Language::Python,
        "java" => Language::Java,
        "ml" | "mli" => Language::Ocaml,
        "zig" => Language::Zig,
        "cs" => Language::CSharp,
        _ => Language::Other,
    }
}

fn is_keyword(lang: Language, token: &str) -> bool {
    match lang {
        Language::C => C_KEYWORDS.contains(&token),
        Language::Cpp => CPP_KEYWORDS.contains(&token),
        Language::Python => PY_KEYWORDS.contains(&token),
        Language::Java => JAVA_KEYWORDS.contains(&token),
        Language::Ocaml => OCAML_KEYWORDS.contains(&token),
        Language::Zig => ZIG_KEYWORDS.contains(&token),
        Language::CSharp => CSHARP_KEYWORDS.contains(&token),
        _ => false,
    }
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

fn extract_func_name(line: &str) -> Option<String> {
    let before = line.split('(').next().unwrap_or("").trim();
    let name = before.split_whitespace().last().unwrap_or("").trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}
