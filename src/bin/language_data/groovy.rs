const KEYWORDS: &[&str] = &["class", "interface", "enum", "trait", "def", "if", "else", "switch", "case", "for", "while", "return", "import", "package", "extends", "implements", "try", "catch", "finally", "new"];
const BUILTINS: &[&str] = &["String", "Integer", "BigDecimal", "List", "Map", "Set", "println", "print", "File", "Date", "Math", "System"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
