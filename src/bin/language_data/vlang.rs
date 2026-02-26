const KEYWORDS: &[&str] = &["module", "import", "fn", "struct", "interface", "enum", "type", "const", "mut", "if", "else", "match", "for", "return", "or", "defer", "go", "select", "unsafe", "asm"];
const BUILTINS: &[&str] = &["println", "print", "string", "int", "f64", "bool", "array", "map", "Option", "Result", "panic", "error"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
