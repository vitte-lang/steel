const KEYWORDS: &[&str] = &["function", "if", "elseif", "else", "for", "while", "switch", "case", "otherwise", "break", "continue", "return", "try", "catch", "end", "classdef", "properties", "methods", "events", "global"];
const BUILTINS: &[&str] = &["disp", "plot", "zeros", "ones", "size", "length", "numel", "mean", "sum", "sqrt", "rand", "linspace"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
