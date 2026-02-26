use super::super::LangSnippet;

const BUILTINS: &[&str] = &[
    "println!",
    "format!",
    "vec!",
    "println",
    "format",
    "vec",
    "Option",
    "Result",
    "String",
    "Vec",
];
const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::Rust, trigger: "main", label: "Rust main", body: "fn main() {\n  ${1:// TODO}\n}" },
    LangSnippet { lang: super::super::Language::Rust, trigger: "func", label: "Rust function", body: "fn ${1:name}(${2:args}) -> ${3:()} {\n  ${4:// TODO}\n}" },
    LangSnippet { lang: super::super::Language::Rust, trigger: "test", label: "Rust #[test]", body: "#[test]\nfn ${1:name}() {\n  assert_eq!(${2:left}, ${3:right});\n}" },
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::RUST_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
