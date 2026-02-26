use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::Haskell, trigger: "main", label: "Haskell main", body: "main :: IO ()\nmain = putStrLn \"${1:hello}\"" },
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::HASKELL_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::HASKELL_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
