use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &super::main_func_test_snippets(
    super::super::Language::Go,
    "Go main",
    "package main\n\nimport \"fmt\"\n\nfunc main() {\n  fmt.Println(\"${1:hello}\")\n}",
    "Go function",
    "func ${1:Name}(${2:args}) ${3:error} {\n  ${4:// TODO}\n}",
    "Go test",
    "func Test${1:Name}(t *testing.T) {\n  ${2:// TODO}\n}",
);

pub(super) fn keywords() -> &'static [&'static str] { super::super::GO_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::GO_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
