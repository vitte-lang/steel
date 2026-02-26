const KEYWORDS: &[&str] = &["module", "namespace", "open", "let", "mutable", "type", "member", "match", "with", "function", "if", "then", "else", "for", "while", "yield", "return", "async", "interface", "inherit"];
const BUILTINS: &[&str] = &["printfn", "List", "Seq", "Array", "Map", "Set", "Option", "Result", "string", "int", "float", "bool"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
