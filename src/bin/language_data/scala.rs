const KEYWORDS: &[&str] = &["class", "object", "trait", "extends", "with", "def", "val", "var", "if", "else", "match", "case", "for", "while", "return", "import", "package", "given", "using", "enum"];
const BUILTINS: &[&str] = &["String", "Int", "Long", "Double", "Boolean", "List", "Map", "Set", "Option", "Either", "println", "Future"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
