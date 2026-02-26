const KEYWORDS: &[&str] = &["program", "module", "subroutine", "function", "implicit", "none", "integer", "real", "logical", "character", "if", "then", "else", "do", "end", "select", "case", "contains", "use", "call"];
const BUILTINS: &[&str] = &["print", "write", "read", "allocate", "deallocate", "size", "shape", "matmul", "sum", "maxval", "minval", "sqrt"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
