const KEYWORDS: &[&str] = &["-module", "-export", "-import", "-record", "-define", "fun", "case", "of", "end", "if", "receive", "after", "try", "catch", "when", "begin", "let", "query", "maybe", "else"];
const BUILTINS: &[&str] = &["io", "lists", "maps", "ets", "gen_server", "supervisor", "spawn", "self", "send", "erlang", "binary", "proplists"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
