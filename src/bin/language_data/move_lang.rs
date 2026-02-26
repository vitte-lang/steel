const KEYWORDS: &[&str] = &["module", "script", "struct", "fun", "public", "entry", "const", "use", "if", "else", "while", "loop", "break", "continue", "return", "acquires", "spec", "aborts_if", "ensures", "let"];
const BUILTINS: &[&str] = &["vector", "u8", "u16", "u32", "u64", "u128", "u256", "bool", "address", "signer", "assert", "borrow_global"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
