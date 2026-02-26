const KEYWORDS: &[&str] = &["class", "module", "struct", "enum", "def", "if", "else", "elsif", "case", "when", "for", "while", "return", "require", "include", "extend", "macro", "lib", "fun", "alias"];
const BUILTINS: &[&str] = &["String", "Int32", "Int64", "Float64", "Bool", "Array", "Hash", "Set", "puts", "print", "pp", "Math"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
