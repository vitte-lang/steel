const KEYWORDS: &[&str] = &["def", "defp", "defmodule", "defprotocol", "defimpl", "if", "else", "case", "cond", "fn", "do", "end", "when", "for", "with", "receive", "after", "try", "rescue", "catch"];
const BUILTINS: &[&str] = &["IO", "String", "Enum", "List", "Map", "Kernel", "Agent", "Task", "GenServer", "spawn", "self", "send"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
