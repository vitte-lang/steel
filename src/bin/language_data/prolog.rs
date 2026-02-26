const KEYWORDS: &[&str] = &[":-", "?-", "is", "not", "fail", "true", "repeat", "once", "if", "then", "else", "module", "use_module", "dynamic", "multifile", "discontiguous", "op", "meta_predicate", "initialization", "consult"];
const BUILTINS: &[&str] = &["write", "writeln", "read", "assertz", "retract", "findall", "bagof", "setof", "member", "append", "length", "maplist"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
