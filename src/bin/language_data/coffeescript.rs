use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::CoffeeScript, trigger: "func", label: "Coffee function", body: "${1:name} = (${2:args}) ->\n  ${3:# TODO}" },
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::COFFEE_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::COFFEE_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
