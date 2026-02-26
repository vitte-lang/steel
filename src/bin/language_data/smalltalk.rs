const KEYWORDS: &[&str] = &["self", "super", "nil", "true", "false", "thisContext", "class", "subclass", "method", "category", "poolDictionaries", "instanceVariableNames", "classVariableNames", "ifTrue", "ifFalse", "whileTrue", "whileFalse", "to", "by", "do"];
const BUILTINS: &[&str] = &["Transcript", "Object", "String", "Array", "Dictionary", "Set", "OrderedCollection", "Integer", "Float", "Character", "Date", "Time"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
