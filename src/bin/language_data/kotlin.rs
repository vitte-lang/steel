use super::super::LangSnippet;

const KEYWORDS: &[&str] = &["class", "object", "interface", "fun", "val", "var", "if", "else", "when", "for", "while", "return", "package", "import", "sealed", "data", "enum", "companion", "suspend", "inline"];
const BUILTINS: &[&str] = &["String", "Int", "Long", "Double", "Float", "Boolean", "List", "Map", "Set", "println", "Result", "Any", "Unit"];
const SNIPPETS: &[LangSnippet] = &super::main_func_test_snippets(
    super::super::Language::Kotlin,
    "Kotlin main",
    "fun main() {\n  println(\"${1:hello}\")\n}",
    "Kotlin function",
    "fun ${1:name}(${2:arg}: ${3:String}): ${4:Unit} {\n  ${5:// TODO}\n}",
    "Kotlin test",
    "@Test\nfun ${1:name}() {\n  assertEquals(${2:expected}, ${3:actual})\n}",
);
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
