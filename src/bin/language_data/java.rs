use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &super::main_func_test_snippets(
    super::super::Language::Java,
    "Java main",
    "public class ${1:Main} {\n  public static void main(String[] args) {\n    System.out.println(\"${2:hello}\");\n  }\n}",
    "Java method",
    "public ${1:void} ${2:name}(${3:args}) {\n  ${4:// TODO}\n}",
    "Java test",
    "@Test\nvoid ${1:name}() {\n  ${2:// TODO}\n}",
);

pub(super) fn keywords() -> &'static [&'static str] { super::super::JAVA_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::JAVA_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
