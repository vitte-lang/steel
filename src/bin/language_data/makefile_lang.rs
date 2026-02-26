use super::super::LangSnippet;

const KEYWORDS: &[&str] = &["include", "ifdef", "ifndef", "ifeq", "ifneq", "else", "endif", "define", "endef", "override", "export", "unexport", "vpath", "private", "sinclude", "-include", "MAKE", "MAKEFLAGS", "PHONY", "SUFFIXES"];
const BUILTINS: &[&str] = &["$(CC)", "$(CXX)", "$(AR)", "$(RM)", "$(MAKE)", "$(shell)", "$(wildcard)", "$(patsubst)", "$(subst)", "$(filter)", "$(sort)", "$(strip)"];
const SNIPPETS: &[LangSnippet] = &super::main_func_test_snippets(
    super::super::Language::Makefile,
    "Makefile all target",
    ".PHONY: all\nall: ${1:build}",
    "Make variable",
    "${1:NAME} := ${2:value}",
    "Make test target",
    ".PHONY: test\ntest:\n\t${1:echo \"ok\"}",
);
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
