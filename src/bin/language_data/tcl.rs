const KEYWORDS: &[&str] = &["proc", "if", "else", "elseif", "for", "foreach", "while", "switch", "return", "break", "continue", "namespace", "package", "source", "set", "unset", "global", "variable", "uplevel", "upvar"];
const BUILTINS: &[&str] = &["puts", "expr", "list", "dict", "array", "string", "regexp", "regsub", "format", "scan", "clock", "file"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
