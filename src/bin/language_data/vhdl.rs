const KEYWORDS: &[&str] = &["entity", "architecture", "is", "begin", "end", "signal", "process", "if", "then", "else", "elsif", "case", "when", "for", "loop", "port", "map", "generic", "library", "use"];
const BUILTINS: &[&str] = &["std_logic", "std_logic_vector", "unsigned", "signed", "integer", "boolean", "rising_edge", "falling_edge", "to_integer", "to_unsigned", "resize", "now"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
