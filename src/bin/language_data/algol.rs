use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::Algol, trigger: "begin", label: "Algol block", body: "begin\n  ${1:comment};\nend" },
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::ALGOL_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::ALGOL_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
