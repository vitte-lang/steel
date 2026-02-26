const KEYWORDS: &[&str] = &["def", "defn", "fn", "let", "if", "do", "loop", "recur", "case", "cond", "when", "for", "doseq", "ns", "require", "import", "try", "catch", "throw", "new"];
const BUILTINS: &[&str] = &["map", "filter", "reduce", "assoc", "dissoc", "conj", "println", "str", "keyword", "symbol", "atom", "swap!"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
