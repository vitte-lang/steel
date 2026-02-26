const KEYWORDS: &[&str] = &["define", "lambda", "let", "let*", "letrec", "if", "cond", "case", "begin", "set!", "and", "or", "delay", "quote", "quasiquote", "unquote", "syntax-rules", "call/cc", "do", "else"];
const BUILTINS: &[&str] = &["car", "cdr", "cons", "list", "map", "filter", "foldl", "foldr", "display", "newline", "apply", "eval"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
