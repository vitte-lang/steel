const KEYWORDS: &[&str] = &["if", "else", "for", "while", "repeat", "function", "return", "next", "break", "in", "TRUE", "FALSE", "NULL", "NA", "Inf", "NaN", "library", "require", "source", "local"];
const BUILTINS: &[&str] = &["print", "cat", "data.frame", "list", "matrix", "factor", "apply", "lapply", "sapply", "tapply", "mean", "sum"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
