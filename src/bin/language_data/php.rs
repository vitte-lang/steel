use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::Php, trigger: "func", label: "PHP function", body: "<?php\nfunction ${1:name}(${2:$arg}) {\n  ${3:// TODO}\n}\n" },
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::PHP_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::PHP_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
