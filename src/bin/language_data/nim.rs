const KEYWORDS: &[&str] = &["proc", "func", "method", "template", "macro", "type", "var", "let", "const", "if", "elif", "else", "for", "while", "case", "of", "when", "return", "import", "include"];
const BUILTINS: &[&str] = &["echo", "seq", "string", "int", "float", "bool", "Table", "Option", "Result", "len", "add", "newSeq"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
