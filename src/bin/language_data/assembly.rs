const KEYWORDS: &[&str] = &["section", "global", "extern", "db", "dw", "dd", "dq", "equ", "org", "times", "macro", "endm", "proc", "endp", "if", "else", "endif", "include", "align", "bits"];
const BUILTINS: &[&str] = &["mov", "lea", "add", "sub", "mul", "div", "cmp", "jmp", "call", "ret", "push", "pop"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
