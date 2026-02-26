const KEYWORDS: &[&str] = &["procedure", "function", "package", "is", "begin", "end", "if", "then", "else", "loop", "for", "while", "case", "when", "type", "record", "with", "use", "return", "declare"];
const BUILTINS: &[&str] = &["Ada.Text_IO", "Put_Line", "Put", "Get", "Integer", "Float", "Boolean", "String", "Character", "Natural", "Positive", "Duration"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
