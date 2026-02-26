const KEYWORDS: &[&str] = &["module", "using", "import", "export", "function", "macro", "struct", "mutable", "if", "else", "elseif", "for", "while", "return", "let", "begin", "end", "quote", "try", "catch"];
const BUILTINS: &[&str] = &["println", "print", "Vector", "Dict", "Set", "String", "Int", "Float64", "Bool", "Nothing", "Union", "Array"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
