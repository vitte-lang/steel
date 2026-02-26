const KEYWORDS: &[&str] = &["function", "if", "else", "for", "while", "switch", "case", "break", "continue", "return", "set", "set_color", "begin", "end", "and", "or", "not", "in", "command", "builtin"];
const BUILTINS: &[&str] = &["echo", "printf", "read", "string", "math", "count", "contains", "source", "status", "set", "abbr", "alias"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
